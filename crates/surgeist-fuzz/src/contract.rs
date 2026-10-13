#![forbid(unsafe_code)]

use crate::{
    CaseFormat, FuzzError, FuzzErrorKind, GeneratorIdentity, PropertyId, Result, TargetId,
    error::invalid,
    identity::{kebab, owner},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Interpretation of the owner codec's concrete input.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InputEncoding {
    RawBytes,
    Structured,
}

/// Explicit capabilities that an execution environment can supply.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Property,
    Replay,
    Coverage,
    Reduction,
    CaseCapture,
    ProcessGroups,
    HardMemoryLimit,
    Browser,
}

/// Build features and mandatory runtime capabilities, with no implicit fallback.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawRequirements")]
pub struct BuildRequirements {
    features: Vec<String>,
    capabilities: Vec<Capability>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRequirements {
    features: Vec<String>,
    capabilities: Vec<Capability>,
}

impl TryFrom<RawRequirements> for BuildRequirements {
    type Error = FuzzError;
    fn try_from(raw: RawRequirements) -> Result<Self> {
        Self::new(raw.features, raw.capabilities)
    }
}

impl BuildRequirements {
    pub fn new(features: Vec<String>, capabilities: Vec<Capability>) -> Result<Self> {
        if features.iter().any(|feature| !feature_name(feature))
            || !unique(&features)
            || !unique(&capabilities)
        {
            return Err(invalid(
                FuzzErrorKind::InvalidDescriptor,
                "features/capabilities must be valid and unique",
            ));
        }
        Ok(Self {
            features,
            capabilities,
        })
    }
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
    #[must_use]
    pub fn capabilities(&self) -> &[Capability] {
        &self.capabilities
    }
    pub fn require_capabilities(&self, available: &[Capability]) -> Result<()> {
        for required in &self.capabilities {
            if !available.contains(required) {
                return Err(invalid(
                    FuzzErrorKind::MissingCapability,
                    format!("missing mandatory capability {required:?}"),
                ));
            }
        }
        Ok(())
    }
}

fn feature_name(value: &str) -> bool {
    (1..=128).contains(&value.len())
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b'-' | b'+'))
        && value.as_bytes()[0].is_ascii_alphanumeric()
}

fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

