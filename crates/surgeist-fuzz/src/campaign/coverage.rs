//! Streaming coverage observations and bounded logical campaign rounds.
use super::*;
use crate::execution::{ConsumerFailure, OutputStream};
use crate::retention::{RetentionReport, SessionLimits, SessionStore};

const CHECKPOINT_BYTES: u64 = 256 * 1024;

fn resource(detail: impl Into<String>) -> FuzzError {
    FuzzError::new(FuzzErrorKind::Resource, "coverage campaign", detail)
}
fn add(left: u64, right: u64) -> Result<u64> {
    left.checked_add(right)
        .ok_or_else(|| resource("campaign counter exhausted"))
}
fn combine(left: ExecutionCounts, right: ExecutionCounts) -> Result<ExecutionCounts> {
    Ok(ExecutionCounts::new(
        add(left.evaluated(), right.evaluated())?,
        add(left.generation_rejected(), right.generation_rejected())?,
    ))
}
fn attempts(counts: ExecutionCounts) -> Result<u64> {
    add(counts.evaluated(), counts.generation_rejected())
}
fn limits(o: &Options) -> Result<SessionLimits> {
    SessionLimits::new(
        o.session_entries,
        o.session_bytes,
        o.input_bytes,
        o.round_cases,
        o.retained_entries,
        o.log_bytes,
        CHECKPOINT_BYTES,
    )
}
pub(super) fn validate_options(o: &Options) -> Result<()> {
    if o.round_ms == 0
        || o.round_ms > 86_400_000
        || o.case_ms > o.round_ms
        || o.progress_ms == 0
        || o.progress_ms > 60_000
        || o.log_bytes == 0
        || o.log_bytes > 16 * 1024 * 1024
    {
        return Err(invalid(
            "positive round/progress budgets required; case deadline must fit round; progress at most 60 seconds; log at most 16 MiB",
        ));
    }
    limits(o)?;
    // Two exact case bundles (reported failure and interrupted capture) and one
    // terminal record. Successful coverage observations consume none of this.
    if o.storage_entries < 3
        || o.storage_bytes
            < o.input_bytes
                .saturating_mul(2)
                .saturating_add(u64::from(FRAME) * 3)
    {
        return Err(invalid(
            "immutable evidence quota cannot reserve two exact cases and a terminal record",
        ));
    }
    Ok(())
}

#[derive(Serialize)]
#[serde(rename_all = "kebab-case")]
enum Mode {
    Finite,
    Continuous,
}
#[derive(Serialize)]
struct Progress<'a> {
    schema: u32,
    mode: Mode,
    completed_rounds: u64,
    evaluated: u64,
    generation_rejected: u64,
    checked_properties: &'a BTreeSet<crate::PropertyId>,
    build: &'a BuildIdentity,
    round_cases: u64,
    retained_limit: u64,
    round_millis: u64,
    retained_entries: u64,
    retained_bytes: u64,
    discarded_entries: u64,
    discarded_bytes: u64,
    terminal: Option<CampaignStatus>,
    elapsed_millis: u64,
    stdout_bytes: u64,
    stderr_bytes: u64,
    diagnostic_tail_truncated: bool,
}
struct Totals {
    counts: ExecutionCounts,
    stdout_bytes: u64,
    stderr_bytes: u64,
    diagnostic_tail_truncated: bool,
    checked: BTreeSet<crate::PropertyId>,
    rounds: u64,
    retained_entries: u64,
    retained_bytes: u64,
    discarded_entries: u64,
    discarded_bytes: u64,
}
impl Totals {
    fn new(report: RetentionReport) -> Self {
        Self {
            counts: ExecutionCounts::default(),
            stdout_bytes: 0,
            stderr_bytes: 0,
            diagnostic_tail_truncated: false,
            checked: BTreeSet::new(),
            rounds: 0,
            retained_entries: report.retained_entries,
            retained_bytes: report.retained_bytes,
            discarded_entries: report.discarded_entries,
            discarded_bytes: report.discarded_bytes,
        }
    }
    fn retained(&mut self, report: RetentionReport) -> Result<()> {
        self.retained_entries = report.retained_entries;
        self.retained_bytes = report.retained_bytes;
        self.discarded_entries = add(self.discarded_entries, report.discarded_entries)?;
        self.discarded_bytes = add(self.discarded_bytes, report.discarded_bytes)?;
        Ok(())
    }
}
struct Checkpoint<'a> {
    options: &'a Options,
    build: &'a BuildIdentity,
    round_cases: u64,
    retained_limit: u64,
    started: Instant,
}
impl Checkpoint<'_> {
    fn write(
        &self,
        session: &mut SessionStore,
        totals: &Totals,
        current: ExecutionCounts,
        checked: &BTreeSet<crate::PropertyId>,
        terminal: Option<CampaignStatus>,
    ) -> Result<()> {
        let counts = combine(totals.counts, current)?;
        let properties = totals.checked.union(checked).cloned().collect();
        let progress = Progress {
            schema: 1,
            mode: if self.options.continuous {
                Mode::Continuous
            } else {
                Mode::Finite
            },
            completed_rounds: totals.rounds,
            evaluated: counts.evaluated(),
            generation_rejected: counts.generation_rejected(),
            checked_properties: &properties,
            build: self.build,
            round_cases: self.round_cases,
            retained_limit: self.retained_limit,
            round_millis: self.options.round_ms,
            retained_entries: totals.retained_entries,
            retained_bytes: totals.retained_bytes,
            discarded_entries: totals.discarded_entries,
            discarded_bytes: totals.discarded_bytes,
            terminal,
            stdout_bytes: totals.stdout_bytes,
            stderr_bytes: totals.stderr_bytes,
            diagnostic_tail_truncated: totals.diagnostic_tail_truncated,
            elapsed_millis: u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX),
        };
        session.replace_checkpoint(
            &serde_json::to_vec(&progress).map_err(|e| infrastructure(e.to_string()))?,
        )
    }
}

