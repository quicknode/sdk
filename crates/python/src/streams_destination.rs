use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use sdk_core::streams::{
    AddressBookConfig, AzureAttributes, ClickhouseAttributes, DestinationAttributes,
    KafkaAttributes, ListStreamsResponse, MongoAttributes, MysqlAttributes, PageInfo,
    PostgresAttributes, RedisAttributes, S3Attributes, SnowflakeAttributes, Stream,
    WebhookAttributes,
};

// Per-destination typed wrappers. PyO3 cannot represent a Rust enum-with-data,
// so each variant of core's DestinationAttributes is exposed as its own
// #[pyclass] here. extract_destination_attributes() reassembles the enum at
// the FFI boundary.

macro_rules! destination_wrapper {
    ($name:ident, $attrs:ident, $variant:ident) => {
        #[gen_stub_pyclass]
        #[pyclass]
        #[derive(Clone)]
        pub struct $name {
            pub(crate) attrs: $attrs,
        }

        #[gen_stub_pymethods]
        #[pymethods]
        impl $name {
            #[new]
            pub fn new(attrs: $attrs) -> Self {
                Self { attrs }
            }

            #[getter]
            pub fn attributes(&self) -> $attrs {
                self.attrs.clone()
            }
        }

        impl $name {
            pub fn to_core(&self) -> DestinationAttributes {
                DestinationAttributes::$variant(self.attrs.clone())
            }

            pub fn from_core(attrs: $attrs) -> Self {
                Self { attrs }
            }
        }
    };
}

destination_wrapper!(StreamWebhookDestination, WebhookAttributes, Webhook);
destination_wrapper!(StreamS3Destination, S3Attributes, S3);
destination_wrapper!(StreamAzureDestination, AzureAttributes, Azure);
destination_wrapper!(StreamPostgresDestination, PostgresAttributes, Postgres);
destination_wrapper!(StreamMysqlDestination, MysqlAttributes, Mysql);
destination_wrapper!(StreamMongoDestination, MongoAttributes, Mongo);
destination_wrapper!(
    StreamClickhouseDestination,
    ClickhouseAttributes,
    Clickhouse
);
destination_wrapper!(StreamSnowflakeDestination, SnowflakeAttributes, Snowflake);
destination_wrapper!(StreamKafkaDestination, KafkaAttributes, Kafka);
destination_wrapper!(StreamRedisDestination, RedisAttributes, Redis);

// ── Conversion helpers ─────────────────────────────────────────────────────

pub fn extract_destination_attributes(obj: &Bound<'_, PyAny>) -> PyResult<DestinationAttributes> {
    if let Ok(v) = obj.extract::<StreamWebhookDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamS3Destination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamAzureDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamPostgresDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamMysqlDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamMongoDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamClickhouseDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamSnowflakeDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamKafkaDestination>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StreamRedisDestination>() {
        return Ok(v.to_core());
    }
    let received = obj
        .get_type()
        .name()
        .map_or_else(|_| "<unknown>".to_string(), |n| n.to_string());
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
        "destination_attributes must be one of StreamWebhookDestination, \
         StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, \
         StreamMysqlDestination, StreamMongoDestination, StreamClickhouseDestination, \
         StreamSnowflakeDestination, StreamKafkaDestination, StreamRedisDestination — \
         got {received}"
    )))
}

pub fn extract_extra_destinations(
    obj: Option<Bound<'_, PyAny>>,
) -> PyResult<Option<Vec<DestinationAttributes>>> {
    let Some(obj) = obj else { return Ok(None) };
    if obj.is_none() {
        return Ok(None);
    }
    let iter = obj.try_iter()?;
    let mut out = Vec::new();
    for item in iter {
        let item = item?;
        out.push(extract_destination_attributes(&item)?);
    }
    Ok(Some(out))
}

pub fn extra_destinations_to_py(
    py: Python<'_>,
    items: Option<Vec<DestinationAttributes>>,
) -> PyResult<Option<Vec<Py<PyAny>>>> {
    items
        .map(|v| {
            v.into_iter()
                .map(|a| destination_attributes_to_py(py, a))
                .collect::<PyResult<Vec<_>>>()
        })
        .transpose()
}

pub fn destination_attributes_to_py(
    py: Python<'_>,
    attrs: DestinationAttributes,
) -> PyResult<Py<PyAny>> {
    Ok(match attrs {
        DestinationAttributes::Webhook(a) => StreamWebhookDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::S3(a) => StreamS3Destination::from_core(a).into_pyobject(py)?.into(),
        DestinationAttributes::Azure(a) => StreamAzureDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Postgres(a) => StreamPostgresDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Mysql(a) => StreamMysqlDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Mongo(a) => StreamMongoDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Clickhouse(a) => StreamClickhouseDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Snowflake(a) => StreamSnowflakeDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Kafka(a) => StreamKafkaDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
        DestinationAttributes::Redis(a) => StreamRedisDestination::from_core(a)
            .into_pyobject(py)?
            .into(),
    })
}

// Core Stream cannot be #[pyclass] because it holds the flattened
// DestinationAttributes enum; this wrapper restores Python exposure and
// converts destination_attributes to the typed Python class.

