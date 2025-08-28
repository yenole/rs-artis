use std::{fmt::Debug, future::Future, sync::Arc};

use serde::de::DeserializeOwned;

use crate::{
    decode::{decode, decode_pluck},
    types::{Args, IntoChunk},
    BoxFuture, ExecResult, IntoArtis, IntoRaw, RawType, Result, Value,
};

pub trait ArtisTxExecutor: Debug + Send + Sync {
    fn query(&self, raw: String, args: Args) -> BoxFuture<'_, Result<Value>>;
    fn exec(&self, raw: String, args: Args) -> BoxFuture<'_, Result<ExecResult>>;

    fn commit(&self) -> BoxFuture<'_, Result<()>>;
    fn rollback(&self) -> BoxFuture<'_, Result<()>>;
}

#[derive(Debug)]
pub struct ArtisTx {
    c: Arc<Box<dyn ArtisTxExecutor>>,
}

impl From<Box<dyn ArtisTxExecutor>> for ArtisTx {
    fn from(value: Box<dyn ArtisTxExecutor>) -> Self {
        Self { c: Arc::new(value) }
    }
}

impl ArtisTx {
    pub async fn chunk<T>(&self, func: T) -> Result<()>
    where
        T: Future<Output = Result<()>>,
    {
        if let Err(v) = func.await {
            self.c.rollback().await?;
            return Err(v);
        }
        self.c.commit().await
    }
}

impl IntoChunk for ArtisTx {
    async fn chunk<F, T>(&self, _: F) -> Result<()>
    where
        F: FnOnce(Arc<ArtisTx>) -> T,
        T: Future<Output = Result<()>>,
    {
        Err("Not support".into())
    }
}

impl IntoArtis for ArtisTx {
    async fn fetch<T: DeserializeOwned>(&self, i: &dyn IntoRaw) -> Result<T> {
        let (raw, args) = i.into_raw(RawType::Fetch);
        Ok(decode(self.c.query(raw, args).await?)?)
    }

    async fn pluck<T: DeserializeOwned>(&self, i: &dyn IntoRaw, colume: &'static str) -> Result<T> {
        let (raw, args) = i.into_raw(RawType::Fetch);
        Ok(decode_pluck(self.c.query(raw, args).await?, colume)?)
    }

    async fn saving(&self, i: &dyn IntoRaw) -> Result<Value> {
        let (raw, args) = i.into_raw(RawType::Saving);
        Ok(self.c.exec(raw, args).await?.last_insert_id)
    }

    async fn update(&self, i: &dyn IntoRaw) -> Result<u64> {
        let (raw, args) = i.into_raw(RawType::Update);
        Ok(self.c.exec(raw, args).await?.rows_affected)
    }

    async fn delete(&self, i: &dyn IntoRaw) -> Result<u64> {
        let (raw, args) = i.into_raw(RawType::Delete);
        Ok(self.c.exec(raw, args).await?.rows_affected)
    }

    async fn query(&self, i: &dyn IntoRaw) -> Result<Value> {
        let (raw, args) = i.into_raw(RawType::Delete);
        Ok(self.c.query(raw, args).await?)
    }

    async fn exec(&self, raw: &str, args: Args) -> Result<ExecResult> {
        Ok(self.c.exec(raw.into(), args).await?)
    }
}
