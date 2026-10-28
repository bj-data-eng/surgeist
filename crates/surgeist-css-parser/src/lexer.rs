//! Tokenization of CSS Syntax Module Level 3 (section 4).
//!
//! The section 4.3 algorithms exist once, as one state machine, and this module
//! offers two ways to drive it:
//!
//! - [`Lexer`] is free-running and unmetered: it reads `&str` and returns one
//!   [`Lexeme`] at a time, consuming a whole run of ordinary code points where
//!   the machine would consume them one at a time. Use it to tokenize text.
//! - [`protocol`] is the stepwise, host-only form of the same machine, for
//!   callers that must admit each action. It is not a general-purpose lexer
//!   API.
//!
//! Both read section 3.3's preprocessed code points: the [`Lexer`] applies the
//! preprocessing as it reads, so it takes text as authored and reports
//! offsets in the original bytes, and the protocol takes the scalars of an
//! [`InputCursor`](crate::input::InputCursor).

mod driver;
mod lexeme;
mod machine;
pub mod protocol;

pub use driver::Lexer;
pub use lexeme::{Lexeme, LexicalError, Options};
