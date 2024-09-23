use std::{future::Future, sync::Arc};

use serde::de::DeserializeOwned;

use crate::{
    decode::decode, types::Args, BoxFuture, ExecResult, Executor, IntoArtis, IntoRaw, RawType,
    Result,
};

pub trait TxExecutor: Executor {
    fn commit(&self) -> BoxFuture<Result<()>>;
    fn rollback(&self) -> BoxFuture<Result<()>>;
}

#[derive(Debug)]
pub struct ArtisTx {
    c: Arc<Box<dyn TxExecutor>>,
}

impl From<Box<dyn TxExecutor>> for ArtisTx {
    fn from(value: Box<dyn TxExecutor>) -> Self {
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

impl IntoArtis for ArtisTx {
    fn fetch<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned,
    {
        let (raw, args) = i.into_raw(RawType::Fetch);
        let wait = self.c.query(raw, args);
        Box::pin(async { Ok(decode(wait.await?)?) })
    }

    fn saving<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned,
    {
        let (raw, args) = i.into_raw(RawType::Saving);
        let wait = self.c.exec(raw, args);
        Box::pin(async { Ok(decode(wait.await?.last_insert_id)?) })
    }

    fn update(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>> {
        let (raw, args) = i.into_raw(RawType::Update);
        let wait = self.c.exec(raw, args);
        Box::pin(async { Ok(wait.await?.rows_affected) })
    }

    fn delete(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>> {
        let (raw, args) = i.into_raw(RawType::Delete);
        let wait = self.c.exec(raw, args);
        Box::pin(async { Ok(wait.await?.rows_affected) })
    }

    fn exec(&self, raw: &'static str, args: Args) -> BoxFuture<Result<ExecResult>> {
        let wait = self.c.exec(raw, args);
        Box::pin(async { Ok(wait.await?) })
    }
}