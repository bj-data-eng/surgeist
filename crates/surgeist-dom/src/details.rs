//! Native Details views and source-ordered raw toggle-intention transfer.
use crate::{DomChanges, DomNode, DomOwnerId, DomSnapshot, NodeId};
use surgeist_html::HtmlDetailsState;

#[derive(Clone, Copy, Debug)]
pub struct DomDetails<'a> {
    pub(crate) node: DomNode<'a>,
}
impl DomDetails<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    /// Actual no-namespace open presence, never a second stored Boolean.
    pub fn state(self) -> HtmlDetailsState {
        self.node
            .as_element()
            .expect("Details")
            .details_state()
            .expect("Details")
    }
}

/// Opaque operation identity; this metadata grants no task execution authority.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct DomDetailsToggleIntentionId {
    pub(crate) owner: DomOwnerId,
    pub(crate) serial: u64,
}

/// One actual raw transition. Ownership can be consumed once; the record cannot
/// be cloned or constructed from copied public facts.
#[derive(Debug, Eq, PartialEq)]
pub struct DomDetailsToggleIntention {
    pub(crate) operation: DomDetailsToggleIntentionId,
    pub(crate) target: NodeId,
    pub(crate) old: HtmlDetailsState,
    pub(crate) new: HtmlDetailsState,
}
impl DomDetailsToggleIntention {
    pub fn operation(&self) -> DomDetailsToggleIntentionId {
        self.operation
    }
    pub fn target(&self) -> NodeId {
        self.target
    }
    pub fn old_state(&self) -> HtmlDetailsState {
        self.old
    }
    pub fn new_state(&self) -> HtmlDetailsState {
        self.new
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomDetailsToggleCancellationReason {
    TargetDestroyed,
}

/// Immediate owned transfer of raw transitions, before any task-owner
/// coalescing or event dispatch. A future consumer admits the entire borrowed
/// batch before consuming ownership, returning it unchanged on refusal.
#[derive(Debug)]
pub struct DomDetailsToggleTransfer {
    pub(crate) source: DomSnapshot,
    pub(crate) changes: DomChanges,
    pub(crate) intentions: Vec<DomDetailsToggleIntention>,
}
impl DomDetailsToggleTransfer {
    /// Canonical snapshot immediately after the atomic ownership transfer.
    pub fn source(&self) -> &DomSnapshot {
        &self.source
    }
    pub fn changes(&self) -> &DomChanges {
        &self.changes
    }
    pub fn intentions(&self) -> &[DomDetailsToggleIntention] {
        &self.intentions
    }
    pub fn len(&self) -> usize {
        self.intentions.len()
    }
    pub fn is_empty(&self) -> bool {
        self.intentions.is_empty()
    }
}
impl IntoIterator for DomDetailsToggleTransfer {
    type Item = DomDetailsToggleIntention;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.intentions.into_iter()
    }
}