#[gen_stub_pyclass]
#[pyclass(name = "Stream")]
pub struct PyStream {
    #[pyo3(get)]
    pub id: String,
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub status: String,
    #[pyo3(get)]
    pub created_at: String,
    #[pyo3(get)]
    pub updated_at: String,
    #[pyo3(get)]
    pub sequence: i64,
    #[pyo3(get)]
    pub network: String,
    #[pyo3(get)]
    pub dataset: String,
    #[pyo3(get)]
    pub region: String,
    #[pyo3(get)]
    pub start_range: i64,
    #[pyo3(get)]
    pub end_range: i64,
    #[pyo3(get)]
    pub plan: Option<String>,
    #[pyo3(get)]
    pub threshold_fetch_buffer: Option<i64>,
    #[pyo3(get)]
    pub dataset_batch_size: Option<i64>,
    #[pyo3(get)]
    pub max_batch_size: Option<i64>,
    #[pyo3(get)]
    pub max_buffer_range_size: Option<i64>,
    #[pyo3(get)]
    pub max_buffer_processing_workers: Option<i64>,
    #[pyo3(get)]
    pub keep_distance_from_tip: Option<i64>,
    #[pyo3(get)]
    pub filter_function: Option<String>,
    #[pyo3(get)]
    pub filter_language: Option<String>,
    #[pyo3(get)]
    pub include_stream_metadata: Option<String>,
    #[pyo3(get)]
    pub product_type: Option<String>,
    #[pyo3(get)]
    pub notification_email: Option<String>,
    #[pyo3(get)]
    pub fix_block_reorgs: Option<i32>,
    #[pyo3(get)]
    pub current_hash: Option<String>,
    // Exposed via a #[getter] below so pyo3-stub-gen can override the stub
    // type to the typed Union; #[pyo3(get)] on Py<PyAny> would produce
    // Optional[Any] in the generated stubs.
    pub destination_attributes: Option<Py<PyAny>>,
    #[pyo3(get)]
    pub elastic_batch_enabled: Option<bool>,
    #[pyo3(get)]
    pub qn_account_id: Option<String>,
    #[pyo3(get)]
    pub charge_min_cap: Option<i32>,
    #[pyo3(get)]
    pub memo: Option<String>,
    #[pyo3(get)]
    pub address_book_config: Option<AddressBookConfig>,
    // Exposed via a #[getter] below so the stub can override the list element
    // type to the typed Union over destination wrappers.
    pub extra_destinations: Option<Vec<Py<PyAny>>>,
}

impl PyStream {
    pub fn from_core(s: Stream, py: Python<'_>) -> PyResult<Self> {
        let destination_attributes = match s.destination_attributes {
            Some(attrs) => Some(destination_attributes_to_py(py, attrs)?),
            None => None,
        };
        let extra_destinations = extra_destinations_to_py(py, s.extra_destinations)?;
        Ok(Self {
            id: s.id,
            name: s.name,
            status: s.status,
            created_at: s.created_at,
            updated_at: s.updated_at,
            sequence: s.sequence,
            network: s.network,
            dataset: s.dataset,
            region: s.region,
            start_range: s.start_range,
            end_range: s.end_range,
            plan: s.plan,
            threshold_fetch_buffer: s.threshold_fetch_buffer,
            dataset_batch_size: s.dataset_batch_size,
            max_batch_size: s.max_batch_size,
            max_buffer_range_size: s.max_buffer_range_size,
            max_buffer_processing_workers: s.max_buffer_processing_workers,
            keep_distance_from_tip: s.keep_distance_from_tip,
            filter_function: s.filter_function,
            filter_language: s.filter_language,
            include_stream_metadata: s.include_stream_metadata,
            product_type: s.product_type,
            notification_email: s.notification_email,
            fix_block_reorgs: s.fix_block_reorgs,
            current_hash: s.current_hash,
            destination_attributes,
            elastic_batch_enabled: s.elastic_batch_enabled,
            qn_account_id: s.qn_account_id,
            charge_min_cap: s.charge_min_cap,
            memo: s.memo,
            address_book_config: s.address_book_config,
            extra_destinations,
        })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyStream {
    // Exposed as a getter so pyo3_stub_gen can override the stub to a typed
    // Union. Without the override, the stub would be `Optional[Any]` and IDEs
    // couldn't surface the 10 destination classes.
    #[getter]
    #[gen_stub(override_return_type(
        type_repr = "typing.Optional[typing.Union[StreamWebhookDestination, StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, StreamMysqlDestination, StreamMongoDestination, StreamClickhouseDestination, StreamSnowflakeDestination, StreamKafkaDestination, StreamRedisDestination]]"
    ))]
    fn destination_attributes<'py>(&self, py: Python<'py>) -> Option<Py<PyAny>> {
        self.destination_attributes
            .as_ref()
            .map(|v| v.clone_ref(py))
    }

    // Typed Union list so IDEs can see the 10 destination classes inside the
    // list, rather than `Optional[List[Any]]`.
    #[getter]
    #[gen_stub(override_return_type(
        type_repr = "typing.Optional[typing.List[typing.Union[StreamWebhookDestination, StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, StreamMysqlDestination, StreamMongoDestination, StreamClickhouseDestination, StreamSnowflakeDestination, StreamKafkaDestination, StreamRedisDestination]]]"
    ))]
    fn extra_destinations<'py>(&self, py: Python<'py>) -> Option<Vec<Py<PyAny>>> {
        self.extra_destinations
            .as_ref()
            .map(|v| v.iter().map(|item| item.clone_ref(py)).collect())
    }
}

#[gen_stub_pyclass]
#[pyclass(name = "ListStreamsResponse")]
pub struct PyListStreamsResponse {
    #[pyo3(get)]
    pub data: Vec<Py<PyStream>>,
    #[pyo3(get)]
    pub page_info: PageInfo,
}

impl PyListStreamsResponse {
    pub fn from_core(resp: ListStreamsResponse, py: Python<'_>) -> PyResult<Self> {
        let mut data = Vec::with_capacity(resp.data.len());
        for s in resp.data {
            data.push(Py::new(py, PyStream::from_core(s, py)?)?);
        }
        Ok(Self {
            data,
            page_info: resp.page_info,
        })
    }
}
