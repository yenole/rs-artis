mod artis;
mod artis_tx;
mod decode;
mod error;
mod into_raw;
mod types;

pub mod driver;
pub mod migrator;

#[cfg(feature = "log")]
pub mod unix;

pub use artis::{Artis, ArtisExecutor};
pub use artis_tx::{ArtisTx, ArtisTxExecutor};
pub use driver::Value;
pub use error::Error;
pub use into_raw::{IntoLimit, IntoRaw, IntoTable, Raw};
pub use types::{BoxFuture, ExecResult, IntoArtis, IntoChunk, RawType};

#[cfg(feature = "derive")]
pub use artis_derive::Artis;

#[cfg_attr(feature = "derive", macro_export)]
macro_rules! meta {
    ($($v:tt),* $(,)?) => {
        vec![$($v::migrator(),)*]
    };
}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! raw {
    ($($arg:tt)*) => {
       format!($($arg)*)
    }
}

#[macro_export]
macro_rules! map {
    {$($k:tt: $v:expr),* $(,)?} => {
        {
        let mut map = std::collections::HashMap::new();
        $(map.insert($k, $v);)*
        map
        }
    };
}

#[macro_export]
macro_rules! rbv {
    ($v:expr) => {
        rbs::value!($v)
    };
    ($($k:tt: $v:expr),* $(,)?) => {
        rbs::Value::Map(rbs::value_map!($($k:$v ,)*))
    };

    ($tt:expr,$($k:tt: $v:expr),* $(,)?) => {
        {
            let v = rbs::value!($tt);
            let extend = rbs::value_map!($($k:$v ,)*);
            if let rbs::Value::Map(mut m) = v {
                extend.into_iter().for_each(|(k, v)| {m.insert(k, v);});
                rbs::Value::Map(m)
            } else {
               rbs::Value::Map(rbs::value_map!($($k:$v ,)*))
            }
        }
    }
}
