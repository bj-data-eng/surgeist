//! Browser-parity tooling for `surgeist-layout`.
//!
//! This crate owns the checked-in browser-parity corpus, the golden generator
//! (`surgeist-layout-generate`), the qualification driver (`qualify-root-context`)
//! and the test support that compares `surgeist-layout` output with captured
//! browser geometry. It depends on `surgeist-layout`'s public API only; layout
//! does not depend on it.
//!
//! The default build exposes:
//!
//! - The module [`portable_qualification`]. Its function
//!   [`admit_registered`](portable_qualification::admit_registered) authenticates
//!   the registered editions under a corpus owner directory, and the returned
//!   registry runs selected cases and lends each completed placement to an
//!   observer;
//! - The constant [`AHEM_TTF`], the pinned Ahem font bytes of the captures;
//! - The function [`prepared_atomic_percentage_root`], the artifact root of the
//!   prepared atomic percentage edition.
//!
//! The `browser-corpus` feature adds the generator and the qualification driver,
//! which need the browser engine, as `run_generator` and `run_qualification`.
//! The `surgeist-layout-generate` binary and the `qualify-root-context` example
//! are separate targets that call these two functions.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod browser_parity;
mod browser_parity_modules;
#[cfg(feature = "browser-corpus")]
mod generate;
#[cfg(feature = "browser-corpus")]
mod qualify;

use browser_parity_modules::*;

#[doc(inline)]
pub use browser_parity::portable_qualification;
use browser_parity::support::capture_authority::SpecializedProfile;

/// The crate-relative directory of the checked-in prepared atomic percentage edition.
#[must_use]
pub fn prepared_atomic_percentage_root() -> &'static str {
    SpecializedProfile::PreparedAtomicPercentage.artifact_root()
}

/// The pinned Ahem font bytes that the registered browser captures were taken with.
pub const AHEM_TTF: &[u8] = pinned_font::BYTES;

#[cfg(feature = "browser-corpus")]
#[doc(inline)]
pub use generate::run as run_generator;
#[cfg(feature = "browser-corpus")]
#[doc(inline)]
pub use qualify::run as run_qualification;
