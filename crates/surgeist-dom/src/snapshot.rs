use std::sync::Arc;

use crate::store::{Node, State};
use crate::{DocumentId, DocumentString, DomError, DomOwnerId, DomRevision, DomStamp, NodeId};
use crate::{DomDocumentMetadata, DomDocumentType, DomElement, DomProcessingInstruction};

/// The complete node kinds supported by this contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomNodeKind {
    Document(DomDocumentMetadata),
    Element(DomElement),
    DocumentFragment,
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(DomProcessingInstruction),
    DocumentType(DomDocumentType),
}

/// Owned immutable state, including detached nodes and all documents.
#[derive(Clone, Debug)]
pub struct DomSnapshot {
    pub(crate) owner: DomOwnerId,
    pub(crate) revision: DomRevision,
    pub(crate) state: Arc<State>,
}

impl DomSnapshot {
    pub fn owner(&self) -> DomOwnerId {
        self.owner
    }
    pub fn revision(&self) -> DomRevision {
        self.revision
    }
    pub fn stamp(&self) -> DomStamp {
        DomStamp {
            owner: self.owner,
            revision: self.revision,
        }
    }
    pub fn node(&self, id: NodeId) -> Result<DomNode<'_>, DomError> {
        let node = self.state.node(self.owner, id)?;
        Ok(DomNode {
            snapshot: self,
            id,
            node,
        })
    }
    pub fn document(&self, id: DocumentId) -> Result<DomDocument<'_>, DomError> {
        if !matches!(self.node(id.node())?.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(DomDocument { snapshot: self, id })
    }
    /// Every managed node, including detached nodes. Arena order is unspecified.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.state
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, value)| {
                value.node.as_ref().map(|_| NodeId {
                    owner: self.owner,
                    slot,
                    generation: value.generation,
                })
            })
    }
}

/// A document-qualified view over one exact immutable store revision.
#[derive(Clone, Copy, Debug)]
pub struct DomDocument<'a> {
    snapshot: &'a DomSnapshot,
    id: DocumentId,
}

impl<'a> DomDocument<'a> {
    pub fn id(self) -> DocumentId {
        self.id
    }
    /// Canonical metadata belonging to this exact captured document revision.
    pub fn metadata(self) -> &'a DomDocumentMetadata {
        let DomNodeKind::Document(metadata) = self
            .snapshot
            .node(self.id.node())
            .expect("captured document")
            .kind()
        else {
            unreachable!("checked document view")
        };
        metadata
    }
    pub fn stamp(self) -> DomDocumentStamp {
        DomDocumentStamp {
            store: self.snapshot.stamp(),
            document: self.id,
        }
    }
    /// Validates membership at this captured revision. A Document belongs to
    /// itself, although its source-defined ownerDocument getter returns None.
    pub fn node(self, id: NodeId) -> Result<DomNode<'a>, DomError> {
        let node = self.snapshot.node(id)?;
        if node.node.document != self.id {
            return Err(DomError::WrongDocument);
        }
        Ok(node)
    }
}

/// Exact store, document and revision qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDocumentStamp {
    store: DomStamp,
    document: DocumentId,
}
impl DomDocumentStamp {
    pub fn store(self) -> DomStamp {
        self.store
    }
    pub fn document(self) -> DocumentId {
        self.document
    }
}

