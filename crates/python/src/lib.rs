use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use sdk_core as core;

// ── Top-level SDK ──────────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
pub struct QuickNodeSdk {
    #[pyo3(get)]
    admin_api: AdminApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl QuickNodeSdk {
    #[new]
    fn new(api_key: String) -> Self {
        let config = core::SdkConfig::new(api_key);
        Self {
            admin_api: AdminApiClient {
                inner: core::admin_api::AdminApiClient::new(config),
            },
        }
    }
}

// ── Sub-clients ────────────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct AdminApiClient {
    inner: core::admin_api::AdminApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl AdminApiClient {
    fn get_endpoints<'py>(
        &self,
        py: Python<'py>,
        params: &core::admin_api::GetEndpointsRequest,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = params.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_endpoints(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }
}

// ── Module ─────────────────────────────────────────────────────

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<QuickNodeSdk>()?;
    m.add_class::<AdminApiClient>()?;
    m.add_class::<core::admin_api::GetEndpointsRequest>()?;
    m.add_class::<core::admin_api::GetEndpointsResponse>()?;
    m.add_class::<core::admin_api::Endpoint>()?;
    m.add_class::<core::admin_api::EndpointTag>()?;
    Ok(())
}
