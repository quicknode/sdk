use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::webhooks::{
    BitcoinWalletFilterTemplate, EvmAbiFilterTemplate, EvmContractEventsTemplate,
    EvmWalletFilterTemplate, HyperliquidWalletEventsFilterTemplate, SolanaWalletFilterTemplate,
    StellarWalletTransactionsFilterTemplate, TemplateArgs, XrplWalletFilterTemplate,
};

// Per-template typed wrappers. PyO3 cannot represent a Rust enum-with-data,
// so each variant of core's TemplateArgs is exposed as its own #[pyclass]
// here. extract_template_args() reassembles the enum at the FFI boundary.

macro_rules! template_wrapper {
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

            // Forward to the inner template attributes so Python users see
            // the actual filter fields rather than `<Wrapper object at 0x...>`.
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
            pub fn to_core(&self) -> TemplateArgs {
                TemplateArgs::$variant(self.attrs.clone())
            }
        }
    };
}

template_wrapper!(
    EvmWalletFilterArgs,
    EvmWalletFilterTemplate,
    EvmWalletFilter
);
template_wrapper!(
    EvmContractEventsArgs,
    EvmContractEventsTemplate,
    EvmContractEvents
);
template_wrapper!(EvmAbiFilterArgs, EvmAbiFilterTemplate, EvmAbiFilter);
template_wrapper!(
    SolanaWalletFilterArgs,
    SolanaWalletFilterTemplate,
    SolanaWalletFilter
);
template_wrapper!(
    BitcoinWalletFilterArgs,
    BitcoinWalletFilterTemplate,
    BitcoinWalletFilter
);
template_wrapper!(
    XrplWalletFilterArgs,
    XrplWalletFilterTemplate,
    XrplWalletFilter
);
template_wrapper!(
    HyperliquidWalletEventsFilterArgs,
    HyperliquidWalletEventsFilterTemplate,
    HyperliquidWalletEventsFilter
);
template_wrapper!(
    StellarWalletTransactionsFilterArgs,
    StellarWalletTransactionsFilterTemplate,
    StellarWalletTransactionsSourceAccountFilter
);

pub fn extract_template_args(obj: &Bound<'_, PyAny>) -> PyResult<TemplateArgs> {
    if let Ok(v) = obj.extract::<EvmWalletFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<EvmContractEventsArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<EvmAbiFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<SolanaWalletFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<BitcoinWalletFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<XrplWalletFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<HyperliquidWalletEventsFilterArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StellarWalletTransactionsFilterArgs>() {
        return Ok(v.to_core());
    }
    let received = obj
        .get_type()
        .name()
        .map_or_else(|_| "<unknown>".to_string(), |n| n.to_string());
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
        "template_args must be one of EvmWalletFilterArgs, EvmContractEventsArgs, \
         EvmAbiFilterArgs, SolanaWalletFilterArgs, BitcoinWalletFilterArgs, \
         XrplWalletFilterArgs, HyperliquidWalletEventsFilterArgs, \
         StellarWalletTransactionsFilterArgs — got {received}"
    )))
}
