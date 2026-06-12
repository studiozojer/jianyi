//! jianyi — a portable, deterministic, offline I Ching and tarot calculator.
//!
//! This library exposes the calculation engine; the `jianyi` binary is a thin
//! CLI over it. The same operations are callable from a host (e.g. over FFI).

pub mod error;
pub mod iching;
pub mod output;
pub mod tarot;
