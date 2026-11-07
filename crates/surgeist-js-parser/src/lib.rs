//! Boa's **`surgeist_js_parser`** crate is a parser targeting the latest [ECMAScript language specification][spec].
//!
//! # Crate Overview
//!
//! This crate contains implementations of a [`Lexer`] and a [`Parser`] for the **ECMAScript**
//! language. The [lexical grammar][lex] and the [syntactic grammar][grammar] being targeted are
//! fully defined in the specification. See the links provided for more information.
//!
//! [spec]: https://tc39.es/ecma262
//! [lex]: https://tc39.es/ecma262/#sec-ecmascript-language-lexical-grammar
//! [grammar]: https://tc39.es/ecma262/#sec-ecmascript-language-expressions
#![forbid(unsafe_code)]
#![cfg_attr(test, allow(clippy::needless_raw_string_hashes))] // Makes strings a bit more copy-pastable
#![cfg_attr(not(test), forbid(clippy::unwrap_used))]
#![allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::cognitive_complexity,
    clippy::let_unit_value,
    clippy::redundant_pub_crate,
    clippy::struct_field_names
)]

mod diagnostic;
pub mod error;
pub mod frontend;
pub mod lexer;
pub mod parser;
pub mod source;

pub use error::Error;
pub use lexer::Lexer;
pub use parser::Parser;
pub use source::Source;
