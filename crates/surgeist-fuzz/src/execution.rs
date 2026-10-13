//! One bounded, explicitly configured local worker, with owned process-group cleanup.
//!
//! This is lifetime hygiene, not containment: a descendant deliberately leaving the
//! group cannot be supervised. The initial backend supports Apple-Silicon macOS.
#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Memory admission policy. A hard ceiling is rejected when it cannot be enforced.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryPolicy {
    /// Caller explicitly accepts that this backend supplies no memory ceiling.
    Unbounded,
    /// Mandatory OS-enforced ceiling, in bytes; unsupported by the initial backend.
    RequireHardLimit(u64),
}

/// Actual host capabilities, rather than an inferred guarantee from a budget.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ExecutionCapabilities {
    pub supported_host: bool,
    pub process_groups: bool,
    pub hard_memory_limit: bool,
    pub sampled_rss: bool,
}

#[must_use]
pub const fn execution_capabilities() -> ExecutionCapabilities {
    let supported_host = cfg!(all(target_os = "macos", target_arch = "aarch64"));
    ExecutionCapabilities {
        supported_host,
        process_groups: supported_host,
        hard_memory_limit: false,
        sampled_rss: false,
    }
}

/// Validated ceilings and cleanup deadlines for one process invocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorkerLimits {
    input_bytes: usize,
    stdout_bytes: usize,
    stderr_bytes: usize,
    timeout: Duration,
    termination_grace: Duration,
    cleanup_timeout: Duration,
    memory: MemoryPolicy,
}

impl WorkerLimits {
    pub fn new(
        input_bytes: usize,
        stdout_bytes: usize,
        stderr_bytes: usize,
        timeout: Duration,
        termination_grace: Duration,
        cleanup_timeout: Duration,
        memory: MemoryPolicy,
    ) -> Result<Self, ExecutionError> {
        if timeout.is_zero() || cleanup_timeout.is_zero() {
            return Err(ExecutionError::InvalidSpec(
                "worker and cleanup deadlines must be positive",
            ));
        }
        let now = Instant::now();
        if now.checked_add(timeout).is_none()
            || now
                .checked_add(termination_grace)
                .and_then(|v| v.checked_add(cleanup_timeout))
                .is_none()
            || matches!(memory, MemoryPolicy::RequireHardLimit(0))
        {
            return Err(ExecutionError::InvalidSpec(
                "unrepresentable deadline or zero mandatory memory ceiling",
            ));
        }
        Ok(Self {
            input_bytes,
            stdout_bytes,
            stderr_bytes,
            timeout,
            termination_grace,
            cleanup_timeout,
            memory,
        })
    }
    #[must_use]
    pub const fn input_bytes(self) -> usize {
        self.input_bytes
    }
    #[must_use]
    pub const fn stdout_bytes(self) -> usize {
        self.stdout_bytes
    }
    #[must_use]
    pub const fn stderr_bytes(self) -> usize {
        self.stderr_bytes
    }
    #[must_use]
    pub const fn timeout(self) -> Duration {
        self.timeout
    }
    #[must_use]
    pub const fn termination_grace(self) -> Duration {
        self.termination_grace
    }
    #[must_use]
    pub const fn cleanup_timeout(self) -> Duration {
        self.cleanup_timeout
    }
    #[must_use]
    pub const fn memory(self) -> MemoryPolicy {
        self.memory
    }
}

/// Immutable exact invocation. Input is captured here before any worker starts.
/// No shell parsing or inherited environment is used.
#[derive(Clone, Debug)]
pub struct WorkerSpec {
    executable: PathBuf,
    cwd: PathBuf,
    args: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
    input: Vec<u8>,
    limits: WorkerLimits,
}

impl WorkerSpec {
    pub fn new(
        executable: PathBuf,
        cwd: PathBuf,
        args: Vec<OsString>,
        environment: BTreeMap<OsString, OsString>,
        input: Vec<u8>,
        limits: WorkerLimits,
    ) -> Result<Self, ExecutionError> {
        if !executable.is_absolute() || !cwd.is_absolute() {
            return Err(ExecutionError::InvalidSpec(
                "executable and working directory must be absolute",
            ));
        }
        if input.len() > limits.input_bytes {
            return Err(ExecutionError::InputLimit {
                actual: input.len(),
                limit: limits.input_bytes,
            });
        }
        Ok(Self {
            executable,
            cwd,
            args,
            environment,
            input,
            limits,
        })
    }
    #[must_use]
    pub fn executable(&self) -> &Path {
        &self.executable
    }
    #[must_use]
    pub fn cwd(&self) -> &Path {
        &self.cwd
    }
    #[must_use]
    pub fn args(&self) -> &[OsString] {
        &self.args
    }
    #[must_use]
    pub fn environment(&self) -> &BTreeMap<OsString, OsString> {
        &self.environment
    }
    #[must_use]
    pub fn input(&self) -> &[u8] {
        &self.input
    }
    #[must_use]
    pub const fn limits(&self) -> WorkerLimits {
        self.limits
    }
}

