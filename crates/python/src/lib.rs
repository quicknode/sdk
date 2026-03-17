use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::{
    define_stub_info_gatherer,
    derive::{gen_stub_pyclass, gen_stub_pymethods},
};
use sdk_core as core;

// ── Top-level SDK ──────────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
pub struct QuickNodeSdk {
    #[pyo3(get)]
    admin: AdminApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl QuickNodeSdk {
    #[new]
    fn new(config: core::SdkFullConfig) -> PyResult<Self> {
        let sdk_config = core::SdkConfig::new(config)
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            admin: AdminApiClient {
                inner: core::admin::AdminApiClient::new(sdk_config),
            },
        })
    }

    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        core::QuickNodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
            })
            .map_err(|e| PyValueError::new_err(e.to_string()))
    }
}

// ── Sub-clients ────────────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct AdminApiClient {
    inner: core::admin::AdminApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl AdminApiClient {
    #[pyo3(signature = (limit=None, offset=None, tag_ids=None, tag_labels=None))]
    // We are using pyo3_async_runtimes::tokio::future_into_py, so we need an override of the
    // return type generation because it will always return PyResult<Bound<'py, PyAny>>.
    // The async wrapper future_into_py returns a generic "any Python object" type because Python's
    // coroutine system doesn't carry information about what type the await will eventually produce.
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetEndpointsResponse]"
    ))]
    // Need to take arguments here so the client doesn't have to initalize a class for the param. If it was
    // params: GetEndpointsRequest, that class needs to be initalized and passed in as param
    fn get_endpoints<'py>(
        &self,
        py: Python<'py>,
        limit: Option<i32>,
        offset: Option<i32>,
        tag_ids: Option<Vec<i32>>,
        tag_labels: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointsRequest {
            limit,
            offset,
            tag_ids,
            tag_labels,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_endpoints(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (chain=None, network=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, CreateEndpointResponse]"
    ))]
    fn create_endpoint<'py>(
        &self,
        py: Python<'py>,
        chain: Option<String>,
        network: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateEndpointRequest { chain, network };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_endpoint(&params)
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
    m.add_class::<core::admin::GetEndpointsRequest>()?;
    m.add_class::<core::admin::GetEndpointsResponse>()?;
    m.add_class::<core::admin::Endpoint>()?;
    m.add_class::<core::admin::EndpointTag>()?;
    m.add_class::<core::admin::CreateEndpointRequest>()?;
    m.add_class::<core::admin::CreateEndpointResponse>()?;
    m.add_class::<core::admin::SingleEndpoint>()?;
    m.add_class::<core::admin::EndpointRateLimits>()?;
    m.add_class::<core::admin::EndpointSecurity>()?;
    m.add_class::<core::admin::EndpointSecurityOptions>()?;
    m.add_class::<core::admin::EndpointIpCustomHeaderOption>()?;
    m.add_class::<core::admin::EndpointToken>()?;
    m.add_class::<core::admin::EndpointJwt>()?;
    m.add_class::<core::admin::EndpointReferrer>()?;
    m.add_class::<core::admin::EndpointDomainMask>()?;
    m.add_class::<core::admin::EndpointIp>()?;
    m.add_class::<core::admin::EndpointRequestFilter>()?;
    m.add_class::<core::HttpConfig>()?;
    m.add_class::<core::AdminConfig>()?;
    m.add_class::<core::SdkFullConfig>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
