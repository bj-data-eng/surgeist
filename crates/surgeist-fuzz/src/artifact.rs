//! Strict evidence schemas, bounded rooted publication, and exact owner-codec replay.
#![forbid(unsafe_code)]

use crate::{
    CaseOutcome, CaseTarget, ExecutionCounts, FuzzError, FuzzErrorKind, PropertyId, RelativePath,
    Result, Sha256Digest, SourceRevision, TargetDescriptor,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

const SCHEMA: u32 = 1;
const NAMESPACES: [&str; 7] = [
    "failures",
    "captures",
    "corpus",
    "runs",
    "comparisons",
    "capture-replays",
    "reductions",
];
const MAX_METADATA: u64 = 1024 * 1024;
const MAX_INPUT: u64 = 64 * 1024 * 1024;

fn invalid(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Artifact, "fuzz artifact", detail)
}
fn resource(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "fuzz artifact quota", detail)
}
fn io_error(error: impl std::error::Error + Send + Sync + 'static) -> FuzzError {
    FuzzError::with_source(
        FuzzErrorKind::Io,
        "fuzz artifact I/O",
        "publication may be partial; inspect recovery report",
        error,
    )
}
fn text(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 4096
        && value.trim() == value
        && !value.chars().any(char::is_control)
}
fn unique<T: Ord>(values: &[T]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}
fn present<'de, D, T>(d: D) -> std::result::Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d)
}

fn bounded_metadata<T: Serialize>(value: &T, ceiling: u64) -> Result<Vec<u8>> {
    struct Buffer {
        bytes: Vec<u8>,
        ceiling: u64,
    }
    impl Write for Buffer {
        fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
            if (self.bytes.len() as u64)
                .checked_add(input.len() as u64)
                .is_none_or(|n| n > self.ceiling)
            {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::StorageFull,
                    "metadata exceeds publication byte quota",
                ));
            }
            self.bytes.extend_from_slice(input);
            Ok(input.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut buffer = Buffer {
        bytes: Vec::new(),
        ceiling,
    };
    serde_json::to_writer(&mut buffer, value).map_err(io_error)?;
    Ok(buffer.bytes)
}

/// Content's role in replay: fixtures affect case meaning; tools and engines
/// identify how the selected executable was built and run.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ContentRole {
    Fixture,
    BuildTool,
    Engine,
}

