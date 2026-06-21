use std::{any::Any, borrow::Cow, sync::Arc};

use super::{AppScope, Diagnostic, SurfaceId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RedrawTarget {
    All,
    Surface(SurfaceId),
    Window(crate::window::Id),
}

impl RedrawTarget {
    #[must_use]
    pub const fn all() -> Self {
        Self::All
    }

    #[must_use]
    pub const fn surface(id: SurfaceId) -> Self {
        Self::Surface(id)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EffectKindId(Cow<'static, str>);

impl EffectKindId {
    pub const REQUEST_REDRAW: Self = Self::from_static("runtime.request_redraw");
    pub const PERSIST: Self = Self::from_static("runtime.persist");
    pub const EMIT_DIAGNOSTIC: Self = Self::from_static("runtime.emit_diagnostic");
    pub const START_TASK: Self = Self::from_static("runtime.start_task");
    pub const CANCEL_TASK: Self = Self::from_static("runtime.cancel_task");
    pub const START_SERVICE: Self = Self::from_static("runtime.start_service");
    pub const STOP_SERVICE: Self = Self::from_static("runtime.stop_service");
    pub const SCHEDULE_TIMER: Self = Self::from_static("runtime.schedule_timer");
    pub const WINDOW_COMMAND: Self = Self::from_static("runtime.window_command");

    #[must_use]
    pub fn new(value: impl Into<Cow<'static, str>>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

#[derive(Clone)]
pub struct EffectPayload {
    value: Arc<dyn Any + Send + Sync>,
}

impl std::fmt::Debug for EffectPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EffectPayload").finish_non_exhaustive()
    }
}

impl EffectPayload {
    #[must_use]
    pub fn new<T>(value: T) -> Self
    where
        T: Any + Send + Sync + 'static,
    {
        Self {
            value: Arc::new(value),
        }
    }

    #[must_use]
    pub fn downcast_ref<T>(&self) -> Option<&T>
    where
        T: Any + Send + Sync + 'static,
    {
        self.value.downcast_ref::<T>()
    }
}

#[derive(Clone, Debug)]
pub struct AppEffect {
    kind: EffectKindId,
    payload: EffectPayload,
}

impl AppEffect {
    #[must_use]
    pub fn new(kind: EffectKindId, payload: EffectPayload) -> Self {
        Self { kind, payload }
    }

    #[must_use]
    pub fn request_redraw(target: RedrawTarget) -> Self {
        Self::new(
            EffectKindId::REQUEST_REDRAW,
            EffectPayload::new(RequestRedrawEffect { target }),
        )
    }

    #[must_use]
    pub fn persist(key: impl Into<String>, scope: AppScope) -> Self {
        Self::new(
            EffectKindId::PERSIST,
            EffectPayload::new(PersistEffect {
                key: key.into(),
                scope,
            }),
        )
    }

    #[must_use]
    pub fn diagnostic(diagnostic: Diagnostic) -> Self {
        Self::new(
            EffectKindId::EMIT_DIAGNOSTIC,
            EffectPayload::new(DiagnosticEffect { diagnostic }),
        )
    }

    #[must_use]
    pub fn kind(&self) -> &EffectKindId {
        &self.kind
    }

    #[must_use]
    pub fn payload(&self) -> &EffectPayload {
        &self.payload
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RequestRedrawEffect {
    target: RedrawTarget,
}

impl RequestRedrawEffect {
    #[must_use]
    pub const fn target(&self) -> &RedrawTarget {
        &self.target
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PersistEffect {
    key: String,
    scope: AppScope,
}

impl PersistEffect {
    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub const fn scope(&self) -> &AppScope {
        &self.scope
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticEffect {
    diagnostic: Diagnostic,
}

impl DiagnosticEffect {
    #[must_use]
    pub const fn diagnostic(&self) -> &Diagnostic {
        &self.diagnostic
    }
}

#[derive(Clone, Debug, Default)]
pub struct EffectBatch {
    effects: Vec<AppEffect>,
}

impl EffectBatch {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn push(mut self, effect: AppEffect) -> Self {
        self.effects.push(effect);
        self
    }

    #[must_use]
    pub fn effects(&self) -> &[AppEffect] {
        &self.effects
    }
}
