use artis::{rbv, Artis, IntoRaw, Raw, Result};
use rbdc_sqlite::SqliteDriver;
use rbs::to_value;
use serde::{Deserialize, Serialize};

const LINK: &'static str = "./dist/sql.db";

#[derive(Debug, Serialize, Deserialize)]
pub struct Person {
    pub id: Option<u64>,
    pub name: String,
}

#[tokio::main]
async fn main() {
    let rb = rbatis::RBatis::new();
    let _ = rb.link(SqliteDriver {}, LINK).await;

    let rb: Artis = rb.into();
    let _ = rb;
    let c: Result<()> = async move {
        let raw = Raw::table("persons")
            .where_("id >= ?", vec![to_value!(6)])
            .select("id,name")
            .order("id DESC");

        println!("raw:{:?}", raw.into_raw(artis::RawType::Fetch));

        let person = Person {
            id: None,
            name: "Jack".into(),
        };
        let props = rbv! {&person,};
        let raw = Raw::table("persons")
            .where_("id = ?", vec![to_value!(3)])
            .model(props);
        println!("raw:{:?}", raw.into_raw(artis::RawType::Saving));
        // rb.saving(&("persions", person)).await?;
        println!("raw:{:?}", raw.into_raw(artis::RawType::Update));
        println!("raw:{:?}", raw.into_raw(artis::RawType::Delete));
        Ok(())
    }
    .await;
    println!("{:?} ", c);
}
