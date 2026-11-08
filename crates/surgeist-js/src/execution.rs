//! Work limits and profiling for host-selected JavaScript operations.
//!
//! [`Context::with_execution`](crate::Context::with_execution) scopes synchronous
//! reentry under one budget. [`Context::execution_task`](crate::Context::execution_task)
//! reserves a Context across an async callback's suspension. Neither evaluates
//! source nor drains jobs unless the callback selects that work. The ordinary
//! unscoped engine remains unbounded.
//!
//! A task's receipt observes one immutable terminal report, including after
//! cancellation. Completed reports accompany a normal result, which owns any
//! exact first host stop. Abandoned reports own that unreturned stop instead.
//! Report storage is prepared before callback work; finalization moves retained
//! data without copying profile vectors or allocating report storage.
//!
//! Profiling is selected once in [`ExecutionOptions`]. The `execution-lifetime`
//! phase includes suspension. `vm-active` observes synchronous VM calls and
//! individual async quanta; `vm-waiting` observes their cooperative scheduling
//! waits. These are inclusive elapsed intervals, not CPU time or disjoint sums.
//! The `job-prepare`, `job-invoke` and `job-poll` phases observe admitted
//! synchronous job actions and their owned cleanup. Native job preparation now
//! dispatches its owned-task factory and therefore marks possible effects. No job observation spans
//! Pending. Host future internals are opaque; these timings do not certify their work.

use std::time::Duration;

mod operation;
mod report;
pub use operation::ExecutionTask;
use report::PreparedReport;
pub use report::{ExecutionOutcome, ExecutionReceipt, ExecutionReport, ExecutionTermination};

use surgeist_js_ast::{operations::NameWork, scope::ScopeWork};
use surgeist_js_interner::{InternerEncoding, InternerWork};
use surgeist_js_parser::{
    frontend::{FrontendControl, FrontendLimit, FrontendWork, InternerLimitKind},
    source::ReadUnit,
};
use surgeist_js_string::StringWork;
use surgeist_ledger::{
    Change, Error as LedgerError, Limits, Operation, Phase, Scope, Tariffs, Work,
};
use thiserror::Error;

use crate::{
    JsError, JsResult,
    bytecompiler::control::{CompilerControl, CompilerDepth, CompilerWork},
    error::{EngineError, RuntimeLimitError},
    optimizer::control::OptimizerWork,
    value::primitive::PrimitiveWork,
};

pub use surgeist_ledger::{EventFilter, ProfileMode};

const INSTRUCTION: Work<30> = Work::new(0).unwrap();
const SOURCE_READ: Work<30> = Work::new(1).unwrap();
const SOURCE_CONSUMPTION: Work<30> = Work::new(2).unwrap();
const SOURCE_RETENTION: Work<30> = Work::new(3).unwrap();
const LEXER_LOOKAHEAD: Work<30> = Work::new(4).unwrap();
const LEXER_DISPATCH: Work<30> = Work::new(5).unwrap();
const TOKEN_PROBE: Work<30> = Work::new(6).unwrap();
const TOKEN_ADVANCE: Work<30> = Work::new(7).unwrap();
const GRAMMAR_DISPATCH: Work<30> = Work::new(8).unwrap();
const EARLY_ERROR_VISIT: Work<30> = Work::new(9).unwrap();
const LABEL_LOOKUP: Work<30> = Work::new(10).unwrap();
const DIRECT_EVAL_VISIT: Work<30> = Work::new(11).unwrap();
const NAME_VISIT: Work<30> = Work::new(12).unwrap();
const NAME_RECORD: Work<30> = Work::new(13).unwrap();
const NAME_COMPARISON: Work<30> = Work::new(14).unwrap();
const INTERNER_BYTES: Work<30> = Work::new(15).unwrap();
const INTERNER_UTF16: Work<30> = Work::new(16).unwrap();
const INTERNER_INDEX: Work<30> = Work::new(17).unwrap();
const INTERNER_CAPACITY: Work<30> = Work::new(18).unwrap();
const INTERNER_RELOCATION: Work<30> = Work::new(19).unwrap();
const DIAGNOSTIC_INPUT: Work<30> = Work::new(20).unwrap();
const DIAGNOSTIC_OUTPUT: Work<30> = Work::new(21).unwrap();
const TEMPLATE_INPUT: Work<30> = Work::new(22).unwrap();
const TEMPLATE_OUTPUT: Work<30> = Work::new(23).unwrap();
const SCOPE_WORK: Work<30> = Work::new(24).unwrap();
const OPTIMIZER_WORK: Work<30> = Work::new(25).unwrap();
const COMPILER_WORK: Work<30> = Work::new(26).unwrap();
const JOB_PREPARATION: Work<30> = Work::new(27).unwrap();
const JOB_INVOCATION: Work<30> = Work::new(28).unwrap();
const JOB_POLL: Work<30> = Work::new(29).unwrap();
const OPTIMIZER_ORIGINS: [&str; 21] = [
    "optimizer-visit",
    "optimizer-iteration",
    "optimizer-fold-attempt",
    "optimizer-reduction-attempt",
    "optimizer-rewrite",
    "optimizer-declaration-visit",
    "optimizer-retirement-visit",
    "optimizer-retirement-reservation",
    "optimizer-literal-copy",
    "optimizer-literal-capacity",
    "primitive-probe",
    "bigint-copy",
    "bigint-kernel",
    "bigint-convert",
    "string-scan",
    "string-copy",
    "string-allocate",
    "string-fast-float",
    "string-checkpoint",
    "bigint-read",
    "bigint-capacity",
];
const COMPILER_ORIGINS: [&str; 32] = [
    "compiler-visit",
    "compiler-search",
    "compiler-register-probe",
    "compiler-register-write",
    "compiler-index-probe",
    "compiler-index-write",
    "compiler-metadata-visit",
    "compiler-metadata-write",
    "compiler-opcode",
    "compiler-emit-bytes",
    "compiler-patch-bytes",
    "compiler-read",
    "compiler-copy",
    "compiler-capacity",
    "compiler-relocate",
    "compiler-depth",
    "primitive-probe",
    "bigint-copy",
    "bigint-kernel",
    "bigint-convert",
    "string-scan",
    "string-copy",
    "string-allocate",
    "string-fast-float",
    "string-checkpoint",
    "bigint-read",
    "bigint-capacity",
    "name-visit",
    "name-record",
    "name-comparison",
    "compiler-name-set-iteration",
    "compiler-environment-probe",
];
// Root, frontend + four sources, three scope phases + 36 origins, optimizer
// + 21 origins, 32 compiler origins, three lifetime/VM scopes and three job scopes.
const FIXED_SCOPES: usize = 1
    + 1
    + 4
    + 3
    + 3 * SCOPE_ORIGINS.len()
    + 1
    + OPTIMIZER_ORIGINS.len()
    + COMPILER_ORIGINS.len()
    + 6;
