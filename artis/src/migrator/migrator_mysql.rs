use serde::Deserialize;

use crate::{map, raw, Artis, BoxFuture, IntoArtis, Raw, Result};

use super::{
    migrator::DriverMigrator,
    types::{Adjust, Mapping},
    ColumeMeta, IndexMeta, TableMeta,
};

const INDEX: &'static str = "information_schema.STATISTICS ";
const INDEX_SELECT: &'static str = "TABLE_NAME,INDEX_NAME,NON_UNIQUE,COLUMN_NAME";

const COLUME: &'static str = "information_schema.columns";
const COLUME_SELECT: &'static str =
    "TABLE_NAME,COLUMN_NAME,COLUMN_TYPE,IS_NULLABLE,COLUMN_DEFAULT,COLUMN_COMMENT";

#[derive(Debug)]
pub struct MysqlMigrator {}

#[derive(Debug, Deserialize)]
struct Schema {
    #[serde(rename = "TABLE_NAME")]
    pub table: String,
    #[serde(rename = "COLUMN_NAME")]
    pub name: String,
    #[serde(rename = "COLUMN_TYPE")]
    pub type_: String,
    #[serde(rename = "IS_NULLABLE")]
    pub nullable: String,
    #[serde(rename = "COLUMN_DEFAULT")]
    pub default: Option<String>,
    #[serde(rename = "COLUMN_COMMENT", default)]
    pub comment: Option<String>,
}

impl Into<ColumeMeta> for &Schema {
    fn into(self) -> ColumeMeta {
        ColumeMeta {
            name: self.name.clone(),
            colume: self.type_.clone(),
            nullable: self.nullable == "YES",
            default: self.default.clone().unwrap_or_default(),
            comment: self.comment.clone().unwrap_or_default(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct Index {
    #[serde(rename = "TABLE_NAME")]
    pub table: String,
    #[serde(rename = "INDEX_NAME")]
    pub name: String,
    #[serde(rename = "NON_UNIQUE")]
    pub unique: u8,
    #[serde(rename = "COLUMN_NAME")]
    pub colume: String,
}

impl<'a> DriverMigrator<'a> for MysqlMigrator {
    fn mapping(&self) -> Mapping {
        map! {
            "i32" : "INT",
            "i64" : "BIGINT",
            "u32" : "INT",
            "u64" : "BIGINT",
            "f32" : "FLOAT",
            "f64" : "DOUBLE",
            "Vec" : "JSON",
            "Map" : "JSON",
            "bool" : "TINYINT",
            "String" : "VARCHAR(255)",
        }
    }

    fn fetch_tables(&self, rb: &'a Artis) -> BoxFuture<'a, Result<Vec<TableMeta>>> {
        let chunk = async move {
            let raw = Raw::table(COLUME)
                .select(COLUME_SELECT.split(",").collect())
                .where_("TABLE_SCHEMA = DATABASE()", vec![])
                .order("TABLE_NAME");
            let list: Vec<Schema> = rb.fetch(&raw).await?;
            let mut metas: Vec<TableMeta> = vec![];
            let mut meta = TableMeta::default();
            for v in list.iter() {
                if meta.name != v.table {
                    meta = TableMeta::default();
                    meta.name = v.table.clone();
                }
                meta.columes.push(v.into());
            }
            if !meta.name.is_empty() {
                metas.push(meta);
            }

            let raw = Raw::table(INDEX)
                .select(INDEX_SELECT.split(",").collect())
                .where_("TABLE_SCHEMA = DATABASE()", vec![])
                .order("TABLE_NAME");
            let list: Vec<Index> = rb.fetch(&raw).await?;
            for v in list {
                for meta in metas.iter_mut() {
                    if meta.name != v.table {
                        continue;
                    }
                    if v.name == "PRIMARY" {
                        meta.primary = v.colume;
                        break;
                    }
                    let inx = if v.unique == 0 {
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
        Ok(meta.into_raw())
    }

    fn colume_raw(&self, table: &str, t: Adjust, meta: &ColumeMeta) -> Result<String> {
        let raw = match t {
            Adjust::Add => raw!("ALTER TABLE {} ADD {}", table, meta),
            Adjust::Alter => raw!("ALTER TABLE {} MODIFY {}", table, meta),
            _ => "".into(),
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
        Ok(raw!("DROP INDEX {} ON {}", meta.name(table), table))
    }
}
