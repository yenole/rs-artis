use rbs::Value;
use serde::Serialize;

use crate::{into_where::IntoWhere, raw};

pub trait IntoUpdate {
    fn into_update(self) -> (&'static str, Vec<Value>);
}

impl<T, W> IntoUpdate for (&'static str, W, T)
where
    W: IntoWhere,
    T: Serialize,
{
    fn into_update(self) -> (&'static str, Vec<Value>) {
        let (table, w, v) = self;
        if let rbs::Value::Map(mut dict) = rbs::to_value!(v) {
            let (mut whe, mut args) = w.into_where();
            if args.is_empty() && !whe.is_empty() {
                let inw_v = dict.remove(&rbs::to_value!(whe));
                if !inw_v.is_null() {
                    whe = raw!("{} = ?", whe);
                    args.push(inw_v);
                }
            }
            let column: Vec<_> = dict
                .0
                .keys()
                .map(|v| format!("{} = ?", v.as_str().unwrap()))
                .collect();
            let raw = raw!("UPDATE {} SET {}", table, column.join(", "));
            let raw = raw!("{} WHERE {}", raw, whe);
            let mut values: Vec<rbs::Value> = dict.0.values().map(|v| v.to_owned()).collect();
            values.extend(args);
            return (raw, values);
        }
        todo!()
    }
}
