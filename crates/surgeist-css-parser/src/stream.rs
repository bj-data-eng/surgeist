//! Token streams (CSS Syntax Module Level 3 section 5.3) and the errors a
//! grammar over one reports.
//!
//! A [`Stream`](crate::Stream) is a position in a [`Document`](crate::Document)
//! from which a grammar consumes tokens. This module holds what goes with it:
//!
//! - [`Mark`], a restore point;
//! - [`StopToken`] and [`StopTokens`], the tokens that end a list of component
//!   values (section 5.5.7);
//! - [`StreamError`], the failure of a stream to supply what was asked, and, at
//!   the crate root, [`SyntaxError`], the failure of a
//!   grammar, which can be a stream failure.
//!
//! The error types report facts through `is_*` predicates and accessors rather
//! than a public enum, so a consumer cannot depend on how the crate classifies
//! internally.

mod error;
mod mark;
mod stop;

pub(crate) mod cursor;
pub(crate) mod site;
pub(crate) mod syntax_error;

pub use error::StreamError;
pub use mark::Mark;
pub use stop::{StopToken, StopTokens};

use site::Site;
use syntax_error::SyntaxError;
