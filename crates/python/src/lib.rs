use pyo3::{exceptions::PyValueError, prelude::*};
use pyo3_stub_gen::{
    define_stub_info_gatherer,
    derive::{gen_stub_pyclass, gen_stub_pymethods},
};
use sdk_core as core;

mod streams_destination;

// ── Top-level SDK ──────────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
pub struct QuickNodeSdk {
    #[pyo3(get)]
    admin: AdminApiClient,
    #[pyo3(get)]
    streams: StreamsApiClient,
    #[pyo3(get)]
    webhooks: WebhooksApiClient,
    #[pyo3(get)]
    kvstore: KvStoreApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl QuickNodeSdk {
    #[new]
    #[allow(clippy::needless_pass_by_value)]
    fn new(config: core::SdkFullConfig) -> PyResult<Self> {
        let sdk_config =
            core::SdkConfig::new(&config).map_err(|e| PyValueError::new_err(e.to_string()))?;
        Ok(Self {
            admin: AdminApiClient {
                inner: core::admin::AdminApiClient::new(sdk_config.clone()),
            },
            webhooks: WebhooksApiClient {
                inner: core::webhooks::WebhooksApiClient::new(sdk_config.clone()),
            },
            streams: StreamsApiClient {
                inner: core::streams::StreamsApiClient::new(sdk_config.clone()),
            },
            kvstore: KvStoreApiClient {
                inner: core::kvstore::KvStoreApiClient::new(sdk_config),
            },
        })
    }

    #[staticmethod]
    fn from_env() -> PyResult<Self> {
        core::QuickNodeSdk::from_env()
            .map(|sdk| Self {
                admin: AdminApiClient { inner: sdk.admin },
                streams: StreamsApiClient { inner: sdk.streams },
                webhooks: WebhooksApiClient {
                    inner: sdk.webhooks,
                },
                kvstore: KvStoreApiClient { inner: sdk.kvstore },
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
    #[pyo3(signature = (
        limit=None,
        offset=None,
        search=None,
        sort_by=None,
        sort_direction=None,
        networks=None,
        statuses=None,
        labels=None,
        dedicated=None,
        is_flat_rate=None,
        tag_ids=None,
        tag_labels=None,
    ))]
    // We are using pyo3_async_runtimes::tokio::future_into_py, so we need an override of the
    // return type generation because it will always return PyResult<Bound<'py, PyAny>>.
    // The async wrapper future_into_py returns a generic "any Python object" type because Python's
    // coroutine system doesn't carry information about what type the await will eventually produce.
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetEndpointsResponse]"
    ))]
    // Need to take arguments here so the client doesn't have to initalize a class for the param. If it was
    // params: GetEndpointsRequest, that class needs to be initalized and passed in as param
    #[allow(clippy::too_many_arguments)]
    fn get_endpoints<'py>(
        &self,
        py: Python<'py>,
        limit: Option<i32>,
        offset: Option<i32>,
        search: Option<String>,
        sort_by: Option<String>,
        sort_direction: Option<String>,
        networks: Option<Vec<String>>,
        statuses: Option<Vec<String>>,
        labels: Option<Vec<String>>,
        dedicated: Option<bool>,
        is_flat_rate: Option<bool>,
        tag_ids: Option<Vec<i32>>,
        tag_labels: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetEndpointsRequest {
            limit,
            offset,
            search,
            sort_by,
            sort_direction,
            networks,
            statuses,
            labels,
            dedicated,
            is_flat_rate,
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
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
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
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
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
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
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
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
                tokens,
                referrers,
                jwts,
                ips,
                domain_masks,
                hsts,
                cors,
                request_filters,
                ip_custom_header,
            },
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_security_options(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn create_jwt<'py>(
        &self,
        py: Python<'py>,
        id: String,
        public_key: Option<String>,
        kid: Option<String>,
        name: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::CreateJwtRequest {
            public_key,
            kid,
            name,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_jwt(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn enable_multichain<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .enable_multichain(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn disable_multichain<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
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
        let params = core::admin::CreateMethodRateLimitRequest {
            interval,
            methods,
            rate,
        };
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
        let params = core::admin::UpdateMethodRateLimitRequest {
            methods,
            status,
            rate,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_method_rate_limit(&id, &method_rate_limit_id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
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
        let params = core::admin::GetAccountMetricsRequest {
            period,
            metric,
            percentile,
        };
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
        let params = core::admin::InviteTeamMemberRequest {
            email,
            full_name,
            role,
        };
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

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, BulkUpdateEndpointStatusResponse]"
    ))]
    fn bulk_update_endpoint_status<'py>(
        &self,
        py: Python<'py>,
        ids: Vec<String>,
        status: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::BulkUpdateEndpointStatusRequest { ids, status };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .bulk_update_endpoint_status(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, BulkAddTagResponse]"
    ))]
    fn bulk_add_tag<'py>(
        &self,
        py: Python<'py>,
        ids: Vec<String>,
        label: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::BulkAddTagRequest { ids, label };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .bulk_add_tag(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, BulkRemoveTagResponse]"
    ))]
    fn bulk_remove_tag<'py>(
        &self,
        py: Python<'py>,
        ids: Vec<String>,
        tag_id: i32,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::BulkRemoveTagRequest { ids, tag_id };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .bulk_remove_tag(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListTagsResponse]"
    ))]
    fn list_tags<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_tags()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, RenameTagResponse]"
    ))]
    fn rename_tag<'py>(
        &self,
        py: Python<'py>,
        id: i32,
        label: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::RenameTagRequest { label };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .rename_tag(id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, DeleteAccountTagResponse]"
    ))]
    fn delete_account_tag<'py>(&self, py: Python<'py>, id: i32) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_account_tag(id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (start_time=None, end_time=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetUsageByTagResponse]"
    ))]
    fn get_usage_by_tag<'py>(
        &self,
        py: Python<'py>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::admin::GetUsageRequest {
            start_time,
            end_time,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_usage_by_tag(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetEndpointSecurityResponse]"
    ))]
    fn get_endpoint_security<'py>(
        &self,
        py: Python<'py>,
        id: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_endpoint_security(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }
}

