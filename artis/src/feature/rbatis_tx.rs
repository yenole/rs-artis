use std::sync::Arc;

use rbatis::executor::RBatisTxExecutor;

use crate::{artis_tx::TxExecutor, types::Args, BoxFuture, ExecResult, Executor, Result, Value};

#[derive(Debug)]
pub struct InnerRBatisTx {
    rb: Arc<RBatisTxExecutor>,
}

impl From<RBatisTxExecutor> for crate::ArtisTx {
    fn from(value: RBatisTxExecutor) -> Self {
        (Box::new(InnerRBatisTx {
            rb: Arc::new(value),
        }) as Box<dyn TxExecutor>)
            .into()
    }
}

impl Executor for InnerRBatisTx {
    fn query(&self, raw: &'static str, args: Args) -> BoxFuture<Result<Value>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.query(raw, args).await?) })
    }

    fn exec(&self, raw: &'static str, values: Args) -> BoxFuture<Result<ExecResult>> {
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
impl TxExecutor for InnerRBatisTx {
    fn commit(&self) -> crate::BoxFuture<crate::Result<()>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.conn.lock().await.commit().await?) })
    }

    fn rollback(&self) -> crate::BoxFuture<crate::Result<()>> {
        let rb = Arc::clone(&self.rb);
        Box::pin(async move { Ok(rb.conn.lock().await.rollback().await?) })
    }
}
