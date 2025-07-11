use rbs::value::map::ValueMap;

use crate::{
    raw, rbv,
    types::{Args, Columns, RawType},
    Value,
};

pub trait IntoRaw {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>);
}

pub trait IntoTable {
    fn into_table(&self) -> String;
}

impl IntoTable for &'static str {
    fn into_table(&self) -> String {
        self.to_string()
    }
}

pub trait IntoLimit: Clone {
    fn into_limit(&self) -> (u32, u32);
}

impl IntoLimit for i32 {
    fn into_limit(&self) -> (u32, u32) {
        (*self as u32, 0)
    }
}

impl IntoLimit for (i32, i32) {
    fn into_limit(&self) -> (u32, u32) {
        (self.0 as u32, self.1 as u32)
    }
}

#[derive(Debug, Clone)]
enum Props {
    Empty,
    Model(crate::Value),
    Limit((u32, u32)),
    Where((String, Vec<crate::Value>)),
    Group(String),
    Order(String),
    Select(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct Raw {
    table: String,
    inner: Vec<Props>, // [model,select,where,order,limit]
}

impl Raw {
    pub fn table(v: &str) -> Self {
        Self {
            table: v.into(),
            inner: vec![Props::Empty; 6],
        }
    }

    pub fn model(mut self, v: crate::Value) -> Self {
        self.inner[0] = Props::Model(v);
        self
    }

    pub fn select(mut self, v: Vec<&str>) -> Self {
        self.inner[1] = Props::Select(v.iter().map(|v| v.to_string()).collect());
        self
    }

    pub fn where_(mut self, v: &str, args: Vec<crate::Value>) -> Self {
        self.inner[2] = Props::Where((v.into(), args));
        self
    }

    pub fn group(mut self, v: &str) -> Self {
        self.inner[3] = Props::Group(v.into());
        self
    }

    pub fn order(mut self, v: &str) -> Self {
        self.inner[4] = Props::Order(v.into());
        self
    }

    pub fn limit(mut self, v: impl IntoLimit) -> Self {
        self.inner[5] = Props::Limit(v.into_limit());
        self
    }
}

impl Raw {
    fn extend_map(t: RawType, v: &Value, args: &mut Args, s: &Columns) -> Columns {
        if !v.is_map() {
            return vec![];
        }
        let dict = v.as_map().unwrap();
        let keys = if !s.is_empty() {
            s.clone()
        } else {
            dict.0.keys().map(|v| v.as_string().unwrap()).collect()
        };
        let mut columns = vec![];
        keys.iter().for_each(|k| {
            let key: rbs::Value = k.as_str().into();
            let mut value = Value::Null;
            if dict.0.contains_key(&key) {
                value = dict[k.as_str()].clone()
            }
            if t.is_saving() && value.is_null() {
                return;
            }
            if t.is_saving() {
                columns.push(k.to_owned());
                args.push(value);
            } else if let Value::Array(list) = value {
                if list.len() == 0 || list.len() > 2 || !list[0].is_str() {
                    return;
                }
                columns.push(format!("{} {}", k, list[0].as_str().unwrap_or_default()));
                if list.len() == 2 {
                    args.push(list[1].clone());
                }
            } else {
                columns.push(format!("{} = ?", k));
                args.push(value);
            }
        });
        columns
    }

    fn into_fetch(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        let mut columns = s.join(", ");
        if s.is_empty() {
            columns = "*".into();
        }
        raw.push_str(&raw!("SELECT {} FROM {}", columns, t));
        let keys: Vec<_> = Raw::extend_map(RawType::Fetch, v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" WHERE {}", keys.join(" AND ")));
        }
    }

    fn into_saving(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("INSERT INTO {}", t));
        let keys = Raw::extend_map(RawType::Saving, &v, args, s);
        if !keys.is_empty() {
            let hold: String = vec!["?"; keys.len()].join(", ");
            raw.push_str(&raw!("({}) VALUES ({})", keys.join(", "), hold));
        }
    }

    fn into_update(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("UPDATE {}", t));
        let keys = Raw::extend_map(RawType::Update, v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" SET {}", keys.join(" , ")));
        }
    }

    fn into_delete(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("DELETE FROM {}", t));
        let keys = Raw::extend_map(RawType::Delete, v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" WHERE {}", keys.join(" AND ")));
        }
    }
}

impl IntoRaw for Raw {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        let table = &self.table;
        let mut raw = String::new();
        let mut args: Vec<crate::Value> = vec![];

        let mut model = &crate::Value::Null;
        if let Props::Model(v) = &self.inner[0] {
            model = v;
        }
        let mut columns: Columns = vec![];
        if let Props::Select(s) = &self.inner[1] {
            columns = s.clone()
        }
        match &v {
            RawType::Fetch => Raw::into_fetch(&mut raw, &mut args, table, &columns, model),
            RawType::Saving => Raw::into_saving(&mut raw, &mut args, table, &columns, model),
            RawType::Update => Raw::into_update(&mut raw, &mut args, table, &columns, model),
            RawType::Delete => Raw::into_delete(&mut raw, &mut args, table, &columns, model),
        };

