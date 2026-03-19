#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::Deserialize;

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct InvoiceLine {
    pub description: String,
    pub amount: i64,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub status: String,
    pub billing_reason: String,
    #[serde(default)]
    pub lines: Vec<InvoiceLine>,
    pub amount_due: i64,
    pub amount_paid: i64,
    pub period_start: i64,
    pub period_end: i64,
    pub created: i64,
    pub subtotal: i64,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct ListInvoicesResponse {
    pub data: Option<ListInvoicesData>,
    pub error: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct ListInvoicesData {
    #[serde(default)]
    pub invoices: Vec<Invoice>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct Payment {
    pub amount: String,
    pub card_last_4: Option<String>,
    pub created_at: String,
    pub currency: String,
    pub status: String,
    pub marketplace_amount: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct ListPaymentsResponse {
    pub data: Option<ListPaymentsData>,
    pub error: Option<String>,
}

#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[derive(Debug, Clone, Deserialize)]
pub struct ListPaymentsData {
    #[serde(default)]
    pub payments: Vec<Payment>,
}
