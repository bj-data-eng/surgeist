//! Bounded reduction with repeated, exact failure-identity confirmation.
//!
//! Campaign callers supply an isolated evaluator. The direct owner helper is for
//! in-process properties and tests and cannot enforce a hard replay deadline.
#![forbid(unsafe_code)]

use crate::{
    CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError, FuzzErrorKind, InputEncoding,
    Result, Sha256Digest,
};
use serde::{Deserialize, Serialize};
use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::{Duration, Instant},
};

/// Finite ceilings shared by discovery and all confirmation replays.
/// `max_attempts` bounds replays and, independently, candidate emissions.
#[derive(Clone, Copy, Debug)]
pub struct ReductionLimits {
    max_attempts: u64,
    max_duration: Duration,
    max_output_bytes: u64,
    confirmations: u32,
}
impl ReductionLimits {
    pub fn new(
        max_attempts: u64,
        max_duration: Duration,
        max_output_bytes: u64,
        confirmations: u32,
    ) -> Result<Self> {
        if confirmations < 2
            || max_attempts < u64::from(confirmations) * 2
            || max_duration.is_zero()
            || max_output_bytes == 0
        {
            return Err(FuzzError::new(
                FuzzErrorKind::InvalidLimits,
                "reduction limits",
                "requires at least two confirmations, enough original/final attempts, and positive time/output ceilings",
            ));
        }
        Ok(Self {
            max_attempts,
            max_duration,
            max_output_bytes,
            confirmations,
        })
    }
    #[must_use]
    pub const fn max_attempts(self) -> u64 {
        self.max_attempts
    }
    #[must_use]
    pub const fn max_duration(self) -> Duration {
        self.max_duration
    }
    #[must_use]
    pub const fn max_output_bytes(self) -> u64 {
        self.max_output_bytes
    }
    #[must_use]
    pub const fn confirmations(self) -> u32 {
        self.confirmations
    }
}

