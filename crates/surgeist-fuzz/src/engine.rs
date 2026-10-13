//! Safe adapters. Owners provide generation, canonical codecs and one evaluator.
#![forbid(unsafe_code)]

use crate::{
    CaseOutcome, CaseTarget, FuzzError, FuzzErrorKind, Generation, GenerationRejection, Result,
};
#[cfg(any(feature = "coverage", feature = "property"))]
use crate::{CoverageAssessment, ExecutionCounts, FailureKey};
use serde::{Deserialize, Serialize};

/// Concrete evidence from one invocation of the owner evaluator.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evaluation {
    pub canonical: Vec<u8>,
    pub outcome: CaseOutcome,
}

/// Generation discard is distinct from a product observation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GeneratedEvaluation {
    Rejected(GenerationRejection),
    Evaluated(Evaluation),
}

/// Admission is separate from generation rejection and owner observations.
#[cfg(feature = "coverage")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoverageAttempt {
    Completed(GeneratedEvaluation),
    LimitReached,
}

fn error(kind: FuzzErrorKind, detail: impl Into<String>) -> FuzzError {
    FuzzError::new(kind, "run fuzz engine", detail)
}

pub(crate) fn admit_bytes<T: CaseTarget>(target: &T, bytes: &[u8]) -> Result<()> {
    if bytes.len() as u64 > target.descriptor().limits().bytes() {
        return Err(error(
            FuzzErrorKind::InvalidLimits,
            "input exceeds byte ceiling",
        ));
    }
    Ok(())
}

/// Canonicalize and capture before evaluation. Capture failure prevents evaluation.
/// The callback may retain in memory or durably publish the concrete case; this
/// adapter never writes files. Unknown owner panics propagate without a made-up key.
pub fn evaluate_case<T: CaseTarget>(
    target: &T,
    case: &T::Case,
    capture: &mut dyn FnMut(&[u8]) -> Result<()>,
) -> Result<Evaluation> {
    let canonical = target.encode(case)?;
    admit_bytes(target, &canonical)?;
    capture(&canonical)?;
    let outcome = target.evaluate(case)?;
    target.descriptor().validate_outcome(&outcome)?;
    Ok(Evaluation { canonical, outcome })
}

/// Materializes generation entropy, then uses the same evaluator as replay.
pub fn evaluate_generated<T: CaseTarget>(
    target: &T,
    entropy: &[u8],
    capture: &mut dyn FnMut(&[u8]) -> Result<()>,
) -> Result<GeneratedEvaluation> {
    admit_bytes(target, entropy)?;
    match target.generate(entropy)? {
        Generation::Rejected(reason) => Ok(GeneratedEvaluation::Rejected(reason)),
        Generation::Generated(case) => {
            evaluate_case(target, &case, capture).map(GeneratedEvaluation::Evaluated)
        }
    }
}

/// Replays concrete canonical bytes, never regenerating from a seed.
pub fn replay<T: CaseTarget>(
    target: &T,
    canonical: &[u8],
    capture: &mut dyn FnMut(&[u8]) -> Result<()>,
) -> Result<Evaluation> {
    admit_bytes(target, canonical)?;
    let case = target.decode(canonical)?;
    let encoded = target.encode(&case)?;
    admit_bytes(target, &encoded)?;
    if encoded != canonical {
        return Err(error(
            FuzzErrorKind::InvalidCase,
            "replay bytes are not canonical",
        ));
    }
    capture(&encoded)?;
    let outcome = target.evaluate(&case)?;
    target.descriptor().validate_outcome(&outcome)?;
    Ok(Evaluation {
        canonical: encoded,
        outcome,
    })
}

#[cfg(any(feature = "coverage", feature = "property"))]
fn record(counts: &mut ExecutionCounts, result: &GeneratedEvaluation) -> Result<()> {
    match result {
        GeneratedEvaluation::Rejected(_) => counts.record_generation_rejection(),
        GeneratedEvaluation::Evaluated(evaluation) => counts.record_outcome(&evaluation.outcome),
    }
}

/// Coverage inputs use owner generation, with separate discard/evaluation counts.
#[cfg(feature = "coverage")]
pub struct CoverageDriver<'a, T: CaseTarget> {
    target: &'a T,
    selected_failure: Option<FailureKey>,
    counts: ExecutionCounts,
    attempts: u64,
    attempt_limit: Option<std::num::NonZeroU64>,
}

#[cfg(feature = "coverage")]
impl<'a, T: CaseTarget> CoverageDriver<'a, T> {
    /// Caller-controlled in-process driver without a campaign attempt ceiling.
    /// Selecting a failure key prevents an external minimizer switching bugs.
    pub fn new(target: &'a T, selected_failure: Option<FailureKey>) -> Result<Self> {
        if selected_failure
            .as_ref()
            .is_some_and(|key| !target.descriptor().properties().contains(key.property()))
        {
            return Err(error(
                FuzzErrorKind::InvalidObservation,
                "selected failure names an undeclared property",
            ));
        }
        Ok(Self {
            target,
            selected_failure,
            counts: ExecutionCounts::default(),
            attempts: 0,
            attempt_limit: None,
        })
    }

