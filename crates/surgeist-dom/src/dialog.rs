//! Canonical native Dialog values for the admitted closed operation domain.
//!
//! Ordinary open/tree hooks and returnValue assignment cannot create modal,
//! watcher, queued-task or request-close state. Positive show/close/hosted service
//! operations require their real owner before this representation can advance.
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
    const fn is_modal(self) -> bool {
        false
    }
    const fn has_tracker(self) -> bool {
        false
    }
    const fn has_watcher(self) -> bool {
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
}
impl DomDialogState {
    /// Called only by the actual native Element initialization owner. Existing
    /// parsed history is not qualified by calling this constructor.
    pub(crate) fn fresh() -> Self {
        let fresh = surgeist_html::HtmlDialogFreshState::fresh();
        Self {
            return_value: fresh.return_value().into(),
            services: ClosedDialogServices::from_fresh(fresh),
        }
    }
    pub(crate) fn return_value(&self) -> &DocumentString {
        &self.return_value
    }
    pub(crate) fn with_return_value(&self, return_value: DocumentString) -> Self {
        Self {
            return_value,
            services: self.services,
        }
    }
    pub(crate) const fn is_modal(&self) -> bool {
        self.services.is_modal()
    }
    pub(crate) const fn has_close_watcher(&self) -> bool {
        self.services.has_watcher()
    }
}

/// A Dialog at one actual immutable canonical revision. An advanced returnValue
/// is read from retained state; the remaining null/false results are established
/// by the closed operation domain, not reconstructed from its open attribute.
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
        self.state().services.has_tracker()
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
/// Current native constructors establish known empty lists. Positive hosted
/// operations must consume the same canonical lists, never a second top layer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DocumentInteractiveState<K> {
    Known {
        open_dialogs: Vec<K>,
        top_layer: Vec<K>,
    },
    Unavailable,
}
impl<K> DocumentInteractiveState<K> {
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
