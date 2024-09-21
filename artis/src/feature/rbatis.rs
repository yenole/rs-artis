use std::sync::Arc;

use rbatis::RBatis;

use crate::{types::BoxExecutor, BoxFuture, ExecResult, Executor, Result};

pub type Value = rbs::Value;

impl From<rbs::Error> for crate::Error {
    fn from(value: rbs::Error) -> Self {
        Self::from(value.to_string())
    }
}

impl From<rbatis::Error> for crate::Error {
    fn from(value: rbatis::Error) -> Self {
        Self::from(value.to_string())
    }
}

#[derive(Debug)]
pub struct WrapRBatis {
    rb: Arc<Box<RBatis>>,
}

impl From<WrapRBatis> for crate::Artis {
    fn from(value: WrapRBatis) -> Self {
        (Box::new(value) as BoxExecutor).into()
    }
}

impl From<RBatis> for crate::Artis {
    fn from(value: RBatis) -> Self {
        WrapRBatis {
            rb: Arc::new(Box::new(value)),
        }
        .into()
    }
}

impl Executor for WrapRBatis {
    fn query(&self, raw: &'static str, args: Vec<Value>) -> BoxFuture<Result<Value>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.query(raw, args).await?) })
    }

    fn exec(&self, raw: &'static str, values: Vec<Value>) -> BoxFuture<Result<ExecResult>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move {
            let rst = rb.exec(raw, values).await?;
            Ok(ExecResult {
                rows_affected: rst.rows_affected,
                last_insert_id: rst.last_insert_id,
            })
        })
    }
}
