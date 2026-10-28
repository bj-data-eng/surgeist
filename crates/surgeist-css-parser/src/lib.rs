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
//! - [`Document`], the token tree of section 5.2, built by
//!   [`Document::parse`] from text or by a [`document::TreeBuilder`] from
//!   components a caller holds, with its navigation (lists, parent groups,
//!   locations, offsets) and an optional per-node payload;
//! - [`Stream`], the token stream of section 5.3 over a document, with its
//!   [`Site`] boundaries, [`stream::Mark`] restore points, the
//!   [`stream::StopToken`]s of section 5.5.7, and the errors a grammar over it
//!   reports, [`stream::StreamError`] and [`SyntaxError`];
//! - [`grammar`]: the parser entry points of section 5.4 and the rule
//!   algorithms of section 5.5, as methods of [`Stream`] and iterators of
//!   units that hand each consumed rule or declaration to a
//!   [`RuleGrammar`](grammar::RuleGrammar) or
//!   [`BlockGrammar`](grammar::BlockGrammar);
//! - [`anb`]: the `An+B` microsyntax of section 6 and its serialization;
//! - [`unicode_range`]: besides the token's value, the algorithm of section
//!   5.5.11 that tokenizes a `unicode-range` descriptor's text;
//! - [`serialize`]: the serialization of section 9, so that tokens written
//!   out read as the same tokens again: escaping, the comment between tokens
//!   that would run together, and a writer that applies both;
//! - [`match_ignore_ascii_case!`], ASCII case-insensitive matching of a string
//!   against literal arms.
//!
//! ```
//! use surgeist_css_parser::lexer::Options;
//! use surgeist_css_parser::limits::{DepthExcess, Limits};
//! use surgeist_css_parser::{Document, SyntaxError};
//!
//! // Parse `a{b:1px}`: an ident, then a `{}`-block of three tokens.
//! let document: Document =
//!     Document::parse("a{b:1px}", &Options::default(), Limits::default(), DepthExcess::Fail)?;
//! let mut stream = document.root();
//! assert_eq!(stream.consume_ident().ok(), Some("a"));
//! stream.consume_token().unwrap(); // the `{`; its block is waiting to be entered
//! let name = stream
//!     .within_block(|block| {
//!         let name = block.consume_ident()?;
//!         block.consume_colon()?;
//!         block.consume_token()?; // the dimension
//!         Ok::<_, SyntaxError<'_, ()>>(name)
//!     })
//!     .unwrap();
//! assert_eq!(name, "b");
//! assert!(stream.is_empty());
//! assert!(document.lexical_errors().is_empty());
//! # Ok::<(), surgeist_css_parser::document::BuildError>(())
//! ```
//!
//! [`Meter`]: limits::Meter
//! [`Number`]: token::Number

#[macro_use]
mod macros;
mod match_ascii;
mod offset;
mod syntax_token;

pub mod anb;
pub mod document;
pub mod grammar;
pub mod input;
pub mod lexer;
pub mod lexical;
pub mod limits;
pub mod serialize;
pub mod stream;
pub mod token;
pub mod unicode_range;

pub use document::tree::Document;
pub use offset::Offset;
pub use stream::cursor::Stream;
pub use stream::site::Site;
pub use stream::syntax_error::SyntaxError;
pub use syntax_token::Token;
