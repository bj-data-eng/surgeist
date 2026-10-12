//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
#![forbid(unsafe_code)]

mod attributes;
mod bidi;
mod capture;
mod color;
mod construction;
mod custom_names;
mod interpretation;
mod numeric;
#[cfg(test)]
mod parser;
mod request;
mod semantics;
mod settings;
mod source;
mod temporal;
mod tokens;
mod values;
mod vocabulary;

pub use attributes::*;
pub use capture::*;
pub use color::*;
pub use construction::*;
pub use custom_names::*;
pub use interpretation::*;
pub use numeric::*;
pub use request::*;
pub use semantics::*;
pub use settings::*;
pub use source::*;
pub use temporal::*;
pub use tokens::*;
pub use values::*;
pub use vocabulary::*;