// The current capture is cleared only after the complete observation has been
// validated and any immutable failure has been published successfully.
struct Observations<'a> {
    context: &'a CampaignContext<'a>,
    pending: Option<Vec<u8>>,
    counts: ExecutionCounts,
    checked: BTreeSet<crate::PropertyId>,
    failures: Vec<EvidenceStatus>,
    known_failure: bool,
}
impl<'a> Observations<'a> {
    fn new(context: &'a CampaignContext<'a>) -> Self {
        Self {
            context,
            pending: None,
            counts: ExecutionCounts::default(),
            checked: BTreeSet::new(),
            failures: Vec::new(),
            known_failure: false,
        }
    }
    fn accept(&mut self, response: WorkerResponse) -> Result<()> {
        match response {
            WorkerResponse::Captured { canonical } => {
                if self.pending.is_some() {
                    return Err(infrastructure(
                        "coverage capture without completing prior case",
                    ));
                }
                if canonical.len() as u64 > self.context.options.input_bytes {
                    return Err(infrastructure("coverage canonical input exceeds limit"));
                }
                self.pending = Some(canonical);
            }
            WorkerResponse::Evaluated {
                outcome,
                counts: new,
            } => {
                let input = self
                    .pending
                    .as_ref()
                    .ok_or_else(|| infrastructure("coverage result without capture"))?;
                self.context.target.validate_outcome(&outcome)?;
                if new.generation_rejected() != self.counts.generation_rejected()
                    || new.evaluated()
                        != add(self.counts.evaluated(), u64::from(outcome.is_evaluated()))?
                    || attempts(new)? > self.context.config.cases()
                {
                    return Err(infrastructure(
                        "coverage counters exceed acknowledged owner evaluations",
                    ));
                }
                if outcome.failure_key().is_some() {
                    if self.known_failure {
                        return Err(infrastructure(
                            "coverage emitted another finding after its first failure",
                        ));
                    }
                    let artifact = CaseArtifact::new(
                        self.context.target.clone(),
                        self.context.build.clone(),
                        self.context.config.clone(),
                        input.clone(),
                        outcome.clone(),
                    )?;
                    let path = self.context.store.publish_failure(&artifact)?;
                    self.failures
                        .push(EvidenceStatus::Available { artifact: path });
                    self.known_failure = true;
                }
                self.counts = new;
                self.checked
                    .extend(outcome.checked_properties().iter().cloned());
                self.pending = None;
            }
            WorkerResponse::GenerationRejected { counts: new, .. } => {
                if self.pending.is_some()
                    || new.evaluated() != self.counts.evaluated()
                    || new.generation_rejected() != add(self.counts.generation_rejected(), 1)?
                    || attempts(new)? > self.context.config.cases()
                {
                    return Err(infrastructure("invalid coverage rejection counters"));
                }
                self.counts = new;
            }
            _ => return Err(infrastructure("unexpected coverage protocol response")),
        }
        Ok(())
    }
}

