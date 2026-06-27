//! Thin cdylib/staticlib shim for the Go binding.
//!
//! All UniFFI types, exports, and `setup_scaffolding!` live in
//! `quicknode-sdk`'s `go` module (they must share a crate with the
//! `#[derive(uniffi::Record)]` annotations on the core data types). This crate
//! exists only to produce the linkable native artifact: re-exporting the facade
//! pulls the core crate's `#[no_mangle]` FFI scaffolding symbols into this
//! cdylib/staticlib so `uniffi-bindgen-go` can find them and so the generated
//! Go can link against them.

pub use quicknode_sdk::go::*;
