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
//! Pending. `job-claim` observes finite snapshot preparation and claim commit,
//! closing before job dispatch or waiting. `generator-request` observes each
//! admitted request algorithm through its synchronous return and cleanup, including promise preparation, without
//! spanning suspension. Host future internals are opaque; these timings do not certify their work.

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
    Change, Defect, Error as LedgerError, Limits, NoDimension, NoTable, Operation, Phase, Scope,
    Tariffs, dimensions,
};
use thiserror::Error;

use crate::{
    JsError, JsResult,
    bytecompiler::control::{CompilerControl, CompilerDepth, CompilerWork},
    error::{EngineError, EngineResourceError, RuntimeLimitError},
    optimizer::control::OptimizerWork,
    value::primitive::PrimitiveWork,
};

pub use surgeist_ledger::{EventFilter, ProfileMode};

dimensions! {
    /// Work units the execution ledger admits, in profile order. Each variant has
    /// its own limit in [`ExecutionOptions`]. Values of different variants are not additive.
    pub enum ExecutionWork => ExecutionWorkTable {
        /// One VM instruction admitted before dispatch, including instructions reached through synchronous reentry.
        Instruction,
        /// One input read attempt across source decoders, including EOF probes.
        SourceRead,
        /// One decoded code point consumed across frontend phases, including trivia.
        SourceConsumption,
        /// One UTF-16 source unit retained across frontend phases.
        SourceRetention,
        /// One bounded lexer lookahead probe, including cache hits and EOF probes.
        LexerLookahead,
        /// One lexical production dispatch, including comments and continuations.
        LexerDispatch,
        /// One buffered token probe, including cache hits and EOF.
        TokenProbe,
        /// One buffered token advance request, including requests at EOF.
        TokenAdvance,
        /// One grammar production attempt, including inline repeated items.
        GrammarDispatch,
        /// One AST visit in a metered early-error walk.
        EarlyErrorVisit,
        /// One logical label-table lookup.
        LabelLookup,
        /// One AST visit that computes cached direct-eval metadata.
        DirectEvalVisit,
        /// One AST visit in name collection, validation and delegated semantic queries.
        NameVisit,
        /// One name record attempt.
        NameRecord,
        /// One symbol comparison or logical name-table operation.
        NameComparison,
        /// One UTF-8 or Latin1 byte of frontend interning work.
        InternerBytes,
        /// One UTF-16 unit of frontend interning and name preparation work.
        InternerUtf16,
        /// One frontend interner slot initialization, probe, reinsertion or publication.
        InternerIndex,
        /// One unit of additional capacity requested by a frontend interner buffer.
        InternerCapacity,
        /// One initialized unit that a frontend interner reserve may relocate.
        InternerRelocation,
        /// One UTF-16 spelling unit inspected while rendering a frontend diagnostic.
        DiagnosticInput,
        /// One UTF-8 byte appended to an owned source-spelling diagnostic payload.
        DiagnosticOutput,
        /// One UTF-16 unit inspected by raw and cooked template preparation.
        TemplateInput,
        /// One UTF-16 unit appended to a normalized raw or cooked template.
        TemplateOutput,
        /// One unit of script scope analysis work, including staged changes and publication.
        ScopeWork,
        /// One unit of optimizer traversal, rewrite preparation and delegated operand work.
        OptimizerWork,
        /// One unit of compiler lowering and delegated payload work.
        CompilerWork,
        /// One native task preparation factory invocation.
        JobPreparation,
        /// One transfer of control to a synchronous native job callback.
        JobInvocation,
        /// One native asynchronous job future poll, including Pending polls.
        JobPoll,
        /// One generator `next`, `return` or `throw` request attempt.
        GeneratorRequest,
        /// One logical credit of module-owned metadata construction work.
        ModuleMetadataWork,
        /// One logical credit of finite executor batch snapshot and claim work.
        JobClaimWork,
    }
}
const JOB_CLAIM_ORIGINS: [&str; 6] = [
    "claim-bucket",
    "claim-node",
    "claim-snapshot-copy",
    "claim-capacity",
    "claim-relocate",
    "claim-commit",
];

#[derive(Clone, Copy)]
pub(crate) enum JobClaimWork {
    Bucket,
    Node,
    SnapshotCopy,
    Capacity(usize),
    Relocate(usize),
    Commit(usize),
}

