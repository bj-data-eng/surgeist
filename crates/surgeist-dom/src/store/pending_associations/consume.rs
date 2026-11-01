//! Authenticated whole-contribution preparation; no native hook replay.
use super::control::Meter;
use super::*;
use crate::parsed_meta::*;
use crate::{
    DocumentString, DomMetaRootCommit, DomParsedHtmlAssociationAdmissionError,
    DomParsedHtmlAssociationLimits, DomParsedHtmlAssociationRuntimeUsage, DomStyleCssomRoot,
};
use std::sync::MutexGuard;
use surgeist_cssom::{CssomBatch, CssomEditGuard, CssomSheetId, CssomSnapshot};
use surgeist_html_parser::{
    HtmlParseAssociationCaptureDisposition as Capture, HtmlParseAssociationOccurrenceKind as Kind,
    HtmlParseMetadataDocument, HtmlParsePendingAssociationStep as Step,
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Prefix,
    Outstanding,
    Closed,
}

/// Exclusive custody of the exact original pending destination contribution.
/// Prefix work/admission must precede external bank formation. Drop retains
/// original policy and spent work. A forgotten preparation cannot reopen it.
///
/// Given an authentic pending identity and its admitted finite policy, admission
/// can inspect immutable history, then release the exclusive owner borrow:
/// ```no_run
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits};
/// use std::sync::atomic::AtomicBool;
/// fn inspect(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///            limits: DomParsedHtmlAssociationLimits, cancel: &AtomicBool) {
///     let mut hold = dom.begin_parsed_html_meta(id, limits, cancel).unwrap();
///     hold.validate_meta(cancel).unwrap();
///     drop(hold);
///     let _ = dom.snapshot();
/// }
/// ```
/// The canonical owner cannot be accessed while its actual hold remains live:
/// ```compile_fail,E0502
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///            limits: DomParsedHtmlAssociationLimits, cancel: &AtomicBool) {
///     let hold = dom.begin_parsed_html_meta(id, limits, cancel).unwrap();
///     let _ = dom.snapshot();
///     let _ = hold.step(cancel);
/// }
/// ```
/// A real preparation exclusively holds its original admission front:
/// ```compile_fail,E0502
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits,
///                    DomStyleCssomRoot};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///            limits: DomParsedHtmlAssociationLimits, caps: &[&DomStyleCssomRoot],
///            cancel: &AtomicBool) {
///     let mut hold = dom.begin_parsed_html_meta(id, limits, cancel).unwrap();
///     let prepared = hold.prepare(caps, cancel).unwrap();
///     let _ = hold.step(cancel);
///     drop(prepared);
/// }
/// ```
pub struct DomParsedHtmlMetaRuntimeHold<'dom> {
    dom: &'dom mut Dom,
    phase: Phase,
    meter: Meter,
}
impl Drop for DomParsedHtmlMetaRuntimeHold<'_> {
    fn drop(&mut self) {
        if self.phase != Phase::Outstanding {
            self.meter.entry().control.leave();
        }
    }
}
impl Dom {
    pub fn begin_parsed_html_meta(
        &mut self,
        id: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlMetaRuntimeHold<'_>, DomParsedHtmlAssociationAdmissionError> {
        if self.source_operation.is_some() || !self.source_parents.is_empty() {
            return Err(DomError::SourceOperationPending.into());
        }
        if self.snapshot().incomplete_source_operation().is_some()
            || self.snapshot().incomplete_hosted_operation().is_some()
        {
            return Err(DomError::IncompleteSourceOperation.into());
        }
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if id.stamp.owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        let record = self.pending_associations.controlled_entry(id)?;
        record.value().control.bind(id, limits)?;
        let meter = Meter::new(record, self.custody_registry.ledger().clone());
        meter.retain(size_of::<DomParsedHtmlMetaRuntimeHold<'_>>(), 0, 0, cancel)?;
        meter.entry().control.enter();
        Ok(DomParsedHtmlMetaRuntimeHold {
            dom: self,
            phase: Phase::Prefix,
            meter,
        })
    }
}
fn document_source(value: HtmlParseMetadataDocument<'_>) -> crate::DomParsedHtmlHistoricalDocument {
    match value {
        HtmlParseMetadataDocument::Owned(node) => {
            crate::DomParsedHtmlHistoricalDocument::Constructed(node.id())
        }
        HtmlParseMetadataDocument::Captured(node) => {
            crate::DomParsedHtmlHistoricalDocument::Captured(node.id())
        }
    }
}
pub(super) fn mapped(
    entry: &Entry,
    source: crate::DomParsedHtmlHistoricalDocument,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<DocumentId, DomParsedHtmlAssociationAdmissionError> {
    for row in &entry.documents {
        meter.step(cancel)?;
        if row.source == source {
            return Ok(row.document);
        }
    }
    Err(DomParsedHtmlAssociationAdmissionError::DocumentMismatch {
        contribution: entry.id,
        document: source,
    })
}
fn validate(
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlAssociationAdmissionError> {
    let entry = meter.entry();
    let mut cursor = entry.report.value().value.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Document(row) => {
                if let Capture::Unavailable(missing) = row.disposition() {
                    return Err(DomParsedHtmlAssociationAdmissionError::Incomplete {
                        contribution: entry.id,
                        missing,
                    });
                }
            }
            Step::Occurrence(row) => {
                if row.kind() == Kind::ChangePreferred {
                    mapped(entry, document_source(row.document()), meter, cancel)?;
                } else if row.requires_processing() {
                    return Err(DomParsedHtmlAssociationAdmissionError::Unsupported {
                        contribution: entry.id,
                        occurrence: row.id(),
                        kind: row.kind(),
                    });
                }
            }
            Step::Skipped => (),
            Step::Complete => return Ok(()),
            Step::Fault(fault) => return Err(fault.into()),
        }
    }
}
impl DomParsedHtmlMetaRuntimeHold<'_> {
    fn require_prefix(&self) -> Result<(), DomError> {
        if self.phase != Phase::Prefix {
            return Err(DomError::ParsedAssociationOperationPending {
                pending: self.meter.entry().id,
            });
        }
        Ok(())
    }
    pub fn pending(&self) -> DomParsedHtmlPendingAssociations<'_> {
        self.meter.entry().view()
    }
    pub fn spent_work(&self) -> u64 {
        self.meter.entry().control.spent_work()
    }
    pub fn limits(&self) -> DomParsedHtmlAssociationLimits {
        self.meter.entry().control.limits()
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.require_prefix()?;
        self.meter.step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.require_prefix()?;
        self.meter.runtime(usage, cancel)
    }
    /// Classify all original history without acquiring or invoking a CSSOM owner.
    /// Later pending entries are valid here; destination execution order is
    /// qualified by preparation after all Session owner setup has completed.
    pub fn validate_meta(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomParsedHtmlAssociationAdmissionError> {
        self.require_prefix()?;
        validate(&self.meter, cancel)
    }
}
struct Root<'caps> {
    root: NodeId,
    store: MutexGuard<'caps, surgeist_cssom::CssomStore>,
    snapshot: CssomSnapshot,
    batch: Option<CssomBatch>,
}
/// Private real producers and canonical acknowledgement; Drop publishes nothing.
pub struct DomParsedHtmlMetaPreparation<'hold, 'dom, 'caps> {
    dom: &'hold mut &'dom mut Dom,
    phase: &'hold mut Phase,
    runtime: &'hold mut Meter,
    occurrences: Vec<DomParsedHtmlMetaOccurrence>,
    roots: Vec<Root<'caps>>,
    acknowledgement: Option<Bank>,
    binding: Option<style_custody::Bank>,
    meter: Option<Meter>,
}
impl Drop for DomParsedHtmlMetaPreparation<'_, '_, '_> {
    fn drop(&mut self) {
        self.occurrences.clear();
        self.roots.clear();
        drop(self.acknowledgement.take());
        drop(self.binding.take());
        // Backing capacities above remain represented until their actual Vec
        // fields drop, so the final field still owns their admission.
        *self.phase = Phase::Closed;
    }
}
/// Dead original bank/control and producer backing, disposed after publication.
pub struct DomParsedHtmlMetaRetirement {
    _pending: Bank,
    _binding: Option<style_custody::Bank>,
    _cssom: Vec<surgeist_cssom::CssomCommitRetirement>,
    _occurrences: Vec<DomParsedHtmlMetaOccurrence>,
    _meter: Meter,
}
/// Actual locks remain held until the adapter installs its prequalified names.
pub struct DomParsedHtmlMetaPublished<'caps> {
    guards: Vec<MutexGuard<'caps, surgeist_cssom::CssomStore>>,
    completion: DomParsedHtmlMetaCompletion,
    retirement: DomParsedHtmlMetaRetirement,
}
impl DomParsedHtmlMetaPublished<'_> {
    pub fn into_parts(self) -> (DomParsedHtmlMetaCompletion, DomParsedHtmlMetaRetirement) {
        let Self {
            guards,
            completion,
            retirement,
        } = self;
        drop(guards);
        (completion, retirement)
    }
}

