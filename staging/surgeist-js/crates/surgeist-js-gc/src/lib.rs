//! Runtime-owned JavaScript heap with generational identities and tracing collection.
#![forbid(unsafe_code)]
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::module_name_repetitions,
    clippy::redundant_pub_crate,
    clippy::let_unit_value
)]

extern crate self as surgeist_js_gc;

pub mod heap;
