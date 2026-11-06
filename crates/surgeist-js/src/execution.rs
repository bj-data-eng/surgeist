//! Work limits and profiling for controlled JavaScript execution.

use std::time::Duration;

use surgeist_js_parser::{
    frontend::{FrontendControl, FrontendLimit, FrontendWork},
    source::ReadUnit,
};
use surgeist_ledger::{
    Change, Error as LedgerError, Limits, Operation, Report, Scope, Tariffs, Work,
};
use thiserror::Error;

use crate::{JsError, JsResult, JsValue, error::RuntimeLimitError};

pub use surgeist_ledger::{EventFilter, ProfileMode};

const INSTRUCTION: Work<10> = Work::new(0).unwrap();
const SOURCE_READ: Work<10> = Work::new(1).unwrap();
const SOURCE_CONSUMPTION: Work<10> = Work::new(2).unwrap();
const SOURCE_RETENTION: Work<10> = Work::new(3).unwrap();
const LEXER_LOOKAHEAD: Work<10> = Work::new(4).unwrap();
const LEXER_DISPATCH: Work<10> = Work::new(5).unwrap();
const TOKEN_PROBE: Work<10> = Work::new(6).unwrap();
const TOKEN_ADVANCE: Work<10> = Work::new(7).unwrap();
const GRAMMAR_DISPATCH: Work<10> = Work::new(8).unwrap();
const EARLY_ERROR_VISIT: Work<10> = Work::new(9).unwrap();
type ExecutionLedger = Operation<10, 0>;

/// Work caps and optional profiling for one synchronous evaluation.
///
/// One unit admits one VM instruction before dispatch, including instructions
/// reached through synchronous reentry in the same context. Parsing,
/// compilation, and work inside an instruction have their own cost and are
/// outside this instruction cap.
#[derive(Clone, Copy, Debug)]
pub struct ExecutionOptions {
    instruction_limit: u64,
    source_read_limit: u64,
    source_consumption_limit: u64,
    source_retention_limit: u64,
    lexer_lookahead_limit: u64,
    lexer_dispatch_limit: u64,
    token_probe_limit: u64,
    token_advance_limit: u64,
    grammar_dispatch_limit: u64,
    early_error_visit_limit: u64,
    profile: ProfileMode,
    clock: Option<fn() -> Duration>,
}

impl ExecutionOptions {
    /// Allows at most `instruction_limit` VM instructions.
    pub const fn new(instruction_limit: u64) -> Self {
        Self {
            instruction_limit,
            source_read_limit: u64::MAX,
            source_consumption_limit: u64::MAX,
            source_retention_limit: u64::MAX,
            lexer_lookahead_limit: u64::MAX,
            lexer_dispatch_limit: u64::MAX,
            token_probe_limit: u64::MAX,
            token_advance_limit: u64::MAX,
            grammar_dispatch_limit: u64::MAX,
            early_error_visit_limit: u64::MAX,
            profile: ProfileMode::Off,
            clock: None,
        }
    }

    /// Caps input read attempts across source decoders, including EOF probes.
    #[must_use]
    pub const fn with_source_read_limit(mut self, limit: u64) -> Self {
        self.source_read_limit = limit;
        self
    }

    /// Caps decoded code points consumed across frontend phases, including trivia.
    #[must_use]
    pub const fn with_source_consumption_limit(mut self, limit: u64) -> Self {
        self.source_consumption_limit = limit;
        self
    }

    /// Caps cumulative UTF-16 source units retained across frontend phases.
    #[must_use]
    pub const fn with_source_retention_limit(mut self, limit: u64) -> Self {
        self.source_retention_limit = limit;
        self
    }

    /// Caps bounded lexer lookahead probes, including cache hits and EOF probes.
    #[must_use]
    pub const fn with_lexer_lookahead_limit(mut self, limit: u64) -> Self {
        self.lexer_lookahead_limit = limit;
        self
    }

    /// Caps lexical production dispatches, including comments and continuations.
    #[must_use]
    pub const fn with_lexer_dispatch_limit(mut self, limit: u64) -> Self {
        self.lexer_dispatch_limit = limit;
        self
    }

    /// Caps buffered token probes, including cache hits and EOF.
    #[must_use]
    pub const fn with_token_probe_limit(mut self, limit: u64) -> Self {
        self.token_probe_limit = limit;
        self
    }

