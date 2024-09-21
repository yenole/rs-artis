use serde::de::DeserializeOwned;

use crate::Value;

pub fn decode<T: DeserializeOwned>(v: Value) -> crate::Result<T> {
    let type_name = std::any::type_name::<T>();
    if type_name == std::any::type_name::<u64>() {
        if !v.is_array() {
            return Ok(rbs::from_value(v)?);
        }
        let list = v.as_array().unwrap();
        let v = if list.is_empty() {
            rbs::Value::I64(-1)
        } else {
            list[0].as_map().unwrap()[0].clone()
        };
        return Ok(rbs::from_value(v)?);
    }
    Ok(rbatis::decode(v)?)
}
