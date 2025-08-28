use std::future::Future;
use std::u64;
use std::{fmt::Debug, sync::Arc};

use serde::de::DeserializeOwned;

use crate::decode::{decode, decode_pluck};
use crate::types::{Args, ExecResult, IntoArtis, IntoChunk, RawType};
use crate::ArtisTx;
use crate::IntoRaw;
use crate::{BoxFuture, Result, Value};

pub trait ArtisExecutor: Debug + Send + Sync {
    fn query(&self, raw: String, args: Args) -> BoxFuture<'_, Result<Value>>;

    fn exec(&self, raw: String, args: Args) -> BoxFuture<'_, Result<ExecResult>>;

    fn begin(&self) -> BoxFuture<'_, Result<ArtisTx>>;
}

#[derive(Debug, Clone)]
pub struct Artis {
    c: Arc<Box<dyn ArtisExecutor>>,
}

impl From<Box<dyn ArtisExecutor>> for Artis {
    fn from(value: Box<dyn ArtisExecutor>) -> Self {
        Self { c: Arc::new(value) }
    }
}

impl Artis {
    pub async fn begin(&self) -> Result<ArtisTx> {
        Ok(self.c.begin().await?)
    }

    pub async fn chunk<F, T>(&self, func: F) -> Result<()>
    where
        F: FnOnce(Arc<ArtisTx>) -> T,
        T: Future<Output = Result<()>>,
    {
        let rb = Arc::new(self.c.begin().await?);
        rb.chunk(func(Arc::clone(&rb))).await
    }
}

impl IntoChunk for Artis {
    async fn chunk<F, T>(&self, func: F) -> Result<()>
    where
        F: FnOnce(Arc<ArtisTx>) -> T,
        T: Future<Output = Result<()>>,
    {
        Ok(self.chunk(func).await?)
    }
}

impl IntoArtis for Artis {
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
        let (raw, args) = i.into_raw(RawType::Fetch);
        Ok(self.c.query(raw, args).await?)
    }

    async fn exec(&self, raw: &str, args: Args) -> Result<ExecResult> {
        Ok(self.c.exec(raw.into(), args).await?)
    }
}