/// Hard admission ceilings. Text is measured in owner-declared code units.
/// A zero structural ceiling prohibits that resource; byte capacity is positive.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawLimits")]
pub struct InputLimits {
    bytes: u64,
    nodes: u64,
    depth: u64,
    text_units: u64,
    operations: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLimits {
    bytes: u64,
    nodes: u64,
    depth: u64,
    text_units: u64,
    operations: u64,
}

impl TryFrom<RawLimits> for InputLimits {
    type Error = FuzzError;
    fn try_from(raw: RawLimits) -> Result<Self> {
        Self::new(
            raw.bytes,
            raw.nodes,
            raw.depth,
            raw.text_units,
            raw.operations,
        )
    }
}

impl InputLimits {
    pub fn new(
        bytes: u64,
        nodes: u64,
        depth: u64,
        text_units: u64,
        operations: u64,
    ) -> Result<Self> {
        if bytes == 0 || (nodes == 0) != (depth == 0) || depth > nodes {
            return Err(invalid(
                FuzzErrorKind::InvalidLimits,
                "byte ceiling must be positive; node/depth ceilings must be consistent",
            ));
        }
        Ok(Self {
            bytes,
            nodes,
            depth,
            text_units,
            operations,
        })
    }
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
    #[must_use]
    pub const fn nodes(&self) -> u64 {
        self.nodes
    }
    #[must_use]
    pub const fn depth(&self) -> u64 {
        self.depth
    }
    #[must_use]
    pub const fn text_units(&self) -> u64 {
        self.text_units
    }
    #[must_use]
    pub const fn operations(&self) -> u64 {
        self.operations
    }
    pub fn check(&self, usage: InputUsage) -> Result<()> {
        if usage.bytes > self.bytes
            || usage.nodes > self.nodes
            || usage.depth > self.depth
            || usage.text_units > self.text_units
            || usage.operations > self.operations
        {
            return Err(invalid(
                FuzzErrorKind::InvalidLimits,
                "case exceeds hard input limits",
            ));
        }
        Ok(())
    }
}

/// Concrete resource consumption, validated independently of admission policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawUsage")]
pub struct InputUsage {
    bytes: u64,
    nodes: u64,
    depth: u64,
    text_units: u64,
    operations: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawUsage {
    bytes: u64,
    nodes: u64,
    depth: u64,
    text_units: u64,
    operations: u64,
}

impl TryFrom<RawUsage> for InputUsage {
    type Error = FuzzError;
    fn try_from(raw: RawUsage) -> Result<Self> {
        Self::new(
            raw.bytes,
            raw.nodes,
            raw.depth,
            raw.text_units,
            raw.operations,
        )
    }
}

impl InputUsage {
    pub fn new(
        bytes: u64,
        nodes: u64,
        depth: u64,
        text_units: u64,
        operations: u64,
    ) -> Result<Self> {
        if (nodes == 0) != (depth == 0) || depth > nodes {
            return Err(invalid(
                FuzzErrorKind::InvalidLimits,
                "case node/depth usage is inconsistent",
            ));
        }
        Ok(Self {
            bytes,
            nodes,
            depth,
            text_units,
            operations,
        })
    }
    #[must_use]
    pub const fn bytes(&self) -> u64 {
        self.bytes
    }
    #[must_use]
    pub const fn nodes(&self) -> u64 {
        self.nodes
    }
    #[must_use]
    pub const fn depth(&self) -> u64 {
        self.depth
    }
    #[must_use]
    pub const fn text_units(&self) -> u64 {
        self.text_units
    }
    #[must_use]
    pub const fn operations(&self) -> u64 {
        self.operations
    }
}

/// Unvalidated descriptor input. Construct a `TargetDescriptor` before use.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetDescriptorSpec {
    pub id: TargetId,
    pub owner: String,
    pub binary: String,
    pub properties: Vec<PropertyId>,
    pub source_references: Vec<String>,
    pub format: CaseFormat,
    pub generator_identity: GeneratorIdentity,
    pub encoding: InputEncoding,
    pub limits: InputLimits,
    pub requirements: BuildRequirements,
    pub min_evaluated: u64,
    pub max_generation_rejections: u64,
}

/// Checked owner metadata used by discovery, execution and persisted evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "TargetDescriptorSpec")]
pub struct TargetDescriptor {
    id: TargetId,
    owner: String,
    binary: String,
    properties: Vec<PropertyId>,
    source_references: Vec<String>,
    format: CaseFormat,
    generator_identity: GeneratorIdentity,
    encoding: InputEncoding,
    limits: InputLimits,
    requirements: BuildRequirements,
    min_evaluated: u64,
    max_generation_rejections: u64,
}

impl TryFrom<TargetDescriptorSpec> for TargetDescriptor {
    type Error = FuzzError;
    fn try_from(spec: TargetDescriptorSpec) -> Result<Self> {
        Self::new(spec)
    }
}

impl TargetDescriptor {
    pub fn new(spec: TargetDescriptorSpec) -> Result<Self> {
        if !owner(&spec.owner)
            || spec.id.owner() != spec.owner
            || !kebab(&spec.binary)
            || spec.properties.is_empty()
            || !unique(&spec.properties)
            || spec.properties.iter().any(|p| p.owner() != spec.owner)
            || spec.source_references.is_empty()
            || !unique(&spec.source_references)
            || spec.source_references.iter().any(|s| !line(s))
            || !spec
                .format
                .identity()
                .starts_with(&format!("{}::", spec.owner))
            || !spec
                .generator_identity
                .identity()
                .starts_with(&format!("{}::", spec.owner))
            || spec.min_evaluated == 0
        {
            return Err(invalid(
                FuzzErrorKind::InvalidDescriptor,
                "descriptor requires consistent owner identities, unique properties/sources, valid binary and positive minimum evaluated count",
            ));
        }
        if spec.encoding == InputEncoding::RawBytes
            && (spec.limits.nodes() != 0
                || spec.limits.depth() != 0
                || spec.limits.operations() != 0)
        {
            return Err(invalid(
                FuzzErrorKind::InvalidDescriptor,
                "raw-byte targets cannot declare structural or operation budgets",
            ));
        }
        Ok(Self {
            id: spec.id,
            owner: spec.owner,
            binary: spec.binary,
            properties: spec.properties,
            source_references: spec.source_references,
            format: spec.format,
            generator_identity: spec.generator_identity,
            encoding: spec.encoding,
            limits: spec.limits,
            requirements: spec.requirements,
            min_evaluated: spec.min_evaluated,
            max_generation_rejections: spec.max_generation_rejections,
        })
    }
    #[must_use]
    pub fn id(&self) -> &TargetId {
        &self.id
    }
    #[must_use]
    pub fn owner(&self) -> &str {
        &self.owner
    }
    #[must_use]
    pub fn binary(&self) -> &str {
        &self.binary
    }
    #[must_use]
    pub fn properties(&self) -> &[PropertyId] {
        &self.properties
    }
    #[must_use]
    pub fn source_references(&self) -> &[String] {
        &self.source_references
    }
    #[must_use]
    pub const fn format(&self) -> &CaseFormat {
        &self.format
    }
    #[must_use]
    pub const fn generator_identity(&self) -> &GeneratorIdentity {
        &self.generator_identity
    }
    #[must_use]
    pub const fn encoding(&self) -> InputEncoding {
        self.encoding
    }
    #[must_use]
    pub const fn limits(&self) -> InputLimits {
        self.limits
    }
    #[must_use]
    pub const fn requirements(&self) -> &BuildRequirements {
        &self.requirements
    }
    #[must_use]
    pub const fn min_evaluated(&self) -> u64 {
        self.min_evaluated
    }
    #[must_use]
    pub const fn max_generation_rejections(&self) -> u64 {
        self.max_generation_rejections
    }

