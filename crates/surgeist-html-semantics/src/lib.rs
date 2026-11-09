//! Qualified HTML captures and bounded intrinsic semantic queries.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

#[macro_use]
mod identity;
mod accounting;
#[cfg(test)]
mod accounting_fault_tests;
mod base;
mod bidi;
mod capture;
mod controlled_option_value;
mod hyperlink;
#[cfg(test)]
mod schema_tests;
mod semantics;
pub use accounting::HtmlSemanticAccounting;
pub use base::*;
pub use capture::*;
pub use controlled_option_value::*;
pub use hyperlink::*;
pub use semantics::*;
