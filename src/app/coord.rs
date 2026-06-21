use super::{CustomScopeId, ResourceId, SurfaceId};

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AppScope {
    segments: Vec<ScopePathSegment>,
}

impl AppScope {
    #[must_use]
    pub fn app() -> Self {
        Self {
            segments: vec![ScopePathSegment::new("app", "app")],
        }
    }

    #[must_use]
    pub fn workspace(id: impl Into<String>) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("workspace", id)],
        }
    }

    #[must_use]
    pub fn document(id: impl Into<String>) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("document", id)],
        }
    }

    #[must_use]
    pub fn window(id: crate::window::Id) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("window", id.as_u64().to_string())],
        }
    }

    #[must_use]
    pub fn surface(id: SurfaceId) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("surface", id.as_u64().to_string())],
        }
    }

    #[must_use]
    pub fn widget(id: impl Into<String>) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("widget", id)],
        }
    }

    #[must_use]
    pub fn resource(id: ResourceId) -> Self {
        Self {
            segments: vec![ScopePathSegment::new("resource", id.as_str())],
        }
    }

    #[must_use]
    pub fn custom(id: impl Into<String>) -> Self {
        let id = CustomScopeId::new(id);
        Self {
            segments: vec![ScopePathSegment::new("custom", id.as_str())],
        }
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
        self.segments.len() == 1
            && self.segments[0].namespace() == "app"
            && self.segments[0].value() == "app"
    }

    #[must_use]
    pub fn window_id(&self) -> Option<crate::window::Id> {
        self.first_segment_value("window")
            .and_then(|value| value.parse::<u64>().ok())
            .map(crate::window::Id::from_u64)
    }

    #[must_use]
    pub fn surface_id(&self) -> Option<SurfaceId> {
        self.first_segment_value("surface")
            .and_then(|value| value.parse::<u64>().ok())
            .map(SurfaceId::from_u64)
    }

    #[must_use]
    pub fn resource_id(&self) -> Option<ResourceId> {
        self.first_segment_value("resource").map(ResourceId::new)
    }

    fn first_segment_value(&self, namespace: &str) -> Option<&str> {
        self.segments
            .first()
            .filter(|segment| segment.namespace() == namespace)
            .map(ScopePathSegment::value)
    }
}
