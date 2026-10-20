//! Public-API integration tests for `surgeist-html-parser`, linked as one binary.
//!
//! The module tree groups tests by the domain they exercise. The crate re-exports
//! its whole API flat from the root, so the domains follow its current source
//! modules: sources, requests and limits, parsing and reports, and construction.

#![forbid(unsafe_code)]

#[path = "parser/mod.rs"]
mod parser;
