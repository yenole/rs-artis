use rbs::Value;

use crate::{into_where::IntoWhere, raw};

pub type ArColume = Vec<&'static str>;

pub trait IntoSelect {
    fn into_select(self) -> (&'static str, Vec<Value>);
}

impl IntoSelect for &'static str {
    fn into_select(self) -> (&'static str, Vec<Value>) {
        (raw!("SELECT * FROM {}", self), vec![])
    }
}

impl IntoSelect for (&'static str, ArColume) {
    fn into_select(self) -> (&'static str, Vec<Value>) {
        let (table, colume) = self;
        (table, "", colume).into_select()
    }
}

impl<T> IntoSelect for (&'static str, T)
where
    T: IntoWhere,
{
    fn into_select(self) -> (&'static str, Vec<Value>) {
        let (table, v) = self;
        let colume: Vec<&'static str> = vec![];
        (table, v, colume).into_select()
    }
}

impl<T> IntoSelect for (&'static str, T, ArColume)
where
    T: IntoWhere,
{
    fn into_select(self) -> (&'static str, Vec<Value>) {
        let (table, v, colume) = self;
        let mut raw = if colume.is_empty() {
            raw!("SELECT * FROM {}", table)
        } else {
            raw!("SELECT {} FROM {}", colume.join(", "), table)
        };
        let (whe, args) = v.into_where();
        if !whe.is_empty() {
            raw = raw!("{} WHERE {}", raw, whe);
        }
        (raw, args)
    }
}
