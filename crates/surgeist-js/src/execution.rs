//! Work limits and profiling for controlled JavaScript execution.

use std::time::Duration;

use surgeist_ledger::{Change, Error as LedgerError, Limits, Operation, Report, Work};
use thiserror::Error;

use crate::{JsError, JsResult, JsValue, error::RuntimeLimitError};

pub use surgeist_ledger::{EventFilter, ProfileMode};

const INSTRUCTION: Work<1> = Work::new(0).unwrap();
type InstructionLedger = Operation<1, 0>;

/// The instruction cap and optional profiling for one synchronous evaluation.
///
/// One unit admits one VM instruction before dispatch, including instructions
/// reached through synchronous reentry in the same context. Parsing,
/// compilation, and work inside an instruction have their own cost and are
/// outside this instruction cap.
#[derive(Clone, Copy, Debug)]
pub struct ExecutionOptions {
    instruction_limit: u64,
    profile: ProfileMode,
    clock: Option<fn() -> Duration>,
}

impl ExecutionOptions {
    /// Allows at most `instruction_limit` VM instructions.
    pub const fn new(instruction_limit: u64) -> Self {
        Self {
            instruction_limit,
            profile: ProfileMode::Off,
            clock: None,
        }
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

/// Admitted VM work and optional profiling for one evaluation.
#[derive(Debug)]
pub struct ExecutionReport {
    instructions: u64,
    profile: Option<Report<1, 0>>,
}

impl ExecutionReport {
    /// Returns the number of admitted instructions.
    pub const fn instructions(&self) -> u64 {
        self.instructions
    }

    /// Returns the optional profile. Work dimension zero counts instructions.
    pub const fn profile(&self) -> Option<&Report<1, 0>> {
        self.profile.as_ref()
    }
}

pub(crate) struct Execution {
    ledger: InstructionLedger,
    instruction_limit: u64,
    stop: Option<JsError>,
}

impl Execution {
    pub(crate) fn new(options: ExecutionOptions) -> Result<Self, ExecutionSetupError> {
        let limits = Limits::new([options.instruction_limit], []);
        let ledger = if let Some(clock) = options.clock {
            Operation::with_profile_clock("evaluation", limits, 1, options.profile, clock)
        } else {
            Operation::with_profile("evaluation", limits, 1, options.profile)
        }
        .map_err(|error| match error {
            LedgerError::Allocation => ExecutionSetupError::Allocation,
            LedgerError::IdentityExhausted => ExecutionSetupError::IdentityExhausted,
            LedgerError::TimingUnavailable => ExecutionSetupError::TimingUnavailable,
            LedgerError::ScopeLimit { .. }
            | LedgerError::ForeignScope
            | LedgerError::Refused(_) => {
                unreachable!("ledger setup only reserves its own root")
            }
        })?;
        Ok(Self {
            ledger,
            instruction_limit: options.instruction_limit,
            stop: None,
        })
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
        let instructions = self
            .ledger
            .usage(self.ledger.root())
            .expect("the ledger owns its root")
            .work(INSTRUCTION);
        let result = self.stop.map_or(result, Err);
        ExecutionOutcome {
            result,
            report: ExecutionReport {
                instructions,
                profile: self.ledger.finish(),
            },
        }
    }
}