    /// Rejects observations naming properties outside the owner's declared contract.
    pub fn validate_outcome(&self, outcome: &CaseOutcome) -> Result<()> {
        if outcome
            .checked_properties()
            .iter()
            .any(|p| !self.properties.contains(p))
            || outcome
                .failure_key()
                .is_some_and(|key| !self.properties.contains(key.property()))
        {
            return Err(invalid(
                FuzzErrorKind::InvalidObservation,
                "outcome names an undeclared property",
            ));
        }
        Ok(())
    }

    #[must_use]
    pub fn assess_generated(&self, counts: ExecutionCounts) -> CoverageAssessment {
        if counts.evaluated < self.min_evaluated
            || counts.generation_rejected > self.max_generation_rejections
        {
            CoverageAssessment::Insufficient
        } else {
            CoverageAssessment::Sufficient
        }
    }
}

/// Stable failure class, separate from expected product rejection and generation discard.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FailureClass {
    PropertyViolation,
    Panic,
    Crash,
    Timeout,
    Resource,
    HarnessError,
}

/// Reduction identity. Diagnostics and panic text do not participate in equality.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawFailureKey")]
pub struct FailureKey {
    property: PropertyId,
    class: FailureClass,
    discriminator: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawFailureKey {
    property: PropertyId,
    class: FailureClass,
    #[serde(deserialize_with = "required_option")]
    discriminator: Option<String>,
}

impl TryFrom<RawFailureKey> for FailureKey {
    type Error = FuzzError;
    fn try_from(raw: RawFailureKey) -> Result<Self> {
        Self::new(raw.property, raw.class, raw.discriminator)
    }
}

impl FailureKey {
    pub fn new(
        property: PropertyId,
        class: FailureClass,
        discriminator: Option<String>,
    ) -> Result<Self> {
        if discriminator.as_deref().is_some_and(|value| !kebab(value)) {
            return Err(invalid(
                FuzzErrorKind::InvalidObservation,
                "failure discriminator must be a stable kebab identifier",
            ));
        }
        Ok(Self {
            property,
            class,
            discriminator,
        })
    }
    #[must_use]
    pub const fn property(&self) -> &PropertyId {
        &self.property
    }
    #[must_use]
    pub const fn class(&self) -> FailureClass {
        self.class
    }
    #[must_use]
    pub fn discriminator(&self) -> Option<&str> {
        self.discriminator.as_deref()
    }
}

/// Observable evaluation state. An expected rejection need not have checked properties.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OutcomeKind {
    Evaluated,
    ExpectedRejection,
    Unsupported,
    Failure,
}

