use std::{
    error::Error,
    fmt,
    io::{self, Read, Write},
};

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use surgeist_fuzz::{
    FuzzError, RelativePath, SourceRevision,
    worker::{read_frame, write_frame},
};

/// An exact source and runnable URL, including the unchanged variant suffix.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "FixtureFields")]
pub struct FixtureId {
    revision: SourceRevision,
    path: RelativePath,
    url: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureFields {
    revision: SourceRevision,
    path: RelativePath,
    url: String,
}

impl TryFrom<FixtureFields> for FixtureId {
    type Error = FixtureUrlError;

    fn try_from(fields: FixtureFields) -> Result<Self, Self::Error> {
        Self::new(fields.revision, fields.path, fields.url)
    }
}

impl FixtureId {
    /// Validate the runnable path and WPT variant syntax. The fixture inventory
    /// supplies the source-to-URL association and verifies the source bytes.
    pub fn new(
        revision: SourceRevision,
        path: RelativePath,
        url: String,
    ) -> Result<Self, FixtureUrlError> {
        let split = url.find(['?', '#']).unwrap_or(url.len());
        let (url_path, variant) = url.split_at(split);
        let relative = url_path.strip_prefix('/').ok_or(FixtureUrlError::Path)?;
        RelativePath::new(relative).map_err(|_| FixtureUrlError::Path)?;
        if url_path.chars().any(|scalar| scalar.is_control()) {
            return Err(FixtureUrlError::Path);
        }
        // The pinned WPT manifest accepts an absent variant or a nonempty
        // query or fragment, with no empty query before a fragment.
        if variant.len() == 1 || variant.starts_with("?#") || variant.chars().any(char::is_control)
        {
            return Err(FixtureUrlError::Variant);
        }
        Ok(Self {
            revision,
            path,
            url,
        })
    }

    pub fn revision(&self) -> &SourceRevision {
        &self.revision
    }

    pub fn path(&self) -> &RelativePath {
        &self.path
    }

    pub fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureUrlError {
    Path,
    Variant,
}

impl fmt::Display for FixtureUrlError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Path => "invalid WPT runnable URL path",
            Self::Variant => "invalid WPT variant suffix",
        })
    }
}

impl Error for FixtureUrlError {}

/// Exact UTF-16 guest diagnostics, including isolated surrogates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuestDiagnostic {
    pub message: Option<Vec<u16>>,
    pub stack: Option<Vec<u16>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum HarnessStatus {
    #[serde(rename = "OK")]
    Ok,
    Error,
    Timeout,
    PreconditionFailed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubtestStatus {
    Pass,
    Fail,
    Timeout,
    #[serde(rename = "NOTRUN")]
    NotRun,
    PreconditionFailed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Subtest {
    pub index: u32,
    pub name: Vec<u16>,
    pub status: SubtestStatus,
    pub diagnostic: GuestDiagnostic,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HarnessResult {
    pub status: HarnessStatus,
    pub diagnostic: GuestDiagnostic,
    /// Ordered as observed at harness completion, including repeated names.
    pub subtests: Vec<Subtest>,
}

/// A worker-observed failure with no qualified harness completion.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum WorkerFailure {
    GuestSyntax(GuestDiagnostic),
    GuestRuntime(GuestDiagnostic),
    NativeRefusal {
        owner: String,
        code: String,
        diagnostic: GuestDiagnostic,
    },
    UnavailableCapability {
        capability: String,
        diagnostic: GuestDiagnostic,
    },
    EngineIncompatibility {
        evidence: String,
        diagnostic: GuestDiagnostic,
    },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum WorkerResult {
    Harness(HarnessResult),
    Failure(WorkerFailure),
}

/// Resource observations carried by the worker before it relinquishes custody.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkerResources {
    pub native: NativeResourceEvidence,
    pub engine: EngineResourceEvidence,
}

/// Root accounting of the one Runtime guest attachment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub enum NativeResourceEvidence {
    /// No guest Ledger was published through an attachment. Preparation may
    /// have attempted work before returning a refusal.
    NotAttached {},
    /// An attachment existed but could not supply its accounting observation.
    Unavailable {},
    Measured(NativeLedgerUsage),
}

/// Inclusive Runtime adapter work and retained backing for one attachment.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "NativeLedgerFields")]
pub struct NativeLedgerUsage {
    adapter_work: u64,
    live_backing_bytes: u64,
    peak_backing_bytes: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeLedgerFields {
    adapter_work: u64,
    live_backing_bytes: u64,
    peak_backing_bytes: u64,
}

impl TryFrom<NativeLedgerFields> for NativeLedgerUsage {
    type Error = &'static str;

    fn try_from(fields: NativeLedgerFields) -> Result<Self, Self::Error> {
        Self::new(
            fields.adapter_work,
            fields.live_backing_bytes,
            fields.peak_backing_bytes,
        )
        .ok_or("native live backing exceeds its peak")
    }
}

impl NativeLedgerUsage {
    /// Create a report whose live backing is within the observed peak.
    #[must_use]
    pub const fn new(adapter_work: u64, live: u64, peak: u64) -> Option<Self> {
        if live > peak {
            return None;
        }
        Some(Self {
            adapter_work,
            live_backing_bytes: live,
            peak_backing_bytes: peak,
        })
    }

    pub const fn adapter_work(&self) -> u64 {
        self.adapter_work
    }

    pub const fn live_backing_bytes(&self) -> u64 {
        self.live_backing_bytes
    }

    pub const fn peak_backing_bytes(&self) -> u64 {
        self.peak_backing_bytes
    }
}

/// Engine heap policy and its optional point sample during execution ownership.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case", deny_unknown_fields)]
pub enum EngineResourceEvidence {
    /// No engine policy was selected for this result.
    Unavailable {},
    /// V8's default policy was selected. Refusal may preclude an observation.
    V8Default { sample: Option<HeapSample> },
    /// Boa's thread-local collector was selected. Refusal may preclude a sample.
    BoaCollector { sample: Option<BoaHeapSample> },
}

/// Used bytes and the heap limit reported by V8.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeapSample {
    pub used_bytes: u64,
    pub limit_bytes: u64,
}

/// Bytes in Boa's shallow, thread-local GC and ephemeron box accounting.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoaHeapSample {
    pub used_bytes: u64,
}

