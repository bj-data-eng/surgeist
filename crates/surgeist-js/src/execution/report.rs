use std::{cell::OnceCell, rc::Rc};

use surgeist_ledger::Report;

use super::Execution;
use crate::{JsError, JsResult};

/// A returned callback result and its admitted work, even after a host refusal.
#[derive(Debug)]
pub struct ExecutionOutcome<T> {
    /// The callback's completion. The exact first retained host stop takes precedence.
    pub result: JsResult<T>,
    /// The same immutable report available through any retained receipt.
    pub report: ExecutionReport,
}

/// How an operation relinquished execution ownership.
#[derive(Debug)]
pub enum ExecutionTermination {
    /// A `JsResult` was delivered, including any guest error or first host stop.
    /// This disposition alone does not certify success; the result owns its error.
    Completed,
    /// No result was delivered. Cancellation and Rust unwinding share this state.
    Abandoned {
        /// The unreturned first host stop, if one had already been retained.
        first_stop: Option<JsError>,
    },
}

/// Immutable admitted work and optional profiling for one terminated operation.
/// Cloning shares its published body; it does not copy retained profile vectors.
#[derive(Clone, Debug)]
pub struct ExecutionReport {
    body: Rc<OnceCell<ReportData>>,
}

/// Observation of a task's one-shot report publication, including after drop.
#[derive(Clone, Debug)]
pub struct ExecutionReceipt {
    body: Rc<OnceCell<ReportData>>,
}

impl ExecutionReceipt {
    /// Returns None until termination, then an immutable handle to the same body.
    /// An unpolled reserved task is running until dropped; access never snapshots
    /// the ledger or allocates another report.
    pub fn report(&self) -> Option<ExecutionReport> {
        self.body.get().map(|_| ExecutionReport {
            body: Rc::clone(&self.body),
        })
    }
}

pub(super) struct PreparedReport {
    body: Rc<OnceCell<ReportData>>,
}

impl PreparedReport {
    pub(super) fn new() -> Self {
        Self {
            body: Rc::new(OnceCell::new()),
        }
    }

    pub(super) fn receipt(&self) -> ExecutionReceipt {
        ExecutionReceipt {
            body: Rc::clone(&self.body),
        }
    }

    fn publish(self, data: ReportData) -> ExecutionReport {
        self.body
            .set(data)
            .expect("one execution publishes its report once");
        ExecutionReport { body: self.body }
    }
}

#[derive(Debug)]
struct ReportData {
    termination: ExecutionTermination,
    effects_started: bool,
    instructions: u64,
    job_preparations: u64,
    job_invocations: u64,
    job_polls: u64,
    generator_requests: u64,
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
    optimizer_work: u64,
    compiler_work: u64,
    compiler_depth_peak: usize,
    profile: Option<Report<31, 0>>,
}

impl ExecutionReport {
    fn data(&self) -> &ReportData {
        self.body.get().expect("a published report has its body")
    }

    /// Whether a result was delivered or the operation was abandoned.
    /// Completed does not imply successful execution; inspect the outcome result.
    pub fn termination(&self) -> &ExecutionTermination {
        &self.data().termination
    }

    /// Whether this operation began a guest instruction, invoked a program callback,
    /// entered a generator request, or committed a declaration effect.
    ///
    /// This remains true across nested evaluation and later refusal. It does not
    /// assert that a particular property changed, or that restarting source is safe.
    pub fn effects_started(&self) -> bool {
        self.data().effects_started
    }

    /// Returns the number of admitted instructions.
    pub fn instructions(&self) -> u64 {
        self.data().instructions
    }

    /// Returns the number of admitted native task preparation factory invocations.
    ///
    /// This includes factories that return an error or unwind. Factory dispatch
    /// marks possible effects and does not itself count as a task poll.
    pub fn job_preparations(&self) -> u64 {
        self.data().job_preparations
    }

    /// Returns the number of admitted synchronous native job callback invocations.
    pub fn job_invocations(&self) -> u64 {
        self.data().job_invocations
    }

    /// Returns the number of admitted native async job future polls.
    pub fn job_polls(&self) -> u64 {
        self.data().job_polls
    }

    /// Returns the number of admitted generator request attempts.
    ///
    /// Each `next`, `return` or `throw` request counts once before validation
    /// or promise preparation, including requests against completed generators
    /// and attempts that subsequently fail validation.
    pub fn generator_requests(&self) -> u64 {
        self.data().generator_requests
    }

