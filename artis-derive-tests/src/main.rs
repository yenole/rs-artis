use std::{collections::HashMap, sync::Arc};

use artis::{
    meta, migrator::ArtisMigrator, rbv, Artis, ArtisTx, IntoArtis, IntoRaw, IntoTable, RawType,
    Result,
};

#[cfg(feature = "mysql")]
use artis::migrator::MysqlMigrator;

#[cfg(feature = "mysql")]
use rbdc_mysql::MysqlDriver;

#[cfg(feature = "sqlite")]
use rbdc_sqlite::SqliteDriver;

#[cfg(feature = "sqlite")]
use artis::migrator::SqliteMigrator;

#[cfg(feature = "postgres")]
use artis::migrator::PostgresMigrator;

#[cfg(feature = "postgres")]
use rbdc_pg::PostgresDriver;

#[cfg(feature = "sqlite")]
const LINK: &'static str = "./dist/sql.db";
#[cfg(feature = "mysql")]
const LINK: &'static str = "mysql://root:root@localhost:3306/artis";
#[cfg(feature = "postgres")]
const LINK: &'static str = "postgres://postgres:root@localhost:5432/artis";

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, artis::Artis)]
#[artis(table = "persons")]
pub struct Person {
    #[artis(PRIMARY, AUTO_INCREMENT)]
    pub id: Option<u64>,
    pub name: String,
    #[artis(default = 18)]
    pub age: u32,
}

#[derive(Debug, Serialize, Deserialize, artis::Artis)]
pub struct Demo {
    #[artis(PRIMARY, AUTO_INCREMENT)]
    pub id: Option<u64>,
    #[artis(type = "VARCHAR", size = 253, default = "Tom")]
    pub name: String,
    #[artis(INDEX)]
    pub age: i32,
    #[artis(UNIQUE)]
    pub id_card: i32,
    pub list: Vec<i32>,
    pub list2: Option<Vec<i32>>,
    pub map: HashMap<i32, i32>,
    pub map2: Option<HashMap<i32, i32>>,
}

fn init_logs() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .target(env_logger::Target::Stdout)
        .init();
}

async fn acuipe() -> Result<Artis> {
    let rb = rbatis::RBatis::new();
    #[cfg(feature = "sqlite")]
    {
        let _ = rb.link(SqliteDriver {}, LINK).await?;
    }
    #[cfg(feature = "mysql")]
    {
        let _ = rb.link(MysqlDriver {}, LINK).await?;
    }

    #[cfg(feature = "postgres")]
    {
        let _ = rb.link(PostgresDriver {}, LINK).await?;
    }
    Ok(rb.into())
}

async fn into_migrator(rb: &Artis) -> Result<()> {
    // let metas = vec![Person::migrator()];
    let metas = meta!(Demo, Person);

    #[cfg(feature = "mysql")]
    rb.auto_migrate(&MysqlMigrator {}, metas).await?;

    #[cfg(feature = "sqlite")]
    rb.auto_migrate(&SqliteMigrator {}, metas).await?;

    #[cfg(feature = "postgres")]
    rb.auto_migrate(&PostgresMigrator {}, metas).await?;
    Ok(())
}

async fn into_delete(rb: &Artis) -> Result<()> {
    let raw = ("persons", rbv! {"id" : 1,});
    let line = rb.delete(&raw).await?;
    println!("delete line: {:?}", line);

    let raw = "persons".to_string();
    let line = rb.delete(&raw).await?;
    println!("delete line: {:?}", line);
    Ok(())
}

async fn into_saving(rb: &Artis) -> Result<()> {
    let person = Person {
        id: None,
        name: "Jack".into(),
        age: 18,
    };
    // 保存对象
    let raw = ("persons", rbv!(person));
    rb.saving(&raw).await?;

    #[derive(Debug, Serialize)]
    pub struct Other {
        pub name: String,
    }
    let other = Other { name: "Tom".into() };
    // 保存其他对象，并扩展一些表属性
    let raw = ("persons", rbv! {other,"age":30});
    // 返回保存ID
    let v = rb.saving(&raw).await?;
    println!("saving table id:{:?}", v);
    Ok(())
}

async fn into_update(rb: &Artis) -> Result<()> {
    let person = Person {
        id: Some(1),
        name: "Tom".into(),
        age: 19,
    };
    let raw = ("persons", rbv!(&person), "name");
    rb.update(&raw).await?;

    Ok(())
}

#[derive(Debug)]
enum Schema {
    Person,
}
impl IntoTable for Schema {
    fn into_table(&self) -> String {
        match self {
            Self::Person => "persons".into(),
        }
    }
}

async fn into_fetch(rb: &Artis) -> Result<()> {
    let raw = ("persons", vec!["COUNT(id)"]);
    let count: i64 = rb.fetch(&raw).await?;
    println!("count:{:?}", count);

    let raw = "persons".to_string();
    let list: Vec<Person> = rb.fetch(&raw).await?;
    println!("list:{:?}", list);

    let list: Vec<String> = rb.pluck(&raw, "name").await?;
    println!("list:{:?}", list);

    let raw = (Schema::Person, 1);
    let one: Option<Person> = rb.fetch(&raw).await?;
    println!("one:{:?}", one);
    Ok(())
}

async fn into_chunk(rb: &Artis) -> Result<()> {
    let chunk = |rb: Arc<ArtisTx>| async move {
        let raw = ("persons", rbv! {"name":"by chunk","age":30});
        let _ = rb.saving(&raw).await?;
        // Err("异常".into())  // rollback
        Ok(()) // commit
    };
    rb.chunk(chunk).await?;
    Ok(())
}

macro_rules! fmt {
    ($k:expr,$t:ident,$v:expr) => {
        assert_eq!($k.into_raw(RawType::$t).0, $v)
    };
}

