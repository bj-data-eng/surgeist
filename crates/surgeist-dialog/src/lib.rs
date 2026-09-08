//! Native dialog services for Surgeist.
//!
//! This module wraps host file-picker APIs behind a small Surgeist-owned API.
//! It is intentionally separate from `surgeist::window`: dialogs are transient
//! host UI requested by the app, not app-owned winit windows.

mod backend;
mod error;
mod fake;
mod file;
mod options;
mod system;

#[cfg(test)]
mod tests;

pub use backend::Backend;
pub use error::{Error, ErrorCode, Result};
pub use fake::{Call, FakeBackend};
pub use file::FileDialog;
pub use options::{Filter, Options};
pub use system::SystemBackend;
