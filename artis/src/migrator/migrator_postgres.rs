use serde::Deserialize;

use crate::{map, raw, Artis, BoxFuture, IntoArtis, IntoRaw, Raw, RawType, Result, Value};

use super::{Adjust, ColumeMeta, DriverMigrator, IndexMeta, TableMeta};

const COLUME: &'static str = "information_schema.columns";
const COLUME_SELECT: &'static str =
    "TABLE_NAME,COLUMN_NAME,UDT_NAME,CHARACTER_MAXIMUM_LENGTH,IS_NULLABLE,COLUMN_DEFAULT";

// reference gorm by golang
const INDEXSQL: &'static str = r#"SELECT ct.relname AS table_name,ci.relname AS index_name,i.indisunique AS non_unique,i.indisprimary AS primary,a.attname AS column_name FROM pg_index i LEFT JOIN pg_class ct ON ct.oid = i.indrelid LEFT JOIN pg_class ci ON ci.oid = i.indexrelid LEFT JOIN pg_attribute a ON a.attrelid = ct.oid LEFT JOIN pg_constraint con ON con.conindid = i.indexrelid WHERE a.attnum = ANY(i.indkey) AND con.oid IS NULL AND ct.relkind = 'r'"#;

#[derive(Debug)]
pub struct PostgresMigrator {}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(rename = "table_name")]
    pub table: String,
    #[serde(rename = "column_name")]
    pub name: String,
    #[serde(rename = "udt_name")]
    pub type_: String,
    #[serde(rename = "is_nullable")]
    pub nullable: String,
    #[serde(rename = "column_default")]
    pub default: Option<String>,
    #[serde(rename = "character_maximum_length")]
    pub max_length: Option<usize>,
}

impl Into<ColumeMeta> for &Schema {
    fn into(self) -> ColumeMeta {
        let mut colume = self.type_.clone().to_uppercase();
        if let Some(v) = self.max_length {
            colume.push_str(&format!("({})", v));
        }
        let mut default = self.default.clone().unwrap_or_default();
        let is_increment = default.starts_with("nextval(");
        if is_increment {
            colume = "SERIAL".into();
            default = "".into();
        }
        if default.contains("::character") {
            default = extract_range(&default, ("'", "'"));
        }
        ColumeMeta {
            name: self.name.clone(),
            size: 0,
            colume,
            nullable: self.nullable == "YES",
            default,
            comment: "".into(),
            increment: false,
        }
    }
}

struct IndexRaw;

impl IntoRaw for IndexRaw {
    fn into_raw(&self, v: RawType) -> (&'static str, Vec<Value>) {
        if !v.is_fetch() {
            panic!("Not Support");
        }
        (INDEXSQL, vec![])
    }
}

#[derive(Debug, Deserialize)]
struct Index {
    #[serde(rename = "table_name")]
    pub table: String,
    #[serde(rename = "index_name")]
    pub name: String,
    #[serde(rename = "non_unique")]
    pub unique: bool,
    #[serde(rename = "column_name")]
    pub colume: String,
}

impl PostgresMigrator {
    fn mapping() -> super::Mapping {
        map! {
            "i32" : "INT4",
            "i64" : "INT8",
            "u32" : "INT4",
            "u64" : "INT8",
            "f32" : "REAL",
            "f64" : "DECIMAL",
            "Vec" : "JSON",
            "Map" : "JSON",
            "bool" : "BOOLEAN",
            "String" : "VARCHAR(255)",
        }
    }
}

impl<'a> DriverMigrator<'a> for PostgresMigrator {
    fn mapping(&self, meta: &mut TableMeta) {
        let dict = PostgresMigrator::mapping();
        meta.columes.iter_mut().for_each(|v: &mut ColumeMeta| {
            if !v.colume.starts_with(":") {
                return;
            }
            if v.increment {
                v.colume = "SERIAL".into();
                return;
            }
            let key = v.colume[1..].trim();
            if !dict.contains_key(key) {
                panic!("mapping not found: {}", key);
            }
            v.colume = dict[key].into();
        });
    }

    fn fetch_tables(&self, rb: &'a Artis) -> BoxFuture<'a, Result<Vec<TableMeta>>> {
        let chunk = async move {
            let inw = "TABLE_SCHEMA = 'public' AND TABLE_CATALOG = CURRENT_DATABASE()";
            let raw = Raw::table(COLUME)
                .select(COLUME_SELECT.split(",").collect())
                .where_(inw, vec![])
                .order("TABLE_NAME");
            let list: Vec<Schema> = rb.fetch(&raw).await?;
            let mut metas: Vec<TableMeta> = vec![];
            let mut meta = TableMeta::default();
            for v in list.iter() {
                if meta.name != v.table {
                    if !meta.name.is_empty() {
                        metas.push(meta);
                    }
                    meta = TableMeta::default();
                    meta.name = v.table.clone();
                }
                meta.columes.push(v.into());
            }
            if !meta.name.is_empty() {
                metas.push(meta);
            }
            let list: Vec<Index> = rb.fetch(&IndexRaw {}).await?;
            for v in list {
                for meta in metas.iter_mut() {
                    if meta.name != v.table {
                        continue;
                    }
                    if v.name == "PRIMARY" {
                        meta.primary = v.colume;
                        break;
                    }
                    let inx = if v.unique {
                        IndexMeta::Unique(v.colume)
                    } else {
                        IndexMeta::Index(v.colume)
                    };
                    meta.indexs.push(inx);
                    break;
                }
            }
            Ok(metas)
        };
        Box::pin(chunk)
    }

    fn create_table(&self, meta: &TableMeta) -> Result<String> {
        let chunk = |v: &ColumeMeta| raw!("{}", v);
        let columes: Vec<_> = meta.columes.iter().map(chunk).collect();
        let mut raw = raw!("CREATE TABLE {} ({})", meta.name, columes.join(", "));
        if !meta.primary.is_empty() {
            raw.truncate(raw.len() - 1);
            raw.push_str(&raw!(", PRIMARY KEY({}))", meta.primary));
        }
        Ok(raw)
    }

    fn colume_raw(&self, t: &TableMeta, v: Adjust, meta: &ColumeMeta) -> Result<Vec<String>> {
        let column = if meta.size == 0 {
            meta.colume.clone()
        } else {
            raw!("{}({})", meta.colume, meta.size)
        };
        let raws = match v {
            Adjust::Add => vec![raw!("ALTER TABLE {} ADD {}", t.name, meta)],
            Adjust::Alter => {
                let raw = raw!("ALTER TABLE {} ALTER COLUMN {}", t.name, meta.name);
                let mut raws = vec![];
                raws.push(raw!("{} TYPE {}", raw, column));
                if meta.nullable {
                    raws.push(raw!("{} DROP NOT NULL", raw));
                } else {
                    raws.push(raw!("{} SET NOT NULL", raw));
                }

                if meta.default.is_empty() {
                    raws.push(raw!("{} DROP DEFAULT", raw));
                } else {
                    raws.push(raw!("{} SET DEFAULT {}", raw, meta.default));
                }
                raws
            }
            _ => vec![],
        };
        Ok(raws)
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

    fn drop_index(&self, t: &TableMeta, meta: &super::IndexMeta) -> Result<String> {
        Ok(raw!("DROP INDEX {}", meta.name(&t.name)))
    }
}

#[inline]
fn extract_range(str: &str, p: (&str, &str)) -> String {
    let r_idx = str.find(p.0).unwrap();
    let l_idx = str.rfind(p.1).unwrap();
    str[r_idx + p.0.len()..l_idx].trim().into()
}