/// Remaining ceilings for one replay; the evaluator must enforce these itself.
#[derive(Clone, Copy, Debug)]
pub struct ReplayBudget {
    pub remaining_time: Duration,
    pub output_bytes: u64,
}
/// An evaluator's semantic observation and actual captured output consumption.
#[derive(Clone, Debug)]
pub struct ReductionEvaluation {
    pub outcome: CaseOutcome,
    pub output_bytes: u64,
    interruption: Option<ReductionInterruption>,
}
/// Operational stops cannot confirm a semantic failure, even after a response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReductionInterruption {
    Cancelled,
    TimeLimit,
    OutputLimit,
}
impl ReductionInterruption {
    const fn status(self) -> ReductionStatus {
        match self {
            Self::Cancelled => ReductionStatus::Cancelled,
            Self::TimeLimit => ReductionStatus::TimeLimit,
            Self::OutputLimit => ReductionStatus::OutputLimit,
        }
    }
}
impl ReductionEvaluation {
    #[must_use]
    pub const fn new(outcome: CaseOutcome, output_bytes: u64) -> Self {
        Self {
            outcome,
            output_bytes,
            interruption: None,
        }
    }
    pub fn interrupted(reason: ReductionInterruption, output_bytes: u64) -> Result<Self> {
        Ok(Self {
            outcome: CaseOutcome::unsupported(format!("isolated replay interrupted: {reason:?}"))?,
            output_bytes,
            interruption: Some(reason),
        })
    }
    #[must_use]
    pub const fn interruption(&self) -> Option<ReductionInterruption> {
        self.interruption
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReductionStatus {
    Complete,
    AttemptLimit,
    TimeLimit,
    OutputLimit,
    Cancelled,
    OriginalNotReproduced,
    FlakyOriginal,
    FlakyCandidate,
    FlakyFinal,
    UnsupportedFailure,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReductionPhase {
    Original,
    Candidate,
    Final,
    Canonical,
}

/// An immutable observation linking a replay to its exact submitted input.
#[derive(Clone, Debug)]
pub struct ReductionObservation {
    pub phase: ReductionPhase,
    pub input_digest: Sha256Digest,
    pub input_bytes: usize,
    pub outcome: CaseOutcome,
    pub output_bytes: u64,
}
/// Accepted links only; failed/different/rejected candidates remain observations.
#[derive(Clone, Debug)]
pub struct ReductionStep {
    pub from: Sha256Digest,
    pub to: Sha256Digest,
    pub replay_count: u64,
}

#[derive(Clone, Debug)]
pub struct ReductionReport {
    limits: ReductionLimits,
    original_input: Vec<u8>,
    best_input: Vec<u8>,
    canonical_input: Option<Vec<u8>>,
    expected: FailureKey,
    status: ReductionStatus,
    replay_count: u64,
    candidate_count: u64,
    proposal_output_bytes: u64,
    output_bytes: u64,
    original_confirmed: bool,
    final_confirmed: bool,
    observations: Vec<ReductionObservation>,
    chain: Vec<ReductionStep>,
}
impl ReductionReport {
    #[must_use]
    pub const fn limits(&self) -> ReductionLimits {
        self.limits
    }
    #[must_use]
    pub fn original_input(&self) -> &[u8] {
        &self.original_input
    }
    /// Exact bytes accepted by repeated replays, or the original if none were verified.
    #[must_use]
    pub fn best_input(&self) -> &[u8] {
        &self.best_input
    }
    /// Canonical owner bytes, present only after exact-key confirmation of those bytes.
    #[must_use]
    pub fn canonical_input(&self) -> Option<&[u8]> {
        self.canonical_input.as_deref()
    }
    #[must_use]
    pub const fn expected(&self) -> &FailureKey {
        &self.expected
    }
    #[must_use]
    pub const fn status(&self) -> ReductionStatus {
        self.status
    }
    #[must_use]
    pub const fn replay_count(&self) -> u64 {
        self.replay_count
    }
    #[must_use]
    pub const fn candidate_count(&self) -> u64 {
        self.candidate_count
    }
    /// Output from candidate proposal tools, separate from owner replay observations.
    #[must_use]
    pub const fn proposal_output_bytes(&self) -> u64 {
        self.proposal_output_bytes
    }
    #[must_use]
    pub const fn output_bytes(&self) -> u64 {
        self.output_bytes
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
    pub fn observations(&self) -> &[ReductionObservation] {
        &self.observations
    }
    #[must_use]
    pub fn chain(&self) -> &[ReductionStep] {
        &self.chain
    }
}

struct Session<'a, T, E> {
    target: &'a T,
    evaluate: E,
    limits: ReductionLimits,
    start: Instant,
    cancellation: &'a AtomicBool,
    report: ReductionReport,
}
impl<T: CaseTarget, E: FnMut(&[u8], ReplayBudget) -> Result<ReductionEvaluation>>
    Session<'_, T, E>
{
    fn stop(&self) -> Option<ReductionStatus> {
        if self.cancellation.load(Ordering::Acquire) {
            Some(ReductionStatus::Cancelled)
        } else if self.start.elapsed() >= self.limits.max_duration {
            Some(ReductionStatus::TimeLimit)
        } else if self.report.replay_count >= self.limits.max_attempts {
            Some(ReductionStatus::AttemptLimit)
        } else if self.report.output_bytes >= self.limits.max_output_bytes {
            Some(ReductionStatus::OutputLimit)
        } else {
            None
        }
    }
    fn search_stop(&self) -> Option<ReductionStatus> {
        self.stop().or_else(|| {
            // Keep a complete final confirmation available after search stops.
            if self.report.candidate_count >= self.limits.max_attempts
                || self
                    .report
                    .replay_count
                    .saturating_add(u64::from(self.limits.confirmations) * 2)
                    > self.limits.max_attempts
            {
                Some(ReductionStatus::AttemptLimit)
            } else {
                None
            }
        })
    }
    fn confirm(&mut self, input: &[u8], phase: ReductionPhase) -> Result<Confirmation> {
        let mut first: Option<CaseOutcome> = None;
        let mut all_same = true;
        for _ in 0..self.limits.confirmations {
            if let Some(status) = self.stop() {
                self.report.status = status;
                return Ok(Confirmation::Stopped);
            }
            let budget = ReplayBudget {
                remaining_time: self
                    .limits
                    .max_duration
                    .saturating_sub(self.start.elapsed()),
                output_bytes: self.limits.max_output_bytes - self.report.output_bytes,
            };
            self.report.replay_count += 1;
            let replay = (self.evaluate)(input, budget)?;
            self.target.descriptor().validate_outcome(&replay.outcome)?;
            self.report.output_bytes = self.report.output_bytes.saturating_add(replay.output_bytes);
            if let Some(previous) = &first {
                all_same &= previous.kind() == replay.outcome.kind()
                    && previous.failure_key() == replay.outcome.failure_key();
            } else {
                first = Some(replay.outcome.clone());
            }
            self.report.observations.push(ReductionObservation {
                phase,
                input_digest: Sha256Digest::from_bytes(input),
                input_bytes: input.len(),
                outcome: replay.outcome,
                output_bytes: replay.output_bytes,
            });
            if let Some(reason) = replay.interruption {
                self.report.status = reason.status();
                return Ok(Confirmation::Stopped);
            }
            // Keep even a rejected over-limit observation in the evidence chain.
            if replay.output_bytes > budget.output_bytes {
                self.report.status = ReductionStatus::OutputLimit;
                return Ok(Confirmation::Stopped);
            }
            if self.cancellation.load(Ordering::Acquire) {
                self.report.status = ReductionStatus::Cancelled;
                return Ok(Confirmation::Stopped);
            }
            if self.start.elapsed() >= self.limits.max_duration {
                self.report.status = ReductionStatus::TimeLimit;
                return Ok(Confirmation::Stopped);
            }
        }
        if !all_same {
            return Ok(Confirmation::Flaky);
        }
        Ok(
            if first.as_ref().and_then(CaseOutcome::failure_key) == Some(&self.report.expected) {
                Confirmation::Same
            } else {
                Confirmation::Different
            },
        )
    }
    fn candidate(&mut self, input: Vec<u8>) -> Result<bool> {
        if let Some(status) = self.search_stop() {
            self.report.status = status;
            return Ok(false);
        }
        self.report.candidate_count += 1;
        if !smaller(&input, &self.report.best_input) {
            return Ok(false);
        }
        if input.len() as u64 > self.target.descriptor().limits().bytes() {
            return Ok(false);
        }
        match self.confirm(&input, ReductionPhase::Candidate)? {
            Confirmation::Same => {
                self.report.chain.push(ReductionStep {
                    from: Sha256Digest::from_bytes(&self.report.best_input),
                    to: Sha256Digest::from_bytes(&input),
                    replay_count: self.report.replay_count,
                });
                self.report.best_input = input;
                Ok(true)
            }
            Confirmation::Flaky => {
                self.report.status = ReductionStatus::FlakyCandidate;
                Ok(false)
            }
            Confirmation::Different | Confirmation::Stopped => Ok(false),
        }
    }
    fn active(&self) -> bool {
        self.report.status == ReductionStatus::Complete
    }
}
#[derive(Clone, Copy)]
enum Confirmation {
    Same,
    Different,
    Flaky,
    Stopped,
}
fn smaller(candidate: &[u8], best: &[u8]) -> bool {
    (candidate.len(), candidate) < (best.len(), best)
}

/// Reduce owner replay bytes using a caller-owned isolated evaluation boundary.
/// Engine entropy is separate provenance and must be converted by its owner before
/// this boundary. Search accepts strictly smaller `(byte length, lexical bytes)`
/// candidates; completion does not claim a globally minimal case.
/// Each replay receives the exact submitted bytes. Evaluation must honor its
/// remaining ceilings and perform cleanup before returning. Owner candidate
/// generation must honor the visitor stop signal, as required by `CaseTarget`.
pub fn reduce_with<T, E>(
    target: &T,
    original: &[u8],
    expected: &FailureKey,
    limits: ReductionLimits,
    cancellation: &AtomicBool,
    evaluate: E,
) -> Result<ReductionReport>
where
    T: CaseTarget,
    E: FnMut(&[u8], ReplayBudget) -> Result<ReductionEvaluation>,
{
    reduce_with_proposals(
        target,
        original,
        expected,
        limits,
        cancellation,
        |_| Ok((Vec::new(), 0, None)),
        evaluate,
    )
}

/// Proposals are generated only after the original has been confirmed. Their
/// time and output consume the same reduction session budget as owner replay.
pub(crate) fn reduce_with_proposals<T, P, E>(
    target: &T,
    original: &[u8],
    expected: &FailureKey,
    limits: ReductionLimits,
    cancellation: &AtomicBool,
    mut proposals: P,
    evaluate: E,
) -> Result<ReductionReport>
where
    T: CaseTarget,
    P: FnMut(ReplayBudget) -> Result<(Vec<Vec<u8>>, u64, Option<ReductionStatus>)>,
    E: FnMut(&[u8], ReplayBudget) -> Result<ReductionEvaluation>,
{
    target.descriptor().validate_outcome(&CaseOutcome::failure(
        expected.clone(),
        "selected reduction failure",
    )?)?;
    if original.len() as u64 > target.descriptor().limits().bytes() {
        return Err(FuzzError::new(
            FuzzErrorKind::InvalidLimits,
            "reduce input",
            "original exceeds owner byte ceiling",
        ));
    }
    let mut session = Session {
        target,
        evaluate,
        limits,
        start: Instant::now(),
        cancellation,
        report: ReductionReport {
            limits,
            original_input: original.to_vec(),
            best_input: original.to_vec(),
            canonical_input: None,
            expected: expected.clone(),
            status: ReductionStatus::Complete,
            replay_count: 0,
            candidate_count: 0,
            proposal_output_bytes: 0,
            output_bytes: 0,
            original_confirmed: false,
            final_confirmed: false,
            observations: Vec::new(),
            chain: Vec::new(),
        },
    };
    if matches!(expected.class(), FailureClass::Panic | FailureClass::Crash)
        && expected.discriminator().is_none()
    {
        session.report.status = ReductionStatus::UnsupportedFailure;
        return Ok(session.report);
    }
    match session.confirm(original, ReductionPhase::Original)? {
        Confirmation::Same => session.report.original_confirmed = true,
        Confirmation::Different => session.report.status = ReductionStatus::OriginalNotReproduced,
        Confirmation::Flaky => session.report.status = ReductionStatus::FlakyOriginal,
        Confirmation::Stopped => {}
    }
    if !session.active() {
        return Ok(session.report);
    }
    if session.search_stop().is_none() {
        let (inputs, output_bytes, proposal_stop) = proposals(ReplayBudget {
            remaining_time: session
                .limits
                .max_duration
                .saturating_sub(session.start.elapsed()),
            output_bytes: session.limits.max_output_bytes - session.report.output_bytes,
        })?;
        session.report.proposal_output_bytes = output_bytes;
        session.report.output_bytes = session.report.output_bytes.saturating_add(output_bytes);
        if let Some(status) = session.stop().or(proposal_stop) {
            session.report.status = status;
            return Ok(session.report);
        }
        for input in inputs {
            session.candidate(input)?;
            if !session.active() {
                break;
            }
        }
    }
    if session.active() {
        match target.descriptor().encoding() {
            InputEncoding::RawBytes => reduce_bytes(&mut session)?,
            InputEncoding::Structured => reduce_structured(&mut session)?,
        }
    }
    if !session.active()
        && !matches!(
            session.report.status,
            ReductionStatus::AttemptLimit | ReductionStatus::FlakyCandidate
        )
    {
        return Ok(session.report);
    }
    let best = session.report.best_input.clone();
    match session.confirm(&best, ReductionPhase::Final)? {
        Confirmation::Same => session.report.final_confirmed = true,
        Confirmation::Different | Confirmation::Flaky => {
            session.report.status = ReductionStatus::FlakyFinal
        }
        Confirmation::Stopped => {}
    }
    if !session.report.final_confirmed {
        return Ok(session.report);
    }
    let canonical = target.encode(&target.decode(&best)?)?;
    if canonical.len() as u64 > target.descriptor().limits().bytes() {
        return Err(FuzzError::new(
            FuzzErrorKind::InvalidLimits,
            "canonical reduction output",
            "owner encoded bytes exceed ceiling",
        ));
    }
    if canonical == best {
        session.report.canonical_input = Some(canonical);
    } else if session.active() {
        match session.confirm(&canonical, ReductionPhase::Canonical)? {
            Confirmation::Same => session.report.canonical_input = Some(canonical),
            Confirmation::Flaky => session.report.status = ReductionStatus::FlakyFinal,
            Confirmation::Different | Confirmation::Stopped => {}
        }
    }
    Ok(session.report)
}

fn reduce_bytes<T, E>(session: &mut Session<'_, T, E>) -> Result<()>
where
    T: CaseTarget,
    E: FnMut(&[u8], ReplayBudget) -> Result<ReductionEvaluation>,
{
    loop {
        if let Some(status) = session.search_stop() {
            session.report.status = status;
            break;
        }
        let best = session.report.best_input.clone();
        let mut accepted = false;
        let mut chunk = best.len();
        while chunk > 0 && !accepted && session.active() {
            for begin in (0..best.len()).step_by(chunk) {
                let end = begin.saturating_add(chunk).min(best.len());
                let mut candidate = Vec::with_capacity(best.len() - (end - begin));
                candidate.extend_from_slice(&best[..begin]);
                candidate.extend_from_slice(&best[end..]);
                if session.candidate(candidate)? {
                    accepted = true;
                    break;
                }
                if !session.active() {
                    break;
                }
            }
            chunk /= 2;
        }
        // Equal-length simplification is finite through the strict lexical measure.
        if !accepted && session.active() {
            for index in 0..best.len() {
                if best[index] == 0 {
                    continue;
                }
                let mut candidate = best.clone();
                candidate[index] = 0;
                if session.candidate(candidate)? {
                    accepted = true;
                    break;
                }
                if !session.active() {
                    break;
                }
            }
        }
        if !accepted || !session.active() {
            break;
        }
    }
    Ok(())
}
fn reduce_structured<T, E>(session: &mut Session<'_, T, E>) -> Result<()>
where
    T: CaseTarget,
    E: FnMut(&[u8], ReplayBudget) -> Result<ReductionEvaluation>,
{
    loop {
        if let Some(status) = session.search_stop() {
            session.report.status = status;
            break;
        }
        let case = session.target.decode(&session.report.best_input)?;
        let target = session.target;
        let mut accepted = false;
        let mut error = None;
        target.reduction_candidates(&case, &mut |candidate| {
            if let Some(status) = session.search_stop() {
                session.report.status = status;
                return false;
            }
            // Bound non-progressing and duplicate owner emissions too, rather than
            // allowing an infinite stream to avoid the replay-attempt budget.
            match target.encode(&candidate).and_then(|bytes| {
                target.decode(&bytes)?;
                session.candidate(bytes)
            }) {
                Ok(true) => {
                    accepted = true;
                    false
                }
                Ok(false) => {
                    if let Some(status) = session.search_stop() {
                        session.report.status = status;
                    }
                    session.active()
                }
                Err(failure) => {
                    error = Some(failure);
                    false
                }
            }
        })?;
        if let Some(failure) = error {
            return Err(failure);
        }
        if let Some(status) = session.search_stop() {
            session.report.status = status;
        }
        if !accepted || !session.active() {
            break;
        }
    }
    Ok(())
}

/// Convenience for finite in-process owner properties/tests. It does not isolate
/// panics or interrupt a blocking owner call; campaigns must use `reduce_with`.
pub fn reduce_direct<T: CaseTarget>(
    target: &T,
    original: &[u8],
    expected: &FailureKey,
    limits: ReductionLimits,
    cancellation: &AtomicBool,
) -> Result<ReductionReport> {
    reduce_with(
        target,
        original,
        expected,
        limits,
        cancellation,
        |input, _budget| {
            let outcome = target.evaluate(&target.decode(input)?)?;
            let output_bytes = outcome.detail().map_or(0, |detail| detail.len() as u64);
            Ok(ReductionEvaluation::new(outcome, output_bytes))
        },
    )
}
