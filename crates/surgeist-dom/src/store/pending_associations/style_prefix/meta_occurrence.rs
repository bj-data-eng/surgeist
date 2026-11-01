//! One authenticated original ChangePreferred after the supported Style prefix.
use super::*;
use crate::{DomMetaRootCommit, DomParsedHtmlMetaOccurrence, DomStamp, DomStyleCssomRoot};
use surgeist_cssom::{
    CssomBatch, CssomCommitRetirement, CssomEditGuard, CssomSheetId, CssomSnapshot, CssomStore,
};
use surgeist_html_parser::{HtmlParseAssociationOccurrence, HtmlParseAssociationOccurrenceId};

/// Refusal of one exact original occurrence. No other row executes on refusal.
#[derive(Debug)]
pub enum DomParsedHtmlMetaOccurrenceError {
    Style(DomParsedHtmlStyleError),
    Remaining(crate::DomParsedHtmlStyleNotificationRequirement),
    WrongOccurrence,
    AlreadyConsumed,
    MissingOwner { root: NodeId },
    MismatchedOwner { root: NodeId },
    OwnerBusy { root: NodeId },
    OwnerPoisoned { root: NodeId },
    Cssom(surgeist_cssom::CssomError),
}
impl From<surgeist_html_parser::HtmlParseUnresolvedReference> for DomParsedHtmlMetaOccurrenceError {
    fn from(e: surgeist_html_parser::HtmlParseUnresolvedReference) -> Self {
        Self::Style(e.into())
    }
}
impl From<DomError> for DomParsedHtmlMetaOccurrenceError {
    fn from(e: DomError) -> Self {
        Self::Style(e.into())
    }
}
impl From<DomParsedHtmlStyleError> for DomParsedHtmlMetaOccurrenceError {
    fn from(e: DomParsedHtmlStyleError) -> Self {
        Self::Style(e)
    }
}
impl From<crate::DomParsedHtmlAssociationAdmissionError> for DomParsedHtmlMetaOccurrenceError {
    fn from(e: crate::DomParsedHtmlAssociationAdmissionError) -> Self {
        Self::Style(e.into())
    }
}
impl std::fmt::Display for DomParsedHtmlMetaOccurrenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "original Meta occurrence refused: {self:?}")
    }
}
impl std::error::Error for DomParsedHtmlMetaOccurrenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Style(e) => Some(e),
            Self::Cssom(e) => Some(e),
            _ => None,
        }
    }
}
type Error = DomParsedHtmlMetaOccurrenceError;

fn select<'a>(
    dom: &Dom,
    meter: &'a Meter,
    expected: &HtmlParseAssociationOccurrenceId,
    cancel: &AtomicBool,
) -> Result<HtmlParseAssociationOccurrence<'a>, Error> {
    meter.entry().control.require_execution(meter.entry().id)?;
    let scan = frontier::scan(dom, meter, Some(expected), cancel)?;
    if let Some(reason) = scan.missing {
        meter.retain(
            size_of::<crate::DomParsedHtmlStyleNotificationRequirement>(),
            0,
            1,
            cancel,
        )?;
        return Err(Error::Remaining(
            crate::DomParsedHtmlStyleNotificationRequirement::Unavailable {
                contribution: meter.entry().id,
                reason,
            },
        ));
    }
    if scan.covered {
        let mut rows = meter.entry().report.value().value.pending_associations();
        loop {
            meter.step(cancel)?;
            match rows.advance() {
                Step::Fault(fault) => return Err(fault.into()),
                Step::Occurrence(row) if row.id() == *expected => {
                    return Err(if row.kind() == Kind::ChangePreferred {
                        Error::AlreadyConsumed
                    } else {
                        Error::WrongOccurrence
                    });
                }
                Step::Complete => return Err(Error::WrongOccurrence),
                Step::Document(_) | Step::Occurrence(_) | Step::Skipped => (),
            }
        }
    }
    let row = scan.next.ok_or(Error::WrongOccurrence)?;
    if row.kind() != Kind::ChangePreferred {
        let graph = lock(meter.entry())?;
        let phase = graph
            .as_ref()
            .filter(|held| held.value().qualification().occurrence() == &row.id())
            .map(|held| held.value().phase());
        return Err(Error::Remaining(frontier::diagnostic(
            meter, &row, phase, cancel,
        )?));
    }
    if row.id() != *expected {
        return Err(Error::WrongOccurrence);
    }

    Ok(row)
}

