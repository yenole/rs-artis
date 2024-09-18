use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Error {
    E(String),
}

impl From<&'static str> for Error {
    fn from(value: &'static str) -> Self {
        Self::E(value.to_string())
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::E(value)
    }
}

impl From<rbs::Error> for Error {
    fn from(value: rbs::Error) -> Self {
        Self::from(value.to_string())
    }
}

impl From<rbatis::Error> for Error {
    fn from(value: rbatis::Error) -> Self {
        Self::from(value.to_string())
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "E")
    }
}
