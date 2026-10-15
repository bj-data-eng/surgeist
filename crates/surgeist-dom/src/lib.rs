//! Canonical ownership for a bounded native document tree.
//!
//! One [`Dom`] exclusively mutates its documents. Batches publish atomically,
//! and owned snapshots retain their exact identities and data after later edits.
//! Native XML and complete HTML document creation, exact namespaced import and
//! same-store adoption are supported. HTML API casing follows the current node
//! document while stored names and namespace-aware operations remain exact.
//! Explicitly initialized View Transitions retain canonical
//! structural state and real CSSOM associations, without rendering resources.
//! Namespace-aware common HTML/SVG/MathML elements retain actual shared common
//! state and nonce lifecycle. Native ShadowRoots retain distinct host/ordinary
//! relations and follow atomic adoption, clone/import and owned destruction.
//! Native Canvas-none and Marquee families retain HTML-owned state and execute
//! their actual attribute/value transitions without allocating pixels or animation.
//! Native Frame retains actual null child-content and consumes source-qualified
//! post-connection, src and removing decisions without navigation capability.
//! Slot assignment and remaining special families require their real adapters.
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
mod host;
mod identity;
mod names;
mod node_values;
mod semantic;
mod shadow;
mod snapshot;
mod store;
mod view_transition;

pub use batch::{CreatedNode, DomBatch, NodeOperand};
pub use document::{
    DomDocumentClass, DomDocumentMetadata, DomNativeDocumentEnvironment, DomOpaqueOrigin,
    DomRegistryBinding,
};
pub use element::{DomElement, DomElementDefinition};
pub use host::{
    DomHostCapture, DomHostChange, DomHostChangeCoverage, DomHostChanges, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher,
    DomHostRecomputeReason, DomHostSourceId, DomHostStamp, HostStateRevision, Observed,
};
pub use identity::{DocumentId, DomOwnerId, DomRevision, DomStamp, NodeId};
pub use node_values::{DomDocumentType, DomProcessingInstruction};
pub use semantic::{
    DomHtmlCapture, DomHtmlCaptureError, DomHtmlCaptureLimits, DomHtmlCaptureResource,
};
pub use shadow::{DomShadowRootInit, DomShadowRootState};
pub use snapshot::{
    DomDocument, DomDocumentStamp, DomNode, DomNodeKind, DomShadowRoot, DomSnapshot, DomTreeOrder,
    DomTreeScope,
};
pub use store::{
    Dom, DomChange, DomChanges, DomCommit, DomError, DomHistoryError, DomLimits, DomResource,
    DomUsage,
};
pub use surgeist_css::CssViewTransitionIdent;
pub use surgeist_html::{
    Attribute, DocumentString, HtmlCustomElementState, HtmlDocumentMode, HtmlShadowRootMode,
    HtmlShadowRootProperties, HtmlSlotAssignment, Namespace, OrderedAttributes, QualifiedName,
};
pub use view_transition::{
    CreatedViewTransition, CreatedViewTransitionRecord, DomDocumentViewTransitions,
    DomViewTransition, DomViewTransitionBranches, DomViewTransitionCssomError, DomViewTransitionId,
    DomViewTransitionImageRules, DomViewTransitionPseudo, DomViewTransitionPseudoChildren,
    DomViewTransitionPseudoId, DomViewTransitionPseudoKind, DomViewTransitionRecord,
    DomViewTransitionRecordId, DomViewTransitionRuleAssociations, DomViewTransitionRules,
    DomViewTransitionSheet, DomViewTransitionSubject, ViewTransitionOperand,
    ViewTransitionRecordOperand,
};
