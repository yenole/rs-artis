use crate::{
    raw,
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

    pub fn select(mut self, v: Vec<&'static str>) -> Self {
        self.inner[1] = Props::Select(v.iter().map(|v| v.to_string()).collect());
        self
    }

    pub fn where_(mut self, v: &'static str, args: Vec<crate::Value>) -> Self {
        self.inner[2] = Props::Where((v.into(), args));
        self
    }

    pub fn group(mut self, v: &'static str) -> Self {
        self.inner[3] = Props::Group(v.into());
        self
    }

    pub fn order(mut self, v: &'static str) -> Self {
        self.inner[4] = Props::Order(v.into());
        self
    }

    pub fn limit(mut self, v: u32) -> Self {
        if let Props::Limit((_, o)) = self.inner[4] {
            self.inner[5] = Props::Limit((v, o));
        } else {
            self.inner[5] = Props::Limit((v, 0));
        }
        self
    }

    pub fn offset(mut self, v: u32) -> Self {
        if let Props::Limit((l, _)) = self.inner[5] {
            self.inner[5] = Props::Limit((l, v));
        } else {
            self.inner[5] = Props::Limit((0, v));
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
        Raw::table(self).into_raw(v)
    }
}

impl IntoRaw for (&'static str, &'static str) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0).order(self.1).into_raw(v)
    }
}

impl IntoRaw for (&'static str, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0).limit(self.1 as u32).into_raw(v)
    }
}

impl IntoRaw for (&'static str, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .limit(self.1 .0 as u32)
            .offset(self.1 .1 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, &'static str, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .order(self.1)
            .limit(self.2 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, &'static str, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .order(self.1)
            .limit(self.2 .0 as u32)
            .offset(self.2 .1 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Vec<&'static str>) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0).select(self.1.clone()).into_raw(v)
    }
}

impl IntoRaw for (&'static str, Vec<&'static str>, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .select(self.1.clone())
            .limit(self.2 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Vec<&'static str>, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .select(self.1.clone())
            .limit(self.2 .0 as u32)
            .offset(self.2 .1 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Vec<&'static str>, &'static str, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .select(self.1.clone())
            .order(self.2)
            .limit(self.3 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Vec<&'static str>, &'static str, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .select(self.1.clone())
            .order(self.2)
            .limit(self.3 .0 as u32)
            .offset(self.3 .1 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0).model(self.1.clone()).into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value, &'static str) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(self.1.clone())
            .order(self.2)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(self.1.clone())
            .limit(self.2 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(self.1.clone())
            .limit(self.2 .0 as u32)
            .offset(self.2 .0 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value, &'static str, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(self.1.clone())
            .order(self.2)
            .limit(self.3 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, Value, &'static str, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .model(self.1.clone())
            .order(self.2)
            .limit(self.3 .0 as u32)
            .offset(self.3 .0 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args), &'static str) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .order(self.2)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args), i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .limit(self.2 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args), (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .limit(self.2 .0 as u32)
            .offset(self.2 .1 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args), &'static str, i32) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .order(self.2)
            .limit(self.3 as u32)
            .into_raw(v)
    }
}

impl IntoRaw for (&'static str, (&'static str, Args), &'static str, (i32, i32)) {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<crate::Value>) {
        Raw::table(self.0)
            .where_(self.1 .0, self.1 .1.clone())
            .order(self.2)
            .limit(self.3 .0 as u32)
            .offset(self.3 .1 as u32)
            .into_raw(v)
    }
}