    /// Returns the number of admitted source read attempts.
    pub fn source_reads(&self) -> u64 {
        self.data().source_reads
    }

    /// Returns the number of decoded points consumed by the frontend.
    pub fn source_code_points(&self) -> u64 {
        self.data().source_code_points
    }

    /// Returns the cumulative number of UTF-16 units retained in source text.
    pub fn retained_source_units(&self) -> u64 {
        self.data().retained_source_units
    }

    /// Returns the number of admitted lexer lookahead probes.
    pub fn lexer_lookahead_probes(&self) -> u64 {
        self.data().lexer_lookahead_probes
    }

    /// Returns the number of admitted lexical production dispatches.
    pub fn lexer_dispatches(&self) -> u64 {
        self.data().lexer_dispatches
    }

    /// Returns the number of admitted buffered token probes.
    pub fn token_probes(&self) -> u64 {
        self.data().token_probes
    }

    /// Returns the number of admitted buffered token advance requests.
    pub fn token_advances(&self) -> u64 {
        self.data().token_advances
    }

    /// Returns the number of admitted grammar production attempts.
    pub fn grammar_dispatches(&self) -> u64 {
        self.data().grammar_dispatches
    }

    /// Returns the number of admitted early-error AST visits.
    pub fn early_error_visits(&self) -> u64 {
        self.data().early_error_visits
    }

    /// Returns the number of admitted logical label-table lookups.
    pub fn label_lookups(&self) -> u64 {
        self.data().label_lookups
    }

    /// Returns the number of admitted direct-eval metadata visits.
    pub fn direct_eval_visits(&self) -> u64 {
        self.data().direct_eval_visits
    }

    /// Returns admitted name, validation and delegated AST semantic-query visits.
    /// This includes compiler completion-value and source-start queries.
    pub fn name_visits(&self) -> u64 {
        self.data().name_visits
    }

    /// Returns the number of admitted name record attempts.
    pub fn name_records(&self) -> u64 {
        self.data().name_records
    }

    /// Returns the number of admitted symbol comparisons or logical name-table operations.
    pub fn name_comparisons(&self) -> u64 {
        self.data().name_comparisons
    }

    /// Returns the admitted UTF-8 and Latin1 byte work in frontend interning.
    pub fn interner_bytes(&self) -> u64 {
        self.data().interner_bytes
    }

    /// Returns the admitted UTF-16 unit work in frontend interning and name preparation.
    pub fn interner_utf16_units(&self) -> u64 {
        self.data().interner_utf16_units
    }

    /// Returns the admitted frontend interner slot initialization, probes, reinsertion and publication.
    pub fn interner_index_work(&self) -> u64 {
        self.data().interner_index_work
    }

    /// Returns the admitted additional capacity requests in the affected buffers' units.
    pub fn interner_capacity(&self) -> u64 {
        self.data().interner_capacity
    }

    /// Returns the admitted potentially relocated initialized units in the affected buffers' units.
    pub fn interner_relocation(&self) -> u64 {
        self.data().interner_relocation
    }

    /// Returns the admitted diagnostic spelling input in UTF-16 units.
    pub fn diagnostic_input(&self) -> u64 {
        self.data().diagnostic_input
    }

    /// Returns the admitted source-spelling diagnostic output in UTF-8 bytes.
    pub fn diagnostic_output(&self) -> u64 {
        self.data().diagnostic_output
    }

    /// Returns admitted template preparation input in UTF-16 units.
    pub fn template_input(&self) -> u64 {
        self.data().template_input
    }

    /// Returns admitted template preparation output in UTF-16 units.
    pub fn template_output(&self) -> u64 {
        self.data().template_output
    }

    /// Returns admitted script scope analysis work units.
    pub fn scope_work(&self) -> u64 {
        self.data().scope_work
    }

    /// Returns admitted optimizer work, including producer request reservations.
    ///
    /// This combines the units described by the optimizer profile origins; it is
    /// neither elapsed time nor a measurement of physical memory consumption.
    pub fn optimizer_work(&self) -> u64 {
        self.data().optimizer_work
    }

    /// Returns admitted compiler work, including delegated payload requests.
    pub fn compiler_work(&self) -> u64 {
        self.data().compiler_work
    }

