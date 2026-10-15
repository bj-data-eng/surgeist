//! Form scalar readback and the canonical identity-bearing past names map.
use crate::{DocumentString, DomNode, NodeId};
use std::sync::Arc;

/// Checked native Form view at one immutable canonical revision.
#[derive(Clone, Copy, Debug)]
pub struct DomForm<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomForm<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> surgeist_html::HtmlFormNativeState {
        self.node
            .as_element()
            .expect("checked Form")
            .form_state()
            .expect("checked Form")
    }
    /// Retained exact-name aliases. Native creation/clone starts empty; this
    /// surface does not perform named lookup or install past-name history.
    pub fn past_names(self) -> impl ExactSizeIterator<Item = (&'a DocumentString, NodeId)> {
        self.node
            .node
            .form_past_names
            .as_ref()
            .expect("checked Form")
            .entries
            .iter()
            .map(|entry| (&entry.name, entry.target))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FormPastNames<K> {
    pub(crate) entries: Arc<Vec<FormPastName<K>>>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FormPastName<K> {
    pub(crate) name: DocumentString,
    pub(crate) target: K,
}
impl<K> FormPastNames<K> {
    pub(crate) fn fresh() -> Self {
        Self {
            entries: Arc::new(Vec::new()),
        }
    }
}
