//! The input stream of CSS Syntax Module Level 3.
//!
//! Sections 3.2 and 3.3 turn what a host has, a style sheet's bytes or UTF-16
//! units, into the code points that the tokenizer reads. This module holds
//! both, with no CSS meaning:
//!
//! - [`decode`] is section 3.2, "The input byte stream": it determines the
//!   fallback [`Encoding`] from the host's [`DecodeHints`] and the bytes
//!   themselves (a protocol label, an `@charset "label";` declaration at the
//!   start of the first 1024 bytes, the environment encoding, then UTF-8), and
//!   decodes with the Encoding Standard's decoders, which give a byte order
//!   mark precedence. It returns the [`Decoded`] text, still unfiltered.
//! - [`filter_code_points`] is section 3.3, "Preprocessing the input stream",
//!   over a whole string.
//! - [`preprocess_utf16`] is section 3.3 over UTF-16 units that may hold lone
//!   surrogates. It builds the normalized text and a [`SourceMap`] that keeps
//!   the original units of every scalar, charging a [`Meter`] for the work and
//!   storage as it goes.
//! - [`InputCursor`] is section 3.3 as a stepwise protocol: it decodes one
//!   kind of original unit ([`InputKind`]), applies the preprocessing and
//!   reports every resulting [`Scalar`] together with the interval of original
//!   units it came from, so a host never loses its own coordinates to the
//!   normalization. [`preprocess_utf16`] is a loop over it.
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
//!
//! [`Meter`]: crate::limits::Meter

mod cursor;
mod decode;
mod filter;
mod preprocess;

pub use cursor::{InputCursor, InputError, InputKind, InputRequest, Scalar};
pub use decode::{
    DecodeError, DecodeHints, DecodeLimits, Decoded, Encoding, EncodingSelection, decode,
};
pub use filter::filter_code_points;
pub use preprocess::{
    PreprocessError, PreprocessLimits, PreprocessResource, Preprocessed, SourceMap,
    preprocess_utf16,
};
