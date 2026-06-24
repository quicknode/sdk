use magnus::{
    exception::ExceptionClass, prelude::*, value::Opaque, Object, RModule, RObject, Ruby, Value,
};
use quicknode_sdk::errors::{HttpKind, SdkError};
use std::sync::OnceLock;

// Class handles are captured at #[magnus::init] time. OnceLock<Opaque<T>> is
// the Magnus-blessed pattern for stashing Ruby values across calls — the
// Opaque wrapper asserts Send/Sync at access time via Ruby::get_inner.
struct ErrorClasses {
    // Held solely to register the base class on the Ruby side; never read
    // back because subclasses dispatch directly.
    #[allow(dead_code)]
    base: Opaque<ExceptionClass>,
    config: Opaque<ExceptionClass>,
    http: Opaque<ExceptionClass>,
    timeout: Opaque<ExceptionClass>,
    connection: Opaque<ExceptionClass>,
    api: Opaque<ExceptionClass>,
    decode: Opaque<ExceptionClass>,
    rpc: Opaque<ExceptionClass>,
}

static CLASSES: OnceLock<ErrorClasses> = OnceLock::new();

pub fn init(ruby: &Ruby, module: &RModule) -> Result<(), magnus::Error> {
    let std_err = ruby.exception_standard_error();
    let base = module.define_error("Error", std_err)?;
    let config = module.define_error("ConfigError", base)?;
    let http = module.define_error("HttpError", base)?;
    let timeout = module.define_error("TimeoutError", http)?;
    let connection = module.define_error("ConnectionError", http)?;
    let api = module.define_error("ApiError", base)?;
    let decode = module.define_error("DecodeError", base)?;
    let rpc = module.define_error("RpcError", base)?;

    // attr_reader :status, :body on ApiError; :body on DecodeError;
    // :code, :message on RpcError
    api.define_method("status", magnus::method!(read_status, 0))?;
    api.define_method("body", magnus::method!(read_body, 0))?;
    decode.define_method("body", magnus::method!(read_body, 0))?;
    rpc.define_method("code", magnus::method!(read_code, 0))?;
    rpc.define_method("message", magnus::method!(read_message, 0))?;

    CLASSES
        .set(ErrorClasses {
            base: base.into(),
            config: config.into(),
            http: http.into(),
            timeout: timeout.into(),
            connection: connection.into(),
            api: api.into(),
            decode: decode.into(),
            rpc: rpc.into(),
        })
        .map_err(|_| {
            magnus::Error::new(
                ruby.exception_runtime_error(),
                "error classes already initialized",
            )
        })?;
    Ok(())
}

fn read_status(rb_self: magnus::Exception) -> Result<Value, magnus::Error> {
    as_object(rb_self)?.ivar_get("@status")
}

fn read_body(rb_self: magnus::Exception) -> Result<Value, magnus::Error> {
    as_object(rb_self)?.ivar_get("@body")
}

fn read_code(rb_self: magnus::Exception) -> Result<Value, magnus::Error> {
    as_object(rb_self)?.ivar_get("@code")
}

fn read_message(rb_self: magnus::Exception) -> Result<Value, magnus::Error> {
    as_object(rb_self)?.ivar_get("@message")
}

fn as_object(exc: magnus::Exception) -> Result<RObject, magnus::Error> {
    RObject::from_value(exc.as_value()).ok_or_else(|| {
        let ruby = Ruby::get().expect("read_ivar called outside a Ruby thread");
        magnus::Error::new(
            ruby.exception_runtime_error(),
            "exception is not a plain Ruby object",
        )
    })
}

fn classes(ruby: &Ruby) -> &ErrorClasses {
    let _ = ruby;
    CLASSES
        .get()
        .expect("QuicknodeSdk error classes not initialized")
}

#[allow(clippy::needless_pass_by_value)]
pub fn map_err(e: SdkError) -> magnus::Error {
    // map_err is only called from within Ruby-initiated SDK calls, so Ruby
    // is always attached to this thread here.
    let ruby = Ruby::get().expect("map_err called outside a Ruby thread");
    let c = classes(&ruby);
    let msg = e.to_string();
    match &e {
        SdkError::Config(_) | SdkError::UrlParse(_) => {
            magnus::Error::new(ruby.get_inner(c.config), msg)
        }
        SdkError::Api { status, body } => build_with_ivars(
            &ruby,
            ruby.get_inner(c.api),
            &msg,
            Some(status.as_u16()),
            Some(body.clone()),
        ),
        SdkError::Decode { body, .. } => build_with_ivars(
            &ruby,
            ruby.get_inner(c.decode),
            &msg,
            None,
            Some(body.clone()),
        ),
        SdkError::Rpc { code, message } => {
            build_rpc(&ruby, ruby.get_inner(c.rpc), &msg, *code, message.clone())
        }
        SdkError::Http(_) => {
            let cls = match e.http_kind() {
                Some(HttpKind::Timeout) => ruby.get_inner(c.timeout),
                Some(HttpKind::Connect) => ruby.get_inner(c.connection),
                _ => ruby.get_inner(c.http),
            };
            magnus::Error::new(cls, msg)
        }
    }
}

fn build_rpc(
    ruby: &Ruby,
    class: ExceptionClass,
    msg: &str,
    code: i64,
    message: String,
) -> magnus::Error {
    match class.new_instance((msg,)) {
        Ok(exc) => {
            if let Some(obj) = RObject::from_value(exc.as_value()) {
                let _ = obj.ivar_set("@code", code);
                let _ = obj.ivar_set("@message", message);
            }
            magnus::Error::from(exc)
        }
        Err(_) => magnus::Error::new(ruby.exception_runtime_error(), msg.to_string()),
    }
}

fn build_with_ivars(
    ruby: &Ruby,
    class: ExceptionClass,
    msg: &str,
    status: Option<u16>,
    body: Option<String>,
) -> magnus::Error {
    match class.new_instance((msg,)) {
        Ok(exc) => {
            if let Some(obj) = RObject::from_value(exc.as_value()) {
                if let Some(s) = status {
                    let _ = obj.ivar_set("@status", s);
                }
                if let Some(b) = body {
                    let _ = obj.ivar_set("@body", b);
                }
            }
            magnus::Error::from(exc)
        }
        Err(_) => magnus::Error::new(ruby.exception_runtime_error(), msg.to_string()),
    }
}
