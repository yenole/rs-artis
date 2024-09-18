use std::{
    borrow::{Borrow, BorrowMut},
    cell::RefCell,
    future::Future,
    i16, i32,
    process::Output,
    rc::Rc,
    result,
    sync::Arc,
};

//
// #[derive(Debug, Artis)]
// pub struct Person {
//     #[artis(index, rename = "person_name")]
//     pub person: String,
//     #[artis(name = "xixi")]
//     pub name: Option<String>,
//     pub address: Vec<String>,
//     pub slice: (String, u32),
// }
//
#[derive(Debug)]
struct Artis {
    pub name: String,
}

#[derive(Debug)]
struct ArtisTx {
    pub name: i32,
}

impl ArtisTx {
    pub fn hi(&self) {
        println!("->>>>{:p} {:#?}", self, self);
    }

    pub fn hi_mut(&mut self) {
        println!("->>>>mut {:p} {:#?}", self, self);
    }
}

impl Artis {
    pub async fn process<T, F>(&self, fun: F) -> artis::Result<()>
    where
        F: FnOnce(Rc<ArtisTx>) -> T,
        T: Future<Output = artis::Result<()>>,
    {
        let rb = Rc::new(ArtisTx { name: 10 });
        fun(Rc::clone(&rb)).await;
        if let Ok(mut rb) = Rc::try_unwrap(rb) {
            rb.hi_mut();
        }
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let c = |e: Rc<ArtisTx>| async move {
        e.hi();
        Ok(())
    };
    let mut rb = Artis {
        name: "Jack".into(),
    };
    rb.process(c).await;
}
