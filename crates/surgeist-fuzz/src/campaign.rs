//! Finite campaigns over registered owner binaries and the shared executor.
#![forbid(unsafe_code)]

use crate::artifact::*;
use crate::execution::{
    self, CleanupStatus, MemoryPolicy, ProcessExit, ProcessOutcome, ProcessStop, WorkerSpec,
};
use crate::registry::{RegisteredTarget, TargetRegistry};
use crate::worker::{self, WorkerRequest, WorkerResponse};
use crate::{
    CaseOutcome, CaseTarget, CoverageAssessment, ExecutionCounts, FuzzError, FuzzErrorKind,
    Generation, Result, Sha256Digest, SourceRevision, TargetDescriptor, TargetId,
};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsString;
use std::fs::{self, File, OpenOptions};
use std::io::{Cursor, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const FRAME: u32 = 1_048_576;
const TOOL_OUTPUT: usize = 16 * 1024 * 1024;
const MARKER: &[u8] = b"surgeist-fuzz campaign workspace schema 1\n";
static RUN_SEQUENCE: AtomicU64 = AtomicU64::new(0);
const COVERAGE_FLAGS: &str = "-Cpasses=sancov-module -Cllvm-args=-sanitizer-coverage-level=4 -Cllvm-args=-sanitizer-coverage-inline-8bit-counters -Cllvm-args=-sanitizer-coverage-pc-table -Cllvm-args=-sanitizer-coverage-trace-compares -Cllvm-args=-simplifycfg-branch-fold-threshold=0 -Zsanitizer=address -Cdebug-assertions -Ccodegen-units=1 --cfg fuzzing";

/// Stable terminal categories; finite success establishes only observed cases.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum CampaignStatus {
    Sufficient,
    Finding,
    InvalidRequest,
    InfrastructureFailure,
    Cancelled,
    ResourceStop,
    Insufficient,
    Empty,
}
impl CampaignStatus {
    #[must_use]
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Sufficient => 0,
            Self::Finding => 1,
            Self::InvalidRequest => 2,
            Self::InfrastructureFailure => 3,
            Self::Cancelled => 4,
            Self::ResourceStop => 5,
            Self::Insufficient => 6,
            Self::Empty => 7,
        }
    }
}
/// Machine summary schema. Detailed immutable evidence remains in artifact storage.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignSummary {
    pub schema: u32,
    pub command: String,
    pub status: CampaignStatus,
    pub targets: Vec<TargetDescriptor>,
    pub builds: Vec<BuildIdentity>,
    pub runs: Vec<String>,
    pub artifacts: Vec<String>,
    pub evaluated: u64,
    pub generation_rejected: u64,
    pub memory_policy: String,
    pub sampled_peak_rss_bytes: Option<u64>,
    pub cleanup: String,
    pub detail: String,
}
impl CampaignSummary {
    fn new(command: &str) -> Self {
        Self {
            schema: 1,
            command: command.into(),
            status: CampaignStatus::Sufficient,
            targets: Vec::new(),
            builds: Vec::new(),
            runs: Vec::new(),
            artifacts: Vec::new(),
            evaluated: 0,
            generation_rejected: 0,
            memory_policy: "no worker started".into(),
            sampled_peak_rss_bytes: None,
            cleanup: "no worker started".into(),
            detail: String::new(),
        }
    }
}
#[derive(Parser)]
#[command(
    name = "surgeist-fuzz",
    version,
    propagate_version = true,
    about = "Bounded fuzz campaigns and reproducible owner-case replay"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Discover registered targets without building or writing campaign data.
    List(Options),
    /// Validate discovery metadata without building owner binaries.
    Validate(Options),
    /// Evaluate finite generated cases, one input, or a regression directory.
    Check(Options),
    /// Run an instrumented coverage campaign with finite budgets.
    Run(Options),
    /// Replay immutable evidence against an exact or explicit comparison build.
    Replay(Options),
    /// Reduce an immutable failure through isolated owner evaluations.
    Minimize(Options),
    /// Report published and partial evidence in an owned campaign directory.
    Report(Options),
}
impl Command {
    fn options(self) -> Options {
        let (name, mut options) = match self {
            Self::List(o) => ("list", o),
            Self::Validate(o) => ("validate", o),
            Self::Check(o) => ("check", o),
            Self::Run(o) => ("run", o),
            Self::Replay(o) => ("replay", o),
            Self::Minimize(o) => ("minimize", o),
            Self::Report(o) => ("report", o),
        };
        options.command = name.into();
        options
    }
}
fn parse_target(value: &str) -> std::result::Result<TargetId, String> {
    TargetId::new(value).map_err(|error| error.to_string())
}
fn parse_cli(args: &[OsString]) -> std::result::Result<Cli, clap::Error> {
    Cli::try_parse_from(
        std::iter::once(OsString::from("surgeist-fuzz")).chain(args.iter().cloned()),
    )
}

#[derive(Clone, Default)]
struct ToolPaths {
    cargo: PathBuf,
    rustc: PathBuf,
    git: PathBuf,
}

