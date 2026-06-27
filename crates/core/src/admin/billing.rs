#[cfg(feature = "node")]
use napi_derive::napi;
#[cfg(feature = "python")]
use pyo3::pyclass;
#[cfg(feature = "python")]
use pyo3_stub_gen::derive::gen_stub_pyclass;
use serde::{Deserialize, Serialize};

/// A single line item on an invoice.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLine {
    /// Human-readable description of the line item.
    pub description: String,
    /// Line item amount in the smallest currency unit.
    pub amount: i64,
}

/// An invoice issued to the account.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice identifier.
    pub id: String,
    /// Payment status (e.g. `paid`, `open`).
    pub status: String,
    /// Reason the invoice was generated (e.g. `subscription_cycle`).
    pub billing_reason: String,
    /// Line items contributing to the invoice total.
    #[serde(default)]
    pub lines: Vec<InvoiceLine>,
    /// Amount due in the smallest currency unit.
    pub amount_due: i64,
    /// Amount already paid in the smallest currency unit.
    pub amount_paid: i64,
    /// Start of the billing period (Unix timestamp).
    pub period_start: i64,
    /// End of the billing period (Unix timestamp).
    pub period_end: i64,
    /// Timestamp when the invoice was created (Unix timestamp).
    pub created: i64,
    /// Subtotal before taxes and adjustments.
    pub subtotal: i64,
}

/// Response from `list_invoices`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInvoicesResponse {
    /// Invoice data payload.
    pub data: Option<ListInvoicesData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Invoice list wrapper.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListInvoicesData {
    /// Invoices on the account.
    #[serde(default)]
    pub invoices: Vec<Invoice>,
}

/// A payment recorded on the account.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payment {
    /// Payment amount as a string in the account's currency.
    pub amount: String,
    /// Last four digits of the card used for the payment.
    pub card_last_4: Option<String>,
    /// Timestamp when the payment was recorded.
    pub created_at: String,
    /// Currency code (e.g. `usd`).
    pub currency: String,
    /// Payment status.
    pub status: String,
    /// Portion of the payment attributed to marketplace spending.
    pub marketplace_amount: Option<String>,
}

/// Response from `list_payments`.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPaymentsResponse {
    /// Payment data payload.
    pub data: Option<ListPaymentsData>,
    /// Error message when the request did not succeed.
    pub error: Option<String>,
}

/// Payment list wrapper.
#[cfg_attr(feature = "python", gen_stub_pyclass)]
#[cfg_attr(feature = "python", pyclass(get_all, set_all))]
#[cfg_attr(feature = "node", napi(object))]
#[cfg_attr(feature = "go", derive(uniffi::Record))]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListPaymentsData {
    /// Payments on the account.
    #[serde(default)]
    pub payments: Vec<Payment>,
}