/// Complete worker evidence, subject to supervisor qualification.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkerReport {
    pub result: WorkerResult,
    pub resources: WorkerResources,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
enum Version {
    V2,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request<T> {
    version: Version,
    fixture: T,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Response<T, R, M> {
    version: Version,
    fixture: T,
    result: R,
    resources: M,
}

#[derive(Debug)]
pub enum ProtocolError {
    FrameLimit,
    Frame(FuzzError),
    MissingFrame,
    TrailingData,
    ReadEof(io::Error),
    FixtureMismatch,
    IncompleteInput { expected: usize, written: usize },
    IncompleteOutput,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameLimit => formatter.write_str("frame needs a header and a positive payload"),
            Self::Frame(error) => write!(formatter, "worker frame: {error}"),
            Self::MissingFrame => formatter.write_str("worker frame is absent"),
            Self::TrailingData => formatter.write_str("data follows the worker frame"),
            Self::ReadEof(error) => write!(formatter, "checking worker stream EOF: {error}"),
            Self::FixtureMismatch => formatter.write_str("worker replied for a different fixture"),
            Self::IncompleteInput { expected, written } => {
                write!(
                    formatter,
                    "worker input transferred {written} of {expected} bytes"
                )
            }
            Self::IncompleteOutput => formatter.write_str("worker stdout was incomplete"),
        }
    }
}

impl Error for ProtocolError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Frame(error) => Some(error),
            Self::ReadEof(error) => Some(error),
            _ => None,
        }
    }
}

pub(super) fn payload_limit(bytes: usize) -> Result<u32, ProtocolError> {
    let capacity = bytes
        .checked_sub(size_of::<u32>())
        .filter(|capacity| *capacity > 0)
        .ok_or(ProtocolError::FrameLimit)?;
    Ok(u32::try_from(capacity).unwrap_or(u32::MAX))
}

fn read_one<T: DeserializeOwned>(
    reader: &mut impl Read,
    max_payload_bytes: u32,
) -> Result<T, ProtocolError> {
    let value = read_frame(reader, max_payload_bytes)
        .map_err(ProtocolError::Frame)?
        .ok_or(ProtocolError::MissingFrame)?;
    let mut extra = [0];
    loop {
        match reader.read(&mut extra) {
            Ok(0) => return Ok(value),
            Ok(_) => return Err(ProtocolError::TrailingData),
            Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
            Err(error) => return Err(ProtocolError::ReadEof(error)),
        }
    }
}

pub(super) fn write_request(
    writer: &mut impl Write,
    fixture: &FixtureId,
    max_payload_bytes: u32,
) -> Result<(), ProtocolError> {
    write_frame(
        writer,
        &Request {
            version: Version::V2,
            fixture,
        },
        max_payload_bytes,
    )
    .map(|_| ())
    .map_err(ProtocolError::Frame)
}

/// Read one validated worker request followed by EOF.
pub fn read_request(
    reader: &mut impl Read,
    max_payload_bytes: u32,
) -> Result<FixtureId, ProtocolError> {
    let request: Request<FixtureId> = read_one(reader, max_payload_bytes)?;
    Ok(request.fixture)
}

/// Write one terminal response. The worker exits successfully after writing it.
pub fn write_report(
    writer: &mut impl Write,
    fixture: &FixtureId,
    report: &WorkerReport,
    max_payload_bytes: u32,
) -> Result<(), ProtocolError> {
    write_frame(
        writer,
        &Response {
            version: Version::V2,
            fixture,
            result: &report.result,
            resources: &report.resources,
        },
        max_payload_bytes,
    )
    .map(|_| ())
    .map_err(ProtocolError::Frame)
}

pub(super) fn read_report(
    reader: &mut impl Read,
    fixture: &FixtureId,
    max_payload_bytes: u32,
) -> Result<WorkerReport, ProtocolError> {
    let response: Response<FixtureId, WorkerResult, WorkerResources> =
        read_one(reader, max_payload_bytes)?;
    if response.fixture != *fixture {
        return Err(ProtocolError::FixtureMismatch);
    }
    Ok(WorkerReport {
        result: response.result,
        resources: response.resources,
    })
}
