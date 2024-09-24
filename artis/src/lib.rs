mod artis;
mod artis_tx;
mod decode;
mod error;
mod feature;
mod into_raw;
mod types;

pub mod migrator;

pub use artis::{Artis, Executor};
pub use artis_tx::ArtisTx;
pub use error::Error;
pub use feature::Value;
pub use into_raw::{IntoRaw, Raw};
pub use types::{BoxFuture, ExecResult, IntoArtis, RawType};

// pub use driver::*;

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
        rbs::to_value!($v)
    };
    ($($k:tt: $v:expr),* $(,)?) => {
        rbs::Value::Map(rbs::value_map!($($k:$v ,)*))
    };

    ($tt:expr,$($k:tt: $v:expr),* $(,)?) => {
        {
            let v = rbs::to_value!($tt);
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
