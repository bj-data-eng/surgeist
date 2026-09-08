//! Layout-ready fixture metadata schemas.
//!
//! These types describe metadata that has already been composed for layout
//! consumption. They intentionally avoid depending on `surgeist-layout` so the
//! fixture schema can remain owned by this test infrastructure crate.

use std::error::Error;
use std::fmt;
use std::path::{Component, Path};

/// Current layout-ready fixture metadata schema version.
pub const CURRENT_SCHEMA_VERSION: SchemaVersion = SchemaVersion::new(1);

/// A layout-ready fixture metadata record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayoutReadyFixtureMetadata {
    schema_version: SchemaVersion,
    identity: FixtureIdentity,
    artifacts: ArtifactPaths,
    provenance: GeneratorProvenance,
    expectation: Option<FixtureExpectation>,
    status: FixtureStatus,
}

impl LayoutReadyFixtureMetadata {
    /// Creates metadata for a layout-ready fixture using the current schema.
    pub fn new(
        identity: FixtureIdentity,
        artifacts: ArtifactPaths,
        provenance: GeneratorProvenance,
    ) -> Self {
        Self {
            schema_version: SchemaVersion::current(),
            identity,
            artifacts,
            provenance,
            expectation: None,
            status: FixtureStatus::ready(),
        }
    }

    /// Returns a copy of this metadata with an expectation label attached.
    pub fn with_expectation(mut self, expectation: FixtureExpectation) -> Self {
        self.expectation = Some(expectation);
        self
    }

    /// Returns a copy of this metadata with an explicit fixture status.
    pub fn with_status(mut self, status: FixtureStatus) -> Self {
        self.status = status;
        self
    }

    pub fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    pub fn identity(&self) -> &FixtureIdentity {
        &self.identity
    }

    pub fn artifacts(&self) -> &ArtifactPaths {
        &self.artifacts
    }

    pub fn provenance(&self) -> &GeneratorProvenance {
        &self.provenance
    }

    pub fn expectation(&self) -> Option<&FixtureExpectation> {
        self.expectation.as_ref()
    }

    pub fn status(&self) -> &FixtureStatus {
        &self.status
    }
}

/// Version marker for layout-ready fixture metadata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SchemaVersion(u16);

impl SchemaVersion {
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    pub const fn current() -> Self {
        CURRENT_SCHEMA_VERSION
    }

    pub const fn get(self) -> u16 {
        self.0
    }
}

/// Stable identity for a fixture case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureIdentity(String);

impl FixtureIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaError> {
        validated_text(value, "fixture identity").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A fixture-corpus-relative path.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixturePath(String);

impl FixturePath {
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaError> {
        let value = validated_text(value, "fixture path")?;
        let path = Path::new(&value);

        if path.is_absolute() {
            return Err(SchemaError::absolute_path(value));
        }

        for component in path.components() {
            match component {
                Component::Normal(_) => {}
                Component::CurDir
                | Component::ParentDir
                | Component::RootDir
                | Component::Prefix(_) => {
                    return Err(SchemaError::non_relative_path(value));
                }
            }
        }

        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Source and generated artifact paths for a fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArtifactPaths {
    source: FixturePath,
    generated: FixturePath,
}

impl ArtifactPaths {
    pub fn new(source: FixturePath, generated: FixturePath) -> Self {
        Self { source, generated }
    }

    pub fn source(&self) -> &FixturePath {
        &self.source
    }

    pub fn generated(&self) -> &FixturePath {
        &self.generated
    }
}

/// Metadata about the generator that produced the layout-ready fixture.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratorProvenance {
    generator: String,
    adapter: Option<String>,
}

impl GeneratorProvenance {
    pub fn new(generator: impl Into<String>) -> Result<Self, SchemaError> {
        Ok(Self {
            generator: validated_text(generator, "generator")?,
            adapter: None,
        })
    }

    pub fn with_adapter(mut self, adapter: impl Into<String>) -> Result<Self, SchemaError> {
        self.adapter = Some(validated_text(adapter, "adapter")?);
        Ok(self)
    }

    pub fn generator(&self) -> &str {
        &self.generator
    }

    pub fn adapter(&self) -> Option<&str> {
        self.adapter.as_deref()
    }
}

/// Optional named expectation attached to layout-ready metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureExpectation(String);

impl FixtureExpectation {
    pub fn new(value: impl Into<String>) -> Result<Self, SchemaError> {
        validated_text(value, "fixture expectation").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// High-level fixture status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FixtureStatusKind {
    Ready,
    Ignored,
    KnownFailure,
}

/// Readiness status for a fixture case.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FixtureStatus {
    kind: FixtureStatusKind,
    reason: Option<String>,
}

impl FixtureStatus {
    pub fn ready() -> Self {
        Self {
            kind: FixtureStatusKind::Ready,
            reason: None,
        }
    }

    pub fn ignored(reason: impl Into<String>) -> Result<Self, SchemaError> {
        Ok(Self {
            kind: FixtureStatusKind::Ignored,
            reason: Some(validated_text(reason, "ignored reason")?),
        })
    }

    pub fn known_failure(reason: impl Into<String>) -> Result<Self, SchemaError> {
        Ok(Self {
            kind: FixtureStatusKind::KnownFailure,
            reason: Some(validated_text(reason, "known failure reason")?),
        })
    }

    pub fn kind(&self) -> FixtureStatusKind {
        self.kind
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }

    pub fn is_ready(&self) -> bool {
        self.kind == FixtureStatusKind::Ready
    }
}

/// Error returned when fixture metadata violates a schema invariant.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    field: &'static str,
    message: String,
}

impl SchemaError {
    fn empty(field: &'static str) -> Self {
        Self {
            field,
            message: "must not be empty".to_string(),
        }
    }

    fn absolute_path(value: String) -> Self {
        Self {
            field: "fixture path",
            message: format!("{value:?} must be relative"),
        }
    }

    fn non_relative_path(value: String) -> Self {
        Self {
            field: "fixture path",
            message: format!(
                "{value:?} must not contain current, parent, root, or prefix components"
            ),
        }
    }

    pub fn field(&self) -> &'static str {
        self.field
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} {}", self.field, self.message)
    }
}

impl Error for SchemaError {}

fn validated_text(value: impl Into<String>, field: &'static str) -> Result<String, SchemaError> {
    let value = value.into();
    if value.trim().is_empty() {
        return Err(SchemaError::empty(field));
    }
    Ok(value)
}
