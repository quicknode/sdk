use napi::bindgen_prelude::*;
use napi_derive::napi;
use sdk_core as core;

// ── Top-level SDK ──────────────────────────────────────────────

#[napi]
pub struct QuickNodeSdk {
    admin: AdminApiClient,
}

#[napi]
impl QuickNodeSdk {
    #[napi(constructor)]
    pub fn new(config: core::SdkFullConfig) -> Self {
        let sdk_config = core::SdkConfig::new(config);
        Self {
            admin: AdminApiClient {
                inner: core::admin::AdminApiClient::new(sdk_config),
            },
        }
    }

    #[napi(getter)]
    pub fn admin(&self) -> AdminApiClient {
        self.admin.clone()
    }

    #[napi(factory)]
    pub fn from_env() -> Result<Self> {
        core::QuickNodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
            })
            .map_err(|e| Error::from_reason(e.to_string()))
    }
}

// ── Sub-clients ───────────────────────────────────────

#[derive(Clone)]
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
