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
    #[allow(clippy::needless_pass_by_value)]
    fn new(config: core::SdkFullConfig) -> PyResult<Self> {
        let sdk_config = core::SdkConfig::new(&config)
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

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ShowEndpointResponse]"
    ))]
    fn show_endpoint<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .show_endpoint(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, label=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn update_endpoint<'py>(
        &self,
        py: Python<'py>,
        id: String,
        label: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateEndpointRequest { label };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_endpoint(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn archive_endpoint<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .archive_endpoint(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, UpdateEndpointStatusResponse]"
    ))]
    fn update_endpoint_status<'py>(
        &self,
        py: Python<'py>,
        id: String,
        status: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateEndpointStatusRequest { status };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_endpoint_status(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, label=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_tag<'py>(
        &self,
        py: Python<'py>,
        id: String,
        label: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateTagRequest { label };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_tag(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn delete_tag<'py>(
        &self,
        py: Python<'py>,
        id: String,
        tag_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_tag(&id, &tag_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (start_time=None, end_time=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetUsageResponse]"
    ))]
    fn get_usage<'py>(
        &self,
        py: Python<'py>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest { start_time, end_time };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_usage(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (start_time=None, end_time=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetUsageByEndpointResponse]"
    ))]
    fn get_usage_by_endpoint<'py>(
        &self,
        py: Python<'py>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest { start_time, end_time };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_usage_by_endpoint(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (start_time=None, end_time=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetUsageByMethodResponse]"
    ))]
    fn get_usage_by_method<'py>(
        &self,
        py: Python<'py>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest { start_time, end_time };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_usage_by_method(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (start_time=None, end_time=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetUsageByChainResponse]"
    ))]
    fn get_usage_by_chain<'py>(
        &self,
        py: Python<'py>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest { start_time, end_time };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_usage_by_chain(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, from_time, to_time, include_details=None, limit=None, next_at=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetEndpointLogsResponse]"
    ))]
    #[allow(clippy::too_many_arguments)]
    fn get_endpoint_logs<'py>(
        &self,
        py: Python<'py>,
        id: String,
        from_time: String,
        to_time: String,
        include_details: Option<bool>,
        limit: Option<i32>,
        next_at: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointLogsRequest {
            from: from_time,
            to: to_time,
            include_details,
            limit,
            next_at,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_endpoint_logs(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetLogDetailsResponse]"
    ))]
    fn get_log_details<'py>(
        &self,
        py: Python<'py>,
        id: String,
        request_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_log_details(&id, &request_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetSecurityOptionsResponse]"
    ))]
    fn get_security_options<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_security_options(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, tokens=None, referrers=None, jwts=None, ips=None, domain_masks=None, hsts=None, cors=None, request_filters=None, ip_custom_header=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, UpdateSecurityOptionsResponse]"
    ))]
    #[allow(clippy::too_many_arguments)]
    fn update_security_options<'py>(
        &self,
        py: Python<'py>,
        id: String,
        tokens: Option<String>,
        referrers: Option<String>,
        jwts: Option<String>,
        ips: Option<String>,
        domain_masks: Option<String>,
        hsts: Option<String>,
        cors: Option<String>,
        request_filters: Option<String>,
        ip_custom_header: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateSecurityOptionsRequest {
            options: core::admin::SecurityOptionsUpdate {
                tokens, referrers, jwts, ips, domain_masks, hsts, cors, request_filters, ip_custom_header,
            },
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_security_options(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_token<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_token(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteBoolResponse]"
    ))]
    fn delete_token<'py>(
        &self,
        py: Python<'py>,
        id: String,
        token_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_token(&id, &token_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, referrer=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_referrer<'py>(
        &self,
        py: Python<'py>,
        id: String,
        referrer: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateReferrerRequest { referrer };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_referrer(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteBoolResponse]"
    ))]
    fn delete_referrer<'py>(
        &self,
        py: Python<'py>,
        id: String,
        referrer_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_referrer(&id, &referrer_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, ip=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_ip<'py>(
        &self,
        py: Python<'py>,
        id: String,
        ip: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateIpRequest { ip };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_ip(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteBoolResponse]"
    ))]
    fn delete_ip<'py>(
        &self,
        py: Python<'py>,
        id: String,
        ip_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_ip(&id, &ip_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, domain_mask=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_domain_mask<'py>(
        &self,
        py: Python<'py>,
        id: String,
        domain_mask: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateDomainMaskRequest { domain_mask };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_domain_mask(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteBoolResponse]"
    ))]
    fn delete_domain_mask<'py>(
        &self,
        py: Python<'py>,
        id: String,
        domain_mask_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_domain_mask(&id, &domain_mask_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, public_key=None, kid=None, name=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn create_jwt<'py>(
        &self,
        py: Python<'py>,
        id: String,
        public_key: Option<String>,
        kid: Option<String>,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateJwtRequest { public_key, kid, name };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_jwt(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn delete_jwt<'py>(
        &self,
        py: Python<'py>,
        id: String,
        jwt_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_jwt(&id, &jwt_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, method=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, CreateRequestFilterResponse]"
    ))]
    fn create_request_filter<'py>(
        &self,
        py: Python<'py>,
        id: String,
        method: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateRequestFilterRequest { method };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_request_filter(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, request_filter_id, method=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn update_request_filter<'py>(
        &self,
        py: Python<'py>,
        id: String,
        request_filter_id: String,
        method: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateRequestFilterRequest { method };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_request_filter(&id, &request_filter_id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn delete_request_filter<'py>(
        &self,
        py: Python<'py>,
        id: String,
        request_filter_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_request_filter(&id, &request_filter_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn enable_multichain<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .enable_multichain(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn disable_multichain<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .disable_multichain(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, CreateOrUpdateIpCustomHeaderResponse]"
    ))]
    fn create_or_update_ip_custom_header<'py>(
        &self,
        py: Python<'py>,
        id: String,
        header_name: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateOrUpdateIpCustomHeaderRequest { header_name };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_or_update_ip_custom_header(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteBoolResponse]"
    ))]
    fn delete_ip_custom_header<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_ip_custom_header(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetMethodRateLimitsResponse]"
    ))]
    fn get_method_rate_limits<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_method_rate_limits(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, CreateMethodRateLimitResponse]"
    ))]
    fn create_method_rate_limit<'py>(
        &self,
        py: Python<'py>,
        id: String,
        interval: String,
        methods: Vec<String>,
        rate: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateMethodRateLimitRequest { interval, methods, rate };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_method_rate_limit(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, method_rate_limit_id, methods=None, status=None, rate=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, UpdateMethodRateLimitResponse]"
    ))]
    fn update_method_rate_limit<'py>(
        &self,
        py: Python<'py>,
        id: String,
        method_rate_limit_id: String,
        methods: Option<Vec<String>>,
        status: Option<String>,
        rate: Option<i32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateMethodRateLimitRequest { methods, status, rate };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_method_rate_limit(&id, &method_rate_limit_id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn delete_method_rate_limit<'py>(
        &self,
        py: Python<'py>,
        id: String,
        method_rate_limit_id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_method_rate_limit(&id, &method_rate_limit_id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, rps=None, rpm=None, rpd=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"
    ))]
    fn update_rate_limits<'py>(
        &self,
        py: Python<'py>,
        id: String,
        rps: Option<i32>,
        rpm: Option<i32>,
        rpd: Option<i32>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateRateLimitsRequest {
            rate_limits: core::admin::RateLimitSettings { rps, rpm, rpd },
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_rate_limits(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetEndpointMetricsResponse]"
    ))]
    fn get_endpoint_metrics<'py>(
        &self,
        py: Python<'py>,
        id: String,
        period: String,
        metric: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointMetricsRequest { period, metric };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_endpoint_metrics(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (period, metric, percentile=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetAccountMetricsResponse]"
    ))]
    fn get_account_metrics<'py>(
        &self,
        py: Python<'py>,
        period: String,
        metric: String,
        percentile: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetAccountMetricsRequest { period, metric, percentile };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_account_metrics(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListChainsResponse]"
    ))]
    fn list_chains<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_chains()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListInvoicesResponse]"
    ))]
    fn list_invoices<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_invoices()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListPaymentsResponse]"
    ))]
    fn list_payments<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_payments()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListTeamsResponse]"
    ))]
    fn list_teams<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_teams()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (name))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, CreateTeamResponse]"
    ))]
    fn create_team<'py>(&self, py: Python<'py>, name: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateTeamRequest { name };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_team(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetTeamResponse]"
    ))]
    fn get_team<'py>(&self, py: Python<'py>, id: i64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_team(id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteTeamResponse]"
    ))]
    fn delete_team<'py>(&self, py: Python<'py>, id: i64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_team(id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListTeamEndpointsResponse]"
    ))]
    fn list_team_endpoints<'py>(&self, py: Python<'py>, id: i64) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_team_endpoints(id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, endpoint_ids))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, UpdateTeamEndpointsResponse]"
    ))]
    fn update_team_endpoints<'py>(
        &self,
        py: Python<'py>,
        id: i64,
        endpoint_ids: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::UpdateTeamEndpointsRequest { endpoint_ids };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_team_endpoints(id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, email, full_name=None, role=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, InviteTeamMemberResponse]"
    ))]
    fn invite_team_member<'py>(
        &self,
        py: Python<'py>,
        id: i64,
        email: String,
        full_name: Option<String>,
        role: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::InviteTeamMemberRequest { email, full_name, role };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .invite_team_member(id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, user_id, destroy_user=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, RemoveTeamMemberResponse]"
    ))]
    fn remove_team_member<'py>(
        &self,
        py: Python<'py>,
        id: i64,
        user_id: i64,
        destroy_user: Option<bool>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::RemoveTeamMemberRequest { destroy_user };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .remove_team_member(id, user_id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, user_id))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ResendTeamInviteResponse]"
    ))]
    fn resend_team_invite<'py>(
        &self,
        py: Python<'py>,
        id: i64,
        user_id: i64,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .resend_team_invite(id, user_id)
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
    m.add_class::<core::admin::ShowEndpointResponse>()?;
    m.add_class::<core::admin::UpdateEndpointRequest>()?;
    m.add_class::<core::admin::UpdateEndpointStatusRequest>()?;
    m.add_class::<core::admin::UpdateEndpointStatusResponse>()?;
    m.add_class::<core::admin::CreateTagRequest>()?;
    m.add_class::<core::admin::GetUsageRequest>()?;
    m.add_class::<core::admin::UsageData>()?;
    m.add_class::<core::admin::GetUsageResponse>()?;
    m.add_class::<core::admin::EndpointUsage>()?;
    m.add_class::<core::admin::MethodUsage>()?;
    m.add_class::<core::admin::ChainUsage>()?;
    m.add_class::<core::admin::UsageByEndpointData>()?;
    m.add_class::<core::admin::GetUsageByEndpointResponse>()?;
    m.add_class::<core::admin::UsageByMethodData>()?;
    m.add_class::<core::admin::GetUsageByMethodResponse>()?;
    m.add_class::<core::admin::UsageByChainData>()?;
    m.add_class::<core::admin::GetUsageByChainResponse>()?;
    m.add_class::<core::admin::GetEndpointLogsRequest>()?;
    m.add_class::<core::admin::LogDetails>()?;
    m.add_class::<core::admin::EndpointLog>()?;
    m.add_class::<core::admin::GetEndpointLogsResponse>()?;
    m.add_class::<core::admin::GetLogDetailsResponse>()?;
    m.add_class::<core::admin::SecurityOption>()?;
    m.add_class::<core::admin::GetSecurityOptionsResponse>()?;
    m.add_class::<core::admin::SecurityOptionsUpdate>()?;
    m.add_class::<core::admin::UpdateSecurityOptionsRequest>()?;
    m.add_class::<core::admin::UpdateSecurityOptionsResponse>()?;
    m.add_class::<core::admin::CreateReferrerRequest>()?;
    m.add_class::<core::admin::CreateIpRequest>()?;
    m.add_class::<core::admin::CreateDomainMaskRequest>()?;
    m.add_class::<core::admin::CreateJwtRequest>()?;
    m.add_class::<core::admin::CreateRequestFilterRequest>()?;
    m.add_class::<core::admin::CreateRequestFilterData>()?;
    m.add_class::<core::admin::CreateRequestFilterResponse>()?;
    m.add_class::<core::admin::UpdateRequestFilterRequest>()?;
    m.add_class::<core::admin::CreateOrUpdateIpCustomHeaderRequest>()?;
    m.add_class::<core::admin::IpCustomHeaderData>()?;
    m.add_class::<core::admin::CreateOrUpdateIpCustomHeaderResponse>()?;
    m.add_class::<core::admin::DeleteBoolResponse>()?;
    m.add_class::<core::admin::MethodRateLimiter>()?;
    m.add_class::<core::admin::GetMethodRateLimitsData>()?;
    m.add_class::<core::admin::GetMethodRateLimitsResponse>()?;
    m.add_class::<core::admin::CreateMethodRateLimitRequest>()?;
    m.add_class::<core::admin::CreateMethodRateLimitResponse>()?;
    m.add_class::<core::admin::UpdateMethodRateLimitRequest>()?;
    m.add_class::<core::admin::UpdateMethodRateLimitResponse>()?;
    m.add_class::<core::admin::RateLimitSettings>()?;
    m.add_class::<core::admin::UpdateRateLimitsRequest>()?;
    m.add_class::<core::admin::GetEndpointMetricsRequest>()?;
    m.add_class::<core::admin::GetAccountMetricsRequest>()?;
    m.add_class::<core::admin::EndpointMetric>()?;
    m.add_class::<core::admin::GetEndpointMetricsResponse>()?;
    m.add_class::<core::admin::GetAccountMetricsResponse>()?;
    m.add_class::<core::admin::ChainNetwork>()?;
    m.add_class::<core::admin::Chain>()?;
    m.add_class::<core::admin::ListChainsResponse>()?;
    m.add_class::<core::admin::InvoiceLine>()?;
    m.add_class::<core::admin::Invoice>()?;
    m.add_class::<core::admin::ListInvoicesData>()?;
    m.add_class::<core::admin::ListInvoicesResponse>()?;
    m.add_class::<core::admin::Payment>()?;
    m.add_class::<core::admin::ListPaymentsData>()?;
    m.add_class::<core::admin::ListPaymentsResponse>()?;
    m.add_class::<core::admin::TeamUser>()?;
    m.add_class::<core::admin::TeamSummary>()?;
    m.add_class::<core::admin::TeamDetail>()?;
    m.add_class::<core::admin::ListTeamsResponse>()?;
    m.add_class::<core::admin::CreateTeamRequest>()?;
    m.add_class::<core::admin::CreateTeamData>()?;
    m.add_class::<core::admin::CreateTeamResponse>()?;
    m.add_class::<core::admin::GetTeamResponse>()?;
    m.add_class::<core::admin::DeleteTeamData>()?;
    m.add_class::<core::admin::DeleteTeamResponse>()?;
    m.add_class::<core::admin::TeamEndpoint>()?;
    m.add_class::<core::admin::ListTeamEndpointsResponse>()?;
    m.add_class::<core::admin::UpdateTeamEndpointsRequest>()?;
    m.add_class::<core::admin::UpdateTeamEndpointsData>()?;
    m.add_class::<core::admin::UpdateTeamEndpointsResponse>()?;
    m.add_class::<core::admin::InviteTeamMemberRequest>()?;
    m.add_class::<core::admin::InviteTeamMemberResponse>()?;
    m.add_class::<core::admin::RemoveTeamMemberRequest>()?;
    m.add_class::<core::admin::TeamMessageData>()?;
    m.add_class::<core::admin::RemoveTeamMemberResponse>()?;
    m.add_class::<core::admin::ResendTeamInviteResponse>()?;
    m.add_class::<core::HttpConfig>()?;
    m.add_class::<core::AdminConfig>()?;
    m.add_class::<core::SdkFullConfig>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