// ── StreamsApiClient ───────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct StreamsApiClient {
    inner: core::streams::StreamsApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl StreamsApiClient {
    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        name,
        network,
        dataset,
        region,
        start_range,
        end_range,
        destination_attributes,
        plan,
        threshold_fetch_buffer,
        dataset_batch_size=None,
        max_batch_size=None,
        max_buffer_range_size=None,
        max_buffer_processing_workers=None,
        keep_distance_from_tip=None,
        filter_function=None,
        filter_language=None,
        include_stream_metadata=None,
        product_type=None,
        status=None,
        notification_email=None,
        charge_min_cap=None,
        fix_block_reorgs=None,
        elastic_batch_enabled=None,
        extra_destinations=None
    ))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Stream]"
    ))]
    fn create_stream<'py>(
        &self,
        py: Python<'py>,
        name: String,
        network: String,
        dataset: String,
        region: String,
        start_range: i64,
        end_range: i64,
        destination_attributes: &Bound<'py, PyAny>,
        plan: String,
        threshold_fetch_buffer: i64,
        dataset_batch_size: Option<i64>,
        max_batch_size: Option<i64>,
        max_buffer_range_size: Option<i64>,
        max_buffer_processing_workers: Option<i64>,
        keep_distance_from_tip: Option<i64>,
        filter_function: Option<String>,
        filter_language: Option<String>,
        include_stream_metadata: Option<String>,
        product_type: Option<String>,
        status: Option<String>,
        notification_email: Option<String>,
        charge_min_cap: Option<i32>,
        fix_block_reorgs: Option<i32>,
        elastic_batch_enabled: Option<bool>,
        extra_destinations: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let destination_attributes =
            streams_destination::extract_destination_attributes(destination_attributes)?;
        let extra_destinations =
            streams_destination::extract_extra_destinations(extra_destinations)?;
        let dataset = serde_json::from_value::<core::streams::StreamDataset>(
            serde_json::Value::String(dataset),
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let region = serde_json::from_value::<core::streams::StreamRegion>(
            serde_json::Value::String(region),
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let filter_language = filter_language
            .map(|s| {
                serde_json::from_value::<core::streams::FilterLanguage>(serde_json::Value::String(
                    s,
                ))
                .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let include_stream_metadata = include_stream_metadata
            .map(|s| {
                serde_json::from_value::<core::streams::StreamMetadataLocation>(
                    serde_json::Value::String(s),
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let product_type = product_type
            .map(|s| {
                serde_json::from_value::<core::streams::ProductType>(serde_json::Value::String(s))
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let status = status
            .map(|s| {
                serde_json::from_value::<core::streams::StreamStatus>(serde_json::Value::String(s))
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let params = core::streams::CreateStreamParams {
                name,
                network,
                dataset,
                region,
                start_range,
                end_range,
                destination_attributes,
                plan,
                threshold_fetch_buffer,
                dataset_batch_size,
                max_batch_size,
                max_buffer_range_size,
                max_buffer_processing_workers,
                keep_distance_from_tip,
                filter_function,
                filter_language,
                address_book_config: None,
                include_stream_metadata,
                product_type,
                status,
                notification_email,
                charge_min_cap,
                fix_block_reorgs,
                elastic_batch_enabled,
                extra_destinations,
            };
            let stream = client
                .create_stream(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Python::attach(|py| streams_destination::PyStream::from_core(stream, py))
        })
    }

    #[pyo3(signature = (stream_type=None, offset=None, limit=None, order_by=None, order_direction=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListStreamsResponse]"
    ))]
    fn list_streams<'py>(
        &self,
        py: Python<'py>,
        stream_type: Option<String>,
        offset: Option<i64>,
        limit: Option<i64>,
        order_by: Option<String>,
        order_direction: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let params = core::streams::ListStreamsParams {
                stream_type,
                offset,
                limit,
                order_by,
                order_direction,
            };
            let resp = client
                .list_streams(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Python::attach(|py| streams_destination::PyListStreamsResponse::from_core(resp, py))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_all_streams<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_all_streams()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Stream]"
    ))]
    fn get_stream<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let stream = client
                .get_stream(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Python::attach(|py| streams_destination::PyStream::from_core(stream, py))
        })
    }

    #[allow(clippy::too_many_arguments)]
    #[pyo3(signature = (
        id,
        name=None,
        network=None,
        dataset=None,
        region=None,
        start_range=None,
        end_range=None,
        destination_attributes=None,
        plan=None,
        threshold_fetch_buffer=None,
        dataset_batch_size=None,
        max_batch_size=None,
        max_buffer_range_size=None,
        max_buffer_processing_workers=None,
        keep_distance_from_tip=None,
        filter_function=None,
        filter_language=None,
        include_stream_metadata=None,
        notification_email=None,
        charge_min_cap=None,
        fix_block_reorgs=None,
        elastic_batch_enabled=None,
        status=None,
        memo=None,
        extra_destinations=None,
    ))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Stream]"
    ))]
    fn update_stream<'py>(
        &self,
        py: Python<'py>,
        id: String,
        name: Option<String>,
        network: Option<String>,
        dataset: Option<String>,
        region: Option<String>,
        start_range: Option<i64>,
        end_range: Option<i64>,
        destination_attributes: Option<Bound<'py, PyAny>>,
        plan: Option<String>,
        threshold_fetch_buffer: Option<i64>,
        dataset_batch_size: Option<i64>,
        max_batch_size: Option<i64>,
        max_buffer_range_size: Option<i64>,
        max_buffer_processing_workers: Option<i64>,
        keep_distance_from_tip: Option<i64>,
        filter_function: Option<String>,
        filter_language: Option<String>,
        include_stream_metadata: Option<String>,
        notification_email: Option<String>,
        charge_min_cap: Option<i32>,
        fix_block_reorgs: Option<i32>,
        elastic_batch_enabled: Option<bool>,
        status: Option<String>,
        memo: Option<String>,
        extra_destinations: Option<Bound<'py, PyAny>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let destination_attributes = destination_attributes
            .map(|obj| streams_destination::extract_destination_attributes(&obj))
            .transpose()?;
        let extra_destinations =
            streams_destination::extract_extra_destinations(extra_destinations)?;
        let dataset = dataset
            .map(|s| {
                serde_json::from_value::<core::streams::StreamDataset>(serde_json::Value::String(s))
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let region = region
            .map(|s| {
                serde_json::from_value::<core::streams::StreamRegion>(serde_json::Value::String(s))
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let filter_language = filter_language
            .map(|s| {
                serde_json::from_value::<core::streams::FilterLanguage>(serde_json::Value::String(
                    s,
                ))
                .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let include_stream_metadata = include_stream_metadata
            .map(|s| {
                serde_json::from_value::<core::streams::StreamMetadataLocation>(
                    serde_json::Value::String(s),
                )
                .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        let status = status
            .map(|s| {
                serde_json::from_value::<core::streams::StreamStatus>(serde_json::Value::String(s))
                    .map_err(|e| PyValueError::new_err(e.to_string()))
            })
            .transpose()?;
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let params = core::streams::UpdateStreamParams {
                name,
                network,
                dataset,
                region,
                start_range,
                end_range,
                destination_attributes,
                plan,
                threshold_fetch_buffer,
                dataset_batch_size,
                max_batch_size,
                max_buffer_range_size,
                max_buffer_processing_workers,
                keep_distance_from_tip,
                filter_function,
                filter_language,
                address_book_config: None,
                include_stream_metadata,
                notification_email,
                charge_min_cap,
                fix_block_reorgs,
                elastic_batch_enabled,
                status,
                memo,
                extra_destinations,
            };
            let stream = client
                .update_stream(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))?;
            Python::attach(|py| streams_destination::PyStream::from_core(stream, py))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_stream<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_stream(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn activate_stream<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .activate_stream(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn pause_stream<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .pause_stream(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (network, dataset, block, filter_function=None, filter_language=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, TestFilterResponse]"
    ))]
    fn test_filter<'py>(
        &self,
        py: Python<'py>,
        network: String,
        dataset: String,
        block: String,
        filter_function: Option<String>,
        filter_language: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            let dataset = serde_json::from_value::<core::streams::StreamDataset>(
                serde_json::Value::String(dataset),
            )
            .map_err(|e| PyValueError::new_err(e.to_string()))?;
            let filter_language = filter_language
                .map(|s| {
                    serde_json::from_value::<core::streams::FilterLanguage>(
                        serde_json::Value::String(s),
                    )
                    .map_err(|e| PyValueError::new_err(e.to_string()))
                })
                .transpose()?;
            let params = core::streams::TestFilterParams {
                network,
                dataset,
                block,
                filter_function,
                filter_language,
                address_book_config: None,
            };
            client
                .test_filter(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (stream_type=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, EnabledCountResponse]"
    ))]
    fn get_enabled_count<'py>(
        &self,
        py: Python<'py>,
        stream_type: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_enabled_count(stream_type.as_deref())
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }
}

// ── WebhooksApiClient ──────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct WebhooksApiClient {
    inner: core::webhooks::WebhooksApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl WebhooksApiClient {
    #[pyo3(signature = (limit=None, offset=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListWebhooksResponse]"
    ))]
    fn list_webhooks<'py>(
        &self,
        py: Python<'py>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::webhooks::GetWebhooksParams { limit, offset };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_webhooks(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_all_webhooks<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_all_webhooks()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Webhook]"
    ))]
    fn get_webhook<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_webhook(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (id, name=None, notification_email=None, destination_attributes=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Webhook]"
    ))]
    fn update_webhook<'py>(
        &self,
        py: Python<'py>,
        id: String,
        name: Option<String>,
        notification_email: Option<String>,
        destination_attributes: Option<core::webhooks::WebhookDestinationAttributes>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::webhooks::UpdateWebhookParams {
            name,
            notification_email,
            destination_attributes,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_webhook(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_webhook<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_webhook(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn pause_webhook<'py>(&self, py: Python<'py>, id: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .pause_webhook(&id)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn activate_webhook<'py>(
        &self,
        py: Python<'py>,
        id: String,
        start_from: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let start_from = serde_json::from_value::<core::webhooks::WebhookStartFrom>(
            serde_json::Value::String(start_from),
        )
        .map_err(|e| PyValueError::new_err(e.to_string()))?;
        let params = core::webhooks::ActivateWebhookParams { start_from };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .activate_webhook(&id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, WebhookEnabledCountResponse]"
    ))]
    fn get_enabled_count<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_enabled_count()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (name, network, destination_attributes, template_args, notification_email=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Webhook]"
    ))]
    fn create_webhook_from_template<'py>(
        &self,
        py: Python<'py>,
        name: String,
        network: String,
        destination_attributes: core::webhooks::WebhookDestinationAttributes,
        template_args: core::webhooks::TemplateArgs,
        notification_email: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::webhooks::CreateWebhookFromTemplateParams {
            name,
            network,
            notification_email,
            destination_attributes,
            template_args,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_webhook_from_template(&params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (webhook_id, template_args, name=None, notification_email=None, destination_attributes=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, Webhook]"
    ))]
    fn update_webhook_template<'py>(
        &self,
        py: Python<'py>,
        webhook_id: String,
        template_args: core::webhooks::TemplateArgs,
        name: Option<String>,
        notification_email: Option<String>,
        destination_attributes: Option<core::webhooks::WebhookDestinationAttributes>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        let params = core::webhooks::UpdateWebhookTemplateParams {
            name,
            notification_email,
            destination_attributes,
            template_args,
        };
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_webhook_template(&webhook_id, &params)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }
}

