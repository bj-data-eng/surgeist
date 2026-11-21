//! Proves that each of the 531 public items resolves at the module path the module plan
//! assigns it: the imports below fail to compile if an item is missing from, or not
//! reachable at, its planned module.
//!
//! Generated from the approved plan (`items.md` of issue 2919), not from the
//! implementation. This test does not prove that an item has only one path or that
//! the crate root holds only its five items; the regenerated public API audit checks
//! both.

#![expect(
    unused_imports,
    reason = "the imports are the test: each one resolves a public item at its module path"
)]

use surgeist_dom::capture::{
    DomHostCapture, DomHostChange, DomHostChangeCoverage, DomHostChanges, DomHostDomain,
    DomHostDomains, DomHostError, DomHostObservations, DomHostPublication, DomHostPublisher,
    DomHostRecomputeReason, DomHostSourceId, DomHostStamp, DomHtmlCapture, DomHtmlCaptureError,
    DomHtmlCaptureLimits, DomHtmlCaptureResource, DomScriptingCapture, DomSlotSemanticLimits,
    HostStateRevision, Observed,
};
use surgeist_dom::custom_element::{
    DomCustomElementReaction, DomCustomElementReactionKind, DomCustomElementReactions,
    DomElementDefinition, DomPreparedElementName, DomRegistry, DomRegistryBinding,
    DomRegistryDefinitionId, DomRegistryDefinitionPreparation, DomRegistryId,
    DomShadowReactionRoute, DomSourceCustomElementCallbackScope, DomSourceCustomElementDisposition,
    DomSourceCustomElementOutcome, DomSourceCustomElementRequest, DomSourceCustomElementRetention,
    DomSourceCustomElementReturn, DomSourceCustomElementUpgrade, DomSynchronousReactionQueue,
    DomWatchedCustomElementResult, DomWatchedElementCreation,
};
use surgeist_dom::details::{
    DomDetails, DomDetailsToggleCancellationReason, DomDetailsToggleIntention,
    DomDetailsToggleIntentionId, DomDetailsToggleTransfer,
};
use surgeist_dom::dialog::{
    DomCloseWatcherHistoryWindows, DomCloseWatcherId, DomCloseWatcherRequestPreparation,
    DomCloseWatcherRequestPublication, DomDialog, DomDialogCloseTaskInfo, DomDialogTask,
    DomDialogTaskInfo, DomDialogTaskQueueCommit, DomDialogTaskQueuePlan,
    DomDialogTaskQueuePublication, DomDialogToggleTaskDisposition, DomDialogToggleTaskInfo,
    DomDialogToggleTracker, DomDialogWatcherPreparation, DomDialogWatcherPublication,
    DomSourceCloseWatcherInvocationId, DomSourceCloseWatcherRequest, DomSourceCloseWatcherStep,
    DomSourceDialogRequest,
};
use surgeist_dom::document::{
    DomDocumentClass, DomDocumentCreationCause, DomDocumentEnvironment, DomDocumentInterface,
    DomDocumentMetadata, DomNativeInputPlatform, DomOpaqueOrigin,
};
use surgeist_dom::edit::{
    AttachedAttribute, AttachedShadowRoot, CreatedNode, DomChange, DomChanges,
    DomChildListNotification, DomChildListObservationLimits, DomChildListObserverId,
    DomChildListRecord, DomDetachedNodeCreation, DomElementCreationName, DomElementCreationOptions,
    DomHistoryError, DomNodeOrString, DomNodeRetirementInbox, DomNodeRetirementLimits,
    DomNodeRetirementNotice, DomNodeRetirementWatch, DomProcessingInstructionAttributeEdit,
    DomSourceNodeOutputWatch, DomWatchedNode, NodeOperand, RemovedAttribute,
};
use surgeist_dom::event::{
    DomDispatchAssociation, DomDispatchCustody, DomDispatchCustodyLimits, DomEventParent,
    DomEventParentQuery, DomEventPayloadRetention, DomEventRecipient, DomEventRecipientLimits,
    DomEventRecipientTarget, DomEventRegistryAssociation, DomEventRegistryPreparation,
    DomSourceAdvance, DomSourceEventClockFailure, DomSourceEventClockMode,
    DomSourceEventDispatchGuard, DomSourceEventDispatchPermit, DomSourceEventExecutor,
    DomSourceEventFiring, DomSourceEventFiringLoan, DomSourceEventId, DomSourceEventKind,
    DomSourceEventOutcome, DomSourceEventPayloadRetention, DomSourceEventPublication,
    DomSourceEventRequest, DomSourceEventRetention, DomSourceEventTime, DomSourceToggleEvent,
};
use surgeist_dom::focus::{
    DomDocumentAutofocus, DomFocusObservation, DomFocusPreparation, DomFocusPublication,
    DomFocusRequest, DomFocusTarget, DomFocusedAreaDesignation, DomSourceAutofocusInsertionRequest,
    DomSourceDialogFocusingRequest, DomSourceFocusDelegateRequest, DomSourceFocusDelegateStep,
};
use surgeist_dom::form::select::{
    DomOption, DomOptionState, DomSelect, DomSelectCompletionStart, DomSelectState, DomSelectTask,
    DomSelectTaskCancellationReason, DomSelectTaskCompletion, DomSelectTaskOutcome,
    DomSelectedcontent,
};
use surgeist_dom::form::{
    DomButton, DomButtonState, DomFieldset, DomFieldsetState, DomForm, DomInput, DomInputState,
    DomOutput, DomOutputState, DomTextarea, DomTextareaReadLimits, DomTextareaState,
};
use surgeist_dom::fullscreen::{
    DomDocumentTopLayer, DomFullscreenEvent, DomFullscreenEventBatch, DomFullscreenEventKind,
    DomFullscreenExitFact, DomFullscreenExitHandoffCommit, DomFullscreenExitId,
    DomFullscreenExitJob, DomFullscreenExitReturn, DomFullscreenExitStatus, DomFullscreenExitStep,
    DomFullscreenModelStep, DomSourceFullscreenExitHandoffRequest, DomTopLayerUaRule,
};
use surgeist_dom::hosted::child::{
    DomHostedChildPreparation, DomHostedChildRemovalHandoff, DomHostedChildRemovalPublication,
    DomHostedChildRemovalRequest, DomHostedChildRequest, DomHostedChildStep, DomHostedEmbedderKind,
    DomHostedEmbedderPolicyInputs, DomHostedPreparedChild,
};
use surgeist_dom::hosted::destruction::{
    DomHostedDestructionInvocation, DomHostedDestructionInvocationRequest,
    DomHostedDestructionLease, DomHostedDestructionPublication, DomHostedDestructionRequest,
    DomHostedDestructionStep, DomHostedDocumentDestructionProof,
};
use surgeist_dom::hosted::history::{
    DomHostedChildHistoryPublication, DomHostedChildHistoryRequest, DomHostedEntry,
    DomHostedEntryId, DomHostedEntryInitialization, DomHostedEntryRequest,
    DomHostedHistoryJobReceipt, DomHostedHistoryJobRequest, DomHostedHistoryStep,
    DomHostedNavigable, DomHostedQueuedHistory,
};
use surgeist_dom::hosted::sheet::{
    DomHostedSheetConstruction, DomHostedSheetConstructionError,
    DomHostedSheetConstructionPublished, DomHostedStyleRootWithdrawal,
    DomHostedStyleRootWithdrawalPreparation, DomHostedStyleRootWithdrawalPublished,
};
use surgeist_dom::hosted::{
    DomHostedBrowsingContextId, DomHostedCreation, DomHostedCreationLease,
    DomHostedCreationPublication, DomHostedCreationStep, DomHostedError, DomHostedExecutionRequest,
    DomHostedFinishLoadingRequest, DomHostedGlobalId, DomHostedHostId, DomHostedLimits,
    DomHostedMakeActiveRequest, DomHostedMoment, DomHostedNavigableId, DomHostedOperationId,
    DomHostedPolicyPublication, DomHostedPolicyRequest, DomHostedPreparedTopLevel,
    DomHostedRecordLimits, DomHostedResource, DomHostedRuntimeRetention, DomHostedSource,
    DomHostedSourceId, DomHostedSourceRevision, DomHostedTopLevelPreparation,
    DomInitialPermissionsPolicy, DomInitialPermissionsPolicyProfile, DomPermissionsPolicyValue,
};
use surgeist_dom::identity::{
    DocumentId, DomAttrId, DomDocumentStamp, DomOwnerId, DomRevision, DomStamp, NodeId,
};
use surgeist_dom::installation::meta::{
    DomParsedHtmlMetaCompletion, DomParsedHtmlMetaError, DomParsedHtmlMetaOccurrence,
    DomParsedHtmlMetaOccurrenceCompletion, DomParsedHtmlMetaOccurrenceError,
    DomParsedHtmlMetaOccurrencePreparation, DomParsedHtmlMetaOccurrencePublished,
    DomParsedHtmlMetaOccurrenceRetirement, DomParsedHtmlMetaPreparation,
    DomParsedHtmlMetaPublished, DomParsedHtmlMetaRetirement, DomParsedHtmlMetaRuntimeHold,
};
use surgeist_dom::installation::style::{
    DomParsedHtmlContributionCompletion, DomParsedHtmlDispatchCursor, DomParsedHtmlDispatchEntry,
    DomParsedHtmlExecutionState, DomParsedHtmlFrontierError, DomParsedHtmlStyleCreateSheetRequest,
    DomParsedHtmlStyleDeliveryPublication, DomParsedHtmlStyleError,
    DomParsedHtmlStyleInitialPolicyRequest, DomParsedHtmlStyleInlinePolicyObservation,
    DomParsedHtmlStyleInlinePolicyRequest, DomParsedHtmlStyleNotificationCompletion,
    DomParsedHtmlStyleNotificationContinuation, DomParsedHtmlStyleNotificationDisposition,
    DomParsedHtmlStyleNotificationError, DomParsedHtmlStyleNotificationHandoff,
    DomParsedHtmlStyleNotificationHandoffStatus, DomParsedHtmlStyleNotificationPreparation,
    DomParsedHtmlStyleNotificationPublished, DomParsedHtmlStyleNotificationRequirement,
    DomParsedHtmlStyleNotificationUnavailable, DomParsedHtmlStylePhase,
    DomParsedHtmlStylePrefixView, DomParsedHtmlStyleQualification,
    DomParsedHtmlStyleRenderingCompletion, DomParsedHtmlStyleRenderingProgress,
    DomParsedHtmlStyleRenderingRequirement, DomParsedHtmlStyleResources,
    DomParsedHtmlStyleRuntimeHold, DomParsedHtmlStyleSheetCompletion, DomParsedHtmlStyleSheetError,
    DomParsedHtmlStyleSheetInput, DomParsedHtmlStyleSheetPreparation,
    DomParsedHtmlStyleSheetPublished, DomParsedHtmlStyleSheetRetirement,
    DomParsedHtmlStyleSubjectCompletion, DomParsedHtmlStyleSubjectError,
    DomParsedHtmlStyleSuccessorCompletion, DomParsedHtmlStyleSuccessorError,
    DomParsedHtmlStyleSuccessorPreparation, DomParsedHtmlStyleSuccessorPublished,
    DomParsedHtmlStyleSuccessorRemoval, DomParsedHtmlStyleSuccessorRetirement,
    DomParsedHtmlSynchronousEnd, DomParsedHtmlSynchronousEndView, DomParsedHtmlTerminalError,
};
use surgeist_dom::installation::{
    DomNativeHtmlAssociation, DomNativeHtmlBinding, DomNativeHtmlInstallError,
    DomNativeHtmlInstallLimits, DomNativeHtmlInstallResource, DomNativeHtmlInstallTarget,
    DomNativeHtmlInstallation, DomParsedDocumentEnvironment,
    DomParsedHtmlAssociationAdmissionError, DomParsedHtmlAssociationDocument,
    DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits, DomParsedHtmlAssociationReadiness,
    DomParsedHtmlAssociationRuntimeUsage, DomParsedHtmlBinding, DomParsedHtmlCapturedBinding,
    DomParsedHtmlHistoricalDocument, DomParsedHtmlInstallError, DomParsedHtmlInstallLimits,
    DomParsedHtmlInstallResource, DomParsedHtmlInstallation, DomParsedHtmlPendingAssociations,
    DomSourceNativeHtmlInstallLimits,
};
use surgeist_dom::limits::{DomLimits, DomNameLimits, DomResource, DomUsage};
use surgeist_dom::meta::{
    DomDocumentCspState, DomDocumentPolicyContainer, DomMetaAtomicError, DomMetaAtomicLimits,
    DomMetaColorSchemeState, DomMetaDocumentState, DomMetaError, DomMetaInlineBatch,
    DomMetaInlineResolver, DomMetaOrdinaryCommit, DomMetaOrdinaryPreparation,
    DomMetaOrdinaryPublished, DomMetaPreferredOccurrence, DomMetaRetirement, DomMetaRootCommit,
    DomMetaRuntimePrefix, DomMetaRuntimeUsage, DomMetaSourceCommit, DomMetaSourcePreparation,
    DomMetaSourcePublished, DomMetaSourceRuntimeHold, DomMetaSourceStep, DomMetaSubject,
    DomMetaThemeColorState, DomSourceMetaCspError, DomSourceMetaCspProcessorFailure,
    DomSourceMetaCspRequest,
};
use surgeist_dom::range::{
    CreatedRange, DomActiveRangeState, DomDesignModeRangeDisposition, DomRange, DomRangeBoundary,
    DomRangeId, DomRangeOperand, DomSourceDesignModeRequest, DomSourceDesignModeStep,
};
use surgeist_dom::script::{
    DomCreationRealmFallback, DomDocumentConstructorContext, DomDocumentConstructorError,
    DomDocumentScriptBinding, DomDocumentScriptRegistration, DomDocumentScriptingFacts,
    DomDocumentScriptingObservation, DomHtmlScript, DomSourceScriptRequest,
    DomSourceScriptSettings, DomSourceScriptStep, DomSvgScript, DomSvgScriptOrigin,
};
use surgeist_dom::shadow::{
    DomFlatRelation, DomFlatTree, DomPreparedSlotAssignment, DomShadowRoot, DomShadowRootInit,
    DomShadowRootState, DomSlot, DomSlotNodes, DomSlotNotificationTarget, DomSlotNotifications,
    DomSlotQueryLimits, DomSlotSignalCancellationReason, DomTemplate, DomTemplateDocumentRole,
    DomTemplatePatch,
};
use surgeist_dom::source::attribute::{
    DomSourceAttributeAttachmentOutput, DomSourceAttributeCallbackScope,
    DomSourceAttributePayloadRetention, DomSourceAttributePolicyRead, DomSourceAttributeSink,
    DomSourceAttributeVerificationOutcome, DomSourceAttributeVerificationRequest,
};
use surgeist_dom::source::{
    DomSourceCleanupCause, DomSourceCompletion, DomSourceEdit, DomSourceExecutionEntryScope,
    DomSourceFailure, DomSourceInput, DomSourceInvariantFailure, DomSourceInvariantReason,
    DomSourceLease, DomSourceLimits, DomSourceOperationId, DomSourceOutput, DomSourcePhase,
    DomSourceProgress, DomSourceRequest, DomSourceStatus, DomSourceStoppedAction,
};
use surgeist_dom::style::inline::{
    DomInlineStyleBatchContext, DomInlineStyleCapture, DomInlineStyleChange, DomInlineStyleCommit,
    DomInlineStyleCompletion, DomInlineStyleError, DomInlineStyleFeedbackLimits,
    DomInlineStyleInputs, DomInlineStyleLimits, DomInlineStyleMutation,
    DomInlineStyleMutationLimits, DomInlineStyleOperation, DomInlineStyleOutcome,
    DomInlineStyleProfile, DomInlineStyleProperty, DomInlineStyleRequestLimits,
    DomInlineStyleReturn, DomInlineStyleSourceContext, DomInlineStyleSourceProgress,
    DomInlineStyleSourcePublication, DomInlineStyleTerminal,
};
use surgeist_dom::style::{
    DomStyle, DomStyleAction, DomStyleAssociationId, DomStyleCspBudget, DomStyleCspRetention,
    DomStyleCssomOwnerBinding, DomStyleCssomPublicationRejection, DomStyleCssomRoot,
    DomStyleCssomRootError, DomStyleInlineCspObservation, DomStyleNonceContent,
    DomStyleNotificationCompletion, DomStyleNotificationDelivery, DomStyleNotificationOrigin,
    DomStyleNotificationToken, DomStyleRequest, DomStyleRootKind, DomStyleSheet,
    DomStyleSheetAvailability, DomStyleSheetPlacement, DomStyleSource, DomStyleState,
};
use surgeist_dom::task::{
    DomElementTaskAssociation, DomElementTaskId, DomElementTaskRetention, DomElementTaskSource,
    DomElementTaskSourceKind, DomNativeCheckpoint, DomNativeCheckpointLimits, DomNativeTaskId,
};
use surgeist_dom::tree::{
    DomAttr, DomAttribute, DomAttributeAttachment, DomDocument, DomDocumentPosition,
    DomDocumentType, DomElement, DomNameAtom, DomNamedNodeMap, DomNamespaceUri, DomNode,
    DomNodeFilterDecision, DomNodeIterator, DomNodeIteratorFilter, DomNodeIteratorFilterRequest,
    DomNodeIteratorId, DomNodeIteratorMatchRequest, DomNodeIteratorProgress, DomNodeKind,
    DomProcessingInstruction, DomQualifiedName, DomSiblingDirection, DomTagName, DomTreeOrder,
    DomTreeScope, DomTreeWalker, DomTreeWalkerDirection, DomTreeWalkerFilter,
    DomTreeWalkerFilterRequest, DomTreeWalkerId, DomTreeWalkerMatchRequest, DomTreeWalkerProgress,
};
use surgeist_dom::url::{
    DomBase, DomBaseFactsError, DomBaseResponse, DomEncodingUrlFactsError, DomFrozenBase,
    DomHyperlink, DomImageUpdate, DomImageUpdateCancellationReason, DomImageUpdateId, DomImg,
    DomLink, DomSourceUrlObservation, DomSourceUrlOrigin, DomSourceUrlRequest, DomSourceUrlStage,
};
use surgeist_dom::view_transition::{
    CreatedViewTransition, CreatedViewTransitionRecord, DomDocumentViewTransitions,
    DomPreparedViewTransitionRules, DomPreparedViewTransitionSheet, DomViewTransition,
    DomViewTransitionBranches, DomViewTransitionCaptureLimits, DomViewTransitionCssomError,
    DomViewTransitionId, DomViewTransitionImageRules, DomViewTransitionPseudo,
    DomViewTransitionPseudoChildren, DomViewTransitionPseudoId, DomViewTransitionPseudoKind,
    DomViewTransitionRecord, DomViewTransitionRecordId, DomViewTransitionRuleAssociations,
    DomViewTransitionRules, DomViewTransitionSheet, DomViewTransitionSubject,
    ViewTransitionOperand, ViewTransitionRecordOperand,
};
use surgeist_dom::{Dom, DomBatch, DomCommit, DomError, DomSnapshot};

#[test]
fn every_public_item_resolves_at_its_module_path() {
    // 531 items in 33 modules and the crate root;
    // the imports above are checked at compile time.
}