#[derive(Args, Clone)]
struct Options {
    #[arg(skip)]
    command: String,
    #[arg(skip)]
    tools: ToolPaths,
    /// Source checkout or workspace to discover.
    #[arg(long, default_value = ".")]
    source: PathBuf,
    /// Workspace manifest relative to source.
    #[arg(long, default_value = "Cargo.toml")]
    manifest: PathBuf,
    /// Absolute campaign output directory outside source.
    #[arg(long)]
    work: Option<PathBuf>,
    /// Owner-qualified target identity.
    #[arg(long, value_parser = parse_target)]
    target: Option<TargetId>,
    /// Select targets belonging to this crate.
    #[arg(long = "crate")]
    owner: Option<String>,
    /// Explicit comma-separated Cargo features.
    #[arg(long, value_delimiter = ',', action = clap::ArgAction::Set)]
    features: Vec<String>,
    /// Installed Rust toolchain; run requires nightly-YYYY-MM-DD.
    #[arg(long, default_value = "stable")]
    toolchain: String,
    /// Absolute Cargo executable; otherwise resolve installed tool from PATH.
    #[arg(long = "cargo")]
    cargo_override: Option<PathBuf>,
    /// Absolute rustc executable; otherwise resolve installed tool from PATH.
    #[arg(long = "rustc")]
    rustc_override: Option<PathBuf>,
    /// Absolute Git executable; otherwise resolve installed tool from PATH.
    #[arg(long = "git")]
    git_override: Option<PathBuf>,
    /// Require this exact source revision.
    #[arg(long)]
    source_sha: Option<String>,
    /// Immutable failure or capture artifact path for replay/minimize.
    #[arg(long)]
    artifact: Option<String>,
    /// Canonical case file for check, or entropy seed for run.
    #[arg(long, conflicts_with = "regressions")]
    input: Option<PathBuf>,
    /// Check each canonical case in this registered fixture directory once.
    #[arg(long, conflicts_with = "input")]
    regressions: Option<PathBuf>,
    /// Compatible prior engine entropy corpus directory.
    #[arg(long = "restart-corpus")]
    restart: Option<PathBuf>,
    /// Owner feature enabling its coverage entry point.
    #[arg(long, default_value = "coverage")]
    coverage_feature: String,
    /// Absolute path whose appearance requests cancellation.
    #[arg(long)]
    cancel_file: Option<PathBuf>,
    /// Finite generation/coverage execution limit.
    #[arg(long, default_value_t = 100)]
    cases: u64,
    /// Maximum campaign wall time in milliseconds.
    #[arg(long, default_value_t = 30_000)]
    wall_ms: u64,
    /// Per-case deadline; coverage requires whole seconds.
    #[arg(long, default_value_t = 1000)]
    case_ms: u64,
    /// Offline build deadline in milliseconds.
    #[arg(long, default_value_t = 300_000)]
    build_ms: u64,
    /// Hard canonical input ceiling, at most 128 KiB.
    #[arg(long, default_value_t = 4096)]
    input_bytes: u64,
    /// Total artifact/corpus byte quota.
    #[arg(long, default_value_t = 64 * 1024 * 1024)]
    storage_bytes: u64,
    /// Total artifact/corpus entry quota.
    #[arg(long, default_value_t = 1024)]
    storage_entries: u64,
    /// Engine memory budget; OS enforcement remains explicit.
    #[arg(long, default_value_t = 2 * 1024 * 1024 * 1024)]
    memory_bytes: u64,
    /// Explicit unbounded OS policy; mandatory hard limits are unsupported.
    #[arg(long)]
    memory_policy: Option<String>,
    /// Deterministic generation seed.
    #[arg(long, default_value_t = 1)]
    seed: u64,
    /// Maximum reduction replay attempts.
    #[arg(long, default_value_t = 100)]
    attempts: u64,
    /// Repeated confirmations for reduction findings.
    #[arg(long, default_value_t = 2)]
    confirmations: u32,
    /// Replay another source revision with a separate comparison record.
    #[arg(long)]
    comparison: bool,
    /// Explicit campaign worker count, from 1 through 8.
    #[arg(long, default_value_t = 1)]
    workers: u32,
    /// Emit the machine-readable summary (also the default).
    #[arg(long)]
    json: bool,
}
impl Options {
    fn parse(args: &[OsString]) -> Result<Self> {
        parse_cli(args)
            .map_err(|error| invalid(error.to_string()))?
            .command
            .options()
            .validate()
    }
    fn validate(mut self) -> Result<Self> {
        let o = &mut self;
        o.source = o.source.canonicalize().map_err(io_error)?;
        if o.toolchain.is_empty()
            || o.toolchain.len() > 128
            || !o
                .toolchain
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
            || o.toolchain.starts_with('-')
        {
            return Err(invalid("invalid installed toolchain name"));
        }
        if o.features.iter().any(|f| {
            f.is_empty()
                || f.len() > 128
                || !f
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_-+".contains(&b))
        }) || o.features.iter().collect::<BTreeSet<_>>().len() != o.features.len()
        {
            return Err(invalid("features must be unique Cargo names"));
        }
        if o.manifest.is_absolute()
            || o.manifest
                .components()
                .any(|c| !matches!(c, Component::Normal(_)))
        {
            return Err(invalid(
                "manifest must be a relative path without traversal",
            ));
        }
        if o.cases == 0
            || o.wall_ms == 0
            || o.case_ms == 0
            || o.build_ms == 0
            || o.case_ms > o.wall_ms
            || o.input_bytes == 0
            || o.input_bytes > 128 * 1024
            || o.storage_bytes == 0
            || o.storage_entries == 0
            || o.memory_bytes == 0
            || o.workers == 0
            || o.workers > 8
            || o.cases < u64::from(o.workers)
            || o.storage_entries < u64::from(o.workers)
            || o.storage_bytes < u64::from(o.workers)
        {
            return Err(invalid(
                "positive finite budgets required; input ceiling 128 KiB; case deadline must fit wall budget",
            ));
        }
        if o.workers > 1 && !matches!(o.command.as_str(), "check" | "run") {
            return Err(invalid("parallel workers only for check/run"));
        }
        if o.comparison && o.command != "replay" {
            return Err(invalid(
                "comparison is an explicit replay operation; minimize requires an exact build",
            ));
        }
        if o.wall_ms > 86_400_000
            || o.case_ms > 600_000
            || o.build_ms > 3_600_000
            || o.attempts > 1_000_000
            || o.confirmations < 2
            || o.confirmations > 1000
        {
            return Err(invalid("budget exceeds local backend limits"));
        }
        if o.command == "run" && !o.case_ms.is_multiple_of(1000) {
            return Err(invalid(
                "coverage case deadlines must be exact multiples of 1000 milliseconds",
            ));
        }
        if o.regressions.is_some() && (o.command != "check" || o.workers != 1) {
            return Err(invalid(
                "regression directories require check with one worker",
            ));
        }
        if o.command == "run" && !dated_nightly(&o.toolchain) {
            return Err(invalid(
                "coverage requires an explicit installed nightly date",
            ));
        }
        if o.memory_policy
            .as_deref()
            .is_some_and(|policy| policy != "unbounded")
        {
            return Err(invalid(
                "unknown or unsupported memory policy; only unbounded is available",
            ));
        }
        if !matches!(o.command.as_str(), "list" | "validate" | "report")
            && o.memory_policy.as_deref() != Some("unbounded")
        {
            return Err(invalid(
                "explicit --memory-policy unbounded required; hard limits unsupported",
            ));
        }
        if let Some(work) = &o.work {
            o.work = Some(absolute_future(work)?);
        }
        if let Some(input) = &o.input {
            o.input = Some(input.canonicalize().map_err(io_error)?);
        }
        if let Some(path) = &o.regressions {
            let absolute = if path.is_absolute() {
                path.clone()
            } else {
                std::env::current_dir().map_err(io_error)?.join(path)
            };
            let canonical = absolute.canonicalize().map_err(io_error)?;
            if absolute != canonical {
                return Err(invalid(
                    "regression directory must not traverse symlinks or dot components",
                ));
            }
            o.regressions = Some(canonical);
        }
        if let Some(restart) = &o.restart {
            o.restart = Some(restart.canonicalize().map_err(io_error)?);
        }
        if let Some(path) = &o.cancel_file {
            o.cancel_file = Some(absolute_future(path)?);
        }
        o.tools = ToolPaths {
            cargo: o
                .cargo_override
                .clone()
                .map_or_else(|| resolve_tool("cargo"), Ok)?,
            rustc: o
                .rustc_override
                .clone()
                .map_or_else(|| resolve_tool("rustc"), Ok)?,
            git: o
                .git_override
                .clone()
                .map_or_else(|| resolve_tool("git"), Ok)?,
        };
        Ok(self)
    }
}
fn dated_nightly(value: &str) -> bool {
    let Some(date) = value.strip_prefix("nightly-") else {
        return false;
    };
    let parts: Vec<_> = date.split('-').collect();
    if parts.len() != 3
        || parts[0].len() != 4
        || parts[1].len() != 2
        || parts[2].len() != 2
        || parts
            .iter()
            .any(|part| !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return false;
    }
    let (Ok(year), Ok(month), Ok(day)) = (
        parts[0].parse::<u32>(),
        parts[1].parse::<u32>(),
        parts[2].parse::<u32>(),
    ) else {
        return false;
    };
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    year != 0 && day != 0 && day <= days
}
fn invalid(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Cli, "campaign preflight", detail)
}
fn infrastructure(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Process, "campaign execution", detail)
}
fn io_error(error: std::io::Error) -> FuzzError {
    FuzzError::with_source(
        FuzzErrorKind::Io,
        "campaign file operation",
        "I/O failure",
        error,
    )
}
fn digest_file(path: &Path) -> Result<Sha256Digest> {
    Sha256Digest::from_file(path).map_err(|e| infrastructure(e.to_string()))
}
fn resolve_tool(name: &str) -> Result<PathBuf> {
    let path = std::env::var_os("PATH").ok_or_else(|| invalid("PATH unavailable"))?;
    for dir in std::env::split_paths(&path) {
        let p = dir.join(name);
        if p.is_file() {
            return Ok(p);
        }
    }
    Err(invalid(format!("required tool {name} unavailable")))
}
fn absolute_future(path: &Path) -> Result<PathBuf> {
    if !path.is_absolute()
        || path
            .components()
            .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid("output paths must be absolute without traversal"));
    }
    let mut ancestor = path;
    while !ancestor.exists() {
        ancestor = ancestor
            .parent()
            .ok_or_else(|| invalid("path has no existing ancestor"))?;
    }
    if ancestor.canonicalize().map_err(io_error)? != ancestor {
        return Err(invalid("output paths cannot traverse symlinks"));
    }
    Ok(path.to_owned())
}
fn environment(o: &Options) -> BTreeMap<OsString, OsString> {
    let mut env = BTreeMap::new();
    for key in [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "TMPDIR",
        "SDKROOT",
        "DEVELOPER_DIR",
    ] {
        if let Some(value) = std::env::var_os(key) {
            env.insert(key.into(), value);
        }
    }
    env.insert("RUSTUP_TOOLCHAIN".into(), o.toolchain.clone().into());
    env.insert("CARGO_NET_OFFLINE".into(), "true".into());
    env.insert("CARGO_BUILD_JOBS".into(), "1".into());
    env.insert("GIT_OPTIONAL_LOCKS".into(), "0".into());
    env
}
struct Invocation<'a> {
    tool: &'a Path,
    cwd: &'a Path,
    args: Vec<OsString>,
    environment: BTreeMap<OsString, OsString>,
    input: Vec<u8>,
    output: usize,
    timeout: Duration,
}
fn invoke(invocation: Invocation<'_>, cancel: &AtomicBool) -> Result<ProcessOutcome> {
    let Invocation {
        tool,
        cwd,
        args,
        environment: env,
        input,
        output,
        timeout,
    } = invocation;
    let limits = execution::WorkerLimits::new(
        input.len(),
        output,
        output,
        timeout,
        Duration::from_millis(100),
        Duration::from_secs(3),
        MemoryPolicy::Unbounded,
    )
    .map_err(|e| infrastructure(e.to_string()))?;
    let spec = WorkerSpec::new(tool.to_owned(), cwd.to_owned(), args, env, input, limits)
        .map_err(|e| infrastructure(e.to_string()))?;
    execution::execute(&spec, cancel).map_err(|e| infrastructure(e.to_string()))
}
fn require_success(p: &ProcessOutcome, operation: &str) -> Result<()> {
    if p.cleanup != CleanupStatus::Complete {
        return Err(infrastructure(format!(
            "{operation} cleanup: {:?}",
            p.cleanup
        )));
    }
    if p.stop != ProcessStop::Exited || p.exit != ProcessExit::Success {
        return Err(infrastructure(format!(
            "{operation}: {:?}/{:?}: {}",
            p.stop,
            p.exit,
            String::from_utf8_lossy(&p.stderr.bytes)
        )));
    }
    Ok(())
}
fn output_text(p: &ProcessOutcome) -> Result<String> {
    String::from_utf8(p.stdout.bytes.clone())
        .map_err(|_| infrastructure("tool output is not UTF-8"))
}
struct Preflight {
    registry: TargetRegistry,
    selected: Vec<RegisteredTarget>,
    source: Option<SourceRevision>,
    compiler: String,
    host: String,
    metadata: serde_json::Value,
    regression_files: Option<Vec<PathBuf>>,
}
fn clean_source(o: &Options, cancel: &AtomicBool) -> Result<SourceRevision> {
    let run = |args: Vec<OsString>| -> Result<String> {
        let p = invoke(
            Invocation {
                tool: &o.tools.git,
                cwd: &o.source,
                args,
                environment: environment(o),
                input: Vec::new(),
                output: TOOL_OUTPUT,
                timeout: Duration::from_secs(30),
            },
            cancel,
        )?;
        require_success(&p, "source validation")?;
        output_text(&p)
    };
    if Path::new(run(vec!["rev-parse".into(), "--show-toplevel".into()])?.trim())
        .canonicalize()
        .map_err(io_error)?
        != o.source
    {
        return Err(invalid("--source must be the prepared Git checkout root"));
    }
    if !run(vec![
        "status".into(),
        "--porcelain=v1".into(),
        "--untracked-files=all".into(),
    ])?
    .is_empty()
    {
        return Err(invalid(
            "source must have no tracked changes or untracked files",
        ));
    }
    SourceRevision::new(run(vec!["rev-parse".into(), "HEAD".into()])?.trim())
        .map_err(|e| invalid(e.to_string()))
}
fn preflight(o: &Options, cancel: &AtomicBool) -> Result<Preflight> {
    if !execution::execution_capabilities().supported_host {
        return Err(invalid("execution requires Apple-Silicon macOS"));
    }
    for tool in [&o.tools.cargo, &o.tools.rustc, &o.tools.git] {
        if !tool.is_absolute() || !tool.is_file() {
            return Err(invalid(format!("missing absolute tool {}", tool.display())));
        }
    }
    let manifest = o
        .source
        .join(&o.manifest)
        .canonicalize()
        .map_err(io_error)?;
    if !manifest.starts_with(&o.source) {
        return Err(invalid("manifest escapes source"));
    }
    let p = invoke(
        Invocation {
            tool: &o.tools.cargo,
            cwd: &o.source,
            args: vec![
                "metadata".into(),
                "--offline".into(),
                "--locked".into(),
                "--frozen".into(),
                "--no-deps".into(),
                "--format-version".into(),
                "1".into(),
                "--manifest-path".into(),
                manifest.as_os_str().to_owned(),
            ],
            environment: environment(o),
            input: Vec::new(),
            output: TOOL_OUTPUT,
            timeout: Duration::from_secs(30),
        },
        cancel,
    )?;
    require_success(&p, "Cargo discovery")?;
    let metadata: serde_json::Value =
        serde_json::from_slice(&p.stdout.bytes).map_err(|e| infrastructure(e.to_string()))?;
    let root = metadata["workspace_root"]
        .as_str()
        .ok_or_else(|| infrastructure("Cargo omitted workspace root"))?;
    let registry = TargetRegistry::from_metadata_json(Path::new(root), &p.stdout.bytes)?;
    let selected: Vec<_> = registry
        .targets()
        .filter(|t| {
            o.target.as_ref().is_none_or(|id| t.descriptor().id() == id)
                && o.owner
                    .as_ref()
                    .is_none_or(|owner| t.descriptor().owner() == owner)
        })
        .cloned()
        .collect();
    if let Some(id) = &o.target {
        registry.get(id)?;
    }
    let mut regression_files = None;
    let mut source = None;
    let mut compiler = String::new();
    let mut host = String::new();
    if !matches!(o.command.as_str(), "list" | "validate" | "report") {
        if selected.len() != 1 {
            return Err(invalid("select exactly one registered target"));
        }
        let work = o.work.as_ref().ok_or_else(|| invalid("--work required"))?;
        if work.starts_with(&o.source) {
            return Err(invalid("output must be outside immutable source"));
        }
        if work.exists() && fs::read(work.join(".surgeist-fuzz-owned")).map_err(io_error)? != MARKER
        {
            return Err(invalid("work directory lacks campaign ownership marker"));
        }
        source = Some(clean_source(o, cancel)?);
        if o.source_sha
            .as_ref()
            .is_some_and(|sha| source.as_ref().is_none_or(|actual| actual.as_str() != sha))
        {
            return Err(invalid("HEAD differs from --source-sha"));
        }
        if !Path::new(root).join("Cargo.lock").is_file() {
            return Err(invalid("prepared workspace requires committed Cargo.lock"));
        }
        let p = invoke(
            Invocation {
                tool: &o.tools.rustc,
                cwd: &o.source,
                args: vec!["-vV".into()],
                environment: environment(o),
                input: Vec::new(),
                output: 65536,
                timeout: Duration::from_secs(30),
            },
            cancel,
        )?;
        require_success(&p, "compiler preflight")?;
        compiler = output_text(&p)?;
        host = compiler
            .lines()
            .find_map(|l| l.strip_prefix("host: "))
            .ok_or_else(|| infrastructure("rustc omitted host"))?
            .into();
        if o.command == "run" && !compiler.contains("nightly") {
            return Err(invalid("coverage compiler is not nightly"));
        }
        let target = &selected[0];
        if let Some(directory) = &o.regressions {
            regression_files = Some(regression_inputs(o, target, directory)?);
        }
        if target
            .descriptor()
            .requirements()
            .features()
            .iter()
            .any(|f| !o.features.contains(f))
        {
            return Err(invalid("explicit features omit owner requirements"));
        }
        target.descriptor().requirements().require_capabilities(&[
            crate::Capability::Property,
            crate::Capability::Replay,
            crate::Capability::Reduction,
            crate::Capability::CaseCapture,
            crate::Capability::ProcessGroups,
            crate::Capability::Coverage,
        ])?;
        if o.input_bytes > target.descriptor().limits().bytes() {
            return Err(invalid("input ceiling exceeds owner limit"));
        }
        let package = metadata["packages"]
            .as_array()
            .and_then(|ps| {
                ps.iter()
                    .find(|p| p["name"].as_str() == Some(target.package()))
            })
            .ok_or_else(|| infrastructure("missing selected package"))?;
        for f in o
            .features
            .iter()
            .chain((o.command == "run").then_some(&o.coverage_feature))
        {
            if package["features"].get(f).is_none() {
                return Err(invalid(format!("unknown feature {f}")));
            }
        }
        if let Some(input) = &o.input
            && fs::metadata(input).map_err(io_error)?.len() > o.input_bytes
        {
            return Err(invalid("input exceeds ceiling"));
        }
        if matches!(o.command.as_str(), "replay" | "minimize") && o.artifact.is_none() {
            return Err(invalid("--artifact required"));
        }
        StorageLimits::new(
            o.storage_entries,
            o.storage_bytes,
            o.input_bytes,
            FRAME.into(),
        )?;
        crate::reduction::ReductionLimits::new(
            o.attempts,
            Duration::from_millis(o.wall_ms),
            o.storage_bytes.min(16 * 1024 * 1024),
            o.confirmations,
        )?;
        if o.command == "run" {
            let worst = o
                .cases
                .checked_add(2)
                .and_then(|n| n.checked_mul(o.input_bytes))
                .ok_or_else(|| invalid("corpus budget overflow"))?;
            if worst > o.storage_bytes || o.cases.saturating_add(2) > o.storage_entries {
                return Err(invalid(
                    "quotas cannot admit conservative finite corpus bound",
                ));
            }
        }
    }
    Ok(Preflight {
        registry,
        selected,
        source,
        compiler,
        host,
        metadata,
        regression_files,
    })
}
/// Enumerate immutable owner inputs before output admission. Fixture roles are
/// selected explicitly by the caller, never inferred from a directory name.
fn regression_inputs(
    options: &Options,
    target: &RegisteredTarget,
    directory: &Path,
) -> Result<Vec<PathBuf>> {
    if !directory.is_dir()
        || !target
            .fixtures()
            .iter()
            .any(|fixture| fixture.is_dir() && directory.starts_with(fixture))
    {
        return Err(invalid(
            "regression directory must be contained in a selected registered fixture directory",
        ));
    }
    let mut pending = vec![directory.to_owned()];
    let mut files = Vec::new();
    let mut entries = 0_u64;
    let mut bytes = 0_u64;
    while let Some(path) = pending.pop() {
        for entry in fs::read_dir(path).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            entries = entries
                .checked_add(1)
                .ok_or_else(|| invalid("regression entry count overflow"))?;
            if entries > options.storage_entries {
                return Err(invalid("regression entry quota exceeded"));
            }
            let metadata = fs::symlink_metadata(entry.path()).map_err(io_error)?;
            if metadata.is_dir() {
                pending.push(entry.path());
            } else if metadata.is_file() {
                if metadata.len() > options.input_bytes {
                    return Err(invalid("regression input exceeds ceiling"));
                }
                bytes = bytes
                    .checked_add(metadata.len())
                    .ok_or_else(|| invalid("regression byte count overflow"))?;
                if bytes > options.storage_bytes {
                    return Err(invalid("regression byte quota exceeded"));
                }
                files.push(entry.path());
            } else {
                return Err(invalid(
                    "regression entries must be regular files/directories without symlinks",
                ));
            }
        }
    }
    if files.len() as u64 > options.cases {
        return Err(invalid("case budget cannot admit every regression input"));
    }
    files.sort();
    Ok(files)
}
fn write_new(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut f = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_error)?;
    f.write_all(bytes).map_err(io_error)?;
    f.sync_all().map_err(io_error)
}
fn read_bounded(path: &Path, max: u64) -> Result<Vec<u8>> {
    let mut f = File::open(path).map_err(io_error)?;
    if f.metadata().map_err(io_error)?.len() > max {
        return Err(invalid("file exceeds ceiling"));
    }
    let mut bytes = Vec::new();
    Read::by_ref(&mut f)
        .take(max.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > max {
        return Err(invalid("file grew beyond ceiling"));
    }
    Ok(bytes)
}
fn create_work(o: &Options) -> Result<PathBuf> {
    let work = o.work.clone().ok_or_else(|| invalid("work required"))?;
    if !work.exists() {
        fs::create_dir_all(&work).map_err(io_error)?;
        write_new(&work.join(".surgeist-fuzz-owned"), MARKER)?;
    }
    Ok(work)
}
fn hash_tree(path: &Path) -> Result<Sha256Digest> {
    fn collect(root: &Path, path: &Path, out: &mut Vec<u8>) -> Result<()> {
        let m = fs::symlink_metadata(path).map_err(io_error)?;
        if m.file_type().is_symlink() {
            return Err(invalid("fixture provider contains symlink"));
        }
        if m.is_file() {
            out.extend_from_slice(
                path.strip_prefix(root)
                    .unwrap_or(path)
                    .as_os_str()
                    .as_encoded_bytes(),
            );
            out.push(0);
            out.extend_from_slice(digest_file(path)?.as_str().as_bytes());
            out.push(0);
        } else if m.is_dir() {
            let mut ps = fs::read_dir(path)
                .map_err(io_error)?
                .map(|v| v.map(|e| e.path()))
                .collect::<std::io::Result<Vec<_>>>()
                .map_err(io_error)?;
            ps.sort();
            for p in ps {
                collect(root, &p, out)?;
            }
        } else {
            return Err(invalid("fixture provider is not ordinary file"));
        }
        Ok(())
    }
    let mut out = Vec::new();
    collect(path, path, &mut out)?;
    Ok(Sha256Digest::from_bytes(out))
}
fn build(
    o: &Options,
    p: &Preflight,
    target: &RegisteredTarget,
    work: &Path,
    coverage: bool,
    cancel: &AtomicBool,
) -> Result<(PathBuf, BuildIdentity)> {
    let source = p
        .source
        .clone()
        .ok_or_else(|| infrastructure("source missing"))?;
    let mut features = o.features.clone();
    if coverage {
        features.push(o.coverage_feature.clone());
    }
    features.sort();
    features.dedup();
    let context = Sha256Digest::from_bytes(format!(
        "{}|{}|{}|{}|{features:?}|{}",
        source,
        p.compiler,
        target.package(),
        target.descriptor().binary(),
        if coverage { COVERAGE_FLAGS } else { "ordinary" }
    ));
    let dir = work
        .join("builds")
        .join(source.as_str())
        .join(context.as_str())
        .join(if coverage { "instrumented" } else { "ordinary" });
    let mut args: Vec<OsString> = [
        "build",
        "--offline",
        "--locked",
        "--frozen",
        "-j",
        "1",
        "--no-default-features",
        "--message-format=json",
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.extend([
        "--manifest-path".into(),
        target.manifest_path().as_os_str().to_owned(),
        "--package".into(),
        target.package().into(),
        "--bin".into(),
        target.descriptor().binary().into(),
        "--target-dir".into(),
        dir.as_os_str().to_owned(),
    ]);
    if !features.is_empty() {
        args.extend(["--features".into(), features.join(",").into()]);
    }
    let mut env = environment(o);
    if coverage {
        args.extend(["--release".into(), "--target".into(), p.host.clone().into()]);
        env.insert("RUSTFLAGS".into(), COVERAGE_FLAGS.into());
    }
    let output = invoke(
        Invocation {
            tool: &o.tools.cargo,
            cwd: &o.source,
            args,
            environment: env,
            input: Vec::new(),
            output: TOOL_OUTPUT,
            timeout: Duration::from_millis(o.build_ms),
        },
        cancel,
    )?;
    require_success(&output, "offline locked owner build")?;
    if clean_source(o, cancel)? != source {
        return Err(infrastructure("source changed during build"));
    }
    let mut executable = None;
    for line in output.stdout.bytes.split(|b| *b == b'\n') {
        if let Ok(m) = serde_json::from_slice::<serde_json::Value>(line)
            && m["reason"] == "compiler-artifact"
            && m["target"]["name"].as_str() == Some(target.descriptor().binary())
            && let Some(path) = m["executable"].as_str()
        {
            executable = Some(PathBuf::from(path));
        }
    }
    let exe = executable
        .ok_or_else(|| infrastructure("Cargo omitted executable"))?
        .canonicalize()
        .map_err(io_error)?;
    if !exe.starts_with(&dir) {
        return Err(infrastructure("executable escaped target directory"));
    }
    let mut providers = Vec::new();
    for (name, path) in [
        ("cargo-tool", o.tools.cargo.as_path()),
        ("rustc-tool", o.tools.rustc.as_path()),
        ("descriptor", target.descriptor_path()),
        ("owner-fuzz-manifest", target.manifest_path()),
    ] {
        providers.push(ContentIdentity::new(
            ContentRole::BuildTool,
            name,
            digest_file(path)?,
        )?);
    }
    for (index, path) in target.fixtures().iter().enumerate() {
        providers.push(ContentIdentity::new(
            ContentRole::Fixture,
            format!("fixture-{index}"),
            hash_tree(path)?,
        )?);
    }
    let root = p.metadata["workspace_root"]
        .as_str()
        .ok_or_else(|| infrastructure("workspace root missing"))?;
    let identity = BuildIdentity::new(BuildIdentitySpec {
        source,
        lockfile: digest_file(&Path::new(root).join("Cargo.lock"))?,
        executable: digest_file(&exe)?,
        toolchain: p.compiler.lines().collect::<Vec<_>>().join("; "),
        engine: if coverage {
            "libfuzzer-sys; framed-owner-protocol"
        } else {
            "isolated-owner-protocol"
        }
        .into(),
        sanitizer: if coverage {
            "address; sanitizer-coverage-level=4"
        } else {
            "none"
        }
        .into(),
        platform: p.host.clone(),
        features,
        providers,
    })?;
    Ok((exe, identity))
}
struct WorkerCall {
    process: ProcessOutcome,
    responses: Vec<WorkerResponse>,
}
fn worker_call(
    executable: &Path,
    cwd: &Path,
    request: &WorkerRequest,
    timeout: Duration,
    output: usize,
    cancel: &AtomicBool,
) -> Result<WorkerCall> {
    let mut input = Vec::new();
    worker::write_frame(&mut input, request, FRAME)?;
    let process = invoke(
        Invocation {
            tool: executable,
            cwd,
            args: Vec::new(),
            environment: BTreeMap::new(),
            input,
            output,
            timeout,
        },
        cancel,
    )?;
    let responses = parse_frames(&process)?;
    Ok(WorkerCall { process, responses })
}
fn parse_frames(process: &ProcessOutcome) -> Result<Vec<WorkerResponse>> {
    let mut responses = Vec::new();
    let mut reader = Cursor::new(&process.stdout.bytes);
    loop {
        match worker::read_frame::<_, WorkerResponse>(&mut reader, FRAME) {
            Ok(Some(r)) => responses.push(r),
            Ok(None) => break,
            Err(_)
                if process.stop != ProcessStop::Exited || process.exit != ProcessExit::Success =>
            {
                break;
            }
            Err(e) => return Err(e),
        }
    }
    Ok(responses)
}
fn cleanup_label(cleanup: &CleanupStatus) -> String {
    match cleanup {
        CleanupStatus::Complete => "complete".into(),
        other => format!("failed: {other:?}"),
    }
}
fn classify_process(p: &ProcessOutcome) -> CampaignStatus {
    if p.cleanup != CleanupStatus::Complete {
        return CampaignStatus::InfrastructureFailure;
    }
    match p.stop {
        ProcessStop::Cancelled => CampaignStatus::Cancelled,
        ProcessStop::Deadline | ProcessStop::OutputLimit(_) => CampaignStatus::ResourceStop,
        ProcessStop::InfrastructureFailure { .. } => CampaignStatus::InfrastructureFailure,
        ProcessStop::Exited if p.exit == ProcessExit::Success => CampaignStatus::Sufficient,
        ProcessStop::Exited => CampaignStatus::Finding,
    }
}
fn captured(call: &WorkerCall) -> Option<&[u8]> {
    call.responses.iter().rev().find_map(|r| {
        if let WorkerResponse::Captured { canonical } = r {
            Some(canonical.as_slice())
        } else {
            None
        }
    })
}
fn observed(call: &WorkerCall) -> Option<&CaseOutcome> {
    call.responses.iter().rev().find_map(|r| {
        if let WorkerResponse::Evaluated { outcome, .. } = r {
            Some(outcome)
        } else {
            None
        }
    })
}
fn persist_call(
    target: &TargetDescriptor,
    identity: &BuildIdentity,
    config: &RunConfig,
    store: &ArtifactStore,
    call: &WorkerCall,
    summary: &mut CampaignSummary,
) -> Result<Option<crate::RelativePath>> {
    let Some(input) = captured(call) else {
        return Ok(None);
    };
    if let Some(outcome) = observed(call) {
        target.validate_outcome(outcome)?;
        if outcome.failure_key().is_some() {
            let path = store.publish_failure(&CaseArtifact::new(
                target.clone(),
                identity.clone(),
                config.clone(),
                input.to_vec(),
                outcome.clone(),
            )?)?;
            summary.artifacts.push(path.as_str().into());
            return Ok(Some(path));
        }
    }
    if classify_process(&call.process) != CampaignStatus::Sufficient {
        let a = CaptureArtifact::new(
            target.clone(),
            identity.clone(),
            config.clone(),
            input.to_vec(),
            ProcessEvidence::from_outcome(&call.process)?,
        )?;
        let path = store.publish_capture(&a)?;
        summary.artifacts.push(path.as_str().into());
        return Ok(Some(path));
    }
    Ok(None)
}

fn handshake(
    exe: &Path,
    cwd: &Path,
    target: &TargetDescriptor,
    o: &Options,
    cancel: &AtomicBool,
) -> Result<()> {
    let call = worker_call(
        exe,
        cwd,
        &WorkerRequest::Descriptor,
        Duration::from_millis(o.case_ms),
        FRAME as usize,
        cancel,
    )?;
    require_success(&call.process, "owner descriptor handshake")?;
    if call.responses
        != vec![WorkerResponse::Descriptor {
            descriptor: target.clone(),
        }]
    {
        return Err(infrastructure(
            "binary descriptor differs from registration",
        ));
    }
    Ok(())
}
fn evaluation(
    call: &WorkerCall,
    canonical: &[u8],
    target: &TargetDescriptor,
) -> Result<Option<CaseOutcome>> {
    match call.responses.as_slice() {
        [
            WorkerResponse::Captured { canonical: actual },
            WorkerResponse::Evaluated { outcome, counts },
        ] if actual == canonical => {
            target.validate_outcome(outcome)?;
            if counts.generation_rejected() != 0
                || counts.evaluated() != u64::from(outcome.is_evaluated())
            {
                return Err(infrastructure(
                    "ordinary worker returned inconsistent counters",
                ));
            }
            Ok(Some(outcome.clone()))
        }
        [WorkerResponse::Captured { canonical: actual }]
            if actual == canonical
                && classify_process(&call.process) != CampaignStatus::Sufficient =>
        {
            Ok(None)
        }
        [] if classify_process(&call.process) != CampaignStatus::Sufficient => Ok(None),
        _ => Err(infrastructure(
            "invalid capture/evaluation ordering or canonical bytes",
        )),
    }
}
fn store(o: &Options, work: &Path) -> Result<ArtifactStore> {
    let root = work.join("artifacts");
    fs::create_dir_all(&root).map_err(io_error)?;
    ArtifactStore::open(
        root,
        StorageLimits::new(
            o.storage_entries,
            o.storage_bytes,
            o.input_bytes,
            FRAME.into(),
        )?,
    )
}
fn publish_run(
    context: &CampaignContext<'_>,
    counts: ExecutionCounts,
    checked: BTreeSet<crate::PropertyId>,
    failures: Vec<EvidenceStatus>,
    summary: &mut CampaignSummary,
) -> Result<()> {
    let CampaignContext {
        target,
        build,
        config,
        store,
        ..
    } = *context;
    let record = RunRecord::new(RunRecordSpec {
        schema: 1,
        target: target.clone(),
        build: build.clone(),
        config: config.clone(),
        counts,
        checked_properties: checked.into_iter().collect(),
        exclusions: vec![
            "finite observations; no exhaustive correctness claim".into(),
            "RSS sampling unavailable; no OS memory ceiling".into(),
        ],
        failures,
        cleanup: summary.cleanup.clone(),
    })?;
    summary
        .artifacts
        .push(store.publish_run(&record)?.as_str().into());
    Ok(())
}
#[derive(Clone, Copy)]
struct CampaignContext<'a> {
    options: &'a Options,
    target: &'a TargetDescriptor,
    executable: &'a Path,
    cwd: &'a Path,
    build: &'a BuildIdentity,
    config: &'a RunConfig,
    store: &'a ArtifactStore,
    cancellation: &'a AtomicBool,
    regression_files: Option<&'a [PathBuf]>,
}
fn check(context: &CampaignContext<'_>, summary: &mut CampaignSummary) -> Result<()> {
    let CampaignContext {
        options: o,
        target,
        executable: exe,
        cwd,
        build,
        config,
        store,
        cancellation: cancel,
        regression_files,
    } = *context;
    let start = Instant::now();
    let mut counts = ExecutionCounts::default();
    let mut checked = BTreeSet::new();
    let mut failures = Vec::new();
    let input = o
        .input
        .as_ref()
        .map(|p| read_bounded(p, o.input_bytes))
        .transpose()?;
    let mut rng = o.seed;
    let case_count = regression_files.map_or(o.cases, |files| files.len() as u64);
    for index in 0..case_count {
        if cancel.load(Ordering::Acquire) {
            summary.status = CampaignStatus::Cancelled;
            break;
        }
        let remaining = Duration::from_millis(o.wall_ms).saturating_sub(start.elapsed());
        if remaining.is_zero() {
            summary.status = CampaignStatus::ResourceStop;
            break;
        }
        let timeout = remaining.min(Duration::from_millis(o.case_ms));
        let canonical = if let Some(files) = regression_files {
            read_bounded(&files[index as usize], o.input_bytes)?
        } else if let Some(input) = &input {
            input.clone()
        } else {
            let len = (index % o.input_bytes.min(64) + 1) as usize;
            let entropy = (0..len)
                .map(|_| {
                    rng ^= rng.wrapping_shl(13);
                    rng ^= rng.wrapping_shr(7);
                    rng ^= rng.wrapping_shl(17);
                    rng as u8
                })
                .collect();
            let call = worker_call(
                exe,
                cwd,
                &WorkerRequest::Generate { entropy },
                timeout,
                FRAME as usize * 4,
                cancel,
            )?;
            let terminal = classify_process(&call.process);
            if terminal != CampaignStatus::Sufficient {
                summary.status = if terminal == CampaignStatus::Finding {
                    CampaignStatus::InfrastructureFailure
                } else {
                    terminal
                };
                summary.cleanup = cleanup_label(&call.process.cleanup);
                break;
            }
            match call.responses.as_slice() {
                [WorkerResponse::Generated { canonical }] => canonical.clone(),
                [
                    WorkerResponse::GenerationRejected {
                        counts: rejected, ..
                    },
                ] if *rejected == ExecutionCounts::new(0, 1) => {
                    counts.record_generation_rejection()?;
                    continue;
                }
                _ => return Err(infrastructure("invalid owner generation response")),
            }
        };
        if canonical.len() as u64 > o.input_bytes {
            return Err(infrastructure(
                "owner canonical case exceeds selected ceiling",
            ));
        }
        let remaining = Duration::from_millis(o.wall_ms).saturating_sub(start.elapsed());
        if remaining.is_zero() {
            summary.status = CampaignStatus::ResourceStop;
            break;
        }
        let call = worker_call(
            exe,
            cwd,
            &WorkerRequest::Evaluate {
                canonical: canonical.clone(),
            },
            remaining.min(Duration::from_millis(o.case_ms)),
            FRAME as usize * 4,
            cancel,
        )?;
        summary.cleanup = cleanup_label(&call.process.cleanup);
        let terminal = classify_process(&call.process);
        let observed = evaluation(&call, &canonical, target)?;
        if let Some(outcome) = &observed {
            counts.record_outcome(outcome)?;
            checked.extend(outcome.checked_properties().iter().cloned());
            if outcome.failure_key().is_none() && outcome.is_evaluated() {
                store.add_corpus(&CaseArtifact::new(
                    target.clone(),
                    build.clone(),
                    config.clone(),
                    canonical,
                    outcome.clone(),
                )?)?;
            }
        }
        if let Some(artifact) = persist_call(target, build, config, store, &call, summary)? {
            failures.push(EvidenceStatus::Available { artifact });
        }
        if terminal != CampaignStatus::Sufficient
            || observed
                .as_ref()
                .is_some_and(|outcome| outcome.failure_key().is_some())
        {
            let finding_status = if observed
                .as_ref()
                .is_some_and(|outcome| outcome.failure_key().is_some())
            {
                combine_status(terminal, CampaignStatus::Finding)
            } else {
                terminal
            };
            summary.status = combine_status(summary.status, finding_status);
            if failures.is_empty() {
                failures.push(EvidenceStatus::Unavailable {
                    reason: EvidenceUnavailable::NoCurrentCase,
                });
            }
            if regression_files.is_some() && summary.status == CampaignStatus::Finding {
                continue;
            }
            break;
        }
        if observed.is_none() {
            return Err(infrastructure("successful owner exit omitted evaluation"));
        }
    }
    summary.evaluated = counts.evaluated();
    summary.generation_rejected = counts.generation_rejected();
    if summary.status == CampaignStatus::Sufficient {
        summary.status = if regression_files.is_some() {
            assessment(counts.assess_regressions())
        } else {
            assessment(target.assess_generated(counts))
        };
    }
    if summary.cleanup == "no worker started" {
        summary.cleanup = "complete".into();
    }
    summary.detail = if let Some(files) = regression_files {
        format!(
            "{} selected regression inputs in deterministic order; no generated cases; no exhaustive correctness claim",
            files.len()
        )
    } else {
        "finite fresh owner evaluations; no exhaustive correctness claim".into()
    };
    publish_run(context, counts, checked, failures, summary)
}
fn assessment(value: CoverageAssessment) -> CampaignStatus {
    match value {
        CoverageAssessment::Sufficient => CampaignStatus::Sufficient,
        CoverageAssessment::Insufficient => CampaignStatus::Insufficient,
        CoverageAssessment::Empty => CampaignStatus::Empty,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct CorpusBasis {
    schema: u32,
    target: TargetDescriptor,
    build: BuildIdentity,
    encoding: String,
}
fn directory_usage(path: &Path, max_entries: u64, max_bytes: u64) -> Result<(u64, u64)> {
    let mut entries = 0_u64;
    let mut bytes = 0_u64;
    for entry in fs::read_dir(path).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let m = entry.file_type().map_err(io_error)?;
        if !m.is_file() {
            return Err(invalid("engine corpus contains non-file entry"));
        }
        entries = entries
            .checked_add(1)
            .ok_or_else(|| invalid("entry overflow"))?;
        bytes = bytes
            .checked_add(entry.metadata().map_err(io_error)?.len())
            .ok_or_else(|| invalid("byte overflow"))?;
        if entries > max_entries || bytes > max_bytes {
            return Err(FuzzError::new(
                FuzzErrorKind::Resource,
                "engine corpus",
                "corpus exceeds quota",
            ));
        }
    }
    Ok((entries, bytes))
}
fn coverage(context: &CampaignContext<'_>, summary: &mut CampaignSummary) -> Result<()> {
    let CampaignContext {
        options: o,
        target,
        executable: exe,
        cwd,
        build,
        config,
        store,
        cancellation: cancel,
        ..
    } = *context;
    let corpus = cwd.join("entropy-corpus");
    fs::create_dir(&corpus).map_err(io_error)?;
    let basis = CorpusBasis {
        schema: 1,
        target: target.clone(),
        build: build.clone(),
        encoding: "engine entropy, not canonical codec bytes".into(),
    };
    if let Some(previous) = &o.restart {
        let metadata = previous
            .parent()
            .ok_or_else(|| invalid("restart corpus has no run directory"))?
            .join("corpus-basis.json");
        let prior: CorpusBasis = serde_json::from_slice(&read_bounded(&metadata, FRAME.into())?)
            .map_err(|e| invalid(e.to_string()))?;
        if prior.schema != 1
            || prior.target != basis.target
            || prior.build != basis.build
            || prior.encoding != basis.encoding
        {
            return Err(invalid("restart corpus/build inputs differ from this run"));
        }
        let (entries, bytes) = directory_usage(previous, o.storage_entries, o.storage_bytes)?;
        if entries.saturating_add(o.cases).saturating_add(2) > o.storage_entries
            || bytes.saturating_add(o.cases.saturating_add(2).saturating_mul(o.input_bytes))
                > o.storage_bytes
        {
            return Err(invalid(
                "restart corpus leaves insufficient conservative quota",
            ));
        }
        for entry in fs::read_dir(previous).map_err(io_error)? {
            let entry = entry.map_err(io_error)?;
            write_new(
                &corpus.join(entry.file_name()),
                &read_bounded(&entry.path(), o.input_bytes)?,
            )?;
        }
    } else {
        let seed = if let Some(input) = &o.input {
            read_bounded(input, o.input_bytes)?
        } else {
            vec![0]
        };
        write_new(&corpus.join("seed"), &seed)?;
    }
    write_new(
        &cwd.join("corpus-basis.json"),
        &serde_json::to_vec_pretty(&basis).map_err(|e| infrastructure(e.to_string()))?,
    )?;
    let engine_artifacts = cwd.join("engine-artifacts");
    fs::create_dir(&engine_artifacts).map_err(io_error)?;
    let rss_mb = o.memory_bytes.div_ceil(1024 * 1024);
    let mut env = BTreeMap::new();
    env.insert("SURGEIST_FUZZ_CASE_LIMIT".into(), o.cases.to_string().into());
    env.insert(
        "ASAN_OPTIONS".into(),
        "abort_on_error=1:detect_leaks=0".into(),
    );
    let args = vec![
        format!("-runs={}", o.cases).into(),
        format!("-max_total_time={}", o.wall_ms.div_ceil(1000)).into(),
        format!("-timeout={}", o.case_ms.div_ceil(1000)).into(),
        format!("-max_len={}", o.input_bytes).into(),
        format!("-rss_limit_mb={rss_mb}").into(),
        format!("-seed={}", o.seed).into(),
        "-print_final_stats=1".into(),
        format!("-artifact_prefix={}/", engine_artifacts.display()).into(),
        corpus.as_os_str().to_owned(),
    ];
    let output_limit = usize::try_from(o.storage_bytes.min(16 * 1024 * 1024))
        .map_err(|_| invalid("output quota exceeds address space"))?;
    let process = invoke(
        Invocation {
            tool: exe,
            cwd,
            args,
            environment: env,
            input: Vec::new(),
            output: output_limit,
            timeout: Duration::from_millis(o.wall_ms),
        },
        cancel,
    )?;
    summary.cleanup = cleanup_label(&process.cleanup);
    summary.status = classify_process(&process);
    summary.sampled_peak_rss_bytes = process.sampled_peak_rss_bytes;
    write_new(&cwd.join("engine.stderr"), &process.stderr.bytes)?;
    let responses = parse_frames(&process)?;
    let mut pending = None;
    let mut counts = ExecutionCounts::default();
    let mut checked = BTreeSet::new();
    let mut failures = Vec::new();
    let mut known_failure = false;
    for response in responses {
        match response {
            WorkerResponse::Captured { canonical } => {
                if pending.is_some() {
                    return Err(infrastructure(
                        "coverage capture without completing prior case",
                    ));
                }
                if canonical.len() as u64 > o.input_bytes {
                    return Err(infrastructure("coverage canonical input exceeds limit"));
                }
                pending = Some(canonical);
            }
            WorkerResponse::Evaluated {
                outcome,
                counts: new,
            } => {
                let input = pending
                    .take()
                    .ok_or_else(|| infrastructure("coverage result without capture"))?;
                target.validate_outcome(&outcome)?;
                if new.generation_rejected() != counts.generation_rejected()
                    || new.evaluated() != counts.evaluated() + u64::from(outcome.is_evaluated())
                {
                    return Err(infrastructure(
                        "coverage counters are not monotonic owner evaluations",
                    ));
                }
                counts = new;
                checked.extend(outcome.checked_properties().iter().cloned());
                let artifact = CaseArtifact::new(
                    target.clone(),
                    build.clone(),
                    config.clone(),
                    input,
                    outcome.clone(),
                )?;
                if outcome.failure_key().is_some() {
                    let path = store.publish_failure(&artifact)?;
                    summary.artifacts.push(path.as_str().into());
                    failures.push(EvidenceStatus::Available { artifact: path });
                    known_failure = true;
                } else if outcome.is_evaluated() {
                    store.add_corpus(&artifact)?;
                }
            }
            WorkerResponse::GenerationRejected { counts: new, .. } => {
                if pending.is_some()
                    || new.evaluated() != counts.evaluated()
                    || new.generation_rejected() != counts.generation_rejected() + 1
                {
                    return Err(infrastructure("invalid coverage rejection counters"));
                }
                counts = new;
            }
            _ => return Err(infrastructure("unexpected coverage protocol response")),
        }
    }
    if let Some(input) = pending {
        if summary.status == CampaignStatus::Sufficient {
            return Err(infrastructure("coverage exit omitted final evaluation"));
        }
        let artifact = CaptureArtifact::new(
            target.clone(),
            build.clone(),
            config.clone(),
            input,
            ProcessEvidence::from_outcome(&process)?,
        )?;
        let path = store.publish_capture(&artifact)?;
        summary.artifacts.push(path.as_str().into());
        failures.push(EvidenceStatus::Available { artifact: path });
    }
    if summary.status == CampaignStatus::Finding && failures.is_empty() {
        failures.push(EvidenceStatus::Unavailable {
            reason: EvidenceUnavailable::NoCurrentCase,
        });
    }
    if known_failure && process.cleanup == CleanupStatus::Complete {
        summary.status = CampaignStatus::Finding;
    }
    if summary.status == CampaignStatus::Sufficient {
        summary.status = assessment(target.assess_generated(counts));
    }
    directory_usage(&corpus, o.storage_entries, o.storage_bytes)?;
    directory_usage(&engine_artifacts, o.storage_entries, o.storage_bytes)?;
    summary.evaluated = counts.evaluated();
    summary.generation_rejected = counts.generation_rejected();
    summary.detail="ASan coverage feedback in engine.stderr; fresh run over persisted entropy corpus; no in-memory checkpoint; byte ceilings stop framed stdout; RSS engine guard is coarse, OS memory unbounded".into();
    publish_run(context, counts, checked, failures, summary)
}

fn status_for_error(error: &FuzzError) -> CampaignStatus {
    match error.kind() {
        FuzzErrorKind::Resource | FuzzErrorKind::Timeout => CampaignStatus::ResourceStop,
        FuzzErrorKind::InsufficientCoverage => CampaignStatus::Insufficient,
        FuzzErrorKind::Cli
        | FuzzErrorKind::Registry
        | FuzzErrorKind::InvalidIdentity
        | FuzzErrorKind::InvalidDescriptor
        | FuzzErrorKind::InvalidLimits
        | FuzzErrorKind::UnsupportedFormat
        | FuzzErrorKind::MissingCapability => CampaignStatus::InvalidRequest,
        _ => CampaignStatus::InfrastructureFailure,
    }
}

fn replay_or_minimize(context: &CampaignContext<'_>, summary: &mut CampaignSummary) -> Result<()> {
    let CampaignContext {
        options,
        target,
        executable,
        cwd: work,
        build: identity,
        config,
        store,
        cancellation,
        ..
    } = *context;
    let path = crate::RelativePath::new(
        options
            .artifact
            .as_ref()
            .ok_or_else(|| invalid("artifact required"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let mode = if options.comparison {
        ReplayMode::Comparison
    } else {
        ReplayMode::Exact
    };
    if path.as_str().starts_with("captures/") {
        if options.command == "minimize" {
            return Err(invalid(
                "capture without a semantic failure key cannot be minimized",
            ));
        }
        let original = store.load_capture(&path)?;
        original.validate_replay(target, identity, mode)?;
        let call = worker_call(
            executable,
            work,
            &WorkerRequest::Evaluate {
                canonical: original.input().to_vec(),
            },
            Duration::from_millis(options.case_ms),
            FRAME as usize * 4,
            cancellation,
        )?;
        let observation = evaluation(&call, original.input(), target)?;
        summary.status = classify_process(&call.process);
        summary.cleanup = cleanup_label(&call.process.cleanup);
        if let Some(outcome) = observation {
            summary.evaluated = u64::from(outcome.is_evaluated());
            if outcome.failure_key().is_some() {
                summary.status = combine_status(summary.status, CampaignStatus::Finding);
            }
        }
        let record = replay_capture_observation(
            &original,
            target,
            identity,
            mode,
            ProcessEvidence::from_outcome(&call.process)?,
        )?;
        summary
            .artifacts
            .push(store.publish_capture_replay(&record)?.as_str().into());
        persist_call(target, identity, config, store, &call, summary)?;
        summary.detail =
            "isolated process capture replay; semantic reproduction requires a reported key".into();
        return Ok(());
    }
    let original = store.load_failure(&path)?;
    original.validate_replay(target, identity, mode)?;
    if options.command == "replay" {
        let call = worker_call(
            executable,
            work,
            &WorkerRequest::Evaluate {
                canonical: original.input().to_vec(),
            },
            Duration::from_millis(options.case_ms),
            FRAME as usize * 4,
            cancellation,
        )?;
        let observation = evaluation(&call, original.input(), target)?;
        summary.status = classify_process(&call.process);
        summary.cleanup = cleanup_label(&call.process.cleanup);
        if let Some(outcome) = observation {
            summary.evaluated = u64::from(outcome.is_evaluated());
            if outcome.failure_key().is_some() {
                summary.status = combine_status(summary.status, CampaignStatus::Finding);
            }
            summary.detail =
                if outcome.failure_key() == original.metadata().outcome().failure_key() {
                    "original failure key reproduced"
                } else {
                    "different semantic observation; inspect comparison record"
                }
                .into();
            let record = replay_observation(&original, target, identity, mode, outcome)?;
            summary
                .artifacts
                .push(store.publish_replay(&record)?.as_str().into());
        } else {
            summary.detail = "replay terminated without a semantic observation".into();
        }
        persist_call(target, identity, config, store, &call, summary)?;
        return Ok(());
    }
    let run = next_run(work)?;
    summary.runs.push(run.to_string_lossy().into_owned());
    let limits = crate::reduction::ReductionLimits::new(
        options.attempts,
        Duration::from_millis(options.wall_ms),
        options.storage_bytes.min(16 * 1024 * 1024),
        options.confirmations,
    )?;
    let remote = RemoteTarget {
        descriptor: target,
        executable,
        cwd: &run,
        cancellation,
        case_deadline: Duration::from_millis(options.case_ms),
        deadline: Instant::now() + Duration::from_millis(options.wall_ms),
        max_candidates: u32::try_from(options.attempts)
            .map_err(|_| invalid("attempts exceeds worker protocol"))?,
        max_output: FRAME as usize * 4,
        pending_output: std::cell::Cell::new(0),
    };
    let expected = original
        .metadata()
        .outcome()
        .failure_key()
        .ok_or_else(|| invalid("failure artifact lacks key"))?;
    let report = crate::reduction::reduce_with(
        &remote,
        original.input(),
        expected,
        limits,
        cancellation,
        |input, budget| {
            let discovery = remote.pending_output.replace(0);
            if discovery >= budget.output_bytes {
                return Ok(crate::reduction::ReductionEvaluation::new(
                    CaseOutcome::unsupported("candidate discovery consumed output budget")?,
                    discovery,
                ));
            }
            let output =
                usize::try_from((budget.output_bytes - discovery).min(remote.max_output as u64))
                    .map_err(|_| invalid("replay output exceeds address space"))?;
            let call = worker_call(
                executable,
                &run,
                &WorkerRequest::Evaluate {
                    canonical: input.to_vec(),
                },
                budget.remaining_time.min(remote.case_deadline),
                output,
                cancellation,
            )?;
            summary.cleanup = cleanup_label(&call.process.cleanup);
            let observation = evaluation(&call, input, target)?.unwrap_or(
                CaseOutcome::unsupported("isolated replay ended without semantic identity")?,
            );
            Ok(crate::reduction::ReductionEvaluation::new(
                observation,
                discovery
                    + call.process.stdout.bytes.len() as u64
                    + call.process.stderr.bytes.len() as u64,
            ))
        },
    )?;
    let mut reduced = None;
    if report.final_confirmed() {
        let outcome = CaseOutcome::failure(
            expected.clone(),
            "same failure reconfirmed after bounded reduction",
        )?;
        let artifact = CaseArtifact::new(
            target.clone(),
            identity.clone(),
            config.clone(),
            report.best_input().to_vec(),
            outcome,
        )?;
        let published = store.publish_failure(&artifact)?;
        summary.artifacts.push(published.as_str().into());
        reduced = Some(published);
    }
    summary.evaluated = report
        .observations()
        .iter()
        .filter(|observation| observation.outcome.is_evaluated())
        .count() as u64;
    // The report subscriber preserves links and observations under the common
    // store's immutable publication and quotas.
    summary.artifacts.push(
        store
            .publish_reduction(&path, reduced.as_ref(), &report)?
            .as_str()
            .into(),
    );
    summary.status = match report.status() {
        crate::reduction::ReductionStatus::Cancelled => CampaignStatus::Cancelled,
        crate::reduction::ReductionStatus::AttemptLimit
        | crate::reduction::ReductionStatus::TimeLimit
        | crate::reduction::ReductionStatus::OutputLimit => CampaignStatus::ResourceStop,
        crate::reduction::ReductionStatus::Complete if report.final_confirmed() => {
            CampaignStatus::Finding
        }
        crate::reduction::ReductionStatus::UnsupportedFailure => CampaignStatus::InvalidRequest,
        _ => CampaignStatus::Insufficient,
    };
    summary.detail = format!(
        "{:?}; original {} bytes; best {} bytes; {} replays; final confirmed={}",
        report.status(),
        report.original_input().len(),
        report.best_input().len(),
        report.replay_count(),
        report.final_confirmed()
    );
    Ok(())
}

// Canonical bytes are transport here. Domain decode, validation, semantics and
// candidate generation remain entirely in the isolated owner binary.
struct RemoteTarget<'a> {
    descriptor: &'a TargetDescriptor,
    executable: &'a Path,
    cwd: &'a Path,
    cancellation: &'a AtomicBool,
    case_deadline: Duration,
    deadline: Instant,
    max_candidates: u32,
    max_output: usize,
    pending_output: std::cell::Cell<u64>,
}
impl CaseTarget for RemoteTarget<'_> {
    type Case = Vec<u8>;
    fn descriptor(&self) -> &TargetDescriptor {
        self.descriptor
    }
    fn decode(&self, input: &[u8]) -> Result<Vec<u8>> {
        if input.len() as u64 > self.descriptor.limits().bytes() {
            return Err(invalid("transport case exceeds owner limit"));
        }
        Ok(input.to_vec())
    }
    fn encode(&self, case: &Vec<u8>) -> Result<Vec<u8>> {
        self.decode(case)
    }
    fn generate(&self, _: &[u8]) -> Result<Generation<Vec<u8>>> {
        Err(invalid("reduction transport does not generate cases"))
    }
    fn evaluate(&self, _: &Vec<u8>) -> Result<CaseOutcome> {
        Err(invalid(
            "reduction evaluation requires the isolated callback",
        ))
    }
    fn reduction_candidates(
        &self,
        case: &Vec<u8>,
        emit: &mut dyn FnMut(Vec<u8>) -> bool,
    ) -> Result<()> {
        let remaining = self.deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            return Err(FuzzError::new(
                FuzzErrorKind::Timeout,
                "candidate generation",
                "reduction deadline expired",
            ));
        }
        let call = worker_call(
            self.executable,
            self.cwd,
            &WorkerRequest::Reduce {
                canonical: case.clone(),
                max_candidates: self.max_candidates,
            },
            remaining.min(self.case_deadline),
            self.max_output,
            self.cancellation,
        )?;
        self.pending_output
            .set(self.pending_output.get().saturating_add(
                call.process.stdout.bytes.len() as u64 + call.process.stderr.bytes.len() as u64,
            ));
        require_success(&call.process, "owner reduction candidates")?;
        let Some(WorkerResponse::Reduced { emitted }) = call.responses.last() else {
            return Err(infrastructure("candidate stream lacks completion"));
        };
        if *emitted as usize != call.responses.len() - 1 || *emitted > self.max_candidates {
            return Err(infrastructure("candidate stream count mismatch"));
        }
        for response in &call.responses[..call.responses.len() - 1] {
            let WorkerResponse::Candidate { canonical } = response else {
                return Err(infrastructure("unexpected candidate response"));
            };
            if !emit(self.decode(canonical)?) {
                break;
            }
        }
        Ok(())
    }
}

/// Runs one finite command. Invalid preflight requests return without creating
/// campaign outputs. Every subprocess uses the shared process executor.
pub fn run_cli(args: &[OsString], cancellation: &AtomicBool) -> Result<CampaignSummary> {
    let options = Options::parse(args)?;
    if let Some(path) = &options.cancel_file {
        let done = AtomicBool::new(false);
        std::thread::scope(|scope| {
            struct StopOnDrop<'a>(&'a AtomicBool);
            impl Drop for StopOnDrop<'_> {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::Release);
                }
            }
            let _stop = StopOnDrop(&done);
            let monitor = scope.spawn(|| {
                while !done.load(Ordering::Acquire) {
                    if path.exists() {
                        cancellation.store(true, Ordering::Release);
                        break;
                    }
                    std::thread::park_timeout(Duration::from_millis(10));
                }
            });
            let result = run_options(&options, cancellation);
            done.store(true, Ordering::Release);
            monitor
                .join()
                .map_err(|_| infrastructure("cancellation monitor panicked"))?;
            result
        })
    } else {
        run_options(&options, cancellation)
    }
}
fn run_options(options: &Options, cancellation: &AtomicBool) -> Result<CampaignSummary> {
    let mut summary = CampaignSummary::new(&options.command);
    if cancellation.load(Ordering::Acquire) {
        summary.status = CampaignStatus::Cancelled;
        summary.detail = "cancelled before preflight; no worker started".into();
        return Ok(summary);
    }
    let preflight = preflight(options, cancellation)?;
    summary.targets = preflight
        .selected
        .iter()
        .map(|t| t.descriptor().clone())
        .collect();
    if matches!(options.command.as_str(), "list" | "validate") {
        summary.detail = format!(
            "{} registered targets; no builds or campaign writes",
            preflight.registry.targets().count()
        );
        return Ok(summary);
    }
    if options.command == "report" {
        let work = options
            .work
            .as_ref()
            .ok_or_else(|| invalid("report requires --work"))?;
        if fs::read(work.join(".surgeist-fuzz-owned")).map_err(io_error)? != MARKER {
            return Err(invalid("work directory lacks ownership marker"));
        }
        let artifact_store = ArtifactStore::open(
            work.join("artifacts"),
            StorageLimits::new(
                options.storage_entries,
                options.storage_bytes,
                options.input_bytes,
                FRAME.into(),
            )?,
        )?;
        let recovery = artifact_store.recovery_report()?;
        summary.status = if recovery.published.is_empty() {
            CampaignStatus::Empty
        } else {
            CampaignStatus::Sufficient
        };
        summary.artifacts = recovery
            .published
            .iter()
            .map(|p| p.as_str().to_owned())
            .collect();
        summary.detail = format!(
            "published={}, partial={}, bytes={}",
            recovery.published.len(),
            recovery.partial.len(),
            recovery.bytes
        );
        return Ok(summary);
    }
    if preflight
        .regression_files
        .as_ref()
        .is_some_and(Vec::is_empty)
    {
        summary.status = CampaignStatus::Empty;
        summary.detail =
            "empty selected regression directory; no generated cases or campaign writes".into();
        return Ok(summary);
    }
    let target = &preflight.selected[0];
    let work = create_work(options)?;
    let (ordinary, mut identity) = build(options, &preflight, target, &work, false, cancellation)?;
    let coverage_needed = options.command == "run"
        || (matches!(options.command.as_str(), "replay" | "minimize")
            && origin_requires_coverage(options, &work)?);
    let coverage_build = if coverage_needed {
        let (executable, engine_identity) =
            build(options, &preflight, target, &work, true, cancellation)?;
        identity = combine_engine_identity(&identity, &engine_identity)?;
        Some(executable)
    } else {
        None
    };
    // Handshake before any semantic call. The ordinary worker is the exact
    // canonical replay executable even when the coverage build is separate.
    handshake(&ordinary, &work, target.descriptor(), options, cancellation)?;
    summary.builds.push(identity.clone());
    summary.memory_policy = "explicit unbounded OS memory; RSS sampling unavailable".into();
    let artifact_store = store(options, &work)?;
    let config = run_config(options)?;
    if matches!(options.command.as_str(), "replay" | "minimize") {
        replay_or_minimize(
            &CampaignContext {
                options,
                target: target.descriptor(),
                executable: &ordinary,
                cwd: &work,
                build: &identity,
                config: &config,
                store: &artifact_store,
                cancellation,
                regression_files: None,
            },
            &mut summary,
        )?;
        return Ok(summary);
    }
    let run_directory = next_run(&work)?;
    summary
        .runs
        .push(run_directory.to_string_lossy().into_owned());
    if options.workers == 1 {
        if let Some(executable) = coverage_build {
            coverage(
                &CampaignContext {
                    options,
                    target: target.descriptor(),
                    executable: &executable,
                    cwd: &run_directory,
                    build: &identity,
                    config: &config,
                    store: &artifact_store,
                    cancellation,
                    regression_files: None,
                },
                &mut summary,
            )?;
        } else {
            check(
                &CampaignContext {
                    options,
                    target: target.descriptor(),
                    executable: &ordinary,
                    cwd: &run_directory,
                    build: &identity,
                    config: &config,
                    store: &artifact_store,
                    cancellation,
                    regression_files: preflight.regression_files.as_deref(),
                },
                &mut summary,
            )?;
        }
    } else {
        run_parallel(
            &CampaignContext {
                options,
                target: target.descriptor(),
                executable: &ordinary,
                cwd: &run_directory,
                build: &identity,
                config: &config,
                store: &artifact_store,
                cancellation,
                regression_files: None,
            },
            coverage_build.as_deref(),
            &mut summary,
        )?;
    }
    if !cancellation.load(Ordering::Acquire)
        && clean_source(options, cancellation)?
            != *preflight.source.as_ref().expect("campaign source")
    {
        return Err(infrastructure("prepared source changed during campaign"));
    }
    Ok(summary)
}