fn qualify_order(
    dom: &Dom,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlAssociationAdmissionError> {
    let entry = meter.entry();
    for row in &entry.documents {
        meter.step(cancel)?;
        dom.snapshot().document(row.document).map_err(|_| {
            DomParsedHtmlAssociationAdmissionError::StaleDocument {
                document: row.document,
            }
        })?;
        match dom.pending_associations.document(row.document) {
            Some(pending) if pending == entry.id => (),
            Some(pending) => {
                return Err(DomParsedHtmlAssociationAdmissionError::EarlierPending {
                    document: row.document,
                    pending,
                });
            }
            None => {
                return Err(DomParsedHtmlAssociationAdmissionError::NotPending {
                    contribution: entry.id,
                });
            }
        }
    }
    Ok(())
}

pub(super) fn decode_occurrence(
    meter: &Meter,
    row: &surgeist_html_parser::HtmlParseAssociationOccurrence<'_>,
    cancel: &AtomicBool,
) -> Result<DomParsedHtmlMetaOccurrence, DomParsedHtmlAssociationAdmissionError> {
    use surgeist_html_parser::{HtmlParseHistoryFrame, HtmlParseHistoryStep};
    let entry = meter.entry();
    let report = &entry.report.value().value;
    let document = mapped(entry, document_source(row.document()), meter, cancel)?;
    let scratch = meter.fork();
    let mut frames = Vec::new();
    let history = report.history_view();
    scratch.reserve(&mut frames, history.len(), cancel)?;
    for _ in 0..history.len() {
        scratch.step(cancel)?;
        frames.push(HtmlParseHistoryFrame::default());
    }
    let group = row
        .preferred_name()
        .expect("sealed ChangePreferred has its original name");
    let mut reader = history
        .cursor(group, &mut frames, None)
        .map_err(DomParsedHtmlAssociationAdmissionError::from)?;
    let mut units = Vec::new();
    loop {
        meter.step(cancel)?;
        match reader.advance() {
            HtmlParseHistoryStep::Leaf(leaf) => {
                meter.reserve_units(&mut units, leaf.units().len(), cancel)?;
                for &unit in leaf.units() {
                    meter.step(cancel)?;
                    units.push(unit);
                }
            }
            HtmlParseHistoryStep::Progress => (),
            HtmlParseHistoryStep::Complete => break,
            HtmlParseHistoryStep::Cancelled => return Err(DomError::Cancelled.into()),
            HtmlParseHistoryStep::Fault(fault) => return Err(fault.into()),
        }
    }
    meter.retain(size_of::<Vec<u16>>(), 0, 0, cancel)?;
    let content = DocumentString::from_utf16(units);
    Ok(DomParsedHtmlMetaOccurrence {
        contribution: entry.id,
        id: row.id(),
        subject: row.subject().id(),
        document,
        content,
    })
}

fn occurrences(
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<Vec<DomParsedHtmlMetaOccurrence>, DomParsedHtmlAssociationAdmissionError> {
    let entry = meter.entry();
    let report = &entry.report.value().value;
    let mut cursor = report.pending_associations();
    let mut result = Vec::new();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Fault(fault) => return Err(fault.into()),
            Step::Complete => return Ok(result),
            Step::Occurrence(row) if row.kind() == Kind::ChangePreferred => {
                meter.reserve(&mut result, 1, cancel)?;
                result.push(decode_occurrence(meter, &row, cancel)?);
            }
            Step::Document(_) | Step::Occurrence(_) | Step::Skipped => (),
        }
    }
}

