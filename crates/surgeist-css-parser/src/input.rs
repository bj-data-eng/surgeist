//! The input stream of CSS Syntax Module Level 3.
//!
//! This module holds the stepwise [`InputCursor`] for section 3.3, "Preprocessing
//! the input stream". It decodes one kind of original unit ([`InputKind`]),
//! applies the preprocessing and reports every resulting [`Scalar`] together
//! with the interval of original units it came from, so a host never loses its
//! own coordinates to the normalization.
//!
//! The cursor is a host-only protocol in the same sense as
//! [`lexer::protocol`](crate::lexer::protocol): it owns no storage and
//! performs one fixed action per request, so a host that meters work admits
//! each action before the cursor takes it. A host that only needs text in
//! and tokens out does not use it.
//!
//! The preprocessing applied to each scalar is:
//!
//! - a U+000D CARRIAGE RETURN, a U+000C FORM FEED, and a carriage return
//!   followed by a U+000A LINE FEED, each become one U+000A LINE FEED whose
//!   interval covers every original unit it replaced;
//! - a U+0000 NULL, a surrogate code point, and a UTF-16 unit that is not
//!   part of a surrogate pair, become U+FFFD REPLACEMENT CHARACTER.

mod cursor;

pub use cursor::{InputCursor, InputError, InputKind, InputRequest, Scalar};
