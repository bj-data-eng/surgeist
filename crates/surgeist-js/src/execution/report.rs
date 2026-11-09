use std::{cell::OnceCell, rc::Rc};

use surgeist_ledger::{NoDimension, Report, Usage};

use super::Execution;
use crate::{JsError, JsResult, error::EngineResourceError};

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
/// Readers of ledger usage return the ledger fault, as an [`EngineResourceError`], when
/// the execution ledger found a broken internal invariant. A completed outcome
/// then carries the same fault as its result.
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

/// The counts and profile the ledger reported, or the failure that kept them out.
#[derive(Debug)]
struct LedgerReport {
    usage: Usage<super::ExecutionWork, NoDimension>,
    profile: Option<Report<super::ExecutionWork, NoDimension>>,
}

#[derive(Debug)]
struct ReportData {
    termination: ExecutionTermination,
    effects_started: bool,
    compiler_depth_peak: usize,
    ledger: Result<LedgerReport, EngineResourceError>,
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

    fn ledger(&self) -> Result<&LedgerReport, EngineResourceError> {
        self.data().ledger.as_ref().map_err(|error| *error)
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
    pub fn instructions(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::Instruction))
    }

    /// Returns the number of admitted native task preparation factory invocations.
    ///
    /// This includes factories that return an error or unwind. Factory dispatch
    /// marks possible effects and does not itself count as a task poll.
    pub fn job_preparations(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::JobPreparation))
    }

    /// Returns the number of admitted synchronous native job callback invocations.
    pub fn job_invocations(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::JobInvocation))
    }

    /// Returns the number of admitted native async job future polls.
    pub fn job_polls(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::JobPoll))
    }

    /// Returns finite executor batch snapshot and claim credits.
    /// Scratch disposal or claim rollback does not refund accepted work.
    /// This counter does not price an entire drain or arbitrary custom executors.
    pub fn job_claim_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::JobClaimWork))
    }

    /// Returns the number of admitted generator request attempts.
    ///
    /// Each `next`, `return` or `throw` request counts once before validation
    /// or promise preparation, including requests against completed generators
    /// and attempts that subsequently fail validation.
    pub fn generator_requests(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::GeneratorRequest))
    }

    /// Returns module-owned metadata construction credits.
    /// Delegated AST name work is reported by the separate name counters.
    pub fn module_metadata_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::ModuleMetadataWork))
    }

    /// Returns the number of admitted source read attempts.
    pub fn source_reads(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::SourceRead))
    }

    /// Returns the number of decoded points consumed by the frontend.
    pub fn source_code_points(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::SourceConsumption))
    }

    /// Returns the cumulative number of UTF-16 units retained in source text.
    pub fn retained_source_units(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::SourceRetention))
    }

    /// Returns the number of admitted lexer lookahead probes.
    pub fn lexer_lookahead_probes(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::LexerLookahead))
    }

    /// Returns the number of admitted lexical production dispatches.
    pub fn lexer_dispatches(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::LexerDispatch))
    }

    /// Returns the number of admitted buffered token probes.
    pub fn token_probes(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::TokenProbe))
    }

    /// Returns the number of admitted buffered token advance requests.
    pub fn token_advances(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::TokenAdvance))
    }

    /// Returns the number of admitted grammar production attempts.
    pub fn grammar_dispatches(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::GrammarDispatch))
    }

    /// Returns the number of admitted early-error AST visits.
    pub fn early_error_visits(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::EarlyErrorVisit))
    }

    /// Returns the number of admitted logical label-table lookups.
    pub fn label_lookups(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::LabelLookup))
    }

    /// Returns the number of admitted direct-eval metadata visits.
    pub fn direct_eval_visits(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::DirectEvalVisit))
    }

    /// Returns admitted name, validation and delegated AST semantic-query visits.
    /// This includes compiler completion-value and source-start queries.
    pub fn name_visits(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::NameVisit))
    }

    /// Returns the number of admitted name record attempts.
    pub fn name_records(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::NameRecord))
    }

    /// Returns the number of admitted symbol comparisons or logical name-table operations.
    pub fn name_comparisons(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::NameComparison))
    }

    /// Returns the admitted UTF-8 and Latin1 byte work in frontend interning.
    pub fn interner_bytes(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::InternerBytes))
    }

    /// Returns the admitted UTF-16 unit work in frontend interning and name preparation.
    pub fn interner_utf16_units(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::InternerUtf16))
    }

    /// Returns the admitted frontend interner slot initialization, probes, reinsertion and publication.
    pub fn interner_index_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::InternerIndex))
    }

    /// Returns the admitted additional capacity requests in the affected buffers' units.
    pub fn interner_capacity(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::InternerCapacity))
    }

    /// Returns the admitted potentially relocated initialized units in the affected buffers' units.
    pub fn interner_relocation(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::InternerRelocation))
    }

    /// Returns the admitted diagnostic spelling input in UTF-16 units.
    pub fn diagnostic_input(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::DiagnosticInput))
    }

    /// Returns the admitted source-spelling diagnostic output in UTF-8 bytes.
    pub fn diagnostic_output(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::DiagnosticOutput))
    }

    /// Returns admitted template preparation input in UTF-16 units.
    pub fn template_input(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::TemplateInput))
    }

    /// Returns admitted template preparation output in UTF-16 units.
    pub fn template_output(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::TemplateOutput))
    }

    /// Returns admitted script scope analysis work units.
    pub fn scope_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self.ledger()?.usage.work(super::ExecutionWork::ScopeWork))
    }

    /// Returns admitted optimizer work, including producer request reservations.
    ///
    /// This combines the units described by the optimizer profile origins; it is
    /// neither elapsed time nor a measurement of physical memory consumption.
    pub fn optimizer_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::OptimizerWork))
    }

    /// Returns admitted compiler work, including delegated payload requests.
    pub fn compiler_work(&self) -> Result<u64, EngineResourceError> {
        Ok(self
            .ledger()?
            .usage
            .work(super::ExecutionWork::CompilerWork))
    }

    /// Returns the greatest admitted structural compiler depth in any phase.
    ///
    /// Refused prospective entries do not increase this value. It is not the
    /// VM call depth or an estimate of native stack memory.
    pub fn compiler_depth_peak(&self) -> usize {
        self.data().compiler_depth_peak
    }

    /// Returns the optional profile. Its work dimensions are the
    /// [`ExecutionWork`](super::ExecutionWork) variants, in declaration order.
    /// Scope work origins have separate profile entries under scope analysis,
    /// bytecode compilation and script installation. Timed frontend phases include
    /// interning, diagnostic rendering and template preparation.
    /// `OptimizerWork` has separate producer origins and an optimizer phase.
    /// Retirement and opaque kernel units are prepaid native request prices.
    /// `CompilerWork` is attributed under bytecode compilation; delegated scope
    /// and name work retain `ScopeWork`, `NameVisit`, `NameRecord` and
    /// `NameComparison`.
    /// `JobPreparation`, `JobInvocation` and `JobPoll` count native task
    /// preparation factory invocations, synchronous native job invocations and
    /// inner task polls. Their action-local phases include owned cleanup and
    /// exclude time between polls.
    /// `GeneratorRequest` counts generator request attempts; `generator-request`
    /// observes each admitted synchronous request algorithm, without spanning
    /// suspension.
    /// `ModuleMetadataWork` counts module-owned metadata credits, with delegated
    /// name dimensions attributed under the same detached metadata phase.
    /// `JobClaimWork` counts finite batch snapshot and claim credits under the
    /// detached `job-claim` phase and its six fixed producer origins. Rollback
    /// does not refund these credits; they do not price an entire drain.
    ///
    /// # Errors
    ///
    /// Returns the ledger fault when the execution ledger found a broken
    /// internal invariant, because its usage can no longer be trusted. Every
    /// reader backed by ledger usage returns the same fault.
    pub fn profile(
        &self,
    ) -> Result<Option<&Report<super::ExecutionWork, NoDimension>>, EngineResourceError> {
        Ok(self.ledger()?.profile.as_ref())
    }
}

