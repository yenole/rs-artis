mod artis;
mod artis_tx;
mod error;
mod into_delete;
mod into_saving;
mod into_select;
mod into_update;
mod into_where;

pub use artis::Artis;
pub use artis::IntoArtis;
pub use artis_tx::ArtisTx;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! rbox {
    ($v:expr) => {
        $crate::Error::from($v)
    };
    ($($arg:tt)*) => {
        $crate::Error::from(format!($($arg)*))
    };
}

#[macro_export]
macro_rules! raw {
    ($($arg:tt)*) => {
        Box::leak(format!($($arg)*).into_boxed_str())
    }
}

#[macro_export]
macro_rules! rbv {
    ($($k:tt: $v:expr),* $(,)?) => {
        rbs::value_map!($($k:$v ,)*)
    };

    ($tt:expr,$($k:tt: $v:expr),* $(,)?) => {
        {
            let v = rbs::to_value!($tt);
            let extend = rbs::value_map!($($k:$v ,)*);
            if let rbs::Value::Map(mut m) = v {
                extend.into_iter().for_each(|(k, v)| {m.insert(k, v);});
                m
            } else {
                rbs::value_map!($($k:$v ,)*)
            }
        }
    }
}

#[macro_export]
macro_rules! rtxblock {
    ($k:ident,$v:block) => {
        |$k: &$crate::ArtisTx| tokio::task::block_in_place(|| {
            futures::executor::block_on(async $v)
        })
    };
}