    /// Caps buffered token advance requests, including requests at EOF.
    #[must_use]
    pub const fn with_token_advance_limit(mut self, limit: u64) -> Self {
        self.token_advance_limit = limit;
        self
    }

    /// Caps grammar production attempts, including inline repeated items.
    #[must_use]
    pub const fn with_grammar_dispatch_limit(mut self, limit: u64) -> Self {
        self.grammar_dispatch_limit = limit;
        self
    }

    /// Caps AST visits in metered early-error walks.
    #[must_use]
    pub const fn with_early_error_visit_limit(mut self, limit: u64) -> Self {
        self.early_error_visit_limit = limit;
        self
    }

    /// Sets profiling for the entire evaluation.
    #[must_use]
    pub const fn with_profile(mut self, profile: ProfileMode) -> Self {
        self.profile = profile;
        self
    }

    /// Supplies an infallible, side-effect-free monotonic clock measured from
    /// a fixed epoch. Untimed modes never call it.
    #[must_use]
    pub const fn with_profile_clock(mut self, clock: fn() -> Duration) -> Self {
        self.clock = Some(clock);
        self
    }
}

/// A failure to start an evaluation before it accepts any work.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExecutionSetupError {
    /// An evaluation in this context already owns the active budget.
    #[error("a controlled evaluation is already active in this context")]
    AlreadyActive,
    /// Storage for the budget or profile could not be reserved.
    #[error("could not allocate execution accounting storage")]
    Allocation,
    /// There are no remaining ledger identities.
    #[error("execution ledger identities exhausted")]
    IdentityExhausted,
    /// The host needs to supply a clock for timed profiling.
    #[error("timed execution profiling requires a host clock")]
    TimingUnavailable,
}

/// The evaluation result and its report, retained even on a host refusal.
#[derive(Debug)]
pub struct ExecutionOutcome {
    /// The script's completion or error.
    pub result: JsResult<JsValue>,
    /// The work admitted during the evaluation.
    pub report: ExecutionReport,
}

/// Admitted work and optional profiling for one evaluation.
#[derive(Debug)]
pub struct ExecutionReport {
    instructions: u64,
    source_reads: u64,
    source_code_points: u64,
    retained_source_units: u64,
    lexer_lookahead_probes: u64,
    lexer_dispatches: u64,
    token_probes: u64,
    token_advances: u64,
    grammar_dispatches: u64,
    early_error_visits: u64,
    profile: Option<Report<10, 0>>,
}

impl ExecutionReport {
    /// Returns the number of admitted instructions.
    pub const fn instructions(&self) -> u64 {
        self.instructions
    }

    /// Returns the number of admitted source read attempts.
    pub const fn source_reads(&self) -> u64 {
        self.source_reads
    }

    /// Returns the number of decoded points consumed by the frontend.
    pub const fn source_code_points(&self) -> u64 {
        self.source_code_points
    }

    /// Returns the cumulative number of UTF-16 units retained in source text.
    pub const fn retained_source_units(&self) -> u64 {
        self.retained_source_units
    }

    /// Returns the number of admitted lexer lookahead probes.
    pub const fn lexer_lookahead_probes(&self) -> u64 {
        self.lexer_lookahead_probes
    }

    /// Returns the number of admitted lexical production dispatches.
    pub const fn lexer_dispatches(&self) -> u64 {
        self.lexer_dispatches
    }

    /// Returns the number of admitted buffered token probes.
    pub const fn token_probes(&self) -> u64 {
        self.token_probes
    }

    /// Returns the number of admitted buffered token advance requests.
    pub const fn token_advances(&self) -> u64 {
        self.token_advances
    }

    /// Returns the number of admitted grammar production attempts.
    pub const fn grammar_dispatches(&self) -> u64 {
        self.grammar_dispatches
    }

    /// Returns the number of admitted early-error AST visits.
    pub const fn early_error_visits(&self) -> u64 {
        self.early_error_visits
    }

    /// Returns the optional profile. Work dimensions are instructions, source
    /// reads, consumed code points, retained UTF-16 source units, and lexer
    /// lookahead probes, lexical production dispatches, buffered token probes,
    /// buffered token advance requests, grammar production attempts, and
    /// early-error AST visits in order.
    pub const fn profile(&self) -> Option<&Report<10, 0>> {
        self.profile.as_ref()
    }
}