impl<'dom> DomParsedHtmlMetaRuntimeHold<'dom> {
    #[expect(
        clippy::result_large_err,
        reason = "retain the actual destination root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn prepare<'hold, 'caps>(
        &'hold mut self,
        caps: &'caps [&'caps DomStyleCssomRoot],
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlMetaPreparation<'hold, 'dom, 'caps>, DomParsedHtmlMetaError> {
        self.require_prefix()?;
        self.phase = Phase::Outstanding;
        let meter = self.meter.fork();
        let built = (|| {
            validate(&meter, cancel)?;
            qualify_order(self.dom, &meter, cancel)?;
            meter.retain(
                size_of::<DomParsedHtmlMetaPreparation<'_, '_, '_>>(),
                0,
                0,
                cancel,
            )?;
            let occurrences = occurrences(&meter, cancel)?;
            let mut roots: Vec<Root<'caps>> = Vec::new();
            for occurrence in &occurrences {
                meter.step(cancel)?;
                let root = occurrence.document.node();
                let mut reached = false;
                for row in &roots {
                    meter.step(cancel)?;
                    if row.root == root {
                        reached = true;
                        break;
                    }
                }
                if reached {
                    continue;
                }
                let canonical = self.dom.style_cssom_roots.get(root);
                let mut selected: Option<&DomStyleCssomRoot> = None;
                for cap in caps {
                    meter.step(cancel)?;
                    if cap.root() != root {
                        continue;
                    }
                    if canonical.is_some_and(|old| !old.same_owner(cap))
                        || selected.is_some_and(|old| !old.same_owner(cap))
                    {
                        return Err(DomParsedHtmlMetaError::MismatchedOwner { root });
                    }
                    selected = Some(*cap);
                }
                let cap = selected.ok_or(DomParsedHtmlMetaError::MissingOwner { root })?;
                meter.reserve(&mut roots, 1, cancel)?;
                meter.step(cancel)?;
                let store = cap.acquire().map_err(|error| match error {
                    crate::style::CssomRootAccessError::Busy => {
                        DomParsedHtmlMetaError::OwnerBusy { root }
                    }
                    crate::style::CssomRootAccessError::Poisoned => {
                        DomParsedHtmlMetaError::OwnerPoisoned { root }
                    }
                })?;
                let snapshot = store.snapshot();
                let batch = store
                    .batch(CssomEditGuard::from_snapshot(&snapshot))
                    .map_err(|error| DomParsedHtmlMetaError::Cssom { root, error })?;
                roots.push(Root {
                    root,
                    store,
                    snapshot,
                    batch: Some(batch),
                });
            }
            self.dom.custody_registry.reconcile_releases();
            let baseline = self.dom.custody_registry.total();
            let mut binding = None;
            for row in &roots {
                meter.step(cancel)?;
                if self.dom.style_cssom_roots.get(row.root).is_some() {
                    continue;
                }
                let cap = caps
                    .iter()
                    .find(|cap| cap.root() == row.root)
                    .expect("qualified capability bank");
                let bank = binding.as_ref().unwrap_or(&self.dom.style_cssom_roots);
                binding = Some(bank.prepare_insert(
                    cap,
                    DocumentId(row.root),
                    |total| meter.controlled(total, baseline, cancel),
                    || meter.step(cancel),
                )?);
            }
            let acknowledgement = self.dom.pending_associations.prepare_remove(
                meter.entry().id,
                self.dom.limits,
                |total| meter.controlled(total, baseline, cancel),
                || meter.step(cancel),
            )?;
            Ok::<_, DomParsedHtmlMetaError>((occurrences, roots, acknowledgement, binding))
        })();
        match built {
            Ok((occurrences, roots, acknowledgement, binding)) => {
                Ok(DomParsedHtmlMetaPreparation {
                    dom: &mut self.dom,
                    phase: &mut self.phase,
                    runtime: &mut self.meter,
                    occurrences,
                    roots,
                    acknowledgement: Some(acknowledgement),
                    binding,
                    meter: Some(meter),
                })
            }
            Err(error) => {
                self.phase = Phase::Closed;
                Err(error)
            }
        }
    }
}

