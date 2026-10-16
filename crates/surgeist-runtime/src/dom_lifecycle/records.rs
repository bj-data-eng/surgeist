//! Actual records of the first no-opener native host composition.
//!
//! Each no-opener top-level's group, opaque-keyed cluster and native agent
//! have the same lifetime in this scope, so they are owned inline. The
//! source-issued Host identity qualifies this actual group; no second allocator
//! fabricates group, agent or settings provenance.
use super::{clock, uuid};
use crate::dom_details::DomDetailsTaskId;
use std::{cell::RefCell, rc::Rc};
use surgeist_dom::{
    DocumentId, DomHostedBrowsingContextId, DomHostedEntryId, DomHostedGlobalId, DomHostedHostId,
    DomHostedNavigableId, DomHostedOperationId, DomOpaqueOrigin,
};

pub(super) struct Host {
    pub id: DomHostedHostId,
    pub top_levels: Vec<Composition>,
}
pub(super) struct Composition {
    pub group: Option<Group>,
    pub top: Option<Top>,
}

pub(super) struct Group {
    pub context: DomHostedBrowsingContextId,
    // This is also the group's historical origin->origin-keyed cluster entry.
    // The opaque identity is preserved even when requestsOAC is false.
    pub cluster: Option<OriginCluster>,
}
pub(super) struct OriginCluster {
    pub origin: DomOpaqueOrigin,
    pub agent: NativeAgent,
}
pub(super) struct NativeAgent {
    pub event_loop: Rc<RefCell<WindowLoop>>,
    pub global: Option<Global>,
}
pub(super) struct Global {
    pub id: DomHostedGlobalId,
    pub settings: Settings,
}
pub(super) struct Settings {
    pub navigation_start: clock::CoarsenedSample,
    pub document: Option<DocumentId>,
    pub ready: bool,
    // Sampled from the original canonical ExecutionRequest before native
    // Settings construction. Current base/origin/policy still read the actual
    // associated Document rather than these historical creation inputs.
    pub creation_url: surgeist_html_semantics::HtmlBaseUrlRecord,
    pub top_level_creation_url: surgeist_html_semantics::HtmlBaseUrlRecord,
}

pub(super) struct Top {
    pub id: DomHostedNavigableId,
    pub entry: Entry,
    pub closing: bool,
    // First initialization installs the same actual entry as current+active;
    // subsequent navigation/replacement is outside this admitted front.
}
pub(super) struct Entry {
    pub id: DomHostedEntryId,
    pub document: DocumentId,
    pub origin: DomOpaqueOrigin,
    pub uuids: uuid::HistoryUuids,
    // Actual top-level initial history step is zero. Target name is empty;
    // initiator/about-base/resource/persisted state are absent, nested history
    // and scroll data are empty. There is no executable JS state API.
}

/// Actual task-source identity currently executing in this native window loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CurrentTask {
    Details {
        task: DomDetailsTaskId,
        document: DocumentId,
    },
    Destroy {
        operation: DomHostedOperationId,
        document: DocumentId,
    },
}

pub(super) struct WindowLoop {
    pub current: Option<CurrentTask>,
    pub executing: bool,
    pub next_checkpoint: u64,
    pub checkpoints: u64,
    pub navigation: Vec<DestroyTask>,
}
impl WindowLoop {
    pub fn new() -> Self {
        Self {
            current: None,
            executing: false,
            next_checkpoint: 0,
            checkpoints: 0,
            navigation: Vec::new(),
        }
    }
}

/// A real parent-global navigation-and-traversal task. Empty child membership
/// satisfies only the descendant wait; it does not execute this task's body.
pub(super) struct DestroyTask {
    pub operation: DomHostedOperationId,
    pub document: DocumentId,
    pub global: DomHostedGlobalId,
    pub captured_children: Vec<DomHostedNavigableId>,
    pub completed_children: usize,
    pub phase: DestroyPhase,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DestroyPhase {
    WaitChildren,
    Queued,
    Cleanup,
    CheckpointComplete,
}
