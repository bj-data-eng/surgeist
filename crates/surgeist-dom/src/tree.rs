//! The read model of the canonical tree: documents, nodes, elements, names, attributes
//! and live node traversal.
//!
//! A `DomSnapshot` hands out checked views: `DomDocument`, `DomNode`, `DomElement`,
//! `DomAttr`, `DomNamedNodeMap`, `DomDocumentType` and `DomProcessingInstruction`. Names
//! are `DomQualifiedName`, `DomTagName`, `DomNameAtom` and `DomNamespaceUri`.
//! `DomTreeOrder` iterates the captured tree in order, `DomTreeScope` names one exact
//! root. `DomNodeIteratorId` identifies a live NodeIterator cursor of the owner,
//! `DomNodeIterator` is that cursor's public state at the time it was read, and the
//! filter, request and progress types drive its traversal.
//! `DomTreeWalkerId` and `DomTreeWalker` describe a separate live walker with a
//! settable current node and source-defined navigation in seven directions.
//! `DomHtmlCollection` retains one exact Document and an element filter.
//! Qualify a new `DomHtmlCollectionRead` to read its current members.
//!
//! Views are immutable and keep the snapshot lifetime; reading creates no node. A
//! NodeIterator starts before its root, walks ordinary tree order within the root, and
//! continues through descendants for both `Reject` and `Skip`. `docs/canonical-tree.md`
//! and `docs/node-iterators.md` own the contracts.

pub use crate::internal::attr::{DomAttr, DomAttributeAttachment, DomNamedNodeMap};
pub use crate::internal::element::DomElement;
pub use crate::internal::name::{DomAttribute, DomNameAtom, DomQualifiedName};
pub use crate::internal::node_iterator::{
    DomNodeFilterDecision, DomNodeIterator, DomNodeIteratorFilter, DomNodeIteratorId,
    DomNodeIteratorProgress,
};
pub use crate::internal::node_values::{DomDocumentType, DomProcessingInstruction};
pub use crate::internal::snapshot::{
    DomDocument, DomDocumentPosition, DomNamespaceUri, DomNode, DomNodeKind, DomSiblingDirection,
    DomTagName, DomTreeOrder, DomTreeScope,
};
pub use crate::internal::store::{
    DomHtmlCollection, DomHtmlCollectionLimits, DomHtmlCollectionNames, DomHtmlCollectionRead,
    DomNodeIteratorFilterRequest, DomNodeIteratorMatchRequest, DomWindowNamedElements,
};
pub use crate::internal::store::{DomTreeWalkerFilterRequest, DomTreeWalkerMatchRequest};
pub use crate::internal::table_part::DomHtmlTablePart;
pub use crate::internal::tree_walker::{
    DomTreeWalker, DomTreeWalkerDirection, DomTreeWalkerFilter, DomTreeWalkerId,
    DomTreeWalkerProgress,
};
