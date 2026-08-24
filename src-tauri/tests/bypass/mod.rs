//! Adversarial bypass tests — the enforcement boundary is treated as an
//! attack surface and tested with known bypass techniques.
//!
//! Run: `cargo test --test bypass_runner`
//! CI:  runs automatically via `cargo test`.

pub mod config;
pub mod path;
pub mod process;
