use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::admin as core;

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct EndpointRequestFilter {
    pub(crate) inner: core::EndpointRequestFilter,
}

impl From<core::EndpointRequestFilter> for EndpointRequestFilter {
    fn from(inner: core::EndpointRequestFilter) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl EndpointRequestFilter {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }
    #[setter]
    fn set_id(&mut self, value: String) {
        self.inner.id = value;
    }
    #[getter]
    #[gen_stub(override_return_type(type_repr = "list[typing.Any]"))]
    fn method<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.method).map_err(Into::into)
    }
    #[setter]
    fn set_method(
        &mut self,
        #[gen_stub(override_type(type_repr = "list[typing.Any]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.method = pythonize::depythonize(value)?;
        Ok(())
    }
    #[getter]
    #[gen_stub(override_return_type(type_repr = "dict[str, typing.Any]"))]
    fn params<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.params).map_err(Into::into)
    }
    #[setter]
    fn set_params(
        &mut self,
        #[gen_stub(override_type(type_repr = "dict[str, typing.Any]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.params = pythonize::depythonize(value)?;
        Ok(())
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct EndpointSecurity {
    pub(crate) inner: core::EndpointSecurity,
}

impl From<core::EndpointSecurity> for EndpointSecurity {
    fn from(inner: core::EndpointSecurity) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl EndpointSecurity {
    #[getter]
    fn options(&self) -> Option<core::EndpointSecurityOptions> {
        self.inner.options.clone()
    }
    #[setter]
    fn set_options(&mut self, value: Option<core::EndpointSecurityOptions>) {
        self.inner.options = value;
    }
    #[getter]
    fn tokens(&self) -> Option<Vec<core::EndpointToken>> {
        self.inner.tokens.clone()
    }
    #[setter]
    fn set_tokens(&mut self, value: Option<Vec<core::EndpointToken>>) {
        self.inner.tokens = value;
    }
    #[getter]
    fn jwts(&self) -> Option<Vec<core::EndpointJwt>> {
        self.inner.jwts.clone()
    }
    #[setter]
    fn set_jwts(&mut self, value: Option<Vec<core::EndpointJwt>>) {
        self.inner.jwts = value;
    }
    #[getter]
    fn referrers(&self) -> Option<Vec<core::EndpointReferrer>> {
        self.inner.referrers.clone()
    }
    #[setter]
    fn set_referrers(&mut self, value: Option<Vec<core::EndpointReferrer>>) {
        self.inner.referrers = value;
    }
    #[getter]
    fn domain_masks(&self) -> Option<Vec<core::EndpointDomainMask>> {
        self.inner.domain_masks.clone()
    }
    #[setter]
    fn set_domain_masks(&mut self, value: Option<Vec<core::EndpointDomainMask>>) {
        self.inner.domain_masks = value;
    }
    #[getter]
    fn ips(&self) -> Option<Vec<core::EndpointIp>> {
        self.inner.ips.clone()
    }
    #[setter]
    fn set_ips(&mut self, value: Option<Vec<core::EndpointIp>>) {
        self.inner.ips = value;
    }
    #[getter]
    fn blocked_ips(&self) -> Option<Vec<core::EndpointIp>> {
        self.inner.blocked_ips.clone()
    }
    #[setter]
    fn set_blocked_ips(&mut self, value: Option<Vec<core::EndpointIp>>) {
        self.inner.blocked_ips = value;
    }
    #[getter]
    fn request_filters(&self) -> Option<Vec<EndpointRequestFilter>> {
        self.inner
            .request_filters
            .clone()
            .map(|value| value.into_iter().map(EndpointRequestFilter::from).collect())
    }
    #[setter]
    fn set_request_filters(&mut self, value: Option<Vec<EndpointRequestFilter>>) {
        self.inner.request_filters =
            value.map(|value| value.into_iter().map(|value| value.inner).collect());
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct SingleEndpoint {
    pub(crate) inner: core::SingleEndpoint,
}

impl From<core::SingleEndpoint> for SingleEndpoint {
    fn from(inner: core::SingleEndpoint) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl SingleEndpoint {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }
    #[setter]
    fn set_id(&mut self, value: String) {
        self.inner.id = value;
    }
    #[getter]
    fn label(&self) -> Option<String> {
        self.inner.label.clone()
    }
    #[setter]
    fn set_label(&mut self, value: Option<String>) {
        self.inner.label = value;
    }
    #[getter]
    fn status(&self) -> Option<String> {
        self.inner.status.clone()
    }
    #[setter]
    fn set_status(&mut self, value: Option<String>) {
        self.inner.status = value;
    }
    #[getter]
    fn chain(&self) -> String {
        self.inner.chain.clone()
    }
    #[setter]
    fn set_chain(&mut self, value: String) {
        self.inner.chain = value;
    }
    #[getter]
    fn network(&self) -> String {
        self.inner.network.clone()
    }
    #[setter]
    fn set_network(&mut self, value: String) {
        self.inner.network = value;
    }
    #[getter]
    fn http_url(&self) -> String {
        self.inner.http_url.clone()
    }
    #[setter]
    fn set_http_url(&mut self, value: String) {
        self.inner.http_url = value;
    }
    #[getter]
    fn wss_url(&self) -> Option<String> {
        self.inner.wss_url.clone()
    }
    #[setter]
    fn set_wss_url(&mut self, value: Option<String>) {
        self.inner.wss_url = value;
    }
    #[getter]
    fn security(&self) -> Option<EndpointSecurity> {
        self.inner.security.clone().map(EndpointSecurity::from)
    }
    #[setter]
    fn set_security(&mut self, value: Option<EndpointSecurity>) {
        self.inner.security = value.map(|value| value.inner);
    }
    #[getter]
    fn rate_limits(&self) -> Option<core::EndpointRateLimits> {
        self.inner.rate_limits.clone()
    }
    #[setter]
    fn set_rate_limits(&mut self, value: Option<core::EndpointRateLimits>) {
        self.inner.rate_limits = value;
    }
    #[getter]
    fn tags(&self) -> Vec<core::EndpointTag> {
        self.inner.tags.clone().into_iter().collect()
    }
    #[setter]
    fn set_tags(&mut self, value: Vec<core::EndpointTag>) {
        self.inner.tags = value.into_iter().collect();
    }
    #[getter]
    fn is_multichain(&self) -> bool {
        self.inner.is_multichain
    }
    #[setter]
    fn set_is_multichain(&mut self, value: bool) {
        self.inner.is_multichain = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct CreateEndpointResponse {
    pub(crate) inner: core::CreateEndpointResponse,
}

impl From<core::CreateEndpointResponse> for CreateEndpointResponse {
    fn from(inner: core::CreateEndpointResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl CreateEndpointResponse {
    #[getter]
    fn data(&self) -> SingleEndpoint {
        SingleEndpoint::from(self.inner.data.clone())
    }
    #[setter]
    fn set_data(&mut self, value: SingleEndpoint) {
        self.inner.data = value.inner;
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
    #[setter]
    fn set_error(&mut self, value: Option<String>) {
        self.inner.error = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct ShowEndpointResponse {
    pub(crate) inner: core::ShowEndpointResponse,
}

impl From<core::ShowEndpointResponse> for ShowEndpointResponse {
    fn from(inner: core::ShowEndpointResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl ShowEndpointResponse {
    #[getter]
    fn data(&self) -> Option<SingleEndpoint> {
        self.inner.data.clone().map(SingleEndpoint::from)
    }
    #[setter]
    fn set_data(&mut self, value: Option<SingleEndpoint>) {
        self.inner.data = value.map(|value| value.inner);
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
    #[setter]
    fn set_error(&mut self, value: Option<String>) {
        self.inner.error = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetEndpointSecurityResponse {
    pub(crate) inner: core::GetEndpointSecurityResponse,
}

impl From<core::GetEndpointSecurityResponse> for GetEndpointSecurityResponse {
    fn from(inner: core::GetEndpointSecurityResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetEndpointSecurityResponse {
    #[getter]
    fn data(&self) -> Option<EndpointSecurity> {
        self.inner.data.clone().map(EndpointSecurity::from)
    }
    #[setter]
    fn set_data(&mut self, value: Option<EndpointSecurity>) {
        self.inner.data = value.map(|value| value.inner);
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
    #[setter]
    fn set_error(&mut self, value: Option<String>) {
        self.inner.error = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct CreateRequestFilterData {
    pub(crate) inner: core::CreateRequestFilterData,
}

impl From<core::CreateRequestFilterData> for CreateRequestFilterData {
    fn from(inner: core::CreateRequestFilterData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl CreateRequestFilterData {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }
    #[setter]
    fn set_id(&mut self, value: String) {
        self.inner.id = value;
    }
    #[getter]
    #[gen_stub(override_return_type(type_repr = "typing.Optional[dict[str, typing.Any]]"))]
    fn pattern<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.pattern).map_err(Into::into)
    }
    #[setter]
    fn set_pattern(
        &mut self,
        #[gen_stub(override_type(type_repr = "typing.Optional[dict[str, typing.Any]]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.pattern = pythonize::depythonize(value)?;
        Ok(())
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct CreateRequestFilterResponse {
    pub(crate) inner: core::CreateRequestFilterResponse,
}

impl From<core::CreateRequestFilterResponse> for CreateRequestFilterResponse {
    fn from(inner: core::CreateRequestFilterResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl CreateRequestFilterResponse {
    #[getter]
    fn data(&self) -> Option<CreateRequestFilterData> {
        self.inner.data.clone().map(CreateRequestFilterData::from)
    }
    #[setter]
    fn set_data(&mut self, value: Option<CreateRequestFilterData>) {
        self.inner.data = value.map(|value| value.inner);
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
    #[setter]
    fn set_error(&mut self, value: Option<String>) {
        self.inner.error = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct RequestFilter {
    pub(crate) inner: core::RequestFilter,
}

impl From<core::RequestFilter> for RequestFilter {
    fn from(inner: core::RequestFilter) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl RequestFilter {
    #[getter]
    fn id(&self) -> String {
        self.inner.id.clone()
    }
    #[setter]
    fn set_id(&mut self, value: String) {
        self.inner.id = value;
    }
    #[getter]
    #[gen_stub(override_return_type(type_repr = "dict[str, typing.Any]"))]
    fn pattern<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.pattern).map_err(Into::into)
    }
    #[setter]
    fn set_pattern(
        &mut self,
        #[gen_stub(override_type(type_repr = "dict[str, typing.Any]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.pattern = pythonize::depythonize(value)?;
        Ok(())
    }
    #[getter]
    fn method_whitelist(&self) -> bool {
        self.inner.method_whitelist
    }
    #[setter]
    fn set_method_whitelist(&mut self, value: bool) {
        self.inner.method_whitelist = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetRequestFiltersResponse {
    pub(crate) inner: core::GetRequestFiltersResponse,
}

impl From<core::GetRequestFiltersResponse> for GetRequestFiltersResponse {
    fn from(inner: core::GetRequestFiltersResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetRequestFiltersResponse {
    #[getter]
    fn data(&self) -> Vec<RequestFilter> {
        self.inner
            .data
            .clone()
            .into_iter()
            .map(RequestFilter::from)
            .collect()
    }
    #[setter]
    fn set_data(&mut self, value: Vec<RequestFilter>) {
        self.inner.data = value.into_iter().map(|value| value.inner).collect();
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
    #[setter]
    fn set_error(&mut self, value: Option<String>) {
        self.inner.error = value;
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct CreateRequestFilterPatternRequest {
    pub(crate) inner: core::CreateRequestFilterPatternRequest,
}

impl From<core::CreateRequestFilterPatternRequest> for CreateRequestFilterPatternRequest {
    fn from(inner: core::CreateRequestFilterPatternRequest) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl CreateRequestFilterPatternRequest {
    #[getter]
    #[gen_stub(override_return_type(type_repr = "dict[str, typing.Any]"))]
    fn pattern<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.pattern).map_err(Into::into)
    }
    #[setter]
    fn set_pattern(
        &mut self,
        #[gen_stub(override_type(type_repr = "dict[str, typing.Any]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.pattern = pythonize::depythonize(value)?;
        Ok(())
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UpdateRequestFilterPatternRequest {
    pub(crate) inner: core::UpdateRequestFilterPatternRequest,
}

impl From<core::UpdateRequestFilterPatternRequest> for UpdateRequestFilterPatternRequest {
    fn from(inner: core::UpdateRequestFilterPatternRequest) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UpdateRequestFilterPatternRequest {
    #[getter]
    #[gen_stub(override_return_type(type_repr = "dict[str, typing.Any]"))]
    fn pattern<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        pythonize::pythonize(py, &self.inner.pattern).map_err(Into::into)
    }
    #[setter]
    fn set_pattern(
        &mut self,
        #[gen_stub(override_type(type_repr = "dict[str, typing.Any]", imports = ("typing")))]
        value: &Bound<'_, PyAny>,
    ) -> PyResult<()> {
        self.inner.pattern = pythonize::depythonize(value)?;
        Ok(())
    }
}
