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
fn qualify_executed(
    dom: &Dom,
    meter: &Meter,
    q: &DomParsedHtmlStyleQualification,
    cancel: &AtomicBool,
) -> Result<bool, DomError> {
    for binding in meter.entry().bindings.iter() {
        meter.step(cancel)?;
        if &binding.source == q.subject() {
            let snapshot = dom.snapshot();
            let node = snapshot.node(binding.node)?;
            return Ok(node.node.document() == q.document()
                && node.as_style().is_some_and(|style| {
                    style.state().association_availability()
                        == crate::DomStyleSheetAvailability::PendingParsedProcessing
                }));
        }
    }
    Ok(false)
}
/// None is genuine sealed producer+consumer end, not cursor exhaustion alone.
pub(super) fn remaining(
    dom: &Dom,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<Option<R>, DomError> {
    let id = meter.entry().id;
    let graph = meter
        .entry()
        .style_prefix
        .try_lock()
        .map_err(|e| notification::guard_error(e, id))?;
    let completed = meter
        .entry()
        .completed_style
        .try_lock()
        .map_err(|e| notification::guard_error(e, id))?;
    let meta = meter
        .entry()
        .completed_meta
        .try_lock()
        .map_err(|e| notification::guard_error(e, id))?;
    let mut meta_pending = meta.as_ref().map(|prefix| &prefix.last);
    meter.step(cancel)?;
    // The final unavailable result may coexist with the first retained candidate.
    meter.retain(size_of::<Option<R>>() + size_of::<R>(), 0, 0, cancel)?;
    let report = &meter.entry().report.value().value;
    let ordinary = meter.entry().context_basis.is_none()
        && report.document().is_some()
        && report.document_state().is_some_and(|state| {
            state.completion() == surgeist_html_parser::HtmlParseCompletion::Complete
                && state.readiness() == surgeist_html_parser::HtmlParseReadiness::Complete
                && state.ready_for_post_load_tasks()
        })
        && report.effective_scripting_mode() == surgeist_html::HtmlParserScriptingMode::Disabled;
    let mut first = None;
    let mut output_admitted = false;
    let mut missing = None;
    let mut documents = 0usize;
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
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
                    missing.get_or_insert(U::UnsupportedInventory);
                }
                let mut mapped = None;
                for binding in &meter.entry().documents {
                    meter.step(cancel)?;
                    if binding.source == source {
                        mapped = Some(binding.document);
                        break;
                    }
                }
                if let Some(document) = mapped {
                    dom.snapshot().document(document)?;
                    if dom.pending_associations.document(document) != Some(id) {
                        missing.get_or_insert(U::UnsupportedInventory);
                    }
                } else {
                    missing.get_or_insert(U::UnsupportedInventory);
                }
            }
            Step::Occurrence(row) if row.requires_processing() => {
                if row.kind() == Kind::ChangePreferred
                    && let Some(last) = meta_pending
                {
                    if row.id() == *last {
                        meta_pending = None;
                    }
                    continue;
                }
                let current_done = graph
                    .as_ref()
                    .is_some_and(|held| match held.value().as_ref() {
                        Graph::Rendered { request, .. } => {
                            request.cursor.profile()
                                == HtmlStyleExecutionProfile::NoExecutableJavaScript
                                && matches(request.qualification(), &row)
                                && origin_matches(&request.origin, &row.context())
                        }
                        Graph::Inline(request) => {
                            request.cursor.action() == Action::Complete
                                && request.created_sheet.is_none()
                                && request.cursor.profile()
                                    == HtmlStyleExecutionProfile::NoExecutableJavaScript
                                && matches(request.qualification(), &row)
                                && origin_matches(&request.origin, &row.context())
                        }
                        _ => false,
                    });
                let prior_done = completed.as_ref().is_some_and(|old| {
                    old.profile == HtmlStyleExecutionProfile::NoExecutableJavaScript
                        && matches(&old.qualification, &row)
                        && origin_matches(&old.origin, &row.context())
                });
                if current_done || prior_done {
                    let q = if current_done {
                        graph
                            .as_ref()
                            .expect("qualified current execution")
                            .value()
                            .qualification()
                    } else {
                        &completed
                            .as_ref()
                            .expect("qualified predecessor")
                            .qualification
                    };
                    if !qualify_executed(dom, meter, q, cancel)? {
                        missing.get_or_insert(U::UnsupportedInventory);
                    }
                    continue;
                }
                if first.is_none() {
                    let source = original_document(row.document());
                    let mut document = None;
                    for binding in &meter.entry().documents {
                        meter.step(cancel)?;
                        if binding.source == source {
                            document = Some(binding.document);
                            break;
                        }
                    }
                    if let Some(document) = document {
                        meter.retain(0, 0, 1, cancel)?;
                        output_admitted = true;
                        first = Some(
                            if graph
                                .as_ref()
                                .is_some_and(|held| matches(held.value().qualification(), &row))
                            {
                                R::Invocation {
                                    contribution: id,
                                    occurrence: row.id(),
                                    document,
                                    phase: graph.as_ref().expect("matched graph").value().phase(),
                                }
                            } else {
                                R::Occurrence {
                                    contribution: id,
                                    occurrence: row.id(),
                                    document,
                                    kind: row.kind(),
                                }
                            },
                        );
                    } else {
                        missing.get_or_insert(U::UnsupportedInventory);
                    }
                }
            }
            Step::Complete => break,
            _ => (),
        }
    }
    if !ordinary || documents == 0 || meta_pending.is_some() {
        missing.get_or_insert(U::UnsupportedInventory);
    }
    Ok(if let Some(reason) = missing {
        if !output_admitted {
            meter.retain(0, 0, 1, cancel)?;
        }
        Some(R::Unavailable {
            contribution: id,
            reason,
        })
    } else {
        first
    })
}
