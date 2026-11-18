//! Every public item of the runtime is reachable at exactly one module path.
//!
//! Each item is imported once by its module path; the root holds only `Runtime`, `DomSession` and
//! `crate_name`. A missing or misplaced export fails to compile. The feature section covers the
//! `document_scripting` surface, which the default audit does not show.
#![allow(unused_imports)]
#![forbid(unsafe_code)]

use surgeist_runtime::{
    DomSession, Runtime,
    coordination::{
        AppScope, CoordinationState, ScopePathSegment, Subscription, SubscriptionAggregate,
        SubscriptionChange, SubscriptionError, SubscriptionErrorCode, SubscriptionKey,
        SubscriptionPriority, SubscriptionTarget, SubscriptionTargetKindId,
    },
    crate_name,
    diagnostic::{Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity, QueueDiagnostic},
    drain::{AppLoop, RuntimeBudget, RuntimeDrainError, RuntimeDrainErrorCode, RuntimeDrainReport},
    effect::{
        AppEffect, AppEffectPayload, CallServiceEffect, CancelTaskEffect, DiagnosticEffect,
        EffectBatch, EffectDisposition, EffectKindId, EffectOutcome, InvalidateResourceEffect,
        LoadResourceEffect, PersistEffect, RedrawTarget, ReprioritizeTaskEffect,
        RequestRedrawEffect, RuntimeIntent, ServiceDiagnosticEffect, StartServiceEffect,
        StartTaskEffect, StopServiceEffect,
    },
    element_task::{
        DomDetailsTask, DomElementTask, DomElementTaskEventContext, DomElementTaskProgress,
        DomElementTaskRunLimits, DomElementTaskRuntime, DomElementTaskRuntimeError,
        DomElementTaskRuntimeLimits,
    },
    event::{
        DomEventCallback, DomEventCallbackError, DomEventComposedPath, DomEventConstructionError,
        DomEventCreationObservation, DomEventDispatch, DomEventDispatchError,
        DomEventDispatchLimits, DomEventDispatchOutcome, DomEventDispatchReceipt,
        DomEventDispatcher, DomEventDom, DomEventHostError, DomEventHostInvocation, DomEventInit,
        DomEventListenerFailure, DomEventListenerFailureKind, DomEventListenerOptions,
        DomEventPhase, DomEventRegistrationId, DomEventRegistryLimits, DomEventResource,
        DomEventStorageLimits, DomEventTarget, DomEventTimestampUnavailable, DomEventType,
        DomStandaloneEventTarget, DomToggleEvent, NativeDomEvent,
    },
    host_capture::{
        RuntimeDomHost, RuntimeDomHostError, RuntimeDomHostPublication, SurfaceDomBindingLimits,
    },
    hosted::{
        DomHostedDestructionQueued, DomHostedHistoryEntry, DomHostedRuntimeError,
        DomHostedRuntimeLimits, DomHostedSettings, DomHostedTaskKind, DomHostedTaskProgress,
        DomHostedTaskWait, DomHostedTopLevel, DomSourceExecutionScope, HostedRuntime,
    },
    identity::{
        AppId, CalcId, CorrelationError, CorrelationId, CustomScopeId, ElementId, ExpressionId,
        ResourceGeneration, ResourceId, ResourceOperationId, RootId, ServiceId, StateVersion,
        SurfaceGeneration, SurfaceId, SurfaceInvalidationGeneration, ValueExprId, VersionError,
        WindowId,
    },
    input::{
        AppInput, RuntimeInputError, RuntimeLane, RuntimeQueueError, RuntimeQueueErrorCode,
        RuntimeQueuePolicy, ServiceInput, TaskInput, UiInput,
    },
    manifest::{
        App, AppCommand, AppDescriptor, AppEvent, AppManifest, CommandDescriptor, CommandName,
        EventDescriptor, EventName, ManifestValidationError, ManifestValidationErrorCode,
        ManifestValidationIssue, NameError, PayloadTypeName, ResourceDescriptor, RootDescriptor,
        StartupWindow, TaskDescriptor, ValidatedAppManifest, WindowDescriptor, WindowDescriptorId,
    },
    provenance::{
        Correlation, InputOrigin, InputProvenance, InputSourceId, ProvenanceError,
        ProvenanceErrorCode, ServiceProvenance, SurfaceProvenance, TaskProvenance,
    },
    proxy::{
        AppProxy, AppProxyError, AppProxyErrorCode, ProxyDrainReport, ProxyInput, QueuePolicy,
        WakeBridge, WakeError,
    },
    reducer::{Reducer, ReducerChange, ReducerCommit, ReducerFailure, ReducerResult},
    resource::{
        FailureVisibility, Freshness, ResourceOperation, ResourceSnapshot, ResourceState,
        ResourceStateError, ResourceStateErrorCode, ResourceStatus,
    },
    service::{
        MailboxOverflow, MailboxPolicy, MailboxPushOutcome, ServiceCommandName,
        ServiceCommandPayload, ServiceMailbox, ServiceRegistration, ServiceRestart,
        ServiceShutdown, ServiceStartup, ServiceStatus,
    },
    session::{
        DomSessionCreateError, DomSessionCreateFailure, DomSessionLimits, DomShutdownOutcome,
        DomShutdownReason,
    },
    snapshot::{
        AppSnapshot, SnapshotBinding, SnapshotBindingId, SnapshotEntry, SnapshotError,
        SnapshotErrorCode, SnapshotSourceType, SnapshotValue,
    },
    source::{
        DomSelectCompletionProgress, DomSourceDriveError, DomSourceDriveProgress,
        DomSourceEventResource, DomSourceEventRuntimeError, DomSourceEventRuntimeLimits,
        DomSourceEventServiceLimits, DomSourceEventServiceProgress, DomSourceInlineStyleEdit,
        DomSourceRuntime, DomSourceRuntimeError, DomSourceServices, DomSourceUnavailable,
        DomSourceUrlObservationResult, DomSourceUrlService,
    },
    style::parsed::{
        DomStyleParsedAdmissionError, DomStyleParsedFrontierError, DomStyleParsedInstallError,
        DomStyleParsedMetaError, DomStyleParsedMetaOccurrenceError,
        DomStyleParsedMetaOccurrenceOutcome, DomStyleParsedMetaOutcome,
        DomStyleParsedMetaPreference, DomStyleParsedNotificationError,
        DomStyleParsedNotificationProgress, DomStyleParsedPrefixError, DomStyleParsedSheetError,
        DomStyleParsedSheetOutcome, DomStyleParsedSubjectError, DomStyleParsedSuccessorError,
        DomStyleParsedTerminalError,
    },
    style::{
        DomConstructedSheet, DomConstructedSheetError, DomConstructedSheetOwnerInit,
        DomStyleDisabledObservation, DomStyleEventContext, DomStyleMetaError,
        DomStyleMetaOrdinaryOutcome, DomStyleMetaPreference, DomStyleMetaSourceOutcome,
        DomStyleNetworkingProgress, DomStyleNetworkingTaskId, DomStyleProcessedSheet,
        DomStyleRootWithdrawal, DomStyleRootWithdrawalError, DomStyleRuntime, DomStyleRuntimeError,
        DomStyleRuntimeLimits, DomStyleSheetSetCommit, DomStyleSheetSetObservation,
        DomStyleUnavailable, DomStyleWithdrawnSheetSetHistory,
    },
    surface::{
        ElementPhase, ElementRegistration, SurfaceElementRef, SurfaceElements, SurfaceError,
        SurfaceErrorCode, SurfaceInvalidation, SurfaceInvalidationKind, SurfaceLifecycle,
        SurfaceMutation, SurfacePoint, SurfaceRef, SurfaceRenderAck, SurfaceRenderFrame,
        SurfaceRenderState, SurfaceRoot, SurfaceRoute, SurfaceRouteStep, SurfaceSize, UiSurface,
    },
    task::{
        TaskIntentAttemptId, TaskIntentHandle, TaskIntentId, TaskIntentKey, TaskIntentName,
        TaskPriorityHint,
    },
    trusted_types_csp::{
        AdmittedPolicyCreation, AttributeMismatch, AttributePolicies, AttributePolicyError,
        Decision, Directive, Policy, PolicyCreation, PolicyCreationError, PolicyDelivery,
        PolicyFactory, PolicyListCapacity, PolicyListError, PolicyProcessor, ReportError,
        Violation, ViolationReport, requires_script_values,
    },
};

