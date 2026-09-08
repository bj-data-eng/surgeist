//! Shared test infrastructure and integration fixtures for Surgeist.
//!
//! This crate is intentionally test-facing. Keep reusable harness code,
//! fixture metadata, and cross-crate verification support here when that
//! reduces context pressure in implementation crates.
//!
//! Production Surgeist crates should not depend on this crate without an
//! explicit boundary decision from the top-level coordinator.

#![forbid(unsafe_code)]

pub mod fixtures;
