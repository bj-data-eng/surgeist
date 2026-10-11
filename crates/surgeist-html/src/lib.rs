//! HTML-owned construction values and language contracts.
//!
//! The selected WHATWG HTML edition defines language behavior. Immutable values
//! preserve exact UTF-16 code units; live document identity and mutation belong
//! to the consuming DOM subsystem.
#![forbid(unsafe_code)]

mod construction;
mod source;
mod values;
mod vocabulary;

pub use construction::*;
pub use source::*;
pub use values::*;
pub use vocabulary::*;