impl JobClaimWork {
    const fn origin_units(self) -> (usize, u64) {
        match self {
            Self::Bucket => (0, 1),
            Self::Node => (1, 1),
            Self::SnapshotCopy => (2, 1),
            Self::Capacity(bytes) => (3, bytes as u64),
            Self::Relocate(bytes) => (4, bytes as u64),
            Self::Commit(count) => (5, count as u64),
        }
    }
}
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
// + 21 origins, 32 compiler origins, three lifetime/VM scopes, three job scopes
// and one generator scope, plus metadata/claim phases and their producer origins.
const FIXED_SCOPES: usize = 1
    + 1
    + 4
    + 3
    + 3 * SCOPE_ORIGINS.len()
    + 1
    + OPTIMIZER_ORIGINS.len()
    + COMPILER_ORIGINS.len()
    + 9
    + crate::module::metadata::ORIGINS.len()
    + JOB_CLAIM_ORIGINS.len();
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
type ExecutionLedger = Operation<ExecutionWork, NoDimension>;

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

fn compiler_charge(work: CompilerWork) -> (usize, ExecutionWork, u64) {
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
                NameWork::Visit => (27, ExecutionWork::NameVisit, 1),
                NameWork::Record => (28, ExecutionWork::NameRecord, 1),
                NameWork::Comparison => (29, ExecutionWork::NameComparison, 1),
            };
        }
        CompilerWork::Scope(_) => unreachable!("scope work retains its existing phase origins"),
    };
    (origin, ExecutionWork::CompilerWork, units)
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

/// Reports a ledger failure that the prepared scope tree rules out. A refusal
/// is handled by the caller before this.
fn ledger_fault(error: LedgerError<ExecutionWork, NoDimension>) -> EngineResourceError {
    match error {
        LedgerError::Defect(defect) => EngineResourceError::LedgerDefect(defect),
        LedgerError::Allocation
        | LedgerError::IdentityExhausted
        | LedgerError::ForeignScope
        | LedgerError::ForeignCheckpoint
        | LedgerError::StaleCheckpoint
        | LedgerError::CheckpointLimit { .. }
        | LedgerError::CheckpointsConfigured
        | LedgerError::ScopeLimit { .. }
        | LedgerError::TimingUnavailable
        | LedgerError::Refused(_) => EngineResourceError::LedgerInvariant,
    }
}

fn ledger_error(error: LedgerError<ExecutionWork, NoDimension>) -> JsError {
    ledger_fault(error).into()
}

