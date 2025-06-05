[![doc.rs](https://docs.rs/artis/badge.svg)](https://docs.rs/artis/)
[![GitHub release](https://img.shields.io/github/v/release/yenole/rs-artis)](https://github.com/yenole/rs-artis/releases)

#### TODO

* [x] 支持mysql
* [x] 支持sqlite
* [x] 支持postgres
* [x] 自动更新表列和索引
* [ ] sqlite 支持修改列
* [ ] rbatis 改为feature


#### 依赖

```toml
#artis deps
artis = {version = "0.2.4", features = ["derive","sqlite","mysql","postgres"]}

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
#[artis(table = "person")]
pub struct Person {
    #[artis(PRIMARY, AUTO_INCREMENT)]
    pub id: Option<u64>,
    pub name: String,
    #[artis(default = "18")]
    pub age: u32,
}

#[derive(Debug, Serialize, Deserialize, artis::Artis)]
pub struct Demo {
    #[artis(PRIMARY, AUTO_INCREMENT)]
    pub id: Option<u64>,
    #[artis(type = "VARCHAR", size = 255, default = "Artis")]
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


async fn sqlite_migrator() -> Result<()> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(SqliteDriver {}, "./dist/sql.db").await?;
    let rb: Artis = rb.into();
    let metas = meta!(Demo, Person);
    rb.auto_migrate(&SqliteMigrator {}, metas).await?;
    Ok(())
}

async fn mysql_migrator() -> Result<()> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(MysqlDriver {}, "mysql://root:xxxx@locahost:3306/database").await?;
    let rb: Artis = rb.into();
    let metas = meta!(Demo, Person);
    rb.auto_migrate(&MysqlMigrator {}, metas).await?;
    Ok(())
}


async fn postgres_migrator() -> Result<()> {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(PostgresDriver {}, "postgres://postgres:xxxx@locahost:5432/database").await?;
    let rb: Artis = rb.into();
    let metas = meta!(Demo, Person);
    rb.auto_migrate(&PostgresMigrator {}, metas).await?;
    Ok(())
}
```


#### 删除操作

```rust
async fn into_delete(rb: &Artis) -> Result<()> {
    let raw = ("persons", rbv!{"id" : 1,});
    let line = rb.delete(&raw).await?;
    println!("delete line: {:?}", line);

    let raw = "persons".to_string();
    let line = rb.delete(&raw).await?;
    println!("delete line: {:?}", line);
    Ok(())
}
```

#### 更新操作

```rust
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
```

#### 保存操作

```rust
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
```


#### 查询操作

```rust
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
```


#### 事务操作

```rust
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
```


#### SQL构造器

```rust
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
    fmt!(
        raw,
        Update,
        "UPDATE persons SET id = ? WHERE name = ?"
    );

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
```
