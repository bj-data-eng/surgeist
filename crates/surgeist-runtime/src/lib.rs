#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! App runtime and authoring DSL boundary for Surgeist.
//!
//! This module coordinates deterministic app state, retained UI surfaces,
//! resources, tasks, services, native wakeups, and declared effects. Native
//! window mechanics stay with the host adapter.
//!
//! # Validated authoring names
//!
//! Command names, event names, and payload type names enter the public API through
//! fallible validation. Their text is preserved after validation:
//!
//! ```
//! use surgeist_runtime::{CommandName, EventName, PayloadTypeName};
//!
//! assert_eq!(CommandName::try_new("save")?.as_str(), "save");
//! assert_eq!(EventName::try_new("saved")?.as_str(), "saved");
//! assert_eq!(PayloadTypeName::try_new("Document")?.as_str(), "Document");
//! # Ok::<(), surgeist_runtime::NameError>(())
//! ```
//!
//! These types provide no unchecked `new` constructor:
//!
//! ```compile_fail
//! use surgeist_runtime::CommandName;
//! let _ = CommandName::new("save");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::EventName;
//! let _ = EventName::new("saved");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::PayloadTypeName;
//! let _ = PayloadTypeName::new("Document");
//! ```
//!
//! Commands and events take validated names:
//!
//! ```
//! use surgeist_runtime::{AppCommand, AppEvent, CommandName, EventName};
//!
//! let command = AppCommand::named(CommandName::try_new("save")?);
//! let event = AppEvent::named(EventName::try_new("saved")?);
//! assert_eq!(command.name().as_str(), "save");
//! assert_eq!(event.name().as_str(), "saved");
//! # Ok::<(), surgeist_runtime::NameError>(())
//! ```
//!
//! A raw string cannot bypass that validation:
//!
//! ```compile_fail
//! use surgeist_runtime::AppCommand;
//! let _ = AppCommand::named("save");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::AppEvent;
//! let _ = AppEvent::named("saved");
//! ```
//!
//! Descriptor construction also validates names and semantic payload types:
//!
//! ```
//! use surgeist_runtime::{
//!     CommandDescriptor, EventDescriptor, ResourceDescriptor, ResourceId,
//!     TaskDescriptor, TaskIntentName,
//! };
//!
//! let command = CommandDescriptor::try_new("save", "Document")?;
//! let event = EventDescriptor::try_new("saved", "Document")?;
//! let task = TaskDescriptor::try_new(TaskIntentName::new("load"), "Path")?;
//! let resource = ResourceDescriptor::try_new(ResourceId::new("document"), "Document")?;
//! assert_eq!(command.payload_type().as_str(), "Document");
//! assert_eq!(event.payload_type().as_str(), "Document");
//! assert_eq!(task.input_type().as_str(), "Path");
//! assert_eq!(resource.value_type().as_str(), "Document");
//! # Ok::<(), surgeist_runtime::NameError>(())
//! ```
//!
//! The former unchecked descriptor constructors are unavailable:
//!
//! ```compile_fail
//! use surgeist_runtime::CommandDescriptor;
//! let _ = CommandDescriptor::new("save", "Document");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::EventDescriptor;
//! let _ = EventDescriptor::new("saved", "Document");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::{TaskDescriptor, TaskIntentName};
//! let name = TaskIntentName::new("load");
//! let _ = TaskDescriptor::new(name, "Path");
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::{ResourceDescriptor, ResourceId};
//! let id = ResourceId::new("document");
//! let _ = ResourceDescriptor::new(id, "Document");
//! ```
//!
//! # Host and test boundaries
//!
//! Hosts implement the public [`WakeBridge`] contract. Concrete retained-to-runtime
//! adapters belong to the composing application, so this crate has no public
//! `bridge` module:
//!
//! ```
//! use surgeist_runtime::{AppProxy, QueuePolicy, WakeBridge, WakeError};
//!
//! struct HostWake;
//! impl WakeBridge for HostWake {
//!     fn wake(&self) -> Result<(), WakeError> {
//!         Ok(())
//!     }
//! }
//! let _proxy = AppProxy::<()>::new(HostWake, QueuePolicy::default());
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::bridge;
//! ```
//!
//! External callers can exercise a runtime through its public reducer and input
//! contracts:
//!
//! ```
//! use surgeist_runtime::{
//!     AppInput, InputProvenance, Reducer, ReducerCommit, ReducerResult,
//!     Runtime, RuntimeBudget, UiInput,
//! };
//!
//! struct KeepState;
//! impl Reducer<u8, ()> for KeepState {
//!     fn reduce(&mut self, _state: &u8, _input: &AppInput<()>) -> ReducerResult<u8> {
//!         ReducerResult::unchanged(ReducerCommit::new())
//!     }
//! }
//! let mut runtime = Runtime::new(7, KeepState);
//! runtime.enqueue_ui(UiInput::new((), InputProvenance::system()).unwrap()).unwrap();
//! runtime.drain_once(RuntimeBudget::default()).unwrap();
//! assert_eq!(runtime.state(), &7);
//! ```
//!
//! The crate's private test fixtures are unavailable to those callers:
//!
//! ```compile_fail
//! use surgeist_runtime::testing;
//! ```
//!
//! ```compile_fail
//! use surgeist_runtime::PrototypeApp;
//! ```
//!
//! These examples check the named public contracts. The root `surgeist`
//! repository owns the complete generated API audit.

