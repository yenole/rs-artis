use std::{fmt::Debug, future::Future, sync::Arc};

use serde::de::DeserializeOwned;

use crate::{ArtisTx, IntoRaw, Result, Value};

pub type Args = Vec<crate::Value>;
pub type Columns = Vec<String>;
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait IntoChunk: Send + Sync {
    fn chunk<F, T, R>(&self, func: F) -> impl Future<Output = Result<R>>
    where
        F: FnOnce(Arc<ArtisTx>) -> T,
        T: Future<Output = Result<R>>;
}

pub trait IntoArtis: Send + Sync {
    fn fetch<T: DeserializeOwned>(&self, i: &dyn IntoRaw) -> impl Future<Output = Result<T>>;

    fn pluck<T: DeserializeOwned>(
        &self,
        i: &dyn IntoRaw,
        colume: &'static str,
    ) -> impl Future<Output = Result<T>>;

    fn saving(&self, i: &dyn IntoRaw) -> impl Future<Output = Result<Value>>;

    fn update(&self, i: &dyn IntoRaw) -> impl Future<Output = Result<u64>>;

    fn delete(&self, i: &dyn IntoRaw) -> impl Future<Output = Result<u64>>;

    fn query(&self, i: &dyn IntoRaw) -> impl Future<Output = Result<Value>>;

    fn exec(&self, raw: &str, args: Args) -> impl Future<Output = Result<ExecResult>>;
}

#[derive(Debug)]
pub struct ExecResult {
    pub rows_affected: u64,
    pub last_insert_id: crate::Value,
}

pub enum RawType {
    Fetch,
    Saving,
    Update,
    Delete,
}

impl RawType {
    pub fn is_fetch(&self) -> bool {
        if let RawType::Fetch = self {
            true
        } else {
            false
        }
    }

    pub fn is_saving(&self) -> bool {
        if let RawType::Saving = self {
            true
        } else {
            false
        }
    }

    pub fn is_update(&self) -> bool {
        if let RawType::Update = self {
            true
        } else {
            false
        }
    }

    pub fn is_delete(&self) -> bool {
        if let RawType::Delete = self {
            true
        } else {
            false
        }
    }

    pub fn is_single_prop(&self) -> bool {
        match self {
            Self::Update => true,
            Self::Delete => true,
            _ => false,
        }
    }
}