    /// Admit at most this many callbacks, including rejected or errored attempts.
    pub fn with_attempt_limit(
        target: &'a T,
        selected_failure: Option<FailureKey>,
        limit: u64,
    ) -> Result<Self> {
        let limit = std::num::NonZeroU64::new(limit).ok_or_else(|| {
            error(
                FuzzErrorKind::InvalidLimits,
                "coverage attempt limit must be positive",
            )
        })?;
        let mut driver = Self::new(target, selected_failure)?;
        driver.attempt_limit = Some(limit);
        Ok(driver)
    }

    /// Campaign binaries require a positive decimal `SURGEIST_FUZZ_CASE_LIMIT`.
    /// The driver enforces admission independently of libFuzzer startup callbacks.
    pub fn for_campaign(target: &'a T, selected_failure: Option<FailureKey>) -> Result<Self> {
        let value = std::env::var("SURGEIST_FUZZ_CASE_LIMIT").map_err(|source| {
            FuzzError::with_source(
                FuzzErrorKind::InvalidLimits,
                "admit coverage campaign",
                "SURGEIST_FUZZ_CASE_LIMIT must be present and UTF-8",
                source,
            )
        })?;
        if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
            return Err(error(
                FuzzErrorKind::InvalidLimits,
                "SURGEIST_FUZZ_CASE_LIMIT must be a positive decimal integer",
            ));
        }
        let limit = value.parse::<u64>().map_err(|source| {
            FuzzError::with_source(
                FuzzErrorKind::InvalidLimits,
                "admit coverage campaign",
                "SURGEIST_FUZZ_CASE_LIMIT exceeds the supported integer range",
                source,
            )
        })?;
        Self::with_attempt_limit(target, selected_failure, limit)
    }

    #[must_use]
    pub const fn attempts(&self) -> u64 {
        self.attempts
    }
    #[must_use]
    pub const fn counts(&self) -> ExecutionCounts {
        self.counts
    }
    #[must_use]
    pub fn assessment(&self) -> CoverageAssessment {
        self.target.descriptor().assess_generated(self.counts)
    }
    pub fn test_one(
        &mut self,
        input: &[u8],
        capture: &mut dyn FnMut(&[u8]) -> Result<()>,
    ) -> Result<GeneratedEvaluation> {
        match self.attempt(input, capture)? {
            CoverageAttempt::Completed(result) => Ok(result),
            CoverageAttempt::LimitReached => Err(error(
                FuzzErrorKind::Resource,
                "coverage attempt limit reached",
            )),
        }
    }
    /// Call from an owner binary's safe `fuzz_target!` closure. Semantic failures
    /// matching the selected key panic for libFuzzer; other failures stay visible
    /// in the returned observation. Harness errors remain errors for the caller.
    pub fn libfuzzer_entry(
        &mut self,
        input: &[u8],
        capture: &mut dyn FnMut(&[u8]) -> Result<()>,
    ) -> Result<CoverageAttempt> {
        let result = self.attempt(input, capture)?;
        if let CoverageAttempt::Completed(observation) = &result {
            self.raise_selected_failure(observation);
        }
        Ok(result)
    }
    /// Emits canonical capture before evaluation and cumulative observations
    /// afterwards, then escalates a matching owner failure. The controller owns
    /// the campaign's total output quota. Exhausted admission emits no frames.
    pub fn libfuzzer_entry_framed<W: std::io::Write>(
        &mut self,
        input: &[u8],
        writer: &mut W,
        max_frame_bytes: u32,
    ) -> Result<CoverageAttempt> {
        use crate::worker::{write_capture, write_coverage_observation};
        let result = self.attempt(input, &mut |canonical| {
            write_capture(writer, canonical, max_frame_bytes).map(|_| ())
        })?;
        if let CoverageAttempt::Completed(observation) = &result {
            write_coverage_observation(writer, observation, self.counts, max_frame_bytes)?;
            self.raise_selected_failure(observation);
        }
        Ok(result)
    }

    fn attempt(
        &mut self,
        input: &[u8],
        capture: &mut dyn FnMut(&[u8]) -> Result<()>,
    ) -> Result<CoverageAttempt> {
        if self
            .attempt_limit
            .is_some_and(|limit| self.attempts >= limit.get())
        {
            return Ok(CoverageAttempt::LimitReached);
        }
        self.attempts = self.attempts.checked_add(1).ok_or_else(|| {
            error(
                FuzzErrorKind::Resource,
                "coverage attempt counter exhausted",
            )
        })?;
        let result = evaluate_generated(self.target, input, capture)?;
        record(&mut self.counts, &result)?;
        Ok(CoverageAttempt::Completed(result))
    }
    fn raise_selected_failure(&self, result: &GeneratedEvaluation) {
        if let GeneratedEvaluation::Evaluated(evaluation) = result
            && let Some(key) = evaluation.outcome.failure_key()
            && self
                .selected_failure
                .as_ref()
                .is_none_or(|selected| selected == key)
        {
            panic!("owner-reported fuzz failure: {key:?}");
        }
    }
}