        self.inner.iter().for_each(|p| match p {
            Props::Where((s, list)) => {
                if !v.is_saving() {
                    if v.is_single_prop() && !model.is_null() && !s.is_empty() && list.is_empty() {
                        if let crate::Value::Map(dict) = model {
                            let key = Value::String(s.into());
                            if let Some(v) = dict.0.get(&key) {
                                raw.push_str(&raw!(" WHERE {} = ?", s));
                                args.push(v.clone());
                            }
                        }
                    } else {
                        raw.push_str(&raw!(" WHERE {}", s));
                        list.iter().for_each(|v| args.push(v.clone()));
                    }
                }
            }
            Props::Group(c) => {
                if v.is_fetch() {
                    raw.push_str(&raw!(" GROUP BY {}", c));
                }
            }
            Props::Order(c) => {
                if v.is_fetch() {
                    raw.push_str(&raw!(" ORDER BY {}", c));
                }
            }
            Props::Limit((l, o)) => {
                if v.is_fetch() {
                    if *o == 0 {
                        raw.push_str(&raw!(" LIMIT {}", l));
                    } else {
                        raw.push_str(&raw!(" LIMIT {},{}", l, o));
                    }
                }
            }
            _ => {}
        });
        (Box::leak(raw.into_boxed_str()), args)
    }
}

impl IntoRaw for String {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() && !v.is_delete() {
            panic!("Not supported")
        }
        Raw::table(self).into_raw(v)
    }
}

impl<T> IntoRaw for (T, &str)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table()).order(self.1).into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .limit(self.1.clone())
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, &str, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .order(self.1)
            .limit(self.2.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Vec<&str>)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Vec<&str>, Value)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if v.is_update() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .model(self.2.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Vec<&str>, (&str, Vec<Value>))
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if v.is_update() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .where_(self.2 .0, self.2 .1.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Vec<&str>, Value, &str)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        match v {
            RawType::Fetch => Raw::table(&self.0.into_table())
                .select(self.1.clone())
                .model(self.2.clone())
                .order(self.3)
                .into_raw(v),
            RawType::Update => Raw::table(&self.0.into_table())
                .select(self.1.clone())
                .model(self.2.clone())
                .where_(self.3, vec![])
                .into_raw(v),
            _ => panic!("Not supported"),
        }
    }
}

impl<T> IntoRaw for (T, Vec<&str>, &str)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .order(self.2)
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, Vec<&str>, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .limit(self.2.clone())
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, Vec<&str>, &str, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .select(self.1.clone())
            .order(self.2)
            .limit(self.3.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Value)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if v.is_update() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .model(self.1.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, Value, &str)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if v.is_fetch() {
            return Raw::table(&self.0.into_table())
                .model(self.1.clone())
                .order(self.2)
                .into_raw(v);
        } else if v.is_update() {
            if self.2.is_empty() || !self.1.is_map() {
                panic!("Not supported")
            }
            let (map, args) = split_model(self.1.clone(), &vec![self.2]);
            let raw = format!("{} = ?", self.2);
            return Raw::table(&self.0.into_table())
                .model(Value::Map(map))
                .where_(&raw, args)
                .into_raw(v);
        }
        panic!("Not supported")
    }
}

impl<T> IntoRaw for (T, Value, Vec<&str>)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if v.is_update() {
            if self.2.is_empty() || !self.1.is_map() {
                panic!("Not supported")
            }
            let (map, args) = split_model(self.1.clone(), &self.2);
            let raw = format!("{} = ?", self.2.join(" = ? AND "));
            return Raw::table(&self.0.into_table())
                .model(Value::Map(map))
                .where_(&raw, args)
                .into_raw(v);
        }
        panic!("Not supported")
    }
}

impl<T, L> IntoRaw for (T, Value, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .model(self.1.clone())
            .limit(self.2.clone())
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, Value, &str, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .model(self.1.clone())
            .order(self.2)
            .limit(self.3.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, (&str, Args))
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() && !v.is_delete() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .where_(self.1 .0, self.1 .1.clone())
            .into_raw(v)
    }
}

impl<T> IntoRaw for (T, (&str, Args), &str)
where
    T: IntoTable,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .where_(self.1 .0, self.1 .1.clone())
            .order(self.2)
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, (&str, Args), L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .where_(self.1 .0, self.1 .1.clone())
            .limit(self.2.clone())
            .into_raw(v)
    }
}

impl<T, L> IntoRaw for (T, (&str, Args), &'static str, L)
where
    T: IntoTable,
    L: IntoLimit,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        if !v.is_fetch() {
            panic!("Not supported")
        }
        Raw::table(&self.0.into_table())
            .where_(self.1 .0, self.1 .1.clone())
            .order(self.2)
            .limit(self.3.clone())
            .into_raw(v)
    }
}

fn split_model(v: Value, column: &Vec<&str>) -> (ValueMap, Vec<Value>) {
    if !v.is_map() {
        panic!("Not supported");
    }
    let mut map = v.into_map().expect("Not supported");
    let mut args: Vec<Value> = vec![];
    for v in column {
        let key = rbv!(v);
        if !map.0.contains_key(&key) {
            args.push(Value::Null);
            continue;
        }
        args.push(map.0.get(&key).unwrap().clone());
        map.remove(&key);
    }
    return (map, args);
}
