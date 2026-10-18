//! Required services of the original designMode setter; no echo fulfillment.
use crate::{DocumentId, DomRangeId, DomSourceOperationId, NodeId};

/// Actual completed range branch of a false-to-on designMode transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDesignModeRangeDisposition {
    NoActiveRange,
    RangeReset(DomRangeId),
}
/// A precise unavailable action; absence of a service never means a null Range
/// or a negative focusability fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSourceDesignModeStep {
    ReadActiveRange,
    Focusing(surgeist_html::HtmlFocusingAction),
}
/// One source-issued requirement of the original designMode invocation. It
/// grants no mutation/acknowledgment authority and cannot complete a focus call.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSourceDesignModeRequest {
    pub(crate) operation: DomSourceOperationId,
    pub(crate) document: DocumentId,
    pub(crate) step: DomSourceDesignModeStep,
    pub(crate) target: Option<NodeId>,
    pub(crate) range: Option<DomDesignModeRangeDisposition>,
    pub(crate) occurrence: super::watcher::HookOccurrence,
}
impl DomSourceDesignModeRequest {
    pub const fn operation(&self) -> DomSourceOperationId {
        self.operation
    }
    pub const fn document(&self) -> DocumentId {
        self.document
    }
    pub const fn step(&self) -> DomSourceDesignModeStep {
        self.step
    }
    pub const fn focus_target(&self) -> Option<NodeId> {
        self.target
    }
    /// Completed local range branch at this published boundary. `None` while
    /// association availability is unresolved.
    pub const fn range_disposition(&self) -> Option<DomDesignModeRangeDisposition> {
        self.range
    }
    pub const fn command(&self) -> usize {
        self.occurrence.command
    }
    pub const fn current_occurrence(&self) -> u64 {
        self.occurrence.ordinal
    }
}
