//! The state-transition contract of an application.
//!
//! A [`Reducer`] reduces an immutable borrowed state and input into a [`ReducerResult`]. The result is
//! [`ReducerResult::Unchanged`] with a [`ReducerCommit`], [`ReducerResult::Changed`] with a [`ReducerChange`] that owns a
//! replacement state, or [`ReducerResult::RecoverableFailure`] with a [`ReducerFailure`], which carries neither state nor
//! effects.

pub use crate::internal::reducer::{
    Reducer, ReducerChange, ReducerCommit, ReducerFailure, ReducerResult,
};
