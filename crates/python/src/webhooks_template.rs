use pyo3::prelude::*;
use pyo3_stub_gen::derive::{gen_stub_pyclass, gen_stub_pymethods};
use quicknode_sdk::webhooks::{
    BitcoinWalletFilterByListTemplate, BitcoinWalletFilterInput, BitcoinWalletFilterTemplate,
    EvmAbiFilterByListTemplate, EvmAbiFilterInput, EvmAbiFilterTemplate,
    EvmContractEventsByListTemplate, EvmContractEventsInput, EvmContractEventsTemplate,
    EvmWalletFilterByListTemplate, EvmWalletFilterInput, EvmWalletFilterTemplate,
    HyperliquidWalletEventsFilterByListTemplate, HyperliquidWalletEventsFilterInput,
    HyperliquidWalletEventsFilterTemplate, SolanaWalletFilterByListTemplate,
    SolanaWalletFilterInput, SolanaWalletFilterTemplate,
    StellarWalletTransactionsFilterByListTemplate, StellarWalletTransactionsFilterInput,
    StellarWalletTransactionsFilterTemplate, TemplateArgs, XrplWalletFilterByListTemplate,
    XrplWalletFilterInput, XrplWalletFilterTemplate,
};

// Per-template typed wrappers. PyO3 cannot represent a Rust enum-with-data,
// so each variant of core's TemplateArgs is exposed as its own #[pyclass]
// here. extract_template_args() reassembles the enum at the FFI boundary.
//
// Each template has two wrappers: an inline `*Args` (wraps the inline value
// struct) and a `*ByListArgs` (wraps the by-list reference struct). At the
// FFI boundary both wrap into TemplateArgs::<Variant>(<Input>::Inline | ByList(...)).

macro_rules! template_wrapper {
    ($name:ident, $attrs:ident, $variant:ident, $input:ident, $sub:ident) => {
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
            pub fn to_core(&self) -> TemplateArgs {
                TemplateArgs::$variant($input::$sub(self.attrs.clone()))
            }
        }
    };
}

// Inline-form wrappers (matched first in extract_template_args below since
// they're more specific than the ByList wrappers on PyO3's extract path).
template_wrapper!(
    EvmWalletFilterArgs,
    EvmWalletFilterTemplate,
    EvmWalletFilter,
    EvmWalletFilterInput,
    Inline
);
template_wrapper!(
    EvmContractEventsArgs,
    EvmContractEventsTemplate,
    EvmContractEvents,
    EvmContractEventsInput,
    Inline
);
template_wrapper!(
    EvmAbiFilterArgs,
    EvmAbiFilterTemplate,
    EvmAbiFilter,
    EvmAbiFilterInput,
    Inline
);
template_wrapper!(
    SolanaWalletFilterArgs,
    SolanaWalletFilterTemplate,
    SolanaWalletFilter,
    SolanaWalletFilterInput,
    Inline
);
template_wrapper!(
    BitcoinWalletFilterArgs,
    BitcoinWalletFilterTemplate,
    BitcoinWalletFilter,
    BitcoinWalletFilterInput,
    Inline
);
template_wrapper!(
    XrplWalletFilterArgs,
    XrplWalletFilterTemplate,
    XrplWalletFilter,
    XrplWalletFilterInput,
    Inline
);
template_wrapper!(
    HyperliquidWalletEventsFilterArgs,
    HyperliquidWalletEventsFilterTemplate,
    HyperliquidWalletEventsFilter,
    HyperliquidWalletEventsFilterInput,
    Inline
);
template_wrapper!(
    StellarWalletTransactionsFilterArgs,
    StellarWalletTransactionsFilterTemplate,
    StellarWalletTransactionsSourceAccountFilter,
    StellarWalletTransactionsFilterInput,
    Inline
);

// ByList-form wrappers.
template_wrapper!(
    EvmWalletFilterByListArgs,
    EvmWalletFilterByListTemplate,
    EvmWalletFilter,
    EvmWalletFilterInput,
    ByList
);
template_wrapper!(
    EvmContractEventsByListArgs,
    EvmContractEventsByListTemplate,
    EvmContractEvents,
    EvmContractEventsInput,
    ByList
);
template_wrapper!(
    EvmAbiFilterByListArgs,
    EvmAbiFilterByListTemplate,
    EvmAbiFilter,
    EvmAbiFilterInput,
    ByList
);
template_wrapper!(
    SolanaWalletFilterByListArgs,
    SolanaWalletFilterByListTemplate,
    SolanaWalletFilter,
    SolanaWalletFilterInput,
    ByList
);
template_wrapper!(
    BitcoinWalletFilterByListArgs,
    BitcoinWalletFilterByListTemplate,
    BitcoinWalletFilter,
    BitcoinWalletFilterInput,
    ByList
);
template_wrapper!(
    XrplWalletFilterByListArgs,
    XrplWalletFilterByListTemplate,
    XrplWalletFilter,
    XrplWalletFilterInput,
    ByList
);
template_wrapper!(
    HyperliquidWalletEventsFilterByListArgs,
    HyperliquidWalletEventsFilterByListTemplate,
    HyperliquidWalletEventsFilter,
    HyperliquidWalletEventsFilterInput,
    ByList
);
template_wrapper!(
    StellarWalletTransactionsFilterByListArgs,
    StellarWalletTransactionsFilterByListTemplate,
    StellarWalletTransactionsSourceAccountFilter,
    StellarWalletTransactionsFilterInput,
    ByList
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
    if let Ok(v) = obj.extract::<EvmWalletFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<EvmContractEventsByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<EvmAbiFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<SolanaWalletFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<BitcoinWalletFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<XrplWalletFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<HyperliquidWalletEventsFilterByListArgs>() {
        return Ok(v.to_core());
    }
    if let Ok(v) = obj.extract::<StellarWalletTransactionsFilterByListArgs>() {
        return Ok(v.to_core());
    }
    let received = obj
        .get_type()
        .name()
        .map_or_else(|_| "<unknown>".to_string(), |n| n.to_string());
    Err(PyErr::new::<pyo3::exceptions::PyTypeError, _>(format!(
        "template_args must be one of the inline `*Args` or `*ByListArgs` \
         wrappers — got {received}"
    )))
}
