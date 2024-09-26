[![doc.rs](https://docs.rs/artis/badge.svg)](https://docs.rs/artis/)
[![GitHub release](https://img.shields.io/github/v/release/yenole/rs-artis)](https://github.com/yenole/rs-artis/releases)

#### 依赖

```toml
#artis deps
artis = {version = "0.2.2", features = ["derive","sqlite"]}

#rbatis deps
rbs = { version = "4.5"}
rbatis = { version = "4.5"}
rbdc-sqlite = { version = "4.5" }
#rbdc-mysql={version="4.5"}
#rbdc-pg={version="4.5"}
#rbdc-mssql={version="4.5"}

#other deps
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
```

#### 自动生成表

```rust
#[derive(Debug, Serialize, Deserialize, artis::Artis)]
pub struct Person {
    #[artis(PRIMARY, AUTO_INCREMENT)]
    pub id: Option<u64>,
    #[artis(type = "VARCHAR", size = 255, INDEX)]
    pub name: String,
    #[artis(default = "18", comment = "年龄")]
    pub age: u32,
    #[artis(NO_NULL, UNIQUE)]
    pub id_card: Vec<String>,
}


async fn sqlite_migrator() -> Result<()> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(SqliteDriver {}, "./dist/sql.db").await?;
    let rb: Artis = rb.into();
    let metas = vec![Person::migrator()];
    rb.auto_migrate(&SqliteMigrator {}, metas).await?;
    Ok(())
}

async fn mysql_migrator() -> Result<()> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(MysqlDriver {}, "mysql://root:xxxx@locahost:3306/database").await?;
    let rb: Artis = rb.into();
    let metas = vec![Person::migrator()];
    rb.auto_migrate(&MysqlMigrator {}, metas).await?;
    Ok(())
}
```

