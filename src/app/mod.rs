//! App runtime and authoring DSL boundary for Surgeist.
//!
//! This module coordinates deterministic app state, retained UI surfaces,
//! resources, tasks, services, native wakeups, and declared effects. Native
//! window mechanics stay in `surgeist::window`.

mod command;
mod coord;
mod descriptor;
mod event;
mod ids;
mod snapshot;

pub use command::{AppCommand, CommandDescriptor, CommandName};
pub use coord::{AppScope, ScopePathSegment};
pub use descriptor::{
    App, AppDescriptor, AppManifest, ResourceDescriptor, RootDescriptor, StartupWindow,
    TaskDescriptor, WindowDescriptor, WindowDescriptorId,
};
pub use event::{AppEvent, EventDescriptor, EventName};
pub use ids::{
    AppId, CalcId, CorrelationId, CustomScopeId, ExpressionId, ResourceId, RootId, ServiceId,
    SurfaceId, TaskAttemptId, TaskId, TaskKey, TaskName, ValueExprId,
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

#[derive(Clone, Debug)]
pub struct AppEffect {
    kind: EffectKindId,
}

impl AppEffect {
    #[must_use]
    pub fn kind(&self) -> &EffectKindId {
        &self.kind
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EffectKindId(String);

impl EffectKindId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
