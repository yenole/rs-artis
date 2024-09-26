use std::sync::Arc;

use rbatis::{rbatis, RBatis};

use crate::{Artis, ArtisExecutor, BoxFuture, ExecResult, Executor, Result, TxExecutor, Value};

#[derive(Debug)]
pub struct InnerRBatis {
    rb: Arc<RBatis>,
}

impl From<Arc<RBatis>> for Artis {
    fn from(value: Arc<RBatis>) -> Self {
        let rb = Box::new(InnerRBatis { rb: value });
        (rb as Box<dyn ArtisExecutor>).into()
    }
}

impl From<RBatis> for crate::Artis {
    fn from(value: RBatis) -> Self {
        Arc::new(value).into()
    }
}

impl ArtisExecutor for InnerRBatis {}

impl Executor for InnerRBatis {
    fn query(&self, raw: String, args: Vec<Value>) -> BoxFuture<Result<Value>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.query(&raw, args).await?) })
    }

    fn exec(&self, raw: String, values: Vec<Value>) -> BoxFuture<Result<ExecResult>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move {
            let rst = rb.exec(&raw, values).await?;
            Ok(ExecResult {
                rows_affected: rst.rows_affected,
                last_insert_id: rst.last_insert_id,
            })
        })
    }
}

impl TxExecutor for InnerRBatis {
    fn begin(&self) -> BoxFuture<Result<crate::ArtisTx>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.acquire_begin().await?.into()) })
    }
}
