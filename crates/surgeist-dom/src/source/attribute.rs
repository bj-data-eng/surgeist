use super::DomSourceOperationId;
use crate::{
    DocumentId, DocumentString, DomDocumentCspState, DomDocumentScriptRegistration, DomError,
    DomHostedGlobalId, NodeId,
};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

/// The exact sink selected for a standard attribute write.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceAttributeSink {
    EventHandler(&'static str),
    HtmlIframeSrcdoc,
    HtmlScriptSrc,
    SvgScriptHref,
}

/// An observation of the original verification occurrence. Its input and
/// reply are available only through the corresponding live registration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceAttributeVerificationRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) depth: usize,
    pub(crate) element: NodeId,
    pub(crate) global: DomHostedGlobalId,
    pub(crate) sink: DomSourceAttributeSink,
}

impl DomSourceAttributeVerificationRequest {
    pub const fn operation(self) -> DomSourceOperationId {
        self.operation
    }

    pub const fn element(self) -> NodeId {
        self.element
    }

    pub const fn global(self) -> DomHostedGlobalId {
        self.global
    }

    pub const fn sink(self) -> DomSourceAttributeSink {
        self.sink
    }
}

/// Result supplied by the qualified verifier for the exact pending request.
/// An abrupt guest value remains owned by the script host.
#[derive(Debug)]
pub enum DomSourceAttributeVerificationOutcome {
    Value(DocumentString),
    Rejected,
    Abrupt,
}

/// Admitted retained payload from the exact Source attribute verification.
/// This storage receipt grants no callback authority and may outlive the request.
#[derive(Debug)]
pub struct DomSourceAttributePayloadRetention {
    pub(crate) _debit: crate::store::custody::Debit,
}

/// The original verifier's live callback scope. Dropping it closes reentry.
#[must_use = "keep the scope only while running the verifier callback"]
#[derive(Debug)]
pub struct DomSourceAttributeCallbackScope<'registration> {
    pub(crate) _scope: Arc<super::callback::DispatchScope>,
    pub(crate) _registration: &'registration DomDocumentScriptRegistration,
}

/// One authenticated read of the registered Global's current policy state.
/// Its borrow ends before any guest callback or nested mutation can run.
pub struct DomSourceAttributePolicyRead<'a> {
    pub(crate) document: DocumentId,
    pub(crate) state: DomDocumentCspState<'a>,
    pub(crate) work: &'a super::callback::CallbackWork,
}

impl<'a> DomSourceAttributePolicyRead<'a> {
    /// The Global's associated Document at this read.
    pub const fn document(&self) -> DocumentId {
        self.document
    }

    /// The actual current canonical CSP state of that Document.
    pub const fn state(&self) -> DomDocumentCspState<'a> {
        self.state
    }

    /// Charge policy inspection to this occurrence and its Source ancestors.
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.work.step(cancel)
    }
}
