use pyo3::{exceptions::PyValueError, prelude::*};

use my_sdk_core as core;

#[pyfunction]
fn add(a: i64, b: i64) -> i64 {
    core::add(a, b)
}

#[pyfunction]
pub fn divide(a: f64, b: f64) -> PyResult<f64> {
    core::divide(a, b).map_err(|e| PyValueError::new_err(e.to_string()))
}

#[pyfunction]
pub fn get_external_uuid(py: Python<'_>) -> PyResult<Bound<'_, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        core::get_external_uuid()
            .await
            .map_err(|e| PyValueError::new_err(e.to_string()))
    })
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(add, m)?)?;
    m.add_function(wrap_pyfunction!(divide, m)?)?;
    m.add_function(wrap_pyfunction!(get_external_uuid, m)?)?;
    Ok(())
}
