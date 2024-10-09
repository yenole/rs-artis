use serde::de::DeserializeOwned;

use crate::Value;

pub fn decode<T: DeserializeOwned>(v: Value) -> crate::Result<T> {
    let type_name = std::any::type_name::<T>();
    if type_name == std::any::type_name::<u64>() {
        return Ok(decode_i64(v)?);
    }
    Ok(rbatis::decode(v)?)
}
pub fn decode_i64<T: DeserializeOwned>(v: Value) -> crate::Result<T> {
    if !v.is_array() || v.is_empty() {
        return match rbs::from_value(rbs::Value::U64(0u64)) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string().into()),
        };
    }
    let v = v.as_array().unwrap().first().unwrap();
    if !v.is_map() || v.is_empty() {
        return match rbs::from_value(rbs::Value::U64(0u64)) {
            Ok(v) => Ok(v),
            Err(e) => Err(e.to_string().into()),
        };
    }
    match rbs::from_value_ref(&v[0]) {
        Ok(v) => Ok(v),
        Err(e) => Err(e.to_string().into()),
    }
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
