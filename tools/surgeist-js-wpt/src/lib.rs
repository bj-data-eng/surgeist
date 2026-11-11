//! Bounded transport for one conformance fixture variant per worker.
#![forbid(unsafe_code)]

mod protocol;

pub use protocol::{
    FixtureId, FixtureUrlError, GuestDiagnostic, HarnessResult, HarnessStatus, ProtocolError,
    Subtest, SubtestStatus, WorkerFailure, WorkerResult, read_request, write_report,
};
pub use surgeist_fuzz::execution::{
    CleanupStatus, ExecutionCapabilities, ExecutionError, MemoryPolicy, OutputStream, ProcessExit,
    ProcessOutcome, ProcessStop, WorkerLimits, execution_capabilities,
};
pub use surgeist_fuzz::{RelativePath, SourceRevision};

use std::{
    collections::BTreeMap, error::Error, ffi::OsString, fmt, path::PathBuf,
    sync::atomic::AtomicBool,
};
use surgeist_fuzz::execution::{WorkerSpec, execute};

/// A reply is exposed only after successful transport and verified cleanup.
#[derive(Debug)]
pub enum Report {
    Received(WorkerResult),
    ProtocolFailure(ProtocolError),
    TransportFailed,
}

/// The process observation remains available regardless of report admission.
#[derive(Debug)]
pub struct Run {
    pub fixture: FixtureId,
    pub process: ProcessOutcome,
    pub report: Report,
}

/// Failure before a worker could be observed.
#[derive(Debug)]
pub enum StartError {
    Request(ProtocolError),
    Process(ExecutionError),
}

impl fmt::Display for StartError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Request(error) => write!(formatter, "worker request: {error}"),
            Self::Process(error) => write!(formatter, "worker start: {error}"),
        }
    }
}

impl Error for StartError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Request(error) => Some(error),
            Self::Process(error) => Some(error),
        }
    }
}

/// Encode one fixture request, supervise its worker, and qualify its response.
/// The process input and stdout limits include the four-byte frame headers.
pub fn run(
    fixture: &FixtureId,
    executable: PathBuf,
    cwd: PathBuf,
    args: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
    limits: WorkerLimits,
    cancellation: &AtomicBool,
) -> Result<Run, StartError> {
    let mut input = Vec::new();
    let request_limit =
        protocol::payload_limit(limits.input_bytes()).map_err(StartError::Request)?;
    let report_limit =
        protocol::payload_limit(limits.stdout_bytes()).map_err(StartError::Request)?;
    protocol::write_request(&mut input, fixture, request_limit).map_err(StartError::Request)?;
    let expected_input = input.len();
    let spec = WorkerSpec::new(executable, cwd, args, environment, input, limits)
        .map_err(StartError::Process)?;
    let process = execute(&spec, cancellation).map_err(StartError::Process)?;

    let report = if process.stop != ProcessStop::Exited
        || process.exit != ProcessExit::Success
        || process.cleanup != CleanupStatus::Complete
    {
        Report::TransportFailed
    } else if process.input_bytes_written != expected_input {
        Report::ProtocolFailure(ProtocolError::IncompleteInput {
            expected: expected_input,
            written: process.input_bytes_written,
        })
    } else if process.stdout.truncated || !process.stdout.eof {
        Report::ProtocolFailure(ProtocolError::IncompleteOutput)
    } else {
        match protocol::read_report(&mut process.stdout.bytes.as_slice(), fixture, report_limit) {
            Ok(result) => Report::Received(result),
            Err(error) => Report::ProtocolFailure(error),
        }
    };
    Ok(Run {
        fixture: fixture.clone(),
        process,
        report,
    })
}