pub(crate) struct Execution {
    ledger: ExecutionLedger,
    instruction_limit: u64,
    source_read_limit: u64,
    source_consumption_limit: u64,
    source_retention_limit: u64,
    lexer_lookahead_limit: u64,
    lexer_dispatch_limit: u64,
    token_probe_limit: u64,
    token_advance_limit: u64,
    grammar_dispatch_limit: u64,
    early_error_visit_limit: u64,
    frontend: Scope,
    utf8: Scope,
    utf16: Scope,
    host: Scope,
    stop: Option<JsError>,
}

impl Execution {
    pub(crate) fn new(options: ExecutionOptions) -> Result<Self, ExecutionSetupError> {
        let limits = Limits::new(
            [
                options.instruction_limit,
                options.source_read_limit,
                options.source_consumption_limit,
                options.source_retention_limit,
                options.lexer_lookahead_limit,
                options.lexer_dispatch_limit,
                options.token_probe_limit,
                options.token_advance_limit,
                options.grammar_dispatch_limit,
                options.early_error_visit_limit,
            ],
            [],
        );
        let mut ledger = if let Some(clock) = options.clock {
            Operation::with_profile_clock("evaluation", limits, 5, options.profile, clock)
        } else {
            Operation::with_profile("evaluation", limits, 5, options.profile)
        }
        .map_err(Self::setup_error)?;
        let frontend = ledger
            .child(
                ledger.root(),
                "frontend",
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .map_err(Self::setup_error)?;
        let mut source_scope = |name| {
            ledger
                .child(frontend, name, Limits::unlimited(), Tariffs::identity())
                .map_err(Self::setup_error)
        };
        let utf8 = source_scope("utf8-source")?;
        let utf16 = source_scope("utf16-source")?;
        let host = source_scope("host-source")?;
        Ok(Self {
            ledger,
            instruction_limit: options.instruction_limit,
            source_read_limit: options.source_read_limit,
            source_consumption_limit: options.source_consumption_limit,
            source_retention_limit: options.source_retention_limit,
            lexer_lookahead_limit: options.lexer_lookahead_limit,
            lexer_dispatch_limit: options.lexer_dispatch_limit,
            token_probe_limit: options.token_probe_limit,
            token_advance_limit: options.token_advance_limit,
            grammar_dispatch_limit: options.grammar_dispatch_limit,
            early_error_visit_limit: options.early_error_visit_limit,
            frontend,
            utf8,
            utf16,
            host,
            stop: None,
        })
    }

    fn setup_error(error: LedgerError<10, 0>) -> ExecutionSetupError {
        match error {
            LedgerError::Allocation => ExecutionSetupError::Allocation,
            LedgerError::IdentityExhausted => ExecutionSetupError::IdentityExhausted,
            LedgerError::TimingUnavailable => ExecutionSetupError::TimingUnavailable,
            LedgerError::ScopeLimit { .. }
            | LedgerError::ForeignScope
            | LedgerError::Refused(_) => unreachable!("setup uses its fixed scope tree"),
        }
    }

    pub(crate) fn with_frontend_control<T>(
        &mut self,
        parse: impl FnOnce(&mut FrontendControl<'_>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.ledger
            .with_phase(self.frontend, |phase| {
                let mut refusal = None;
                let result = parse(&mut |work| {
                    if let Some(refusal) = refusal {
                        return Err(refusal);
                    }
                    let admission = match work {
                        FrontendWork::Read(unit) => {
                            let scope = match unit {
                                ReadUnit::Utf8Byte => self.utf8,
                                ReadUnit::Utf16Unit => self.utf16,
                                ReadUnit::HostRead => self.host,
                            };
                            phase.apply(scope, Change::Work(SOURCE_READ, 1))
                        }
                        FrontendWork::Lookahead => {
                            phase.apply(self.frontend, Change::Work(LEXER_LOOKAHEAD, 1))
                        }
                        FrontendWork::LexicalDispatch => {
                            phase.apply(self.frontend, Change::Work(LEXER_DISPATCH, 1))
                        }
                        FrontendWork::TokenProbe => {
                            phase.apply(self.frontend, Change::Work(TOKEN_PROBE, 1))
                        }
                        FrontendWork::TokenAdvance => {
                            phase.apply(self.frontend, Change::Work(TOKEN_ADVANCE, 1))
                        }
                        FrontendWork::GrammarDispatch => {
                            phase.apply(self.frontend, Change::Work(GRAMMAR_DISPATCH, 1))
                        }
                        FrontendWork::EarlyErrorVisit => {
                            phase.apply(self.frontend, Change::Work(EARLY_ERROR_VISIT, 1))
                        }
                        FrontendWork::Consume { retained_units } => phase.apply_batch(
                            self.frontend,
                            &[
                                Change::Work(SOURCE_CONSUMPTION, 1),
                                Change::Work(SOURCE_RETENTION, u64::from(retained_units)),
                            ],
                        ),
                    };
                    match admission {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(denial)) => {
                            let error = match denial.change {
                                Change::Work(SOURCE_READ, _) => {
                                    FrontendLimit::Read(self.source_read_limit)
                                }
                                Change::Work(SOURCE_CONSUMPTION, _) => {
                                    FrontendLimit::Consumption(self.source_consumption_limit)
                                }
                                Change::Work(SOURCE_RETENTION, _) => {
                                    FrontendLimit::Retention(self.source_retention_limit)
                                }
                                Change::Work(LEXER_LOOKAHEAD, _) => {
                                    FrontendLimit::Lookahead(self.lexer_lookahead_limit)
                                }
                                Change::Work(LEXER_DISPATCH, _) => {
                                    FrontendLimit::LexicalDispatch(self.lexer_dispatch_limit)
                                }
                                Change::Work(TOKEN_PROBE, _) => {
                                    FrontendLimit::TokenProbe(self.token_probe_limit)
                                }
                                Change::Work(TOKEN_ADVANCE, _) => {
                                    FrontendLimit::TokenAdvance(self.token_advance_limit)
                                }
                                Change::Work(GRAMMAR_DISPATCH, _) => {
                                    FrontendLimit::GrammarDispatch(self.grammar_dispatch_limit)
                                }
                                Change::Work(EARLY_ERROR_VISIT, _) => {
                                    FrontendLimit::EarlyErrorVisit(self.early_error_visit_limit)
                                }
                                _ => unreachable!("frontend uses only its own work dimensions"),
                            };
                            refusal = Some(error);
                            Err(error)
                        }
                        Err(_) => unreachable!("frontend work uses a preallocated scope"),
                    }
                });
                refusal.map_or(result, |error| Err(RuntimeLimitError::from(error).into()))
            })
            .expect("the ledger owns the frontend scope")
    }

    pub(crate) const fn stop(&self) -> Option<&JsError> {
        self.stop.as_ref()
    }

    pub(crate) fn charge_instruction(&mut self) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        match self
            .ledger
            .apply(self.ledger.root(), Change::Work(INSTRUCTION, 1))
        {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(RuntimeLimitError::Instruction {
                limit: self.instruction_limit,
            }
            .into()),
            Err(_) => unreachable!("charging the operation root requires no setup"),
        }
    }

    pub(crate) fn retain_stop(&mut self, stop: &JsError) {
        self.stop.get_or_insert_with(|| stop.clone());
    }

    pub(crate) fn finish(self, result: JsResult<JsValue>) -> ExecutionOutcome {
        let usage = self
            .ledger
            .usage(self.ledger.root())
            .expect("the ledger owns its root");
        let result = self.stop.map_or(result, Err);
        ExecutionOutcome {
            result,
            report: ExecutionReport {
                instructions: usage.work(INSTRUCTION),
                source_reads: usage.work(SOURCE_READ),
                source_code_points: usage.work(SOURCE_CONSUMPTION),
                retained_source_units: usage.work(SOURCE_RETENTION),
                lexer_lookahead_probes: usage.work(LEXER_LOOKAHEAD),
                lexer_dispatches: usage.work(LEXER_DISPATCH),
                token_probes: usage.work(TOKEN_PROBE),
                token_advances: usage.work(TOKEN_ADVANCE),
                grammar_dispatches: usage.work(GRAMMAR_DISPATCH),
                early_error_visits: usage.work(EARLY_ERROR_VISIT),
                profile: self.ledger.finish(),
            },
        }
    }
}