fn charge_interner(
    phase: &mut Phase<'_, ExecutionWork, NoDimension>,
    scope: Scope,
    work: InternerWork,
) -> Result<(), LedgerError<ExecutionWork, NoDimension>> {
    match work {
        InternerWork::Hash { encoding, units }
        | InternerWork::Compare { encoding, units }
        | InternerWork::Convert { encoding, units } => {
            let dimension = match encoding {
                InternerEncoding::Utf8 | InternerEncoding::Latin1 => ExecutionWork::InternerBytes,
                InternerEncoding::Utf16 => ExecutionWork::InternerUtf16,
            };
            phase.apply(scope, Change::Work(dimension, units as u64))
        }
        InternerWork::Classify { units } => phase.apply(
            scope,
            Change::Work(ExecutionWork::InternerUtf16, units as u64),
        ),
        InternerWork::ValidateStatic {
            utf8_bytes,
            utf16_units,
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(ExecutionWork::InternerBytes, utf8_bytes as u64),
                Change::Work(ExecutionWork::InternerUtf16, utf16_units as u64),
            ],
        ),
        InternerWork::Probe | InternerWork::Reinsert => {
            phase.apply(scope, Change::Work(ExecutionWork::InternerIndex, 1))
        }
        InternerWork::InitializeSlots { slots } => phase.apply(
            scope,
            Change::Work(ExecutionWork::InternerIndex, slots as u64),
        ),
        InternerWork::Reserve {
            additional,
            existing,
            ..
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(ExecutionWork::InternerCapacity, additional as u64),
                Change::Work(ExecutionWork::InternerRelocation, existing as u64),
            ],
        ),
        InternerWork::Publish {
            utf8_bytes,
            utf16_units,
        } => phase.apply_batch(
            scope,
            &[
                Change::Work(ExecutionWork::InternerIndex, 1),
                Change::Work(ExecutionWork::InternerBytes, utf8_bytes as u64),
                Change::Work(ExecutionWork::InternerUtf16, utf16_units as u64),
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
    generator_request_limit: u64,
    module_metadata_work_limit: u64,
    job_claim_work_limit: u64,
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
            generator_request_limit: u64::MAX,
            module_metadata_work_limit: u64::MAX,
            job_claim_work_limit: u64::MAX,
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

    /// Caps finite executor batch snapshot and claim work.
    ///
    /// Credits cover selected bucket and linked-node inspections, snapshot
    /// copies, logical scratch capacity and relocation, and final claim marks.
    /// Discarding scratch or returning claims to the queue does not refund work.
    /// This does not bound the whole drain, clock-tree searches, publication,
    /// cancellation, live resources or arbitrary custom executor work.
    #[must_use]
    pub const fn with_job_claim_work_limit(mut self, limit: u64) -> Self {
        self.job_claim_work_limit = limit;
        self
    }

    /// Caps attempted synchronous and asynchronous generator requests.
    ///
    /// One unit admits one `next`, `return` or `throw` request before receiver
    /// validation or promise preparation, including completed generators and
    /// requests that subsequently fail validation. Refusal preserves the
    /// generator's state and queue. VM instructions and native job actions
    /// retain their separate limits.
    /// This cap does not bound continuation payload copies, queue or drain work,
    /// or completion publication.
    #[must_use]
    pub const fn with_generator_request_limit(mut self, limit: u64) -> Self {
        self.generator_request_limit = limit;
        self
    }

    /// Caps module-owned metadata construction work.
    ///
    /// Logical credits cover visits, probes, records, writes, requested capacity,
    /// copies, possible relocation, String work and final publication. Delegated
    /// AST name work retains its separate name caps. These are not physical
    /// memory, GC, arbitrary host iterator/capture work, or module loading,
    /// linking or evaluation work.
    #[must_use]
    pub const fn with_module_metadata_work_limit(mut self, limit: u64) -> Self {
        self.module_metadata_work_limit = limit;
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
    /// The execution ledger found a broken internal invariant during setup.
    #[error("execution ledger defect: {0}")]
    LedgerDefect(Defect),
    /// The execution ledger rejected a setup call that its fixed scope tree rules out.
    #[error("execution ledger rejected a setup call its fixed scope tree rules out")]
    LedgerInvariant,
}

#[derive(Clone, Copy)]
pub(crate) enum ExecutionPhase {
    Lifetime,
    VmActive,
    VmWaiting,
    JobPreparation,
    JobInvocation,
    JobPoll,
    GeneratorRequest,
    ModuleMetadata,
    JobClaim,
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
    execution_phases: [Scope; 9],
    claim_origins: [Scope; JOB_CLAIM_ORIGINS.len()],
    metadata_origins: [Scope; crate::module::metadata::ORIGINS.len()],
    instruction_limit: u64,
    job_preparation_limit: u64,
    job_invocation_limit: u64,
    job_poll_limit: u64,
    generator_request_limit: u64,
    module_metadata_work_limit: u64,
    job_claim_work_limit: u64,
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
            ExecutionWorkTable {
                Instruction: options.instruction_limit,
                SourceRead: options.source_read_limit,
                SourceConsumption: options.source_consumption_limit,
                SourceRetention: options.source_retention_limit,
                LexerLookahead: options.lexer_lookahead_limit,
                LexerDispatch: options.lexer_dispatch_limit,
                TokenProbe: options.token_probe_limit,
                TokenAdvance: options.token_advance_limit,
                GrammarDispatch: options.grammar_dispatch_limit,
                EarlyErrorVisit: options.early_error_visit_limit,
                LabelLookup: options.label_lookup_limit,
                DirectEvalVisit: options.direct_eval_visit_limit,
                NameVisit: options.name_visit_limit,
                NameRecord: options.name_record_limit,
                NameComparison: options.name_comparison_limit,
                InternerBytes: options.interner_byte_limit,
                InternerUtf16: options.interner_utf16_limit,
                InternerIndex: options.interner_index_limit,
                InternerCapacity: options.interner_capacity_limit,
                InternerRelocation: options.interner_relocation_limit,
                DiagnosticInput: options.diagnostic_input_limit,
                DiagnosticOutput: options.diagnostic_output_limit,
                TemplateInput: options.template_input_limit,
                TemplateOutput: options.template_output_limit,
                ScopeWork: options.scope_work_limit,
                OptimizerWork: options.optimizer_work_limit,
                CompilerWork: options.compiler_work_limit,
                JobPreparation: options.job_preparation_limit,
                JobInvocation: options.job_invocation_limit,
                JobPoll: options.job_poll_limit,
                GeneratorRequest: options.generator_request_limit,
                ModuleMetadataWork: options.module_metadata_work_limit,
                JobClaimWork: options.job_claim_work_limit,
            },
            NoTable::EMPTY,
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
        let mut execution_phases = [ledger.root(); 9];
        for (slot, name) in execution_phases.iter_mut().zip([
            "execution-lifetime",
            "vm-active",
            "vm-waiting",
            "job-prepare",
            "job-invoke",
            "job-poll",
            "generator-request",
            "module-metadata",
            "job-claim",
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
        let mut metadata_origins = [execution_phases[7]; crate::module::metadata::ORIGINS.len()];
        for (slot, name) in metadata_origins
            .iter_mut()
            .zip(crate::module::metadata::ORIGINS)
        {
            *slot = ledger
                .child(
                    execution_phases[7],
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
        }
        let mut claim_origins = [execution_phases[8]; JOB_CLAIM_ORIGINS.len()];
        for (slot, name) in claim_origins.iter_mut().zip(JOB_CLAIM_ORIGINS) {
            *slot = ledger
                .child(
                    execution_phases[8],
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
        }
        Ok(Self {
            ledger,
            claim_origins,
            metadata_origins,
            prepared_report: PreparedReport::new(),
            execution_phases,
            instruction_limit: options.instruction_limit,
            job_preparation_limit: options.job_preparation_limit,
            job_invocation_limit: options.job_invocation_limit,
            job_poll_limit: options.job_poll_limit,
            generator_request_limit: options.generator_request_limit,
            module_metadata_work_limit: options.module_metadata_work_limit,
            job_claim_work_limit: options.job_claim_work_limit,
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

    fn setup_error(error: LedgerError<ExecutionWork, NoDimension>) -> ExecutionSetupError {
        match error {
            LedgerError::Allocation => ExecutionSetupError::Allocation,
            LedgerError::IdentityExhausted => ExecutionSetupError::IdentityExhausted,
            LedgerError::TimingUnavailable => ExecutionSetupError::TimingUnavailable,
            LedgerError::Defect(defect) => ExecutionSetupError::LedgerDefect(defect),
            LedgerError::ScopeLimit { .. }
            | LedgerError::ForeignScope
            | LedgerError::ForeignCheckpoint
            | LedgerError::StaleCheckpoint
            | LedgerError::CheckpointLimit { .. }
            | LedgerError::CheckpointsConfigured
            | LedgerError::Refused(_) => ExecutionSetupError::LedgerInvariant,
        }
    }

    pub(crate) fn with_frontend_control<T>(
        &mut self,
        parse: impl FnOnce(&mut FrontendControl<'_>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.ledger
            .with_phase(self.frontend, |phase| {
                let mut refusal = None;
                let mut fault = None;
                let result = parse(&mut |work| {
                    if let Some(refusal) = refusal {
                        return Err(refusal);
                    }
                    let admission = match work {
                        FrontendWork::Scope(work) => {
                            let (origin, units) = scope_charge(work);
                            phase.apply(
                                self.scope_origins[0][origin],
                                Change::Work(ExecutionWork::ScopeWork, units),
                            )
                        }
                        FrontendWork::Read(unit) => {
                            let scope = match unit {
                                ReadUnit::Utf8Byte => self.utf8,
                                ReadUnit::Utf16Unit => self.utf16,
                                ReadUnit::HostRead => self.host,
                                ReadUnit::Latin1Byte => self.latin1,
                            };
                            phase.apply(scope, Change::Work(ExecutionWork::SourceRead, 1))
                        }
                        FrontendWork::Lookahead => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::LexerLookahead, 1),
                        ),
                        FrontendWork::LexicalDispatch => phase
                            .apply(self.frontend, Change::Work(ExecutionWork::LexerDispatch, 1)),
                        FrontendWork::TokenProbe => {
                            phase.apply(self.frontend, Change::Work(ExecutionWork::TokenProbe, 1))
                        }
                        FrontendWork::TokenAdvance => {
                            phase.apply(self.frontend, Change::Work(ExecutionWork::TokenAdvance, 1))
                        }
                        FrontendWork::GrammarDispatch => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::GrammarDispatch, 1),
                        ),
                        FrontendWork::EarlyErrorVisit => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::EarlyErrorVisit, 1),
                        ),
                        FrontendWork::LabelLookup => {
                            phase.apply(self.frontend, Change::Work(ExecutionWork::LabelLookup, 1))
                        }
                        FrontendWork::DirectEvalVisit => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::DirectEvalVisit, 1),
                        ),
                        FrontendWork::NameVisit => {
                            phase.apply(self.frontend, Change::Work(ExecutionWork::NameVisit, 1))
                        }
                        FrontendWork::NameRecord => {
                            phase.apply(self.frontend, Change::Work(ExecutionWork::NameRecord, 1))
                        }
                        FrontendWork::NameComparison => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::NameComparison, 1),
                        ),
                        FrontendWork::Interner(work) => charge_interner(phase, self.frontend, work),
                        FrontendWork::DiagnosticInput => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::DiagnosticInput, 1),
                        ),
                        FrontendWork::DiagnosticOutput(bytes) => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::DiagnosticOutput, bytes as u64),
                        ),
                        FrontendWork::TemplateInput => phase
                            .apply(self.frontend, Change::Work(ExecutionWork::TemplateInput, 1)),
                        FrontendWork::TemplateOutput(units) => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::TemplateOutput, u64::from(units)),
                        ),
                        FrontendWork::InternerNamePreparation(units) => phase.apply(
                            self.frontend,
                            Change::Work(ExecutionWork::InternerUtf16, units as u64),
                        ),
                        FrontendWork::Consume { retained_units } => phase.apply_batch(
                            self.frontend,
                            &[
                                Change::Work(ExecutionWork::SourceConsumption, 1),
                                Change::Work(
                                    ExecutionWork::SourceRetention,
                                    u64::from(retained_units),
                                ),
                            ],
                        ),
                    };
                    match admission {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(denial)) => {
                            let error = match denial.change {
                                Change::Work(ExecutionWork::SourceRead, _) => {
                                    FrontendLimit::Read(self.source_read_limit)
                                }
                                Change::Work(ExecutionWork::SourceConsumption, _) => {
                                    FrontendLimit::Consumption(self.source_consumption_limit)
                                }
                                Change::Work(ExecutionWork::SourceRetention, _) => {
                                    FrontendLimit::Retention(self.source_retention_limit)
                                }
                                Change::Work(ExecutionWork::LexerLookahead, _) => {
                                    FrontendLimit::Lookahead(self.lexer_lookahead_limit)
                                }
                                Change::Work(ExecutionWork::LexerDispatch, _) => {
                                    FrontendLimit::LexicalDispatch(self.lexer_dispatch_limit)
                                }
                                Change::Work(ExecutionWork::TokenProbe, _) => {
                                    FrontendLimit::TokenProbe(self.token_probe_limit)
                                }
                                Change::Work(ExecutionWork::TokenAdvance, _) => {
                                    FrontendLimit::TokenAdvance(self.token_advance_limit)
                                }
                                Change::Work(ExecutionWork::GrammarDispatch, _) => {
                                    FrontendLimit::GrammarDispatch(self.grammar_dispatch_limit)
                                }
                                Change::Work(ExecutionWork::EarlyErrorVisit, _) => {
                                    FrontendLimit::EarlyErrorVisit(self.early_error_visit_limit)
                                }
                                Change::Work(ExecutionWork::LabelLookup, _) => {
                                    FrontendLimit::LabelLookup(self.label_lookup_limit)
                                }
                                Change::Work(ExecutionWork::DirectEvalVisit, _) => {
                                    FrontendLimit::DirectEvalVisit(self.direct_eval_visit_limit)
                                }
                                Change::Work(ExecutionWork::NameVisit, _) => {
                                    FrontendLimit::NameVisit(self.name_visit_limit)
                                }
                                Change::Work(ExecutionWork::NameRecord, _) => {
                                    FrontendLimit::NameRecord(self.name_record_limit)
                                }
                                Change::Work(ExecutionWork::NameComparison, _) => {
                                    FrontendLimit::NameComparison(self.name_comparison_limit)
                                }
                                Change::Work(ExecutionWork::InternerBytes, _) => {
                                    FrontendLimit::Interner {
                                        kind: InternerLimitKind::Bytes,
                                        limit: self.interner_byte_limit,
                                    }
                                }
                                Change::Work(ExecutionWork::InternerUtf16, _) => {
                                    FrontendLimit::Interner {
                                        kind: InternerLimitKind::Utf16,
                                        limit: self.interner_utf16_limit,
                                    }
                                }
                                Change::Work(ExecutionWork::InternerIndex, _) => {
                                    FrontendLimit::Interner {
                                        kind: InternerLimitKind::Index,
                                        limit: self.interner_index_limit,
                                    }
                                }
                                Change::Work(ExecutionWork::InternerCapacity, _) => {
                                    FrontendLimit::Interner {
                                        kind: InternerLimitKind::Capacity,
                                        limit: self.interner_capacity_limit,
                                    }
                                }
                                Change::Work(ExecutionWork::DiagnosticInput, _) => {
                                    FrontendLimit::DiagnosticInput(self.diagnostic_input_limit)
                                }
                                Change::Work(ExecutionWork::DiagnosticOutput, _) => {
                                    FrontendLimit::DiagnosticOutput(self.diagnostic_output_limit)
                                }
                                Change::Work(ExecutionWork::TemplateInput, _) => {
                                    FrontendLimit::TemplateInput(self.template_input_limit)
                                }
                                Change::Work(ExecutionWork::TemplateOutput, _) => {
                                    FrontendLimit::TemplateOutput(self.template_output_limit)
                                }
                                Change::Work(ExecutionWork::ScopeWork, _) => {
                                    FrontendLimit::Scope(self.scope_work_limit)
                                }
                                Change::Work(ExecutionWork::InternerRelocation, _) => {
                                    FrontendLimit::Interner {
                                        kind: InternerLimitKind::Relocation,
                                        limit: self.interner_relocation_limit,
                                    }
                                }
                                _ => {
                                    // No frontend limit names this dimension. The
                                    // stand-in ends the parse and the fault replaces it.
                                    fault = Some(EngineResourceError::LedgerInvariant);
                                    FrontendLimit::Read(self.source_read_limit)
                                }
                            };
                            refusal = Some(error);
                            Err(error)
                        }
                        Err(error) => {
                            // The parser can only stop on a limit. This stand-in
                            // ends the parse and is replaced by the fault below.
                            let stop = FrontendLimit::Read(self.source_read_limit);
                            fault = Some(ledger_fault(error));
                            refusal = Some(stop);
                            Err(stop)
                        }
                    }
                });
                match fault {
                    Some(fault) => Err(fault.into()),
                    None => {
                        refusal.map_or(result, |error| Err(RuntimeLimitError::from(error).into()))
                    }
                }
            })
            .map_err(ledger_error)?
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
                            Change::Work(ExecutionWork::OptimizerWork, units),
                        )
                    };
                    match admission {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(denial)) => {
                            let limit = match denial.change {
                                Change::Work(ExecutionWork::InternerBytes, _) => {
                                    Some(RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Bytes,
                                        limit: self.interner_byte_limit,
                                    })
                                }
                                Change::Work(ExecutionWork::InternerUtf16, _) => {
                                    Some(RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Utf16,
                                        limit: self.interner_utf16_limit,
                                    })
                                }
                                Change::Work(ExecutionWork::InternerIndex, _) => {
                                    Some(RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Index,
                                        limit: self.interner_index_limit,
                                    })
                                }
                                Change::Work(ExecutionWork::InternerCapacity, _) => {
                                    Some(RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Capacity,
                                        limit: self.interner_capacity_limit,
                                    })
                                }
                                Change::Work(ExecutionWork::InternerRelocation, _) => {
                                    Some(RuntimeLimitError::Interner {
                                        kind: InternerLimitKind::Relocation,
                                        limit: self.interner_relocation_limit,
                                    })
                                }
                                Change::Work(ExecutionWork::OptimizerWork, _) => {
                                    Some(RuntimeLimitError::OptimizerWork {
                                        limit: self.optimizer_work_limit,
                                    })
                                }
                                _ => None,
                            };
                            let error = limit.map_or_else(
                                || JsError::from(EngineResourceError::LedgerInvariant),
                                JsError::from,
                            );
                            refusal = Some(error.clone());
                            Err(error)
                        }
                        Err(error) => {
                            let error = ledger_error(error);
                            refusal = Some(error.clone());
                            Err(error)
                        }
                    }
                });
                refusal.map_or(result, Err)
            })
            .map_err(ledger_error)?
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
                        (self.scope_origins[1][origin], ExecutionWork::ScopeWork, units)
                    } else {
                        let (origin, dimension, units) = compiler_charge(work);
                        (self.compiler_origins[origin], dimension, units)
                    };
                    match phase.apply(scope, Change::Work(dimension, units)) {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(_)) => {
                            let limit = match dimension {
                                ExecutionWork::CompilerWork => Some(RuntimeLimitError::CompilerWork {
                                    limit: self.compiler_work_limit,
                                }),
                                ExecutionWork::ScopeWork => Some(RuntimeLimitError::ScopeWork {
                                    limit: self.scope_work_limit,
                                }),
                                ExecutionWork::NameVisit => Some(RuntimeLimitError::NameVisit {
                                    limit: self.name_visit_limit,
                                }),
                                ExecutionWork::NameRecord => Some(RuntimeLimitError::NameRecord {
                                    limit: self.name_record_limit,
                                }),
                                ExecutionWork::NameComparison => Some(RuntimeLimitError::NameComparison {
                                    limit: self.name_comparison_limit,
                                }),
                                _ => None,
                            };
                            let error = limit.map_or_else(
                                || JsError::from(EngineResourceError::LedgerInvariant),
                                JsError::from,
                            );
                            // Latch before returning to the producer or closing
                            // the timed phase. Later errors cannot replace it.
                            *stop = Some(error.clone());
                            Err(error)
                        }
                        Err(error) => {
                            let error = ledger_error(error);
                            *stop = Some(error.clone());
                            Err(error)
                        }
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
            .map_err(ledger_error)
            .and_then(|result| result);
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
                let mut fault = None;
                let result = run(&mut |work| {
                    if let Some(error) = refusal {
                        return Err(error);
                    }
                    let (origin, units) = scope_charge(work);
                    match phase.apply(
                        self.scope_origins[index][origin],
                        Change::Work(ExecutionWork::ScopeWork, units),
                    ) {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(_)) => {
                            let error = FrontendLimit::Scope(self.scope_work_limit);
                            refusal = Some(error);
                            Err(error)
                        }
                        Err(error) => {
                            // The caller can only stop on a limit. This stand-in
                            // ends the analysis and is replaced by the fault below.
                            let stop = FrontendLimit::Scope(self.scope_work_limit);
                            fault = Some(ledger_fault(error));
                            refusal = Some(stop);
                            Err(stop)
                        }
                    }
                });
                match fault {
                    Some(fault) => Err(fault.into()),
                    None => {
                        refusal.map_or(result, |error| Err(RuntimeLimitError::from(error).into()))
                    }
                }
            })
            .map_err(ledger_error)?
    }

    pub(crate) const fn stop(&self) -> Option<&JsError> {
        self.stop.as_ref()
    }

    pub(crate) fn charge_instruction(&mut self) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        match self.ledger.apply(
            self.ledger.root(),
            Change::Work(ExecutionWork::Instruction, 1),
        ) {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(RuntimeLimitError::Instruction {
                limit: self.instruction_limit,
            }
            .into()),
            Err(error) => Err(ledger_error(error)),
        }
    }

    fn charge_job_action(&mut self, action: JobAction) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let (work, refusal) = match action {
            JobAction::Preparation => (
                ExecutionWork::JobPreparation,
                RuntimeLimitError::JobPreparation {
                    limit: self.job_preparation_limit,
                },
            ),
            JobAction::Invocation => (
                ExecutionWork::JobInvocation,
                RuntimeLimitError::JobInvocation {
                    limit: self.job_invocation_limit,
                },
            ),
            JobAction::Poll => (
                ExecutionWork::JobPoll,
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
            Err(error) => Err(ledger_error(error)),
        }
    }

    fn charge_job_claim(&mut self, work: JobClaimWork) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let (origin, units) = work.origin_units();
        match self.ledger.apply(
            self.claim_origins[origin],
            Change::Work(ExecutionWork::JobClaimWork, units),
        ) {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(RuntimeLimitError::JobClaimWork {
                limit: self.job_claim_work_limit,
            }
            .into()),
            Err(error) => Err(ledger_error(error)),
        }
    }

    fn charge_generator_request(&mut self) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        match self.ledger.apply(
            self.execution_phases[6],
            Change::Work(ExecutionWork::GeneratorRequest, 1),
        ) {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(RuntimeLimitError::GeneratorRequest {
                limit: self.generator_request_limit,
            }
            .into()),
            Err(error) => Err(ledger_error(error)),
        }
    }

    fn charge_module_metadata(&mut self, work: crate::module::metadata::Work) -> JsResult<()> {
        if let Some(stop) = &self.stop {
            return Err(stop.clone());
        }
        let Some((origin, units)) = work.origin_units() else {
            return Ok(());
        };
        let (dimension, limit) = match work {
            crate::module::metadata::Work::Name(NameWork::Visit) => (
                ExecutionWork::NameVisit,
                RuntimeLimitError::NameVisit {
                    limit: self.name_visit_limit,
                },
            ),
            crate::module::metadata::Work::Name(NameWork::Record) => (
                ExecutionWork::NameRecord,
                RuntimeLimitError::NameRecord {
                    limit: self.name_record_limit,
                },
            ),
            crate::module::metadata::Work::Name(NameWork::Comparison) => (
                ExecutionWork::NameComparison,
                RuntimeLimitError::NameComparison {
                    limit: self.name_comparison_limit,
                },
            ),
            _ => (
                ExecutionWork::ModuleMetadataWork,
                RuntimeLimitError::ModuleMetadataWork {
                    limit: self.module_metadata_work_limit,
                },
            ),
        };
        match self.ledger.apply(
            self.metadata_origins[origin],
            Change::Work(dimension, units),
        ) {
            Ok(()) => Ok(()),
            Err(LedgerError::Refused(_)) => Err(JsError::from(limit)),
            Err(error) => Err(ledger_error(error)),
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

    pub(crate) fn admit_job_claim(&mut self, work: JobClaimWork) -> JsResult<()> {
        self.check_execution_stop()?;
        if let Some(execution) = &mut self.execution
            && let Err(stop) = execution.charge_job_claim(work)
        {
            execution.retain_stop(&stop);
            return Err(stop);
        }
        Ok(())
    }

    /// Admits one attempted request before validation or promise preparation.
    pub(crate) fn with_generator_request<T>(
        &mut self,
        run: impl FnOnce(&mut Self) -> JsResult<T>,
    ) -> JsResult<T> {
        self.check_execution_stop()?;
        if let Some(execution) = &mut self.execution
            && let Err(stop) = execution.charge_generator_request()
        {
            execution.retain_stop(&stop);
            return Err(stop);
        }
        let result = self.with_execution_phase(ExecutionPhase::GeneratorRequest, |context| {
            context.note_execution_effect();
            run(context)
        });
        self.check_execution_stop().and(result)
    }

    pub(crate) fn admit_module_metadata(
        &mut self,
        work: crate::module::metadata::Work,
    ) -> JsResult<()> {
        self.check_execution_stop()?;
        let result = self
            .execution
            .as_mut()
            .map_or(Ok(()), |execution| execution.charge_module_metadata(work));
        result.map_err(|mut error| {
            self.retain_preparation_error(&mut error);
            error
        })
    }

    pub(crate) fn with_module_metadata<T>(
        &mut self,
        run: impl FnOnce(&mut Self) -> JsResult<T>,
    ) -> JsResult<T> {
        self.check_execution_stop()?;
        let result = self.with_execution_phase(ExecutionPhase::ModuleMetadata, |context| {
            let mut result = run(context);
            // Latch producer/resource failures before the phase's clock cleanup.
            if let Err(error) = &mut result {
                context.retain_preparation_error(error);
            }
            result
        });
        self.check_execution_stop().and(result)
    }

    /// Called immediately before an accepted callback, generator request or declaration mutation.
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
    fn ledger_defect_is_reported_as_a_typed_failure_at_setup_and_runtime() {
        let defect = Defect::ReservedCapacity;
        assert_eq!(
            Execution::setup_error(LedgerError::Defect(defect)),
            ExecutionSetupError::LedgerDefect(defect)
        );
        assert_eq!(
            ledger_fault(LedgerError::Defect(defect)),
            EngineResourceError::LedgerDefect(defect)
        );
        assert_eq!(
            ledger_error(LedgerError::Defect(defect)).as_engine(),
            Some(&EngineError::Resource(EngineResourceError::LedgerDefect(
                defect
            )))
        );
    }

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
            assert_eq!(outcome.report.compiler_work().unwrap(), 2);
            assert_eq!(outcome.report.scope_work().unwrap(), 0);
            assert_eq!(outcome.report.optimizer_work().unwrap(), 0);
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
        assert_eq!(outcome.report.compiler_work().unwrap(), 0);
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
        assert_eq!(report.compiler_work().unwrap(), 12);
        assert_eq!(report.scope_work().unwrap(), 1);
        assert_eq!(report.name_visits().unwrap(), 1);
        assert_eq!(report.name_records().unwrap(), 1);
        assert_eq!(report.name_comparisons().unwrap(), 1);
        assert_eq!(report.optimizer_work().unwrap(), 0);
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
                    .compiler_work()
                    .unwrap(),
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
