//! jianyi — a portable, deterministic, offline I Ching and tarot calculator.
//!
//! This library exposes the calculation engine; the `jianyi` binary is a thin
//! CLI over it. The same operations are callable from a host (e.g. over FFI).

#[cfg(feature = "cli")]
pub mod error;
#[cfg(feature = "iching")]
pub mod iching;
#[cfg(feature = "cli")]
pub mod output;
pub mod tarot;
