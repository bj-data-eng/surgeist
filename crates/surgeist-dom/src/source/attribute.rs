use super::DomSourceOperationId;
use crate::{DocumentString, DomDocumentScriptRegistration, DomHostedGlobalId, NodeId};
use std::sync::Arc;

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

/// The original verifier's live callback scope. Dropping it closes reentry.
#[must_use = "keep the scope only while running the verifier callback"]
#[derive(Debug)]
pub struct DomSourceAttributeCallbackScope<'registration> {
    pub(crate) _scope: Arc<super::callback::DispatchScope>,
    pub(crate) _registration: &'registration DomDocumentScriptRegistration,
}
