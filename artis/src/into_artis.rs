use serde::de::DeserializeOwned;

use crate::{
    types::{Args, ExecResult},
    BoxFuture, IntoRaw, Result,
};

pub trait IntoArtis {
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
