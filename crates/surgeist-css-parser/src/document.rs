//! The parsing results of CSS Syntax Module Level 3 section 5.2, as a token
//! tree.
//!
//! A [`Document`](crate::Document) holds the tokens of a text, or of
//! components a caller already has, as nodes in lists: the root list, and one
//! list inside each function and simple block. Reading it is the job of a
//! [`Stream`](crate::Stream); this module holds what a document is made of and
//! how one is built:
//!
//! - [`NodeId`] and [`ListId`] name nodes and lists; [`Component`] is a node
//!   read as a component value, [`BlockKind`] and [`GroupKind`] say what kind
//!   of group a node is, and [`Fault`] names the parse errors section 5.2
//!   preserves in the token stream;
//! - [`GroupEnd`] says whether a group ended at its closing token or at the end
//!   of the input;
//! - [`TreeBuilder`] adds tokens and groups one at a time, for a caller that
//!   holds components rather than text, and [`Metrics`] is what a build
//!   counted;
//! - [`BuildError`] says why a build stopped.
//!
//! The document itself, [`Document`](crate::Document), is at the crate root.

mod builder;
mod component;
mod error;
mod ids;
mod metrics;
mod store;
pub(crate) mod tree;

pub use builder::TreeBuilder;
pub use component::{BlockKind, Component, Fault, GroupEnd, GroupKind};
pub use error::BuildError;
pub use ids::{ListId, NodeId};
pub use metrics::Metrics;
