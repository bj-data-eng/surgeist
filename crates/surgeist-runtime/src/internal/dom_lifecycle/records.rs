//! Actual native execution objects in each original browsing-context group.
//!
//! A group owns its origin-keyed clusters, agents and window loops. Globals may
//! retire independently; queued work retains the actual original group/loop
//! after live top-level membership ends. All canonical roles use source-issued
//! identities; these records do not mirror the canonical navigation graph.
use super::clock;
use std::{cell::RefCell, rc::Rc};
use surgeist_dom::task::DomElementTaskId;
use surgeist_dom::{
    document::DomOpaqueOrigin,
    hosted::{
        DomHostedBrowsingContextId, DomHostedGlobalId, DomHostedHostId, DomHostedNavigableId,
        DomHostedOperationId,
    },
    identity::DocumentId,
};

pub(super) struct Host {
    pub id: DomHostedHostId,
    pub top_levels: Vec<Composition>,
    pub focus: super::focus::HostFocusDisposition,
}
pub(super) struct Composition {
    pub group: Option<Group>,
    pub top: Option<Top>,
}

pub(super) struct Group {
    pub context: DomHostedBrowsingContextId,
    pub traversal: super::history::Traversal,
    pub completions: Vec<super::destruction::Completion>,
    pub fullscreen: super::fullscreen::Jobs,
    // This is also the group's historical origin->origin-keyed cluster entry.
    // The opaque identity is preserved even when requestsOAC is false.
    pub clusters: Vec<OriginCluster>,
}
impl Group {
    // The original top-level Global is installed first; children append to the
    // same bank and their retirement preserves this first record's position.
    pub fn global(&self) -> Option<&Global> {
        self.clusters
            .first()?
            .agent
            .globals
            .first()
            .filter(|global| global.context.id == self.context)
    }
    pub fn global_mut(&mut self) -> Option<&mut Global> {
        let context = self.context;
        self.clusters
            .first_mut()?
            .agent
            .globals
            .first_mut()
            .filter(|global| global.context.id == context)
    }
}
pub(super) struct OriginCluster {
    pub origin: DomOpaqueOrigin,
    pub agent: NativeAgent,
}
pub(super) struct NativeAgent {
    pub event_loop: Rc<RefCell<WindowLoop>>,
    pub globals: Vec<Global>,
    pub windows: super::watcher::WindowBank,
}

#[derive(Default)]
pub(super) enum WindowAgentHistory {
    #[default]
    Empty,
    One,
    Multiple,
}

impl WindowAgentHistory {
    pub(super) fn record(&mut self, event_loop: &Rc<RefCell<WindowLoop>>) {
        let mut event_loop = event_loop.borrow_mut();
        if !event_loop.had_active_document {
            event_loop.had_active_document = true;
            *self = match self {
                Self::Empty => Self::One,
                Self::One | Self::Multiple => Self::Multiple,
            };
        }
    }

    pub(super) fn is_only(&self, event_loop: &Rc<RefCell<WindowLoop>>) -> bool {
        // Agent retirement cannot make the session-wide notification queue
        // qualify for a single agent again.
        matches!(self, Self::One) && event_loop.borrow().had_active_document
    }
}
pub(super) struct BrowsingContext {
    pub id: DomHostedBrowsingContextId,
    pub group_root: DomHostedBrowsingContextId,
    pub creator: Option<DomHostedGlobalId>,
    pub global: Option<DomHostedGlobalId>,
}
pub(super) struct Global {
    pub context: BrowsingContext,
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
    pub top_level_origin: DomOpaqueOrigin,
}

impl Settings {
    pub(super) fn make_active(&mut self, document: DocumentId) {
        self.document = Some(document);
        self.ready = true;
    }
}

pub(super) struct Top {
    pub id: DomHostedNavigableId,
    pub closing: bool,
    pub local_parser_checkpoint: Option<surgeist_dom::identity::DomStamp>,
}

/// Actual task-source identity currently executing in this native window loop.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CurrentTask {
    ElementTask {
        source: surgeist_dom::task::DomElementTaskSourceKind,
        task: DomElementTaskId,
        document: DocumentId,
    },
    #[cfg(feature = "document-scripting")]
    CspViolation { task: u64, document: DocumentId },
    #[cfg(feature = "document-scripting")]
    HtmlCompletion {
        task: super::HtmlCompletionTask,
        document: DocumentId,
    },
    History {
        operation: DomHostedOperationId,
        occurrence: u64,
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
    pub navigation: Vec<NavigationTask>,
    pub slot_checkpoint: Option<crate::internal::dom_event::NativeSlotCheckpoint>,
    had_active_document: bool,
}
impl WindowLoop {
    pub fn new() -> Self {
        Self {
            current: None,
            executing: false,
            next_checkpoint: 0,
            checkpoints: 0,
            navigation: Vec::new(),
            slot_checkpoint: None,
            had_active_document: false,
        }
    }
}

/// A real parent-global navigation-and-traversal task. Empty child membership
/// satisfies only the descendant wait; it does not execute this task's body.
pub(super) struct DestroyTask {
    pub operation: DomHostedOperationId,
    pub document: DocumentId,
    pub global: DomHostedGlobalId,
    pub phase: DestroyPhase,
}
#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum DestroyPhase {
    Queued,
    Cleanup,
    CheckpointComplete,
}

/// Real navigation-and-traversal tasks share the original Global loop FIFO.
/// A receipt marks an actual canonical History write; it survives a refused
/// checkpoint and is settled exactly once without executing that write again.
pub(super) enum NavigationTask {
    Destroy(DestroyTask),
    History(HistoryTask),
}
impl NavigationTask {
    pub fn destroy_mut(&mut self) -> Option<&mut DestroyTask> {
        match self {
            Self::Destroy(task) => Some(task),
            Self::History(_) => None,
        }
    }
    pub fn destroy(&self) -> Option<&DestroyTask> {
        match self {
            Self::Destroy(task) => Some(task),
            Self::History(_) => None,
        }
    }
}
pub(super) struct HistoryTask {
    pub operation: DomHostedOperationId,
    pub occurrence: u64,
    pub document: DocumentId,
    pub global: DomHostedGlobalId,
    pub receipt: Option<surgeist_dom::hosted::history::DomHostedHistoryJobReceipt>,
}
