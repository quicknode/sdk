use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::sql::{ColumnMeta, QueryResponse, QueryStatistics};

// Core QueryResponse cannot be #[pyclass] because `data` holds
// serde_json::Value rows whose shape depends on the SQL query. This wrapper
// keeps meta/statistics/counts typed and converts each dynamic row to a native
// Python object via `pythonize`.
#[gen_stub_pyclass]
#[pyclass(name = "QueryResponse")]
pub struct PyQueryResponse {
    #[pyo3(get)]
    pub meta: Vec<ColumnMeta>,
    // Exposed via a #[getter] below so pyo3-stub-gen can override the stub type
    // to list[dict[str, Any]]; #[pyo3(get)] on Py<PyAny> would produce Any.
    pub data: Vec<Py<PyAny>>,
    #[pyo3(get)]
    pub rows: i64,
    #[pyo3(get)]
    pub rows_before_limit_at_least: i64,
    #[pyo3(get)]
    pub statistics: QueryStatistics,
    #[pyo3(get)]
    pub credits: i64,
}

impl PyQueryResponse {
    pub fn from_core(resp: QueryResponse, py: Python<'_>) -> PyResult<Self> {
        let mut data = Vec::with_capacity(resp.data.len());
        for row in resp.data {
            // pythonize turns an arbitrary serde_json::Value into the matching
            // native Python object (dict/list/str/number/bool/None).
            let obj = pythonize::pythonize(py, &row)
                .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))?;
            data.push(obj.unbind());
        }
        Ok(Self {
            meta: resp.meta,
            data,
            rows: resp.rows,
            rows_before_limit_at_least: resp.rows_before_limit_at_least,
            statistics: resp.statistics,
            credits: resp.credits,
        })
    }
}

#[gen_stub_pymethods]
#[pymethods]
impl PyQueryResponse {
    // Exposed as a getter so pyo3-stub-gen can type the rows. Without the
    // override the stub would be `Any` and IDEs couldn't surface that rows are
    // dicts keyed by column name.
    #[getter]
    #[gen_stub(override_return_type(type_repr = "list[dict[str, typing.Any]]"))]
    fn data<'py>(&self, py: Python<'py>) -> Vec<Py<PyAny>> {
        self.data.iter().map(|o| o.clone_ref(py)).collect()
    }
}
