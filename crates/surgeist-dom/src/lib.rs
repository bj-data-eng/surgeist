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
//! node Document's private acquisition/execution qualification and a fresh
//! operational recipient check. Processing refuses applicable Rust Load/Error
//! listeners without adding a loader, executor or SVG event output.
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
//! let names = surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names)?)?;
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
//! use surgeist_dom::{Dom, DomBatch, DomLimits};
//! use surgeist_html::values::Namespace;
//! let names = surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192, names)?)?;
//! let mut batch = DomBatch::new();
//! let document = batch.create_document();
//! let root = batch.create_element(&document, "Root");
//! batch.set_attribute_ns(&root, Namespace::from_uri("urn:example"), "p:key", "value");
//! batch.insert_before(&root, &document, None);
//! let committed = dom.apply(batch, &AtomicBool::new(false))?;
//! let snapshot = dom.snapshot();
//! let root = snapshot.node(committed.created(&root)?.unwrap())?;
//! assert_eq!(root.node_name().to_utf8()?, "Root");
//! assert_eq!(root.attributes().unwrap().len(), 1);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! ```
//! use std::sync::atomic::AtomicBool;
//! use surgeist_dom::{Dom, DomBatch, DomLimits, DomSlotQueryLimits};
//! use surgeist_html::construction::{
//!     HtmlShadowRootMode, HtmlShadowRootProperties, HtmlSlotAssignment};
//! use surgeist_html::values::Namespace;
//! let names = surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576)?;
//! let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 64, 32, 8, 8192, names)?)?;
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

mod internal;

pub use internal::compound::DomNodeOrString;
pub use internal::element_tasks::{
    DomElementTaskId, DomElementTaskRetention, DomElementTaskSource, DomElementTaskSourceKind,
};
pub use internal::event::{
    DomEventParent, DomEventParentQuery, DomEventPayloadRetention, DomEventRecipient,
    DomEventRecipientLimits, DomEventRecipientTarget, DomEventRegistryAssociation,
    DomEventRegistryPreparation,
};
pub use internal::inline_style::{
    DomInlineStyleBatchContext, DomInlineStyleCapture, DomInlineStyleChange, DomInlineStyleCommit,
    DomInlineStyleCompletion, DomInlineStyleError, DomInlineStyleFeedbackLimits,
    DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleMutation,
    DomInlineStyleMutationLimits, DomInlineStyleOutcome, DomInlineStyleProfile,
    DomInlineStyleProperty, DomInlineStyleRequestLimits, DomInlineStyleReturn,
    DomInlineStyleSourceContext, DomInlineStyleSourceProgress, DomInlineStyleSourcePublication,
    DomInlineStyleTerminal,
};
pub use internal::name::{DomAttribute, DomNameAtom, DomQualifiedName};
pub use internal::name_limits::DomNameLimits;
pub use internal::source::*;
pub use internal::store::DomInlineStyleOperation;

