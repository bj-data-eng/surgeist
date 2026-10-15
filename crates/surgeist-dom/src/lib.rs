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
//! Native Slot retains manual intent and stored assignment separately from fresh
//! assignedSlot/flattened/CSS flat queries. An explicit intrinsic checkpoint takes
//! slotchange intentions; no callbacks run inside a transaction. Remaining special
//! families require their real adapters.
//! Native Fieldset retains actual association/custom-error state and bounded
//! source reset hooks. Custom messages normalize through HTML's shared rule;
//! semantic capture and Slot supplementation retain actual control facts.
//! Native Button shares those controls and retains independent weak reflected
//! targets, source-ordered setters, scoped semantic getters and retirement cleanup.
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::{Dom, DomBatch, DomLimits};
//!
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192)?)?;
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
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192)?)?;
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
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::{Dom, DomBatch, DomLimits, DomSlotQueryLimits, Namespace,
//!     HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 64, 32, 8, 8192)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let host = batch.create_element_ns(&document, Namespace::html(), "div");
//! batch.insert_before(&host, &document, None);
//! let root = batch.attach_shadow(&host, HtmlShadowRootProperties::new(
//!     HtmlShadowRootMode::Open, HtmlSlotAssignment::Manual, false, false, false));
//! let slot = batch.create_element_ns(&document, Namespace::html(), "slot");
//! let text = batch.create_text(&document, "assigned");
//! batch.insert_before(&slot, &root, None);
//! batch.insert_before(&text, &host, None);
//! batch.assign_slot(&slot, vec![(&text).into()]);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let slot = committed.created(&slot)?.unwrap();
//! let text = committed.created(&text)?.unwrap();
//! assert_eq!(dom.snapshot().slot(slot)?.assigned_nodes(), &[text]);
//! let notifications = dom.take_slot_notifications(
//!     DomSlotQueryLimits::new(32, 64)?, &AtomicBool::new(false))?;
//! assert_eq!(notifications.targets(), &[slot]);
//! # Ok::<(), surgeist_dom::DomError>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod batch;
mod button;
mod control;
mod document;
mod element;
mod fieldset;
mod form;
mod host;
mod identity;
mod names;
mod node_values;
mod semantic;
mod shadow;
mod slot;
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
    DomSlotSemanticLimits,
};
pub use shadow::{DomShadowRootInit, DomShadowRootState};
pub use slot::{
    DomFlatRelation, DomFlatTree, DomSlot, DomSlotNodes, DomSlotNotificationTarget,
    DomSlotNotifications, DomSlotQueryLimits, DomSlotSignalCancellationReason,
};
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

pub use button::{DomButton, DomButtonState};
pub use fieldset::{DomFieldset, DomFieldsetState};
pub use form::DomForm;
