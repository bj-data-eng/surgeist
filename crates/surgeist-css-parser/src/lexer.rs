//! Tokenization of CSS Syntax Module Level 3 (section 4).
//!
//! The only tokenizer this crate offers today is the stepwise state machine in
//! [`protocol`]. It is a host-only protocol for callers that must admit each
//! action; it is not a general-purpose lexer API.

pub mod protocol;
