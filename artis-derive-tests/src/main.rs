use artis::{migrator::ArtisMigrator, rbv, Artis, IntoArtis, Result};

#[cfg(feature = "mysql")]
use artis::migrator::MysqlMigrator;
#[cfg(feature = "mysql")]
use rbdc_mysql::MysqlDriver;

#[cfg(feature = "sqlite")]
use rbdc_sqlite::SqliteDriver;

#[cfg(feature = "sqlite")]
use artis::migrator::SqliteMigrator;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, artis::Artis)]
pub struct Person {
    #[artis(PRIMARY, AUTO_INCREMENT)]
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

#[cfg(feature = "sqlite")]
const LINK: &'static str = "./dist/sql.db";
#[cfg(feature = "mysql")]
const LINK: &'static str = "mysql://root:root@localhost:32306/artis";

async fn acuipe() -> Result<Artis> {
    let rb = rbatis::RBatis::new();
    #[cfg(feature = "sqlite")]
    {
        let _ = rb.link(SqliteDriver {}, LINK).await?;
    }
    #[cfg(feature = "mysql")]
    {
        let _ = rb.link(MysqlDriver {}, LINK2).await?;
    }
    Ok(rb.into())
}

async fn into_migrator() -> Result<()> {
    let rb = acuipe().await?;
    let metas = vec![Person::migrator()];
    #[cfg(feature = "mysql")]
    rb.auto_migrate(&MysqlMigrator {}, metas).await?;

    #[cfg(feature = "sqlite")]
    rb.auto_migrate(&SqliteMigrator {}, metas).await?;

    Ok(())
}

async fn into_plus() -> Result<()> {
    let rb = acuipe().await?;
    // let raw = ("persons", rbv! { "name":"Jack", });
    // let _ = rb.saving(&raw).await?;
    let raw = ("persons", ("id > 0", vec![]));
    let names: Vec<String> = rb.pluck(&raw, "name").await?;
    println!("{:?}", names);
    Ok(())
}

#[tokio::main]
async fn main() {
    println!("into_migrator:{:?}", into_migrator().await);
    println!("into_raw:{:?}", into_plus().await);
}
