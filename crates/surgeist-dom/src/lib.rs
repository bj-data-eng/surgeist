//! Canonical ownership for a bounded native XML document tree.
//!
//! One [`Dom`] exclusively mutates its documents. Batches publish atomically,
//! and owned snapshots retain their exact identities and data after later edits.
//! Generic XML Element creation and same-store adoption are supported. HTML
//! element initialization, namespace-aware element creation and import are not exposed.
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::{Dom, DomBatch, DomLimits};
//!
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 8, 8192)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let comment = batch.create_comment(&document, "retained");
//! batch.insert_before(&comment, &document, None);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let comment = committed.created(&comment)?.expect("surviving creation");
//! let snapshot = dom.snapshot();
//! assert!(snapshot.node(comment)?.is_connected());
//! # Ok::<(), surgeist_dom::DomError>(())
//! ```
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::{Dom, DomBatch, DomLimits, Namespace};
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 8, 8192)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let root = batch.create_element(&document, "Root");
//! batch.set_attribute_ns(&root, Namespace::from_uri("urn:example"), "p:key", "value");
//! batch.insert_before(&root, &document, None);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let snapshot = dom.snapshot();
//! let root = snapshot.node(committed.created(&root)?.unwrap())?;
//! assert_eq!(root.node_name().to_utf8()?, "Root");
//! assert_eq!(root.as_element().unwrap().attributes().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod batch;
mod document;
mod element;
mod identity;
mod names;
mod node_values;
mod snapshot;
mod store;

pub use batch::{CreatedNode, DomBatch, NodeOperand};
pub use document::{DomDocumentClass, DomDocumentMetadata, DomOpaqueOrigin, DomRegistryBinding};
pub use element::{DomElement, DomElementDefinition};
pub use identity::{DocumentId, DomOwnerId, DomRevision, DomStamp, NodeId};
pub use node_values::{DomDocumentType, DomProcessingInstruction};
pub use snapshot::{DomDocument, DomDocumentStamp, DomNode, DomNodeKind, DomSnapshot};
pub use store::{
    Dom, DomChange, DomChanges, DomCommit, DomError, DomHistoryError, DomLimits, DomResource,
    DomUsage,
};
pub use surgeist_html::{
    Attribute, DocumentString, HtmlCustomElementState, HtmlDocumentMode, Namespace,
    OrderedAttributes, QualifiedName,
};
