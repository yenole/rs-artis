use std::future::Future;

pub type Args = Vec<crate::Value>;
pub type Columns = Vec<String>;
pub type BoxExecutor = Box<dyn crate::Executor>;
pub type BoxFuture<T> = std::pin::Pin<Box<dyn Future<Output = T>>>;

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
