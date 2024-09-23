use std::{fmt::Debug, future::Future};

use serde::de::DeserializeOwned;

use crate::{IntoRaw, Result};

pub type Args = Vec<crate::Value>;
pub type Columns = Vec<String>;
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait IntoArtis: Debug + Send + Sync {
    fn fetch<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned;

    fn saving<T>(&self, i: &dyn IntoRaw) -> BoxFuture<Result<T>>
    where
        T: DeserializeOwned;

    fn update(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>>;

    fn delete(&self, i: &dyn IntoRaw) -> BoxFuture<Result<u64>>;

    fn exec(&self, raw: &'static str, args: Args) -> BoxFuture<Result<ExecResult>>;
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

    pub fn is_single_prop(&self) -> bool {
        match self {
            Self::Update => true,
            Self::Delete => true,
            _ => false,
        }
    }
}
