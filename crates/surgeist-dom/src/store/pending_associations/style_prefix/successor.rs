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
    joined: Option<JoinedCommits>,
    _formation: Option<source::formation::Formation>,
    _meter: Meter,
}
impl DomParsedHtmlStyleSuccessorCompletion {
    pub fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        &self.qualification
    }
    pub fn removal(&self) -> DomParsedHtmlStyleSuccessorRemoval<'_> {
        match &self.joined {
            Some(joined) => DomParsedHtmlStyleSuccessorRemoval::Removed {
                dom: &joined.dom,
                cssom: &joined.cssom,
            },
            None => DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet,
        }
    }
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        DomParsedHtmlStylePhase::InlinePolicyPending
    }
}
impl std::fmt::Debug for DomParsedHtmlStyleSuccessorCompletion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomParsedHtmlStyleSuccessorCompletion")
            .field("qualification", self.qualification())
            .field("removal", &self.removal())
            .finish()
    }
}
/// Actual displaced backing stays alive through the complete publication fence.
/// Graphs drop before the original admission and Meter which charged them.
pub struct DomParsedHtmlStyleSuccessorRetirement {
    _text: DocumentString,
    _nonce: Option<DocumentString>,
    _joined: Option<JoinedRetirement>,
    _meter: Meter,
}
pub struct DomParsedHtmlStyleSuccessorPublished<'caps> {
    _guard: Option<std::sync::MutexGuard<'caps, CssomStore>>,
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
    joined: Option<JoinedPreparation<'caps>>,
    selection: Selection,
    text: DocumentString,
    nonce: Option<DocumentString>,
    element: crate::NodeId,
    meter: Meter,
    formed: Meter,
}
struct JoinedCommits {
    dom: DomCommit,
    cssom: CssomCommit,
}
struct JoinedPreparation<'caps> {
    store: std::sync::MutexGuard<'caps, CssomStore>,
    snapshot: CssomSnapshot,
    batch: CssomBatch,
    old: DomStyleSheet,
}
struct JoinedRetirement {
    _state: Arc<State>,
    _state_custody: transaction::StateCustody,
    _history: history::Retirement,
    _cssom: CssomCommitRetirement,
    _snapshot: CssomSnapshot,
    _admission: control::Admission,
}
fn predecessor(
    graph: &Graph,
) -> Result<&DomParsedHtmlStyleInlinePolicyRequest, DomParsedHtmlStyleError> {
    match graph {
        Graph::Rendered { request, .. } if request.created_sheet.is_some() => Ok(request),
        Graph::Inline(request)
            if request.cursor.action() == Action::Complete && request.created_sheet.is_none() =>
        {
            Ok(request)
        }
        _ => Err(DomParsedHtmlStyleError::WrongPhase {
            phase: graph.phase(),
        }),
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    /// Only an actual created-sheet predecessor needs a producer owner. A
    /// completed CSP refusal proves known-null removal without a Store loan.
    pub fn successor_owner_document(
        &self,
        cancel: &AtomicBool,
    ) -> Result<Option<DocumentId>, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let value = graph
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?
            .value();
        let request = predecessor(value)?;
        Ok(request
            .created_sheet
            .as_ref()
            .map(|_| request.qualification.document()))
    }
}
impl<'dom> DomParsedHtmlStyleRuntimeHold<'dom> {
    pub fn prepare_successor<'hold, 'caps>(
        &'hold mut self,
        cap: Option<&'caps DomStyleCssomRoot>,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleSuccessorPreparation<'hold, 'dom, 'caps>, Error> {
        self.meter.step(cancel)?;
        let graph = lock(self.meter.entry())?;
        let value = graph
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::WrongRequest)?
            .value();
        let request = predecessor(value)?;
        let completed = completed_lock(self.meter.entry())?;
        if completed.is_some() {
            return Err(DomParsedHtmlStyleError::UnsupportedProfile {
                contribution: self.meter.entry().id,
            }
            .into());
        }
        let old = request.created_sheet.clone();
        let root = request.qualification.document().node();
        // This validates the optional capability against the authenticated
        // outcome, never inferring no-sheet from a missing caller owner.
        let owner = match (old.as_ref(), cap) {
            (Some(_), Some(cap)) => {
                if cap.root() != root
                    || self
                        .dom
                        .style_cssom_roots
                        .get(root)
                        .is_none_or(|bank| !bank.same_owner(cap))
                {
                    return Err(Error::MismatchedOwner { root });
                }
                Some(cap)
            }
            (Some(_), None) => return Err(Error::MissingOwner { root }),
            (None, None) => None,
            (None, Some(_)) => return Err(DomParsedHtmlStyleError::WrongRequest.into()),
        };
        let formed = self
            .meter
            .fork()
            .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
        let selection = select_next(&formed, request, cancel)?;
        let (text, nonce) = form_text_nonce(&formed, selection.qualification.occurrence(), cancel)?;
        let mut element = None;
        for binding in self.meter.entry().bindings.iter() {
            self.meter.step(cancel)?;
            if binding.source == *request.qualification.subject() {
                element = Some(binding.node);
                break;
            }
        }
        let element = element.ok_or(DomParsedHtmlStyleError::WrongRequest)?;
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
            || style.state().sheet.as_ref() != old.as_ref()
            || old
                .as_ref()
                .is_some_and(|sheet| sheet.association().element() != element)
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
        let joined = if let Some(cap) = owner {
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
            let old = old.expect("authenticated created predecessor");
            batch.remove_sheet(old.id()).map_err(Error::Cssom)?;
            Some(JoinedPreparation {
                store,
                snapshot,
                batch,
                old,
            })
        } else {
            None
        };
        Ok(DomParsedHtmlStyleSuccessorPreparation {
            graph,
            completed,
            dom: &mut self.dom,
            runtime: &self.meter,
            joined,
            selection,
            text,
            nonce,
            element,
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
// Admission to the existing creation editor is based on the current sealed
// occurrence and genuine predecessor outcome, never empty membership alone.
pub(super) fn qualify_creation(
    meter: &Meter,
    current: &DomParsedHtmlStyleInlinePolicyRequest,
    previous: &CompletedInvocation,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlStyleError> {
    let q = current.qualification();
    if q == &previous.qualification
        || q.subject() != previous.qualification.subject()
        || q.document() != previous.qualification.document()
        || q.original_document() != previous.qualification.original_document()
        || q.slot() != previous.qualification.slot()
        || current.cursor.profile() != previous.profile
        || current.created_sheet.is_some()
    {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    let mut cursor = meter.entry().report.value().value.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if &row.id() == q.occurrence() => {
                if row.kind() != Kind::StyleUpdate(surgeist_html::HtmlStyleTrigger::ParserPopped)
                    || row.subject().id() != *q.subject()
                    || row.context().epoch() != *q.context_epoch()
                    || row.context().slot(Role::InitialPolicy).as_ref() != Some(q.slot())
                    || row.context().input_epoch(Role::InitialPolicy) != *q.slot().epoch()
                    || row
                        .style()
                        .is_none_or(|inputs| inputs.state() != current.state)
                {
                    return Err(DomParsedHtmlStyleError::WrongRequest);
                }
                for prior in [row.prior_document(), row.prior_style()]
                    .into_iter()
                    .flatten()
                {
                    if !dependency(
                        meter,
                        &prior,
                        &row.id(),
                        previous.qualification.occurrence(),
                        cancel,
                    )? {
                        return Err(DomParsedHtmlStyleError::WrongRequest);
                    }
                }
                return Ok(());
            }
            Step::Complete => return Err(DomParsedHtmlStyleError::WrongRequest),
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
        let request = predecessor(held.value())?;
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
        self.meter.check_peak(custody::Usage::ZERO)?;
        let qualification = self.selection.qualification.clone();
        let (guard, joined, retired, formation) = if let Some(mut joined) = self.joined {
            self.dom.custody_registry.reconcile_releases();
            let admission = self
                .meter
                .admission(self.dom.custody_registry.total(), cancel)?;
            self.meter.step(cancel)?;
            let producer = joined
                .store
                .prepare_commit(joined.batch)
                .map_err(Error::Cssom)?;
            #[cfg(test)]
            PREPARED.with(|count| count.set(count.get() + 1));
            let mut draft =
                Draft::new_parsed(self.dom, Arc::new(BatchIdentity), &admission, cancel)?;
            let mut style = draft.style_state_mut(Key::Existing(self.element), cancel)?;
            if style.sheet.as_ref() != Some(&joined.old) {
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
            let state = self.dom.state.clone();
            let custody = self.dom.state_custody.clone();
            let formation = publication._formation.take();
            admission.step(cancel)?;
            // The joined branch's sole fence; common controller moves below
            // are prequalified and infallible. No backing is disposed here.
            let (cssom, dead) = producer.publish();
            let dom = self.dom.publish_prepared(publication);
            (
                Some(joined.store),
                Some(JoinedCommits { dom, cssom }),
                Some(JoinedRetirement {
                    _state: state,
                    _state_custody: custody,
                    _history: history,
                    _cssom: dead,
                    _snapshot: joined.snapshot,
                    _admission: admission,
                }),
                formation,
            )
        } else {
            // No CSSOM object or canonical tree changed. Only the original
            // Entry/request transition publishes; there is no dummy commit.
            self.meter.step(cancel)?;
            (None, None, None, None)
        };
        let old = std::mem::replace(held.value_mut().as_mut(), Graph::Transition);
        let (request, issuance) = match old {
            Graph::Rendered {
                request,
                notification,
            } => (request, Some(notification)),
            Graph::Inline(request) => (request, None),
            _ => unreachable!("authenticated completed invocation"),
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
        let outcome = match created_sheet {
            Some(sheet) => CompletedOutcome::Created {
                sheet,
                notification: issuance.expect("actual rendered predecessor"),
            },
            None => CompletedOutcome::InlinePolicyBlocked,
        };
        *self.completed = Some(CompletedInvocation {
            qualification: old_q,
            profile: cursor.profile(),
            outcome,
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
            _guard: guard,
            completion: DomParsedHtmlStyleSuccessorCompletion {
                qualification,
                joined,
                _formation: formation,
                _meter: output_meter,
            },
            retirement: DomParsedHtmlStyleSuccessorRetirement {
                _text: text,
                _nonce: nonce,
                _joined: retired,
                _meter: self.meter,
            },
        })
    }
}
#[cfg(test)]
thread_local! { static PREPARED: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
#[cfg(test)]
mod tests;
