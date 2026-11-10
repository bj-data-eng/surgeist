//! The parser entry points and rule algorithms of CSS Syntax Module Level 3
//! sections 5.4 and 5.5.
//!
//! The specification normalizes its input into a token stream and runs the
//! algorithms of section 5.5 over it. Here the normalization is
//! [`Document::parse`](crate::Document::parse) (or a
//! [`TreeBuilder`](crate::document::TreeBuilder) for components a caller
//! holds), the token stream is a [`Stream`], and the algorithms
//! are methods of the stream and iterators of units:
//!
//! | Section | Where |
//! | --- | --- |
//! | 5.4.1 parse something according to a CSS grammar | [`Stream::parse_complete`] |
//! | 5.4.2 parse a comma-separated list according to a CSS grammar | [`Stream::parse_comma_separated_list`] |
//! | 5.4.3 parse a stylesheet, 5.4.4 a stylesheet's contents | [`Stream::stylesheet_contents`] (the stylesheet object and the decoding of bytes are not this crate's) |
//! | 5.4.5 parse a block's contents | [`Stream::block_contents`] |
//! | 5.4.6 parse a rule | [`Stream::parse_rule`] |
//! | 5.4.7 parse a declaration | [`Stream::parse_declaration`] |
//! | 5.4.8 parse a component value | [`Stream::parse_component_value`] |
//! | 5.4.9 parse a list of component values | [`Stream::parse_list_of_component_values`] |
//! | 5.4.10 parse a comma-separated list of component values | [`Stream::parse_comma_separated_component_values`] |
//! | 5.5.1 consume a stylesheet's contents | [`StylesheetContents`] |
//! | 5.5.2 consume an at-rule | [`AtRule`] |
//! | 5.5.3 consume a qualified rule | [`QualifiedRule`] |
//! | 5.5.4 consume a block, 5.5.5 a block's contents | [`BlockContents`], and the `block` of a rule |
//! | 5.5.6 consume a declaration | [`Declaration`] |
//! | 5.5.7 consume a list of component values | [`Stream::within_list_until`] |
//! | 5.5.8 consume a component value | [`Stream::consume_component_value`] |
//! | 5.5.9 consume a simple block, 5.5.10 a function | [`Stream::within_block`] |
//!
//! # Whole units
//!
//! "Valid in the current context" is the decision of the grammar, which this
//! crate does not know. A driver consumes a whole unit, its prelude and its
//! block, and then asks the grammar through [`RuleGrammar`] and
//! [`BlockGrammar`]. A grammar that rejects the unit therefore leaves the
//! stream at the end of it, and nothing has to be recovered; and the driver
//! reports how the unit ended ([`RuleEnd`]) so no diagnostic has to inspect
//! its text.
//!
//! The units are streams over the document: the prelude of a rule is a stream
//! of the nodes before its terminator, and its block is the list the `{}`-block
//! owns. Nothing is copied.
//!
//! # Metering and refusals
//!
//! Every time a driver examines a node, or finds the end of the stream, it takes
//! one [`Meter`](crate::limits::Meter) step, as a [`Stream`]
//! observation does. A driver can examine a node more than once (the semicolon
//! that stopped a declaration is examined again by the loop that discards it,
//! and a unit read again after a declaration that was not accepted is read from
//! its first node a second time), and it never examines one it has already
//! examined in the same pass. The first refusal latches on the document
//! ([`Document::is_refused`]) and a driver
//! reports it, as a [`Failed`] for which `is_refused()` holds, or as the
//! stream error of an entry point. No driver turns a refusal into the end of a
//! stream, a dropped rule, a shorter list or a valid result. The lists the entry
//! points return are charged to the meter as they grow.
//!
//! [`Document::is_refused`]: crate::Document::is_refused
//!
//! # Readings of the specification
//!
//! - Section 5.5.6 step 8 replaces the value of a `unicode-range` declaration
//!   with the result of tokenizing its original text with unicode ranges
//!   allowed. A value is a stream over the document that was built and cannot be
//!   replaced, so the declaration offers that text
//!   ([`Declaration::original_text`]) and the grammar calls
//!   [`unicode_range::consume_value`](crate::unicode_range::consume_value). The
//!   step says to use the text of the tokens "returned by the consume a list of
//!   component values call", which includes `!important`; the text offered is
//!   that of the value after steps 6 and 7, because a value that still held the
//!   importance the declaration has already flagged would not be a descriptor's
//!   value.
//! - Section 5.5.5 returns the rules at the end of the contents (or at a `}`)
//!   without appending the declarations it has collected since the last rule,
//!   which would drop a trailing run of declarations. That cannot be the
//!   intent, and the iterators do not model the grouping at all: they report
//!   every declaration and rule in order, and a consumer groups them (the
//!   documentation of [`BlockContents`] says how), including the last run.
//! - A declaration that the grammar rejects and whose text is then read as a
//!   qualified rule that looks like a custom property is reported as a failed
//!   declaration with the grammar's error. The specification consumes the
//!   remnants and returns nothing, without an error; the verdict of the grammar
//!   is not lost for it.
//! - A comment is not a token of the specification. The drivers are transparent
//!   to it as the stream is: it does not separate the tokens of a rule, it ends
//!   no run of whitespace, and it is part of the prelude it is in.
//! - A `{}`-block that the build denied for nesting too deep
//!   ([`DepthExcess::DenyBlock`](crate::limits::DepthExcess::DenyBlock)) has no
//!   contents. A rule whose block is denied is not given to the grammar: it is
//!   a [`Failed`] whose error holds
//!   [`is_denied_block`](crate::SyntaxError::is_denied_block).

#[cfg(doc)]
use crate::Stream;

mod algorithm;
mod entry;
mod iterator;
mod traits;
mod unit;

pub use iterator::{BlockContents, StylesheetContents};
pub use traits::{BlockGrammar, RuleGrammar};
pub use unit::{AtRule, Declaration, Failed, QualifiedRule, RuleEnd, Unit, UnitKind};