/// Closed observation record validated through the same boundary on deserialization.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawOutcome")]
pub struct CaseOutcome {
    kind: OutcomeKind,
    checked_properties: Vec<PropertyId>,
    failure_key: Option<FailureKey>,
    detail: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawOutcome {
    kind: OutcomeKind,
    checked_properties: Vec<PropertyId>,
    #[serde(deserialize_with = "required_option")]
    failure_key: Option<FailureKey>,
    #[serde(deserialize_with = "required_option")]
    detail: Option<String>,
}

fn required_option<'de, D, T>(deserializer: D) -> std::result::Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer)
}

impl TryFrom<RawOutcome> for CaseOutcome {
    type Error = FuzzError;
    fn try_from(raw: RawOutcome) -> Result<Self> {
        Self::checked(
            raw.kind,
            raw.checked_properties,
            raw.failure_key,
            raw.detail,
        )
    }
}

impl CaseOutcome {
    pub fn evaluated(properties: Vec<PropertyId>) -> Result<Self> {
        Self::checked(OutcomeKind::Evaluated, properties, None, None)
    }
    pub fn expected_rejection(
        properties: Vec<PropertyId>,
        reason: impl Into<String>,
    ) -> Result<Self> {
        Self::checked(
            OutcomeKind::ExpectedRejection,
            properties,
            None,
            Some(reason.into()),
        )
    }
    pub fn unsupported(reason: impl Into<String>) -> Result<Self> {
        Self::checked(
            OutcomeKind::Unsupported,
            Vec::new(),
            None,
            Some(reason.into()),
        )
    }
    pub fn failure(key: FailureKey, detail: impl Into<String>) -> Result<Self> {
        Self::checked(
            OutcomeKind::Failure,
            Vec::new(),
            Some(key),
            Some(detail.into()),
        )
    }

    fn checked(
        kind: OutcomeKind,
        checked_properties: Vec<PropertyId>,
        failure_key: Option<FailureKey>,
        detail: Option<String>,
    ) -> Result<Self> {
        let valid_fields = match kind {
            OutcomeKind::Evaluated => {
                !checked_properties.is_empty() && failure_key.is_none() && detail.is_none()
            }
            OutcomeKind::ExpectedRejection => failure_key.is_none() && detail.is_some(),
            OutcomeKind::Unsupported => {
                checked_properties.is_empty() && failure_key.is_none() && detail.is_some()
            }
            OutcomeKind::Failure => {
                checked_properties.is_empty() && failure_key.is_some() && detail.is_some()
            }
        };
        if !valid_fields
            || !unique(&checked_properties)
            || detail.as_deref().is_some_and(|value| !line(value))
        {
            return Err(invalid(
                FuzzErrorKind::InvalidObservation,
                "outcome fields do not match their declared kind",
            ));
        }
        Ok(Self {
            kind,
            checked_properties,
            failure_key,
            detail,
        })
    }
    #[must_use]
    pub const fn kind(&self) -> OutcomeKind {
        self.kind
    }
    #[must_use]
    pub fn checked_properties(&self) -> &[PropertyId] {
        &self.checked_properties
    }
    #[must_use]
    pub const fn failure_key(&self) -> Option<&FailureKey> {
        self.failure_key.as_ref()
    }
    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
    #[must_use]
    pub fn is_evaluated(&self) -> bool {
        !self.checked_properties.is_empty()
            || self
                .failure_key
                .as_ref()
                .is_some_and(|key| key.class == FailureClass::PropertyViolation)
    }
}