pub use internal::attr::{DomAttr, DomAttributeAttachment, DomNamedNodeMap};
pub use internal::autofocus::DomDocumentAutofocus;
pub use internal::base::{DomBase, DomBaseFactsError, DomBaseResponse, DomFrozenBase, DomLink};
pub use internal::batch::{
    AttachedAttribute, AttachedShadowRoot, CreatedNode, DomBatch, NodeOperand, RemovedAttribute,
};
pub use internal::child_list::{
    DomChildListObservationLimits, DomChildListObserverId, DomChildListRecord,
};
pub use internal::details::{
    DomDetails, DomDetailsToggleCancellationReason, DomDetailsToggleIntention,
    DomDetailsToggleIntentionId, DomDetailsToggleTransfer, DomElementTaskAssociation,
};
pub use internal::dialog::{DomCloseWatcherId, DomDialog, DomDialogToggleTracker};
pub use internal::dispatch::{
    DomDispatchAssociation, DomDispatchCustody, DomDispatchCustodyLimits,
};
pub use internal::document::DomNativeInputPlatform;
pub use internal::document::{
    DomDocumentClass, DomDocumentCreationCause, DomDocumentEnvironment, DomDocumentInterface,
    DomDocumentMetadata, DomFocusedAreaDesignation, DomOpaqueOrigin, DomRegistryBinding,
};
pub use internal::element::{DomElement, DomElementDefinition};
pub use internal::fullscreen::{
    DomFullscreenEvent, DomFullscreenEventBatch, DomFullscreenEventKind, DomFullscreenExitFact,
    DomFullscreenExitId, DomFullscreenExitReturn, DomFullscreenExitStatus, DomFullscreenExitStep,
    DomFullscreenModelStep,
};
pub use internal::host::{
    DomHostCapture, DomHostChange, DomHostChangeCoverage, DomHostChanges, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher,
    DomHostRecomputeReason, DomHostSourceId, DomHostStamp, HostStateRevision, Observed,
};
pub use internal::hosted::{
    DomCreationRealmFallback, DomDocumentScriptBinding, DomDocumentScriptRegistration,
    DomDocumentScriptingFacts, DomDocumentScriptingObservation,
};
pub use internal::hosted::{DomDocumentConstructorContext, DomDocumentConstructorError};
pub use internal::hosted::{
    DomHostedBrowsingContextId, DomHostedChildHistoryPublication, DomHostedChildHistoryRequest,
    DomHostedChildPreparation, DomHostedChildRemovalHandoff, DomHostedChildRemovalPublication,
    DomHostedChildRemovalRequest, DomHostedChildRequest, DomHostedChildStep, DomHostedCreation,
    DomHostedCreationLease, DomHostedCreationPublication, DomHostedCreationStep,
    DomHostedDestructionInvocation, DomHostedDestructionInvocationRequest,
    DomHostedDestructionLease, DomHostedDestructionPublication, DomHostedDestructionRequest,
    DomHostedDestructionStep, DomHostedDocumentDestructionProof, DomHostedEmbedderKind,
    DomHostedEmbedderPolicyInputs, DomHostedEntry, DomHostedEntryId, DomHostedEntryInitialization,
    DomHostedEntryRequest, DomHostedError, DomHostedExecutionRequest,
    DomHostedFinishLoadingRequest, DomHostedGlobalId, DomHostedHistoryJobReceipt,
    DomHostedHistoryJobRequest, DomHostedHistoryStep, DomHostedHostId, DomHostedLimits,
    DomHostedMakeActiveRequest, DomHostedMoment, DomHostedNavigable, DomHostedNavigableId,
    DomHostedOperationId, DomHostedPolicyPublication, DomHostedPolicyRequest,
    DomHostedPreparedChild, DomHostedPreparedTopLevel, DomHostedQueuedHistory,
    DomHostedRecordLimits, DomHostedResource, DomHostedRuntimeRetention,
    DomHostedSheetConstruction, DomHostedSheetConstructionError,
    DomHostedSheetConstructionPublished, DomHostedSource, DomHostedSourceId,
    DomHostedSourceRevision, DomHostedStyleRootWithdrawal, DomHostedStyleRootWithdrawalPreparation,
    DomHostedStyleRootWithdrawalPublished, DomHostedTopLevelPreparation,
    DomInitialPermissionsPolicy, DomInitialPermissionsPolicyProfile, DomPermissionsPolicyValue,
};
pub use internal::hyperlink::{DomEncodingUrlFactsError, DomHyperlink};
pub use internal::identity::{DocumentId, DomAttrId, DomOwnerId, DomRevision, DomStamp, NodeId};
pub use internal::image::{
    DomImageUpdate, DomImageUpdateCancellationReason, DomImageUpdateId, DomImg,
};
pub use internal::input::{DomInput, DomInputState};
pub use internal::installation::{
    DomNativeHtmlAssociation, DomNativeHtmlBinding, DomNativeHtmlInstallError,
    DomNativeHtmlInstallLimits, DomNativeHtmlInstallResource, DomNativeHtmlInstallTarget,
    DomNativeHtmlInstallation,
};
pub use internal::meta::{
    DomDocumentCspState, DomDocumentPolicyContainer, DomMetaColorSchemeState, DomMetaDocumentState,
    DomMetaError, DomMetaThemeColorState,
};
pub use internal::meta_atomic::*;
pub use internal::node_iterator::{
    DomNodeFilterDecision, DomNodeIterator, DomNodeIteratorFilter, DomNodeIteratorId,
    DomNodeIteratorProgress,
};
pub use internal::node_values::{DomDocumentType, DomProcessingInstruction};
pub use internal::output::{DomOutput, DomOutputState};
pub use internal::parsed_association::*;
pub use internal::parsed_installation::{
    DomParsedDocumentEnvironment, DomParsedHtmlAssociationDocument, DomParsedHtmlAssociationId,
    DomParsedHtmlAssociationReadiness, DomParsedHtmlBinding, DomParsedHtmlCapturedBinding,
    DomParsedHtmlHistoricalDocument, DomParsedHtmlInstallError, DomParsedHtmlInstallLimits,
    DomParsedHtmlInstallResource, DomParsedHtmlInstallation, DomParsedHtmlPendingAssociations,
};
pub use internal::parsed_meta::*;
pub use internal::parsed_style::*;
pub use internal::range::{
    CreatedRange, DomActiveRangeState, DomRange, DomRangeBoundary, DomRangeId, DomRangeOperand,
};
pub use internal::registry::{DomRegistry, DomRegistryDefinitionId, DomRegistryId};
pub use internal::script::DomHtmlScript;
pub use internal::select::{
    DomNativeCheckpoint, DomNativeCheckpointLimits, DomNativeTaskId, DomOption, DomOptionState,
    DomSelect, DomSelectCompletionStart, DomSelectState, DomSelectTask,
    DomSelectTaskCancellationReason, DomSelectTaskCompletion, DomSelectTaskOutcome,
    DomSelectedcontent,
};
pub use internal::semantic::{
    DomHtmlCapture, DomHtmlCaptureError, DomHtmlCaptureLimits, DomHtmlCaptureResource,
    DomScriptingCapture, DomSlotSemanticLimits,
};
pub use internal::shadow::{DomShadowRootInit, DomShadowRootState};
pub use internal::slot::{
    DomFlatRelation, DomFlatTree, DomSlot, DomSlotNodes, DomSlotNotificationTarget,
    DomSlotNotifications, DomSlotQueryLimits, DomSlotSignalCancellationReason,
};
pub use internal::snapshot::{
    DomDocument, DomDocumentPosition, DomDocumentStamp, DomNamespaceUri, DomNode, DomNodeKind,
    DomShadowRoot, DomSiblingDirection, DomSnapshot, DomTagName, DomTreeOrder, DomTreeScope,
};
pub use internal::store::DomChildListNotification;
pub use internal::store::DomNodeIteratorFilterRequest;
pub use internal::store::meta_atomic::{DomMetaOrdinaryPreparation, DomMetaOrdinaryPublished};
pub use internal::store::pending_associations::{
    DomParsedHtmlMetaOccurrenceCompletion, DomParsedHtmlMetaOccurrenceError,
    DomParsedHtmlMetaOccurrencePreparation, DomParsedHtmlMetaOccurrencePublished,
    DomParsedHtmlMetaOccurrenceRetirement, DomParsedHtmlMetaPreparation,
    DomParsedHtmlMetaPublished, DomParsedHtmlMetaRetirement, DomParsedHtmlMetaRuntimeHold,
    DomParsedHtmlStylePrefixView, DomParsedHtmlStyleRuntimeHold, DomParsedHtmlStyleSheetCompletion,
    DomParsedHtmlStyleSheetPreparation, DomParsedHtmlStyleSheetPublished,
    DomParsedHtmlStyleSheetRetirement, DomParsedHtmlStyleSubjectCompletion,
};
pub use internal::store::registry::DomRegistryDefinitionPreparation;
pub use internal::store::source::operation::dialog_queue::{
    DomDialogCloseTaskInfo, DomDialogTask, DomDialogTaskInfo, DomDialogTaskQueueCommit,
    DomDialogTaskQueuePlan, DomDialogTaskQueuePublication, DomDialogToggleTaskInfo,
};
pub use internal::store::{
    Dom, DomChange, DomChanges, DomCommit, DomError, DomHistoryError, DomLimits,
    DomNodeRetirementInbox, DomNodeRetirementLimits, DomNodeRetirementNotice,
    DomNodeRetirementWatch, DomResource, DomUsage, DomWatchedNode,
};
pub use internal::svg_script::{DomSvgScript, DomSvgScriptOrigin};
pub use internal::template::{DomTemplate, DomTemplateDocumentRole, DomTemplatePatch};
pub use internal::textarea::{DomTextarea, DomTextareaReadLimits, DomTextareaState};
pub use internal::top_layer::{DomDocumentTopLayer, DomTopLayerUaRule};
pub use internal::view_transition::{
    CreatedViewTransition, CreatedViewTransitionRecord, DomDocumentViewTransitions,
    DomPreparedViewTransitionRules, DomPreparedViewTransitionSheet, DomViewTransition,
    DomViewTransitionBranches, DomViewTransitionCaptureLimits, DomViewTransitionCssomError,
    DomViewTransitionId, DomViewTransitionImageRules, DomViewTransitionPseudo,
    DomViewTransitionPseudoChildren, DomViewTransitionPseudoId, DomViewTransitionPseudoKind,
    DomViewTransitionRecord, DomViewTransitionRecordId, DomViewTransitionRuleAssociations,
    DomViewTransitionRules, DomViewTransitionSheet, DomViewTransitionSubject,
    ViewTransitionOperand, ViewTransitionRecordOperand,
};

