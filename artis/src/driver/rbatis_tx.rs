use std::sync::Arc;

use rbatis::executor::RBatisTxExecutor;

use crate::{types::Args, ArtisTxExecutor, BoxFuture, ExecResult, Result, Value};

#[derive(Debug)]
pub struct InnerRBatisTx {
    rb: Arc<RBatisTxExecutor>,
}

impl From<RBatisTxExecutor> for crate::ArtisTx {
    fn from(value: RBatisTxExecutor) -> Self {
        (Box::new(InnerRBatisTx {
            rb: Arc::new(value),
        }) as Box<dyn ArtisTxExecutor>)
            .into()
    }
}

impl ArtisTxExecutor for InnerRBatisTx {
    fn query(&self, raw: String, args: Args) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move { Ok(self.rb.query(&raw, args).await?) })
    }

    fn exec(&self, raw: String, args: Args) -> BoxFuture<'_, Result<ExecResult>> {
        Box::pin(async move {
            let rst = self.rb.exec(&raw, args).await?;
            Ok(ExecResult {
                rows_affected: rst.rows_affected,
                last_insert_id: rst.last_insert_id,
            })
        })
    }

    fn commit(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(self.rb.conn.lock().await.commit().await?) })
    }

    fn rollback(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move { Ok(self.rb.conn.lock().await.rollback().await?) })
    }
}
