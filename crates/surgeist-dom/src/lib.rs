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
//! Native Input retains actual current/dirty/check/selection/file/Image state,
//! all shared type sanitizers, radio groups and one weak popover slot. Explicit
//! native platform facts and finite numeric/CSS processing remain independent.
//! Native Textarea retains exact raw value and shared flags/selection; bounded
//! API/default reads, setters/reset, immediate Text hooks and capture preserve
//! current/default distinctions without a second value owner or parser history.
//! Native Output stores one nullable default override and consumes the shared
//! ordered setter/reset cursor with real string-replace-all hooks. Existing
//! bounded semantic queries read exact ordinary descendant/default text, and
//! both initial and Slot captures preserve null versus established empty.
//! Native Template owns real separate contents and cached inert Documents;
//! clone/adopt/retirement preserve source ownership and document distinctions.
//! Native HTML Script retains complete supported state and real document history.
//! Source-ordered preparation reaches the actual Disabled return without fetching
//! or executing; attributes, children and CharacterData invoke their real hooks.
//! Native SVG Script separately retains the exact processed flag and Native/Clone
//! cause. The shared static shadow-including insertion walk consumes the actual
//! node Document's private closed acquisition/execution/notification witness.
//! This makes the processed write without a loader, executor or SVG event output.
//! Explicit native HTML fragment installation preserves immutable component
//! fields and UTF-16 through that same canonical owner, including real native
//! Template contents and Script hooks. It returns a source-qualified historical
//! map and installed snapshot. Completed offline parser reports use separate
//! canonical installation entries that preserve actual retained history and
//! qualified contextual effects, with typed refusal for unsupported profiles.
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
//! let surgeist_dom::DomNativeCheckpoint::Slots(notifications) = dom.native_checkpoint_step(
//!     surgeist_dom::DomNativeCheckpointLimits::new(0, 0, DomSlotQueryLimits::new(32, 64)?)?, &AtomicBool::new(false))? else { panic!("reached Slot marker") };
//! assert_eq!(notifications.targets(), &[slot]);
//! # Ok::<(), surgeist_dom::DomError>(())
//! ```
#![forbid(unsafe_code)]
#![deny(unfulfilled_lint_expectations)]

mod base;
mod batch;
mod button;
mod control;
mod details;
mod dialog;
mod document;
mod element;
mod fieldset;
mod form;
mod host;
mod hosted;
mod hyperlink;
mod identity;
mod image;
mod input;
mod installation;
mod meta;
mod names;
mod node_values;
mod output;
mod parsed_installation;
mod reflected;
mod registry;
mod script;
mod select;
mod semantic;
mod shadow;
mod slot;
mod snapshot;
mod store;
mod style;
mod svg_script;
mod template;
mod textarea;
mod view_transition;