async fn into_raw() -> Result<()> {
    let raw = "persons".to_string();
    fmt!(raw, Fetch, "SELECT * FROM persons");
    fmt!(raw, Delete, "DELETE FROM persons");

    // (table,order)
    let raw = (Schema::Person, "id");
    fmt!(raw, Fetch, "SELECT * FROM persons ORDER BY id");

    // (table,order,limit)
    let raw = (Schema::Person, "id", 1);
    fmt!(raw, Fetch, "SELECT * FROM persons ORDER BY id LIMIT 1");

    // (table,(where,args))
    let raw = (Schema::Person, ("id = ?", vec![rbv!(1)]));
    fmt!(raw, Fetch, "SELECT * FROM persons WHERE id = ?");
    fmt!(raw, Delete, "DELETE FROM persons WHERE id = ?");

    // (table,(wher,args),limit))
    let raw = (Schema::Person, ("id = ?", vec![rbv!(1)]), 1);
    fmt!(raw, Fetch, "SELECT * FROM persons WHERE id = ? LIMIT 1");

    // (table,(wher,args),order))
    let raw = (Schema::Person, ("id = ?", vec![rbv!(1)]), "id DESC");
    fmt!(
        raw,
        Fetch,
        "SELECT * FROM persons WHERE id = ? ORDER BY id DESC"
    );

    // (table,(wher,args),order,limit))
    let raw = (Schema::Person, ("id = ?", vec![rbv!(1)]), "id DESC", 1);
    fmt!(
        raw,
        Fetch,
        "SELECT * FROM persons WHERE id = ? ORDER BY id DESC LIMIT 1"
    );

    // (table,limit)
    let raw = (Schema::Person, 1);
    fmt!(raw, Fetch, "SELECT * FROM persons LIMIT 1");

    // (table,model)
    let raw = (Schema::Person, rbv! {"id":1,});
    fmt!(raw, Fetch, "SELECT * FROM persons WHERE id = ?");
    fmt!(raw, Saving, "INSERT INTO persons(id) VALUES (?)");
    fmt!(raw, Delete, "DELETE FROM persons WHERE id = ?");

    // (table,model,limit)
    let raw = (Schema::Person, rbv! {"id":1,}, 1);
    fmt!(raw, Fetch, "SELECT * FROM persons WHERE id = ? LIMIT 1");

    // (table,model,order)
    let raw = (Schema::Person, rbv! {"id":1,}, "id DESC");
    fmt!(
        raw,
        Fetch,
        "SELECT * FROM persons WHERE id = ? ORDER BY id DESC"
    );
    // (table,mode,colume)
    let raw = (Schema::Person, rbv! {"id":1,"name":"Tom"}, "name");
    fmt!(raw, Update, "UPDATE persons SET id = ? WHERE name = ?");
    // (table,mode,vec)
    let raw = (
        Schema::Person,
        rbv! {"id":1,"name":"Tom","age":123},
        vec!["name", "id"],
    );
    fmt!(
        raw,
        Update,
        "UPDATE persons SET age = ? WHERE name = ? AND id = ?"
    );

    // (table,model,order,limit)
    let raw = (Schema::Person, rbv! {"id":1,}, "id DESC", 1);
    fmt!(
        raw,
        Fetch,
        "SELECT * FROM persons WHERE id = ? ORDER BY id DESC LIMIT 1"
    );

    // (table,select)
    let raw = (Schema::Person, vec!["id"]);
    fmt!(raw, Fetch, "SELECT id FROM persons");

    // (table,select,where)
    let raw = (Schema::Person, vec!["id"], ("id = ?", vec![rbv!(1)]));
    fmt!(raw, Fetch, "SELECT id FROM persons WHERE id = ?");

    // (table,select,limit)
    let raw = (Schema::Person, vec!["id"], 1);
    fmt!(raw, Fetch, "SELECT id FROM persons LIMIT 1");

    // (table,select,model)
    let raw = (Schema::Person, vec!["id"], rbv! {"id":1,"age":19,});
    fmt!(raw, Saving, "INSERT INTO persons(id) VALUES (?)");

    // (table,select,model,order)
    let raw = (
        Schema::Person,
        vec!["id"],
        rbv! {"id":1,"age":19,},
        "id DESC",
    );
    fmt!(
        raw,
        Fetch,
        "SELECT id FROM persons WHERE id = ? ORDER BY id DESC"
    );

    // (table,select,model,colume)
    let raw = (Schema::Person, vec!["age"], rbv! {"id":1,"age":19,}, "id");
    fmt!(raw, Update, "UPDATE persons SET age = ? WHERE id = ?");

    // (table,select,order)
    let raw = (Schema::Person, vec!["id"], "id DESC");
    fmt!(raw, Fetch, "SELECT id FROM persons ORDER BY id DESC");

    // (table,select,order,limit)
    let raw = (Schema::Person, vec!["id"], "id DESC", 1);
    fmt!(
        raw,
        Fetch,
        "SELECT id FROM persons ORDER BY id DESC LIMIT 1"
    );

    Ok(())
}

#[tokio::main]
async fn main() {
    init_logs();
    let rb = acuipe().await.unwrap();
    println!("into_migrator reuslt:{:?}", into_migrator(&rb).await);
    println!("into_delete result:{:?}", into_delete(&rb).await);
    println!("into_saving result:{:?}", into_saving(&rb).await);
    println!("into_saving result:{:?}", into_update(&rb).await);
    println!("into_fetch  result:{:?}", into_fetch(&rb).await);
    println!("into_chunk  result:{:?}", into_chunk(&rb).await);
    println!("into_raw    result:{:?}", into_raw().await);
}