mod command;
mod coord;
mod descriptor;
mod diagnostic;
mod effect;
mod event;
mod ids;
mod input;
mod loop_;
mod provenance;
mod proxy;
mod reducer;
mod resource;
mod runtime;
mod service;
mod snapshot;
mod surface;
mod task;

#[cfg(test)]
mod testing;

#[cfg(test)]
mod tests;

pub use command::{AppCommand, CommandDescriptor, CommandName, NameError, PayloadTypeName};
pub use coord::{
    AppScope, CoordinationState, ScopePathSegment, Subscription, SubscriptionAggregate,
    SubscriptionChange, SubscriptionError, SubscriptionErrorCode, SubscriptionKey,
    SubscriptionPriority, SubscriptionTarget, SubscriptionTargetKindId,
};
pub use descriptor::{
    App, AppDescriptor, AppManifest, ManifestValidationError, ManifestValidationErrorCode,
    ManifestValidationIssue, ResourceDescriptor, RootDescriptor, StartupWindow, TaskDescriptor,
    ValidatedAppManifest, WindowDescriptor, WindowDescriptorId,
};
pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity, QueueDiagnostic,
};
pub use effect::{
    AppEffect, AppEffectPayload, CallServiceEffect, CancelTaskEffect, DiagnosticEffect,
    EffectBatch, EffectDisposition, EffectKindId, EffectOutcome, InvalidateResourceEffect,
    LoadResourceEffect, PersistEffect, RedrawTarget, ReprioritizeTaskEffect, RequestRedrawEffect,
    RuntimeIntent, ServiceDiagnosticEffect, StartServiceEffect, StartTaskEffect, StopServiceEffect,
};
pub use event::{AppEvent, EventDescriptor, EventName};
pub use ids::{
    AppId, CalcId, CorrelationError, CorrelationId, CustomScopeId, ElementId, ExpressionId,
    ResourceGeneration, ResourceId, ResourceOperationId, RootId, ServiceId, SurfaceGeneration,
    SurfaceId, SurfaceInvalidationGeneration, ValueExprId, VersionError, WindowId,
};
pub use input::AppInput;
pub use loop_::AppLoop;
pub use provenance::{
    Correlation, InputOrigin, InputProvenance, InputSourceId, ProvenanceError, ProvenanceErrorCode,
    ServiceProvenance, SurfaceProvenance, TaskProvenance,
};
pub use proxy::{
    AppProxy, AppProxyError, AppProxyErrorCode, ProxyDrainReport, ProxyInput, QueuePolicy,
    WakeBridge, WakeError,
};
pub use reducer::{Reducer, ReducerChange, ReducerCommit, ReducerFailure, ReducerResult};
pub use resource::{
    FailureVisibility, Freshness, ResourceOperation, ResourceSnapshot, ResourceState,
    ResourceStateError, ResourceStateErrorCode, ResourceStatus,
};
pub use runtime::{
    Runtime, RuntimeBudget, RuntimeDrainError, RuntimeDrainErrorCode, RuntimeDrainReport,
    RuntimeInputError, RuntimeLane, RuntimeQueueError, RuntimeQueueErrorCode, RuntimeQueuePolicy,
    ServiceInput, TaskInput, UiInput,
};
pub use service::{
    MailboxOverflow, MailboxPolicy, MailboxPushOutcome, ServiceCommandName, ServiceCommandPayload,
    ServiceMailbox, ServiceRegistration, ServiceRestart, ServiceShutdown, ServiceStartup,
    ServiceStatus,
};
pub use snapshot::{
    AppSnapshot, SnapshotBinding, SnapshotBindingId, SnapshotEntry, SnapshotError,
    SnapshotErrorCode, SnapshotSourceType, SnapshotValue, StateVersion,
};
pub use surface::{
    ElementPhase, ElementRegistration, SurfaceElementRef, SurfaceElements, SurfaceError,
    SurfaceErrorCode, SurfaceInvalidation, SurfaceInvalidationKind, SurfaceLifecycle,
    SurfaceMutation, SurfacePoint, SurfaceRef, SurfaceRenderAck, SurfaceRenderFrame,
    SurfaceRenderState, SurfaceRoot, SurfaceRoute, SurfaceRouteStep, SurfaceSize, UiSurface,
};
pub use task::{
    TaskIntentAttemptId, TaskIntentHandle, TaskIntentId, TaskIntentKey, TaskIntentName,
    TaskPriorityHint,
};

/// Returns the crate identity while the runtime API is being designed.
#[must_use]
pub const fn crate_name() -> &'static str {
    "surgeist-runtime"
}
