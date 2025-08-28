use std::sync::Arc;

use rbatis::{rbatis, RBatis};

use crate::{Artis, ArtisExecutor, BoxFuture, ExecResult, Result, Value};

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

impl ArtisExecutor for InnerRBatis {
    fn query(&self, raw: String, args: crate::types::Args) -> BoxFuture<'_, Result<Value>> {
        Box::pin(async move {
            #[cfg(feature = "log")]
            let elapsed = crate::unix::Elapsed::default();
            let rst = self.rb.query(&raw, args).await?;
            #[cfg(feature = "log")]
            elapsed.finish(&format!("{}", raw))?;
            Ok(rst)
        })
    }

    fn exec(&self, raw: String, args: crate::types::Args) -> BoxFuture<'_, Result<ExecResult>> {
        Box::pin(async move {
            #[cfg(feature = "log")]
            let elapsed = crate::unix::Elapsed::default();
            let rst = self.rb.exec(&raw, args).await?;
            #[cfg(feature = "log")]
            elapsed.finish(&format!("{}", raw))?;
            Ok(ExecResult {
                rows_affected: rst.rows_affected,
                last_insert_id: rst.last_insert_id,
            })
        })
    }

    fn begin(&self) -> BoxFuture<'_, Result<crate::ArtisTx>> {
        Box::pin(async move { Ok(self.rb.acquire_begin().await?.into()) })
    }
}
