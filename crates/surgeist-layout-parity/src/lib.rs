//! Browser-parity tooling for `surgeist-layout`.
//!
//! This crate owns the checked-in browser-parity corpus, the golden generator
//! (`surgeist-layout-generate`), the qualification driver (`qualify-root-context`)
//! and the test support that compares `surgeist-layout` output with captured
//! browser geometry. It depends on `surgeist-layout`'s public API only; layout
//! does not depend on it.
//!
//! The public surface is the registered-edition admission that other crates use
//! to qualify one captured case, and the entry points of the two executables.
//! Everything else is private to the crate's own binary, example and tests.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod browser_parity;
mod browser_parity_modules;
mod generate;
mod qualify;

use browser_parity_modules::*;

pub use browser_parity::portable_qualification;
use browser_parity::support::capture_authority::SpecializedProfile;

/// The crate-relative directory of the checked-in prepared atomic percentage edition.
#[must_use]
pub fn prepared_atomic_percentage_root() -> &'static str {
    SpecializedProfile::PreparedAtomicPercentage.artifact_root()
}

/// The pinned Ahem font bytes that the registered browser captures were taken with.
pub const AHEM_TTF: &[u8] = pinned_font::BYTES;

pub use generate::run as run_generator;
pub use qualify::run as run_qualification;