#[cfg(feature = "document-scripting")]
use surgeist_runtime::document_scripting::{
    DocumentCspViolationProgress, DocumentGuestAccounting, DocumentGuestAccountingReport,
    DocumentGuestConstructorException, DocumentGuestCustomElementCallback,
    DocumentGuestCustomElementCallbackException, DocumentGuestEvent,
    DocumentGuestEventDispatchFailure, DocumentGuestEventError, DocumentGuestEventLimits,
    DocumentGuestEventReport, DocumentGuestEventTarget, DocumentGuestEventView,
    DocumentGuestListenerException, DocumentGuestResource, DocumentGuestWork, DocumentScriptError,
    DocumentScriptEventError, DocumentScriptEventProgress, DocumentScriptEventReport,
    DocumentScriptGuestResult, DocumentScriptHooks, DocumentScriptHost, DocumentScriptNativeError,
    DocumentScriptPermit, EnableDocumentGuestEventsFailure, EnableDocumentScriptFailure,
    PrepareDocumentScriptFailure, PreparedDocumentScriptHost, SelectedDocumentScriptListener,
    StoppedDocumentScriptHost, WindowTimerLimits, WindowTimerProgress, media_scripting_for_status,
};

#[test]
fn the_root_holds_the_owners_and_the_crate_identity() {
    let _: Option<Runtime> = None;
    let _: Option<DomSession> = None;
    assert_eq!(crate_name(), "surgeist-runtime");
}
