//! An implementation of a `CompletionRecord` for Boa's VM.

#![allow(clippy::inline_always)]

use crate::{Context, JsError, JsResult, JsValue};
use std::ops::ControlFlow;

/// An implementation of the ECMAScript's `CompletionRecord` [specification] for
/// Boa's VM output Completion and Result.
///
/// [specification]: https://tc39.es/ecma262/#sec-completion-record-specification-type
#[derive(Debug, Clone)]
pub(crate) enum CompletionRecord {
    Normal(JsValue),
    Return(JsValue),
    Throw(JsError),
}

#[derive(Debug, Clone, surgeist_js_gc::heap::Trace)]
pub(crate) enum StoredCompletionRecord {
    Normal(crate::value::StoredValue),
    Return(crate::value::StoredValue),
    Throw(crate::error::StoredJsError),
}

impl StoredCompletionRecord {
    pub(crate) fn to_completion(&self) -> CompletionRecord {
        match self {
            Self::Normal(value) => CompletionRecord::Normal(value.to_value()),
            Self::Return(value) => CompletionRecord::Return(value.to_value()),
            Self::Throw(error) => CompletionRecord::Throw(error.to_error()),
        }
    }
}

impl CompletionRecord {
    pub(crate) fn into_stored(self) -> StoredCompletionRecord {
        match self {
            Self::Normal(value) => StoredCompletionRecord::Normal(value.into_stored()),
            Self::Return(value) => StoredCompletionRecord::Return(value.into_stored()),
            Self::Throw(error) => StoredCompletionRecord::Throw(error.into_stored()),
        }
    }
}

// ---- `CompletionRecord` methods ----
impl CompletionRecord {
    pub(crate) const fn is_throw_completion(&self) -> bool {
        matches!(self, Self::Throw(_))
    }

    /// This function will consume the current `CompletionRecord` and return a `JsResult<JsValue>`
    // NOTE: rustc bug around evaluating destructors that prevents this from being a const function.
    // Related issue(s):
    //   - https://github.com/rust-lang/rust-clippy/issues/4041
    //   - https://github.com/rust-lang/rust/issues/60964
    //   - https://github.com/rust-lang/rust/issues/73255
    #[allow(clippy::missing_const_for_fn)]
    pub(crate) fn consume(self) -> JsResult<JsValue> {
        match self {
            Self::Throw(error) => Err(error),
            Self::Normal(value) | Self::Return(value) => Ok(value),
        }
    }
}

pub(crate) trait IntoCompletionRecord {
    fn into_completion_record(self, context: &mut Context) -> ControlFlow<CompletionRecord>;
}

impl IntoCompletionRecord for () {
    #[inline(always)]
    fn into_completion_record(self, _: &mut Context) -> ControlFlow<CompletionRecord> {
        ControlFlow::Continue(())
    }
}

impl IntoCompletionRecord for JsError {
    #[inline(always)]
    fn into_completion_record(self, context: &mut Context) -> ControlFlow<CompletionRecord> {
        context.handle_error(self)
    }
}

impl IntoCompletionRecord for JsResult<()> {
    #[inline(always)]
    fn into_completion_record(self, context: &mut Context) -> ControlFlow<CompletionRecord> {
        match self {
            Ok(()) => ControlFlow::Continue(()),
            Err(err) => context.handle_error(err),
        }
    }
}

impl IntoCompletionRecord for ControlFlow<CompletionRecord> {
    #[inline(always)]
    fn into_completion_record(self, _: &mut Context) -> ControlFlow<CompletionRecord> {
        self
    }
}
