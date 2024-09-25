use std::future::Future;
use std::u64;
use std::{fmt::Debug, sync::Arc};

use serde::de::DeserializeOwned;

use crate::decode::decode;
use crate::types::{Args, ExecResult, IntoArtis, RawType};
use crate::ArtisTx;
use crate::IntoRaw;
use crate::{BoxFuture, Result, Value};

pub trait Executor: Debug + Send + Sync {
    fn query(&self, raw: String, args: Args) -> BoxFuture<Result<Value>>;

    fn exec(&self, raw: String, args: Args) -> BoxFuture<Result<ExecResult>>;
}
pub trait ChunkExecutor: Executor {
    fn begin(&self) -> BoxFuture<Result<ArtisTx>>;
}

#[derive(Debug, Clone)]
pub struct Artis {
    c: Arc<Box<dyn ChunkExecutor>>,
}

impl From<Box<dyn ChunkExecutor>> for Artis {
    fn from(value: Box<dyn ChunkExecutor>) -> Self {
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
        T: Future<Output = Result<()>> + Send,
    {
        let rb = Arc::new(self.c.begin().await?);
        rb.chunk(func(Arc::clone(&rb))).await
    }
}

impl IntoArtis for Artis {
    fn fetch<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned,
    {
        let (raw, args) = i.into_raw(RawType::Fetch);
        let wait = self.c.query(raw.into(), args);
        Box::pin(async { Ok(decode(wait.await?)?) })
    }

    fn saving<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned,
    {
        let (raw, args) = i.into_raw(RawType::Saving);
        let wait = self.c.exec(raw.into(), args);
        Box::pin(async { Ok(decode(wait.await?.last_insert_id)?) })
    }

    fn update(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>> {
        let (raw, args) = i.into_raw(RawType::Update);
        let wait = self.c.exec(raw.into(), args);
        Box::pin(async { Ok(wait.await?.rows_affected) })
    }

    fn delete(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>> {
        let (raw, args) = i.into_raw(RawType::Delete);
        let wait = self.c.exec(raw.into(), args);
        Box::pin(async { Ok(wait.await?.rows_affected) })
    }

    fn exec(&self, raw: &str, args: crate::types::Args) -> BoxFuture<Result<ExecResult>> {
        let wait = self.c.exec(raw.into(), args);
        Box::pin(async { Ok(wait.await?) })
    }
}