fn initial_inputs(o: &Options, basis: &CorpusBasis) -> Result<Vec<Vec<u8>>> {
    if let Some(previous) = &o.restart {
        let prior = read_corpus_basis(previous)?;
        if prior.schema != basis.schema
            || prior.target != basis.target
            || prior.build != basis.build
            || prior.encoding != basis.encoding
        {
            return Err(invalid("restart corpus/build inputs differ from this run"));
        }
        directory_usage(previous, o.session_entries, o.session_bytes)?;
        let mut inputs = Vec::new();
        let mut bytes = 0_u64;
        for entry in fs::read_dir(previous).map_err(io_error)? {
            if inputs.len() as u64 >= o.session_entries {
                return Err(resource("restart corpus grew past entry limit"));
            }
            let entry = entry.map_err(io_error)?;
            if !entry.file_type().map_err(io_error)?.is_file() {
                return Err(invalid("restart corpus has nonregular input"));
            }
            let input = read_bounded(&entry.path(), o.input_bytes)?;
            bytes = add(bytes, input.len() as u64)?;
            if bytes > o.session_bytes {
                return Err(resource("restart corpus grew past byte limit"));
            }
            inputs.push(input);
        }
        if inputs.is_empty() {
            return Err(invalid("restart corpus is empty"));
        }
        Ok(inputs)
    } else {
        Ok(vec![if let Some(input) = &o.input {
            read_bounded(input, o.input_bytes)?
        } else {
            vec![0]
        }])
    }
}

fn reserve_evidence(context: &CampaignContext<'_>) -> Result<()> {
    let report = context.store.recovery_report()?;
    let used = report.published.len() as u64 + report.partial.len() as u64;
    let o = context.options;
    let bytes = o
        .input_bytes
        .saturating_mul(2)
        .saturating_add(u64::from(FRAME) * 3);
    if used.saturating_add(3) > o.storage_entries
        || report.bytes.saturating_add(bytes) > o.storage_bytes
    {
        return Err(resource(
            "insufficient immutable evidence reserve; existing findings were preserved",
        ));
    }
    Ok(())
}

// ProcessEvidence has a smaller diagnostic ceiling than session logs. Select
// its newest bytes here so its ordinary prefix conversion preserves our tail.
fn capture_evidence(process: &ProcessOutcome) -> Result<ProcessEvidence> {
    let mut evidence = process.clone();
    if evidence.stderr.bytes.len() > 32 * 1024 {
        evidence
            .stderr
            .bytes
            .drain(..evidence.stderr.bytes.len() - 32 * 1024);
        evidence.stderr.truncated = true;
    }
    ProcessEvidence::from_outcome(&evidence)
}

pub(super) fn run(context: &CampaignContext<'_>, summary: &mut CampaignSummary) -> Result<()> {
    run_with_executor(context, summary, execution::execute_streaming)
}

