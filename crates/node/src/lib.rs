use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core::{httpbin::HttpbinClient as _HttpbinClient, init as _init};

#[napi]
pub fn init(api_key: String) {
    _init(api_key);
}

#[napi]
pub struct HttpbinClient {
    inner: _HttpbinClient,
}

#[napi]
impl HttpbinClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self {
            inner: _HttpbinClient::new(),
        }
    }

    #[napi]
    pub async fn get_uuid(&self) -> Result<String> {
        self.inner
            .get_uuid()
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
