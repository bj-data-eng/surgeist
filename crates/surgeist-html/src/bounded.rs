//! Controlled HTML-owned operations, driven by a caller's admission policy.
//!
//! Actions describe their work before reading their operands. This module owns
//! value invariants; callers retain cancellation, work and allocation accounting.
//! Each submodule mirrors a domain module.

pub mod attributes;
pub mod construction;
pub mod form;
pub mod frame;
pub mod input;
pub mod lexical;
pub mod media;
pub mod microdata;
pub mod native;
pub mod numeric;
pub mod parsing;
pub mod rel;
pub mod script;
pub mod security;
pub mod select;
pub mod template;
pub mod temporal;
pub mod values;
