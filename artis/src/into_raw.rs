use serde::Serialize;

use crate::{
    raw, rbv,
    types::{Args, Columns, RawType},
    Value,
};

pub trait IntoRaw {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>);
}

#[derive(Debug, Clone)]
enum Props {
    Empty,
    Model(crate::Value),
    Limit((u32, u32)),
    Where((String, Vec<crate::Value>)),
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

    pub fn where_(mut self, v: &'static str, args: Vec<crate::Value>) -> Self {
        self.inner[2] = Props::Where((v.into(), args));
        self
    }

    pub fn select(mut self, v: Vec<&'static str>) -> Self {
        self.inner[1] = Props::Select(v.iter().map(|v| v.to_string()).collect());
        self
    }

    pub fn order(mut self, v: &'static str) -> Self {
        self.inner[3] = Props::Order(v.into());
        self
    }

    pub fn limit(mut self, v: u32) -> Self {
        if let Props::Limit((_, o)) = self.inner[3] {
            self.inner[4] = Props::Limit((v, o));
        } else {
            self.inner[4] = Props::Limit((v, 0));
        }
        self
    }

    pub fn offset(mut self, v: u32) -> Self {
        if let Props::Limit((l, _)) = self.inner[3] {
            self.inner[4] = Props::Limit((l, v));
        } else {
            self.inner[4] = Props::Limit((0, v));
        }
        self
    }
}

impl Raw {
    fn extend_map(v: &Value, args: &mut Args, s: &Columns) -> Columns {
        if !v.is_map() {
            return vec![];
        }
        let dict = v.as_map().unwrap();
        let keys = if !s.is_empty() {
            s.clone()
        } else {
            dict.0.keys().map(|v| v.as_string().unwrap()).collect()
        };

        keys.iter().for_each(|k| {
            let key: rbs::Value = k.as_str().into();
            if dict.0.contains_key(&key) {
                args.push(dict[k.as_str()].clone());
            } else {
                args.push(Value::Null);
            }
        });
        keys
    }

    fn into_fetch(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        let mut columns = s.join(", ");
        if s.is_empty() {
            columns = "*".into();
        }
        raw.push_str(&raw!("SELECT {} FROM {}", columns, t));
        let keys: Vec<_> = Raw::extend_map(v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" WHERE {} = ?", keys.join(" = ? AND ")));
        }
    }

    fn into_saving(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("INSERT INTO {}", t));
        let keys = Raw::extend_map(&v, args, s);
        if !keys.is_empty() {
            let hold: String = vec!["?"; keys.len()].join(", ");
            raw.push_str(&raw!("({}) VALUES ({})", keys.join(", "), hold));
        }
    }

    fn into_update(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("UPDATE {}", t));
        let keys = Raw::extend_map(v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" SET {} = ?", keys.join("= ? , ")));
        }
    }

    fn into_delete(raw: &mut String, args: &mut Vec<Value>, t: &str, s: &Columns, v: &Value) {
        raw.push_str(&raw!("DELETE FROM {}", t));
        let keys = Raw::extend_map(v, args, s);
        if !keys.is_empty() {
            raw.push_str(&raw!(" WHERE {} = ?", keys.join("= ? AND ")));
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
            Props::Order(o) => {
                if v.is_fetch() {
                    raw.push_str(&raw!(" ORDER BY {}", o));
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
        Raw::table(self.as_str()).into_raw(v)
    }
}

impl<T> IntoRaw for (&'static str, T)
where
    T: Serialize,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0).model(rbv! {&self.1,}).into_raw(v)
    }
}

impl<T> IntoRaw for (&'static str, T, &'static str)
where
    T: Serialize,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(rbv! {&self.1,})
            .where_(self.2, vec![])
            .into_raw(v)
    }
}

impl<T> IntoRaw for (&'static str, T, Vec<&'static str>)
where
    T: Serialize,
{
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(rbv! {&self.1,})
            .select(self.2.clone())
            .into_raw(v)
    }
}
