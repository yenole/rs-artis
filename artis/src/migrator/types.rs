use std::{collections::HashMap, fmt::Display};

use crate::raw;

pub type Mapping = HashMap<&'static str, &'static str>;

#[derive(Debug, Clone, PartialEq, Eq)]
// 考虑多字段索引
pub enum IndexMeta {
    Index(String),
    Unique(String),
}

impl IndexMeta {
    pub fn name(&self, t: &str) -> String {
        match self {
            IndexMeta::Index(v) => raw!("idx_{}_{}", t, v),
            IndexMeta::Unique(v) => raw!("unq_{}_{}", t, v),
        }
    }

    pub fn column(&self) -> String {
        let raw = match self {
            IndexMeta::Index(v) => v,
            IndexMeta::Unique(v) => v,
        };
        raw.into()
    }
}

#[derive(Debug, Clone)]
pub enum Adjust {
    Add,
    Drop,
    Alter,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ColumeMeta {
    pub name: String,    // 字段
    pub colume: String,  // 类型
    pub nullable: bool,  // 是否为空
    pub default: String, // 默认值
    pub comment: String, // 注释
}

impl Display for ColumeMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.name, self.colume)?;
        if !self.nullable {
            write!(f, " {}", "NOT NULL")?;
        }
        if !self.default.is_empty() {
            write!(f, " DEFAULT {}", self.default)?;
        }
        Ok(())
    }
}

impl Into<ColumeMeta> for String {
    fn into(self) -> ColumeMeta {
        let mut meta = ColumeMeta::default();
        let mut itr = self.split_whitespace();
        meta.name = itr.next().unwrap().into();
        meta.colume = itr.next().unwrap().into();
        meta.nullable = true;
        while let Some(v) = itr.next() {
            match v {
                "NOT" => {
                    meta.nullable = false;
                    itr.next();
                }
                "DEFAULT" => {
                    meta.default = itr.next().unwrap().into();
                }
                "COMMENT" => {
                    meta.comment = itr.next().unwrap().into();
                }
                _ => {}
            };
        }
        meta
    }
}

#[derive(Debug, Clone, Default)]
pub struct TableMeta {
    pub name: String,             // 表名
    pub primary: String,          // 主键字段  考虑复合主键
    pub indexs: Vec<IndexMeta>,   // 索引
    pub columes: Vec<ColumeMeta>, // 字段
}

impl TableMeta {
    pub fn into_raw(&self) -> String {
        let columes: Vec<_> = self.columes.iter().map(|v| v.to_string()).collect();
        let mut raw = raw!("CREATE TABLE {} ({}", self.name, columes.join(","));
        if !self.primary.is_empty() {
            raw.push_str(&raw!(", PRIMARY KEY ({}))", self.primary));
        }
        raw
    }
}