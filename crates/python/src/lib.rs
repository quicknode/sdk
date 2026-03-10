use pyo3::{exceptions::PyValueError, prelude::*};

use sdk_core as core;

#[pyfunction]
fn init(api_key: String) {
    core::init(api_key);
}

#[pyclass]
pub struct HttpbinClient {
    inner: core::httpbin::HttpbinClient,
}

#[pymethods]
impl HttpbinClient {
    #[new]
    fn new() -> Self {
        Self {
            inner: core::httpbin::HttpbinClient::new(),
        }
    }

    fn get_uuid<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyAny>> {
        let client = self.inner.clone();
        pyo3_async_runtimes::tokio::future_into_py(py, async move {
            client
                .get_uuid()
                .await
                .map_err(|e| PyValueError::new_err(e.to_string()))
        })
    }
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(init, m)?)?;
    m.add_class::<HttpbinClient>()?;
    Ok(())
}
