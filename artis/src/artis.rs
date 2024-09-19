use std::{future::Future, sync::Arc};

use rbatis::RBatis;
use serde::de::DeserializeOwned;

use crate::{
    artis_tx::ArtisTx, into_delete::IntoDelete, into_saving::IntoSaving, into_select::IntoSelect,
    into_update::IntoUpdate, rbox, Result,
};

#[derive(Debug)]
pub struct Artis {
    pub rb: Arc<RBatis>,
}

impl From<Arc<RBatis>> for Artis {
    fn from(value: Arc<RBatis>) -> Self {
        Self { rb: value }
    }
}

pub trait IntoArtis: Sync + Send {
    fn list<T>(&self, i: impl IntoSelect) -> impl Future<Output = Result<Vec<T>>>
    where
        T: DeserializeOwned;

    // fn list<T, R, S>(&self, i: R) -> S
    // where
    //     R: IntoSelect,
    //     S: Future<Output = Result<Vec<T>>>,
    //     T: DeserializeOwned;

    fn saving(&self, i: impl IntoSaving) -> impl Future<Output = Result<rbs::Value>>;

    fn update(&self, i: impl IntoUpdate) -> impl Future<Output = Result<u64>>;

    fn delete(&self, i: impl IntoDelete) -> impl Future<Output = Result<u64>>;
}

impl IntoArtis for Artis {
    fn list<T>(&self, i: impl IntoSelect) -> impl Future<Output = Result<Vec<T>>>
    where
        T: DeserializeOwned,
    {
        let c = async move {
            let (raw, list) = i.into_select();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            Ok(self.rb.query_decode::<Vec<T>>(raw, list).await?)
        };
        Box::pin(c)
    }

    fn saving(&self, i: impl IntoSaving) -> impl Future<Output = Result<rbs::Value>> {
        let c = async move {
            let (raw, args) = i.into_saving();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.last_insert_id)
        };
        Box::pin(c)
    }

    fn update(&self, i: impl IntoUpdate) -> impl Future<Output = Result<u64>> {
        let c = async move {
            let (raw, args) = i.into_update();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.rows_affected)
        };
        Box::pin(c)
    }

    fn delete(&self, i: impl IntoDelete) -> impl Future<Output = Result<u64>> {
        let c = async move {
            let (raw, args) = i.into_raw();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.rows_affected)
        };
        Box::pin(c)
    }
}

impl Artis {
    pub async fn acquire(&self) -> Result<ArtisTx> {
        let tx = self.rb.acquire().await?;
        let tx = tx.begin().await?;
        Ok(ArtisTx::from(tx))
    }

    pub async fn transaction<F>(&self, func: F) -> Result<()>
    where
        F: FnOnce(&ArtisTx) -> Result<()>,
    {
        let mut rb = self.acquire().await?;
        if let Err(e) = func(&rb) {
            rb.rollback().await?;
            return Err(e);
        }
        rb.commit().await
    }
}