fn run_config(options: &Options) -> Result<RunConfig> {
    RunConfig::new(RunConfigSpec {
        cases: options.cases,
        wall_millis: options.wall_ms,
        case_millis: options.case_ms,
        memory_bytes: options.memory_bytes,
        seed: Some(options.seed),
    })
}
fn next_run(work: &Path) -> Result<PathBuf> {
    let runs = work.join("sessions");
    fs::create_dir_all(&runs).map_err(io_error)?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| infrastructure(e.to_string()))?
        .as_nanos();
    let directory = runs.join(format!(
        "{}-{}-{}",
        stamp,
        std::process::id(),
        RUN_SEQUENCE.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&directory).map_err(io_error)?;
    Ok(directory)
}
fn combine_engine_identity(
    ordinary: &BuildIdentity,
    engine: &BuildIdentity,
) -> Result<BuildIdentity> {
    let mut spec: BuildIdentitySpec = serde_json::from_slice(
        &serde_json::to_vec(ordinary).map_err(|e| infrastructure(e.to_string()))?,
    )
    .map_err(|e| infrastructure(e.to_string()))?;
    spec.providers.push(ContentIdentity::new(
        ContentRole::Engine,
        "coverage-executable",
        engine.executable().clone(),
    )?);
    spec.providers.push(ContentIdentity::new(
        ContentRole::Engine,
        "coverage-build",
        Sha256Digest::from_bytes(
            serde_json::to_vec(engine).map_err(|e| infrastructure(e.to_string()))?,
        ),
    )?);
    spec.engine = "libfuzzer-sys 0.4.13; canonical replay via isolated owner protocol".into();
    spec.sanitizer = "address; sanitizer-coverage-level=4; coverage feature in engine build".into();
    BuildIdentity::new(spec)
}
fn origin_requires_coverage(options: &Options, work: &Path) -> Result<bool> {
    let store = ArtifactStore::open(
        work.join("artifacts"),
        StorageLimits::new(
            options.storage_entries,
            options.storage_bytes,
            options.input_bytes,
            FRAME.into(),
        )?,
    )?;
    let path = crate::RelativePath::new(
        options
            .artifact
            .as_ref()
            .ok_or_else(|| invalid("artifact required"))?,
    )
    .map_err(|e| invalid(e.to_string()))?;
    let providers = if path.as_str().starts_with("failures/") {
        store
            .load_failure(&path)?
            .metadata()
            .build()
            .providers()
            .to_vec()
    } else if path.as_str().starts_with("captures/") {
        store
            .load_capture(&path)?
            .metadata()
            .build()
            .providers()
            .to_vec()
    } else {
        return Err(invalid(
            "artifact must name immutable failure or capture evidence",
        ));
    };
    Ok(providers.iter().any(|p| p.role() == ContentRole::Engine))
}

