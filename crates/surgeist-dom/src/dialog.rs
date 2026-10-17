//! Canonical Dialog state with actual optional task and watcher associations.
//!
//! Native Window-manager receipts establish or clear watcher identities through
//! the original Source hooks. Modal/request-close state remains a closed proof.
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
    const fn force_watcher_enabled(self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DomDialogState {
    return_value: DocumentString,
    services: ClosedDialogServices,
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
}
impl DomCloseWatcherId {
    pub const fn subject(self) -> NodeId {
        self.target
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
            toggle_tracker: self.toggle_tracker,
            watcher: self.watcher,
        }
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
    /// Known actual null. Request-close execution is not admitted by this domain.
    pub fn request_close_return_value(self) -> Option<&'a DocumentString> {
        let _ = self.state().services;
        None
    }
    /// Known actual null, not an unavailable identity or a replacement generation.
    pub fn request_close_source(self) -> Option<NodeId> {
        let _ = self.state().services;
        None
    }
    pub fn force_close_watcher_enabled(self) -> bool {
        self.state().services.force_watcher_enabled()
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
