mod artis;
mod decode;
mod error;
mod feature;
mod into_artis;
mod into_raw;
mod types;

pub use artis::{Artis, Executor};
pub use error::Error;
pub use feature::rbatis::Value;
pub use into_artis::IntoArtis;
pub use into_raw::{IntoRaw, Raw};
pub use types::{BoxFuture, ExecResult, RawType};

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! raw {
    ($($arg:tt)*) => {
       format!($($arg)*)
    }
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
