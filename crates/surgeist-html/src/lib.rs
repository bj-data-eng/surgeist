//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
#![forbid(unsafe_code)]

mod attributes;
mod capture;
mod color;
mod construction;
mod interpretation;
mod numeric;
#[cfg(test)]
mod parser;
mod source;
mod temporal;
mod tokens;
mod values;
mod vocabulary;

pub use attributes::*;
pub use capture::*;
pub use color::*;
pub use construction::*;
pub use interpretation::*;
pub use numeric::*;
pub use source::*;
pub use temporal::*;
pub use tokens::*;
pub use values::*;
pub use vocabulary::*;