pub use internal::button::{DomButton, DomButtonState};
pub use internal::fieldset::{DomFieldset, DomFieldsetState};
pub use internal::form::DomForm;

pub use internal::store::source::operation::fullscreen::{
    DomFullscreenExitHandoffCommit, DomFullscreenExitJob, DomSourceFullscreenExitHandoffRequest,
};
pub use internal::store::source::operation::meta::{
    DomMetaSourcePreparation, DomMetaSourcePublished, DomMetaSourceRuntimeHold, DomMetaSourceStep,
    DomSourceMetaCspError, DomSourceMetaCspProcessorFailure,
};
pub use internal::store::source::operation::{
    DomPreparedElementName, DomShadowReactionRoute, DomSourceCustomElementCallbackScope,
    DomSourceCustomElementDisposition, DomSourceCustomElementOutcome,
    DomSourceCustomElementRetention, DomSourceCustomElementReturn, DomSourceCustomElementUpgrade,
    DomSourceEventFiringLoan, DomSourceLease, DomSourceNodeOutputWatch, DomStyleCspBudget,
    DomWatchedCustomElementResult, DomWatchedElementCreation,
};
pub use internal::store::source::reactions::{
    DomCustomElementReaction, DomCustomElementReactionKind, DomCustomElementReactions,
    DomSynchronousReactionQueue,
};
pub use internal::style::{
    DomStyle, DomStyleAction, DomStyleAssociationId, DomStyleCspRetention,
    DomStyleCssomOwnerBinding, DomStyleCssomPublicationRejection, DomStyleCssomRoot,
    DomStyleCssomRootError, DomStyleInlineCspObservation, DomStyleNonceContent,
    DomStyleNotificationCompletion, DomStyleNotificationDelivery, DomStyleNotificationOrigin,
    DomStyleNotificationToken, DomStyleRequest, DomStyleRootKind, DomStyleSheet,
    DomStyleSheetAvailability, DomStyleSheetPlacement, DomStyleSource, DomStyleState,
};
