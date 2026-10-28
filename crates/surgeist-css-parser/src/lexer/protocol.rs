//! The stepwise tokenizer protocol: a narrow, host-only sans-IO contract.
//!
//! [`TokenCursor`] is the section 4.3 tokenizer as a state machine over
//! preprocessed scalars (see [`InputCursor`](crate::input::InputCursor) for
//! section 3.3). It performs no input and allocates nothing. Its host
//!
//! 1. asks [`TokenCursor::request`] what is needed next;
//! 2. admits that one action, however the host meters work and storage;
//! 3. answers with [`TokenCursor::read_scalar`], [`TokenCursor::compute`] or
//!    [`TokenCursor::appended`];
//!
//! and repeats until the request is [`TokenRequest::Done`]. Every request is
//! one scalar, one bounded grammar decision, or one record to store, so a host
//! admits exactly the work it performs. A refused append is not acknowledged;
//! the cursor stays unchanged and the same request remains pending.
//!
//! The host owns every bank. The cursor generates [`Record`]s (decoded name
//! units, number digits, and tokens) in order; the host stores them, and the
//! spans in each [`RawToken`] index that store.
//!
//! This module exports exactly what the hosts in Surgeist use: the cursors and
//! the records, requests and errors that pass between them. It carries the
//! same stability as the rest of the crate and no promise of a general-purpose
//! API. A caller that only wants tokens from text should not use it.
//!
//! The decision whether a scalar starts a name, a number or an escape uses the
//! predicates of [`lexical`](crate::lexical), which describe preprocessed
//! input: the cursor never sees U+0000, U+000D or U+000C, and
//! [`TokenCursor::read_scalar`] refuses a scalar that is one of them.

mod error;
pub(super) mod name;
mod number;
mod tokenizer;

pub use error::ProtocolError;
pub use number::{NumberCursor, NumberLayout, NumberOverflow, NumberPart, NumberStep};
pub use tokenizer::{
    NameKind, NumberKind, RawToken, RawTokenKind, Record, Span, TokenCursor, TokenRequest,
};
