use std::future::Future;

use rbatis::executor::RBatisTxExecutor;

use crate::{
    into_delete::IntoDelete, into_saving::IntoSaving, into_select::IntoSelect,
    into_update::IntoUpdate, rbox, IntoArtis,
};

#[derive(Debug)]
pub struct ArtisTx {
    rb: rbatis::executor::RBatisTxExecutor,
}

impl From<RBatisTxExecutor> for ArtisTx {
    fn from(value: RBatisTxExecutor) -> Self {
        Self { rb: value }
    }
}

impl ArtisTx {
    pub async fn commit(&mut self) -> crate::Result<()> {
        self.rb
            .commit()
            .await
            .map_err(|e| crate::Error::from(e.to_string()))
    }

    pub async fn rollback(&mut self) -> crate::Result<()> {
        self.rb
            .rollback()
            .await
            .map_err(|e| crate::Error::from(e.to_string()))
    }

    pub async fn exec<F>(&mut self, fun: F) -> crate::Result<()>
    where
        F: Future<Output = crate::Result<()>>,
    {
        let rst = fun.await;
        if rst.is_ok() {
            self.rb.commit().await?;
        } else {
            self.rb.rollback().await?;
        }
        rst
    }
}

impl IntoArtis for ArtisTx {
    fn list<T>(&self, i: impl IntoSelect) -> impl Future<Output = crate::Result<Vec<T>>>
    where
        T: serde::de::DeserializeOwned,
    {
        let c = async move {
            let (raw, list) = i.into_select();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            Ok(self.rb.query_decode::<Vec<T>>(raw, list).await?)
        };
        Box::pin(c)
    }

    fn saving(&self, i: impl IntoSaving) -> impl Future<Output = crate::Result<rbs::Value>> {
        let c = async move {
            let (raw, args) = i.into_saving();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.last_insert_id)
        };
        Box::pin(c)
    }

    fn update(&self, i: impl IntoUpdate) -> impl Future<Output = crate::Result<u64>> {
        let c = async move {
            let (raw, args) = i.into_update();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.rows_affected)
        };
        Box::pin(c)
    }

    fn delete(&self, i: impl IntoDelete) -> impl Future<Output = crate::Result<u64>> {
        let c = async move {
            let (raw, args) = i.into_raw();
            if raw.is_empty() {
                return Err(rbox!("数据异常"));
            }
            let rst = self.rb.exec(raw, args).await?;
            Ok(rst.rows_affected)
        };
        Box::pin(c)
    }
}