/// Actual output graphs precede the receipt controlling their last-owner drop.
pub struct DomParsedHtmlMetaOccurrenceCompletion {
    occurrence: DomParsedHtmlMetaOccurrence,
    root: DomMetaRootCommit,
    stamp: DomStamp,
    _meter: Meter,
}
impl std::fmt::Debug for DomParsedHtmlMetaOccurrenceCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlMetaOccurrenceCompletion")
            .field("occurrence", &self.occurrence)
            .field("stamp", &self.stamp)
            .finish()
    }
}
impl DomParsedHtmlMetaOccurrenceCompletion {
    pub fn occurrence(&self) -> &DomParsedHtmlMetaOccurrence {
        &self.occurrence
    }
    pub fn root(&self) -> &DomMetaRootCommit {
        &self.root
    }
    pub fn cssom(&self) -> &surgeist_cssom::CssomCommit {
        self.root.cssom()
    }
    pub fn stamp(&self) -> DomStamp {
        self.stamp
    }
}
/// Displaced actual producer/binding graphs remain charged until disposal.
pub struct DomParsedHtmlMetaOccurrenceRetirement {
    _cssom: CssomCommitRetirement,
    _binding: Option<style_custody::Bank>,
    _snapshot: CssomSnapshot,
    _marker: Option<frontier::ExecutedPrefix>,
    _meter: Meter,
}
pub struct DomParsedHtmlMetaOccurrencePublished<'caps> {
    _guard: MutexGuard<'caps, CssomStore>,
    completion: DomParsedHtmlMetaOccurrenceCompletion,
    retirement: DomParsedHtmlMetaOccurrenceRetirement,
}
impl DomParsedHtmlMetaOccurrencePublished<'_> {
    pub fn completion(&self) -> &DomParsedHtmlMetaOccurrenceCompletion {
        &self.completion
    }
    pub fn into_parts(
        self,
    ) -> (
        DomParsedHtmlMetaOccurrenceCompletion,
        DomParsedHtmlMetaOccurrenceRetirement,
    ) {
        let Self {
            _guard,
            completion,
            retirement,
        } = self;
        drop(_guard);
        (completion, retirement)
    }
}
pub struct DomParsedHtmlMetaOccurrencePreparation<'hold, 'dom, 'caps> {
    dom: &'hold mut &'dom mut Dom,
    _graph: MutexGuard<'hold, Stored>,
    marker: MutexGuard<'hold, Option<frontier::ExecutedPrefix>>,
    occurrence: DomParsedHtmlMetaOccurrence,
    store: MutexGuard<'caps, CssomStore>,
    snapshot: CssomSnapshot,
    batch: Option<CssomBatch>,
    binding: Option<style_custody::Bank>,
    runtime: &'hold Meter,
    name_usage: custody::Usage,
    meter: Meter,
    output: Option<Meter>,
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    pub fn meta_occurrence_document(
        &self,
        expected: &HtmlParseAssociationOccurrenceId,
        cancel: &AtomicBool,
    ) -> Result<DocumentId, Error> {
        let row = select(self.dom, &self.meter, expected, cancel)?;
        super::super::consume::mapped(
            self.meter.entry(),
            frontier::original_document(row.document()),
            &self.meter,
            cancel,
        )
        .map_err(Into::into)
    }
    pub fn prepare_meta_occurrence<'hold, 'caps>(
        &'hold mut self,
        expected: &HtmlParseAssociationOccurrenceId,
        cap: &'caps DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlMetaOccurrencePreparation<'hold, 'dom, 'caps>, Error> {
        let meter = self.meter.fork();
        let row = select(self.dom, &meter, expected, cancel)?;
        let occurrence = super::super::consume::decode_occurrence(&meter, &row, cancel)?;
        let root = occurrence.document().node();
        if cap.root() != root
            || self
                .dom
                .style_cssom_roots
                .get(root)
                .is_some_and(|old| !old.same_owner(cap))
        {
            return Err(Error::MismatchedOwner { root });
        }
        meter.retain(
            size_of::<DomParsedHtmlMetaOccurrencePreparation<'_, '_, '_>>()
                + size_of::<DomParsedHtmlMetaOccurrencePublished<'_>>(),
            0,
            0,
            cancel,
        )?;
        meter.step(cancel)?;
        let store = cap.acquire().map_err(|e| match e {
            crate::style::CssomRootAccessError::Busy => Error::OwnerBusy { root },
            crate::style::CssomRootAccessError::Poisoned => Error::OwnerPoisoned { root },
        })?;
        let snapshot = store.snapshot();
        let batch = store
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .map_err(Error::Cssom)?;
        self.dom.custody_registry.reconcile_releases();
        let baseline = self.dom.custody_registry.total();
        let binding = if self.dom.style_cssom_roots.get(root).is_none() {
            Some(self.dom.style_cssom_roots.prepare_insert(
                cap,
                occurrence.document(),
                |total| meter.controlled(total, baseline, cancel),
                || meter.step(cancel),
            )?)
        } else {
            None
        };
        let graph = lock(self.meter.entry())?;
        let marker = self
            .meter
            .entry()
            .executed_prefix
            .try_lock()
            .map_err(|e| notification::guard_error(e, self.meter.entry().id))?;
        let capacity = occurrence.content().shared_code_units().capacity();
        let name_usage = custody::Usage::new(
            capacity
                .checked_mul(size_of::<u16>())
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .ok_or(DomError::RepresentationExhausted)?,
            capacity,
        );
        // A Runtime candidate may alias this name before consuming commit.
        // The caller Hold outlives its candidate on every refusal path.
        meter.transfer_to(&self.meter, name_usage)?;
        Ok(DomParsedHtmlMetaOccurrencePreparation {
            dom: &mut self.dom,
            _graph: graph,
            marker,
            occurrence,
            store,
            snapshot,
            batch: Some(batch),
            binding,
            runtime: &self.meter,
            name_usage,
            meter,
            output: None,
        })
    }
}
impl DomParsedHtmlMetaOccurrencePreparation<'_, '_, '_> {
    pub fn occurrence(&self) -> &DomParsedHtmlMetaOccurrence {
        &self.occurrence
    }
    pub fn root_snapshot(&self) -> &CssomSnapshot {
        &self.snapshot
    }
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.meter.step(cancel)
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.runtime.runtime(usage, cancel)
    }
    pub fn set_disabled(
        &mut self,
        sheet: &CssomSheetId,
        disabled: bool,
        cancel: &AtomicBool,
    ) -> Result<(), Error> {
        let mut found = false;
        for candidate in self.snapshot.sheets() {
            self.meter.step(cancel)?;
            if candidate == sheet {
                found = true;
                break;
            }
        }
        if !found {
            return Err(Error::Cssom(surgeist_cssom::CssomError::MissingObject));
        }
        self.meter.step(cancel)?;
        self.batch
            .as_mut()
            .expect("unattempted producer batch")
            .set_disabled(sheet, disabled)
            .map(|_| ())
            .map_err(Error::Cssom)
    }
}
impl<'caps> DomParsedHtmlMetaOccurrencePreparation<'_, '_, 'caps> {
    /// Last fallible operation. The report describes only actual Runtime output
    /// already included in admit_runtime. Refusal leaves its caller Hold debit
    /// in place until the caller disposes its candidate graphs.
    pub fn commit(
        mut self,
        surviving: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlMetaOccurrencePublished<'caps>, Error> {
        self.meter.retain(
            size_of::<DomParsedHtmlMetaOccurrenceCompletion>()
                + size_of::<DomParsedHtmlMetaOccurrenceRetirement>(),
            0,
            1,
            cancel,
        )?;
        let admitted = self.runtime.runtime_usage();
        if surviving.bytes() > admitted.bytes()
            || surviving.units() > admitted.units()
            || surviving.outputs() > admitted.outputs()
        {
            return Err(DomError::RepresentationExhausted.into());
        }
        let output = self.meter.fork();
        self.meter.transfer_to(
            &output,
            custody::Usage::new(size_of::<DomParsedHtmlMetaOccurrenceCompletion>(), 0),
        )?;
        // This staged receipt remains an actual field after its payload. Early
        // return therefore disposes the name before either controlling receipt.
        self.output = Some(output);
        let output = self.output.as_ref().expect("staged actual output");
        let survivor_usage = self
            .name_usage
            .checked_add(custody::Usage::new(surviving.bytes(), surviving.units()))?;
        self.runtime.check_transfer_to(output, survivor_usage)?;
        self.meter.step(cancel)?;
        let next = frontier::ExecutedPrefix {
            last: self.occurrence.id().clone(),
        };
        let root = self.occurrence.document().node();
        let stamp = self.dom.snapshot().stamp();
        let prepared = self
            .store
            .prepare_commit(self.batch.take().expect("unattempted producer batch"))
            .map_err(Error::Cssom)?;
        #[cfg(test)]
        PREPARED.with(|n| n.set(n.get() + 1));
        self.meter.step(cancel)?;
        // Everything fallible is finished. These exclusive prevalidated receipt
        // moves change no total, output count or work. Caller candidates remain
        // charged by Hold on Err; successful survivors now belong to Completion.
        self.runtime
            .transfer_runtime_output_to(output, surviving, self.name_usage);
        let (cssom, dead) = prepared.publish();
        let old_marker = self.marker.replace(next);
        let old_binding = self
            .binding
            .take()
            .map(|bank| std::mem::replace(&mut self.dom.style_cssom_roots, bank));
        // Destructure only after the successful fence, when every owned graph
        // moves directly to its preformed result or actual retirement owner.
        let Self {
            occurrence,
            store,
            snapshot,
            meter,
            mut output,
            ..
        } = self;
        Ok(DomParsedHtmlMetaOccurrencePublished {
            _guard: store,
            completion: DomParsedHtmlMetaOccurrenceCompletion {
                occurrence,
                root: DomMetaRootCommit { root, cssom },
                stamp,
                _meter: output.take().expect("actual preformed output"),
            },
            retirement: DomParsedHtmlMetaOccurrenceRetirement {
                _cssom: dead,
                _binding: old_binding,
                _snapshot: snapshot,
                _marker: old_marker,
                _meter: meter,
            },
        })
    }
}

#[cfg(test)]
mod tests;
#[cfg(test)]
thread_local! {static PREPARED: std::cell::Cell<usize> = const {std::cell::Cell::new(0)};}
