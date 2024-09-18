use std::iter::repeat;

use rbs::Value;
use serde::Serialize;

use crate::raw;

pub trait IntoSaving {
    fn into_saving(self) -> (&'static str, Vec<Value>);
}

impl<T> IntoSaving for (&'static str, T)
where
    T: Serialize,
{
    fn into_saving(self) -> (&'static str, Vec<Value>) {
        let (table, value) = self;
        if let rbs::Value::Map(dict) = rbs::to_value!(value) {
            let colume: Vec<_> = dict.0.keys().map(|v| v.as_str().unwrap()).collect();
            let values: Vec<_> = dict.0.values().map(|v| v.to_owned()).collect();
            let mut raw = raw!("INSERT INTO {}({})", table, colume.join(", "));
            let placeholder: Vec<_> = repeat("?").take(colume.len()).collect();
            raw = raw!("{} VALUE({})", raw, placeholder.join(", "));

            (raw, values)
        } else {
            ("", vec![])
        }
    }
}
