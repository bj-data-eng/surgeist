//! Qualified HTML captures and bounded intrinsic semantic queries.
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

use surgeist_html::bounded;
use surgeist_html::*;
mod base;
mod bidi;
mod capture;
mod semantics;
pub use base::*;
pub use capture::*;
pub use semantics::*;