const SCOPE_ORIGINS: [&str; 12] = [
    "scope-node",
    "scope-probe",
    "scope-hop",
    "scope-binding",
    "scope-name-unit",
    "scope-name-preparation",
    "scope-binding-write",
    "scope-write",
    "scope-record",
    "scope-capacity",
    "scope-publication",
    "scope-context-validation",
];
type ExecutionLedger = Operation<30, 0>;

// The shared producer offsets retain the existing optimizer origin order.
fn string_charge(work: StringWork) -> (usize, u64) {
    match work {
        StringWork::Scan { units } => (4, units as u64),
        StringWork::Copy { units } => (5, units as u64),
        StringWork::Allocate { bytes } => (6, bytes as u64),
        StringWork::FastFloat { bytes } => (7, bytes as u64),
        StringWork::Checkpoint => (8, 0),
    }
}

fn primitive_charge(work: PrimitiveWork) -> (usize, u64) {
    match work {
        PrimitiveWork::Checkpoint => (0, 0),
        PrimitiveWork::BigIntCopy { words } => (1, words),
        PrimitiveWork::BigIntKernel { units } => (2, units),
        PrimitiveWork::BigIntConvert { units } => (3, units),
        PrimitiveWork::String(work) => string_charge(work),
        PrimitiveWork::BigIntRead { words } => (9, words),
        PrimitiveWork::BigIntCapacity { bytes } => (10, bytes),
    }
}

fn compiler_charge(work: CompilerWork) -> (usize, Work<30>, u64) {
    let (origin, units) = match work {
        CompilerWork::Visit => (0, 1),
        CompilerWork::Search => (1, 1),
        CompilerWork::RegisterProbe => (2, 1),
        CompilerWork::RegisterWrite => (3, 1),
        CompilerWork::IndexProbe => (4, 1),
        CompilerWork::IndexWrite => (5, 1),
        CompilerWork::MetadataVisit => (6, 1),
        CompilerWork::MetadataWrite => (7, 1),
        CompilerWork::Opcode => (8, 1),
        CompilerWork::EmitBytes { bytes } => (9, bytes as u64),
        CompilerWork::PatchBytes { bytes } => (10, bytes as u64),
        CompilerWork::Read { bytes } => (11, bytes as u64),
        CompilerWork::Copy { bytes } => (12, bytes as u64),
        CompilerWork::Capacity { bytes } => (13, bytes as u64),
        CompilerWork::Relocate { bytes } => (14, bytes as u64),
        CompilerWork::Depth { .. } => (15, 0),
        CompilerWork::NameSetIteration { capacity } => (30, capacity as u64),
        CompilerWork::EnvironmentProbe => (31, 1),
        CompilerWork::Primitive(work) => {
            let (origin, units) = primitive_charge(work);
            (16 + origin, units)
        }
        CompilerWork::String(work) => {
            let (origin, units) = string_charge(work);
            (16 + origin, units)
        }
        CompilerWork::Name(work) => {
            return match work {
                NameWork::Visit => (27, NAME_VISIT, 1),
                NameWork::Record => (28, NAME_RECORD, 1),
                NameWork::Comparison => (29, NAME_COMPARISON, 1),
            };
        }
        CompilerWork::Scope(_) => unreachable!("scope work retains its existing phase origins"),
    };
    (origin, COMPILER_WORK, units)
}

fn scope_charge(work: ScopeWork) -> (usize, u64) {
    match work {
        ScopeWork::Node => (0, 1),
        ScopeWork::ScopeProbe => (1, 1),
        ScopeWork::ScopeHop => (2, 1),
        ScopeWork::BindingProbe => (3, 1),
        ScopeWork::NameUnit => (4, 1),
        ScopeWork::NamePreparation => (5, 1),
        ScopeWork::BindingWrite => (6, 1),
        ScopeWork::ScopeWrite => (7, 1),
        ScopeWork::Record => (8, 1),
        ScopeWork::Capacity { elements } => (9, elements as u64),
        ScopeWork::Publication => (10, 1),
        ScopeWork::ContextValidation { contexts } => (11, contexts as u64),
    }
}

fn charge_interner(
    phase: &mut Phase<'_, 30, 0>,
    scope: Scope,
    work: InternerWork,
) -> Result<(), LedgerError<30, 0>> {
    match work {
        InternerWork::Hash { encoding, units }
        | InternerWork::Compare { encoding, units }
        | InternerWork::Convert { encoding, units } => {
            let dimension = match encoding {
                InternerEncoding::Utf8 | InternerEncoding::Latin1 => INTERNER_BYTES,
                InternerEncoding::Utf16 => INTERNER_UTF16,
            };
            phase.apply(scope, Change::Work(dimension, units as u64))
        }
        InternerWork::Classify { units } => {
            phase.apply(scope, Change::Work(INTERNER_UTF16, units as u64))
        }
        InternerWork::ValidateStatic {
            utf8_bytes,
            utf16_units,
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(INTERNER_BYTES, utf8_bytes as u64),
                Change::Work(INTERNER_UTF16, utf16_units as u64),
            ],
        ),
        InternerWork::Probe | InternerWork::Reinsert => {
            phase.apply(scope, Change::Work(INTERNER_INDEX, 1))
        }
        InternerWork::InitializeSlots { slots } => {
            phase.apply(scope, Change::Work(INTERNER_INDEX, slots as u64))
        }
        InternerWork::Reserve {
            additional,
            existing,
            ..
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(INTERNER_CAPACITY, additional as u64),
                Change::Work(INTERNER_RELOCATION, existing as u64),
            ],
        ),
        InternerWork::Publish {
            utf8_bytes,
            utf16_units,
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(INTERNER_INDEX, 1),
                Change::Work(INTERNER_BYTES, utf8_bytes as u64),
                Change::Work(INTERNER_UTF16, utf16_units as u64),
            ],
        ),
    }
}

