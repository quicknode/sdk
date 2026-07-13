use pyo3::{
    create_exception,
    exceptions::{PyException, PyValueError},
    prelude::*,
};
use quicknode_sdk::errors::{HttpKind, SdkError};

// Invalid-argument parse errors for user-supplied strings (enum values, etc.)
// stay as PyValueError — they're argument errors, not SDK errors.
#[allow(clippy::needless_pass_by_value)]
pub fn map_parse_err(e: serde_json::Error) -> PyErr {
    PyValueError::new_err(e.to_string())
}

// Conversion failures between Python objects and JSON (rpc params/results) are
// argument errors, surfaced as PyValueError like other parse failures.
#[allow(clippy::needless_pass_by_value)]
pub fn map_pythonize_err(e: pythonize::PythonizeError) -> PyErr {
    PyValueError::new_err(e.to_string())
}

create_exception!(_core, QuicknodeError, PyException);
create_exception!(_core, ConfigError, QuicknodeError);
create_exception!(_core, HttpError, QuicknodeError);
create_exception!(_core, TimeoutError, HttpError);
create_exception!(_core, ConnectionError, HttpError);
create_exception!(_core, ApiError, QuicknodeError);
create_exception!(_core, DecodeError, QuicknodeError);
create_exception!(_core, RpcError, QuicknodeError);
// Payment-lane errors. PaymentError is the family base; PaymentRejectedError
// carries the gateway status/body like ApiError; PaymentIndeterminateError is
// its own class so a caller can catch "may have been charged — do not retry"
// distinctly from every other failure.
create_exception!(_core, PaymentError, QuicknodeError);
create_exception!(_core, PaymentUnsupportedError, PaymentError);
create_exception!(_core, PaymentRejectedError, PaymentError);
create_exception!(_core, PaymentIndeterminateError, PaymentError);

#[allow(clippy::needless_pass_by_value)]
pub fn map_sdk_err(e: SdkError) -> PyErr {
    let msg = e.to_string();
    match &e {
        SdkError::Config(_) | SdkError::UrlParse(_) => ConfigError::new_err(msg),
        SdkError::Api { status, body } => {
            // Stash status/body on the exception instance so callers can
            // branch on them without regex-parsing the message.
            let status = status.as_u16();
            let body = body.clone();
            Python::attach(|py| {
                let err = ApiError::new_err(msg);
                let val = err.value(py);
                let _ = val.setattr("status", status);
                let _ = val.setattr("body", body);
                err
            })
        }
        SdkError::Decode { body, .. } => {
            let body = body.clone();
            Python::attach(|py| {
                let err = DecodeError::new_err(msg);
                let val = err.value(py);
                let _ = val.setattr("body", body);
                err
            })
        }
        SdkError::Rpc { code, message } => {
            let code = *code;
            let message = message.clone();
            Python::attach(|py| {
                let err = RpcError::new_err(msg);
                let val = err.value(py);
                let _ = val.setattr("code", code);
                let _ = val.setattr("message", message);
                err
            })
        }
        SdkError::Http(_) => match e.http_kind() {
            Some(HttpKind::Timeout) => TimeoutError::new_err(msg),
            Some(HttpKind::Connect) => ConnectionError::new_err(msg),
            _ => HttpError::new_err(msg),
        },
        SdkError::PaymentUnsupported { .. } => PaymentUnsupportedError::new_err(msg),
        SdkError::PaymentRejected { status, body } => {
            let status = *status;
            let body = body.clone();
            Python::attach(|py| {
                let err = PaymentRejectedError::new_err(msg);
                let val = err.value(py);
                let _ = val.setattr("status", status);
                let _ = val.setattr("body", body);
                err
            })
        }
        SdkError::PaymentIndeterminate => PaymentIndeterminateError::new_err(msg),
    }
}

pub fn add_to_module(m: &Bound<'_, PyModule>) -> PyResult<()> {
    let py = m.py();
    m.add("QuicknodeError", py.get_type::<QuicknodeError>())?;
    m.add("ConfigError", py.get_type::<ConfigError>())?;
    m.add("HttpError", py.get_type::<HttpError>())?;
    m.add("TimeoutError", py.get_type::<TimeoutError>())?;
    m.add("ConnectionError", py.get_type::<ConnectionError>())?;
    m.add("ApiError", py.get_type::<ApiError>())?;
    m.add("DecodeError", py.get_type::<DecodeError>())?;
    m.add("RpcError", py.get_type::<RpcError>())?;
    m.add("PaymentError", py.get_type::<PaymentError>())?;
    m.add(
        "PaymentUnsupportedError",
        py.get_type::<PaymentUnsupportedError>(),
    )?;
    m.add(
        "PaymentRejectedError",
        py.get_type::<PaymentRejectedError>(),
    )?;
    m.add(
        "PaymentIndeterminateError",
        py.get_type::<PaymentIndeterminateError>(),
    )?;
    Ok(())
}
