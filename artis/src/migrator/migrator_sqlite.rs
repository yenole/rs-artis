use serde::Deserialize;

use crate::{
    map,
    migrator::{ColumeMeta, IndexMeta},
    raw, Artis, BoxFuture, IntoArtis, Result,
};

use super::{migrator::DriverMigrator, types::Adjust, TableMeta};

const MASTER: &'static str = "sqlite_master";

#[derive(Debug)]
pub struct SqliteMigrator {}
impl SqliteMigrator {
    fn mapping() -> super::Mapping {
        map! {
            "i32" : "INTEGER",
            "i64" : "INTEGER",
            "u32" : "INTEGER",
            "u64" : "INTEGER",
            "f32" : "DOUBLE",
            "f64" : "DOUBLE",
            "Vec" : "BLOB",
            "Map" : "BLOB",
            "bool" : "BOOLEAN",
            "String" : "TEXT",
        }
    }
}

#[derive(Debug, Deserialize)]
struct SqliteTable {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub sql: String,
}

#[inline]
fn extract_range(str: &str, p: (&str, &str)) -> String {
    let r_idx = str.find(p.0).unwrap();
    let l_idx = str.rfind(p.1).unwrap();
    str[r_idx + p.0.len()..l_idx].trim().into()
}

impl Into<TableMeta> for &SqliteTable {
    fn into(self) -> TableMeta {
        let mut meta = TableMeta::default();
        meta.name = self.name.clone();
        let raw = extract_range(&self.sql, ("(", ")"));
        let columes: Vec<_> = raw.split(",").collect();
        columes.iter().for_each(|v| {
            if v.trim().starts_with("PRIMARY KEY") {
                meta.primary = extract_range(v, ("(", ")"));
            } else {
                meta.columes.push(v.to_string().into());
            }
        });
        meta
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

impl<'a> DriverMigrator<'a> for SqliteMigrator {
    fn mapping(&self, meta: &mut TableMeta) {
        let dict = SqliteMigrator::mapping();
        meta.columes.iter_mut().for_each(|v: &mut ColumeMeta| {
            if !v.colume.starts_with(":") {
                return;
            }
            let key = v.colume[1..].trim();
            if !dict.contains_key(&key) {
                panic!("mapping not found: {}", key);
            }
            v.colume = dict[key].into();
        });
    }

    fn fetch_tables(&self, rb: &'a Artis) -> BoxFuture<'a, Result<Vec<TableMeta>>> {
        Box::pin(async move {
            let mut metas: Vec<TableMeta> = vec![];
            let raw = (MASTER, ("sql NOT NULL", vec![]), "`type` DESC");
            let list: Vec<SqliteTable> = rb.fetch(&raw).await?;
            let tables: Vec<_> = list.iter().filter(|v| v.type_ == "table").collect();
            for v in tables {
                if v.name.starts_with("sqlite_") {
                    continue;
                }
                metas.push(v.into());
            }
            let indexs: Vec<_> = list.iter().filter(|v| v.type_ == "index").collect();
            for v in indexs {
                let table = extract_range(&v.sql, ("ON", "("));
                let colume = extract_range(&v.sql, ("(", ")"));
                for meta in metas.iter_mut() {
                    if meta.name == table {
                        if v.sql.contains("UNIQUE") {
                            meta.indexs.push(IndexMeta::Unique(colume));
                        } else if v.sql.contains("INDEX") {
                            meta.indexs.push(IndexMeta::Index(colume));
                        } else {
                            continue;
                        }
                        break;
                    }
                }
            }
            Ok(metas)
        })
    }

    fn create_table(&self, meta: &TableMeta) -> Result<String> {
        let chunk = |v: &ColumeMeta| {
            let mut raw = v.to_string();
            let is_primary = v.name == meta.primary;
            if is_primary && v.increment {
                raw.push_str(&raw!(" PRIMARY KEY AUTOINCREMENT"));
            } else if is_primary {
                raw.push_str(&raw!(" PRIMARY KEY"));
            }
            raw
        };
        let columes: Vec<_> = meta.columes.iter().map(chunk).collect();
        Ok(raw!("CREATE TABLE {} ({})", meta.name, columes.join(",")))
    }

    fn colume_raw(&self, t: &TableMeta, v: Adjust, meta: &ColumeMeta) -> Result<Vec<String>> {
        if let Adjust::Add = v {
            return Ok(vec![raw!("ALTER TABLE {} ADD {}", t.name, meta)]);
        }
        panic!("This isn't supported at : {}.{:#?}", t.name, meta)
    }

    fn create_index(&self, t: &TableMeta, meta: &IndexMeta) -> Result<String> {
        let mut raw = raw!("CREATE ");
        if let IndexMeta::Unique(_) = meta {
            raw.push_str("UNIQUE ");
        }
        let name = meta.name(&t.name);
        let column = meta.column();
        raw.push_str(&raw!("INDEX {} ON {} ({})", name, t.name, column));
        Ok(raw)
    }

    fn drop_index(&self, t: &TableMeta, meta: &IndexMeta) -> Result<String> {
        Ok(raw!("DROP INDEX {}", meta.name(&t.name)))
    }
}
