use my_sdk_core as core;
use napi::bindgen_prelude::*;
use napi_derive::napi;

/// Add two numbers
#[napi]
pub fn add(a: i64, b: i64) -> i64 {
    core::add(a, b)
}

/// Divide two numbers
#[napi]
pub fn divide(a: f64, b: f64) -> Result<f64> {
    core::divide(a, b).map_err(|e| Error::from_reason(e.to_string()))
}

/// Get random UUID with async http request
#[napi]
pub async fn get_external_uuid() -> Result<String> {
    core::get_external_uuid()
        .await
        .map_err(|e| Error::from_reason(e.to_string()))
}
