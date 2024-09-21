use std::u64;
use std::{fmt::Debug, sync::Arc};

use serde::de::DeserializeOwned;

use crate::decode::decode;
use crate::into_artis::IntoArtis;
use crate::types::{BoxExecutor, ExecResult, RawType};
use crate::IntoRaw;
use crate::{BoxFuture, Result, Value};

pub trait Executor: Debug + Send + Sync {
    fn query(&self, raw: &'static str, args: Vec<Value>) -> BoxFuture<Result<Value>>;

    fn exec(&self, raw: &'static str, args: Vec<Value>) -> BoxFuture<Result<ExecResult>>;
}

#[derive(Debug, Clone)]
pub struct Artis {
    c: Arc<Box<dyn Executor>>,
}

impl From<BoxExecutor> for Artis {
    fn from(value: BoxExecutor) -> Self {
        Self { c: Arc::new(value) }
    }
}

impl IntoArtis for Artis {
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

    fn exec(&self, raw: &'static str, args: crate::types::Args) -> BoxFuture<Result<ExecResult>> {
        let wait = self.c.exec(raw, args);
        Box::pin(async { Ok(wait.await?) })
    }
}
