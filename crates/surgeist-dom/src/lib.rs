//! Canonical ownership for a bounded native, non-element document tree.
//!
//! One [`Dom`] exclusively mutates its documents. Batches publish atomically,
//! and owned snapshots retain their exact identities and data after later edits.
//! Element creation, HTML initialization, adoption and import are not exposed.
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
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod batch;
mod identity;
mod snapshot;
mod store;

pub use batch::{CreatedNode, DomBatch, NodeOperand};
pub use identity::{DocumentId, DomOwnerId, DomRevision, DomStamp, NodeId};
pub use snapshot::{DomDocument, DomDocumentStamp, DomNode, DomNodeKind, DomSnapshot};
pub use store::{
    Dom, DomChange, DomChanges, DomCommit, DomError, DomHistoryError, DomLimits, DomResource,
    DomUsage,
};
pub use surgeist_html::DocumentString;
