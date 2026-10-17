//! Canonical Dialog state with actual optional task and watcher associations.
//!
//! Native Window-manager receipts establish or clear watcher identities through
//! the original Source hooks. Modal state remains separately qualified.
use crate::{DocumentString, DomNode, NodeId};

/// Qualification of the actual null/false service slots through the complete
/// admitted transition set. This is private state, not a public fresh descriptor
/// and not permission to acknowledge an external service action.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ClosedDialogServices;
impl ClosedDialogServices {
    fn from_fresh(_: surgeist_html::HtmlDialogFreshState) -> Self {
        Self
    }
    fn from_parsed<'a>(
        disposition: surgeist_html_parser::HtmlParseDialogDisposition<'a>,
    ) -> Result<Self, surgeist_html_parser::HtmlParseDialogMissing<'a>> {
        match disposition {
            surgeist_html_parser::HtmlParseDialogDisposition::Closed => Ok(Self),
            surgeist_html_parser::HtmlParseDialogDisposition::Unavailable(missing) => Err(missing),
        }
    }
    const fn is_modal(self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DomDialogState {
    return_value: DocumentString,
    services: ClosedDialogServices,
    pub(crate) request_return_value: Option<DocumentString>,
    pub(crate) request_source: Option<NodeId>,
    pub(crate) force_watcher_enabled: bool,
    pub(crate) watcher: Option<DomCloseWatcherId>,
    pub(crate) toggle_tracker: Option<DomDialogToggleTracker>,
}
/// One exact watcher occurrence owned by the original Window manager.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomCloseWatcherId {
    pub(crate) operation: crate::DomSourceOperationId,
    pub(crate) command: usize,
    pub(crate) occurrence: u64,
    pub(crate) target: NodeId,
    pub(crate) global: crate::DomHostedGlobalId,
    pub(crate) window_document: crate::DocumentId,
}
impl DomCloseWatcherId {
    pub const fn subject(self) -> NodeId {
        self.target
    }
    /// Original Window associated Document established with this actual watcher.
    pub const fn window_document(self) -> crate::DocumentId {
        self.window_document
    }
    pub const fn global(self) -> crate::DomHostedGlobalId {
        self.global
    }
}
/// Historical canonical identity and first state of the currently tracked task.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDialogToggleTracker {
    pub(crate) task: crate::DomElementTaskId,
    pub(crate) old: surgeist_html::HtmlDialogState,
}
impl DomDialogToggleTracker {
    /// Exact actual task identity; this observation grants no execution right.
    pub const fn task(self) -> crate::DomElementTaskId {
        self.task
    }
    /// First old state preserved by coalescing.
    pub const fn old_state(self) -> surgeist_html::HtmlDialogState {
        self.old
    }
}
impl DomDialogState {
    /// Called only by the actual native Element initialization owner. Existing
    /// parsed history is not qualified by calling this constructor.
    pub(crate) fn fresh() -> Self {
        let fresh = surgeist_html::HtmlDialogFreshState::fresh();
        Self {
            return_value: fresh.return_value().into(),
            services: ClosedDialogServices::from_fresh(fresh),
            request_return_value: None,
            request_source: None,
            force_watcher_enabled: false,
            toggle_tracker: None,
            watcher: None,
        }
    }
    /// Transfers the original controller's current state, never native fresh
    /// initialization. The caller admits the independent string occurrence.
    pub(crate) fn from_parsed<'a>(
        return_value: DocumentString,
        disposition: surgeist_html_parser::HtmlParseDialogDisposition<'a>,
    ) -> Result<Self, surgeist_html_parser::HtmlParseDialogMissing<'a>> {
        let services = ClosedDialogServices::from_parsed(disposition)?;
        Ok(Self {
            return_value,
            services,
            request_return_value: None,
            request_source: None,
            force_watcher_enabled: false,
            toggle_tracker: None,
            watcher: None,
        })
    }
    pub(crate) fn return_value(&self) -> &DocumentString {
        &self.return_value
    }
    pub(crate) fn with_return_value(&self, return_value: DocumentString) -> Self {
        Self {
            return_value,
            services: self.services,
            request_return_value: self.request_return_value.clone(),
            request_source: self.request_source,
            force_watcher_enabled: self.force_watcher_enabled,
            toggle_tracker: self.toggle_tracker,
            watcher: self.watcher,
        }
    }
    pub(crate) fn with_request_return_value(&self, value: Option<DocumentString>) -> Self {
        Self {
            return_value: self.return_value.clone(),
            services: self.services,
            request_return_value: value,
            request_source: self.request_source,
            force_watcher_enabled: self.force_watcher_enabled,
            watcher: self.watcher,
            toggle_tracker: self.toggle_tracker,
        }
    }
    pub(crate) fn units(&self) -> Option<usize> {
        self.return_value.len().checked_add(
            self.request_return_value
                .as_ref()
                .map_or(0, DocumentString::len),
        )
    }
    pub(crate) const fn is_modal(&self) -> bool {
        self.services.is_modal()
    }
    pub(crate) const fn has_close_watcher(&self) -> bool {
        self.watcher.is_some()
    }
}