/// A checked node read from an immutable snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomNode<'a> {
    snapshot: &'a DomSnapshot,
    id: NodeId,
    pub(crate) node: &'a Node,
}
impl<'a> DomNode<'a> {
    pub fn id(self) -> NodeId {
        self.id
    }
    pub fn kind(self) -> &'a DomNodeKind {
        &self.node.kind
    }
    /// DOM nodeName readback; PI targets and doctype names preserve exact units.
    pub fn node_name(self) -> DocumentString {
        match &self.node.kind {
            DomNodeKind::Document(_) => "#document".into(),
            DomNodeKind::Element(element) => crate::names::qualified_spelling(element.name()),
            DomNodeKind::DocumentFragment => "#document-fragment".into(),
            DomNodeKind::Text(_) => "#text".into(),
            DomNodeKind::Comment(_) => "#comment".into(),
            DomNodeKind::ProcessingInstruction(value) => value.target().clone(),
            DomNodeKind::DocumentType(value) => value.name().clone(),
        }
    }
    /// Checked document view, including for documents discovered by enumeration.
    pub fn as_document(self) -> Option<DomDocument<'a>> {
        matches!(self.node.kind, DomNodeKind::Document(_)).then_some(DomDocument {
            snapshot: self.snapshot,
            id: DocumentId(self.id),
        })
    }
    pub fn as_element(self) -> Option<&'a DomElement> {
        match &self.node.kind {
            DomNodeKind::Element(value) => Some(value),
            _ => None,
        }
    }
    /// Source tagName. Native XML Elements preserve their qualified spelling.
    pub fn tag_name(self) -> Option<DocumentString> {
        self.as_element()
            .map(|element| crate::names::qualified_spelling(element.name()))
    }
    pub fn parent_element(self) -> Option<NodeId> {
        self.parent().filter(|id| {
            self.snapshot
                .node(*id)
                .expect("captured parent")
                .as_element()
                .is_some()
        })
    }
    pub fn element_children(self) -> impl DoubleEndedIterator<Item = NodeId> + 'a {
        self.node.children.iter().copied().filter(move |id| {
            self.snapshot
                .node(*id)
                .expect("captured child")
                .as_element()
                .is_some()
        })
    }
    pub fn previous_sibling(self) -> Option<NodeId> {
        let parent = self.snapshot.node(self.parent()?).expect("captured parent");
        let position = parent
            .children()
            .iter()
            .position(|id| *id == self.id)
            .expect("captured sibling");
        position
            .checked_sub(1)
            .map(|position| parent.children()[position])
    }
    pub fn next_sibling(self) -> Option<NodeId> {
        let parent = self.snapshot.node(self.parent()?).expect("captured parent");
        let position = parent
            .children()
            .iter()
            .position(|id| *id == self.id)
            .expect("captured sibling");
        parent.children().get(position + 1).copied()
    }
    pub fn previous_element_sibling(self) -> Option<NodeId> {
        let mut next = self.previous_sibling();
        while let Some(id) = next {
            let node = self.snapshot.node(id).expect("captured sibling");
            if node.as_element().is_some() {
                return Some(id);
            }
            next = node.previous_sibling();
        }
        None
    }
    pub fn next_element_sibling(self) -> Option<NodeId> {
        let mut next = self.next_sibling();
        while let Some(id) = next {
            let node = self.snapshot.node(id).expect("captured sibling");
            if node.as_element().is_some() {
                return Some(id);
            }
            next = node.next_sibling();
        }
        None
    }
    /// Zero-based Element-only sibling index; detached Elements have index 0.
    pub fn element_index(self) -> Option<usize> {
        self.as_element()?;
        let Some(parent) = self.parent() else {
            return Some(0);
        };
        self.snapshot
            .node(parent)
            .expect("captured parent")
            .element_children()
            .position(|id| id == self.id)
    }
    /// DOM's ownerDocument getter: None for Document, otherwise node document.
    pub fn owner_document(self) -> Option<DocumentId> {
        (!matches!(self.node.kind, DomNodeKind::Document(_))).then_some(self.node.document)
    }
    pub fn parent(self) -> Option<NodeId> {
        self.node.parent
    }
    pub fn children(self) -> &'a [NodeId] {
        &self.node.children
    }
    pub fn data(self) -> Option<&'a DocumentString> {
        match &self.node.kind {
            DomNodeKind::Text(data) | DomNodeKind::Comment(data) => Some(data),
            DomNodeKind::ProcessingInstruction(value) => Some(value.data()),
            _ => None,
        }
    }
    /// Ordinary tree root; a detached node's owner document is not its root.
    pub fn root(self) -> NodeId {
        let mut id = self.id;
        let mut node = self.node;
        while let Some(parent) = node.parent {
            id = parent;
            node = self
                .snapshot
                .state
                .node(self.snapshot.owner, parent)
                .expect("snapshot parent invariant");
        }
        id
    }
    pub fn is_connected(self) -> bool {
        matches!(
            self.snapshot
                .node(self.root())
                .expect("snapshot root invariant")
                .kind(),
            DomNodeKind::Document(_)
        )
    }
}

impl DomSnapshot {
    pub fn view_transition(
        &self,
        id: crate::DomViewTransitionId,
    ) -> Result<crate::DomViewTransition<'_>, DomError> {
        let value = self.state.view_transitions.transition(self.owner, id)?;
        Ok(crate::DomViewTransition {
            snapshot: self,
            value,
        })
    }
    pub fn view_transition_record(
        &self,
        id: crate::DomViewTransitionRecordId,
    ) -> Result<crate::DomViewTransitionRecord<'_>, DomError> {
        let transition = self.view_transition(id.transition())?;
        let value = transition
            .value
            .records
            .iter()
            .find(|r| r.id == id)
            .ok_or(DomError::StaleViewTransitionRecord)?;
        Ok(crate::DomViewTransitionRecord { transition, value })
    }
    /// Complete relation reads require this snapshot's actual document element.
    pub fn view_transition_pseudo(
        &self,
        id: crate::DomViewTransitionPseudoId,
    ) -> Result<crate::DomViewTransitionPseudo<'_>, DomError> {
        let (value, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        let origin = self
            .document(value.document)?
            .document_element()
            .ok_or(DomError::ViewTransitionOriginUnavailable)?;
        Ok(crate::DomViewTransitionPseudo {
            transition: crate::DomViewTransition {
                snapshot: self,
                value,
            },
            record,
            id,
            origin,
        })
    }
    /// Identity/flag reads remain available without an originating element.
    pub fn view_transition_pseudo_captured(
        &self,
        id: crate::DomViewTransitionPseudoId,
    ) -> Result<bool, DomError> {
        let (transition, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        Ok(record.map_or(transition.root_captured, |r| {
            r.captured[crate::view_transition::flag_index(id.kind())]
        }))
    }
}
impl<'a> DomDocument<'a> {
    pub fn document_element(self) -> Option<NodeId> {
        self.snapshot
            .node(self.id.node())
            .expect("captured Document")
            .element_children()
            .next()
    }
    /// None means feature-uninitialized, not a source null stylesheet default.
    pub fn view_transitions(self) -> Option<&'a crate::DomDocumentViewTransitions> {
        self.snapshot
            .state
            .view_transitions
            .documents
            .iter()
            .find(|d| d.document == self.id)
    }
}