/// Work caps and optional profiling for one host-selected operation.
///
/// One unit admits one VM instruction before dispatch, including instructions
/// reached through synchronous reentry in the same context. Parsing,
/// compilation, and work inside an instruction have their own cost and are
/// outside this instruction cap.
#[derive(Clone, Copy, Debug)]
pub struct ExecutionOptions {
    instruction_limit: u64,
    job_preparation_limit: u64,
    job_invocation_limit: u64,
    job_poll_limit: u64,
    source_read_limit: u64,
    source_consumption_limit: u64,
    source_retention_limit: u64,
    lexer_lookahead_limit: u64,
    lexer_dispatch_limit: u64,
    token_probe_limit: u64,
    token_advance_limit: u64,
    grammar_dispatch_limit: u64,
    early_error_visit_limit: u64,
    label_lookup_limit: u64,
    name_visit_limit: u64,
    name_record_limit: u64,
    name_comparison_limit: u64,
    direct_eval_visit_limit: u64,
    interner_byte_limit: u64,
    interner_utf16_limit: u64,
    interner_index_limit: u64,
    interner_capacity_limit: u64,
    interner_relocation_limit: u64,
    diagnostic_input_limit: u64,
    diagnostic_output_limit: u64,
    template_input_limit: u64,
    template_output_limit: u64,
    scope_work_limit: u64,
    optimizer_work_limit: u64,
    compiler_work_limit: u64,
    compiler_depth_limit: usize,
    profile: ProfileMode,
    clock: Option<fn() -> Duration>,
}

impl ExecutionOptions {
    /// Allows at most `instruction_limit` VM instructions.
    pub const fn new(instruction_limit: u64) -> Self {
        Self {
            instruction_limit,
            job_preparation_limit: u64::MAX,
            job_invocation_limit: u64::MAX,
            job_poll_limit: u64::MAX,
            source_read_limit: u64::MAX,
            source_consumption_limit: u64::MAX,
            source_retention_limit: u64::MAX,
            lexer_lookahead_limit: u64::MAX,
            lexer_dispatch_limit: u64::MAX,
            token_probe_limit: u64::MAX,
            token_advance_limit: u64::MAX,
            grammar_dispatch_limit: u64::MAX,
            early_error_visit_limit: u64::MAX,
            label_lookup_limit: u64::MAX,
            name_visit_limit: u64::MAX,
            name_record_limit: u64::MAX,
            name_comparison_limit: u64::MAX,
            direct_eval_visit_limit: u64::MAX,
            interner_byte_limit: u64::MAX,
            interner_utf16_limit: u64::MAX,
            interner_index_limit: u64::MAX,
            interner_capacity_limit: u64::MAX,
            interner_relocation_limit: u64::MAX,
            diagnostic_input_limit: u64::MAX,
            diagnostic_output_limit: u64::MAX,
            template_input_limit: u64::MAX,
            template_output_limit: u64::MAX,
            scope_work_limit: u64::MAX,
            optimizer_work_limit: u64::MAX,
            compiler_work_limit: u64::MAX,
            compiler_depth_limit: usize::MAX,
            profile: ProfileMode::Off,
            clock: None,
        }
    }

    /// Caps admitted native task preparation factory invocations.
    ///
    /// A factory counts once even if it returns an error or unwinds. Dispatching
    /// it marks possible effects; preparation admits no inner task poll.
    #[must_use]
    pub const fn with_job_preparation_limit(mut self, limit: u64) -> Self {
        self.job_preparation_limit = limit;
        self
    }

    /// Caps transfers of control to synchronous native job callbacks.
    #[must_use]
    pub const fn with_job_invocation_limit(mut self, limit: u64) -> Self {
        self.job_invocation_limit = limit;
        self
    }

    /// Caps actual native asynchronous job future polls, including Pending polls.
    #[must_use]
    pub const fn with_job_poll_limit(mut self, limit: u64) -> Self {
        self.job_poll_limit = limit;
        self
    }

    /// Caps optimizer traversal, rewrite preparation and delegated operand work.
    ///
    /// Units are cumulative requests defined at their producer. Opaque BigInt
    /// kernels use size-dependent native reference prices, not internal operation
    /// counts or elapsed-time limits. Interner work retains its separate caps.
    #[must_use]
    pub const fn with_optimizer_work_limit(mut self, limit: u64) -> Self {
        self.optimizer_work_limit = limit;
        self
    }

    /// Caps cumulative compiler lowering and delegated payload work.
    ///
    /// Units describe logical visits, searches, writes and requested payload
    /// extents. They are not elapsed time, physical memory or a guarantee that
    /// allocation succeeds. Scope and AST name work retain their separate caps.
    #[must_use]
    pub const fn with_compiler_work_limit(mut self, limit: u64) -> Self {
        self.compiler_work_limit = limit;
        self
    }

