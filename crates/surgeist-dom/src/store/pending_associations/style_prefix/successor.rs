//! Real old-sheet removal and the next original policy request in one fence.
use super::*;
use crate::parsed_style::DomParsedHtmlStyleSuccessorError as Error;
use crate::{DomCommit, DomStyleCssomRoot, DomStyleSheet};
use surgeist_cssom::{
    CssomBatch, CssomCommit, CssomCommitRetirement, CssomEditGuard, CssomSnapshot, CssomStore,
};

/// Genuine publications for one successor. The next invocation remains Pending
/// at CheckInlineCsp; this is neither recreation nor a synchronous frontier.
pub struct DomParsedHtmlStyleSuccessorCompletion {
    qualification: DomParsedHtmlStyleInitialPolicyRequest,
    dom: DomCommit,
    cssom: CssomCommit,
    _formation: Option<source::formation::Formation>,
    _meter: Meter,
}
impl DomParsedHtmlStyleSuccessorCompletion {
    pub fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        &self.qualification
    }
    pub fn dom(&self) -> &DomCommit {
        &self.dom
    }
    pub fn cssom(&self) -> &CssomCommit {
        &self.cssom
    }
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        DomParsedHtmlStylePhase::InlinePolicyPending
    }
}
impl std::fmt::Debug for DomParsedHtmlStyleSuccessorCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleSuccessorCompletion")
            .field("qualification", self.qualification())
            .field("stamp", &self.dom.changes().to())
            .finish()
    }
}
/// Actual displaced backing stays alive through the complete publication fence.
/// Graphs drop before the original admission and Meter which charged them.
pub struct DomParsedHtmlStyleSuccessorRetirement {
    _text: DocumentString,
    _nonce: Option<DocumentString>,
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _history: history::Retirement,
    _cssom: CssomCommitRetirement,
    _snapshot: CssomSnapshot,
    _admission: control::Admission,
    _meter: Meter,
}
pub struct DomParsedHtmlStyleSuccessorPublished<'caps> {
    _guard: std::sync::MutexGuard<'caps, CssomStore>,
    completion: DomParsedHtmlStyleSuccessorCompletion,
    retirement: DomParsedHtmlStyleSuccessorRetirement,
}
impl DomParsedHtmlStyleSuccessorPublished<'_> {
    pub fn completion(&self) -> &DomParsedHtmlStyleSuccessorCompletion {
        &self.completion
    }
    pub fn into_parts(
        self,
    ) -> (
        DomParsedHtmlStyleSuccessorCompletion,
        DomParsedHtmlStyleSuccessorRetirement,
    ) {
        (self.completion, self.retirement)
    }
}
/// Closed original removal editor. Caller supplies the real owner, never an
/// occurrence ordinal, a native Source receipt or a mutable producer batch.
pub struct DomParsedHtmlStyleSuccessorPreparation<'hold, 'dom, 'caps> {
    graph: MutexGuard<'hold, Stored>,
    completed: MutexGuard<'hold, Option<CompletedInvocation>>,
    dom: &'hold mut &'dom mut Dom,
    runtime: &'hold Meter,
    store: std::sync::MutexGuard<'caps, CssomStore>,
    snapshot: CssomSnapshot,
    batch: CssomBatch,
    selection: Selection,
    text: DocumentString,
    nonce: Option<DocumentString>,
    element: crate::NodeId,
    old: DomStyleSheet,
    meter: Meter,
    formed: Meter,
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    pub fn successor_document(
        &self,
        cancel: &AtomicBool,
    ) -> Result<DocumentId, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        match graph.as_ref().map(|held| held.value().as_ref()) {
            Some(Graph::Rendered { request, .. }) => Ok(request.qualification.document()),
            value => Err(DomParsedHtmlStyleError::WrongPhase {
                phase: value.map_or(DomParsedHtmlStylePhase::Unstarted, Graph::phase),
            }),
        }
    }
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    pub fn prepare_successor<'hold, 'caps>(
        &'hold mut self,
        cap: &'caps DomStyleCssomRoot,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSuccessorPreparation<'hold, 'dom, 'caps>, Error> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let value = graph
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?
            .value();
        let Graph::Rendered { request, .. } = value.as_ref() else {
            return Err(DomParsedHtmlStyleError::WrongPhase {
                phase: value.phase(),
            }
            .into());
        };
        let completed = completed_lock(self.meter.entry())?;
        if completed.is_some() {
            return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                contribution: self.meter.entry().id,
            }
            .into());
        }
        let old = request
            .created_sheet
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::InvalidSheetInput)?;
        let root = request.qualification.document().node();
        if cap.root() != root
            || self
                .dom
                .style_cssom_roots
                .get(root)
                .is_none_or(|bank| !bank.same_owner(cap))
        {
            return Err(Error::MismatchedOwner { root });
        }
        let formed = self
            .meter
            .fork()
            .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
        let selection = select_next(&formed, request, cancel)?;
        let (text, nonce) = form_text_nonce(&formed, selection.qualification.occurrence(), cancel)?;
        let old = old.clone();
        let element = old.association().element();
        let view = self.dom.snapshot();
        view.document(request.qualification.document())?;
        let node = view.node(element)?;
        let style = node
            .as_style()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
        if node.node.document() != selection.qualification.document()
            || self
                .dom
                .pending_associations
                .document(selection.qualification.document())
                != Some(self.meter.entry().id)
            || style.state().sheet.as_ref() != Some(&old)
            || style.state().availability
                != crate::DomStyleSheetAvailability::PendingParsedProcessing
        {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        drop(view);
        let meter = self.meter.fork();
        meter.retain(
            size_of::<DomParsedHtmlStyleSuccessorPreparation<'_, '_, '_>>()
                + size_of::<DomParsedHtmlStyleSuccessorPublished<'_>>()
                + size_of::<DomParsedHtmlStyleSuccessorRetirement>(),
            0,
            0,
            cancel,
        )?;
        let store = cap.lock().map_err(|error| match error {
            crate::DomStyleCssomRootError::Busy => Error::OwnerBusy { root },
            crate::DomStyleCssomRootError::Poisoned => Error::OwnerPoisoned { root },
            _ => Error::MismatchedOwner { root },
        })?;
        let snapshot = store.snapshot();
        let mut batch = store
            .batch(CssomEditGuard::from_snapshot(&snapshot))
            .map_err(Error::Cssom)?;
        meter.step(cancel)?;
        batch.remove_sheet(old.id()).map_err(Error::Cssom)?;
        Ok(DomParsedHtmlStyleSuccessorPreparation {
            graph,
            completed,
            dom: &mut self.dom,
            runtime: &self.meter,
            store,
            snapshot,
            batch,
            selection,
            text,
            nonce,
            element,
            old,
            meter,
            formed,
        })
    }
}
// Complete all source inventory authentication before selecting a candidate.
fn select_next(
    meter: &Meter,
    old: &DomParsedHtmlStyleInlinePolicyRequest,
    cancel: &AtomicBool,
) -> Result<Selection, Error> {
    let report = &meter.entry().report.value().value;
    if meter.entry().context_basis.is_some()
        || report.document().is_none()
        || report.effective_scripting_mode() != surgeist_html::HtmlParserScriptingMode::Disabled
    {
        return Err(DomParsedHtmlStyleError::UnsupportedProfile {
            contribution: meter.entry().id,
        }
        .into());
    }
    let mut all = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match all.advance() {
            Step::Document(row) => {
                if let surgeist_html_parser::HtmlParseAssociationCaptureDisposition::Unavailable(
                    missing,
                ) = row.disposition()
                {
                    return Err(crate::DomParsedHtmlAssociationAdmissionError::Incomplete {
                        contribution: meter.entry().id,
                        missing,
                    }
                    .into());
                }
            }
            Step::Complete => break,
            _ => (),
        }
    }
    let mut seen = false;
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if row.requires_processing() => {
                if row.id() == *old.qualification.occurrence() {
                    seen = true;
                    continue;
                }
                let mut mapped = None;
                for binding in &meter.entry().documents {
                    meter.step(cancel)?;
                    if matches!(row.document(), HtmlParseMetadataDocument::Owned(node) if binding.source == crate::DomParsedHtmlHistoricalDocument::Constructed(node.id()))
                    {
                        mapped = Some(binding.document);
                        break;
                    }
                }
                let document = mapped.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                if !seen
                    || row.kind()
                        != Kind::StyleUpdate(surgeist_html::HtmlStyleTrigger::ParserPopped)
                    || row.subject().id() != *old.qualification.subject()
                    || document != old.qualification.document()
                {
                    return Err(remaining(meter, &row, document, cancel)?);
                }
                for prior in [row.prior_document(), row.prior_style()]
                    .into_iter()
                    .flatten()
                {
                    if !dependency(
                        meter,
                        &prior,
                        &row.id(),
                        old.qualification.occurrence(),
                        cancel,
                    )? {
                        return Err(DomParsedHtmlStyleError::WrongRequest.into());
                    }
                }
                let context = row.context();
                let requirement = context.requirement(Role::InitialPolicy);
                if requirement != Requirement::OfflineUnbound {
                    return Err(DomParsedHtmlStyleError::PolicyContext { requirement }.into());
                }
                let slot = context
                    .slot(Role::InitialPolicy)
                    .ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                if slot != old.qualification.slot
                    || context.input_epoch(Role::InitialPolicy) != *old.qualification.slot.epoch()
                {
                    return Err(DomParsedHtmlStyleError::WrongRequest.into());
                }
                let inputs = row.style().ok_or(DomParsedHtmlStyleError::WrongRequest)?;
                if !inputs.connected() || !inputs.type_eligible() {
                    return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                        contribution: meter.entry().id,
                    }
                    .into());
                }
                let mut cursor = HtmlStyleUpdateCursor::new(old.cursor.profile());
                for reply in [
                    Reply::Applied,
                    Reply::Connected(inputs.connected()),
                    Reply::TypeEligible(inputs.type_eligible()),
                ] {
                    meter.step(cancel)?;
                    cursor.reply(reply)?;
                }
                let HtmlParseMetadataDocument::Owned(original) = row.document() else {
                    return Err(DomParsedHtmlStyleError::WrongRequest.into());
                };
                return Ok(Selection {
                    qualification: DomParsedHtmlStyleInitialPolicyRequest {
                        contribution: meter.entry().id,
                        occurrence: row.id(),
                        subject: row.subject().id(),
                        original_document: crate::DomParsedHtmlHistoricalDocument::Constructed(
                            original.id(),
                        ),
                        document,
                        context_epoch: context.epoch(),
                        slot,
                    },
                    state: inputs.state(),
                    cursor,
                });
            }
            Step::Complete => {
                return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                    contribution: meter.entry().id,
                }
                .into());
            }
            _ => (),
        }
    }
}
// Formation of a truthful unsupported-influence diagnostic is itself finite
// original output. The borrowed row remains producer-qualified throughout.
fn remaining(
    meter: &Meter,
    row: &surgeist_html_parser::HtmlParseAssociationOccurrence<'_>,
    document: DocumentId,
    cancel: &AtomicBool,
) -> Result<Error, DomError> {
    meter.retain(
        size_of::<DomParsedHtmlStyleNotificationRequirement>(),
        0,
        1,
        cancel,
    )?;
    Ok(Error::Remaining(
        DomParsedHtmlStyleNotificationRequirement::Occurrence {
            contribution: meter.entry().id,
            occurrence: row.id(),
            document,
            kind: row.kind(),
        },
    ))
}
fn dependency(
    meter: &Meter,
    prior: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    next: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    executed: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    cancel: &AtomicBool,
) -> Result<bool, DomError> {
    let mut cursor = meter.entry().report.value().value.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if row.id() == *next => return Ok(false),
            Step::Occurrence(row) if row.id() == *prior => {
                return Ok(!row.requires_processing() || prior == executed);
            }
            Step::Complete => return Ok(false),
            _ => (),
        }
    }
}
impl<'caps> DomParsedHtmlStyleSuccessorPreparation<'_, '_, 'caps> {
    pub fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        &self.selection.qualification
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
    pub fn commit(
        mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSuccessorPublished<'caps>, Error> {
        let new_usage = text_usage(&self.text, self.nonce.as_ref())?;
        let held = self.graph.as_mut().expect("qualified predecessor");
        let Graph::Rendered { request, .. } = held.value().as_ref() else {
            unreachable!("exclusive predecessor")
        };
        let old_usage = text_usage(&request.text, request.nonce.as_ref())?;
        let next_usage = self
            .dom
            .pending_associations
            .usage
            .checked_add(new_usage)?
            .checked_sub(old_usage)?;
        Bank::admit_usage(next_usage, self.dom.limits)?;
        let output_meter = self.meter.fork();
        output_meter.retain(
            size_of::<DomParsedHtmlStyleSuccessorCompletion>(),
            0,
            1,
            cancel,
        )?;
        self.dom.custody_registry.reconcile_releases();
        let admission = self
            .meter
            .admission(self.dom.custody_registry.total(), cancel)?;
        self.meter.step(cancel)?;
        let producer = self
            .store
            .prepare_commit(self.batch)
            .map_err(Error::Cssom)?;
        #[cfg(test)]
        PREPARED.with(|count| count.set(count.get() + 1));
        let mut draft = Draft::new_parsed(self.dom, Arc::new(BatchIdentity), &admission, cancel)?;
        let mut style = draft.style_state_mut(Key::Existing(self.element), cancel)?;
        if style.sheet.as_ref() != Some(&self.old) {
            return Err(DomParsedHtmlStyleError::WrongRequest.into());
        }
        style.sheet = None;
        style.commit(cancel)?;
        let prepared = draft.finish(self.dom, cancel)?;
        let mut publication = self.dom.prepare_publication(prepared, cancel)?;
        publication.close_ordinary_sharing(cancel)?;
        let history = self.dom.history.retain_evictions(
            &publication.history_plan,
            self.dom.custody_registry.ledger(),
            |total| admission.controlled(total, cancel),
            || admission.step(cancel),
        )?;
        drop(draft);
        self.meter.retain(
            0,
            0,
            publication
                .changes
                .len()
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            cancel,
        )?;
        // Precheck arithmetic for the later persistent-to-retirement transfer.
        self.meter.check_peak(custody::Usage::ZERO)?;
        let qualification = self.selection.qualification.clone();
        let old_state = self.dom.state.clone();
        let old_custody = self.dom.state_custody.clone();
        let formation = publication._formation.take();
        admission.step(cancel)?;
        // Sole fence. All allocations/admission and language replies preceded it.
        let (cssom, dead) = producer.publish();
        let dom = self.dom.publish_prepared(publication);
        let old = std::mem::replace(held.value_mut().as_mut(), Graph::Transition);
        let Graph::Rendered {
            request,
            notification,
        } = old
        else {
            unreachable!("qualified predecessor")
        };
        let DomParsedHtmlStyleInlinePolicyRequest {
            qualification: old_q,
            text,
            nonce,
            policy,
            cursor,
            created_sheet,
            ..
        } = request;
        *self.completed = Some(CompletedInvocation {
            qualification: old_q,
            sheet: created_sheet.expect("real creation sealed"),
            profile: cursor.profile(),
            notification,
        });
        let retired_debit = held
            .debit_mut()
            .split(old_usage)
            .expect("actual old text subset");
        self.meter.retire_persistent(retired_debit, old_usage);
        held.debit_mut()
            .absorb(self.formed.detach(new_usage))
            .expect("same original ledger");
        self.meter.entry().control.add_persistent(new_usage);
        self.dom.pending_associations.usage = next_usage;
        **held.value_mut() = Graph::Inline(DomParsedHtmlStyleInlinePolicyRequest {
            qualification: self.selection.qualification,
            text: self.text,
            nonce: self.nonce,
            state: self.selection.state,
            policy,
            cursor: self.selection.cursor,
            created_sheet: None,
        });
        Ok(DomParsedHtmlStyleSuccessorPublished {
            _guard: self.store,
            completion: DomParsedHtmlStyleSuccessorCompletion {
                qualification,
                dom,
                cssom,
                _formation: formation,
                _meter: output_meter,
            },
            retirement: DomParsedHtmlStyleSuccessorRetirement {
                _text: text,
                _nonce: nonce,
                _state: old_state,
                _state_custody: old_custody,
                _history: history,
                _cssom: dead,
                _snapshot: self.snapshot,
                _admission: admission,
                _meter: self.meter,
            },
        })
    }
}
#[cfg(test)]
thread_local! { static PREPARED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
mod tests;
