//! jianyi — a portable, deterministic, offline I Ching and tarot calculator.
//!
//! This library exposes the calculation engine; the `jianyi` binary is a thin
//! CLI over it. The same operations are callable from a host (e.g. over FFI).

#[cfg(feature = "cli")]
pub mod error;
#[cfg(feature = "ffi")]
pub mod ffi;
#[cfg(feature = "iching")]
pub mod iching;
#[cfg(feature = "iching")]
pub mod json_api;
#[cfg(feature = "cli")]
pub mod output;
pub mod tarot;
#[cfg(feature = "wasm")]
pub mod wasm;
