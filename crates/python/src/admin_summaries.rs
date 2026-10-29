use pyo3::exceptions::PyAttributeError;
use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::admin as core;

// Optional grouping dimensions retain presence: an omitted dimension raises
// AttributeError, while a selected dimension with a null value returns None.
#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageRpcSummaryRow {
    pub(crate) inner: core::UsageRpcSummaryRow,
}

impl From<core::UsageRpcSummaryRow> for UsageRpcSummaryRow {
    fn from(inner: core::UsageRpcSummaryRow) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageRpcSummaryRow {
    #[getter]
    fn requests(&self) -> Option<f64> {
        self.inner.requests
    }
    #[getter]
    fn bytes(&self) -> f64 {
        self.inner.bytes
    }
    #[getter]
    fn rpc_credits(&self) -> f64 {
        self.inner.rpc_credits
    }
    #[getter]
    fn data_credits(&self) -> f64 {
        self.inner.data_credits
    }
    #[getter]
    fn credits(&self) -> f64 {
        self.inner.credits
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn method(&self) -> PyResult<Option<String>> {
        self.inner.method.clone().ok_or_else(|| {
            PyAttributeError::new_err("method was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn endpoint(&self) -> PyResult<Option<i64>> {
        self.inner.endpoint.ok_or_else(|| {
            PyAttributeError::new_err("endpoint was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn chain(&self) -> PyResult<Option<String>> {
        self.inner.chain.clone().ok_or_else(|| {
            PyAttributeError::new_err("chain was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn network(&self) -> PyResult<Option<String>> {
        self.inner.network.clone().ok_or_else(|| {
            PyAttributeError::new_err("network was not included in this summary group")
        })
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageRpcSummaryData {
    pub(crate) inner: core::UsageRpcSummaryData,
}

impl From<core::UsageRpcSummaryData> for UsageRpcSummaryData {
    fn from(inner: core::UsageRpcSummaryData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageRpcSummaryData {
    #[getter(from_time)]
    fn get_from_time(&self) -> String {
        self.inner.from_time.clone()
    }
    #[getter]
    fn to_time(&self) -> String {
        self.inner.to_time.clone()
    }
    #[getter]
    fn totals(&self) -> core::UsageRpcSummaryTotals {
        self.inner.totals.clone()
    }
    #[getter]
    fn breakdown(&self) -> Option<Vec<UsageRpcSummaryRow>> {
        self.inner
            .breakdown
            .clone()
            .map(|rows| rows.into_iter().map(UsageRpcSummaryRow::from).collect())
    }
    #[getter]
    fn next_cursor(&self) -> Option<String> {
        self.inner.next_cursor.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetUsageRpcSummaryResponse {
    pub(crate) inner: core::GetUsageRpcSummaryResponse,
}

impl From<core::GetUsageRpcSummaryResponse> for GetUsageRpcSummaryResponse {
    fn from(inner: core::GetUsageRpcSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetUsageRpcSummaryResponse {
    #[getter]
    fn data(&self) -> UsageRpcSummaryData {
        UsageRpcSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageStreamsSummaryRow {
    pub(crate) inner: core::UsageStreamsSummaryRow,
}

impl From<core::UsageStreamsSummaryRow> for UsageStreamsSummaryRow {
    fn from(inner: core::UsageStreamsSummaryRow) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageStreamsSummaryRow {
    #[getter]
    fn blocks(&self) -> f64 {
        self.inner.blocks
    }
    #[getter]
    fn bytes(&self) -> f64 {
        self.inner.bytes
    }
    #[getter]
    fn block_credits(&self) -> f64 {
        self.inner.block_credits
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn stream(&self) -> PyResult<Option<String>> {
        self.inner.stream.clone().ok_or_else(|| {
            PyAttributeError::new_err("stream was not included in this summary group")
        })
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageStreamsSummaryData {
    pub(crate) inner: core::UsageStreamsSummaryData,
}

impl From<core::UsageStreamsSummaryData> for UsageStreamsSummaryData {
    fn from(inner: core::UsageStreamsSummaryData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageStreamsSummaryData {
    #[getter(from_time)]
    fn get_from_time(&self) -> String {
        self.inner.from_time.clone()
    }
    #[getter]
    fn to_time(&self) -> String {
        self.inner.to_time.clone()
    }
    #[getter]
    fn totals(&self) -> core::UsageStreamsSummaryTotals {
        self.inner.totals.clone()
    }
    #[getter]
    fn breakdown(&self) -> Option<Vec<UsageStreamsSummaryRow>> {
        self.inner
            .breakdown
            .clone()
            .map(|rows| rows.into_iter().map(UsageStreamsSummaryRow::from).collect())
    }
    #[getter]
    fn next_cursor(&self) -> Option<String> {
        self.inner.next_cursor.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetUsageStreamsSummaryResponse {
    pub(crate) inner: core::GetUsageStreamsSummaryResponse,
}

impl From<core::GetUsageStreamsSummaryResponse> for GetUsageStreamsSummaryResponse {
    fn from(inner: core::GetUsageStreamsSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetUsageStreamsSummaryResponse {
    #[getter]
    fn data(&self) -> UsageStreamsSummaryData {
        UsageStreamsSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageWebhooksSummaryRow {
    pub(crate) inner: core::UsageWebhooksSummaryRow,
}

impl From<core::UsageWebhooksSummaryRow> for UsageWebhooksSummaryRow {
    fn from(inner: core::UsageWebhooksSummaryRow) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageWebhooksSummaryRow {
    #[getter]
    fn deliveries(&self) -> f64 {
        self.inner.deliveries
    }
    #[getter]
    fn credits(&self) -> f64 {
        self.inner.credits
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn webhook(&self) -> PyResult<Option<String>> {
        self.inner.webhook.clone().ok_or_else(|| {
            PyAttributeError::new_err("webhook was not included in this summary group")
        })
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct UsageWebhooksSummaryData {
    pub(crate) inner: core::UsageWebhooksSummaryData,
}

impl From<core::UsageWebhooksSummaryData> for UsageWebhooksSummaryData {
    fn from(inner: core::UsageWebhooksSummaryData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl UsageWebhooksSummaryData {
    #[getter(from_time)]
    fn get_from_time(&self) -> String {
        self.inner.from_time.clone()
    }
    #[getter]
    fn to_time(&self) -> String {
        self.inner.to_time.clone()
    }
    #[getter]
    fn totals(&self) -> core::UsageWebhooksSummaryTotals {
        self.inner.totals.clone()
    }
    #[getter]
    fn breakdown(&self) -> Option<Vec<UsageWebhooksSummaryRow>> {
        self.inner.breakdown.clone().map(|rows| {
            rows.into_iter()
                .map(UsageWebhooksSummaryRow::from)
                .collect()
        })
    }
    #[getter]
    fn next_cursor(&self) -> Option<String> {
        self.inner.next_cursor.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetUsageWebhooksSummaryResponse {
    pub(crate) inner: core::GetUsageWebhooksSummaryResponse,
}

impl From<core::GetUsageWebhooksSummaryResponse> for GetUsageWebhooksSummaryResponse {
    fn from(inner: core::GetUsageWebhooksSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetUsageWebhooksSummaryResponse {
    #[getter]
    fn data(&self) -> UsageWebhooksSummaryData {
        UsageWebhooksSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct TrafficSummaryRow {
    pub(crate) inner: core::TrafficSummaryRow,
}

impl From<core::TrafficSummaryRow> for TrafficSummaryRow {
    fn from(inner: core::TrafficSummaryRow) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl TrafficSummaryRow {
    #[getter]
    fn successful_requests(&self) -> Option<f64> {
        self.inner.successful_requests
    }
    #[getter]
    fn bytes(&self) -> f64 {
        self.inner.bytes
    }
    #[getter]
    fn request_credits(&self) -> f64 {
        self.inner.request_credits
    }
    #[getter]
    fn bytes_credits(&self) -> f64 {
        self.inner.bytes_credits
    }
    #[getter]
    fn credits(&self) -> f64 {
        self.inner.credits
    }
    #[getter]
    fn active_methods(&self) -> f64 {
        self.inner.active_methods
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn method(&self) -> PyResult<Option<String>> {
        self.inner.method.clone().ok_or_else(|| {
            PyAttributeError::new_err("method was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn endpoint(&self) -> PyResult<Option<i64>> {
        self.inner.endpoint.ok_or_else(|| {
            PyAttributeError::new_err("endpoint was not included in this summary group")
        })
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct TrafficSummaryData {
    pub(crate) inner: core::TrafficSummaryData,
}

impl From<core::TrafficSummaryData> for TrafficSummaryData {
    fn from(inner: core::TrafficSummaryData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl TrafficSummaryData {
    #[getter(from_time)]
    fn get_from_time(&self) -> String {
        self.inner.from_time.clone()
    }
    #[getter]
    fn to_time(&self) -> String {
        self.inner.to_time.clone()
    }
    #[getter]
    fn totals(&self) -> core::TrafficSummaryTotals {
        self.inner.totals.clone()
    }
    #[getter]
    fn breakdown(&self) -> Option<Vec<TrafficSummaryRow>> {
        self.inner
            .breakdown
            .clone()
            .map(|rows| rows.into_iter().map(TrafficSummaryRow::from).collect())
    }
    #[getter]
    fn next_cursor(&self) -> Option<String> {
        self.inner.next_cursor.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetTrafficSummaryResponse {
    pub(crate) inner: core::GetTrafficSummaryResponse,
}

impl From<core::GetTrafficSummaryResponse> for GetTrafficSummaryResponse {
    fn from(inner: core::GetTrafficSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetTrafficSummaryResponse {
    #[getter]
    fn data(&self) -> TrafficSummaryData {
        TrafficSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct ErrorsSummaryRow {
    pub(crate) inner: core::ErrorsSummaryRow,
}

impl From<core::ErrorsSummaryRow> for ErrorsSummaryRow {
    fn from(inner: core::ErrorsSummaryRow) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl ErrorsSummaryRow {
    #[getter]
    fn kind(&self) -> String {
        self.inner.kind.clone()
    }
    #[getter]
    fn count(&self) -> f64 {
        self.inner.count
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn code(&self) -> PyResult<Option<String>> {
        self.inner
            .code
            .clone()
            .ok_or_else(|| PyAttributeError::new_err("code was not included in this summary group"))
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn endpoint(&self) -> PyResult<Option<i64>> {
        self.inner.endpoint.ok_or_else(|| {
            PyAttributeError::new_err("endpoint was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn network(&self) -> PyResult<Option<String>> {
        self.inner.network.clone().ok_or_else(|| {
            PyAttributeError::new_err("network was not included in this summary group")
        })
    }
    /// Present only when this dimension was selected for grouping.
    #[getter]
    fn method(&self) -> PyResult<Option<String>> {
        self.inner.method.clone().ok_or_else(|| {
            PyAttributeError::new_err("method was not included in this summary group")
        })
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct ErrorsSummaryData {
    pub(crate) inner: core::ErrorsSummaryData,
}

impl From<core::ErrorsSummaryData> for ErrorsSummaryData {
    fn from(inner: core::ErrorsSummaryData) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl ErrorsSummaryData {
    #[getter(from_time)]
    fn get_from_time(&self) -> String {
        self.inner.from_time.clone()
    }
    #[getter]
    fn to_time(&self) -> String {
        self.inner.to_time.clone()
    }
    #[getter]
    fn totals(&self) -> core::ErrorsSummaryTotals {
        self.inner.totals.clone()
    }
    #[getter]
    fn breakdown(&self) -> Option<Vec<ErrorsSummaryRow>> {
        self.inner
            .breakdown
            .clone()
            .map(|rows| rows.into_iter().map(ErrorsSummaryRow::from).collect())
    }
    #[getter]
    fn next_cursor(&self) -> Option<String> {
        self.inner.next_cursor.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetErrorsSummaryResponse {
    pub(crate) inner: core::GetErrorsSummaryResponse,
}

impl From<core::GetErrorsSummaryResponse> for GetErrorsSummaryResponse {
    fn from(inner: core::GetErrorsSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetErrorsSummaryResponse {
    #[getter]
    fn data(&self) -> ErrorsSummaryData {
        ErrorsSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}

#[gen_stub_pyclass]
#[pyclass]
#[derive(Clone)]
pub struct GetEndpointErrorsSummaryResponse {
    pub(crate) inner: core::GetEndpointErrorsSummaryResponse,
}

impl From<core::GetEndpointErrorsSummaryResponse> for GetEndpointErrorsSummaryResponse {
    fn from(inner: core::GetEndpointErrorsSummaryResponse) -> Self {
        Self { inner }
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl GetEndpointErrorsSummaryResponse {
    #[getter]
    fn data(&self) -> ErrorsSummaryData {
        ErrorsSummaryData::from(self.inner.data.clone())
    }
    #[getter]
    fn error(&self) -> Option<String> {
        self.inner.error.clone()
    }
}
