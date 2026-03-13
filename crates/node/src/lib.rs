use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

// ── Top-level SDK ──────────────────────────────────────────────

#[napi]
pub struct QuickNodeSdk {
    admin: core::admin::AdminApiClient,
}

#[napi]
impl QuickNodeSdk {
    #[napi(constructor)]
    pub fn new(api_key: String) -> Self {
        let config = core::SdkConfig::new(api_key);
        Self {
            admin: core::admin::AdminApiClient::new(config),
        }
    }

    #[napi(getter)]
    pub fn admin(&self) -> AdminApiClient {
        AdminApiClient {
            inner: self.admin.clone(),
        }
    }
}

// ── Sub-clients ───────────────────────────────────────

#[napi]
pub struct AdminApiClient {
    inner: core::admin::AdminApiClient,
}

#[napi]
impl AdminApiClient {
    #[napi]
    pub async fn get_endpoints(
        &self,
        params: Option<core::admin::GetEndpointsRequest>,
    ) -> Result<core::admin::GetEndpointsResponse> {
        let params = params.unwrap_or_default();
        self.inner
            .get_endpoints(&params)
            .await
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}
