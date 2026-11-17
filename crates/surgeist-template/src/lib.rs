//! Template and DSL-facing contracts for Surgeist.
//!
//! Strict parsing, native authored AST construction, registry validation and
//! symbolic Rust generation. HTML metadata supplies an explicit known-name
//! registry profile; concrete templates queue ordinary canonical DOM edits.
//! Runtime evaluation and host policy belong to their consuming owners.
//!
//! The pipeline is three functions at the crate root: [`parse_template`]
//! turns source text into an [`ast::TemplateDocument`], [`validate_template`]
//! checks it against explicit [`registry`] vocabularies and returns a
//! [`validation::ValidatedTemplate`], and [`render_to_rust`] emits Rust source
//! that constructs the authored document again. The modules hold the values
//! those functions exchange:
//!
//! - [`ast`]: the authored tree and its native constructors.
//! - [`expr`]: symbolic expressions and variable paths.
//! - [`name`]: validated element, attribute, component and variable names.
//! - [`span`]: diagnostic source positions and spans.
//! - [`parse`]: diagnostics reported by [`parse_template`].
//! - [`registry`]: native-element and component vocabularies.
//! - [`validation`]: the validated tree and validation diagnostics.
//! - [`dom`]: static HTML composition into a caller's DOM batch.
//!
//! Each public item has exactly one path.

#![forbid(unsafe_code)]

pub mod ast;
pub mod dom;
pub mod expr;
pub mod name;
pub mod parse;
pub mod registry;
pub mod span;
pub mod validation;

mod error;
mod lexer;
mod parser;
mod render;
mod validate;

#[doc(inline)]
pub use parser::parse_template;
#[doc(inline)]
pub use render::render_to_rust;
#[doc(inline)]
pub use validate::validate_template;

/// Crate identity string used by smoke tests and API artifacts.
pub const CRATE_NAME: &str = "surgeist-template";

#[cfg(test)]
mod tests {
    use super::CRATE_NAME;

    #[test]
    fn exposes_crate_identity() {
        assert_eq!(CRATE_NAME, "surgeist-template");
    }
}
