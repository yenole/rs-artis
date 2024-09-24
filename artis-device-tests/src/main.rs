use artis::{
    migrator::{migrator::ArtisMigrator, ColumeMeta, IndexMeta, MysqlMigrator, TableMeta},
    rbv, Artis, IntoRaw, Result,
};
use artis_device::Artis;
use rbdc_mysql::MysqlDriver;
// use rbdc_sqlite::SqliteDriver;
use serde::{Deserialize, Serialize};

#[derive(Debug, Artis)]
pub struct User {
    #[artis(type = "BIGINT", NOT_NULL, PRIMARY)]
    pub id: Option<u64>,
    #[artis(type = "TEXT", NOT_NULL, UNIQUE)]
    pub name: String,
    #[artis(default = 10, comment = "age")]
    pub age: i32,
    #[artis(type = "JSON", INDEX)]
    pub list: Vec<i32>,
}

impl ArtisMigrator for User {
    fn migrator() -> TableMeta {
        TableMeta {
            name: "persons".into(),
            primary: "id".into(),
            columes: vec![
                // ("id", "BIGINT", false, "", "").into(),
                // ("name", "VARCHAR(255)", true, "'Jack'", "").into(),
                // ("age", ":String", true, "18", "").into(),
            ],
            indexs: vec![IndexMeta::Index("age".into())],
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<u64>,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub id: String,
    pub name: String,
}

const LINK: &'static str = "mysql://root:root@127.0.0.1:32306/artis";
// const LINK: &'static str = "./dist/sql.db";

async fn acuipe() -> Result<Artis> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(MysqlDriver {}, LINK).await;
    Ok(rb.into())
}

async fn into_raw() -> Result<()> {
    let rb: Artis = acuipe().await?;
    let _ = rb;
    let person = Person {
        id: None,
        name: "Tom".into(),
    };
    let raw = "persons";
    println!("raw:{:?}", raw.to_string().into_raw(artis::RawType::Fetch));
    let raw = ("persons", 1);
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", "id DESC", (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", rbv! {&person,});
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", rbv! {&person,}, 1);
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", rbv! {&person,}, (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", rbv! {&person,}, "id DESC", (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", ("id = ?", vec![rbv!(3)]));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", ("id = ?", vec![rbv!(3)]), 1);
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", ("id = ?", vec![rbv!(3)]), (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    let raw = ("persons", ("id = ?", vec![rbv!(3)]), "id DESC", (1, 2));
    println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
    Ok(())
}

async fn into_migrator() -> Result<()> {
    let rb = acuipe().await?;
    let metas: Vec<_> = vec![User::migrator()];
    rb.auto_migrate(&MysqlMigrator {}, metas).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("into_sqlite:{:?}", into_migrator().await);
    // println!("into_raw:{:?}", into_raw().await);
}
