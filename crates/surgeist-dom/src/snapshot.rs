use std::sync::Arc;

use crate::store::{Node, State};
use crate::{DocumentId, DocumentString, DomError, DomOwnerId, DomRevision, DomStamp, NodeId};
use crate::{DomDocumentType, DomProcessingInstruction};

/// The complete node kinds supported by this contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomNodeKind {
    Document,
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
        if self.node(id.node())?.kind() != &DomNodeKind::Document {
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
            DomNodeKind::Document => "#document".into(),
            DomNodeKind::DocumentFragment => "#document-fragment".into(),
            DomNodeKind::Text(_) => "#text".into(),
            DomNodeKind::Comment(_) => "#comment".into(),
            DomNodeKind::ProcessingInstruction(value) => value.target().clone(),
            DomNodeKind::DocumentType(value) => value.name().clone(),
        }
    }
    /// Checked document view, including for documents discovered by enumeration.
    pub fn as_document(self) -> Option<DomDocument<'a>> {
        (self.node.kind == DomNodeKind::Document).then_some(DomDocument {
            snapshot: self.snapshot,
            id: DocumentId(self.id),
        })
    }
    /// DOM's ownerDocument getter: None for Document, otherwise node document.
    pub fn owner_document(self) -> Option<DocumentId> {
        (self.node.kind != DomNodeKind::Document).then_some(self.node.document)
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
        self.snapshot
            .node(self.root())
            .expect("snapshot root invariant")
            .kind()
            == &DomNodeKind::Document
    }
}
