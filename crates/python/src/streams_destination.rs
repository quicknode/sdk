use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::streams::{
    AddressBookConfig, AzureAttributes, DestinationAttributes, KafkaAttributes,
    ListStreamsResponse, PageInfo, PostgresAttributes, S3Attributes, Stream, WebhookAttributes,
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

            // Forward to the inner attributes so Python users see the actual
            // destination fields rather than `<Wrapper object at 0x...>`.
            fn __repr__(&self) -> String {
                format!("{}({:?})", stringify!($name), self.attrs)
            }

            fn to_dict<'py>(
                &self,
                py: pyo3::Python<'py>,
            ) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::PyAny>> {
                pythonize::pythonize(py, &self.attrs)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
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
destination_wrapper!(StreamKafkaDestination, KafkaAttributes, Kafka);

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
    if let Ok(v) = obj.extract::<StreamKafkaDestination>() {
        return Ok(v.to_core());
    }
    let received = obj
        .get_type()
        .name()
        .map_or_else(|_| "<unknown>".to_string(), |n| n.to_string());
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
        "destination_attributes must be one of StreamWebhookDestination, \
         StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, \
         StreamKafkaDestination — got {received}"
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
        DestinationAttributes::Kafka(a) => StreamKafkaDestination::from_core(a)
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
    // couldn't surface the destination classes.
    #[getter]
    #[gen_stub(override_return_type(
        type_repr = "typing.Optional[typing.Union[StreamWebhookDestination, StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, StreamKafkaDestination]]"
    ))]
    fn destination_attributes<'py>(&self, py: Python<'py>) -> Option<Py<PyAny>> {
        self.destination_attributes
            .as_ref()
            .map(|v| v.clone_ref(py))
    }

    // Typed Union list so IDEs can see the destination classes inside the
    // list, rather than `Optional[List[Any]]`.
    #[getter]
    #[gen_stub(override_return_type(
        type_repr = "typing.Optional[typing.List[typing.Union[StreamWebhookDestination, StreamS3Destination, StreamAzureDestination, StreamPostgresDestination, StreamKafkaDestination]]]"
    ))]
    fn extra_destinations<'py>(&self, py: Python<'py>) -> Option<Vec<Py<PyAny>>> {
        self.extra_destinations
            .as_ref()
            .map(|v| v.iter().map(|item| item.clone_ref(py)).collect())
    }

    fn __repr__(&self) -> String {
        format!(
            "Stream(id={:?}, name={:?}, status={:?}, network={:?}, dataset={:?})",
            self.id, self.name, self.status, self.network, self.dataset
        )
    }

    // Hand-rolled because PyStream holds Py<PyAny> for destination_attributes
    // and extra_destinations, so pythonize can't serialize the struct directly.
    // The nested destination wrappers expose their own to_dict() which we call
    // recursively so the output is a fully native dict tree.
    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pyo3::types::{PyDict, PyList};
        let d = PyDict::new(py);
        d.set_item("id", &self.id)?;
        d.set_item("name", &self.name)?;
        d.set_item("status", &self.status)?;
        d.set_item("created_at", &self.created_at)?;
        d.set_item("updated_at", &self.updated_at)?;
        d.set_item("sequence", self.sequence)?;
        d.set_item("network", &self.network)?;
        d.set_item("dataset", &self.dataset)?;
        d.set_item("region", &self.region)?;
        d.set_item("start_range", self.start_range)?;
        d.set_item("end_range", self.end_range)?;
        d.set_item("plan", &self.plan)?;
        d.set_item("threshold_fetch_buffer", self.threshold_fetch_buffer)?;
        d.set_item("dataset_batch_size", self.dataset_batch_size)?;
        d.set_item("max_batch_size", self.max_batch_size)?;
        d.set_item("max_buffer_range_size", self.max_buffer_range_size)?;
        d.set_item(
            "max_buffer_processing_workers",
            self.max_buffer_processing_workers,
        )?;
        d.set_item("keep_distance_from_tip", self.keep_distance_from_tip)?;
        d.set_item("filter_function", &self.filter_function)?;
        d.set_item("filter_language", &self.filter_language)?;
        d.set_item("include_stream_metadata", &self.include_stream_metadata)?;
        d.set_item("product_type", &self.product_type)?;
        d.set_item("notification_email", &self.notification_email)?;
        d.set_item("fix_block_reorgs", self.fix_block_reorgs)?;
        d.set_item("current_hash", &self.current_hash)?;
        d.set_item("elastic_batch_enabled", self.elastic_batch_enabled)?;
        d.set_item("qn_account_id", &self.qn_account_id)?;
        d.set_item("charge_min_cap", self.charge_min_cap)?;
        d.set_item("memo", &self.memo)?;
        // Recurse into destination wrappers via their own to_dict().
        let dest = match &self.destination_attributes {
            Some(v) => v.bind(py).call_method0("to_dict")?.into_any().unbind(),
            None => py.None(),
        };
        d.set_item("destination_attributes", dest)?;
        // address_book_config is a Serialize struct; let pythonize handle it.
        let abc = match &self.address_book_config {
            Some(c) => pythonize::pythonize(py, c)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?,
            None => py.None().into_bound(py),
        };
        d.set_item("address_book_config", abc)?;
        let extras = match &self.extra_destinations {
            Some(vec) => {
                let list = PyList::empty(py);
                for item in vec {
                    list.append(item.bind(py).call_method0("to_dict")?)?;
                }
                list.into_any().unbind()
            }
            None => py.None(),
        };
        d.set_item("extra_destinations", extras)?;
        Ok(d)
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

#[gen_stub_pymethods]
#[pymethods]
impl PyListStreamsResponse {
    fn __repr__(&self) -> String {
        format!(
            "ListStreamsResponse(data=[{} streams], page_info={:?})",
            self.data.len(),
            self.page_info
        )
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, pyo3::types::PyDict>> {
        use pyo3::types::{PyDict, PyList};
        let d = PyDict::new(py);
        let list = PyList::empty(py);
        for s in &self.data {
            list.append(s.bind(py).call_method0("to_dict")?)?;
        }
        d.set_item("data", list)?;
        let pi = pythonize::pythonize(py, &self.page_info)
            .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
        d.set_item("page_info", pi)?;
        Ok(d)
    }
}