fn run_with_executor(
    context: &CampaignContext<'_>,
    summary: &mut CampaignSummary,
    mut execute: impl FnMut(
        &WorkerSpec,
        &AtomicBool,
        &mut execution::OutputConsumer<'_, FuzzError>,
    ) -> std::result::Result<execution::StreamingOutcome<FuzzError>, execution::ExecutionError>,
) -> Result<()> {
    let o = context.options;
    reserve_evidence(context)?;
    let limits = limits(o)?;
    let round_capacity = limits.round_cases();
    let retained_limit = limits.retained_entries();
    let mut session = SessionStore::create(context.cwd.join("session-state"), limits)?;
    let basis = CorpusBasis {
        schema: 1,
        target: context.target.clone(),
        build: context.build.clone(),
        encoding: "engine entropy, not canonical codec bytes".into(),
    };
    let mut totals = Totals::new(session.initialize(&initial_inputs(o, &basis)?)?);
    session.write_basis(
        &serde_json::to_vec_pretty(&basis).map_err(|e| infrastructure(e.to_string()))?,
    )?;
    summary
        .corpora
        .push(session.corpus_path().to_string_lossy().into_owned());
    let checkpoint = Checkpoint {
        options: o,
        build: context.build,
        round_cases: round_capacity,
        retained_limit,
        started: Instant::now(),
    };
    checkpoint.write(
        &mut session,
        &totals,
        ExecutionCounts::default(),
        &BTreeSet::new(),
        None,
    )?;
    let mut final_counts = ExecutionCounts::default();
    let mut final_checked = BTreeSet::new();
    let mut final_config = context.config.clone();
    let mut failures = Vec::new();
    summary.cleanup = "complete".into();
    loop {
        if context.cancellation.load(Ordering::Acquire) {
            summary.status = CampaignStatus::Cancelled;
            break;
        }
        let elapsed = u64::try_from(checkpoint.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let remaining = if o.continuous {
            round_capacity
        } else {
            o.cases.saturating_sub(attempts(totals.counts)?)
        };
        if remaining == 0 {
            summary.status = assessment(context.target.assess_generated(totals.counts));
            break;
        }
        let wall = if o.continuous {
            o.round_ms
        } else {
            o.round_ms.min(o.wall_ms.saturating_sub(elapsed))
        };
        if wall < o.case_ms {
            summary.status = CampaignStatus::ResourceStop;
            break;
        }
        // A campaign pins source once; each round verifies that the prepared
        // executable is still being attributed to that exact clean source.
        match clean_source(o, context.cancellation) {
            Ok(source) if source == *context.build.source() => {}
            result => {
                summary.status = if context.cancellation.load(Ordering::Acquire) {
                    CampaignStatus::Cancelled
                } else {
                    CampaignStatus::InfrastructureFailure
                };
                summary.detail = match result {
                    Err(error) => error.to_string(),
                    Ok(_) => "prepared source changed between coverage rounds".into(),
                };
                break;
            }
        }
        session.reset_round()?;
        let cases = remaining.min(round_capacity);
        let config = RunConfig::new(RunConfigSpec {
            cases,
            wall_millis: wall,
            case_millis: o.case_ms,
            memory_bytes: o.memory_bytes,
            seed: Some(o.seed.wrapping_add(totals.rounds)),
        })?;
        let round = CampaignContext {
            config: &config,
            ..*context
        };
        let mut observed = Observations::new(&round);
        let mut decoder = worker::ResponseDecoder::new(FRAME)?;
        let mut last_checkpoint = Instant::now();
        let mut env = BTreeMap::new();
        env.insert("SURGEIST_FUZZ_CASE_LIMIT".into(), cases.to_string().into());
        env.insert(
            "ASAN_OPTIONS".into(),
            "abort_on_error=1:detect_leaks=0".into(),
        );
        let args = vec![
            format!("-runs={cases}").into(),
            format!("-max_total_time={}", wall.div_ceil(1000)).into(),
            format!("-timeout={}", o.case_ms.div_ceil(1000)).into(),
            format!("-max_len={}", o.input_bytes).into(),
            format!("-rss_limit_mb={}", o.memory_bytes.div_ceil(1024 * 1024)).into(),
            format!("-seed={}", config.seed().expect("round seed")).into(),
            "-print_final_stats=1".into(),
            "-reload=0".into(),
            format!(
                "-artifact_prefix={}/",
                session.engine_artifacts_path().display()
            )
            .into(),
            session.corpus_path().as_os_str().to_owned(),
        ];
        let worker_limits = execution::WorkerLimits::new(
            0,
            FRAME as usize,
            o.log_bytes as usize,
            Duration::from_millis(wall),
            Duration::from_millis(100),
            Duration::from_secs(3),
            MemoryPolicy::Unbounded,
        )
        .map_err(|e| infrastructure(e.to_string()))?;
        let spec = WorkerSpec::new(
            context.executable.to_owned(),
            context.cwd.to_owned(),
            args,
            env,
            Vec::new(),
            worker_limits,
        )
        .map_err(|e| infrastructure(e.to_string()))?;
        let streamed = execute(
            &spec,
            context.cancellation,
            &mut |stream, bytes| -> Result<()> {
                if stream == OutputStream::Stdout {
                    decoder.push(bytes, &mut |response| observed.accept(response))?;
                    if last_checkpoint.elapsed() >= Duration::from_millis(o.progress_ms) {
                        checkpoint.write(
                            &mut session,
                            &totals,
                            observed.counts,
                            &observed.checked,
                            None,
                        )?;
                        last_checkpoint = Instant::now();
                    }
                }
                Ok(())
            },
        )
        .map_err(|e| infrastructure(e.to_string()))?;
        totals.stdout_bytes = add(totals.stdout_bytes, streamed.stdout_bytes)?;
        totals.stderr_bytes = add(totals.stderr_bytes, streamed.stderr_bytes)?;
        let process = streamed.process;
        totals.diagnostic_tail_truncated = process.stderr.truncated;
        summary.cleanup = cleanup_label(&process.cleanup);
        summary.status = classify_process(&process);
        summary.sampled_peak_rss_bytes = process.sampled_peak_rss_bytes;
        let mut observation_error = match streamed.consumer_failure {
            Some(ConsumerFailure::Error(error)) => Some(error),
            Some(ConsumerFailure::Panicked) => {
                Some(infrastructure("coverage output consumer panicked"))
            }
            None => None,
        };
        if let Err(error) = decoder.finish()
            && summary.status == CampaignStatus::Sufficient
            && observation_error.is_none()
        {
            observation_error = Some(error);
        }
        if summary.status == CampaignStatus::Sufficient && observed.pending.is_some() {
            observation_error = Some(infrastructure("coverage exit omitted final evaluation"));
        }
        if let Some(error) = &observation_error {
            if process.cleanup == CleanupStatus::Complete {
                summary.status = status_for_error(error);
            }
            summary.detail = error.to_string();
        }
        // Preserve exact current bytes even when observation validation or
        // immutable publication failed. A failed publication remains an error.
        if let Some(input) = &observed.pending {
            let artifact = CaptureArtifact::new(
                context.target.clone(),
                context.build.clone(),
                config.clone(),
                input.clone(),
                capture_evidence(&process)?,
            )?;
            let path = context.store.publish_capture(&artifact)?;
            observed
                .failures
                .push(EvidenceStatus::Available { artifact: path });
        }
        if summary.status == CampaignStatus::Finding && observed.failures.is_empty() {
            observed.failures.push(EvidenceStatus::Unavailable {
                reason: EvidenceUnavailable::NoCurrentCase,
            });
        }
        if observed.known_failure
            && observation_error.is_none()
            && process.cleanup == CleanupStatus::Complete
        {
            summary.status = CampaignStatus::Finding;
        }
        totals.counts = combine(totals.counts, observed.counts)?;
        totals.checked.extend(observed.checked.iter().cloned());
        totals.rounds = add(totals.rounds, 1)?;
        final_counts = observed.counts;
        final_checked = observed.checked;
        final_config = config.clone();
        failures = observed.failures;
        session.replace_log(&process.stderr.bytes)?;
        // A clean boundary without an in-flight case can rotate safely. A
        // process deadline with an in-flight case is terminal resource evidence.
        let round_complete = summary.status == CampaignStatus::Sufficient
            || (process.stop == ProcessStop::Deadline
                && process.cleanup == CleanupStatus::Complete
                && observed.pending.is_none()
                && observation_error.is_none());
        if round_complete {
            match session.finish_round() {
                Ok(report) => totals.retained(report)?,
                Err(error) => {
                    summary.status = status_for_error(&error);
                    summary.detail = error.to_string();
                    break;
                }
            }
        }
        if !round_complete {
            break;
        }
        if observed.counts.evaluated() == 0 {
            summary.status = assessment(context.target.assess_generated(observed.counts));
            break;
        }
        checkpoint.write(
            &mut session,
            &totals,
            ExecutionCounts::default(),
            &BTreeSet::new(),
            None,
        )?;
    }
    summary.corpora = vec![
        session
            .retained_corpus_path()?
            .to_string_lossy()
            .into_owned(),
    ];
    summary.evaluated = totals.counts.evaluated();
    summary.generation_rejected = totals.counts.generation_rejected();
    for failure in &failures {
        if let EvidenceStatus::Available { artifact } = failure {
            summary.artifacts.push(artifact.as_str().into());
        }
    }
    checkpoint.write(
        &mut session,
        &totals,
        ExecutionCounts::default(),
        &BTreeSet::new(),
        Some(summary.status),
    )?;
    if summary.detail.is_empty() {
        summary.detail = format!(
            "{} bounded engine rounds; streamed observations; aggregate progress in session-state/progress.json; capped entropy retention may lose coverage; fresh engine state each round; OS memory unbounded; {} discarded entropy inputs; immutable terminal record {}",
            totals.rounds,
            totals.discarded_entries,
            "describes the final finite round; aggregate counts are in the progress checkpoint"
        );
    }
    let terminal = CampaignContext {
        config: &final_config,
        ..*context
    };
    publish_run(&terminal, final_counts, final_checked, failures, summary)
}

#[cfg(test)]
mod tests;
