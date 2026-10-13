//! Bounded libFuzzer proposals. Owner replay decides semantic equivalence.
#![forbid(unsafe_code)]

use crate::execution::{self, ProcessOutcome, WorkerLimits, WorkerSpec};
use crate::{FuzzError, FuzzErrorKind, Result, Sha256Digest};
use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
};

static NEXT: AtomicU64 = AtomicU64::new(0);

/// Admission bounds for copied entropy inputs and proposed outputs.
#[derive(Clone, Copy, Debug)]
pub struct ProposalLimits {
    attempts: u64,
    files: u64,
    input_bytes: u64,
    total_bytes: u64,
    case_timeout: std::time::Duration,
    engine_memory_bytes: u64,
}
impl ProposalLimits {
    pub fn new(attempts: u64, files: u64, input_bytes: u64, total_bytes: u64) -> Result<Self> {
        if attempts == 0
            || attempts > 1_000_000
            || files == 0
            || input_bytes == 0
            || input_bytes > 128 * 1024
            || total_bytes < input_bytes
        {
            return Err(error("positive finite proposal limits required"));
        }
        Ok(Self {
            attempts,
            files,
            input_bytes,
            total_bytes,
            case_timeout: std::time::Duration::from_secs(1),
            engine_memory_bytes: 0,
        })
    }
    /// Select the engine's coarse RSS guard. This is not an OS memory limit.
    /// The default zero disables this guard; the caller's worker policy remains explicit.
    pub fn with_engine_memory_bytes(mut self, bytes: u64) -> Result<Self> {
        if bytes == 0 {
            return Err(error("engine memory guard must be positive"));
        }
        self.engine_memory_bytes = bytes;
        Ok(self)
    }
    pub fn with_case_timeout(mut self, timeout: std::time::Duration) -> Result<Self> {
        if timeout.is_zero() || timeout.subsec_nanos() != 0 || timeout.as_secs() > 600 {
            return Err(error(
                "engine case timeout must be 1 through 600 whole seconds",
            ));
        }
        self.case_timeout = timeout;
        Ok(self)
    }
}
/// One byte-preserving engine input, distinct from a structured canonical case.
#[derive(Clone, Debug)]
pub struct EntropyInput {
    bytes: Vec<u8>,
    digest: Sha256Digest,
}
impl EntropyInput {
    fn new(bytes: Vec<u8>) -> Self {
        Self {
            digest: Sha256Digest::from_bytes(&bytes),
            bytes,
        }
    }
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn digest(&self) -> &Sha256Digest {
        &self.digest
    }
}
/// Engine exit and cleanup remain visible when no valid proposal was produced.
#[derive(Debug)]
pub struct EngineProposal {
    directory: PathBuf,
    originals: Vec<EntropyInput>,
    proposed: Vec<EntropyInput>,
    process: ProcessOutcome,
    readback_error: Option<String>,
}
impl EngineProposal {
    pub fn directory(&self) -> &Path {
        &self.directory
    }
    pub fn originals(&self) -> &[EntropyInput] {
        &self.originals
    }
    pub fn proposed(&self) -> &[EntropyInput] {
        &self.proposed
    }
    pub fn process(&self) -> &ProcessOutcome {
        &self.process
    }
    pub fn readback_error(&self) -> Option<&str> {
        self.readback_error.as_deref()
    }
}
/// Consumes a prepared instrumented executable; never builds or installs tools.
pub struct EngineMinimizer {
    executable: PathBuf,
    workspace: PathBuf,
    worker: WorkerLimits,
    limits: ProposalLimits,
}
impl EngineMinimizer {
    pub fn new(
        executable: PathBuf,
        workspace: PathBuf,
        worker: WorkerLimits,
        limits: ProposalLimits,
    ) -> Result<Self> {
        regular_path(&executable, false)?;
        regular_path(&workspace, true)?;
        Ok(Self {
            executable,
            workspace,
            worker,
            limits,
        })
    }
    /// The returned bytes require repeated ordinary owner replay against the
    /// selected failure key. A smaller input can trigger a different failure.
    pub fn failure_proposal(&self, entropy: &[u8], cancel: &AtomicBool) -> Result<EngineProposal> {
        if entropy.len() as u64 > self.limits.input_bytes {
            return Err(error("original entropy exceeds input ceiling"));
        }
        let directory = self.directory()?;
        let original = directory.join("original.entropy");
        write_new(&original, entropy)?;
        let output = directory.join("proposed.entropy");
        let mut output_argument = OsString::from("-exact_artifact_path=");
        output_argument.push(&output);
        let args = vec![
            "-minimize_crash=1".into(),
            format!("-runs={}", self.limits.attempts).into(),
            format!(
                "-rss_limit_mb={}",
                self.limits.engine_memory_bytes.div_ceil(1024 * 1024)
            )
            .into(),
            // The pinned minimizer sets its own maximum from the admitted
            // original. Supplying -max_len sets it twice and trips its assertion.
            format!("-timeout={}", self.limits.case_timeout.as_secs()).into(),
            format!("-max_total_time={}", self.worker.timeout().as_secs().max(1)).into(),
            output_argument,
            original.into_os_string(),
        ];
        let process = self.execute(&directory, args, cancel)?;
        let readback = if output.exists() {
            read(&output, self.limits.input_bytes).map(|bytes| vec![EntropyInput::new(bytes)])
        } else {
            Ok(Vec::new())
        };
        Ok(self.proposal(
            directory,
            vec![EntropyInput::new(entropy.to_vec())],
            process,
            readback,
        ))
    }
    /// Coverage minimization preserves the source corpus and all separate
    /// failure evidence. Proposed inputs are entropy, not semantic regressions.
    pub fn coverage_corpus_proposal(
        &self,
        source: &Path,
        cancel: &AtomicBool,
    ) -> Result<EngineProposal> {
        regular_path(source, true)?;
        let originals = self.read_directory(source)?;
        if originals.is_empty() {
            return Err(error("empty entropy corpus"));
        }
        if originals.len() as u64 >= self.limits.attempts {
            return Err(error(
                "engine attempt budget must admit every original input and startup input",
            ));
        }
        let directory = self.directory()?;
        let inputs = directory.join("originals");
        let output = directory.join("proposed");
        fs::create_dir(&inputs).map_err(io_error)?;
        fs::create_dir(&output).map_err(io_error)?;
        for input in &originals {
            write_new(&inputs.join(input.digest.as_str()), input.bytes())?;
        }
        let args = vec![
            "-merge=1".into(),
            format!("-runs={}", self.limits.attempts).into(),
            format!(
                "-rss_limit_mb={}",
                self.limits.engine_memory_bytes.div_ceil(1024 * 1024)
            )
            .into(),
            format!("-max_len={}", self.limits.input_bytes).into(),
            format!("-timeout={}", self.limits.case_timeout.as_secs()).into(),
            format!("-max_total_time={}", self.worker.timeout().as_secs().max(1)).into(),
            output.into_os_string(),
            inputs.into_os_string(),
        ];
        let process = self.execute(&directory, args, cancel)?;
        let readback = self.read_directory(&directory.join("proposed"));
        Ok(self.proposal(directory, originals, process, readback))
    }
    fn proposal(
        &self,
        directory: PathBuf,
        originals: Vec<EntropyInput>,
        process: ProcessOutcome,
        readback: Result<Vec<EntropyInput>>,
    ) -> EngineProposal {
        let (proposed, readback_error) = match readback {
            Ok(inputs) => (inputs, None),
            Err(error) => (Vec::new(), Some(error.to_string())),
        };
        EngineProposal {
            directory,
            originals,
            proposed,
            process,
            readback_error,
        }
    }
    fn directory(&self) -> Result<PathBuf> {
        let directory = self.workspace.join(format!(
            "engine-proposal-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&directory).map_err(io_error)?;
        Ok(directory)
    }
    fn execute(
        &self,
        directory: &Path,
        args: Vec<OsString>,
        cancel: &AtomicBool,
    ) -> Result<ProcessOutcome> {
        let mut environment = BTreeMap::new();
        environment.insert(
            "SURGEIST_FUZZ_CASE_LIMIT".into(),
            self.limits.attempts.to_string().into(),
        );
        environment.insert(
            "ASAN_OPTIONS".into(),
            "abort_on_error=1:detect_leaks=0".into(),
        );
        let spec = WorkerSpec::new(
            self.executable.clone(),
            directory.to_owned(),
            args,
            environment,
            Vec::new(),
            self.worker,
        )
        .map_err(|error| {
            crate::FuzzError::new(
                FuzzErrorKind::Process,
                "engine proposal invocation",
                error.to_string(),
            )
        })?;
        execution::execute(&spec, cancel).map_err(|error| {
            crate::FuzzError::new(
                FuzzErrorKind::Process,
                "engine proposal execution",
                error.to_string(),
            )
        })
    }
    fn read_directory(&self, path: &Path) -> Result<Vec<EntropyInput>> {
        let mut inputs = BTreeMap::new();
        let mut entries = 0_u64;
        let mut total = 0_u64;
        for entry in fs::read_dir(path).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            entries = entries
                .checked_add(1)
                .ok_or_else(|| error("entry count overflow"))?;
            if entries > self.limits.files {
                return Err(error("entropy entry quota exceeded"));
            }
            let bytes = read(&entry.path(), self.limits.input_bytes)?;
            total = total
                .checked_add(bytes.len() as u64)
                .ok_or_else(|| error("entropy byte count overflow"))?;
            if total > self.limits.total_bytes {
                return Err(error("entropy byte quota exceeded"));
            }
            let input = EntropyInput::new(bytes);
            inputs.entry(input.digest.clone()).or_insert(input);
        }
        Ok(inputs.into_values().collect())
    }
}
fn regular_path(path: &Path, directory: bool) -> Result<()> {
    if !path.is_absolute() {
        return Err(error("paths must be absolute"));
    }
    let mut part = PathBuf::new();
    for component in path.components() {
        if !matches!(component, Component::RootDir | Component::Normal(_)) {
            return Err(error("path traversal is unsupported"));
        }
        part.push(component);
        if fs::symlink_metadata(&part)
            .map_err(io_error)?
            .file_type()
            .is_symlink()
        {
            return Err(error("symlink path is unsupported"));
        }
    }
    let metadata = fs::symlink_metadata(path).map_err(io_error)?;
    if if directory {
        !metadata.is_dir()
    } else {
        !metadata.is_file()
    } {
        return Err(error("unexpected path type"));
    }
    Ok(())
}
fn read(path: &Path, ceiling: u64) -> Result<Vec<u8>> {
    regular_path(path, false)?;
    let mut bytes = Vec::new();
    let parent = path.parent().ok_or_else(|| error("input has no parent"))?;
    let root = crate::artifact::rooted::root(parent)?;
    let name = path
        .file_name()
        .ok_or_else(|| error("input has no filename"))?;
    let fd = rustix::fs::openat(
        &root,
        name,
        rustix::fs::OFlags::RDONLY
            | rustix::fs::OFlags::NOFOLLOW
            | rustix::fs::OFlags::NONBLOCK
            | rustix::fs::OFlags::CLOEXEC,
        rustix::fs::Mode::empty(),
    )
    .map_err(|e| io_error(e.into()))?;
    if rustix::fs::FileType::from_raw_mode(
        rustix::fs::fstat(&fd)
            .map_err(|e| io_error(e.into()))?
            .st_mode,
    ) != rustix::fs::FileType::RegularFile
    {
        return Err(error("input changed to a nonregular file"));
    }
    fs::File::from(fd)
        .take(ceiling + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > ceiling {
        return Err(error("entropy input ceiling exceeded"));
    }
    Ok(bytes)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    file.write_all(bytes).map_err(io_error)?;
    file.sync_all().map_err(io_error)
}
fn error(detail: &str) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "engine proposal", detail)
}
fn io_error(source: std::io::Error) -> FuzzError {
    FuzzError::with_source(FuzzErrorKind::Io, "engine proposal", "I/O failure", source)
}