fn line(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && value.trim() == value
        && !value.chars().any(char::is_control)
}

/// Generation either materializes a concrete case or records an explicit discard.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Generation<C> {
    Generated(C),
    Rejected(GenerationRejection),
}

/// A generation discard is never a checked property or product failure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawGenerationRejection")]
pub struct GenerationRejection {
    reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawGenerationRejection {
    reason: String,
}

impl TryFrom<RawGenerationRejection> for GenerationRejection {
    type Error = FuzzError;
    fn try_from(raw: RawGenerationRejection) -> Result<Self> {
        Self::new(raw.reason)
    }
}

impl GenerationRejection {
    pub fn new(reason: impl Into<String>) -> Result<Self> {
        let reason = reason.into();
        if !line(&reason) {
            return Err(invalid(
                FuzzErrorKind::InvalidObservation,
                "generation discard requires a nonempty trimmed reason",
            ));
        }
        Ok(Self { reason })
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Nonvacuous evaluation and generation-discard counts, with checked increments.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionCounts {
    evaluated: u64,
    generation_rejected: u64,
}

impl ExecutionCounts {
    #[must_use]
    pub const fn new(evaluated: u64, generation_rejected: u64) -> Self {
        Self {
            evaluated,
            generation_rejected,
        }
    }
    #[must_use]
    pub const fn evaluated(&self) -> u64 {
        self.evaluated
    }
    #[must_use]
    pub const fn generation_rejected(&self) -> u64 {
        self.generation_rejected
    }
    pub fn record_outcome(&mut self, outcome: &CaseOutcome) -> Result<()> {
        if outcome.is_evaluated() {
            self.evaluated = increment(self.evaluated)?;
        }
        Ok(())
    }
    pub fn record_generation_rejection(&mut self) -> Result<()> {
        self.generation_rejected = increment(self.generation_rejected)?;
        Ok(())
    }
    #[must_use]
    pub const fn assess_regressions(self) -> CoverageAssessment {
        if self.evaluated == 0 {
            CoverageAssessment::Empty
        } else {
            CoverageAssessment::Sufficient
        }
    }
}

fn increment(value: u64) -> Result<u64> {
    value
        .checked_add(1)
        .ok_or_else(|| invalid(FuzzErrorKind::Harness, "execution counter overflow"))
}

/// Coverage sufficiency only; product failures remain separate observations.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageAssessment {
    Sufficient,
    Insufficient,
    Empty,
}

/// Owner-implemented semantic boundary shared by generation, engines and replay.
/// Decode must admit limits before allocation; encode produces canonical replay bytes.
/// Every call starts from fresh per-case state. No engine implements domain semantics.
pub trait CaseTarget {
    type Case;
    fn descriptor(&self) -> &TargetDescriptor;
    fn decode(&self, input: &[u8]) -> Result<Self::Case>;
    fn encode(&self, case: &Self::Case) -> Result<Vec<u8>>;
    fn generate(&self, input: &[u8]) -> Result<Generation<Self::Case>>;
    fn evaluate(&self, case: &Self::Case) -> Result<CaseOutcome>;

    /// Emits owner-valid candidates until the visitor returns false. Implementations
    /// must stop immediately then; the caller owns attempt/time/output budgets.
    fn reduction_candidates(
        &self,
        _case: &Self::Case,
        _emit: &mut dyn FnMut(Self::Case) -> bool,
    ) -> Result<()> {
        Ok(())
    }
}