/// A Dialog at one immutable canonical revision. Return value, watcher and task
/// associations are retained state; none grants current execution authority.
#[derive(Clone, Copy, Debug)]
pub struct DomDialog<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomDialog<'a> {
    fn state(self) -> &'a DomDialogState {
        self.node
            .as_element()
            .expect("checked Dialog")
            .dialog_state()
            .expect("checked Dialog")
    }
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn return_value(self) -> &'a DocumentString {
        self.state().return_value()
    }
    pub fn is_modal(self) -> bool {
        self.state().is_modal()
    }
    pub fn has_toggle_task_tracker(self) -> bool {
        self.state().toggle_tracker.is_some()
    }
    /// Current canonical tracker at this historical snapshot revision.
    pub fn toggle_task_tracker(self) -> Option<DomDialogToggleTracker> {
        self.state().toggle_tracker
    }
    /// Historical association; grants no manager execution authority.
    pub fn close_watcher(self) -> Option<DomCloseWatcherId> {
        self.state().watcher
    }
    pub fn has_close_watcher(self) -> bool {
        self.state().has_close_watcher()
    }
    /// Actual nullable request argument at this immutable revision.
    pub fn request_close_return_value(self) -> Option<&'a DocumentString> {
        self.state().request_return_value.as_ref()
    }
    /// Actual nullable request source at this immutable revision.
    pub fn request_close_source(self) -> Option<NodeId> {
        self.state().request_source
    }
    pub fn force_close_watcher_enabled(self) -> bool {
        self.state().force_watcher_enabled
    }
}

/// The one Document-owned managed state, independent of each Dialog and of the
/// Runtime task owner. None at the Node level means non-Document; Unavailable
/// means a Document whose actual retained history has not established these lists.
/// Native constructors and qualified original parser history establish known
/// empty lists. Positive hosted operations must consume these same canonical
/// lists, never a second top layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DocumentInteractiveState<K> {
    Known {
        open_dialogs: Vec<K>,
        top_layer: Vec<K>,
    },
    Unavailable,
}
impl<K> DocumentInteractiveState<K> {
    /// Called only for a genuinely new owned Document whose exact report row
    /// establishes its complete noninteraction prefix. This never qualifies an
    /// existing captured Document or resets its historical list contents.
    pub(crate) fn from_parsed<'a>(
        disposition: surgeist_html_parser::HtmlParseDialogDisposition<'a>,
    ) -> Result<Self, surgeist_html_parser::HtmlParseDialogMissing<'a>> {
        ClosedDialogServices::from_parsed(disposition)?;
        Ok(Self::Known {
            open_dialogs: Vec::new(),
            top_layer: Vec::new(),
        })
    }
    pub(crate) fn native_closed() -> Self {
        Self::Known {
            open_dialogs: Vec::new(),
            top_layer: Vec::new(),
        }
    }
}

impl<'a> DomNode<'a> {
    pub fn as_dialog(self) -> Option<DomDialog<'a>> {
        self.as_element()?.dialog_state()?;
        Some(DomDialog { node: self })
    }
}
