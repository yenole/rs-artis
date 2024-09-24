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

#[derive(Debug, Deserialize)]
struct SqliteTable {
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub sql: String,
}

fn extract_range(str: &str) -> String {
    let r_idx = str.find("(").unwrap();
    let l_idx = str.rfind(")").unwrap();
    str[r_idx + 1..l_idx].into()
}

impl Into<TableMeta> for &SqliteTable {
    fn into(self) -> TableMeta {
        let mut meta = TableMeta::default();
        meta.name = self.name.clone();
        let raw = extract_range(&self.sql);
        let columes: Vec<_> = raw.split(",").collect();
        columes.iter().for_each(|v| {
            if v.trim().starts_with("PRIMARY KEY") {
                meta.primary = extract_range(v);
            } else {
                meta.columes.push(v.to_string().into());
            }
        });
        meta
    }
}

impl<'a> DriverMigrator<'a> for SqliteMigrator {
    fn mapping(&self) -> super::Mapping {
        map! {
            "i32" : "INTEGER",
            "i64" : "INT8",
            "u32" : "INTEGER",
            "u64" : "INT8",
            "f32" : "DOUBLE",
            "f64" : "DOUBLE",
            "Vec" : "BLOB",
            "Map" : "BLOB",
            "bool" : "BOOLEAN",
            "String" : "TEXT",
        }
    }

    fn fetch_tables(&self, rb: &'a Artis) -> BoxFuture<'a, Result<Vec<TableMeta>>> {
        Box::pin(async move {
            let mut metas: Vec<TableMeta> = vec![];
            let raw = (MASTER, ("sql NOT NULL", vec![]), "`type` DESC");
            let list: Vec<SqliteTable> = rb.fetch(&raw).await?;
            let tables: Vec<_> = list.iter().filter(|v| v.type_ == "table").collect();
            for v in tables {
                metas.push(v.into());
            }
            let indexs: Vec<_> = list.iter().filter(|v| v.type_ == "index").collect();
            for v in indexs {
                let slice: Vec<_> = v.name.split("_").collect();
                for meta in metas.iter_mut() {
                    if meta.name == slice[1] {
                        let inx = match slice[0] {
                            "idx" => IndexMeta::Index(slice[2].into()),
                            "unq" => IndexMeta::Unique(slice[2].into()),
                            _ => continue,
                        };
                        meta.indexs.push(inx);
                        break;
                    }
                }
            }
            Ok(metas)
        })
    }

    fn create_table(&self, meta: &TableMeta) -> Result<String> {
        Ok(meta.into_raw())
    }

    fn colume_raw(&self, table: &str, t: Adjust, meta: &ColumeMeta) -> Result<String> {
        let raw = match t {
            Adjust::Add => raw!("ALTER TABLE {} ADD {}", table, meta),
            Adjust::Drop => panic!("Drop colume is not supported"),
            Adjust::Alter => raw!("ALTER TABLE {} ALTER COLUMN {}", table, meta),
        };
        Ok(raw)
    }

    fn create_index(&self, table: &str, meta: &IndexMeta) -> Result<String> {
        let mut raw = raw!("CREATE ");
        if let IndexMeta::Unique(_) = meta {
            raw.push_str("UNIQUE ");
        }
        let name = meta.name(table);
        let column = meta.column();
        raw.push_str(&raw!("INDEX {} ON {} ({})", name, table, column));
        Ok(raw)
    }

    fn drop_index(&self, table: &str, meta: &IndexMeta) -> Result<String> {
        Ok(raw!("DROP INDEX {}", meta.name(table)))
    }
}
