use serde::de::DeserializeOwned;

use crate::Value;

pub fn decode<T: DeserializeOwned>(v: Value) -> crate::Result<T> {
    Ok(rbatis::decode(v)?)
}

pub fn decode_pluck<T: DeserializeOwned>(v: Value, colume: &str) -> crate::Result<T> {
    if v.is_empty() {
        return Ok(rbatis::decode(v)?);
    }
    let mut list: Vec<Value> = vec![];
    for v in v.as_array().unwrap().iter() {
        if !v.is_map() {
            break;
        }
        let dict = v.as_map().unwrap();
        list.push(dict[colume].clone());
    }
    Ok(rbatis::decode(list.into())?)
}
