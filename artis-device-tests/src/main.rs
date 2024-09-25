use artis::{
    migrator::{ArtisMigrator, SqliteMigrator},
    Artis, Result,
};
use artis_device::Artis;
use rbdc_sqlite::SqliteDriver;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Artis)]
pub struct Person {
    #[artis(PRIMARY)]
    pub id: Option<u64>,
    pub name: String,
    #[artis(default = "18")]
    pub age: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Address {
    pub id: String,
    pub name: String,
}

const LINK: &'static str = "./dist/sql.db";

async fn acuipe() -> Result<Artis> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(SqliteDriver {}, LINK).await;
    Ok(rb.into())
}

async fn into_migrator() -> Result<()> {
    let rb = acuipe().await?;
    let metas = vec![Person::migrator()];
    println!("{:#?}", metas);
    rb.auto_migrate(&SqliteMigrator {}, metas).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("into_sqlite:{:?}", into_migrator().await);
    // println!("into_raw:{:?}", into_raw().await);
}