/// Explicit content identity; role is mandatory in every persisted schema.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ContentIdentitySpec")]
pub struct ContentIdentity {
    role: ContentRole,
    name: String,
    digest: Sha256Digest,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContentIdentitySpec {
    pub role: ContentRole,
    pub name: String,
    pub digest: Sha256Digest,
}
impl TryFrom<ContentIdentitySpec> for ContentIdentity {
    type Error = FuzzError;
    fn try_from(s: ContentIdentitySpec) -> Result<Self> {
        Self::new(s.role, s.name, s.digest)
    }
}
impl ContentIdentity {
    pub fn new(role: ContentRole, name: impl Into<String>, digest: Sha256Digest) -> Result<Self> {
        let name = name.into();
        if !text(&name) {
            return Err(invalid("invalid content identity name"));
        }
        Ok(Self { role, name, digest })
    }
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    #[must_use]
    pub const fn role(&self) -> ContentRole {
        self.role
    }
    #[must_use]
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}

/// Complete selected executable basis. This describes the executable actually used.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "BuildIdentitySpec")]
pub struct BuildIdentity {
    source: SourceRevision,
    lockfile: Sha256Digest,
    executable: Sha256Digest,
    toolchain: String,
    engine: String,
    sanitizer: String,
    platform: String,
    features: Vec<String>,
    providers: Vec<ContentIdentity>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BuildIdentitySpec {
    pub source: SourceRevision,
    pub lockfile: Sha256Digest,
    pub executable: Sha256Digest,
    pub toolchain: String,
    pub engine: String,
    pub sanitizer: String,
    pub platform: String,
    pub features: Vec<String>,
    pub providers: Vec<ContentIdentity>,
}
impl TryFrom<BuildIdentitySpec> for BuildIdentity {
    type Error = FuzzError;
    fn try_from(s: BuildIdentitySpec) -> Result<Self> {
        Self::new(s)
    }
}
impl BuildIdentity {
    pub fn new(mut s: BuildIdentitySpec) -> Result<Self> {
        if [&s.toolchain, &s.engine, &s.sanitizer, &s.platform]
            .iter()
            .any(|v| !text(v))
            || s.features.len() > 256
            || s.features.iter().any(|v| !text(v))
            || !unique(&s.features)
            || s.providers.len() > 256
            || !unique(
                &s.providers
                    .iter()
                    .map(ContentIdentity::name)
                    .collect::<Vec<_>>(),
            )
        {
            return Err(invalid(
                "build requires explicit bounded toolchain/engine/sanitizer/platform and unique features/providers",
            ));
        }
        s.features.sort();
        s.providers.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Self {
            source: s.source,
            lockfile: s.lockfile,
            executable: s.executable,
            toolchain: s.toolchain,
            engine: s.engine,
            sanitizer: s.sanitizer,
            platform: s.platform,
            features: s.features,
            providers: s.providers,
        })
    }
    #[must_use]
    pub fn source(&self) -> &SourceRevision {
        &self.source
    }
    #[must_use]
    pub fn executable(&self) -> &Sha256Digest {
        &self.executable
    }
    #[must_use]
    pub fn features(&self) -> &[String] {
        &self.features
    }
    #[must_use]
    pub fn providers(&self) -> &[ContentIdentity] {
        &self.providers
    }
    fn fixtures(&self) -> impl Iterator<Item = &ContentIdentity> {
        self.providers
            .iter()
            .filter(|content| content.role == ContentRole::Fixture)
    }
    fn same_fixtures(&self, other: &Self) -> bool {
        self.fixtures().eq(other.fixtures())
    }
}

/// Finite campaign ceilings. Milliseconds and bytes are positive integer units.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RunConfigSpec")]
pub struct RunConfig {
    cases: u64,
    wall_millis: u64,
    case_millis: u64,
    memory_bytes: u64,
    seed: Option<u64>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunConfigSpec {
    pub cases: u64,
    pub wall_millis: u64,
    pub case_millis: u64,
    pub memory_bytes: u64,
    #[serde(deserialize_with = "present")]
    pub seed: Option<u64>,
}
impl TryFrom<RunConfigSpec> for RunConfig {
    type Error = FuzzError;
    fn try_from(s: RunConfigSpec) -> Result<Self> {
        Self::new(s)
    }
}
impl RunConfig {
    pub fn new(s: RunConfigSpec) -> Result<Self> {
        if s.cases == 0
            || s.wall_millis == 0
            || s.case_millis == 0
            || s.memory_bytes == 0
            || s.case_millis > s.wall_millis
        {
            return Err(invalid(
                "run requires positive finite budgets; case duration cannot exceed run duration",
            ));
        }
        Ok(Self {
            cases: s.cases,
            wall_millis: s.wall_millis,
            case_millis: s.case_millis,
            memory_bytes: s.memory_bytes,
            seed: s.seed,
        })
    }
    #[must_use]
    pub const fn cases(&self) -> u64 {
        self.cases
    }
    #[must_use]
    pub const fn wall_millis(&self) -> u64 {
        self.wall_millis
    }
    #[must_use]
    pub const fn case_millis(&self) -> u64 {
        self.case_millis
    }
    #[must_use]
    pub const fn memory_bytes(&self) -> u64 {
        self.memory_bytes
    }
    #[must_use]
    pub const fn seed(&self) -> Option<u64> {
        self.seed
    }
}

/// Metadata references a separate exact binary input; JSON never contains case bytes.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "CaseMetadataSpec")]
pub struct CaseMetadata {
    schema: u32,
    target: TargetDescriptor,
    build: BuildIdentity,
    config: RunConfig,
    input_digest: Sha256Digest,
    input_bytes: u64,
    outcome: CaseOutcome,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaseMetadataSpec {
    pub schema: u32,
    pub target: TargetDescriptor,
    pub build: BuildIdentity,
    pub config: RunConfig,
    pub input_digest: Sha256Digest,
    pub input_bytes: u64,
    pub outcome: CaseOutcome,
}
impl TryFrom<CaseMetadataSpec> for CaseMetadata {
    type Error = FuzzError;
    fn try_from(s: CaseMetadataSpec) -> Result<Self> {
        if s.schema != SCHEMA
            || s.input_bytes > s.target.limits().bytes()
            || s.input_bytes > MAX_INPUT
            || s.target
                .requirements()
                .features()
                .iter()
                .any(|f| !s.build.features.contains(f))
        {
            return Err(invalid(
                "unsupported schema or inconsistent case/build limits",
            ));
        }
        s.target.validate_outcome(&s.outcome)?;
        Ok(Self {
            schema: s.schema,
            target: s.target,
            build: s.build,
            config: s.config,
            input_digest: s.input_digest,
            input_bytes: s.input_bytes,
            outcome: s.outcome,
        })
    }
}
impl CaseMetadata {
    #[must_use]
    pub fn target(&self) -> &TargetDescriptor {
        &self.target
    }
    #[must_use]
    pub fn build(&self) -> &BuildIdentity {
        &self.build
    }
    #[must_use]
    pub fn config(&self) -> &RunConfig {
        &self.config
    }
    #[must_use]
    pub fn input_digest(&self) -> &Sha256Digest {
        &self.input_digest
    }
    #[must_use]
    pub const fn input_bytes(&self) -> u64 {
        self.input_bytes
    }
    #[must_use]
    pub fn outcome(&self) -> &CaseOutcome {
        &self.outcome
    }
}

/// Validated coupling of exact owner-encoded case bytes and their observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaseArtifact {
    metadata: CaseMetadata,
    input: Vec<u8>,
}
impl CaseArtifact {
    pub fn new(
        target: TargetDescriptor,
        build: BuildIdentity,
        config: RunConfig,
        input: Vec<u8>,
        outcome: CaseOutcome,
    ) -> Result<Self> {
        let metadata = CaseMetadata::try_from(CaseMetadataSpec {
            schema: SCHEMA,
            target,
            build,
            config,
            input_digest: Sha256Digest::from_bytes(&input),
            input_bytes: input.len() as u64,
            outcome,
        })?;
        Self::from_parts(metadata, input)
    }
    pub fn from_parts(metadata: CaseMetadata, input: Vec<u8>) -> Result<Self> {
        if input.len() as u64 != metadata.input_bytes
            || Sha256Digest::from_bytes(&input) != metadata.input_digest
        {
            return Err(invalid(
                "input length/digest does not match recorded exact case",
            ));
        }
        Ok(Self { metadata, input })
    }
    #[must_use]
    pub fn metadata(&self) -> &CaseMetadata {
        &self.metadata
    }
    #[must_use]
    pub fn input(&self) -> &[u8] {
        &self.input
    }
}

/// Capture state supplied by the engine/supervisor; never reconstructed from a seed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EvidenceStatus {
    Available { artifact: RelativePath },
    Unavailable { reason: EvidenceUnavailable },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceUnavailable {
    NoCurrentCase,
    InterruptedCapture,
    PublicationFailed,
    QuotaExceeded,
}

fn failure_reference(reference: &RelativePath) -> bool {
    reference
        .as_str()
        .split_once('/')
        .is_some_and(|(namespace, id)| {
            matches!(namespace, "failures" | "captures") && Sha256Digest::from_text(id).is_ok()
        })
}

/// Terminal run observation. Counts describe actual execution, not conformance.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RunRecordSpec")]
pub struct RunRecord {
    schema: u32,
    target: TargetDescriptor,
    build: BuildIdentity,
    config: RunConfig,
    counts: ExecutionCounts,
    checked_properties: Vec<PropertyId>,
    exclusions: Vec<String>,
    failures: Vec<EvidenceStatus>,
    cleanup: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordSpec {
    pub schema: u32,
    pub target: TargetDescriptor,
    pub build: BuildIdentity,
    pub config: RunConfig,
    pub counts: ExecutionCounts,
    pub checked_properties: Vec<PropertyId>,
    pub exclusions: Vec<String>,
    pub failures: Vec<EvidenceStatus>,
    pub cleanup: String,
}
impl TryFrom<RunRecordSpec> for RunRecord {
    type Error = FuzzError;
    fn try_from(s: RunRecordSpec) -> Result<Self> {
        Self::new(s)
    }
}
impl RunRecord {
    pub fn new(s: RunRecordSpec) -> Result<Self> {
        if s.schema != SCHEMA
            || !text(&s.cleanup)
            || !unique(&s.checked_properties)
            || s.checked_properties
                .iter()
                .any(|p| !s.target.properties().contains(p))
            || s.exclusions.len() > 256
            || s.exclusions.iter().any(|v| !text(v))
            || s.failures.len() > 100_000
            || s.failures.iter().any(|evidence| {
                matches!(evidence,
                EvidenceStatus::Available { artifact } if !failure_reference(artifact))
            })
            || s.counts
                .evaluated()
                .checked_add(s.counts.generation_rejected())
                .is_none_or(|total| total > s.config.cases)
            || s.target
                .requirements()
                .features()
                .iter()
                .any(|f| !s.build.features.contains(f))
        {
            return Err(invalid(
                "unsupported schema or inconsistent bounded run observation",
            ));
        }
        Ok(Self {
            schema: s.schema,
            target: s.target,
            build: s.build,
            config: s.config,
            counts: s.counts,
            checked_properties: s.checked_properties,
            exclusions: s.exclusions,
            failures: s.failures,
            cleanup: s.cleanup,
        })
    }
    #[must_use]
    pub const fn counts(&self) -> ExecutionCounts {
        self.counts
    }
    #[must_use]
    pub fn readable_report(&self) -> String {
        let properties = self
            .checked_properties
            .iter()
            .map(PropertyId::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        format!(
            "Target: {}\nSource: {}\nChecked properties: {}\nExecuted: {}\nGeneration rejected: {}\nExclusions: {}\nFailure evidence: {:?}\nCleanup: {}\nFinite observations only; no exhaustive correctness claim.\n",
            self.target.id(),
            self.build.source,
            properties,
            self.counts.evaluated(),
            self.counts.generation_rejected(),
            self.exclusions.join("; "),
            self.failures,
            self.cleanup
        )
    }
}

/// Strict replay requires the complete recorded build; comparison still requires
/// compatible target, owner codec, limits and fixture/provider identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplayMode {
    Exact,
    Comparison,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ReplayRecordSpec")]
pub struct ReplayRecord {
    schema: u32,
    target: TargetDescriptor,
    original_input: Sha256Digest,
    original_build: BuildIdentity,
    selected_build: BuildIdentity,
    comparison: bool,
    observed: CaseOutcome,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplayRecordSpec {
    schema: u32,
    target: TargetDescriptor,
    original_input: Sha256Digest,
    original_build: BuildIdentity,
    selected_build: BuildIdentity,
    comparison: bool,
    observed: CaseOutcome,
}
impl TryFrom<ReplayRecordSpec> for ReplayRecord {
    type Error = FuzzError;
    fn try_from(s: ReplayRecordSpec) -> Result<Self> {
        if s.schema != SCHEMA
            || (!s.comparison && s.original_build != s.selected_build)
            || !s.original_build.same_fixtures(&s.selected_build)
            || s.target
                .requirements()
                .features()
                .iter()
                .any(|f| !s.selected_build.features.contains(f))
        {
            return Err(invalid(
                "unsupported schema or inconsistent replay identity",
            ));
        }
        s.target.validate_outcome(&s.observed)?;
        Ok(Self {
            schema: s.schema,
            target: s.target,
            original_input: s.original_input,
            original_build: s.original_build,
            selected_build: s.selected_build,
            comparison: s.comparison,
            observed: s.observed,
        })
    }
}
impl ReplayRecord {
    #[must_use]
    pub const fn is_comparison(&self) -> bool {
        self.comparison
    }
    #[must_use]
    pub fn observed(&self) -> &CaseOutcome {
        &self.observed
    }
}
/// Validate before starting an isolated replay worker.
fn validate_replay_identity(
    selected: &TargetDescriptor,
    original: &TargetDescriptor,
    original_build: &BuildIdentity,
    build: &BuildIdentity,
    mode: ReplayMode,
) -> Result<()> {
    // Generator revision changes cannot affect exact materialized replay.
    if selected.id() != original.id()
        || selected.format() != original.format()
        || selected.encoding() != original.encoding()
        || selected.limits() != original.limits()
        || selected.properties() != original.properties()
        || !build.same_fixtures(original_build)
        || selected
            .requirements()
            .features()
            .iter()
            .any(|f| !build.features.contains(f))
    {
        return Err(invalid(
            "replay target, codec, limits, properties or fixture identity mismatch",
        ));
    }
    if mode == ReplayMode::Exact && build != original_build {
        return Err(invalid(
            "exact replay requires the recorded executable/build identity",
        ));
    }
    Ok(())
}
impl CaseArtifact {
    pub fn validate_replay(
        &self,
        target: &TargetDescriptor,
        build: &BuildIdentity,
        mode: ReplayMode,
    ) -> Result<()> {
        validate_replay_identity(
            target,
            &self.metadata.target,
            &self.metadata.build,
            build,
            mode,
        )
    }
}
/// Record an observation supplied by an isolated worker after the caller has
/// validated replay identity. This does not infer reproduction from process exit.
pub fn replay_observation(
    original: &CaseArtifact,
    descriptor: &TargetDescriptor,
    build: &BuildIdentity,
    mode: ReplayMode,
    observed: CaseOutcome,
) -> Result<ReplayRecord> {
    original.validate_replay(descriptor, build, mode)?;
    descriptor.validate_outcome(&observed)?;
    Ok(ReplayRecord {
        schema: SCHEMA,
        target: descriptor.clone(),
        original_input: original.metadata.input_digest.clone(),
        original_build: original.metadata.build.clone(),
        selected_build: build.clone(),
        comparison: mode == ReplayMode::Comparison,
        observed,
    })
}
/// Direct in-process replay for callers whose execution boundary permits it.
pub fn replay_case<T: CaseTarget>(
    target: &T,
    artifact: &CaseArtifact,
    build: &BuildIdentity,
    mode: ReplayMode,
) -> Result<ReplayRecord> {
    artifact.validate_replay(target.descriptor(), build, mode)?;
    let case = target.decode(artifact.input())?;
    if target.encode(&case)?.as_slice() != artifact.input() {
        return Err(invalid(
            "owner codec did not preserve canonical persisted bytes",
        ));
    }
    replay_observation(
        artifact,
        target.descriptor(),
        build,
        mode,
        target.evaluate(&case)?,
    )
}

/// Absolute bounds apply to both publication and hostile/partial storage decoding.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageLimits {
    entries: u64,
    bytes: u64,
    input_bytes: u64,
    metadata_bytes: u64,
}
impl StorageLimits {
    pub fn new(entries: u64, bytes: u64, input_bytes: u64, metadata_bytes: u64) -> Result<Self> {
        if entries == 0
            || entries > 100_000
            || bytes == 0
            || input_bytes == 0
            || input_bytes > MAX_INPUT
            || metadata_bytes == 0
            || metadata_bytes > MAX_METADATA
            || input_bytes > bytes
            || metadata_bytes > bytes
        {
            return Err(invalid("invalid bounded storage quotas"));
        }
        Ok(Self {
            entries,
            bytes,
            input_bytes,
            metadata_bytes,
        })
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryReport {
    pub published: Vec<RelativePath>,
    pub partial: Vec<RelativePath>,
    pub bytes: u64,
}

// Safe handle-relative operations prevent component substitution or symlink escapes.
// Roots are explicitly caller-owned mutable campaign directories. Checked-in
// owner seeds stay outside this namespace and are never minimized or overwritten.
#[cfg(unix)]
pub(crate) mod rooted {
    use super::*;
    use rustix::fd::OwnedFd;
    use rustix::fs::{self, AtFlags, Dir, FileType, FlockOperation, Mode, OFlags};

    pub(super) fn directory(parent: &OwnedFd, name: &str, create: bool) -> Result<OwnedFd> {
        if create {
            match fs::mkdirat(parent, name, Mode::from_raw_mode(0o700)) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(e) => return Err(io_error(e)),
            }
        }
        fs::openat(
            parent,
            name,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io_error)
    }
    pub(crate) fn root(path: &Path) -> Result<OwnedFd> {
        let mut fd = fs::open(
            "/",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io_error)?;
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir().map_err(io_error)?.join(path)
        };
        for component in absolute.components() {
            match component {
                Component::RootDir => {}
                Component::Normal(name) => {
                    fd = fs::openat(
                        &fd,
                        name,
                        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
                        Mode::empty(),
                    )
                    .map_err(io_error)?;
                }
                _ => {
                    return Err(invalid(
                        "storage root must not contain dot/parent components",
                    ));
                }
            }
        }
        Ok(fd)
    }
    pub(super) fn read(fd: &OwnedFd, name: &str, ceiling: u64) -> Result<Vec<u8>> {
        let owned = fs::openat(
            fd,
            name,
            OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map_err(io_error)?;
        let mut file = File::from(owned);
        let metadata = file.metadata().map_err(io_error)?;
        if !metadata.is_file() || metadata.len() > ceiling {
            return Err(invalid(
                "artifact file is nonregular or exceeds decode ceiling",
            ));
        }
        let mut bytes = Vec::new();
        (&mut file)
            .take(ceiling + 1)
            .read_to_end(&mut bytes)
            .map_err(io_error)?;
        if bytes.len() as u64 > ceiling {
            return Err(invalid("artifact grew past decode ceiling"));
        }
        Ok(bytes)
    }
    pub(super) fn write(fd: &OwnedFd, name: &str, bytes: &[u8]) -> Result<()> {
        let owned = fs::openat(
            fd,
            name,
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(io_error)?;
        let mut file = File::from(owned);
        file.write_all(bytes).map_err(io_error)?;
        file.sync_all().map_err(io_error)
    }
    pub(super) fn names(fd: &OwnedFd, ceiling: u64) -> Result<Vec<(String, FileType)>> {
        let mut dir = Dir::read_from(fd).map_err(io_error)?;
        let mut entries = Vec::new();
        while let Some(entry) = dir.read() {
            let entry = entry.map_err(io_error)?;
            let name = entry
                .file_name()
                .to_str()
                .map_err(|_| invalid("non-UTF-8 artifact entry"))?;
            if name == "." || name == ".." {
                continue;
            }
            if entries.len() as u64 >= ceiling {
                return Err(resource("artifact entry count exceeds quota"));
            }
            entries.push((name.to_owned(), entry.file_type()));
        }
        entries.sort_by(|a, b| a.0.cmp(&b.0));
        Ok(entries)
    }
    pub(super) fn lock(root: &OwnedFd) -> Result<OwnedFd> {
        let fd = fs::openat(
            root,
            "publication.lock",
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
            Mode::from_raw_mode(0o600),
        )
        .map_err(io_error)?;
        if FileType::from_raw_mode(fs::fstat(&fd).map_err(io_error)?.st_mode)
            != FileType::RegularFile
        {
            return Err(invalid("publication lock is not a regular file"));
        }
        // Publication is serialized across store handles and processes. A
        // bounded wait admits concurrent workers without masking a stuck writer.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            match fs::flock(&fd, FlockOperation::NonBlockingLockExclusive) {
                Ok(()) => break,
                Err(rustix::io::Errno::WOULDBLOCK) => {
                    if std::time::Instant::now() >= deadline {
                        return Err(FuzzError::new(
                            FuzzErrorKind::Resource,
                            "artifact publication",
                            "publication lock remained busy for five seconds; no write started",
                        ));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
                Err(error) => return Err(io_error(error)),
            }
        }
        Ok(fd)
    }
    pub(super) fn marker(fd: &OwnedFd) -> Result<bool> {
        match fs::statat(fd, "published", AtFlags::SYMLINK_NOFOLLOW) {
            Ok(stat) => {
                if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile
                    || stat.st_size != 0
                {
                    return Err(invalid("invalid publication marker"));
                }
                Ok(true)
            }
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(e) => Err(io_error(e)),
        }
    }
}

/// Immutable evidence bundles and separate mutable campaign corpus namespace.
/// The caller must create the run/target root (normally tmp/surgeist-fuzz/...).
/// Directory handles and no-follow opens provide containment even if names move.
#[cfg(unix)]
pub struct ArtifactStore {
    root: rustix::fd::OwnedFd,
    display_root: PathBuf,
    limits: StorageLimits,
}
#[cfg(unix)]
impl ArtifactStore {
    pub fn open(root: impl AsRef<Path>, limits: StorageLimits) -> Result<Self> {
        let fd = rooted::root(root.as_ref())?;
        Self::check_root_entries(&fd)?;
        let _lock = rooted::lock(&fd)?;
        Self::check_root_entries(&fd)?;
        for namespace in NAMESPACES {
            rooted::directory(&fd, namespace, true)?;
        }
        let store = Self {
            root: fd,
            display_root: root.as_ref().to_path_buf(),
            limits,
        };
        store.inventory()?;
        Ok(store)
    }
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.display_root
    }
    fn check_root_entries(root: &rustix::fd::OwnedFd) -> Result<()> {
        use rustix::fs::FileType;
        for (name, kind) in rooted::names(root, NAMESPACES.len() as u64 + 1)? {
            let valid = if name == "publication.lock" {
                kind == FileType::RegularFile
            } else {
                NAMESPACES.contains(&name.as_str()) && kind == FileType::Directory
            };
            if !valid {
                return Err(invalid(
                    "unexpected entry or symlink in owned artifact root",
                ));
            }
        }
        Ok(())
    }
    fn inventory(&self) -> Result<RecoveryReport> {
        use rustix::fs::FileType;
        Self::check_root_entries(&self.root)?;
        let mut report = RecoveryReport {
            published: Vec::new(),
            partial: Vec::new(),
            bytes: 0,
        };
        for namespace in NAMESPACES {
            let dir = rooted::directory(&self.root, namespace, false)?;
            for (name, kind) in rooted::names(&dir, self.limits.entries)? {
                if kind != FileType::Directory || Sha256Digest::from_text(&name).is_err() {
                    return Err(invalid("unexpected artifact bundle or symlink"));
                }
                if (report.published.len() + report.partial.len()) as u64 >= self.limits.entries {
                    return Err(resource("artifact count exceeds quota"));
                }
                let bundle = rooted::directory(&dir, &name, false)?;
                let files = rooted::names(&bundle, 3)?;
                if rooted::marker(&bundle)? {
                    let expected = if matches!(namespace, "failures" | "captures" | "corpus") {
                        3
                    } else {
                        2
                    };
                    if files.len() != expected
                        || !files.iter().any(|(name, _)| name == "metadata.json")
                        || (matches!(namespace, "failures" | "captures" | "corpus")
                            && !files.iter().any(|(name, _)| name == "input.bin"))
                    {
                        return Err(invalid("published marker has incomplete evidence"));
                    }
                }
                for (file, kind) in files {
                    if kind != FileType::RegularFile
                        || !matches!(file.as_str(), "input.bin" | "metadata.json" | "published")
                    {
                        return Err(invalid("unexpected artifact content or symlink"));
                    }
                    let limit = if file == "input.bin" {
                        self.limits.input_bytes
                    } else if file == "metadata.json" {
                        self.limits.metadata_bytes
                    } else {
                        0
                    };
                    let size =
                        rustix::fs::statat(&bundle, &file, rustix::fs::AtFlags::SYMLINK_NOFOLLOW)
                            .map_err(io_error)?
                            .st_size;
                    let size =
                        u64::try_from(size).map_err(|_| invalid("negative artifact file size"))?;
                    if size > limit {
                        return Err(resource("artifact file exceeds decoding quota"));
                    }
                    report.bytes = report
                        .bytes
                        .checked_add(size)
                        .ok_or_else(|| invalid("artifact byte count overflow"))?;
                    if report.bytes > self.limits.bytes {
                        return Err(resource("artifact bytes exceed quota"));
                    }
                }
                let relative =
                    RelativePath::new(format!("{namespace}/{name}")).map_err(io_error)?;
                if rooted::marker(&bundle)? {
                    report.published.push(relative);
                } else {
                    report.partial.push(relative);
                }
            }
        }
        Ok(report)
    }
    pub fn recovery_report(&self) -> Result<RecoveryReport> {
        let _lock = rooted::lock(&self.root)?;
        self.inventory()
    }
    fn publish(
        &self,
        namespace: &str,
        identity: Sha256Digest,
        metadata: &[u8],
        input: Option<&[u8]>,
    ) -> Result<RelativePath> {
        let _lock = rooted::lock(&self.root)?;
        if metadata.len() as u64 > self.limits.metadata_bytes
            || input.is_some_and(|i| i.len() as u64 > self.limits.input_bytes)
        {
            return Err(resource("publication exceeds per-file byte quota"));
        }
        let report = self.inventory()?;
        let relative = RelativePath::new(format!("{namespace}/{identity}")).map_err(io_error)?;
        let dir = rooted::directory(&self.root, namespace, false)?;
        if report.published.contains(&relative) {
            let bundle = rooted::directory(&dir, identity.as_str(), false)?;
            let existing_meta = rooted::read(&bundle, "metadata.json", self.limits.metadata_bytes)?;
            if existing_meta != metadata {
                return Err(invalid("duplicate identity has different metadata"));
            }
            if let Some(input) = input
                && rooted::read(&bundle, "input.bin", self.limits.input_bytes)? != input
            {
                return Err(invalid("duplicate identity has different bytes"));
            }
            return Ok(relative);
        }
        if report.partial.contains(&relative) {
            return Err(invalid(
                "interrupted bundle remains partial; original evidence not claimed",
            ));
        }
        let added = metadata.len() as u64 + input.map_or(0, |i| i.len() as u64);
        if (report.published.len() + report.partial.len()) as u64 >= self.limits.entries
            || report
                .bytes
                .checked_add(added)
                .is_none_or(|sum| sum > self.limits.bytes)
        {
            return Err(resource(
                "artifact quota exhausted; original failures retained",
            ));
        }
        rustix::fs::mkdirat(
            &dir,
            identity.as_str(),
            rustix::fs::Mode::from_raw_mode(0o700),
        )
        .map_err(io_error)?;
        let bundle = rooted::directory(&dir, identity.as_str(), false)?;
        if let Some(input) = input {
            rooted::write(&bundle, "input.bin", input)?;
        }
        rooted::write(&bundle, "metadata.json", metadata)?;
        rustix::fs::fsync(&bundle).map_err(io_error)?;
        rooted::write(&bundle, "published", &[])?;
        rustix::fs::fsync(&bundle).map_err(io_error)?;
        rustix::fs::fsync(&dir).map_err(io_error)?;
        Ok(relative)
    }
    pub fn publish_failure(&self, artifact: &CaseArtifact) -> Result<RelativePath> {
        if artifact.metadata.outcome.failure_key().is_none() {
            return Err(invalid(
                "failure publication requires a failure observation",
            ));
        }
        let metadata = bounded_metadata(artifact.metadata(), self.limits.metadata_bytes)?;
        self.publish(
            "failures",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            Some(artifact.input()),
        )
    }
    /// Corpora deduplicate exact input only within compatible target/format/generator encoding.
    pub fn add_corpus(&self, artifact: &CaseArtifact) -> Result<RelativePath> {
        let target = artifact.metadata.target();
        let metadata = bounded_metadata(
            &CorpusMetadata::new(
                target,
                artifact.metadata.build.fixtures().cloned().collect(),
                artifact.input(),
            ),
            self.limits.metadata_bytes,
        )?;
        self.publish(
            "corpus",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            Some(artifact.input()),
        )
    }
    pub fn publish_run(&self, run: &RunRecord) -> Result<RelativePath> {
        for evidence in &run.failures {
            if let EvidenceStatus::Available { artifact } = evidence {
                if artifact.as_str().starts_with("captures/") {
                    let capture = self.load_capture(artifact)?;
                    if capture.metadata.target != run.target
                        || capture.metadata.build != run.build
                        || capture.metadata.config != run.config
                    {
                        return Err(invalid(
                            "run references capture from a different run/build/target",
                        ));
                    }
                } else {
                    let failure = self.load_failure(artifact)?;
                    if failure.metadata.target != run.target
                        || failure.metadata.build != run.build
                        || failure.metadata.config != run.config
                    {
                        return Err(invalid(
                            "run references failure from a different run/build/target",
                        ));
                    }
                }
            }
        }
        let metadata = bounded_metadata(run, self.limits.metadata_bytes)?;
        self.publish("runs", Sha256Digest::from_bytes(&metadata), &metadata, None)
    }
    pub fn publish_replay(&self, replay: &ReplayRecord) -> Result<RelativePath> {
        let metadata = bounded_metadata(replay, self.limits.metadata_bytes)?;
        self.publish(
            "comparisons",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            None,
        )
    }
    fn bundle(&self, path: &RelativePath, namespace: &str) -> Result<rustix::fd::OwnedFd> {
        let (prefix, id) = path
            .as_str()
            .split_once('/')
            .ok_or_else(|| invalid("invalid artifact reference"))?;
        if prefix != namespace || Sha256Digest::from_text(id).is_err() {
            return Err(invalid("wrong namespace or invalid artifact reference"));
        }
        let dir = rooted::directory(&self.root, prefix, false)?;
        let bundle = rooted::directory(&dir, id, false)?;
        if !rooted::marker(&bundle)? {
            return Err(invalid("publication interrupted; evidence unavailable"));
        }
        Ok(bundle)
    }
    pub fn load_failure(&self, path: &RelativePath) -> Result<CaseArtifact> {
        let bundle = self.bundle(path, "failures")?;
        let metadata_bytes = rooted::read(&bundle, "metadata.json", self.limits.metadata_bytes)?;
        let metadata: CaseMetadata = serde_json::from_slice(&metadata_bytes).map_err(io_error)?;
        if path.as_str().rsplit('/').next()
            != Some(Sha256Digest::from_bytes(&metadata_bytes).as_str())
        {
            return Err(invalid("metadata identity mismatch"));
        }
        if metadata.input_bytes > self.limits.input_bytes {
            return Err(resource("input exceeds storage decode quota"));
        }
        let input = rooted::read(&bundle, "input.bin", metadata.input_bytes)?;
        let artifact = CaseArtifact::from_parts(metadata, input)?;
        if artifact.metadata.outcome.failure_key().is_none() {
            return Err(invalid("failure bundle contains a nonfailure"));
        }
        Ok(artifact)
    }
    /// Load provider-free corpus cases. Fixture-dependent cases require an explicit build basis.
    pub fn load_corpus(&self, path: &RelativePath, target: &TargetDescriptor) -> Result<Vec<u8>> {
        self.load_corpus_with_providers(path, target, &[])
    }
    pub fn load_corpus_for_build(
        &self,
        path: &RelativePath,
        target: &TargetDescriptor,
        build: &BuildIdentity,
    ) -> Result<Vec<u8>> {
        if target
            .requirements()
            .features()
            .iter()
            .any(|f| !build.features.contains(f))
        {
            return Err(invalid("corpus build lacks mandatory features"));
        }
        self.load_corpus_with_providers(
            path,
            target,
            &build.fixtures().cloned().collect::<Vec<_>>(),
        )
    }
    fn load_corpus_with_providers(
        &self,
        path: &RelativePath,
        target: &TargetDescriptor,
        providers: &[ContentIdentity],
    ) -> Result<Vec<u8>> {
        let bundle = self.bundle(path, "corpus")?;
        let metadata_bytes = rooted::read(&bundle, "metadata.json", self.limits.metadata_bytes)?;
        let metadata: CorpusMetadata = serde_json::from_slice(&metadata_bytes).map_err(io_error)?;
        if metadata.schema != SCHEMA
            || metadata.target != *target.id()
            || metadata.format != *target.format()
            || metadata.generator_identity != *target.generator_identity()
            || metadata.encoding != target.encoding()
            || metadata.limits != target.limits()
            || metadata.providers != providers
            || metadata.input_bytes > target.limits().bytes()
            || path.as_str().rsplit('/').next()
                != Some(Sha256Digest::from_bytes(&metadata_bytes).as_str())
        {
            return Err(invalid("incompatible corpus identity/encoding"));
        }
        let bytes = rooted::read(
            &bundle,
            "input.bin",
            self.limits.input_bytes.min(metadata.input_bytes),
        )?;
        if bytes.len() as u64 != metadata.input_bytes
            || Sha256Digest::from_bytes(&bytes) != metadata.input_digest
        {
            return Err(invalid("corpus input mismatch"));
        }
        Ok(bytes)
    }
    fn load_metadata<T: for<'de> Deserialize<'de>>(
        &self,
        path: &RelativePath,
        namespace: &str,
    ) -> Result<T> {
        let bundle = self.bundle(path, namespace)?;
        let bytes = rooted::read(&bundle, "metadata.json", self.limits.metadata_bytes)?;
        if path.as_str().rsplit('/').next() != Some(Sha256Digest::from_bytes(&bytes).as_str()) {
            return Err(invalid("metadata identity mismatch"));
        }
        serde_json::from_slice(&bytes).map_err(io_error)
    }
    pub fn load_run(&self, path: &RelativePath) -> Result<RunRecord> {
        self.load_metadata(path, "runs")
    }
    pub fn load_replay(&self, path: &RelativePath) -> Result<ReplayRecord> {
        self.load_metadata(path, "comparisons")
    }
    /// Copies only exact input plus metadata into another explicitly caller-owned store.
    pub fn export_failure(
        &self,
        path: &RelativePath,
        destination: &ArtifactStore,
    ) -> Result<RelativePath> {
        destination.publish_failure(&self.load_failure(path)?)
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusMetadata {
    schema: u32,
    target: crate::TargetId,
    format: crate::CaseFormat,
    generator_identity: crate::GeneratorIdentity,
    encoding: crate::InputEncoding,
    limits: crate::InputLimits,
    providers: Vec<ContentIdentity>,
    input_digest: Sha256Digest,
    input_bytes: u64,
}
impl CorpusMetadata {
    fn new(target: &TargetDescriptor, providers: Vec<ContentIdentity>, input: &[u8]) -> Self {
        Self {
            schema: SCHEMA,
            target: target.id().clone(),
            format: target.format().clone(),
            generator_identity: target.generator_identity().clone(),
            encoding: target.encoding(),
            limits: target.limits(),
            providers,
            input_digest: Sha256Digest::from_bytes(input),
            input_bytes: input.len() as u64,
        }
    }
}

/// Supervisor facts are distinct from a semantic property failure. Unknown
/// panic/abort exits never acquire a guessed property or reduction failure key.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", from = "RawProcessStopReason")]
pub enum ProcessStopReason {
    Exited,
    Deadline,
    Cancelled,
    OutputLimit { stream: DiagnosticStream },
    InfrastructureFailure { operation: String, detail: String },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiagnosticStream {
    Stdout,
    Stderr,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", from = "RawProcessExitStatus")]
pub enum ProcessExitStatus {
    Code { code: i32 },
    Signal { signal: i32 },
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", from = "RawProcessCleanup")]
pub enum ProcessCleanup {
    Complete,
    Incomplete {
        child_reaped: bool,
        group_gone: bool,
        errors: Vec<String>,
    },
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum RawProcessStopReason {
    Exited {},
    Deadline {},
    Cancelled {},
    OutputLimit { stream: DiagnosticStream },
    InfrastructureFailure { operation: String, detail: String },
}
impl From<RawProcessStopReason> for ProcessStopReason {
    fn from(raw: RawProcessStopReason) -> Self {
        match raw {
            RawProcessStopReason::Exited {} => Self::Exited,
            RawProcessStopReason::Deadline {} => Self::Deadline,
            RawProcessStopReason::Cancelled {} => Self::Cancelled,
            RawProcessStopReason::OutputLimit { stream } => Self::OutputLimit { stream },
            RawProcessStopReason::InfrastructureFailure { operation, detail } => {
                Self::InfrastructureFailure { operation, detail }
            }
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
enum RawProcessExitStatus {
    Code { code: i32 },
    Signal { signal: i32 },
    Unavailable {},
}
impl From<RawProcessExitStatus> for ProcessExitStatus {
    fn from(raw: RawProcessExitStatus) -> Self {
        match raw {
            RawProcessExitStatus::Code { code } => Self::Code { code },
            RawProcessExitStatus::Signal { signal } => Self::Signal { signal },
            RawProcessExitStatus::Unavailable {} => Self::Unavailable,
        }
    }
}
#[derive(Deserialize)]
#[serde(tag = "state", rename_all = "kebab-case", deny_unknown_fields)]
enum RawProcessCleanup {
    Complete {},
    Incomplete {
        child_reaped: bool,
        group_gone: bool,
        errors: Vec<String>,
    },
}
impl From<RawProcessCleanup> for ProcessCleanup {
    fn from(raw: RawProcessCleanup) -> Self {
        match raw {
            RawProcessCleanup::Complete {} => Self::Complete,
            RawProcessCleanup::Incomplete {
                child_reaped,
                group_gone,
                errors,
            } => Self::Incomplete {
                child_reaped,
                group_gone,
                errors,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ProcessEvidenceSpec")]
pub struct ProcessEvidence {
    stop: ProcessStopReason,
    exit: ProcessExitStatus,
    diagnostics: String,
    diagnostics_truncated: bool,
    cleanup: ProcessCleanup,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProcessEvidenceSpec {
    pub stop: ProcessStopReason,
    pub exit: ProcessExitStatus,
    pub diagnostics: String,
    pub diagnostics_truncated: bool,
    pub cleanup: ProcessCleanup,
}
impl TryFrom<ProcessEvidenceSpec> for ProcessEvidence {
    type Error = FuzzError;
    fn try_from(s: ProcessEvidenceSpec) -> Result<Self> {
        Self::new(s)
    }
}
impl ProcessEvidence {
    pub fn new(s: ProcessEvidenceSpec) -> Result<Self> {
        let valid_exit = match s.exit {
            ProcessExitStatus::Code { code } => (0..=255).contains(&code),
            ProcessExitStatus::Signal { signal } => (1..=127).contains(&signal),
            ProcessExitStatus::Unavailable => s.stop != ProcessStopReason::Exited,
        };
        let valid_cleanup = match &s.cleanup {
            ProcessCleanup::Complete => true,
            ProcessCleanup::Incomplete { errors, .. } => {
                errors.len() <= 64 && errors.iter().all(|e| !e.is_empty() && e.len() <= 4096)
            }
        };
        if !valid_exit
            || !valid_cleanup
            || s.diagnostics.len() > 64 * 1024
            || matches!(&s.stop, ProcessStopReason::InfrastructureFailure { operation, detail } if !text(operation) || detail.is_empty() || detail.len() > 4096)
        {
            return Err(invalid("invalid bounded process evidence"));
        }
        Ok(Self {
            stop: s.stop,
            exit: s.exit,
            diagnostics: s.diagnostics,
            diagnostics_truncated: s.diagnostics_truncated,
            cleanup: s.cleanup,
        })
    }
    #[must_use]
    pub fn stop(&self) -> &ProcessStopReason {
        &self.stop
    }
    #[must_use]
    pub const fn exit(&self) -> ProcessExitStatus {
        self.exit
    }
    #[must_use]
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }
    #[must_use]
    pub const fn diagnostics_truncated(&self) -> bool {
        self.diagnostics_truncated
    }
    #[must_use]
    pub fn cleanup(&self) -> &ProcessCleanup {
        &self.cleanup
    }
    /// Preserve actual supervisor facts. Stderr is a bounded readable diagnostic;
    /// exact causative input stays in the separate binary capture artifact.
    #[cfg(feature = "execution")]
    pub fn from_outcome(outcome: &crate::execution::ProcessOutcome) -> Result<Self> {
        use crate::execution::{CleanupStatus, OutputStream, ProcessExit, ProcessStop};
        let stop = match &outcome.stop {
            ProcessStop::Exited => ProcessStopReason::Exited,
            ProcessStop::Deadline => ProcessStopReason::Deadline,
            ProcessStop::Cancelled => ProcessStopReason::Cancelled,
            ProcessStop::OutputLimit(stream) => ProcessStopReason::OutputLimit {
                stream: match stream {
                    OutputStream::Stdout => DiagnosticStream::Stdout,
                    OutputStream::Stderr => DiagnosticStream::Stderr,
                },
            },
            ProcessStop::InfrastructureFailure { operation, detail } => {
                ProcessStopReason::InfrastructureFailure {
                    operation: (*operation).to_owned(),
                    detail: detail.clone(),
                }
            }
        };
        let exit = match outcome.exit {
            ProcessExit::Success => ProcessExitStatus::Code { code: 0 },
            ProcessExit::Nonzero(code) => ProcessExitStatus::Code { code },
            ProcessExit::Signal(signal) => ProcessExitStatus::Signal { signal },
            ProcessExit::Unavailable => ProcessExitStatus::Unavailable,
        };
        let cleanup = match &outcome.cleanup {
            CleanupStatus::Complete => ProcessCleanup::Complete,
            CleanupStatus::Failed {
                child_reaped,
                group_gone,
                errors,
            } => ProcessCleanup::Incomplete {
                child_reaped: *child_reaped,
                group_gone: *group_gone,
                errors: errors.clone(),
            },
        };
        let mut diagnostics = String::from_utf8_lossy(
            &outcome.stderr.bytes[..outcome.stderr.bytes.len().min(32 * 1024)],
        )
        .into_owned();
        let encoded_truncated = diagnostics.len() > 64 * 1024;
        if encoded_truncated {
            let mut boundary = 64 * 1024;
            while !diagnostics.is_char_boundary(boundary) {
                boundary -= 1;
            }
            diagnostics.truncate(boundary);
        }
        Self::new(ProcessEvidenceSpec {
            stop,
            exit,
            diagnostics,
            diagnostics_truncated: outcome.stderr.truncated
                || outcome.stderr.bytes.len() > 32 * 1024
                || encoded_truncated,
            cleanup,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "CaptureMetadataSpec")]
pub struct CaptureMetadata {
    schema: u32,
    target: TargetDescriptor,
    build: BuildIdentity,
    config: RunConfig,
    input_digest: Sha256Digest,
    input_bytes: u64,
    process: ProcessEvidence,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMetadataSpec {
    pub schema: u32,
    pub target: TargetDescriptor,
    pub build: BuildIdentity,
    pub config: RunConfig,
    pub input_digest: Sha256Digest,
    pub input_bytes: u64,
    pub process: ProcessEvidence,
}
impl TryFrom<CaptureMetadataSpec> for CaptureMetadata {
    type Error = FuzzError;
    fn try_from(s: CaptureMetadataSpec) -> Result<Self> {
        if s.schema != SCHEMA
            || s.input_bytes > s.target.limits().bytes()
            || s.input_bytes > MAX_INPUT
            || s.target
                .requirements()
                .features()
                .iter()
                .any(|f| !s.build.features.contains(f))
        {
            return Err(invalid(
                "unsupported schema or inconsistent captured case basis",
            ));
        }
        Ok(Self {
            schema: s.schema,
            target: s.target,
            build: s.build,
            config: s.config,
            input_digest: s.input_digest,
            input_bytes: s.input_bytes,
            process: s.process,
        })
    }
}
impl CaptureMetadata {
    #[must_use]
    pub fn target(&self) -> &TargetDescriptor {
        &self.target
    }
    #[must_use]
    pub fn build(&self) -> &BuildIdentity {
        &self.build
    }
    #[must_use]
    pub fn config(&self) -> &RunConfig {
        &self.config
    }
    #[must_use]
    pub fn input_digest(&self) -> &Sha256Digest {
        &self.input_digest
    }
    #[must_use]
    pub const fn input_bytes(&self) -> u64 {
        self.input_bytes
    }
    #[must_use]
    pub fn process(&self) -> &ProcessEvidence {
        &self.process
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CaptureArtifact {
    metadata: CaptureMetadata,
    input: Vec<u8>,
}
impl CaptureArtifact {
    pub fn new(
        target: TargetDescriptor,
        build: BuildIdentity,
        config: RunConfig,
        input: Vec<u8>,
        process: ProcessEvidence,
    ) -> Result<Self> {
        let metadata = CaptureMetadata::try_from(CaptureMetadataSpec {
            schema: SCHEMA,
            target,
            build,
            config,
            input_digest: Sha256Digest::from_bytes(&input),
            input_bytes: input.len() as u64,
            process,
        })?;
        Self::from_parts(metadata, input)
    }
    pub fn from_parts(metadata: CaptureMetadata, input: Vec<u8>) -> Result<Self> {
        if metadata.input_bytes != input.len() as u64
            || metadata.input_digest != Sha256Digest::from_bytes(&input)
        {
            return Err(invalid("captured input digest/length mismatch"));
        }
        Ok(Self { metadata, input })
    }
    #[must_use]
    pub fn metadata(&self) -> &CaptureMetadata {
        &self.metadata
    }
    #[must_use]
    pub fn input(&self) -> &[u8] {
        &self.input
    }
    pub fn validate_replay(
        &self,
        target: &TargetDescriptor,
        build: &BuildIdentity,
        mode: ReplayMode,
    ) -> Result<()> {
        validate_replay_identity(
            target,
            &self.metadata.target,
            &self.metadata.build,
            build,
            mode,
        )
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "CaptureReplayRecordSpec")]
pub struct CaptureReplayRecord {
    schema: u32,
    target: TargetDescriptor,
    original_input: Sha256Digest,
    original_build: BuildIdentity,
    selected_build: BuildIdentity,
    comparison: bool,
    process: ProcessEvidence,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CaptureReplayRecordSpec {
    schema: u32,
    target: TargetDescriptor,
    original_input: Sha256Digest,
    original_build: BuildIdentity,
    selected_build: BuildIdentity,
    comparison: bool,
    process: ProcessEvidence,
}
impl TryFrom<CaptureReplayRecordSpec> for CaptureReplayRecord {
    type Error = FuzzError;
    fn try_from(s: CaptureReplayRecordSpec) -> Result<Self> {
        validate_replay_identity(
            &s.target,
            &s.target,
            &s.original_build,
            &s.selected_build,
            if s.comparison {
                ReplayMode::Comparison
            } else {
                ReplayMode::Exact
            },
        )?;
        if s.schema != SCHEMA {
            return Err(invalid("unsupported capture replay schema"));
        }
        Ok(Self {
            schema: s.schema,
            target: s.target,
            original_input: s.original_input,
            original_build: s.original_build,
            selected_build: s.selected_build,
            comparison: s.comparison,
            process: s.process,
        })
    }
}
impl CaptureReplayRecord {
    #[must_use]
    pub const fn is_comparison(&self) -> bool {
        self.comparison
    }
    #[must_use]
    pub fn process(&self) -> &ProcessEvidence {
        &self.process
    }
}
pub fn replay_capture_observation(
    original: &CaptureArtifact,
    descriptor: &TargetDescriptor,
    build: &BuildIdentity,
    mode: ReplayMode,
    process: ProcessEvidence,
) -> Result<CaptureReplayRecord> {
    original.validate_replay(descriptor, build, mode)?;
    Ok(CaptureReplayRecord {
        schema: SCHEMA,
        target: descriptor.clone(),
        original_input: original.metadata.input_digest.clone(),
        original_build: original.metadata.build.clone(),
        selected_build: build.clone(),
        comparison: mode == ReplayMode::Comparison,
        process,
    })
}
#[cfg(unix)]
impl ArtifactStore {
    pub fn publish_capture(&self, artifact: &CaptureArtifact) -> Result<RelativePath> {
        let metadata = bounded_metadata(artifact.metadata(), self.limits.metadata_bytes)?;
        self.publish(
            "captures",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            Some(artifact.input()),
        )
    }
    pub fn load_capture(&self, path: &RelativePath) -> Result<CaptureArtifact> {
        let bundle = self.bundle(path, "captures")?;
        let metadata: CaptureMetadata = self.load_metadata(path, "captures")?;
        let bytes = rooted::read(
            &bundle,
            "input.bin",
            self.limits.input_bytes.min(metadata.input_bytes),
        )?;
        CaptureArtifact::from_parts(metadata, bytes)
    }
    pub fn publish_capture_replay(&self, record: &CaptureReplayRecord) -> Result<RelativePath> {
        let metadata = bounded_metadata(record, self.limits.metadata_bytes)?;
        self.publish(
            "capture-replays",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            None,
        )
    }
    pub fn load_capture_replay(&self, path: &RelativePath) -> Result<CaptureReplayRecord> {
        self.load_metadata(path, "capture-replays")
    }
    pub fn export_capture(
        &self,
        path: &RelativePath,
        destination: &ArtifactStore,
    ) -> Result<RelativePath> {
        destination.publish_capture(&self.load_capture(path)?)
    }
}

/// Compact, immutable replay facts; actual case bytes remain in linked artifacts.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawReductionEvidenceObservation")]
pub struct ReductionEvidenceObservation {
    phase: crate::reduction::ReductionPhase,
    input_digest: Sha256Digest,
    input_bytes: u64,
    outcome: CaseOutcome,
    output_bytes: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReductionEvidenceObservation {
    phase: crate::reduction::ReductionPhase,
    input_digest: Sha256Digest,
    input_bytes: u64,
    outcome: CaseOutcome,
    output_bytes: u64,
}
impl TryFrom<RawReductionEvidenceObservation> for ReductionEvidenceObservation {
    type Error = FuzzError;
    fn try_from(raw: RawReductionEvidenceObservation) -> Result<Self> {
        if raw.input_bytes > MAX_INPUT {
            return Err(invalid("reduction observation exceeds input ceiling"));
        }
        Ok(Self {
            phase: raw.phase,
            input_digest: raw.input_digest,
            input_bytes: raw.input_bytes,
            outcome: raw.outcome,
            output_bytes: raw.output_bytes,
        })
    }
}
impl ReductionEvidenceObservation {
    #[must_use]
    pub const fn phase(&self) -> crate::reduction::ReductionPhase {
        self.phase
    }
    #[must_use]
    pub fn input_digest(&self) -> &Sha256Digest {
        &self.input_digest
    }
    #[must_use]
    pub const fn input_bytes(&self) -> u64 {
        self.input_bytes
    }
    #[must_use]
    pub fn outcome(&self) -> &CaseOutcome {
        &self.outcome
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawReductionEvidenceStep")]
pub struct ReductionEvidenceStep {
    from: Sha256Digest,
    to: Sha256Digest,
    replay_count: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReductionEvidenceStep {
    from: Sha256Digest,
    to: Sha256Digest,
    replay_count: u64,
}
impl TryFrom<RawReductionEvidenceStep> for ReductionEvidenceStep {
    type Error = FuzzError;
    fn try_from(raw: RawReductionEvidenceStep) -> Result<Self> {
        if raw.from == raw.to || raw.replay_count == 0 {
            return Err(invalid("non-progressing reduction step"));
        }
        Ok(Self {
            from: raw.from,
            to: raw.to,
            replay_count: raw.replay_count,
        })
    }
}
impl ReductionEvidenceStep {
    #[must_use]
    pub fn from(&self) -> &Sha256Digest {
        &self.from
    }
    #[must_use]
    pub fn to(&self) -> &Sha256Digest {
        &self.to
    }
    #[must_use]
    pub const fn replay_count(&self) -> u64 {
        self.replay_count
    }
}
/// Exact reduction admission ceilings with integer duration units.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "RawReductionBudget")]
pub struct ReductionBudgetRecord {
    max_attempts: u64,
    duration_seconds: u64,
    duration_nanoseconds: u32,
    max_output_bytes: u64,
    confirmations: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawReductionBudget {
    max_attempts: u64,
    duration_seconds: u64,
    duration_nanoseconds: u32,
    max_output_bytes: u64,
    confirmations: u32,
}
impl TryFrom<RawReductionBudget> for ReductionBudgetRecord {
    type Error = FuzzError;
    fn try_from(raw: RawReductionBudget) -> Result<Self> {
        if raw.duration_nanoseconds >= 1_000_000_000 {
            return Err(invalid("noncanonical reduction duration"));
        }
        let limits = crate::reduction::ReductionLimits::new(
            raw.max_attempts,
            std::time::Duration::new(raw.duration_seconds, raw.duration_nanoseconds),
            raw.max_output_bytes,
            raw.confirmations,
        )?;
        Ok(Self::from_limits(limits))
    }
}
impl ReductionBudgetRecord {
    fn from_limits(limits: crate::reduction::ReductionLimits) -> Self {
        Self {
            max_attempts: limits.max_attempts(),
            duration_seconds: limits.max_duration().as_secs(),
            duration_nanoseconds: limits.max_duration().subsec_nanos(),
            max_output_bytes: limits.max_output_bytes(),
            confirmations: limits.confirmations(),
        }
    }
    #[must_use]
    pub const fn max_attempts(&self) -> u64 {
        self.max_attempts
    }
    #[must_use]
    pub fn duration(&self) -> std::time::Duration {
        std::time::Duration::new(self.duration_seconds, self.duration_nanoseconds)
    }
    #[must_use]
    pub const fn max_output_bytes(&self) -> u64 {
        self.max_output_bytes
    }
    #[must_use]
    pub const fn confirmations(&self) -> u32 {
        self.confirmations
    }
}

/// Persisted reduction evidence, validated intrinsically and against linked cases.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ReductionRecordSpec")]
pub struct ReductionRecord {
    schema: u32,
    original: RelativePath,
    reduced: Option<RelativePath>,
    expected: crate::FailureKey,
    original_config: RunConfig,
    selected_config: Option<RunConfig>,
    limits: ReductionBudgetRecord,
    original_digest: Sha256Digest,
    original_bytes: u64,
    best_digest: Sha256Digest,
    best_bytes: u64,
    canonical_digest: Option<Sha256Digest>,
    canonical_bytes: Option<u64>,
    status: crate::reduction::ReductionStatus,
    replay_count: u64,
    candidate_count: u64,
    proposal_output_bytes: u64,
    output_bytes: u64,
    original_confirmed: bool,
    final_confirmed: bool,
    observations: Vec<ReductionEvidenceObservation>,
    chain: Vec<ReductionEvidenceStep>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReductionRecordSpec {
    schema: u32,
    original: RelativePath,
    #[serde(deserialize_with = "present")]
    reduced: Option<RelativePath>,
    expected: crate::FailureKey,
    original_config: RunConfig,
    #[serde(deserialize_with = "present")]
    selected_config: Option<RunConfig>,
    limits: ReductionBudgetRecord,
    original_digest: Sha256Digest,
    original_bytes: u64,
    best_digest: Sha256Digest,
    best_bytes: u64,
    #[serde(deserialize_with = "present")]
    canonical_digest: Option<Sha256Digest>,
    #[serde(deserialize_with = "present")]
    canonical_bytes: Option<u64>,
    status: crate::reduction::ReductionStatus,
    replay_count: u64,
    candidate_count: u64,
    proposal_output_bytes: u64,
    output_bytes: u64,
    original_confirmed: bool,
    final_confirmed: bool,
    observations: Vec<ReductionEvidenceObservation>,
    chain: Vec<ReductionEvidenceStep>,
}
impl TryFrom<ReductionRecordSpec> for ReductionRecord {
    type Error = FuzzError;
    fn try_from(s: ReductionRecordSpec) -> Result<Self> {
        let record = Self {
            schema: s.schema,
            original: s.original,
            reduced: s.reduced,
            expected: s.expected,
            original_config: s.original_config,
            selected_config: s.selected_config,
            limits: s.limits,
            original_digest: s.original_digest,
            original_bytes: s.original_bytes,
            best_digest: s.best_digest,
            best_bytes: s.best_bytes,
            canonical_digest: s.canonical_digest,
            canonical_bytes: s.canonical_bytes,
            status: s.status,
            replay_count: s.replay_count,
            candidate_count: s.candidate_count,
            proposal_output_bytes: s.proposal_output_bytes,
            output_bytes: s.output_bytes,
            original_confirmed: s.original_confirmed,
            final_confirmed: s.final_confirmed,
            observations: s.observations,
            chain: s.chain,
        };
        record.validate()?;
        Ok(record)
    }
}
impl ReductionRecord {
    fn validate(&self) -> Result<()> {
        use crate::reduction::{ReductionPhase, ReductionStatus};
        let failure_link = |p: &RelativePath| {
            p.as_str()
                .strip_prefix("failures/")
                .is_some_and(|id| Sha256Digest::from_text(id).is_ok())
        };
        if self.schema != SCHEMA
            || !failure_link(&self.original)
            || self.reduced.as_ref().is_some_and(|p| !failure_link(p))
            || self.original_bytes > MAX_INPUT
            || self.best_bytes > self.original_bytes
            || self.canonical_digest.is_some() != self.canonical_bytes.is_some()
            || self.canonical_bytes.is_some_and(|n| n > MAX_INPUT)
            || (self.canonical_digest.is_some() || self.reduced.is_some() || self.final_confirmed)
                && !self.original_confirmed
            || (self.canonical_digest.is_some() || self.reduced.is_some()) && !self.final_confirmed
            || self.reduced.is_some() != self.selected_config.is_some()
            || self.replay_count > self.limits.max_attempts
            || self.candidate_count > self.limits.max_attempts
            || self.replay_count != self.observations.len() as u64
            || self.observations.len() > 100_000
            || self.chain.len() as u64 > self.candidate_count
            || self.chain.len() > 100_000
            || self.observations.iter().any(|o| o.input_bytes > MAX_INPUT)
            || self
                .observations
                .iter()
                .fold(self.proposal_output_bytes, |sum, o| {
                    sum.saturating_add(o.output_bytes)
                })
                != self.output_bytes
            || self.status == ReductionStatus::Complete
                && (!self.original_confirmed || !self.final_confirmed)
            || self.status == ReductionStatus::UnsupportedFailure
                && (self.replay_count != 0 || self.original_confirmed || self.final_confirmed)
        {
            return Err(invalid("inconsistent bounded reduction record"));
        }
        let confirmed = |phase, digest: &Sha256Digest, bytes| {
            let observations = self
                .observations
                .iter()
                .filter(|o| o.phase == phase && &o.input_digest == digest);
            let mut count = 0;
            for o in observations {
                if o.input_bytes != bytes || o.outcome.failure_key() != Some(&self.expected) {
                    return false;
                }
                count += 1;
            }
            count >= self.limits.confirmations
        };
        if self.original_confirmed
            && !confirmed(
                ReductionPhase::Original,
                &self.original_digest,
                self.original_bytes,
            )
            || self.final_confirmed
                && !confirmed(ReductionPhase::Final, &self.best_digest, self.best_bytes)
            || self
                .canonical_digest
                .as_ref()
                .zip(self.canonical_bytes)
                .is_some_and(|(digest, bytes)| {
                    (digest != &self.best_digest || bytes != self.best_bytes)
                        && !confirmed(ReductionPhase::Canonical, digest, bytes)
                })
        {
            return Err(invalid(
                "reduction confirmation flags disagree with observations",
            ));
        }
        let mut current = &self.original_digest;
        let mut prior_count = 0;
        for step in &self.chain {
            if &step.from != current
                || step.from == step.to
                || step.replay_count <= prior_count
                || step.replay_count > self.replay_count
            {
                return Err(invalid("broken accepted reduction chain"));
            }
            let end =
                usize::try_from(step.replay_count).map_err(|_| invalid("replay index overflow"))?;
            let confirmations = self.observations[..end]
                .iter()
                .rev()
                .take_while(|o| {
                    o.phase == ReductionPhase::Candidate
                        && o.input_digest == step.to
                        && o.outcome.failure_key() == Some(&self.expected)
                })
                .count();
            if (confirmations as u64) < u64::from(self.limits.confirmations) {
                return Err(invalid(
                    "accepted reduction step lacks repeated exact-key observations",
                ));
            }
            current = &step.to;
            prior_count = step.replay_count;
        }
        if current != &self.best_digest {
            return Err(invalid("accepted chain does not reach recorded best input"));
        }
        Ok(())
    }
    #[must_use]
    pub fn original_config(&self) -> &RunConfig {
        &self.original_config
    }
    #[must_use]
    pub fn selected_config(&self) -> Option<&RunConfig> {
        self.selected_config.as_ref()
    }
    #[must_use]
    pub const fn limits(&self) -> &ReductionBudgetRecord {
        &self.limits
    }
    #[must_use]
    pub fn original(&self) -> &RelativePath {
        &self.original
    }
    #[must_use]
    pub fn reduced(&self) -> Option<&RelativePath> {
        self.reduced.as_ref()
    }
    #[must_use]
    pub fn expected(&self) -> &crate::FailureKey {
        &self.expected
    }
    #[must_use]
    pub const fn status(&self) -> crate::reduction::ReductionStatus {
        self.status
    }
    #[must_use]
    pub const fn replay_count(&self) -> u64 {
        self.replay_count
    }
    #[must_use]
    pub const fn original_confirmed(&self) -> bool {
        self.original_confirmed
    }
    #[must_use]
    pub const fn final_confirmed(&self) -> bool {
        self.final_confirmed
    }
    #[must_use]
    pub fn best_digest(&self) -> &Sha256Digest {
        &self.best_digest
    }
    #[must_use]
    pub fn observations(&self) -> &[ReductionEvidenceObservation] {
        &self.observations
    }
    #[must_use]
    pub fn chain(&self) -> &[ReductionEvidenceStep] {
        &self.chain
    }
}
#[cfg(unix)]
impl ArtifactStore {
    fn validate_reduction_links(&self, record: &ReductionRecord) -> Result<()> {
        let original = self.load_failure(&record.original)?;
        if original.metadata.input_digest != record.original_digest
            || original.metadata.input_bytes != record.original_bytes
            || original.metadata.config != record.original_config
            || original.metadata.outcome.failure_key() != Some(&record.expected)
        {
            return Err(invalid(
                "reduction original does not match selected input/failure key",
            ));
        }
        for observation in &record.observations {
            original
                .metadata
                .target
                .validate_outcome(&observation.outcome)?;
            if observation.input_bytes > original.metadata.target.limits().bytes() {
                return Err(invalid("reduction observation exceeds owner limits"));
            }
        }
        if let Some(path) = &record.reduced {
            let reduced = self.load_failure(path)?;
            let matches_best = reduced.metadata.input_digest == record.best_digest
                && reduced.metadata.input_bytes == record.best_bytes;
            let matches_canonical = record
                .canonical_digest
                .as_ref()
                .zip(record.canonical_bytes)
                .is_some_and(|(digest, bytes)| {
                    &reduced.metadata.input_digest == digest
                        && reduced.metadata.input_bytes == bytes
                });
            if !record.final_confirmed
                || (!matches_best && !matches_canonical)
                || reduced.metadata.outcome.failure_key() != Some(&record.expected)
                || reduced.metadata.target != original.metadata.target
                || reduced.metadata.build != original.metadata.build
                || Some(&reduced.metadata.config) != record.selected_config.as_ref()
            {
                return Err(invalid(
                    "reduced artifact does not match verified case/failure/build basis",
                ));
            }
        }
        Ok(())
    }
    pub fn publish_reduction(
        &self,
        original_path: &RelativePath,
        reduced_path: Option<&RelativePath>,
        report: &crate::reduction::ReductionReport,
    ) -> Result<RelativePath> {
        if report.observations().len() > 100_000 || report.chain().len() > 100_000 {
            return Err(invalid("reduction record count exceeds quota"));
        }
        let original = self.load_failure(original_path)?;
        if original.input() != report.original_input()
            || original.metadata.outcome.failure_key() != Some(report.expected())
        {
            return Err(invalid(
                "reduction report original bytes/failure key mismatch",
            ));
        }
        if let Some(path) = reduced_path {
            let reduced = self.load_failure(path)?;
            if reduced.input() != report.best_input()
                && report.canonical_input() != Some(reduced.input())
            {
                return Err(invalid(
                    "reduced artifact bytes differ from verified report input",
                ));
            }
        }
        let record = ReductionRecord {
            schema: SCHEMA,
            original: original_path.clone(),
            reduced: reduced_path.cloned(),
            expected: report.expected().clone(),
            original_config: original.metadata.config.clone(),
            selected_config: reduced_path
                .map(|path| {
                    self.load_failure(path)
                        .map(|artifact| artifact.metadata.config)
                })
                .transpose()?,
            limits: ReductionBudgetRecord::from_limits(report.limits()),
            original_digest: Sha256Digest::from_bytes(report.original_input()),
            original_bytes: report.original_input().len() as u64,
            best_digest: Sha256Digest::from_bytes(report.best_input()),
            best_bytes: report.best_input().len() as u64,
            canonical_digest: report.canonical_input().map(Sha256Digest::from_bytes),
            canonical_bytes: report.canonical_input().map(|bytes| bytes.len() as u64),
            status: report.status(),
            replay_count: report.replay_count(),
            candidate_count: report.candidate_count(),
            proposal_output_bytes: report.proposal_output_bytes(),
            output_bytes: report.output_bytes(),
            original_confirmed: report.original_confirmed(),
            final_confirmed: report.final_confirmed(),
            observations: report
                .observations()
                .iter()
                .map(|o| ReductionEvidenceObservation {
                    phase: o.phase,
                    input_digest: o.input_digest.clone(),
                    input_bytes: o.input_bytes as u64,
                    outcome: o.outcome.clone(),
                    output_bytes: o.output_bytes,
                })
                .collect(),
            chain: report
                .chain()
                .iter()
                .map(|step| ReductionEvidenceStep {
                    from: step.from.clone(),
                    to: step.to.clone(),
                    replay_count: step.replay_count,
                })
                .collect(),
        };
        record.validate()?;
        self.validate_reduction_links(&record)?;
        let metadata = bounded_metadata(&record, self.limits.metadata_bytes)?;
        self.publish(
            "reductions",
            Sha256Digest::from_bytes(&metadata),
            &metadata,
            None,
        )
    }
    pub fn load_reduction(&self, path: &RelativePath) -> Result<ReductionRecord> {
        let record = self.load_metadata(path, "reductions")?;
        self.validate_reduction_links(&record)?;
        Ok(record)
    }
}