/// Failures before spawn. After spawn, all failures retain cleanup evidence.
#[derive(Debug)]
pub enum ExecutionError {
    InvalidSpec(&'static str),
    InputLimit { actual: usize, limit: usize },
    UnsupportedHost,
    UnsupportedMemoryPolicy,
    CancelledBeforeSpawn,
    Spawn(io::Error),
}
impl fmt::Display for ExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSpec(detail) => write!(f, "invalid worker specification: {detail}"),
            Self::InputLimit { actual, limit } => {
                write!(f, "worker input {actual} exceeds limit {limit}")
            }
            Self::UnsupportedHost => f.write_str("worker execution requires Apple-Silicon macOS"),
            Self::UnsupportedMemoryPolicy => {
                f.write_str("mandatory hard memory limits are unsupported")
            }
            Self::CancelledBeforeSpawn => f.write_str("worker cancelled before spawn"),
            Self::Spawn(error) => write!(f, "worker spawn failed: {error}"),
        }
    }
}
impl std::error::Error for ExecutionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Spawn(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

/// Supervisor stop reason; a worker exit is classified separately.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProcessStop {
    Exited,
    Deadline,
    Cancelled,
    OutputLimit(OutputStream),
    InfrastructureFailure {
        operation: &'static str,
        detail: String,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProcessExit {
    Success,
    Nonzero(i32),
    Signal(i32),
    Unavailable,
}

/// Retained prefix plus honest truncation/stream completion evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapturedOutput {
    pub bytes: Vec<u8>,
    pub truncated: bool,
    pub eof: bool,
}

/// Cleanup success requires both child reaping and observed group disappearance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CleanupStatus {
    Complete,
    Failed {
        child_reaped: bool,
        group_gone: bool,
        errors: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProcessOutcome {
    pub stop: ProcessStop,
    pub exit: ProcessExit,
    pub stdout: CapturedOutput,
    pub stderr: CapturedOutput,
    pub input_bytes_written: usize,
    pub duration: Duration,
    pub cleanup: CleanupStatus,
    /// Group established by `Command::process_group(0)`, never a caller-provided ID.
    pub process_group: u32,
    /// No memory sample is available in the initial backend. Never a hard ceiling.
    pub sampled_peak_rss_bytes: Option<u64>,
}

/// Run one worker. Cancellation is polled during pipe transfer and waiting.
/// The returned input count is transport evidence, not evaluator acknowledgement.
/// A multi-case engine must establish its own acknowledged case-capture protocol.
/// The caller can persist `spec.input()` before calling this boundary.
pub fn execute(
    spec: &WorkerSpec,
    cancellation: &AtomicBool,
) -> Result<ProcessOutcome, ExecutionError> {
    if !execution_capabilities().supported_host {
        return Err(ExecutionError::UnsupportedHost);
    }
    if spec.limits.memory != MemoryPolicy::Unbounded {
        return Err(ExecutionError::UnsupportedMemoryPolicy);
    }
    if cancellation.load(Ordering::Acquire) {
        return Err(ExecutionError::CancelledBeforeSpawn);
    }
    #[cfg(unix)]
    {
        unix::execute(spec, cancellation)
    }
    #[cfg(not(unix))]
    {
        Err(ExecutionError::UnsupportedHost)
    }
}

#[cfg(unix)]
mod unix {
    use super::*;
    use rustix::fs::{OFlags, fcntl_getfl, fcntl_setfl};
    use rustix::process::{Pid, Signal, kill_process_group, test_kill_process_group};
    use std::io::{Read, Write};
    use std::os::fd::AsFd;
    use std::os::unix::process::{CommandExt, ExitStatusExt};
    use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};

    const POLL: Duration = Duration::from_millis(2);
    const CHUNK: usize = 8192;

    fn nonblocking(fd: &impl AsFd) -> io::Result<()> {
        let flags = fcntl_getfl(fd).map_err(io::Error::from)?;
        fcntl_setfl(fd, flags | OFlags::NONBLOCK).map_err(io::Error::from)
    }

    fn failed(operation: &'static str, error: io::Error) -> ProcessStop {
        ProcessStop::InfrastructureFailure {
            operation,
            detail: error.to_string(),
        }
    }

    struct Pipes {
        stdin: Option<ChildStdin>,
        stdout: Option<ChildStdout>,
        stderr: Option<ChildStderr>,
        out: CapturedOutput,
        err: CapturedOutput,
        written: usize,
    }
    impl Pipes {
        fn new(child: &mut Child) -> Self {
            Self {
                stdin: child.stdin.take(),
                stdout: child.stdout.take(),
                stderr: child.stderr.take(),
                out: CapturedOutput {
                    bytes: Vec::new(),
                    truncated: false,
                    eof: false,
                },
                err: CapturedOutput {
                    bytes: Vec::new(),
                    truncated: false,
                    eof: false,
                },
                written: 0,
            }
        }
        fn configure(&self) -> io::Result<()> {
            if let Some(pipe) = &self.stdin {
                nonblocking(pipe)?;
            }
            if let Some(pipe) = &self.stdout {
                nonblocking(pipe)?;
            }
            if let Some(pipe) = &self.stderr {
                nonblocking(pipe)?;
            }
            Ok(())
        }
        fn transfer(&mut self, spec: &WorkerSpec) -> Option<ProcessStop> {
            if self.written == spec.input.len() {
                self.stdin.take();
            }
            if let Some(pipe) = &mut self.stdin {
                let end = self.written.saturating_add(CHUNK).min(spec.input.len());
                match pipe.write(&spec.input[self.written..end]) {
                    Ok(0) => {
                        return Some(failed(
                            "write stdin",
                            io::Error::new(
                                io::ErrorKind::WriteZero,
                                "worker stdin made no progress",
                            ),
                        ));
                    }
                    Ok(count) => self.written += count,
                    Err(error) if error.kind() == io::ErrorKind::BrokenPipe => {
                        self.stdin.take();
                    }
                    Err(error)
                        if matches!(
                            error.kind(),
                            io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                        ) => {}
                    Err(error) => return Some(failed("write stdin", error)),
                }
            }
            self.drain(spec)
        }
        fn drain(&mut self, spec: &WorkerSpec) -> Option<ProcessStop> {
            // One chunk per stream per iteration preserves deadline/cancellation fairness.
            let stdout = read(
                &mut self.stdout,
                &mut self.out,
                spec.limits.stdout_bytes,
                OutputStream::Stdout,
            );
            let stderr = read(
                &mut self.stderr,
                &mut self.err,
                spec.limits.stderr_bytes,
                OutputStream::Stderr,
            );
            stdout.or(stderr)
        }
    }

    fn read<R: Read>(
        pipe: &mut Option<R>,
        capture: &mut CapturedOutput,
        limit: usize,
        stream: OutputStream,
    ) -> Option<ProcessStop> {
        let reader = pipe.as_mut()?;
        let mut chunk = [0_u8; CHUNK];
        match reader.read(&mut chunk) {
            Ok(0) => {
                capture.eof = true;
                pipe.take();
                None
            }
            Ok(count) => {
                let retained = count.min(limit.saturating_sub(capture.bytes.len()));
                capture.bytes.extend_from_slice(&chunk[..retained]);
                if retained < count {
                    capture.truncated = true;
                    Some(ProcessStop::OutputLimit(stream))
                } else {
                    None
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::WouldBlock | io::ErrorKind::Interrupted
                ) =>
            {
                None
            }
            Err(error) => {
                pipe.take();
                Some(failed("read output", error))
            }
        }
    }

    // ESRCH is the only positive disappearance evidence. Permission/other errors
    // retain the group as potentially present and become cleanup failures.
    fn group_exists(group: Pid) -> Result<bool, rustix::io::Errno> {
        match test_kill_process_group(group) {
            Ok(()) => Ok(true),
            Err(rustix::io::Errno::SRCH) => Ok(false),
            Err(error) => Err(error),
        }
    }
    fn signal(group: Pid, signal: Signal, errors: &mut Vec<String>) {
        if let Err(error) = kill_process_group(group, signal)
            && error != rustix::io::Errno::SRCH
        {
            errors.push(format!("signal {signal:?}: {error}"));
        }
    }
    fn status(status: Option<ExitStatus>) -> ProcessExit {
        match status {
            Some(value) if value.success() => ProcessExit::Success,
            Some(value) => value
                .code()
                .map(ProcessExit::Nonzero)
                .or_else(|| value.signal().map(ProcessExit::Signal))
                .unwrap_or(ProcessExit::Unavailable),
            None => ProcessExit::Unavailable,
        }
    }

    pub(super) fn execute(
        spec: &WorkerSpec,
        cancellation: &AtomicBool,
    ) -> Result<ProcessOutcome, ExecutionError> {
        execute_with_probe(spec, cancellation, group_exists)
    }

    fn execute_with_probe(
        spec: &WorkerSpec,
        cancellation: &AtomicBool,
        mut probe_group: impl FnMut(Pid) -> Result<bool, rustix::io::Errno>,
    ) -> Result<ProcessOutcome, ExecutionError> {
        let start = Instant::now();
        let deadline =
            start
                .checked_add(spec.limits.timeout)
                .ok_or(ExecutionError::InvalidSpec(
                    "worker deadline is no longer representable",
                ))?;
        // Validate the entire maximum lifecycle before spawning a process.
        deadline
            .checked_add(spec.limits.termination_grace)
            .and_then(|value| value.checked_add(spec.limits.cleanup_timeout))
            .ok_or(ExecutionError::InvalidSpec(
                "cleanup deadline is not representable",
            ))?;
        let mut child = Command::new(&spec.executable)
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .env_clear()
            .envs(&spec.environment)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .process_group(0)
            .spawn()
            .map_err(ExecutionError::Spawn)?;
        let group_id = child.id();
        // A spawned child ID is positive and fits the native signed pid type.
        // Keep an explicit error path rather than constructing PID 0/1 signal targets.
        let Some(group) =
            Pid::from_raw(i32::try_from(group_id).unwrap_or(0)).filter(|pid| *pid != Pid::INIT)
        else {
            let kill = child.kill();
            let deadline = Instant::now() + spec.limits.cleanup_timeout;
            let wait = loop {
                match child.try_wait() {
                    Ok(Some(status)) => break Ok(status),
                    Ok(None) if Instant::now() < deadline => std::thread::sleep(POLL),
                    Ok(None) => {
                        break Err(io::Error::new(io::ErrorKind::TimedOut, "child not reaped"));
                    }
                    Err(error) => break Err(error),
                }
            };
            return Ok(ProcessOutcome {
                stop: ProcessStop::InfrastructureFailure {
                    operation: "group identity",
                    detail: "invalid spawned process group".into(),
                },
                exit: status(wait.as_ref().ok().copied()),
                stdout: CapturedOutput {
                    bytes: Vec::new(),
                    truncated: false,
                    eof: false,
                },
                stderr: CapturedOutput {
                    bytes: Vec::new(),
                    truncated: false,
                    eof: false,
                },
                input_bytes_written: 0,
                duration: start.elapsed(),
                cleanup: CleanupStatus::Failed {
                    child_reaped: wait.is_ok(),
                    group_gone: false,
                    errors: kill.err().into_iter().map(|e| e.to_string()).collect(),
                },
                process_group: group_id,
                sampled_peak_rss_bytes: None,
            });
        };
        let mut pipes = Pipes::new(&mut child);
        let mut exit = None;
        let mut stop = match pipes.configure() {
            Err(error) => {
                // Partially configured pipes may still block; close them before cleanup.
                pipes.stdin.take();
                pipes.stdout.take();
                pipes.stderr.take();
                failed("configure pipes", error)
            }
            Ok(()) => loop {
                if cancellation.load(Ordering::Acquire) {
                    break ProcessStop::Cancelled;
                }
                if Instant::now() >= deadline {
                    break ProcessStop::Deadline;
                }
                if let Some(reason) = pipes.transfer(spec) {
                    break reason;
                }
                match child.try_wait() {
                    Ok(Some(value)) => {
                        exit = Some(value);
                        break ProcessStop::Exited;
                    }
                    Ok(None) => {}
                    Err(error) => break failed("wait worker", error),
                }
                std::thread::sleep(POLL);
            },
        };
        pipes.stdin.take();
        let mut errors = Vec::new();
        let mut group_gone = matches!(probe_group(group), Ok(false));
        if !group_gone {
            signal(group, Signal::TERM, &mut errors);
        }
        let grace_deadline = Instant::now() + spec.limits.termination_grace;
        let cleanup_deadline = grace_deadline + spec.limits.cleanup_timeout;
        let mut killed = false;
        loop {
            if let Some(reason) = pipes.drain(spec)
                && stop == ProcessStop::Exited
            {
                stop = reason;
            }
            if exit.is_none() {
                match child.try_wait() {
                    Ok(Some(value)) => exit = Some(value),
                    Ok(None) => {}
                    Err(error) => {
                        let detail = format!("reap worker: {error}");
                        if !errors.contains(&detail) {
                            errors.push(detail);
                        }
                    }
                }
            }
            if !group_gone {
                match probe_group(group) {
                    Ok(false) => group_gone = true,
                    Ok(true) => {}
                    Err(error) => {
                        let detail = format!("check group disappearance: {error}");
                        if !errors.contains(&detail) {
                            errors.push(detail);
                        }
                    }
                }
            }
            if group_gone && exit.is_some() && pipes.stdout.is_none() && pipes.stderr.is_none() {
                break;
            }
            if !group_gone && !killed && Instant::now() >= grace_deadline {
                signal(group, Signal::KILL, &mut errors);
                killed = true;
            }
            if Instant::now() >= cleanup_deadline {
                break;
            }
            std::thread::sleep(POLL);
        }
        if pipes.stdout.is_some() || pipes.stderr.is_some() {
            errors.push("output pipes did not reach EOF before cleanup deadline".into());
        }
        let cleanup = if group_gone && exit.is_some() && errors.is_empty() {
            CleanupStatus::Complete
        } else {
            CleanupStatus::Failed {
                child_reaped: exit.is_some(),
                group_gone,
                errors,
            }
        };
        Ok(ProcessOutcome {
            stop,
            exit: status(exit),
            stdout: pipes.out,
            stderr: pipes.err,
            input_bytes_written: pipes.written,
            duration: start.elapsed(),
            cleanup,
            process_group: group_id,
            sampled_peak_rss_bytes: None,
        })
    }

    #[cfg(all(test, target_os = "macos", target_arch = "aarch64"))]
    mod tests {
        use super::*;

        #[test]
        fn transient_probe_error_followed_by_observed_disappearance_completes_cleanup() {
            let limits = WorkerLimits::new(
                0, 1024, 1024, Duration::from_secs(2), Duration::ZERO,
                Duration::from_millis(100), MemoryPolicy::Unbounded,
            ).unwrap();
            let spec = WorkerSpec::new(
                PathBuf::from("/usr/bin/true"), PathBuf::from("/"), Vec::new(),
                BTreeMap::new(), Vec::new(), limits,
            ).unwrap();
            let mut transient = 0;
            let outcome = execute_with_probe(&spec, &AtomicBool::new(false), |pid| {
                match group_exists(pid) {
                    Ok(false) if transient < 2 => {
                        transient += 1;
                        Err(rustix::io::Errno::PERM)
                    }
                    value => value,
                }
            }).unwrap();
            assert_eq!(transient, 2);
            assert_eq!(outcome.cleanup, CleanupStatus::Complete);
            assert_eq!(outcome.exit, ProcessExit::Success);
        }

        #[test]
        fn failed_disappearance_probe_is_cleanup_failure_even_after_reaping() {
            let limits = WorkerLimits::new(
                0,
                1024,
                1024,
                Duration::from_secs(2),
                Duration::ZERO,
                Duration::from_millis(20),
                MemoryPolicy::Unbounded,
            )
            .unwrap();
            let spec = WorkerSpec::new(
                PathBuf::from("/usr/bin/true"),
                PathBuf::from("/"),
                Vec::new(),
                BTreeMap::new(),
                Vec::new(),
                limits,
            )
            .unwrap();
            // The process and signals remain real. The double represents an OS
            // refusing the disappearance query after the actual group is gone.
            let outcome = execute_with_probe(&spec, &AtomicBool::new(false), |pid| {
                match group_exists(pid) {
                    Ok(false) => Err(rustix::io::Errno::PERM),
                    value => value,
                }
            })
            .unwrap();
            assert_eq!(outcome.stop, ProcessStop::Exited);
            assert_eq!(outcome.exit, ProcessExit::Success);
            assert!(matches!(outcome.cleanup, CleanupStatus::Failed {
                child_reaped: true, group_gone: false, ref errors
            } if errors.iter().any(|error| error.contains("check group disappearance"))));
            let group = Pid::from_raw(i32::try_from(outcome.process_group).unwrap()).unwrap();
            assert_eq!(group_exists(group), Ok(false), "real process group leaked");
        }
    }
}