impl DomParsedHtmlMetaPreparation<'_, '_, '_> {
    pub fn occurrences(&self) -> &[DomParsedHtmlMetaOccurrence] {
        &self.occurrences
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.meter
            .as_ref()
            .expect("live parsed preparation")
            .step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.runtime.runtime(usage, cancel)
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain the actual destination root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn root_snapshot(
        &self,
        root: NodeId,
        cancel: &AtomicBool,
    ) -> Result<&CssomSnapshot, DomParsedHtmlMetaError> {
        for row in &self.roots {
            self.step(cancel)?;
            if row.root == root {
                return Ok(&row.snapshot);
            }
        }
        Err(DomParsedHtmlMetaError::MissingOwner { root })
    }
    #[expect(
        clippy::result_large_err,
        reason = "retain the actual destination root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn set_disabled(
        &mut self,
        root: NodeId,
        sheet: &CssomSheetId,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomParsedHtmlMetaError> {
        let meter = self.meter.as_ref().expect("live parsed preparation");
        for row in &mut self.roots {
            meter.step(cancel)?;
            if row.root != root {
                continue;
            }
            let mut found = false;
            for candidate in row.snapshot.sheets() {
                meter.step(cancel)?;
                if candidate == sheet {
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(DomParsedHtmlMetaError::StaleProducer { root });
            }
            row.batch
                .as_mut()
                .expect("unattempted producer batch")
                .set_disabled(sheet, disabled)
                .map_err(|error| DomParsedHtmlMetaError::Cssom { root, error })?;
            return Ok(());
        }
        Err(DomParsedHtmlMetaError::MissingOwner { root })
    }
}

impl<'caps> DomParsedHtmlMetaPreparation<'_, '_, 'caps> {
    /// Last fallible operation. Every producer, canonical bank and output slot
    /// is prepared before the first genuine producer publication.
    #[expect(
        clippy::result_large_err,
        reason = "retain the actual destination root and original producer diagnostic without refusal-time allocation"
    )]
    pub fn commit(
        mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlMetaPublished<'caps>, DomParsedHtmlMetaError> {
        let meter = self.meter.as_ref().expect("live parsed preparation");
        qualify_order(self.dom, meter, cancel)?;
        let count = self.roots.len();
        let mut roots = Vec::new();
        meter.reserve(&mut roots, count, cancel)?;
        let mut retired = Vec::new();
        meter.reserve(&mut retired, count, cancel)?;
        let mut guards = Vec::new();
        meter.reserve(&mut guards, count, cancel)?;
        let mut prepared = Vec::new();
        meter.reserve(&mut prepared, count, cancel)?;
        meter.retain(
            size_of::<DomParsedHtmlMetaPublished<'caps>>(),
            0,
            count
                .checked_add(2)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        let mut name_bytes = self
            .occurrences
            .capacity()
            .checked_mul(size_of::<DomParsedHtmlMetaOccurrence>())
            .ok_or(DomError::RepresentationExhausted)?;
        let mut name_units = 0usize;
        for occurrence in &self.occurrences {
            meter.step(cancel)?;
            let capacity = occurrence.content.shared_code_units().capacity();
            name_units = name_units
                .checked_add(capacity)
                .ok_or(DomError::RepresentationExhausted)?;
            name_bytes = name_bytes
                .checked_add(
                    capacity
                        .checked_mul(size_of::<u16>())
                        .ok_or(DomError::RepresentationExhausted)?,
                )
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let surviving = guards
            .capacity()
            .checked_mul(size_of::<MutexGuard<'caps, surgeist_cssom::CssomStore>>())
            .and_then(|n| {
                n.checked_add(
                    roots
                        .capacity()
                        .checked_mul(size_of::<DomMetaRootCommit>())?,
                )
            })
            .and_then(|n| {
                n.checked_add(
                    retired
                        .capacity()
                        .checked_mul(size_of::<surgeist_cssom::CssomCommitRetirement>())?,
                )
            })
            .and_then(|n| n.checked_add(size_of::<DomParsedHtmlMetaPublished<'caps>>()))
            .and_then(|n| n.checked_add(name_bytes))
            .ok_or(DomError::RepresentationExhausted)?;
        for row in &mut self.roots {
            meter.step(cancel)?;
            let value = row
                .store
                .prepare_commit(row.batch.take().expect("unattempted producer batch"))
                .map_err(|error| DomParsedHtmlMetaError::Cssom {
                    root: row.root,
                    error,
                })?;
            prepared.push((row.root, value));
        }
        meter.step(cancel)?;
        // Whole supported Meta consumption activates immediately before its
        // actual effects and retires this Entry by the same infallible fence;
        // no partial Meta execution/frontier capability is introduced.
        meter.entry().control.activate_execution();
        // Infallible composed fence: all slots, actual loans and headroom exist.
        for (root, producer) in prepared.drain(..) {
            let (cssom, dead) = producer.publish();
            roots.push(DomMetaRootCommit { root, cssom });
            retired.push(dead);
        }
        drop(prepared);
        let id = meter.entry().id;
        let pending = std::mem::replace(
            &mut self.dom.pending_associations,
            self.acknowledgement
                .take()
                .expect("prepared whole acknowledgement"),
        );
        let binding = self
            .binding
            .take()
            .map(|binding| std::mem::replace(&mut self.dom.style_cssom_roots, binding));
        for row in self.roots.drain(..) {
            guards.push(row.store);
        }
        drop(std::mem::take(&mut self.roots));
        let occurrences = std::mem::take(&mut self.occurrences);
        let meter = self
            .meter
            .take()
            .expect("actual output and retired graph custody");
        meter.shrink_to(custody::Usage::new(surviving, name_units));
        *self.phase = Phase::Closed;
        Ok(DomParsedHtmlMetaPublished {
            guards,
            completion: DomParsedHtmlMetaCompletion {
                contribution: id,
                stamp: self.dom.snapshot().stamp(),
                roots,
            },
            retirement: DomParsedHtmlMetaRetirement {
                _pending: pending,
                _binding: binding,
                _cssom: retired,
                _occurrences: occurrences,
                _meter: meter,
            },
        })
    }
}

#[cfg(test)]
mod tests;
