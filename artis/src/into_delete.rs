use rbs::Value;

use crate::{into_where::IntoWhere, raw};

pub trait IntoDelete {
    fn into_raw(self) -> (&'static str, Vec<Value>);
}

impl<W> IntoDelete for (&'static str, W)
where
    W: IntoWhere,
{
    fn into_raw(self) -> (&'static str, Vec<Value>) {
        let (table, w) = self;
        let (whe, args) = w.into_where();
        let raw = raw!("DELETE FROM {} WHERE {}", table, whe);
        return (raw, args);
    }
}
