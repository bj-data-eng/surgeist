//! One canonical synchronous end; asynchronous task/resource custody is separate.
use super::*;
use crate::DomParsedHtmlStyleNotificationRequirement as R;
use crate::DomParsedHtmlStyleNotificationUnavailable as U;
use crate::{DomParsedHtmlExecutionState as Execution, DomParsedHtmlFrontierError};

/// Actual admitted result custody. Entry stores only its fixed terminal header,
/// never this Meter or a backreference to its own controlled record.
pub struct DomParsedHtmlSynchronousEnd {
    contribution: DomParsedHtmlAssociationId,
    _meter: Meter,
}
impl std::fmt::Debug for DomParsedHtmlSynchronousEnd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlSynchronousEnd")
            .field("contribution", &self.contribution)
            .finish()
    }
}
impl DomParsedHtmlSynchronousEnd {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.contribution
    }
}
/// Borrowed canonical readback, granting no execution or cleanup authority.
pub struct DomParsedHtmlSynchronousEndView<'dom> {
    entry: &'dom Entry,
}
impl DomParsedHtmlSynchronousEndView<'_> {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.entry.id
    }
}
/// A finite sparse-bank header walk. The composing dispatcher must admit one
/// caller work step BEFORE every advance, including the final None. It never
/// eagerly scans, clones a bank or acquires an original request graph loan.
pub struct DomParsedHtmlDispatchCursor<'dom> {
    dom: &'dom Dom,
    entries: bank::Iter<'dom, Entry>,
}
pub struct DomParsedHtmlDispatchEntry<'dom> {
    dom: &'dom Dom,
    entry: &'dom Entry,
}
impl<'dom> DomParsedHtmlDispatchCursor<'dom> {
    pub fn advance(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<Option<DomParsedHtmlDispatchEntry<'dom>>, DomError> {
        check_cancel(cancel)?;
        Ok(self
            .entries
            .next()
            .map(|(_, entry)| DomParsedHtmlDispatchEntry {
                dom: self.dom,
                entry,
            }))
    }
}
impl DomParsedHtmlDispatchEntry<'_> {
    pub fn contribution(&self) -> DomParsedHtmlAssociationId {
        self.entry.id
    }
    pub fn state(&self) -> Execution {
        self.entry.control.execution()
    }
    /// Only an activated unfinished entry reads its graph. Exact requirement
    /// formation and the complete producer scan spend its ORIGINAL Control.
    pub fn requirement(&self, cancel: &AtomicBool) -> Result<Option<R>, DomError> {
        check_cancel(cancel)?;
        if self.state() != Execution::Activated {
            return Ok(None);
        }
        let record = self
            .dom
            .pending_associations
            .controlled_entry(self.entry.id)
            .expect("canonical cursor retains the paired existing entry");
        let meter = Meter::new(record, self.dom.custody_registry.ledger().clone());
        let requirement = match remaining(self.dom, &meter, cancel)? {
            Some(requirement) => requirement,
            None => {
                meter.retain(size_of::<R>(), 0, 1, cancel)?;
                R::Unavailable {
                    contribution: self.entry.id,
                    reason: U::ConsumerFrontierUnavailable,
                }
            }
        };
        Ok(Some(requirement))
    }
}
impl Dom {
    pub fn parsed_html_dispatch_cursor(&self) -> DomParsedHtmlDispatchCursor<'_> {
        DomParsedHtmlDispatchCursor {
            dom: self,
            entries: self.pending_associations.entries.iter(),
        }
    }
    pub fn parsed_html_synchronous_end(
        &self,
        id: DomParsedHtmlAssociationId,
    ) -> Result<
        Option<DomParsedHtmlSynchronousEndView<'_>>,
        crate::DomParsedHtmlAssociationAdmissionError,
    > {
        if id.stamp().owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        let index = usize::try_from(id.stamp().revision().get())
            .map_err(|_| DomError::RepresentationExhausted)?;
        let entry = self
            .pending_associations
            .entries
            .get(index)
            .filter(|entry| entry.id == id)
            .ok_or(crate::DomParsedHtmlAssociationAdmissionError::NotPending {
                contribution: id,
            })?;
        Ok((entry.control.execution() == Execution::SynchronousEnd)
            .then_some(DomParsedHtmlSynchronousEndView { entry }))
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    /// Authenticate the complete sealed inventory and every influencing row.
    /// No task issuance, resource wait or queue delivery is a synchronous row.
    pub fn complete_synchronous_execution(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlSynchronousEnd, DomParsedHtmlFrontierError> {
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
        if self.meter.entry().control.execution() != Execution::SynchronousEnd
            && let Some(requirement) = remaining(self.dom, &self.meter, cancel)?
        {
            return Err(DomParsedHtmlFrontierError::Remaining(requirement));
        }
        let output = self.meter.fork();
        output.retain(size_of::<DomParsedHtmlSynchronousEnd>(), 0, 1, cancel)?;
        output.step(cancel)?;
        let result = DomParsedHtmlSynchronousEnd {
            contribution: self.meter.entry().id,
            _meter: output,
        };
        // All proof/storage/output/work admission precedes the durable fence.
        self.meter.entry().control.seal_execution();
        Ok(result)
    }
}
pub(super) fn original_document(
    value: HtmlParseMetadataDocument<'_>,
) -> crate::DomParsedHtmlHistoricalDocument {
    match value {
        HtmlParseMetadataDocument::Owned(node) => {
            crate::DomParsedHtmlHistoricalDocument::Constructed(node.id())
        }
        HtmlParseMetadataDocument::Captured(node) => {
            crate::DomParsedHtmlHistoricalDocument::Captured(node.id())
        }
    }
}
fn matches(
    q: &DomParsedHtmlStyleQualification,
    row: &surgeist_html_parser::HtmlParseAssociationOccurrence<'_>,
) -> bool {
    q.occurrence() == &row.id()
        && q.subject() == &row.subject().id()
        && q.original_document() == &original_document(row.document())
        && q.context_epoch() == &row.context().epoch()
}
/// Fixed authenticated contiguous progress, never an async authority owner.
pub(in crate::store::pending_associations) struct ExecutedPrefix {
    pub(super) last: surgeist_html_parser::HtmlParseAssociationOccurrenceId,
}
pub(super) struct Scan<'a> {
    pub(super) next: Option<surgeist_html_parser::HtmlParseAssociationOccurrence<'a>>,
    pub(super) covered: bool,
    pub(super) missing: Option<U>,
    // Whole-End currently supports one original root Document. An unrelated
    // inert no-effect Document does not invalidate an authenticated root command.
    pub(super) root_inventory: bool,
}
/// Authenticate the entire producer inventory before exposing its next row.
/// The only trusted execution fact is the issuer-controlled exact prefix token.
pub(super) fn scan<'a>(
    dom: &Dom,
    meter: &'a Meter,
    expected: Option<&surgeist_html_parser::HtmlParseAssociationOccurrenceId>,
    cancel: &AtomicBool,
) -> Result<Scan<'a>, DomError> {
    let prefix = meter
        .entry()
        .executed_prefix
        .try_lock()
        .map_err(|e| notification::guard_error(e, meter.entry().id))?;
    let marker = prefix.as_ref().map(|p| &p.last);
    let mut reached = marker.is_none();
    let mut covered = false;
    let mut first = None;
    let mut missing = None;
    let mut documents = 0usize;
    let mut root_inventory = true;
    let report = &meter.entry().report.value().value;
    let ordinary = meter.entry().context_basis.is_none()
        && report.document().is_some()
        && report.document_state().is_some_and(|s| {
            s.completion() == surgeist_html_parser::HtmlParseCompletion::Complete
                && s.readiness() == surgeist_html_parser::HtmlParseReadiness::Complete
                && s.ready_for_post_load_tasks()
        })
        && report.effective_scripting_mode() == surgeist_html::HtmlParserScriptingMode::Disabled;
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Fault(fault) => return Err(DomError::ParsedReportUnresolved(fault)),
            Step::Document(row) => {
                documents = documents
                    .checked_add(1)
                    .ok_or(DomError::RepresentationExhausted)?;
                if let surgeist_html_parser::HtmlParseAssociationCaptureDisposition::Unavailable(
                    kind,
                ) = row.disposition()
                {
                    missing.get_or_insert(U::IncompleteCapture(kind));
                }
                let source = original_document(row.document());
                if report.document().is_none_or(|root| {
                    source != crate::DomParsedHtmlHistoricalDocument::Constructed(root.root().id())
                }) {
                    root_inventory = false;
                }
                // Installation records only actual obligation Documents in the
                // pending bank. No-effect inert owners require no fake binding.
                if !row.has_obligations() {
                    continue;
                }
                let mut document = None;
                for binding in &meter.entry().documents {
                    meter.step(cancel)?;
                    if binding.source == source {
                        document = Some(binding.document);
                        break;
                    }
                }
                if let Some(document) = document {
                    dom.snapshot().document(document)?;
                    if dom.pending_associations.document(document) != Some(meter.entry().id) {
                        missing.get_or_insert(U::UnsupportedInventory);
                    }
                } else {
                    missing.get_or_insert(U::UnsupportedInventory);
                }
            }
            Step::Occurrence(row) => {
                // prior_document is per original Document, not a global row
                // ordinal. Verify its exact immediately preceding source edge
                // without introducing another history/index owner.
                let source = original_document(row.document());
                let mut preceding = None;
                let mut prior_rows = report.pending_associations();
                let valid = loop {
                    meter.step(cancel)?;
                    match prior_rows.advance() {
                        Step::Fault(fault) => return Err(DomError::ParsedReportUnresolved(fault)),
                        Step::Occurrence(prior) if prior.id() == row.id() => {
                            break row.prior_document() == preceding;
                        }
                        Step::Occurrence(prior)
                            if original_document(prior.document()) == source =>
                        {
                            preceding = Some(prior.id())
                        }
                        Step::Complete => break false,
                        _ => (),
                    }
                };
                if !valid {
                    missing.get_or_insert(U::UnsupportedInventory);
                }
                let id = row.id();
                if !reached {
                    covered |= expected == Some(&id) && row.requires_processing();
                    if marker == Some(&id) {
                        reached = true;
                    }
                } else if row.requires_processing() && first.is_none() {
                    first = Some(row);
                }
            }
            Step::Complete => break,
            _ => (),
        }
    }
    if !ordinary || documents == 0 || !reached {
        missing.get_or_insert(U::UnsupportedInventory);
    }
    Ok(Scan {
        next: first,
        covered,
        missing,
        root_inventory,
    })
}
pub(super) fn diagnostic(
    meter: &Meter,
    row: &surgeist_html_parser::HtmlParseAssociationOccurrence<'_>,
    phase: Option<DomParsedHtmlStylePhase>,
    cancel: &AtomicBool,
) -> Result<R, DomError> {
    let document = super::super::consume::mapped(
        meter.entry(),
        original_document(row.document()),
        meter,
        cancel,
    )
    .map_err(|error| match error {
        crate::DomParsedHtmlAssociationAdmissionError::Dom(error) => error,
        _ => DomError::RepresentationExhausted,
    })?;
    meter.retain(size_of::<R>(), 0, 1, cancel)?;
    Ok(if let Some(phase) = phase {
        R::Invocation {
            contribution: meter.entry().id,
            occurrence: row.id(),
            document,
            phase,
        }
    } else {
        R::Occurrence {
            contribution: meter.entry().id,
            occurrence: row.id(),
            document,
            kind: row.kind(),
        }
    })
}
pub(super) fn ensure_next(
    dom: &Dom,
    meter: &Meter,
    q: &DomParsedHtmlStyleQualification,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlStyleError> {
    let scan = scan(dom, meter, Some(q.occurrence()), cancel)?;
    if scan.missing.is_some() || scan.next.as_ref().is_none_or(|row| !matches(q, row)) {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    Ok(())
}
/// None is genuine sealed producer+consumer end, not mere cursor exhaustion.
pub(super) fn remaining(
    dom: &Dom,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<Option<R>, DomError> {
    let scan = scan(dom, meter, None, cancel)?;
    if let Some(reason) = scan
        .missing
        .or((!scan.root_inventory).then_some(U::UnsupportedInventory))
    {
        meter.retain(size_of::<R>(), 0, 1, cancel)?;
        return Ok(Some(R::Unavailable {
            contribution: meter.entry().id,
            reason,
        }));
    }
    let Some(row) = scan.next else {
        return Ok(None);
    };
    let graph = meter
        .entry()
        .style_prefix
        .try_lock()
        .map_err(|e| notification::guard_error(e, meter.entry().id))?;
    let phase = graph
        .as_ref()
        .filter(|held| matches(held.value().qualification(), &row))
        .map(|held| held.value().phase());
    diagnostic(meter, &row, phase, cancel).map(Some)
}