impl Execution {
    pub(crate) fn finish<T>(mut self, result: JsResult<T>) -> ExecutionOutcome<T> {
        let result = self.stop.take().map_or(result, Err);
        let report = self.publish_report(ExecutionTermination::Completed);
        // A ledger fault leaves the counts untrustworthy, so it replaces the result.
        let result = match report.ledger() {
            Ok(_) => result,
            Err(fault) => Err(fault.into()),
        };
        ExecutionOutcome { result, report }
    }

    pub(super) fn abandon(mut self) -> ExecutionReport {
        let termination = ExecutionTermination::Abandoned {
            first_stop: self.stop.take(),
        };
        self.publish_report(termination)
    }

    fn publish_report(self, termination: ExecutionTermination) -> ExecutionReport {
        let root = self.ledger.root();
        let ledger = match self.ledger.usage(root).copied() {
            Ok(usage) => self
                .ledger
                .finish()
                .map(|profile| LedgerReport { usage, profile })
                .map_err(super::ledger_fault),
            Err(error) => Err(super::ledger_fault(error)),
        };
        self.prepared_report.publish(ReportData {
            termination,
            effects_started: self.effects_started,
            compiler_depth_peak: self.compiler_depth_peak,
            ledger,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn faulted(fault: EngineResourceError) -> ExecutionReport {
        PreparedReport::new().publish(ReportData {
            termination: ExecutionTermination::Completed,
            effects_started: true,
            compiler_depth_peak: 3,
            ledger: Err(fault),
        })
    }

    #[test]
    fn ledger_backed_readers_return_the_fault_and_the_rest_stay_available() {
        let fault = EngineResourceError::LedgerDefect(surgeist_ledger::Defect::EventLimit);
        let report = faulted(fault);
        assert_eq!(report.instructions(), Err(fault));
        assert_eq!(report.job_polls(), Err(fault));
        assert_eq!(report.module_metadata_work(), Err(fault));
        assert_eq!(report.compiler_work(), Err(fault));
        assert!(matches!(report.profile(), Err(error) if error == fault));
        assert!(matches!(
            report.termination(),
            ExecutionTermination::Completed
        ));
        assert!(report.effects_started());
        assert_eq!(report.compiler_depth_peak(), 3);
    }
}