// ── KvStoreApiClient ───────────────────────────────────────────

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct KvStoreApiClient {
    inner: core::kvstore::KvStoreApiClient,
}

#[gen_stub_pymethods]
#[pymethods]
impl KvStoreApiClient {
    #[pyo3(signature = (key, value))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn create_set<'py>(
        &self,
        py: Python<'py>,
        key: String,
        value: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_set(&core::kvstore::CreateSetParams { key, value })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (limit=None, cursor=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetSetsResponse]"
    ))]
    fn get_sets<'py>(
        &self,
        py: Python<'py>,
        limit: Option<i64>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_sets(&core::kvstore::GetSetsParams { limit, cursor })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetSetResponse]"
    ))]
    fn get_set<'py>(&self, py: Python<'py>, key: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_set(&key)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (add_sets=None, delete_sets=None))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn bulk_sets<'py>(
        &self,
        py: Python<'py>,
        add_sets: Option<std::collections::HashMap<String, String>>,
        delete_sets: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .bulk_sets(&core::kvstore::BulkSetsParams {
                    add_sets,
                    delete_sets,
                })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_set<'py>(&self, py: Python<'py>, key: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_set(&key)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, items))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn create_list<'py>(
        &self,
        py: Python<'py>,
        key: String,
        items: Vec<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .create_list(&core::kvstore::CreateListParams { key, items })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (limit=None, cursor=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetListsResponse]"
    ))]
    fn get_lists<'py>(
        &self,
        py: Python<'py>,
        limit: Option<i64>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_lists(&core::kvstore::GetListsParams { limit, cursor })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, limit=None, cursor=None))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, GetListResponse]"
    ))]
    fn get_list<'py>(
        &self,
        py: Python<'py>,
        key: String,
        limit: Option<i64>,
        cursor: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_list(&key, &core::kvstore::GetListParams { limit, cursor })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, add_items=None, remove_items=None))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn update_list<'py>(
        &self,
        py: Python<'py>,
        key: String,
        add_items: Option<Vec<String>>,
        remove_items: Option<Vec<String>>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .update_list(
                    &key,
                    &core::kvstore::UpdateListParams {
                        add_items,
                        remove_items,
                    },
                )
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, item))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn add_list_item<'py>(
        &self,
        py: Python<'py>,
        key: String,
        item: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .add_list_item(&key, &core::kvstore::AddListItemParams { item })
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, item))]
    #[gen_stub(override_return_type(
        type_repr = "typing.Coroutine[typing.Any, typing.Any, ListContainsItemResponse]"
    ))]
    fn list_contains_item<'py>(
        &self,
        py: Python<'py>,
        key: String,
        item: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .list_contains_item(&key, &item)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key, item))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_list_item<'py>(
        &self,
        py: Python<'py>,
        key: String,
        item: String,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_list_item(&key, &item)
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }

    #[pyo3(signature = (key))]
    #[gen_stub(override_return_type(type_repr = "typing.Coroutine[typing.Any, typing.Any, None]"))]
    fn delete_list<'py>(&self, py: Python<'py>, key: String) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .delete_list(&key)
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
    m.add_class::<core::admin::Pagination>()?;
    m.add_class::<core::admin::GetEndpointSecurityResponse>()?;
    m.add_class::<core::admin::BulkOperationResult>()?;
    m.add_class::<core::admin::BulkUpdateEndpointStatusRequest>()?;
    m.add_class::<core::admin::BulkUpdateEndpointStatusData>()?;
    m.add_class::<core::admin::BulkUpdateEndpointStatusResponse>()?;
    m.add_class::<core::admin::BulkTag>()?;
    m.add_class::<core::admin::BulkAddTagRequest>()?;
    m.add_class::<core::admin::BulkAddTagData>()?;
    m.add_class::<core::admin::BulkAddTagResponse>()?;
    m.add_class::<core::admin::BulkRemoveTagRequest>()?;
    m.add_class::<core::admin::BulkRemoveTagData>()?;
    m.add_class::<core::admin::BulkRemoveTagResponse>()?;
    m.add_class::<core::admin::AccountTag>()?;
    m.add_class::<core::admin::ListTagsData>()?;
    m.add_class::<core::admin::ListTagsResponse>()?;
    m.add_class::<core::admin::RenameTagRequest>()?;
    m.add_class::<core::admin::RenameTagResponse>()?;
    m.add_class::<core::admin::DeleteAccountTagData>()?;
    m.add_class::<core::admin::DeleteAccountTagResponse>()?;
    m.add_class::<core::admin::TagUsage>()?;
    m.add_class::<core::admin::UsageByTagData>()?;
    m.add_class::<core::admin::GetUsageByTagResponse>()?;
    m.add_class::<core::HttpConfig>()?;
    m.add_class::<core::AdminConfig>()?;
    m.add_class::<core::StreamsConfig>()?;
    m.add_class::<core::WebhooksConfig>()?;
    m.add_class::<core::KvStoreConfig>()?;
    m.add_class::<core::SdkFullConfig>()?;
    m.add_class::<StreamsApiClient>()?;
    m.add_class::<streams_destination::PyStream>()?;
    m.add_class::<streams_destination::PyListStreamsResponse>()?;
    m.add_class::<streams_destination::StreamWebhookDestination>()?;
    m.add_class::<streams_destination::StreamS3Destination>()?;
    m.add_class::<streams_destination::StreamAzureDestination>()?;
    m.add_class::<streams_destination::StreamPostgresDestination>()?;
    m.add_class::<streams_destination::StreamMysqlDestination>()?;
    m.add_class::<streams_destination::StreamMongoDestination>()?;
    m.add_class::<streams_destination::StreamClickhouseDestination>()?;
    m.add_class::<streams_destination::StreamSnowflakeDestination>()?;
    m.add_class::<streams_destination::StreamKafkaDestination>()?;
    m.add_class::<streams_destination::StreamRedisDestination>()?;
    m.add_class::<core::streams::AddressBookConfig>()?;
    m.add_class::<core::streams::WebhookAttributes>()?;
    m.add_class::<core::streams::S3Attributes>()?;
    m.add_class::<core::streams::AzureAttributes>()?;
    m.add_class::<core::streams::PostgresAttributes>()?;
    m.add_class::<core::streams::MysqlAttributes>()?;
    m.add_class::<core::streams::MongoAttributes>()?;
    m.add_class::<core::streams::ClickhouseAttributes>()?;
    m.add_class::<core::streams::SnowflakeAttributes>()?;
    m.add_class::<core::streams::KafkaAttributes>()?;
    m.add_class::<core::streams::RedisAttributes>()?;
    m.add_class::<core::streams::PageInfo>()?;
    m.add_class::<core::streams::TestFilterResponse>()?;
    m.add_class::<core::streams::EnabledCountResponse>()?;
    m.add_class::<WebhooksApiClient>()?;
    m.add_class::<core::webhooks::TemplateArgs>()?;
    m.add_class::<core::webhooks::WebhookDestinationAttributes>()?;
    m.add_class::<core::webhooks::Webhook>()?;
    m.add_class::<core::webhooks::ListWebhooksResponse>()?;
    m.add_class::<core::webhooks::WebhookEnabledCountResponse>()?;
    m.add_class::<core::webhooks::GetWebhooksParams>()?;
    m.add_class::<core::webhooks::UpdateWebhookParams>()?;
    m.add_class::<core::webhooks::EvmWalletFilterTemplate>()?;
    m.add_class::<core::webhooks::EvmContractEventsTemplate>()?;
    m.add_class::<core::webhooks::EvmAbiFilterTemplate>()?;
    m.add_class::<core::webhooks::SolanaWalletFilterTemplate>()?;
    m.add_class::<core::webhooks::BitcoinWalletFilterTemplate>()?;
    m.add_class::<core::webhooks::XrplWalletFilterTemplate>()?;
    m.add_class::<core::webhooks::HyperliquidWalletEventsFilterTemplate>()?;
    m.add_class::<core::webhooks::StellarWalletTransactionsFilterTemplate>()?;
    m.add_class::<KvStoreApiClient>()?;
    m.add_class::<core::kvstore::KvSetEntry>()?;
    m.add_class::<core::kvstore::GetSetsResponse>()?;
    m.add_class::<core::kvstore::GetSetResponse>()?;
    m.add_class::<core::kvstore::GetListsData>()?;
    m.add_class::<core::kvstore::GetListsResponse>()?;
    m.add_class::<core::kvstore::GetListData>()?;
    m.add_class::<core::kvstore::GetListResponse>()?;
    m.add_class::<core::kvstore::ListContainsItemResponse>()?;
    Ok(())
}

define_stub_info_gatherer!(stub_info);
