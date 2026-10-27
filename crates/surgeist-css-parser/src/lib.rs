#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
//! CSS Syntax Module Level 3 tokenization and parsing for Surgeist.
//!
//! This crate owns the language-independent layer that every CSS grammar in
//! Surgeist sits on: decoding and preprocessing the input stream, the
//! tokenizer, the parsing results and token streams, the parser entry points
//! and algorithms, the `An+B` microsyntax, and token serialization. It gives
//! syntax no CSS meaning. Properties, values, rules, selectors, queries and
//! recovery diagnostics belong to `surgeist-css`, which is to build CSS
//! language interpretation on this crate.
//!
//! The crate depends on no other Surgeist crate. The dependency is to run from
//! `surgeist-css` to this crate, never the other way, because Syntax 3 is a
//! layer below every CSS grammar rather than a consumer of CSS values.
//! `surgeist-css` does not depend on this crate yet.
//!
//! The behavioral authority is the pinned CSS Syntax Module Level 3 edition
//! recorded in the repository's `references/SOURCE-EDITIONS.md` (the "CSS
//! Syntax 3 edition" section), which also names its local copy. The crate's
//! `docs/conformance-coverage.md` maps each clause of that edition to the
//! tests that exercise it and states what is not claimed.
//!
//! # Current state
//!
//! The crate is newly created and exposes no items yet. Its public API is
//! introduced module by module, with the specification-derived tests for each
//! clause, and the coverage map claims only what those tests establish.
