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
mod resource;
mod snapshot;

#[cfg(test)]
mod tests;

pub use command::{AppCommand, CommandDescriptor, CommandName};
pub use coord::{AppScope, ScopePathSegment};
pub use descriptor::{
    App, AppDescriptor, AppManifest, ResourceDescriptor, RootDescriptor, StartupWindow,
    TaskDescriptor, WindowDescriptor, WindowDescriptorId,
};
pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticLog, DiagnosticSeverity, QueueDiagnostic,
};
pub use effect::{
    AppEffect, AppEffectPayload, DiagnosticEffect, EffectBatch, EffectKindId,
    InvalidateResourceEffect, LoadResourceEffect, PersistEffect, RedrawTarget, RequestRedrawEffect,
};
pub use event::{AppEvent, EventDescriptor, EventName};
pub use ids::{
    AppId, CalcId, CorrelationId, CustomScopeId, ExpressionId, ResourceId, RootId, ServiceId,
    SurfaceId, TaskAttemptId, TaskId, TaskKey, TaskName, ValueExprId,
};
pub use input::AppInput;
pub use provenance::{
    InputOrigin, InputProvenance, InputSourceId, ServiceProvenance, SurfaceProvenance,
    TaskProvenance,
};
pub use reducer::{Reducer, ReducerResult};
pub use resource::{
    FailureVisibility, Freshness, ResourceSnapshot, ResourceState, ResourceStateReadyTransition,
    ResourceStatus,
};
pub use snapshot::{
    AppSnapshot, SnapshotBinding, SnapshotBindingId, SnapshotSourceType, StateVersion,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AppLoop;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Runtime<State = ()> {
    state: State,
}

impl<State> Runtime<State> {
    #[must_use]
    pub fn state(&self) -> &State {
        &self.state
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct UiSurface;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct WindowRoot;