    /// Returns the greatest admitted structural compiler depth in any phase.
    ///
    /// Refused prospective entries do not increase this value. It is not the
    /// VM call depth or an estimate of native stack memory.
    pub fn compiler_depth_peak(&self) -> usize {
        self.data().compiler_depth_peak
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
    /// origins have separate profile entries under scope analysis, bytecode
    /// compilation and script installation. Timed frontend phases include
    /// interning, diagnostic rendering and template preparation. Dimension 25 is
    /// optimizer work, with separate producer origins and an optimizer phase.
    /// Retirement and opaque kernel units are prepaid native request prices.
    /// Dimension 26 is compiler work, attributed under bytecode compilation;
    /// delegated scope and name work retain dimensions 24 and 12–14.
    /// Dimensions 27–29 count native task preparation factory invocations,
    /// synchronous native job invocations and inner task polls.
    /// Their action-local phases include owned cleanup and exclude time between polls.
    /// Dimension 30 counts generator request attempts; `generator-request` observes
    /// each admitted synchronous request algorithm, without spanning suspension.
    pub fn profile(&self) -> Option<&Report<31, 0>> {
        self.data().profile.as_ref()
    }
}

impl Execution {
    pub(crate) fn finish<T>(mut self, result: JsResult<T>) -> ExecutionOutcome<T> {
        let result = self.stop.take().map_or(result, Err);
        ExecutionOutcome {
            result,
            report: self.publish_report(ExecutionTermination::Completed),
        }
    }

    pub(super) fn abandon(mut self) -> ExecutionReport {
        let termination = ExecutionTermination::Abandoned {
            first_stop: self.stop.take(),
        };
        self.publish_report(termination)
    }

    fn publish_report(self, termination: ExecutionTermination) -> ExecutionReport {
        let usage = self
            .ledger
            .usage(self.ledger.root())
            .expect("the ledger owns its root");
        self.prepared_report.publish(ReportData {
            termination,
            effects_started: self.effects_started,
            instructions: usage.work(super::INSTRUCTION),
            job_preparations: usage.work(super::JOB_PREPARATION),
            job_invocations: usage.work(super::JOB_INVOCATION),
            job_polls: usage.work(super::JOB_POLL),
            generator_requests: usage.work(super::GENERATOR_REQUEST),
            source_reads: usage.work(super::SOURCE_READ),
            source_code_points: usage.work(super::SOURCE_CONSUMPTION),
            retained_source_units: usage.work(super::SOURCE_RETENTION),
            lexer_lookahead_probes: usage.work(super::LEXER_LOOKAHEAD),
            lexer_dispatches: usage.work(super::LEXER_DISPATCH),
            token_probes: usage.work(super::TOKEN_PROBE),
            token_advances: usage.work(super::TOKEN_ADVANCE),
            grammar_dispatches: usage.work(super::GRAMMAR_DISPATCH),
            early_error_visits: usage.work(super::EARLY_ERROR_VISIT),
            label_lookups: usage.work(super::LABEL_LOOKUP),
            name_visits: usage.work(super::NAME_VISIT),
            name_records: usage.work(super::NAME_RECORD),
            name_comparisons: usage.work(super::NAME_COMPARISON),
            direct_eval_visits: usage.work(super::DIRECT_EVAL_VISIT),
            interner_bytes: usage.work(super::INTERNER_BYTES),
            interner_utf16_units: usage.work(super::INTERNER_UTF16),
            interner_index_work: usage.work(super::INTERNER_INDEX),
            interner_capacity: usage.work(super::INTERNER_CAPACITY),
            interner_relocation: usage.work(super::INTERNER_RELOCATION),
            diagnostic_input: usage.work(super::DIAGNOSTIC_INPUT),
            diagnostic_output: usage.work(super::DIAGNOSTIC_OUTPUT),
            template_input: usage.work(super::TEMPLATE_INPUT),
            template_output: usage.work(super::TEMPLATE_OUTPUT),
            scope_work: usage.work(super::SCOPE_WORK),
            optimizer_work: usage.work(super::OPTIMIZER_WORK),
            compiler_work: usage.work(super::COMPILER_WORK),
            compiler_depth_peak: self.compiler_depth_peak,
            profile: self.ledger.finish(),
        })
    }
}
