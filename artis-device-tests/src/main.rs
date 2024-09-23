use artis::{rbv, Artis, IntoRaw, Result};
use rbdc_sqlite::SqliteDriver;
use serde::{Deserialize, Serialize};

const LINK: &'static str = "./dist/sql.db";

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

#[tokio::main]
async fn main() {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(SqliteDriver {}, LINK).await;

    let rb: Artis = rb.into();
    let _ = rb;
    let person = Person {
        id: None,
        name: "Tom".into(),
    };
    let c: Result<()> = async move {
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

        // // let raw = Raw::table("persons")
        // //     .where_("id >= ?", vec![rbv!(6)])
        // //     // .select("id,name")
        // //     .select(vec!["id"])
        // //     .order("id DESC");
        // let raw = "persions";
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
        //
        // let person = Person {
        //     id: None,
        //     name: "TOM".into(),
        // };
        // // let raw = ("persons", rbv! {&person,}, vec!["id", "age"]);
        // // println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
        // // let props = rbv! {&person,};
        // // let raw = Raw::table("persons")
        // //     .where_("id = ?", vec![rbv!(3)])
        // //     .model(props)
        // //     .select(vec!["id"]);
        // // println!("raw:{:?}", raw.into_raw(artis::RawType::Saving));
        // let raw = ("persons", rbv! {&person,});
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Saving));
        // let rst = rb
        //     .chunk(|rb| async move {
        //         let last_id: u64 = rb.saving(&raw).await?;
        //         println!("->>>list:{:?}", last_id);
        //         // Err("xixi".into())
        //         Ok(())
        //     })
        //     .await;
        // println!("rst:{:?}", rst);
        // let raw = "persons";
        // let list: Vec<Person> = rb.fetch(&raw).await?;
        // println!("list:{:#?}", list);
        // let (rb) = rb.begin().await?;
        // let ok = rb
        //     .chunk(async {
        //         let last_id: u64 = rb.saving(&raw).await?;
        //         println!("->>>list:{:?}", last_id);
        //         Err("xixi".into())
        //     })
        //     .await?;

        // println!("raw:{:?}", raw.into_raw(artis::RawType::Saving));
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Saving));
        // let last_id: u64 = rb.saving(&raw).await?;
        // println!("_>>>>last_id:{}", last_id);

        // let raw = (
        //     "address",
        //     rbv! {&Address{id:"12".into(),name:"Tom".into()},},
        // );
        // let last_id: String = rb.saving(&raw).await?;
        // println!("_>>>>last_id:{}", last_id);

        // let raw = ("persons", rbv! {&person,}, "id");
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Update));
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Delete));
        //
        // let raw = Raw::table("persons");
        // let raw = raw.limit(1);
        // println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));
        // let list: Vec<Person> = rb.fetch(&raw).await?;
        // println!("list:{:?}", list);
        //
        // println!("->>>>{:?}", rbv!(1));
        Ok(())
    }
    .await;
    println!("{:?} ", c);
}
