use rbs::Value;

pub trait IntoWhere {
    fn into_where(self) -> (&'static str, Vec<Value>);
}

impl IntoWhere for &'static str {
    fn into_where(self) -> (&'static str, Vec<Value>) {
        (self, vec![])
    }
}

impl IntoWhere for rbs::value::map::ValueMap {
    fn into_where(self) -> (&'static str, Vec<Value>) {
        let column: Vec<String> = self
            .0
            .keys()
            .map(|v| format!("{} = ?", v.as_str().unwrap()))
            .collect();
        let args: Vec<_> = self.0.values().map(|v| v.to_owned()).collect();
        let raw = column.join("AND");
        return (Box::leak(raw.into_boxed_str()), args);
    }
}
impl IntoWhere for rbs::Value {
    fn into_where(self) -> (&'static str, Vec<Value>) {
        if let rbs::Value::Map(dict) = self {
            return dict.into_where();
        }
        todo!()
    }
}

impl IntoWhere for (&'static str, Vec<Value>) {
    fn into_where(self) -> (&'static str, Vec<Value>) {
        self
    }
}
