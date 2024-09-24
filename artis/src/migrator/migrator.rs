use std::{collections::HashMap, fmt::Debug};

use crate::{Artis, BoxFuture, IntoArtis, Result};

use super::{
    types::{Adjust, Mapping},
    ColumeMeta, IndexMeta, TableMeta,
};

pub trait ArtisMigrator: Sized {
    fn migrator() -> TableMeta;
}

pub trait DriverMigrator<'a>: Debug + Send + Sync + 'a {
    fn mapping(&self) -> Mapping;
    fn create_table(&self, meta: &TableMeta) -> Result<String>;
    fn colume_raw(&self, table: &str, t: Adjust, meta: &ColumeMeta) -> Result<String>;
    fn drop_index(&self, table: &str, meta: &IndexMeta) -> Result<String>;
    fn create_index(&self, table: &str, meta: &IndexMeta) -> Result<String>;
    fn fetch_tables(&self, rb: &'a Artis) -> BoxFuture<'a, Result<Vec<TableMeta>>>;
}

type AlterIndex = Vec<(Adjust, IndexMeta)>;
type AlterColume = Vec<(Adjust, ColumeMeta)>;

impl TableMeta {
    fn mapping(&mut self, dict: &Mapping) {
        for meta in self.columes.iter_mut() {
            meta.mapping(dict);
        }
    }

    fn patch(&self, meta: &TableMeta) -> Result<(AlterColume, AlterIndex)> {
        let mut columes: AlterColume = vec![];
        let dict: HashMap<_, _> = meta.columes.iter().map(|v| (v.name.clone(), v)).collect();
        for cmeta in self.columes.iter() {
            if !dict.contains_key(&cmeta.name) {
                columes.push((Adjust::Add, cmeta.clone()));
                continue;
            }
            if cmeta.ne(dict.get(&cmeta.name).unwrap()) {
                columes.push((Adjust::Alter, cmeta.clone()));
            }
        }

        let mut indexs: AlterIndex = vec![];
        let dict: HashMap<_, _> = meta.indexs.iter().map(|v| (v.column(), v)).collect();
        for imeta in self.indexs.iter() {
            let column = imeta.column();
            if !dict.contains_key(&column) {
                indexs.push((Adjust::Add, imeta.clone()));
                continue;
            }
            let meta = dict[&column];
            if imeta.ne(meta) {
                indexs.push((Adjust::Drop, meta.to_owned()));
                indexs.push((Adjust::Add, imeta.clone()));
            }
        }
        Ok((columes, indexs))
    }
}

impl ColumeMeta {
    pub fn mapping(&mut self, dict: &Mapping) {
        if !self.colume.starts_with(":") {
            return;
        }
        let key = self.colume.trim_start_matches(":");
        if !dict.contains_key(key) {
            panic!("mapping not found: {}", key);
        }
        self.colume = dict[key].into();
    }
}

impl<'a> Artis {
    pub async fn auto_migrate(
        &'a self,
        m: &dyn DriverMigrator<'a>,
        v: Vec<TableMeta>,
    ) -> Result<()> {
        let list = m.fetch_tables(self).await?;
        let dict: HashMap<_, _> = list.iter().map(|v| (&v.name, v)).collect();
        let mapping = m.mapping();
        let mut metas = v.clone();
        metas.iter_mut().for_each(|v| v.mapping(&mapping));
        for v in metas.iter() {
            if !dict.contains_key(&v.name) {
                let raw = m.create_table(v)?;
                let raw = Box::leak(raw.into_boxed_str());
                let _ = self.exec(raw, vec![]).await?;
                continue;
            }
            let (columes, indexs) = v.patch(dict.get(&v.name).unwrap())?;
            for (t, meta) in columes.iter() {
                let raw = m.colume_raw(&v.name, t.clone(), meta)?;
                let raw = Box::leak(raw.into_boxed_str());
                let _ = self.exec(raw, vec![]).await?;
            }
            for (t, meta) in indexs.iter() {
                let raw = match t {
                    Adjust::Add => m.create_index(&v.name, meta)?,
                    Adjust::Drop => m.drop_index(&v.name, meta)?,
                    _ => continue,
                };
                let raw = Box::leak(raw.into_boxed_str());
                let _ = self.exec(raw, vec![]).await?;
            }
        }
        Ok(())
    }
}
