//! Captured canonical session-history values. Execution and queue custody live
//! with the original Runtime owner, independently from these immutable views.
use super::*;
use crate::{DocumentId, DomOpaqueOrigin, NodeId};

/// Actual native producer inputs for an initial history entry. The issuer
/// validates and admits these fixed UTF-16 strings before publication.
#[derive(Debug)]
pub struct DomHostedEntryInitialization {
    pub(crate) navigation_key: [u16; 36],
    pub(crate) navigation_id: [u16; 36],
}
impl DomHostedEntryInitialization {
    pub const fn new(navigation_key: [u16; 36], navigation_id: [u16; 36]) -> Self {
        Self {
            navigation_key,
            navigation_id,
        }
    }
}

/// One actual captured navigable and its real retained entry collection.
#[derive(Clone, Copy, Debug)]
pub struct DomHostedNavigable<'a> {
    pub(crate) record: &'a crate::store::hosted::history::Navigable,
    pub(crate) active: &'a crate::store::hosted::history::Entry,
}
impl<'a> DomHostedNavigable<'a> {
    pub fn id(self) -> DomHostedNavigableId {
        self.record.id
    }
    pub fn parent(self) -> Option<DomHostedNavigableId> {
        self.record.parent
    }
    pub fn container(self) -> Option<NodeId> {
        self.record.container
    }
    pub fn active_document(self) -> Option<DocumentId> {
        self.active.document
    }
    pub fn host(self) -> DomHostedHostId {
        self.record.host
    }
    pub fn browsing_context(self) -> DomHostedBrowsingContextId {
        self.record.context
    }
    pub fn global(self) -> DomHostedGlobalId {
        self.record.global
    }
    pub fn traversable(self) -> DomHostedNavigableId {
        self.record.traversable
    }
    pub fn current_entry(self) -> DomHostedEntryId {
        self.record.current
    }
    pub fn active_entry(self) -> DomHostedEntryId {
        self.record.active
    }
    pub fn entries(self) -> &'a [DomHostedEntryId] {
        &self.record.entries.values
    }
    /// Former-parent provenance survives unlinking for exact cleanup/history.
    pub fn former_parent(self) -> Option<DomHostedNavigableId> {
        self.record.former_parent
    }
    pub fn is_retiring(self) -> bool {
        self.record.retiring
    }
    pub fn is_retired(self) -> bool {
        self.record.retired
    }
}

/// One actual captured entry, including its owned DocumentState and nested
/// history links. Pending is not an assigned zero step.
#[derive(Clone, Copy, Debug)]
pub struct DomHostedEntry<'a> {
    pub(crate) record: &'a crate::store::hosted::history::Entry,
}
impl<'a> DomHostedEntry<'a> {
    pub fn id(self) -> DomHostedEntryId {
        self.record.id
    }
    pub fn document(self) -> Option<DocumentId> {
        self.record.document
    }
    pub fn origin(self) -> &'a DomOpaqueOrigin {
        &self.record.origin
    }
    /// Source constructor operand: absent child name is null; supplied empty is
    /// a real string and remains distinct through queued initialization.
    pub fn target_name(self) -> Option<&'a crate::DocumentString> {
        self.record.target_name.as_ref()
    }
    pub fn initiator_origin(self) -> Option<&'a DomOpaqueOrigin> {
        self.record.initiator_origin.as_ref()
    }
    pub fn about_base_url(self) -> &'a surgeist_html_semantics::HtmlAboutBaseUrl {
        &self.record.about_base_url
    }

    pub fn navigation_key(self) -> &'a [u16; 36] {
        &self.record.initialization.navigation_key
    }
    pub fn navigation_id(self) -> &'a [u16; 36] {
        &self.record.initialization.navigation_id
    }
    pub fn step(self) -> Option<u64> {
        self.record.step
    }
    pub fn nested_histories(self) -> &'a [DomHostedNavigableId] {
        &self.record.nested.values
    }
    /// The real History object's initialized values, subsequently changed by
    /// actual source-selected Global jobs; no unavailable-to-zero fallback.
    pub fn history(self) -> Option<(usize, usize)> {
        self.record.history
    }
}
