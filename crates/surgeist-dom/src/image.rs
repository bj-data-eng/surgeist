//! Native standalone image state and wholly inactive update ownership.
use crate::{DocumentId, DomNode, NodeId};
use surgeist_html::{HtmlImageUpdateFlags, HtmlImgNativeState};

/// An exact canonical target and its checked, nonwrapping update invocation.
/// This metadata grants no execution or activation authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct DomImageUpdateId {
    pub(crate) target: NodeId,
    pub(crate) invocation: u64,
}
impl DomImageUpdateId {
    pub fn target(self) -> NodeId {
        self.target
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomImageUpdateCancellationReason {
    TargetDestroyed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Waiting {
    pub(crate) invocation: u64,
    pub(crate) flags: HtmlImageUpdateFlags,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ImageState<K> {
    pub(crate) native: HtmlImgNativeState,
    pub(crate) dimension_source: K,
    pub(crate) parser_inserted: bool,
    // Independent of whether a particular invocation remains pending. A later
    // start supersedes an older wait even when the later invocation aborts.
    pub(crate) high_water: u64,
    pub(crate) waiting: Option<Waiting>,
}
impl<K: Copy> ImageState<K> {
    pub(crate) fn fresh(key: K) -> Self {
        Self {
            native: HtmlImgNativeState::fresh(),
            dimension_source: key,
            parser_inserted: false,
            high_water: 0,
            waiting: None,
        }
    }
    pub(crate) fn map<T>(&self, map: impl FnOnce(K) -> T) -> ImageState<T> {
        ImageState {
            native: self.native.clone(),
            dimension_source: map(self.dimension_source),
            parser_inserted: self.parser_inserted,
            high_water: self.high_water,
            waiting: self.waiting,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DomImg<'a> {
    node: DomNode<'a>,
}
/// Readback of a canonical invocation still waiting for the actual node
/// Document to become fully active. This metadata transfers no work ownership;
/// no continuation microtask has been queued.
#[derive(Clone, Copy, Debug)]
pub struct DomImageUpdate {
    id: DomImageUpdateId,
    document: DocumentId,
    flags: HtmlImageUpdateFlags,
}
impl DomImageUpdate {
    pub fn id(self) -> DomImageUpdateId {
        self.id
    }
    pub fn document(self) -> DocumentId {
        self.document
    }
    pub fn flags(self) -> HtmlImageUpdateFlags {
        self.flags
    }
}
impl<'a> DomImg<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a HtmlImgNativeState {
        &self.node.node.image.as_ref().expect("checked Img").native
    }
    pub fn dimension_attribute_source(self) -> NodeId {
        self.node
            .node
            .image
            .as_ref()
            .expect("checked Img")
            .dimension_source
    }
    pub fn form_owner(self) -> Option<NodeId> {
        self.node.node.control_owner
    }
    pub fn parser_inserted(self) -> bool {
        self.node
            .node
            .image
            .as_ref()
            .expect("checked Img")
            .parser_inserted
    }
    pub fn suspended_update(self) -> Option<DomImageUpdate> {
        let waiting = self.node.node.image.as_ref()?.waiting?;
        Some(DomImageUpdate {
            id: DomImageUpdateId {
                target: self.id(),
                invocation: waiting.invocation,
            },
            document: self.node.node.document,
            flags: waiting.flags,
        })
    }
}
impl<'a> DomNode<'a> {
    pub fn as_img(self) -> Option<DomImg<'a>> {
        self.node.image.as_ref()?;
        Some(DomImg { node: self })
    }
}
