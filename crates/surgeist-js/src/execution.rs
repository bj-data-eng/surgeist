//! Work limits and profiling for controlled JavaScript execution.

use std::time::Duration;

use surgeist_js_ast::scope::ScopeWork;
use surgeist_js_interner::{InternerEncoding, InternerWork};
use surgeist_js_parser::{
    frontend::{FrontendControl, FrontendLimit, FrontendWork, InternerLimitKind},
    source::ReadUnit,
};
use surgeist_ledger::{
    Change, Error as LedgerError, Limits, Operation, Phase, Report, Scope, Tariffs, Work,
};
use thiserror::Error;

use crate::{JsError, JsResult, JsValue, error::RuntimeLimitError};

pub use surgeist_ledger::{EventFilter, ProfileMode};

const INSTRUCTION: Work<25> = Work::new(0).unwrap();
const SOURCE_READ: Work<25> = Work::new(1).unwrap();
const SOURCE_CONSUMPTION: Work<25> = Work::new(2).unwrap();
const SOURCE_RETENTION: Work<25> = Work::new(3).unwrap();
const LEXER_LOOKAHEAD: Work<25> = Work::new(4).unwrap();
const LEXER_DISPATCH: Work<25> = Work::new(5).unwrap();
const TOKEN_PROBE: Work<25> = Work::new(6).unwrap();
const TOKEN_ADVANCE: Work<25> = Work::new(7).unwrap();
const GRAMMAR_DISPATCH: Work<25> = Work::new(8).unwrap();
const EARLY_ERROR_VISIT: Work<25> = Work::new(9).unwrap();
const LABEL_LOOKUP: Work<25> = Work::new(10).unwrap();
const DIRECT_EVAL_VISIT: Work<25> = Work::new(11).unwrap();
const NAME_VISIT: Work<25> = Work::new(12).unwrap();
const NAME_RECORD: Work<25> = Work::new(13).unwrap();
const NAME_COMPARISON: Work<25> = Work::new(14).unwrap();
const INTERNER_BYTES: Work<25> = Work::new(15).unwrap();
const INTERNER_UTF16: Work<25> = Work::new(16).unwrap();
const INTERNER_INDEX: Work<25> = Work::new(17).unwrap();
const INTERNER_CAPACITY: Work<25> = Work::new(18).unwrap();
const INTERNER_RELOCATION: Work<25> = Work::new(19).unwrap();
const DIAGNOSTIC_INPUT: Work<25> = Work::new(20).unwrap();
const DIAGNOSTIC_OUTPUT: Work<25> = Work::new(21).unwrap();
const TEMPLATE_INPUT: Work<25> = Work::new(22).unwrap();
const TEMPLATE_OUTPUT: Work<25> = Work::new(23).unwrap();
const SCOPE_WORK: Work<25> = Work::new(24).unwrap();
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
type ExecutionLedger = Operation<25, 0>;

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
    phase: &mut Phase<'_, 25, 0>,
    scope: Scope,
    work: InternerWork,
) -> Result<(), LedgerError<25, 0>> {
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

    /// Caps AST visits in name collection and validation.
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
    label_lookups: u64,
    name_visits: u64,
    name_records: u64,
    name_comparisons: u64,
    direct_eval_visits: u64,
    interner_bytes: u64,
    interner_utf16_units: u64,
    interner_index_work: u64,
    interner_capacity: u64,
    interner_relocation: u64,
    diagnostic_input: u64,
    diagnostic_output: u64,
    template_input: u64,
    template_output: u64,
    scope_work: u64,
    profile: Option<Report<25, 0>>,
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

    /// Returns the number of admitted logical label-table lookups.
    pub const fn label_lookups(&self) -> u64 {
        self.label_lookups
    }

    /// Returns the number of admitted direct-eval metadata visits.
    pub const fn direct_eval_visits(&self) -> u64 {
        self.direct_eval_visits
    }

    /// Returns the number of admitted AST visits in name collection and validation.
    pub const fn name_visits(&self) -> u64 {
        self.name_visits
    }

    /// Returns the number of admitted name record attempts.
    pub const fn name_records(&self) -> u64 {
        self.name_records
    }

    /// Returns the number of admitted symbol comparisons or logical name-table operations.
    pub const fn name_comparisons(&self) -> u64 {
        self.name_comparisons
    }

    /// Returns the admitted UTF-8 and Latin1 byte work in frontend interning.
    pub const fn interner_bytes(&self) -> u64 {
        self.interner_bytes
    }

    /// Returns the admitted UTF-16 unit work in frontend interning and name preparation.
    pub const fn interner_utf16_units(&self) -> u64 {
        self.interner_utf16_units
    }

    /// Returns the admitted frontend interner slot initialization, probes, reinsertion and publication.
    pub const fn interner_index_work(&self) -> u64 {
        self.interner_index_work
    }

    /// Returns the admitted additional capacity requests in the affected buffers' units.
    pub const fn interner_capacity(&self) -> u64 {
        self.interner_capacity
    }

    /// Returns the admitted potentially relocated initialized units in the affected buffers' units.
    pub const fn interner_relocation(&self) -> u64 {
        self.interner_relocation
    }

    /// Returns the admitted diagnostic spelling input in UTF-16 units.
    pub const fn diagnostic_input(&self) -> u64 {
        self.diagnostic_input
    }

    /// Returns the admitted source-spelling diagnostic output in UTF-8 bytes.
    pub const fn diagnostic_output(&self) -> u64 {
        self.diagnostic_output
    }

    /// Returns admitted template preparation input in UTF-16 units.
    pub const fn template_input(&self) -> u64 {
        self.template_input
    }

    /// Returns admitted template preparation output in UTF-16 units.
    pub const fn template_output(&self) -> u64 {
        self.template_output
    }

    /// Returns admitted script scope analysis work units.
    pub const fn scope_work(&self) -> u64 {
        self.scope_work
    }

    /// Returns the optional profile. Work dimensions are instructions, source
    /// reads, consumed code points, retained UTF-16 source units, and lexer
    /// lookahead probes, lexical production dispatches, buffered token probes,
    /// buffered token advance requests, grammar production attempts, and
    /// early-error AST visits, label-table lookups, direct-eval metadata visits,
    /// name visits, name records, name comparisons, interner bytes, interner
    /// UTF-16 units, interner index work, interner capacity and interner
    /// relocation, diagnostic spelling units and diagnostic payload bytes, then
    /// template input and output units, then scope analysis work. Scope work
    /// origins have separate profile entries under the scope analysis phase.
    /// Timed frontend phases include
    /// interning, diagnostic rendering and template preparation.
    pub const fn profile(&self) -> Option<&Report<25, 0>> {
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
    frontend: Scope,
    scope_analysis: Scope,
    scope_origins: [Scope; 12],
    utf8: Scope,
    utf16: Scope,
    host: Scope,
    latin1: Scope,
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
            ],
            [],
        );
        let mut ledger = if let Some(clock) = options.clock {
            Operation::with_profile_clock("evaluation", limits, 19, options.profile, clock)
        } else {
            Operation::with_profile("evaluation", limits, 19, options.profile)
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
        let scope_analysis = ledger
            .child(
                ledger.root(),
                "scope-analysis",
                Limits::unlimited(),
                Tariffs::identity(),
            )
            .map_err(Self::setup_error)?;
        let mut scope_origins = [scope_analysis; 12];
        for (slot, name) in scope_origins.iter_mut().zip(SCOPE_ORIGINS) {
            *slot = ledger
                .child(
                    scope_analysis,
                    name,
                    Limits::unlimited(),
                    Tariffs::identity(),
                )
                .map_err(Self::setup_error)?;
        }
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
            frontend,
            scope_analysis,
            scope_origins,
            utf8,
            utf16,
            host,
            latin1,
            stop: None,
        })
    }

    fn setup_error(error: LedgerError<25, 0>) -> ExecutionSetupError {
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
                            phase.apply(self.scope_origins[origin], Change::Work(SCOPE_WORK, units))
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
        analyze: impl FnOnce(&mut dyn FnMut(ScopeWork) -> Result<(), FrontendLimit>) -> JsResult<T>,
    ) -> JsResult<T> {
        self.ledger
            .with_phase(self.scope_analysis, |phase| {
                let mut refusal = None;
                let result = analyze(&mut |work| {
                    if let Some(error) = refusal {
                        return Err(error);
                    }
                    let (origin, units) = scope_charge(work);
                    match phase.apply(self.scope_origins[origin], Change::Work(SCOPE_WORK, units)) {
                        Ok(()) => Ok(()),
                        Err(LedgerError::Refused(_)) => {
                            let error = FrontendLimit::Scope(self.scope_work_limit);
                            refusal = Some(error);
                            Err(error)
                        }
                        Err(_) => unreachable!("scope analysis uses preallocated origins"),
                    }
                });
                refusal.map_or(result, |error| Err(RuntimeLimitError::from(error).into()))
            })
            .expect("the ledger owns its scope analysis phase")
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
                label_lookups: usage.work(LABEL_LOOKUP),
                name_visits: usage.work(NAME_VISIT),
                name_records: usage.work(NAME_RECORD),
                name_comparisons: usage.work(NAME_COMPARISON),
                direct_eval_visits: usage.work(DIRECT_EVAL_VISIT),
                interner_bytes: usage.work(INTERNER_BYTES),
                interner_utf16_units: usage.work(INTERNER_UTF16),
                interner_index_work: usage.work(INTERNER_INDEX),
                interner_capacity: usage.work(INTERNER_CAPACITY),
                interner_relocation: usage.work(INTERNER_RELOCATION),
                diagnostic_input: usage.work(DIAGNOSTIC_INPUT),
                diagnostic_output: usage.work(DIAGNOSTIC_OUTPUT),
                template_input: usage.work(TEMPLATE_INPUT),
                template_output: usage.work(TEMPLATE_OUTPUT),
                scope_work: usage.work(SCOPE_WORK),
                profile: self.ledger.finish(),
            },
        }
    }
}