/// Decode a safe arbitrary recipe; the owner still validates/materializes it.
#[cfg(feature = "coverage")]
pub fn arbitrary_recipe<'a, C: arbitrary::Arbitrary<'a>>(
    input: &'a [u8],
) -> std::result::Result<C, arbitrary::Error> {
    C::arbitrary(&mut arbitrary::Unstructured::new(input))
}

/// Owners can compose real state-machine strategies into generation recipes.
#[cfg(feature = "property")]
pub use proptest_state_machine as state_machine;

/// Explicit finite property-run limits. Environment variables do not select them.
#[cfg(feature = "property")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PropertyConfig {
    pub cases: u32,
    pub max_rejections: u32,
    pub max_shrink_iterations: u32,
    pub seed: u64,
}

#[cfg(feature = "property")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyReport {
    pub counts: ExecutionCounts,
    pub assessment: CoverageAssessment,
    pub failure: Option<Evaluation>,
}

/// Executes a real proptest strategy whose values are owner generation entropy.
/// All evaluated cases, including shrinking, use the owner evaluator and concrete
/// capture. A successful report requires nonvacuous owner-declared coverage.
#[cfg(feature = "property")]
pub fn run_property<T, S>(
    target: &T,
    strategy: &S,
    config: PropertyConfig,
    capture: &mut dyn FnMut(&[u8]) -> Result<()>,
) -> Result<PropertyReport>
where
    T: CaseTarget,
    S: proptest::strategy::Strategy<Value = Vec<u8>>,
{
    use proptest::test_runner::{Config, RngSeed, TestCaseError, TestError, TestRunner};
    use std::cell::RefCell;
    if config.cases == 0 || config.max_shrink_iterations == u32::MAX {
        return Err(error(
            FuzzErrorKind::InvalidLimits,
            "property runs need positive cases and a finite shrink ceiling",
        ));
    }
    let runner_config = Config {
        cases: config.cases,
        max_local_rejects: config.max_rejections,
        max_global_rejects: config.max_rejections,
        max_shrink_iters: config.max_shrink_iterations,
        max_shrink_time: 0,
        fork: false,
        timeout: 0,
        failure_persistence: None,
        rng_seed: RngSeed::Fixed(config.seed),
        ..Config::default()
    };
    let counts = RefCell::new(ExecutionCounts::default());
    let capture = RefCell::new(capture);
    let harness_error = RefCell::new(None);
    let failure = RefCell::new(None::<Evaluation>);
    let selected = RefCell::new(None::<FailureKey>);
    let mut runner = TestRunner::new(runner_config);
    let result = runner.run(strategy, |input| {
        if harness_error.borrow().is_some() {
            return Err(TestCaseError::fail("harness stopped"));
        }
        let evaluated = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            evaluate_generated(target, &input, &mut **capture.borrow_mut()).and_then(|evaluation| {
                record(&mut counts.borrow_mut(), &evaluation)?;
                Ok(evaluation)
            })
        }))
        .unwrap_or_else(|_| {
            Err(error(
                FuzzErrorKind::Harness,
                "owner panicked without a reported failure identity",
            ))
        });
        match evaluated {
            Err(cause) => {
                *harness_error.borrow_mut() = Some(cause);
                Err(TestCaseError::fail("owner adapter error"))
            }
            Ok(GeneratedEvaluation::Rejected(reason)) => {
                Err(TestCaseError::reject(reason.reason().to_owned()))
            }
            Ok(GeneratedEvaluation::Evaluated(evaluation)) => {
                if let Some(key) = evaluation.outcome.failure_key() {
                    if selected.borrow().as_ref().is_some_and(|first| first != key) {
                        return Err(TestCaseError::reject("different failure identity"));
                    }
                    *selected.borrow_mut() = Some(key.clone());
                    *failure.borrow_mut() = Some(evaluation);
                    Err(TestCaseError::fail("owner-reported semantic failure"))
                } else if !evaluation.outcome.is_evaluated() {
                    Err(TestCaseError::reject("no semantic property was checked"))
                } else {
                    Ok(())
                }
            }
        }
    });
    if let Some(cause) = harness_error.into_inner() {
        return Err(cause);
    }
    let failure = failure.into_inner();
    match result {
        Err(TestError::Abort(reason)) => {
            return Err(error(
                FuzzErrorKind::InsufficientCoverage,
                reason.to_string(),
            ));
        }
        Err(TestError::Fail(_, _)) if failure.is_none() => {
            return Err(error(
                FuzzErrorKind::Harness,
                "owner or strategy panicked without a reported failure identity",
            ));
        }
        _ => {}
    }
    let counts = counts.into_inner();
    let assessment = target.descriptor().assess_generated(counts);
    if failure.is_none() && assessment != CoverageAssessment::Sufficient {
        return Err(error(
            FuzzErrorKind::InsufficientCoverage,
            "owner-declared coverage was not met",
        ));
    }
    Ok(PropertyReport {
        counts,
        assessment,
        failure,
    })
}