pub use base::{DomBase, DomBaseFactsError, DomBaseResponse, DomFrozenBase, DomLink};
pub use batch::{CreatedNode, DomBatch, NodeOperand};
pub use details::{
    DomDetails, DomDetailsQueueAssociation, DomDetailsToggleCancellationReason,
    DomDetailsToggleIntention, DomDetailsToggleIntentionId, DomDetailsToggleTransfer,
};
pub use dialog::DomDialog;
pub use document::DomNativeInputPlatform;
pub use document::{
    DomDocumentClass, DomDocumentCreationCause, DomDocumentEnvironment, DomDocumentMetadata,
    DomOpaqueOrigin, DomRegistryBinding,
};
pub use element::{DomElement, DomElementDefinition};
pub use host::{
    DomHostCapture, DomHostChange, DomHostChangeCoverage, DomHostChanges, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher,
    DomHostRecomputeReason, DomHostSourceId, DomHostStamp, HostStateRevision, Observed,
};
pub use hosted::{
    DomFullscreenInherited, DomHostedBrowsingContextId, DomHostedCreation, DomHostedCreationLease,
    DomHostedCreationPublication, DomHostedCreationStep, DomHostedDestructionLease,
    DomHostedDestructionPublication, DomHostedDestructionRequest, DomHostedDestructionStep,
    DomHostedEntryId, DomHostedEntryRequest, DomHostedError, DomHostedExecutionRequest,
    DomHostedFinishLoadingRequest, DomHostedGlobalId, DomHostedHostId, DomHostedLimits,
    DomHostedMakeActiveRequest, DomHostedMoment, DomHostedNavigableId, DomHostedOperationId,
    DomHostedPolicyPublication, DomHostedPolicyRequest, DomHostedPreparedTopLevel,
    DomHostedRecordLimits, DomHostedResource, DomHostedSource, DomHostedSourceId,
    DomHostedSourceRevision, DomHostedTopLevelPreparation, DomInitialPermissionsPolicy,
    DomInitialPermissionsPolicyProfile,
};
pub use hyperlink::{DomEncodingUrlFactsError, DomHyperlink};
pub use identity::{DocumentId, DomOwnerId, DomRevision, DomStamp, NodeId};
pub use image::{DomImageUpdate, DomImageUpdateCancellationReason, DomImageUpdateId, DomImg};
pub use input::{DomInput, DomInputState};
pub use installation::{
    DomNativeHtmlAssociation, DomNativeHtmlBinding, DomNativeHtmlInstallError,
    DomNativeHtmlInstallLimits, DomNativeHtmlInstallResource, DomNativeHtmlInstallation,
};
pub use meta::{DomDocumentPolicyContainer, DomMetaDocumentState, DomMetaError};
pub use node_values::{DomDocumentType, DomProcessingInstruction};
pub use output::{DomOutput, DomOutputState};
pub use parsed_installation::{
    DomParsedDocumentEnvironment, DomParsedHtmlAssociationReadiness, DomParsedHtmlBinding,
    DomParsedHtmlCapturedBinding, DomParsedHtmlInstallError, DomParsedHtmlInstallLimits,
    DomParsedHtmlInstallResource, DomParsedHtmlInstallation,
};
pub use registry::{DomRegistry, DomRegistryId};
pub use script::DomHtmlScript;
pub use select::{
    DomNativeCheckpoint, DomNativeCheckpointLimits, DomNativeTaskId, DomOption, DomOptionState,
    DomSelect, DomSelectState, DomSelectTask, DomSelectTaskCancellationReason,
    DomSelectTaskCompletion, DomSelectTaskOutcome, DomSelectedcontent,
};
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
pub use svg_script::{DomSvgScript, DomSvgScriptOrigin};
pub use template::{DomTemplate, DomTemplateDocumentRole, DomTemplatePatch};
pub use textarea::{DomTextarea, DomTextareaReadLimits, DomTextareaState};
pub use view_transition::{
    CreatedViewTransition, CreatedViewTransitionRecord, DomDocumentViewTransitions,
    DomPreparedViewTransitionRules, DomPreparedViewTransitionSheet, DomViewTransition,
    DomViewTransitionBranches, DomViewTransitionCaptureLimits, DomViewTransitionCssomError,
    DomViewTransitionId, DomViewTransitionImageRules, DomViewTransitionPseudo,
    DomViewTransitionPseudoChildren, DomViewTransitionPseudoId, DomViewTransitionPseudoKind,
    DomViewTransitionRecord, DomViewTransitionRecordId, DomViewTransitionRuleAssociations,
    DomViewTransitionRules, DomViewTransitionSheet, DomViewTransitionSubject,
    ViewTransitionOperand, ViewTransitionRecordOperand,
};

pub use button::{DomButton, DomButtonState};
pub use fieldset::{DomFieldset, DomFieldsetState};
pub use form::DomForm;

pub use store::style::operation::DomStyleEditLease;
pub use style::{
    DomStyle, DomStyleAction, DomStyleAssociationId, DomStyleCssomOwnerBinding,
    DomStyleCssomPublicationRejection, DomStyleCssomRoot, DomStyleCssomRootError,
    DomStyleEditCompletion, DomStyleEditLimits, DomStyleEditStatus, DomStyleInlineCspObservation,
    DomStyleNativeHtmlInstallLimits, DomStyleNotificationCompletion, DomStyleNotificationToken,
    DomStyleOperationId, DomStyleProgress, DomStyleRequest, DomStyleRootKind, DomStyleSheet,
    DomStyleSheetAvailability, DomStyleSheetPlacement, DomStyleSource, DomStyleState,
};
