//! Macros to attach the standard Python conveniences (`__repr__`, `to_dict`)
//! to PyO3-exposed types. Reduces ~15-line boilerplate per type to one line.

/// Generates `__repr__` (via `Debug`) and `to_dict` (via `Serialize` →
/// Python dict/list/scalar tree using `pythonize`).
///
/// Call once per `#[pyclass]` type, in the same module as the type definition.
///
/// `__repr__` delegates to `format!("{:?}", self)`, so any field-level
/// redaction in a manual `Debug` impl (e.g. credentials printed as
/// `"[redacted]"`) is preserved.
///
/// `to_dict` returns a native Python `dict` and will mirror the type's
/// `serde::Serialize` output verbatim — **including** raw values of any
/// `#[serde]`-visible fields. Types that hold credential material must
/// either skip this macro and hand-roll `to_dict` to redact those fields,
/// or use `#[serde(serialize_with = "...")]` to redact at the serde layer.
#[cfg(feature = "python")]
#[macro_export]
macro_rules! python_repr_dict {
    ($type:ty) => {
        #[cfg(feature = "python")]
        #[pyo3_stub_gen::derive::gen_stub_pymethods]
        #[pyo3::pymethods]
        impl $type {
            fn __repr__(&self) -> String {
                format!("{:?}", self)
            }

            fn to_dict<'py>(
                &self,
                py: pyo3::Python<'py>,
            ) -> pyo3::PyResult<pyo3::Bound<'py, pyo3::PyAny>> {
                pythonize::pythonize(py, self)
                    .map_err(|e| pyo3::exceptions::PyValueError::new_err(e.to_string()))
            }
        }
    };
}

/// No-op when the `python` feature is off, so call sites compile cleanly
/// in pure-Rust and Node/Ruby-only builds.
#[cfg(not(feature = "python"))]
#[macro_export]
macro_rules! python_repr_dict {
    ($type:ty) => {};
}