fn run_parallel(
    context: &CampaignContext<'_>,
    coverage_executable: Option<&Path>,
    summary: &mut CampaignSummary,
) -> Result<()> {
    let CampaignContext {
        options,
        target,
        executable: ordinary,
        cwd: run,
        build: identity,
        store,
        cancellation,
        ..
    } = *context;
    let count = u64::from(options.workers);
    let mut partitions = Vec::new();
    for worker in 0..options.workers {
        let mut partition = options.clone();
        partition.workers = 1;
        partition.cases =
            options.cases / count + u64::from(u64::from(worker) < options.cases % count);
        partition.storage_bytes = options.storage_bytes / count;
        partition.storage_entries = options.storage_entries / count;
        partition.seed = options.seed.wrapping_add(u64::from(worker));
        let config = run_config(&partition)?;
        let directory = run.join(format!("worker-{worker}"));
        fs::create_dir(&directory).map_err(io_error)?;
        partitions.push((partition, config, directory));
    }
    let (results, errors) = std::thread::scope(|scope| {
        let handles: Vec<_> = partitions
            .into_iter()
            .map(|(partition, config, directory)| {
                scope.spawn(move || -> Result<CampaignSummary> {
                    struct CancelOnPanic<'a>(&'a AtomicBool);
                    impl Drop for CancelOnPanic<'_> {
                        fn drop(&mut self) {
                            if std::thread::panicking() {
                                self.0.store(true, Ordering::Release);
                            }
                        }
                    }
                    let _panic = CancelOnPanic(cancellation);
                    let mut result = CampaignSummary::new(&partition.command);
                    let context = CampaignContext {
                        options: &partition,
                        target,
                        executable: coverage_executable.unwrap_or(ordinary),
                        cwd: &directory,
                        build: identity,
                        config: &config,
                        store,
                        cancellation,
                        regression_files: None,
                    };
                    let execution = if coverage_executable.is_some() {
                        coverage(&context, &mut result)
                    } else {
                        check(&context, &mut result)
                    };
                    if execution.is_err() || result.cleanup.starts_with("failed:") {
                        cancellation.store(true, Ordering::Release);
                    }
                    execution.map(|()| result)
                })
            })
            .collect();
        let mut results = Vec::new();
        let mut errors = Vec::new();
        // Every handle is explicitly joined and inspected, including after an
        // earlier failure. Scoped implicit joins cannot hide cleanup outcomes.
        for (worker, handle) in handles.into_iter().enumerate() {
            match handle.join() {
                Ok(Ok(result)) => results.push((worker, result)),
                Ok(Err(error)) => {
                    cancellation.store(true, Ordering::Release);
                    errors.push(format!("worker {worker}: {error}"));
                }
                Err(_) => {
                    cancellation.store(true, Ordering::Release);
                    errors.push(format!("worker {worker}: supervisor panicked"));
                }
            }
        }
        (results, errors)
    });
    summary.status = CampaignStatus::Sufficient;
    summary.cleanup = "complete".into();
    let mut cleanup_failures = Vec::new();
    for (worker, result) in results {
        summary.evaluated = summary
            .evaluated
            .checked_add(result.evaluated)
            .ok_or_else(|| infrastructure("count overflow"))?;
        summary.generation_rejected = summary
            .generation_rejected
            .checked_add(result.generation_rejected)
            .ok_or_else(|| infrastructure("count overflow"))?;
        summary.artifacts.extend(result.artifacts);
        if result.status != CampaignStatus::Sufficient {
            summary.status = combine_status(summary.status, result.status);
        }
        if result.cleanup != "complete" && result.cleanup != "no worker started" {
            cleanup_failures.push(format!("worker {worker}: {}", result.cleanup));
        }
    }
    if !cleanup_failures.is_empty() {
        summary.cleanup = cleanup_failures.join("; ");
    }
    if !errors.is_empty() {
        summary.cleanup = format!(
            "worker error prevented complete cleanup evidence; {}",
            summary.cleanup
        );
        summary.status = CampaignStatus::InfrastructureFailure;
        summary.detail = format!("{}; cleanup={}", errors.join("; "), summary.cleanup);
        return Ok(());
    }
    if matches!(
        summary.status,
        CampaignStatus::Sufficient | CampaignStatus::Insufficient | CampaignStatus::Empty
    ) {
        summary.status = assessment(target.assess_generated(ExecutionCounts::new(
            summary.evaluated,
            summary.generation_rejected,
        )));
    }
    summary.detail = format!(
        "{} explicitly selected workers; total execution budget partitioned across workers",
        options.workers
    );
    Ok(())
}
fn combine_status(left: CampaignStatus, right: CampaignStatus) -> CampaignStatus {
    fn rank(value: CampaignStatus) -> u8 {
        match value {
            CampaignStatus::InfrastructureFailure => 8,
            CampaignStatus::InvalidRequest => 7,
            CampaignStatus::Finding => 6,
            CampaignStatus::Cancelled => 5,
            CampaignStatus::ResourceStop => 4,
            CampaignStatus::Insufficient => 3,
            CampaignStatus::Empty => 2,
            CampaignStatus::Sufficient => 1,
        }
    }
    if rank(right) > rank(left) {
        right
    } else {
        left
    }
}

