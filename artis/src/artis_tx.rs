use std::{future::Future, sync::Arc};

use rbatis::executor::RBatisTxExecutor;

use crate::{rbox, IntoArtis};

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
    fn list<T>(
        &self,
        i: impl crate::into_select::IntoSelect,
    ) -> impl std::future::Future<Output = crate::Result<Vec<T>>>
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

    fn saving(
        &self,
        i: impl crate::into_saving::IntoSaving,
    ) -> impl std::future::Future<Output = crate::Result<rbs::Value>> {
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

    fn update(
        &self,
        i: impl crate::into_update::IntoUpdate,
    ) -> impl std::future::Future<Output = crate::Result<u64>> {
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

    fn delete(
        &self,
        i: impl crate::into_delete::IntoDelete,
    ) -> impl std::future::Future<Output = crate::Result<u64>> {
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
