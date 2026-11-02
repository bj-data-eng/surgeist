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
    qualification: DomParsedHtmlStyleQualification,
    joined: Option<JoinedCommits>,
    _formation: Option<source::formation::Formation>,
    _meter: Meter,
}
impl DomParsedHtmlStyleSuccessorCompletion {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
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
    _policy: Option<HtmlCspAssociation>,
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
    policy: Option<HtmlCspAssociation>,
    policy_usage: custody::Usage,
    cleanup_atomic: usize,
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
        self.meter
            .entry()
            .control
            .require_execution(self.meter.entry().id)?;
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
        let old = request
            .created_sheet
            .as_ref()
            .map(|created| created.sheet.clone());
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
        let selection = select_next(self.dom, &formed, request, cancel)?;
        let (policy, policy_usage, cleanup_atomic) = match &selection.origin {
            StylePolicyOrigin::Known { .. } => {
                let (policy, usage, atomic) = form_original_policy(&formed, &selection, cancel)?;
                (Some(policy), usage, atomic)
            }
            StylePolicyOrigin::Offline(_) => (None, custody::Usage::ZERO, 0),
        };
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
            policy,
            policy_usage,
            cleanup_atomic,
            element,
            meter,
            formed,
        })
    }
}
// Complete all source inventory authentication before selecting a candidate.
fn select_next(
    dom: &Dom,
    meter: &Meter,
    old: &DomParsedHtmlStyleInlinePolicyRequest,
    cancel: &AtomicBool,
) -> Result<Selection, Error> {
    let scan = frontier::scan(dom, meter, None, cancel)?;
    if let Some(reason) = scan.missing {
        meter.retain(
            size_of::<DomParsedHtmlStyleNotificationRequirement>(),
            0,
            1,
            cancel,
        )?;
        return Err(Error::Remaining(
            DomParsedHtmlStyleNotificationRequirement::Unavailable {
                contribution: meter.entry().id,
                reason,
            },
        ));
    }
    let row = scan
        .next
        .ok_or(DomParsedHtmlStyleError::UnsupportedProfile {
            contribution: meter.entry().id,
        })?;
    let document = super::super::consume::mapped(
        meter.entry(),
        frontier::original_document(row.document()),
        meter,
        cancel,
    )?;
    if row.kind() != Kind::StyleUpdate(surgeist_html::HtmlStyleTrigger::ParserPopped)
        || row.subject().id() != *old.qualification.subject()
        || document != old.qualification.document()
    {
        return Err(remaining(meter, &row, document, cancel)?);
    }
    if row.prior_style().as_ref() != Some(old.qualification.occurrence()) {
        return Err(DomParsedHtmlStyleError::WrongRequest.into());
    }
    subject::selection(meter, &row, old, cancel).map_err(Into::into)
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
            Step::Fault(fault) => return Err(DomError::ParsedReportUnresolved(fault)),
            Step::Occurrence(row) if row.id() == *next => return Ok(false),
            Step::Occurrence(row) if row.id() == *prior => {
                return Ok(!row.requires_processing() || prior == executed);
            }
            Step::Complete => return Ok(false),
            Step::Document(_) | Step::Occurrence(_) | Step::Skipped => (),
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
        || current.cursor.profile() != previous.profile
        || current.created_sheet.is_some()
    {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    let same_lineage = match (&current.origin, &previous.origin) {
        (StylePolicyOrigin::Offline(a), StylePolicyOrigin::Offline(b)) => a == b,
        (
            StylePolicyOrigin::Known { document: a, .. },
            StylePolicyOrigin::Known { document: b, .. },
        ) => a == b,
        _ => false,
    };
    if !same_lineage {
        return Err(DomParsedHtmlStyleError::WrongRequest);
    }
    let mut cursor = meter.entry().report.value().value.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Fault(fault) => return Err(fault.into()),
            Step::Occurrence(row) if &row.id() == q.occurrence() => {
                if row.kind() != Kind::StyleUpdate(surgeist_html::HtmlStyleTrigger::ParserPopped)
                    || row.subject().id() != *q.subject()
                    || row.context().epoch() != *q.context_epoch()
                    || !origin_matches(&current.origin, &row.context())
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
            Step::Document(_) | Step::Occurrence(_) | Step::Skipped => (),
        }
    }
}
impl<'caps> DomParsedHtmlStyleSuccessorPreparation<'_, '_, 'caps> {
    pub fn qualification(&self) -> &DomParsedHtmlStyleQualification {
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
        let new_usage =
            text_usage(&self.text, self.nonce.as_ref())?.checked_add(self.policy_usage)?;
        let held = self.graph.as_mut().expect("qualified predecessor");
        let request = predecessor(held.value())?;
        let mut old_usage = text_usage(&request.text, request.nonce.as_ref())?;
        if self.policy.is_some() {
            old_usage = old_usage.checked_add(policy_usage(&request.policy)?)?;
        }
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
            origin,
            cursor,
            created_sheet,
            ..
        } = request;
        let outcome = match created_sheet {
            Some(created) => CompletedOutcome::Created {
                sheet: created.sheet,
                notification: issuance.expect("actual rendered predecessor"),
            },
            None => CompletedOutcome::InlinePolicyBlocked,
        };
        *self.completed = Some(CompletedInvocation {
            qualification: old_q,
            profile: cursor.profile(),
            origin,
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
        let (policy, retired_policy) = match self.policy {
            Some(next) => (next, Some(policy)),
            None => (policy, None),
        };
        self.meter
            .entry()
            .control
            .reserve_cleanup(self.cleanup_atomic);
        **held.value_mut() = Graph::Inline(DomParsedHtmlStyleInlinePolicyRequest {
            qualification: self.selection.qualification,
            text: self.text,
            nonce: self.nonce,
            state: self.selection.state,
            origin: self.selection.origin,
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
                _policy: retired_policy,
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