/// CLI entry point; Ctrl-C and SIGTERM request cooperative cleanup through the
/// common executor. A cancellation file is also available for automation.
pub fn main_entry(args: impl IntoIterator<Item = OsString>) -> i32 {
    let args: Vec<_> = args.into_iter().collect();
    if let Err(error) = parse_cli(&args)
        && matches!(
            error.kind(),
            clap::error::ErrorKind::DisplayHelp | clap::error::ErrorKind::DisplayVersion
        )
    {
        return match error.print() {
            Ok(()) => 0,
            Err(_) => 3,
        };
    }
    let cancel = Arc::new(AtomicBool::new(false));
    let signal = Arc::clone(&cancel);
    if let Err(error) = ctrlc::set_handler(move || {
        signal.store(true, Ordering::Release);
    }) {
        eprintln!("cannot install cancellation handler: {error}");
        return CampaignStatus::InfrastructureFailure.exit_code();
    }
    let result = run_cli(&args, &cancel);
    let summary = match result {
        Ok(summary) => summary,
        Err(error) => {
            let mut summary =
                CampaignSummary::new(args.first().and_then(|a| a.to_str()).unwrap_or("unknown"));
            summary.status = if cancel.load(Ordering::Acquire) {
                CampaignStatus::Cancelled
            } else {
                status_for_error(&error)
            };
            summary.detail = error.to_string();
            summary
        }
    };
    match serde_json::to_string_pretty(&summary) {
        Ok(json) => println!("{json}"),
        Err(error) => {
            eprintln!("cannot serialize campaign summary: {error}");
            return 3;
        }
    }
    eprintln!(
        "{}: {:?}; evaluated={}, rejected={}",
        summary.command, summary.status, summary.evaluated, summary.generation_rejected
    );
    summary.status.exit_code()
}