    /// Caps simultaneously active recursive compiler entries.
    ///
    /// The prospective depth is checked before entry. Nested compilers share
    /// their compilation phase's depth state; successful guard release does not
    /// request further admission. This is distinct from cumulative work.
    #[must_use]
    pub const fn with_compiler_depth_limit(mut self, limit: usize) -> Self {
        self.compiler_depth_limit = limit;
        self
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

    /// Caps logical label-table lookups.
    #[must_use]
    pub const fn with_label_lookup_limit(mut self, limit: u64) -> Self {
        self.label_lookup_limit = limit;
        self
    }

    /// Caps AST visits that compute cached direct-eval metadata.
    #[must_use]
    pub const fn with_direct_eval_visit_limit(mut self, limit: u64) -> Self {
        self.direct_eval_visit_limit = limit;
        self
    }

    /// Caps AST visits in name collection, validation and delegated semantic queries.
    /// Compiler completion-value and source-start queries share this cap.
    #[must_use]
    pub const fn with_name_visit_limit(mut self, limit: u64) -> Self {
        self.name_visit_limit = limit;
        self
    }

    /// Caps name record attempts.
    #[must_use]
    pub const fn with_name_record_limit(mut self, limit: u64) -> Self {
        self.name_record_limit = limit;
        self
    }

    /// Caps symbol comparisons or logical name-table operations.
    #[must_use]
    pub const fn with_name_comparison_limit(mut self, limit: u64) -> Self {
        self.name_comparison_limit = limit;
        self
    }

    /// Caps UTF-8 and Latin1 byte work in frontend interning.
    #[must_use]
    pub const fn with_interner_byte_limit(mut self, limit: u64) -> Self {
        self.interner_byte_limit = limit;
        self
    }

    /// Caps UTF-16 unit work in frontend interning and name preparation.
    #[must_use]
    pub const fn with_interner_utf16_limit(mut self, limit: u64) -> Self {
        self.interner_utf16_limit = limit;
        self
    }

    /// Caps frontend interner slot initialization, probes, reinsertion and publication.
    #[must_use]
    pub const fn with_interner_index_limit(mut self, limit: u64) -> Self {
        self.interner_index_limit = limit;
        self
    }

    /// Caps cumulative additional capacity requested by frontend interner buffers.
    #[must_use]
    pub const fn with_interner_capacity_limit(mut self, limit: u64) -> Self {
        self.interner_capacity_limit = limit;
        self
    }

    /// Caps cumulative initialized units that frontend interner reserves may relocate.
    #[must_use]
    pub const fn with_interner_relocation_limit(mut self, limit: u64) -> Self {
        self.interner_relocation_limit = limit;
        self
    }

    /// Caps UTF-16 spelling units inspected while rendering frontend diagnostics.
    #[must_use]
    pub const fn with_diagnostic_input_limit(mut self, limit: u64) -> Self {
        self.diagnostic_input_limit = limit;
        self
    }

    /// Caps UTF-8 bytes appended to owned source-spelling diagnostic payloads.
    #[must_use]
    pub const fn with_diagnostic_output_limit(mut self, limit: u64) -> Self {
        self.diagnostic_output_limit = limit;
        self
    }

    /// Caps UTF-16 units inspected by raw and cooked template preparation.
    #[must_use]
    pub const fn with_template_input_limit(mut self, limit: u64) -> Self {
        self.template_input_limit = limit;
        self
    }

    /// Caps UTF-16 units appended to normalized raw and cooked templates.
    #[must_use]
    pub const fn with_template_output_limit(mut self, limit: u64) -> Self {
        self.template_output_limit = limit;
        self
    }

    /// Caps script scope analysis work, including staged changes and publication.
    /// Individual work origins remain available in the optional profile.
    #[must_use]
    pub const fn with_scope_work_limit(mut self, limit: u64) -> Self {
        self.scope_work_limit = limit;
        self
    }

    /// Sets profiling for the entire selected operation.
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

/// A failure to start an operation before it accepts any work.
#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ExecutionSetupError {
    /// An operation in this context already owns the active budget.
    #[error("a controlled operation is already active in this context")]
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

#[derive(Clone, Copy)]
pub(crate) enum ExecutionPhase {
    Lifetime,
    VmActive,
    VmWaiting,
    JobPreparation,
    JobInvocation,
    JobPoll,
}

#[derive(Clone, Copy)]
pub(crate) enum JobAction {
    Preparation,
    Invocation,
    Poll,
}

impl JobAction {
    pub(crate) const fn phase(self) -> ExecutionPhase {
        match self {
            Self::Preparation => ExecutionPhase::JobPreparation,
            Self::Invocation => ExecutionPhase::JobInvocation,
            Self::Poll => ExecutionPhase::JobPoll,
        }
    }
}

pub(crate) struct Execution {
    ledger: ExecutionLedger,
    prepared_report: PreparedReport,
    execution_phases: [Scope; 6],
    instruction_limit: u64,
    job_preparation_limit: u64,
    job_invocation_limit: u64,
    job_poll_limit: u64,
    source_read_limit: u64,
    source_consumption_limit: u64,
    source_retention_limit: u64,
    lexer_lookahead_limit: u64,
    lexer_dispatch_limit: u64,
    token_probe_limit: u64,
    token_advance_limit: u64,
    grammar_dispatch_limit: u64,
    early_error_visit_limit: u64,
    label_lookup_limit: u64,
    name_visit_limit: u64,
    name_record_limit: u64,
    name_comparison_limit: u64,
    direct_eval_visit_limit: u64,
    interner_byte_limit: u64,
    interner_utf16_limit: u64,
    interner_index_limit: u64,
    interner_capacity_limit: u64,
    interner_relocation_limit: u64,
    diagnostic_input_limit: u64,
    diagnostic_output_limit: u64,
    template_input_limit: u64,
    template_output_limit: u64,
    scope_work_limit: u64,
    optimizer_work_limit: u64,
    compiler_work_limit: u64,
    compiler_depth_limit: usize,
    frontend: Scope,
    optimizer: Scope,
    optimizer_origins: [Scope; OPTIMIZER_ORIGINS.len()],
    compiler_origins: [Scope; COMPILER_ORIGINS.len()],
    compiler_depth_peak: usize,
    scope_phases: [Scope; 3],
    scope_origins: [[Scope; 12]; 3],
    utf8: Scope,
    utf16: Scope,
    host: Scope,
    latin1: Scope,
    stop: Option<JsError>,
    effects_started: bool,
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
                options.label_lookup_limit,
                options.direct_eval_visit_limit,
                options.name_visit_limit,
                options.name_record_limit,
                options.name_comparison_limit,
                options.interner_byte_limit,
                options.interner_utf16_limit,
                options.interner_index_limit,
                options.interner_capacity_limit,
                options.interner_relocation_limit,
                options.diagnostic_input_limit,
                options.diagnostic_output_limit,
                options.template_input_limit,
                options.template_output_limit,
                options.scope_work_limit,
                options.optimizer_work_limit,
                options.compiler_work_limit,
                options.job_preparation_limit,
                options.job_invocation_limit,
                options.job_poll_limit,
            ],
            [],
        );
        let mut ledger = if let Some(clock) = options.clock {
            Operation::with_profile_clock("execution", limits, FIXED_SCOPES, options.profile, clock)
        } else {
            Operation::with_profile("execution", limits, FIXED_SCOPES, options.profile)
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
        let latin1 = source_scope("latin1-source")?;
        let mut scope_phases = [frontend; 3];
        let mut scope_origins = [[frontend; 12]; 3];
        for ((phase, origins), name) in scope_phases.iter_mut().zip(scope_origins.iter_mut()).zip([
            "scope-analysis",
            "bytecode-compilation",
            "script-installation",
        ]) {
            *phase = ledger
                .child(
                    ledger.root(),
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
            for (slot, origin) in origins.iter_mut().zip(SCOPE_ORIGINS) {
                *slot = ledger
                    .child(*phase, origin, Limits::unlimited(), Tariffs::identity())
                    .map_err(Self::setup_error)?;
            }
        }
        let optimizer = ledger
            .child(
                ledger.root(),
                "optimizer",
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .map_err(Self::setup_error)?;
        let mut optimizer_origins = [optimizer; OPTIMIZER_ORIGINS.len()];
        for (slot, name) in optimizer_origins.iter_mut().zip(OPTIMIZER_ORIGINS) {
            *slot = ledger
                .child(optimizer, name, Limits::unlimited(), Tariffs::identity())
                .map_err(Self::setup_error)?;
        }
        let mut compiler_origins = [scope_phases[1]; COMPILER_ORIGINS.len()];
        for (slot, name) in compiler_origins.iter_mut().zip(COMPILER_ORIGINS) {
            *slot = ledger
                .child(
                    scope_phases[1],
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
        }
        let mut execution_phases = [ledger.root(); 6];
        for (slot, name) in execution_phases.iter_mut().zip([
            "execution-lifetime",
            "vm-active",
            "vm-waiting",
            "job-prepare",
            "job-invoke",
            "job-poll",
        ]) {
            *slot = ledger
                .child(
                    ledger.root(),
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
        }
        Ok(Self {
            ledger,
            prepared_report: PreparedReport::new(),
            execution_phases,
            instruction_limit: options.instruction_limit,
            job_preparation_limit: options.job_preparation_limit,
            job_invocation_limit: options.job_invocation_limit,
            job_poll_limit: options.job_poll_limit,
            source_read_limit: options.source_read_limit,
            source_consumption_limit: options.source_consumption_limit,
            source_retention_limit: options.source_retention_limit,
            lexer_lookahead_limit: options.lexer_lookahead_limit,
            lexer_dispatch_limit: options.lexer_dispatch_limit,
            token_probe_limit: options.token_probe_limit,
            token_advance_limit: options.token_advance_limit,
            grammar_dispatch_limit: options.grammar_dispatch_limit,
            early_error_visit_limit: options.early_error_visit_limit,
            label_lookup_limit: options.label_lookup_limit,
            name_visit_limit: options.name_visit_limit,
            name_record_limit: options.name_record_limit,
            name_comparison_limit: options.name_comparison_limit,
            direct_eval_visit_limit: options.direct_eval_visit_limit,
            interner_byte_limit: options.interner_byte_limit,
            interner_utf16_limit: options.interner_utf16_limit,
            interner_index_limit: options.interner_index_limit,
            interner_capacity_limit: options.interner_capacity_limit,
            interner_relocation_limit: options.interner_relocation_limit,
            diagnostic_input_limit: options.diagnostic_input_limit,
            diagnostic_output_limit: options.diagnostic_output_limit,
            template_input_limit: options.template_input_limit,
            template_output_limit: options.template_output_limit,
            scope_work_limit: options.scope_work_limit,
            optimizer_work_limit: options.optimizer_work_limit,
            compiler_work_limit: options.compiler_work_limit,
            compiler_depth_limit: options.compiler_depth_limit,
            frontend,
            optimizer,
            optimizer_origins,
            compiler_origins,
            compiler_depth_peak: 0,
            scope_phases,
            scope_origins,
            utf8,
            utf16,
            host,
            latin1,
            stop: None,
            effects_started: false,
        })
    }

    fn setup_error(error: LedgerError<30, 0>) -> ExecutionSetupError {
        match error {
            LedgerError::Allocation => ExecutionSetupError::Allocation,
            LedgerError::IdentityExhausted => ExecutionSetupError::IdentityExhausted,
            LedgerError::TimingUnavailable => ExecutionSetupError::TimingUnavailable,
            LedgerError::ScopeLimit { .. }
            | LedgerError::ForeignScope
            | LedgerError::ForeignCheckpoint
            | LedgerError::StaleCheckpoint
            | LedgerError::CheckpointLimit { .. }
            | LedgerError::CheckpointsConfigured
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
                        FrontendWork::Scope(work) => {
                            let (origin, units) = scope_charge(work);
                            phase.apply(
                                self.scope_origins[0][origin],
                                Change::Work(SCOPE_WORK, units),
                            )
                        }
                        FrontendWork::Read(unit) => {
                            let scope = match unit {
                                ReadUnit::Utf8Byte => self.utf8,
                                ReadUnit::Utf16Unit => self.utf16,
                                ReadUnit::HostRead => self.host,
                                ReadUnit::Latin1Byte => self.latin1,
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
                        FrontendWork::LabelLookup => {
                            phase.apply(self.frontend, Change::Work(LABEL_LOOKUP, 1))
                        }
                        FrontendWork::DirectEvalVisit => {
                            phase.apply(self.frontend, Change::Work(DIRECT_EVAL_VISIT, 1))
                        }
                        FrontendWork::NameVisit => {
                            phase.apply(self.frontend, Change::Work(NAME_VISIT, 1))
                        }
                        FrontendWork::NameRecord => {
                            phase.apply(self.frontend, Change::Work(NAME_RECORD, 1))
                        }
                        FrontendWork::NameComparison => {
                            phase.apply(self.frontend, Change::Work(NAME_COMPARISON, 1))
                        }
                        FrontendWork::Interner(work) => charge_interner(phase, self.frontend, work),
                        FrontendWork::DiagnosticInput => {
                            phase.apply(self.frontend, Change::Work(DIAGNOSTIC_INPUT, 1))
                        }
                        FrontendWork::DiagnosticOutput(bytes) => phase
                            .apply(self.frontend, Change::Work(DIAGNOSTIC_OUTPUT, bytes as u64)),
                        FrontendWork::TemplateInput => {
                            phase.apply(self.frontend, Change::Work(TEMPLATE_INPUT, 1))
                        }
                        FrontendWork::TemplateOutput(units) => phase.apply(
                            self.frontend,
                            Change::Work(TEMPLATE_OUTPUT, u64::from(units)),
                        ),
                        FrontendWork::InternerNamePreparation(units) => {
                            phase.apply(self.frontend, Change::Work(INTERNER_UTF16, units as u64))
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
                                Change::Work(LABEL_LOOKUP, _) => {
                                    FrontendLimit::LabelLookup(self.label_lookup_limit)
                                }
                                Change::Work(DIRECT_EVAL_VISIT, _) => {
                                    FrontendLimit::DirectEvalVisit(self.direct_eval_visit_limit)
                                }
                                Change::Work(NAME_VISIT, _) => {
                                    FrontendLimit::NameVisit(self.name_visit_limit)
                                }
                                Change::Work(NAME_RECORD, _) => {
                                    FrontendLimit::NameRecord(self.name_record_limit)
                                }
                                Change::Work(NAME_COMPARISON, _) => {
                                    FrontendLimit::NameComparison(self.name_comparison_limit)
                                }
                                Change::Work(INTERNER_BYTES, _) => FrontendLimit::Interner {
                                    kind: InternerLimitKind::Bytes,
                                    limit: self.interner_byte_limit,
                                },
                                Change::Work(INTERNER_UTF16, _) => FrontendLimit::Interner {
                                    kind: InternerLimitKind::Utf16,
                                    limit: self.interner_utf16_limit,
                                },
                                Change::Work(INTERNER_INDEX, _) => FrontendLimit::Interner {
                                    kind: InternerLimitKind::Index,
                                    limit: self.interner_index_limit,
                                },
                                Change::Work(INTERNER_CAPACITY, _) => FrontendLimit::Interner {
                                    kind: InternerLimitKind::Capacity,
                                    limit: self.interner_capacity_limit,
                                },
                                Change::Work(DIAGNOSTIC_INPUT, _) => {
                                    FrontendLimit::DiagnosticInput(self.diagnostic_input_limit)
                                }
                                Change::Work(DIAGNOSTIC_OUTPUT, _) => {
                                    FrontendLimit::DiagnosticOutput(self.diagnostic_output_limit)
                                }
                                Change::Work(TEMPLATE_INPUT, _) => {
                                    FrontendLimit::TemplateInput(self.template_input_limit)
                                }
                                Change::Work(TEMPLATE_OUTPUT, _) => {
                                    FrontendLimit::TemplateOutput(self.template_output_limit)
                                }
                                Change::Work(SCOPE_WORK, _) => {
                                    FrontendLimit::Scope(self.scope_work_limit)
                                }
                                Change::Work(INTERNER_RELOCATION, _) => FrontendLimit::Interner {
                                    kind: InternerLimitKind::Relocation,
                                    limit: self.interner_relocation_limit,
                                },
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

    pub(crate) fn with_scope_control<T>(
        &mut self,
        run: impl FnOnce(&mut dyn FnMut(ScopeWork) -> Result<(), FrontendLimit>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_scope_phase(0, run)
    }

    pub(crate) fn with_optimizer_control<T>(
        &mut self,
        run: impl FnOnce(&mut dyn FnMut(OptimizerWork) -> JsResult<()>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.ledger
            .with_phase(self.optimizer, |phase| {
                let mut refusal: Option<JsError> = None;
                let result = run(&mut |work| {
                    if let Some(error) = &refusal {
                        return Err(error.clone());
                    }
                    let admission = if let OptimizerWork::Interner(work) = work {
                        charge_interner(phase, self.optimizer, work)
                    } else {
                        let (origin, units) = match work {
                            OptimizerWork::Visit => (0, 1),
                            OptimizerWork::Iteration => (1, 1),
                            OptimizerWork::FoldAttempt => (2, 1),
                            OptimizerWork::ReductionAttempt => (3, 1),
                            OptimizerWork::Rewrite => (4, 1),
                            OptimizerWork::DeclarationVisit => (5, 1),
                            OptimizerWork::RetirementVisit => (6, 1),
                            OptimizerWork::RetirementReservation => (7, 1),
                            OptimizerWork::LiteralCopy { words } => (8, words),
                            OptimizerWork::LiteralCapacity { elements } => (9, elements),
                            OptimizerWork::Primitive(work) => {
                                let (origin, units) = primitive_charge(work);
                                (10 + origin, units)
                            }
                            OptimizerWork::Interner(_) => {
                                unreachable!("handled by the interner owner")
                            }
                        };
                        phase.apply(
                            self.optimizer_origins[origin],
                            Change::Work(OPTIMIZER_WORK, units),
                        )
                    };
                    match admission {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(denial)) => {
                            let error = match denial.change {
                                Change::Work(INTERNER_BYTES, _) => RuntimeLimitError::Interner {
                                    kind: InternerLimitKind::Bytes,
                                    limit: self.interner_byte_limit,
                                },
                                Change::Work(INTERNER_UTF16, _) => RuntimeLimitError::Interner {
                                    kind: InternerLimitKind::Utf16,
                                    limit: self.interner_utf16_limit,
                                },
                                Change::Work(INTERNER_INDEX, _) => RuntimeLimitError::Interner {
                                    kind: InternerLimitKind::Index,
                                    limit: self.interner_index_limit,
                                },
                                Change::Work(INTERNER_CAPACITY, _) => RuntimeLimitError::Interner {
                                    kind: InternerLimitKind::Capacity,
                                    limit: self.interner_capacity_limit,
                                },
                                Change::Work(INTERNER_RELOCATION, _) => {
                                    RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Relocation,
                                        limit: self.interner_relocation_limit,
                                    }
                                }
                                Change::Work(OPTIMIZER_WORK, _) => {
                                    RuntimeLimitError::OptimizerWork {
                                        limit: self.optimizer_work_limit,
                                    }
                                }
                                _ => unreachable!(
                                    "optimizer charges only its own and interner dimensions"
                                ),
                            };
                            let error = JsError::from(error);
                            refusal = Some(error.clone());
                            Err(error)
                        }
                        Err(_) => unreachable!("optimizer work uses preallocated origins"),
                    }
                });
                refusal.map_or(result, Err)
            })
            .expect("the ledger owns the optimizer phase")
    }

    pub(crate) fn with_compilation_control<T>(
        &mut self,
        run: impl FnOnce(CompilerControl<'_>) -> JsResult<T>,
    ) -> JsResult<T> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let depth = CompilerDepth::new();
        let stop = &mut self.stop;
        let result = self
            .ledger
            .with_phase(self.scope_phases[1], |phase| {
                let mut admit = |work| {
                    if let Some(error) = stop.as_ref() {
                        return Err(error.clone());
                    }
                    if let CompilerWork::Depth { depth } = work
                        && depth > self.compiler_depth_limit
                    {
                        let error = JsError::from(RuntimeLimitError::CompilerDepth {
                            limit: self.compiler_depth_limit,
                        });
                        *stop = Some(error.clone());
                        return Err(error);
                    }
                    let (scope, dimension, units) = if let CompilerWork::Scope(work) = work {
                        let (origin, units) = scope_charge(work);
                        (self.scope_origins[1][origin], SCOPE_WORK, units)
                    } else {
                        let (origin, dimension, units) = compiler_charge(work);
                        (self.compiler_origins[origin], dimension, units)
                    };
                    match phase.apply(scope, Change::Work(dimension, units)) {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(_)) => {
                            let limit = match dimension {
                                COMPILER_WORK => RuntimeLimitError::CompilerWork {
                                    limit: self.compiler_work_limit,
                                },
                                SCOPE_WORK => RuntimeLimitError::ScopeWork {
                                    limit: self.scope_work_limit,
                                },
                                NAME_VISIT => RuntimeLimitError::NameVisit {
                                    limit: self.name_visit_limit,
                                },
                                NAME_RECORD => RuntimeLimitError::NameRecord {
                                    limit: self.name_record_limit,
                                },
                                NAME_COMPARISON => RuntimeLimitError::NameComparison {
                                    limit: self.name_comparison_limit,
                                },
                                _ => unreachable!("compiler uses its own and delegated dimensions"),
                            };
                            let error = JsError::from(limit);
                            // Latch before returning to the producer or closing
                            // the timed phase. Later errors cannot replace it.
                            *stop = Some(error.clone());
                            Err(error)
                        }
                        Err(_) => unreachable!("compiler work uses preallocated origins"),
                    }
                };
                let result = run(CompilerControl::new(Some(&mut admit), &depth));
                if let Err(error) = &result
                    && (matches!(error.as_engine(), Some(EngineError::ScopeAnalysis(_) | EngineError::Resource(_)))
                        || matches!(error.as_engine(), Some(EngineError::RuntimeLimit(limit)) if limit.is_frontend_limit()))
                {
                    stop.get_or_insert_with(|| error.clone());
                }
                stop.as_ref().map_or(result, |error| Err(error.clone()))
            })
            .expect("the ledger owns the compilation phase");
        self.compiler_depth_peak = self.compiler_depth_peak.max(depth.peak());
        self.stop
            .as_ref()
            .map_or(result, |error| Err(error.clone()))
    }

    pub(crate) fn with_installation_control<T>(
        &mut self,
        run: impl FnOnce(&mut dyn FnMut(ScopeWork) -> Result<(), FrontendLimit>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.with_scope_phase(2, run)
    }

    fn with_scope_phase<T>(
        &mut self,
        index: usize,
        run: impl FnOnce(&mut dyn FnMut(ScopeWork) -> Result<(), FrontendLimit>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.ledger
            .with_phase(self.scope_phases[index], |phase| {
                let mut refusal = None;
                let result = run(&mut |work| {
                    if let Some(error) = refusal {
                        return Err(error);
                    }
                    let (origin, units) = scope_charge(work);
                    match phase.apply(
                        self.scope_origins[index][origin],
                        Change::Work(SCOPE_WORK, units),
                    ) {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(_)) => {
                            let error = FrontendLimit::Scope(self.scope_work_limit);
                            refusal = Some(error);
                            Err(error)
                        }
                        Err(_) => unreachable!("scope work uses preallocated origins"),
                    }
                });
                refusal.map_or(result, |error| Err(RuntimeLimitError::from(error).into()))
            })
            .expect("the ledger owns its scope work phase")
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

    fn charge_job_action(&mut self, action: JobAction) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let (work, refusal) = match action {
            JobAction::Preparation => (
                JOB_PREPARATION,
                RuntimeLimitError::JobPreparation {
                    limit: self.job_preparation_limit,
                },
            ),
            JobAction::Invocation => (
                JOB_INVOCATION,
                RuntimeLimitError::JobInvocation {
                    limit: self.job_invocation_limit,
                },
            ),
            JobAction::Poll => (
                JOB_POLL,
                RuntimeLimitError::JobPoll {
                    limit: self.job_poll_limit,
                },
            ),
        };
        let scope = self.execution_phases[match action {
            JobAction::Preparation => 3,
            JobAction::Invocation => 4,
            JobAction::Poll => 5,
        }];
        match self.ledger.apply(scope, Change::Work(work, 1)) {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(refusal.into()),
            Err(_) => unreachable!("job admission uses its preallocated action scope"),
        }
    }

    /// Records an actual accepted execution event; nested work cannot clear it.
    pub(crate) fn note_effect(&mut self) {
        self.effects_started = true;
    }

    pub(crate) fn retain_stop(&mut self, stop: &JsError) {
        self.stop.get_or_insert_with(|| stop.clone());
    }
}

impl crate::Context {
    /// Admits one engine-owned job action on the currently selected operation.
    pub(crate) fn admit_job_action(&mut self, action: JobAction) -> JsResult<()> {
        if let Some(execution) = &mut self.execution
            && let Err(stop) = execution.charge_job_action(action)
        {
            execution.retain_stop(&stop);
            return Err(stop);
        }
        Ok(())
    }

    /// Called immediately before an accepted callback or declaration mutation.
    pub(crate) fn note_execution_effect(&mut self) {
        if let Some(execution) = &mut self.execution {
            execution.note_effect();
        }
    }
}

#[cfg(test)]
mod compiler_admission_tests {
    use super::*;
    use crate::JsValue;
    use crate::error::{EngineResourceError, JsNativeError};

    #[test]
    fn compiler_work_refusal_preserves_accepted_work_and_the_first_stop() {
        for profile in [
            ProfileMode::Off,
            ProfileMode::Summary { timing: false },
            ProfileMode::Detailed {
                timing: false,
                event_limit: 4,
                filter: EventFilter::All,
            },
        ] {
            let mut execution = Execution::new(
                ExecutionOptions::new(32)
                    .with_compiler_work_limit(2)
                    .with_profile(profile),
            )
            .unwrap();
            let result: JsResult<()> = execution.with_compilation_control(|mut control| {
                control.admit(CompilerWork::Visit)?;
                control.admit(CompilerWork::EmitBytes { bytes: 1 })?;
                let refused = control.admit(CompilerWork::PatchBytes { bytes: 1 });
                assert_eq!(
                    refused.unwrap_err().as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::CompilerWork { limit: 2 }
                    ))
                );
                assert_eq!(
                    control
                        .admit(CompilerWork::Depth { depth: 1 })
                        .unwrap_err()
                        .as_engine(),
                    Some(&EngineError::RuntimeLimit(
                        RuntimeLimitError::CompilerWork { limit: 2 }
                    ))
                );
                // A caught producer refusal cannot be replaced by a later error.
                Err(JsNativeError::syntax()
                    .with_message("later native error")
                    .into())
            });
            assert_eq!(
                result.unwrap_err().as_engine(),
                execution.stop().unwrap().as_engine()
            );
            let outcome = execution.finish(Ok(JsValue::new(42)));
            assert_eq!(
                outcome.result.unwrap_err().as_engine(),
                Some(&EngineError::RuntimeLimit(
                    RuntimeLimitError::CompilerWork { limit: 2 }
                ))
            );
            assert_eq!(outcome.report.compiler_work(), 2);
            assert_eq!(outcome.report.scope_work(), 0);
            assert_eq!(outcome.report.optimizer_work(), 0);
        }
    }

    #[test]
    fn compiler_depth_refusal_reports_only_admitted_entries_on_failure() {
        let mut execution =
            Execution::new(ExecutionOptions::new(32).with_compiler_depth_limit(2)).unwrap();
        let error = execution
            .with_compilation_control(|mut control| {
                let _outer = control.enter()?;
                let mut nested = control.reborrow();
                let _inner = nested.enter()?;
                nested.enter().map(|_| ())
            })
            .unwrap_err();
        assert_eq!(
            error.as_engine(),
            Some(&EngineError::RuntimeLimit(
                RuntimeLimitError::CompilerDepth { limit: 2 }
            ))
        );
        let outcome = execution.finish(Ok(JsValue::new(42)));
        assert!(outcome.result.is_err());
        assert_eq!(outcome.report.compiler_depth_peak(), 2);
        assert_eq!(outcome.report.compiler_work(), 0);
    }

    #[test]
    fn compiler_delegation_preserves_dimensions_and_charges_payloads_once() {
        let mut execution =
            Execution::new(ExecutionOptions::new(32).with_compiler_work_limit(12)).unwrap();
        execution
            .with_compilation_control(|mut control| {
                control.with_scope_admission(|admission| admission.unwrap()(ScopeWork::Node))?;
                control.with_name_admission(|admission| {
                    let admission = admission.unwrap();
                    admission(NameWork::Visit)?;
                    admission(NameWork::Record)?;
                    admission(NameWork::Comparison)
                })?;
                control.admit(CompilerWork::String(StringWork::Copy { units: 3 }))?;
                control.admit(CompilerWork::Primitive(PrimitiveWork::String(
                    StringWork::Scan { units: 2 },
                )))?;
                control.admit(CompilerWork::Primitive(PrimitiveWork::BigIntRead {
                    words: 2,
                }))?;
                control.admit(CompilerWork::Primitive(PrimitiveWork::BigIntCapacity {
                    bytes: 4,
                }))?;
                control.admit(CompilerWork::EmitBytes { bytes: 1 })
            })
            .unwrap();
        let report = execution.finish(Ok(JsValue::new(42))).report;
        assert_eq!(report.compiler_work(), 12);
        assert_eq!(report.scope_work(), 1);
        assert_eq!(report.name_visits(), 1);
        assert_eq!(report.name_records(), 1);
        assert_eq!(report.name_comparisons(), 1);
        assert_eq!(report.optimizer_work(), 0);
    }

    #[test]
    fn compiler_delegated_limits_keep_their_typed_errors() {
        for (options, work, expected) in [
            (
                ExecutionOptions::new(32).with_scope_work_limit(0),
                CompilerWork::Scope(ScopeWork::Node),
                RuntimeLimitError::ScopeWork { limit: 0 },
            ),
            (
                ExecutionOptions::new(32).with_name_visit_limit(0),
                CompilerWork::Name(NameWork::Visit),
                RuntimeLimitError::NameVisit { limit: 0 },
            ),
            (
                ExecutionOptions::new(32).with_name_record_limit(0),
                CompilerWork::Name(NameWork::Record),
                RuntimeLimitError::NameRecord { limit: 0 },
            ),
            (
                ExecutionOptions::new(32).with_name_comparison_limit(0),
                CompilerWork::Name(NameWork::Comparison),
                RuntimeLimitError::NameComparison { limit: 0 },
            ),
        ] {
            let mut execution = Execution::new(options).unwrap();
            let error = execution
                .with_compilation_control(|mut control| control.admit(work))
                .unwrap_err();
            assert_eq!(
                error.as_engine(),
                Some(&EngineError::RuntimeLimit(expected))
            );
            assert_eq!(execution.stop().unwrap().as_engine(), error.as_engine());
            assert_eq!(
                execution
                    .finish(Ok(JsValue::new(42)))
                    .report
                    .compiler_work(),
                0
            );
        }
    }

    #[test]
    fn compiler_resource_failure_is_latched_before_returning_the_phase() {
        let mut execution = Execution::new(ExecutionOptions::new(32)).unwrap();
        let error = JsError::from(EngineError::Resource(EngineResourceError::SizeOverflow));
        let result: JsResult<()> = execution.with_compilation_control(|_| Err(error.clone()));
        assert_eq!(result.unwrap_err().as_engine(), error.as_engine());
        assert_eq!(execution.stop().unwrap().as_engine(), error.as_engine());
        assert_eq!(
            execution
                .finish(Ok(JsValue::new(42)))
                .result
                .unwrap_err()
                .as_engine(),
            error.as_engine()
        );
    }

    #[test]
    fn compiler_scope_error_conversion_preserves_each_error_domain() {
        use surgeist_js_ast::scope::{ScopeAnalysisError, ScopeStateError};
        let refusal = JsError::from(RuntimeLimitError::CompilerWork { limit: 7 });
        let converted = JsError::from(ScopeAnalysisError::Refused(refusal.clone()));
        assert_eq!(converted.as_engine(), refusal.as_engine());
        let syntax = JsError::from(ScopeAnalysisError::<JsError>::Syntax(
            "invalid binding".into(),
        ));
        let syntax = syntax.as_native().unwrap();
        assert!(syntax.is_syntax());
        assert_eq!(syntax.message(), "invalid binding");
        let state = JsError::from(ScopeAnalysisError::<JsError>::State(
            ScopeStateError::Conflict,
        ));
        assert_eq!(
            state.as_engine(),
            Some(&EngineError::ScopeAnalysis(ScopeStateError::Conflict))
        );
    }
}
