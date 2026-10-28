#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
//! CSS Syntax Module Level 3 tokenization and parsing for Surgeist.
//!
//! This crate owns the language-independent layer that every CSS grammar in
//! Surgeist sits on: decoding and preprocessing the input stream, the
//! tokenizer, the parsing results and token streams, the parser entry points
//! and algorithms, the `An+B` microsyntax, and token serialization. It gives
//! syntax no CSS meaning. Properties, values, rules, selectors, queries and
//! recovery diagnostics belong to `surgeist-css`, which builds CSS language
//! interpretation on this crate.
//!
//! The crate depends on no other Surgeist crate. The dependency runs from
//! `surgeist-css` to this crate, never the other way, because Syntax 3 is a
//! layer below every CSS grammar rather than a consumer of CSS values.
//!
//! The behavioral authority is the pinned CSS Syntax Module Level 3 edition
//! recorded in the repository's `references/SOURCE-EDITIONS.md` (the "CSS
//! Syntax 3 edition" section), which also names its local copy. The crate's
//! `docs/conformance-coverage.md` maps each clause of that edition to the
//! tests that exercise it and states what is not claimed.
//!
//! # Current state
//!
//! The crate's public API is introduced module by module, with the
//! specification-derived tests for each clause, and the coverage map claims
//! only what those tests establish. Today it offers:
//!
//! - [`lexer`]: the free-running [`Lexer`](lexer::Lexer), which tokenizes
//!   text as section 4.3 says and reports each token with its source interval
//!   and the places the specification calls parse errors, and
//!   [`lexer::protocol`], the stepwise form of the same state machine for
//!   hosts that admit each action, a narrow host-only contract rather than a
//!   general lexer API;
//! - [`Token`], and in [`token`] its kind, hash type flag and [`Number`]; and in
//!   [`unicode_range`] the value of the `<unicode-range-token>`;
//! - [`input`]: the stepwise [`InputCursor`](input::InputCursor), which decodes
//!   UTF-8 or UTF-16 units and applies the preprocessing of section 3.3 while
//!   keeping each scalar's original unit interval;
//! - [`lexical`]: the code-point classes of Syntax 3 section 4.2 and the
//!   checks of sections 4.3.8 to 4.3.11, as `const fn`s;
//! - [`Offset`]: a byte offset into the text a tokenizer reads;
//! - [`limits`]: the depth, node and byte limits of a build, the [`Meter`]
//!   contract through which a host admits work and storage, and the growth
//!   and copy helpers that charge it;
//! - [`match_ignore_ascii_case!`], ASCII case-insensitive matching of a string
//!   against literal arms.
//!
//! No document or parser exists yet.
//!
//! ```
//! use surgeist_css_parser::lexer::{Lexer, Options};
//! use surgeist_css_parser::Token;
//!
//! let mut lexer = Lexer::new("a{b:1px}", &Options::default());
//! let mut kinds = Vec::new();
//! while let Some(lexeme) = lexer.next_lexeme() {
//!     kinds.push(lexeme.token().kind());
//! }
//! assert_eq!(kinds.len(), 6);
//! assert!(lexer.errors().is_empty());
//! assert_eq!(Token::Colon.kind(), surgeist_css_parser::token::TokenKind::Colon);
//! ```
//!
//! [`Meter`]: limits::Meter
//! [`Number`]: token::Number

#[macro_use]
mod macros;
mod match_ascii;
mod offset;
mod syntax_token;

pub mod input;
pub mod lexer;
pub mod lexical;
pub mod limits;
pub mod token;
pub mod unicode_range;

pub use offset::Offset;
pub use syntax_token::Token;
