//! App runtime and authoring DSL boundary for Surgeist.
//!
//! This module coordinates deterministic app state, retained UI surfaces,
//! resources, tasks, services, native wakeups, and declared effects. Native
//! window mechanics stay in `surgeist::window`.

mod command;
mod coord;
mod descriptor;
mod diagnostic;
mod effect;
mod event;
mod ids;
mod input;
mod provenance;
mod reducer;
mod snapshot;

#[cfg(test)]
mod tests;

pub use command::{AppCommand, CommandDescriptor, CommandName};
pub use coord::{AppScope, ScopePathSegment};
pub use descriptor::{
    App, AppDescriptor, AppManifest, ResourceDescriptor, RootDescriptor, StartupWindow,
    TaskDescriptor, WindowDescriptor,
};
pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity, QueueDiagnostic,
};
pub use effect::{
    AppEffect, DiagnosticEffect, EffectBatch, EffectKindId, EffectPayload, PersistEffect,
    RedrawTarget, RequestRedrawEffect,
};
pub use event::{AppEvent, EventDescriptor, EventName};
pub use ids::{
    AppId, CorrelationId, CustomScopeId, ResourceId, RootId, ServiceId, SurfaceId, TaskAttemptId,
    TaskId, TaskKey, TaskName,
};
pub use input::AppInput;
pub use provenance::{InputProvenance, InputSourceId};
pub use reducer::{Reducer, ReducerResult};
pub use snapshot::{AppSnapshot, SnapshotBinding, StateVersion};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppLoop;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Runtime<State = ()> {
    state: State,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiSurface;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowRoot;
