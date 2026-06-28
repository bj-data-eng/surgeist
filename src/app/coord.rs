use super::{CustomScopeId, ResourceId, SurfaceId};
use crate::window::Id;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ScopePathSegment {
    namespace: String,
    value: String,
}

impl ScopePathSegment {
    #[must_use]
    pub fn new(namespace: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            value: value.into(),
        }
    }

    #[must_use]
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AppScope {
    segments: Vec<ScopePathSegment>,
}

impl AppScope {
    #[must_use]
    pub fn app() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    #[must_use]
    pub fn window(id: Id) -> Self {
        Self::app().then(ScopePathSegment::new("window", id.as_u64().to_string()))
    }

    #[must_use]
    pub fn surface(id: SurfaceId) -> Self {
        Self::app().then(ScopePathSegment::new("surface", id.as_u64().to_string()))
    }

    #[must_use]
    pub fn resource(id: ResourceId) -> Self {
        Self::app().then(ScopePathSegment::new("resource", id.as_str()))
    }

    #[must_use]
    pub fn custom(id: CustomScopeId) -> Self {
        Self::app().then(ScopePathSegment::new("custom", id.as_str()))
    }

    #[must_use]
    pub fn workspace(value: impl Into<String>) -> Self {
        Self::app().then(ScopePathSegment::new("workspace", value))
    }

    #[must_use]
    pub fn document(value: impl Into<String>) -> Self {
        Self::app().then(ScopePathSegment::new("document", value))
    }

    #[must_use]
    pub fn widget(value: impl Into<String>) -> Self {
        Self::app().then(ScopePathSegment::new("widget", value))
    }

    #[must_use]
    pub fn then(mut self, segment: ScopePathSegment) -> Self {
        self.segments.push(segment);
        self
    }

    #[must_use]
    pub fn segments(&self) -> &[ScopePathSegment] {
        &self.segments
    }

    #[must_use]
    pub fn is_app(&self) -> bool {
        self.segments.is_empty()
    }

    #[must_use]
    pub fn resource_id(&self) -> Option<ResourceId> {
        self.last_value("resource").map(ResourceId::new)
    }

    #[must_use]
    pub fn window_id(&self) -> Option<Id> {
        self.last_value("window")
            .and_then(|value| value.parse().ok())
            .map(Id::from_u64)
    }

    #[must_use]
    pub fn surface_id(&self) -> Option<SurfaceId> {
        self.last_value("surface")
            .and_then(|value| value.parse().ok())
            .map(SurfaceId::from_u64)
    }

    fn last_value(&self, namespace: &str) -> Option<&str> {
        self.segments
            .iter()
            .rev()
            .find(|segment| segment.namespace() == namespace)
            .map(ScopePathSegment::value)
    }
}
