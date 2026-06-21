use std::borrow::Cow;

use super::{CorrelationId, ServiceId, SurfaceId, TaskAttemptId, TaskId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct InputSourceId(Cow<'static, str>);

impl InputSourceId {
    pub const UI: Self = Self::from_static("ui");
    pub const RETAINED: Self = Self::from_static("retained");
    pub const TASK: Self = Self::from_static("task");
    pub const SERVICE: Self = Self::from_static("service");
    pub const WINDOW: Self = Self::from_static("window");
    pub const SYSTEM: Self = Self::from_static("system");

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InputProvenance {
    source: InputSourceId,
    surface_id: Option<SurfaceId>,
    task_id: Option<TaskId>,
    task_attempt_id: Option<TaskAttemptId>,
    service_id: Option<ServiceId>,
    correlation_id: CorrelationId,
    parent_correlation_id: Option<CorrelationId>,
    sequence: Option<u64>,
}

impl InputProvenance {
    #[must_use]
    pub fn system() -> Self {
        Self::new(InputSourceId::SYSTEM)
    }

    #[must_use]
    pub fn ui(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::UI).with_surface(surface_id)
    }

    #[must_use]
    pub fn retained(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::RETAINED).with_surface(surface_id)
    }

    #[must_use]
    pub fn task(task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        Self::new(InputSourceId::TASK).with_task(task_id, attempt_id)
    }

    #[must_use]
    pub fn service(service_id: ServiceId) -> Self {
        let mut value = Self::new(InputSourceId::SERVICE);
        value.service_id = Some(service_id);
        value
    }

    #[must_use]
    pub fn window(surface_id: SurfaceId) -> Self {
        Self::new(InputSourceId::WINDOW).with_surface(surface_id)
    }

    #[must_use]
    pub fn with_surface(mut self, id: SurfaceId) -> Self {
        self.surface_id = Some(id);
        self
    }

    #[must_use]
    pub fn with_correlation(mut self, id: CorrelationId) -> Self {
        self.correlation_id = id;
        self
    }

    #[must_use]
    pub fn with_parent(mut self, id: CorrelationId) -> Self {
        self.parent_correlation_id = Some(id);
        self
    }

    #[must_use]
    pub fn with_sequence(mut self, sequence: u64) -> Self {
        self.sequence = Some(sequence);
        self
    }

    #[must_use]
    pub const fn source(&self) -> &InputSourceId {
        &self.source
    }

    #[must_use]
    pub const fn surface_id(&self) -> Option<SurfaceId> {
        self.surface_id
    }

    #[must_use]
    pub const fn task_id(&self) -> Option<TaskId> {
        self.task_id
    }

    #[must_use]
    pub const fn task_attempt_id(&self) -> Option<TaskAttemptId> {
        self.task_attempt_id
    }

    #[must_use]
    pub fn service_id(&self) -> Option<&ServiceId> {
        self.service_id.as_ref()
    }

    #[must_use]
    pub const fn correlation_id(&self) -> CorrelationId {
        self.correlation_id
    }

    #[must_use]
    pub const fn parent_correlation_id(&self) -> Option<CorrelationId> {
        self.parent_correlation_id
    }

    #[must_use]
    pub const fn sequence(&self) -> Option<u64> {
        self.sequence
    }

    #[must_use]
    pub fn new(source: InputSourceId) -> Self {
        Self {
            source,
            surface_id: None,
            task_id: None,
            task_attempt_id: None,
            service_id: None,
            correlation_id: CorrelationId::from_u64(0),
            parent_correlation_id: None,
            sequence: None,
        }
    }

    fn with_task(mut self, task_id: TaskId, attempt_id: TaskAttemptId) -> Self {
        self.task_id = Some(task_id);
        self.task_attempt_id = Some(attempt_id);
        self
    }
}
