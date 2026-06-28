use std::borrow::Cow;

use super::{AppScope, Diagnostic, ResourceId, SurfaceId};

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

static REQUEST_REDRAW_EFFECT_KIND: EffectKindId = EffectKindId::REQUEST_REDRAW;
static PERSIST_EFFECT_KIND: EffectKindId = EffectKindId::PERSIST;
static EMIT_DIAGNOSTIC_EFFECT_KIND: EffectKindId = EffectKindId::EMIT_DIAGNOSTIC;
static LOAD_RESOURCE_EFFECT_KIND: EffectKindId = EffectKindId::LOAD_RESOURCE;
static INVALIDATE_RESOURCE_EFFECT_KIND: EffectKindId = EffectKindId::INVALIDATE_RESOURCE;

impl EffectKindId {
    pub const REQUEST_REDRAW: Self = Self::from_static("runtime.request_redraw");
    pub const PERSIST: Self = Self::from_static("runtime.persist");
    pub const EMIT_DIAGNOSTIC: Self = Self::from_static("runtime.emit_diagnostic");
    pub const LOAD_RESOURCE: Self = Self::from_static("runtime.load_resource");
    pub const INVALIDATE_RESOURCE: Self = Self::from_static("runtime.invalidate_resource");
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

#[derive(Clone, Debug)]
pub struct AppEffect {
    payload: AppEffectPayload,
}

impl AppEffect {
    #[must_use]
    pub fn request_redraw(target: RedrawTarget) -> Self {
        Self {
            payload: AppEffectPayload::RequestRedraw(RequestRedrawEffect { target }),
        }
    }

    #[must_use]
    pub fn persist(key: impl Into<String>, scope: AppScope) -> Self {
        Self {
            payload: AppEffectPayload::Persist(PersistEffect {
                key: key.into(),
                scope,
            }),
        }
    }

    #[must_use]
    pub fn diagnostic(diagnostic: Diagnostic) -> Self {
        Self {
            payload: AppEffectPayload::Diagnostic(DiagnosticEffect {
                diagnostic: Box::new(diagnostic),
            }),
        }
    }

    #[must_use]
    pub fn load_resource(id: ResourceId, scope: AppScope) -> Self {
        Self {
            payload: AppEffectPayload::LoadResource(LoadResourceEffect { id, scope }),
        }
    }

    #[must_use]
    pub fn invalidate_resource(id: ResourceId, reason: impl Into<String>) -> Self {
        Self {
            payload: AppEffectPayload::InvalidateResource(InvalidateResourceEffect {
                id,
                reason: reason.into(),
            }),
        }
    }

    #[must_use]
    pub fn kind(&self) -> &EffectKindId {
        match &self.payload {
            AppEffectPayload::RequestRedraw(_) => &REQUEST_REDRAW_EFFECT_KIND,
            AppEffectPayload::Persist(_) => &PERSIST_EFFECT_KIND,
            AppEffectPayload::Diagnostic(_) => &EMIT_DIAGNOSTIC_EFFECT_KIND,
            AppEffectPayload::LoadResource(_) => &LOAD_RESOURCE_EFFECT_KIND,
            AppEffectPayload::InvalidateResource(_) => &INVALIDATE_RESOURCE_EFFECT_KIND,
        }
    }

    #[must_use]
    pub const fn payload(&self) -> &AppEffectPayload {
        &self.payload
    }
}

#[derive(Clone, Debug)]
pub enum AppEffectPayload {
    RequestRedraw(RequestRedrawEffect),
    Persist(PersistEffect),
    Diagnostic(DiagnosticEffect),
    LoadResource(LoadResourceEffect),
    InvalidateResource(InvalidateResourceEffect),
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
    diagnostic: Box<Diagnostic>,
}

impl DiagnosticEffect {
    #[must_use]
    pub fn diagnostic(&self) -> &Diagnostic {
        self.diagnostic.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LoadResourceEffect {
    id: ResourceId,
    scope: AppScope,
}

impl LoadResourceEffect {
    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    #[must_use]
    pub const fn scope(&self) -> &AppScope {
        &self.scope
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidateResourceEffect {
    id: ResourceId,
    reason: String,
}

impl InvalidateResourceEffect {
    #[must_use]
    pub fn id(&self) -> &ResourceId {
        &self.id
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
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
