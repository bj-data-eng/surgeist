//! Inline-style identity bound to CSSOM, its capture, mutation and feedback.
//!
//! `DomInlineStyleCapture` and `DomInlineStyleProfile` capture an element's inline style,
//! `DomInlineStyleMutation` and `DomInlineStyleOperation` change it, and the limits types
//! bound the work. The `DomInlineStyleSource*` types carry the Source continuation, and
//! `DomInlineStyleChange`, `DomInlineStyleCommit` and `DomInlineStyleOutcome` report the
//! published result.
//!
//! `docs/source-and-style.md` owns the contract.

pub use crate::internal::inline_style::{
    DomInlineStyleBatchContext, DomInlineStyleCapture, DomInlineStyleChange, DomInlineStyleCommit,
    DomInlineStyleCompletion, DomInlineStyleError, DomInlineStyleFeedbackLimits,
    DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleMutation,
    DomInlineStyleMutationLimits, DomInlineStyleOutcome, DomInlineStyleProfile,
    DomInlineStyleProperty, DomInlineStyleRequestLimits, DomInlineStyleReturn,
    DomInlineStyleSourceContext, DomInlineStyleSourceProgress, DomInlineStyleSourcePublication,
    DomInlineStyleTerminal,
};
pub use crate::internal::store::DomInlineStyleOperation;
