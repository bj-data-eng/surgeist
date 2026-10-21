//! The authentic original Style prefix and its one external inline decision.
use super::control::Meter;
use super::*;
use crate::parsed_style::*;
use crate::{DocumentString, DomParsedHtmlAssociationLimits, DomParsedHtmlAssociationRuntimeUsage};
use std::sync::{MutexGuard, TryLockError};
use surgeist_html::{
    HtmlCspAssociation, HtmlStyleExecutionProfile, HtmlStyleUpdateAction as Action,
    HtmlStyleUpdateCursor, HtmlStyleUpdateReply as Reply,
};
use surgeist_html_parser::{
    HtmlParseAssociationContextRequirement as Requirement, HtmlParseAssociationContextRole as Role,
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParseHistoryFrame, HtmlParseHistoryStep,
    HtmlParseMetadataDocument, HtmlParseOriginGroup, HtmlParsePendingAssociationStep as Step,
};

mod sheet_join;
pub use sheet_join::{
    DomParsedHtmlStyleSheetCompletion, DomParsedHtmlStyleSheetPreparation,
    DomParsedHtmlStyleSheetPublished, DomParsedHtmlStyleSheetRetirement,
};

type Stored = Option<custody::Held<Box<Graph>>>;
pub(super) enum Graph {
    Awaiting {
        selection: Selection,
        policy: Option<HtmlCspAssociation>,
    },
    Inline(DomParsedHtmlStyleInlinePolicyRequest),
    Transition,
}
pub(super) struct Selection {
    qualification: DomParsedHtmlStyleInitialPolicyRequest,
    state: surgeist_html::HtmlStyleState,
    cursor: HtmlStyleUpdateCursor,
}
impl Graph {
    fn phase(&self) -> DomParsedHtmlStylePhase {
        match self {
            Self::Awaiting { .. } => DomParsedHtmlStylePhase::AwaitingInitialPolicy,
            Self::Inline(request) => match request.cursor.action() {
                Action::CheckInlineCsp => DomParsedHtmlStylePhase::InlinePolicyPending,
                Action::CreateSheet => DomParsedHtmlStylePhase::CreateSheetPending,
                Action::Complete => DomParsedHtmlStylePhase::InlinePolicyBlocked,
                Action::ReadScriptBlocking => DomParsedHtmlStylePhase::ReadScriptBlockingPending,
                Action::ReadRenderBlocking => DomParsedHtmlStylePhase::ReadRenderBlockingPending,
                _ => unreachable!("only the selected inline decision can advance this cursor"),
            },
            Self::Transition => unreachable!("private infallible transition"),
        }
    }
    fn qualification(&self) -> &DomParsedHtmlStyleInitialPolicyRequest {
        match self {
            Self::Awaiting { selection, .. } => &selection.qualification,
            Self::Inline(request) => request.qualification(),
            Self::Transition => unreachable!("private infallible transition"),
        }
    }
}
fn lock(entry: &Entry) -> Result<MutexGuard<'_, Stored>, DomParsedHtmlStyleError> {
    entry.style_prefix.try_lock().map_err(|e| match e {
        TryLockError::WouldBlock => DomParsedHtmlStyleError::Busy {
            contribution: entry.id,
        },
        TryLockError::Poisoned(_) => DomParsedHtmlStyleError::Poisoned {
            contribution: entry.id,
        },
    })
}
/// Observation of the actual stored request. Forgetting this guard can leave it
/// Busy; no path discards or recreates the phase to recover access.
///
/// Inspect an authentic pending contribution without retaining a command hold:
/// ```no_run
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId};
/// fn inspect(dom: &Dom, id: DomParsedHtmlAssociationId) {
///     let view = dom.parsed_html_style_prefix(id).unwrap();
///     let _ = view.phase();
/// }
/// ```
/// A canonical read guard excludes a conflicting owner command:
/// ```compile_fail,E0502
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///            limits: DomParsedHtmlAssociationLimits, cancel: &AtomicBool) {
///     let view = dom.parsed_html_style_prefix(id).unwrap();
///     let _ = dom.begin_parsed_html_style(id, limits, cancel);
///     let _ = view.phase();
/// }
/// ```
pub struct DomParsedHtmlStylePrefixView<'dom> {
    state: MutexGuard<'dom, Stored>,
}
impl DomParsedHtmlStylePrefixView<'_> {
    pub fn phase(&self) -> DomParsedHtmlStylePhase {
        self.state
            .as_ref()
            .map_or(DomParsedHtmlStylePhase::Unstarted, |s| s.value().phase())
    }
    pub fn initial_policy_request(&self) -> Option<&DomParsedHtmlStyleInitialPolicyRequest> {
        self.state.as_ref().map(|s| s.value().qualification())
    }
    pub fn inline_policy_request(&self) -> Option<&DomParsedHtmlStyleInlinePolicyRequest> {
        match self.state.as_ref().map(|s| s.value().as_ref()) {
            Some(Graph::Inline(r)) if r.action() == Action::CheckInlineCsp => Some(r),
            _ => None,
        }
    }
    /// The retained, operation-qualified exclusion. The cursor's immutable
    /// profile permits this reached phase only through the specific exclusion
    /// reply; no script predicate or membership boolean was supplied.
    pub fn script_bookkeeping_exclusion(&self) -> Option<HtmlStyleExecutionProfile> {
        match self.state.as_ref().map(|held| held.value().as_ref()) {
            Some(Graph::Inline(request))
                if request.cursor.action() == Action::ReadRenderBlocking
                    && request.cursor.profile()
                        == HtmlStyleExecutionProfile::NoExecutableJavaScript =>
            {
                Some(request.cursor.profile())
            }
            _ => None,
        }
    }
    pub fn initial_policy_bound(&self) -> bool {
        match self.state.as_ref().map(|s| s.value().as_ref()) {
            Some(Graph::Awaiting { policy, .. }) => policy.is_some(),
            Some(Graph::Inline(_)) => true,
            Some(Graph::Transition) => unreachable!("private infallible transition"),
            None => false,
        }
    }
}
impl Dom {
    pub fn parsed_html_style_prefix(
        &self,
        id: DomParsedHtmlAssociationId,
    ) -> Result<DomParsedHtmlStylePrefixView<'_>, DomParsedHtmlStyleError> {
        if id.stamp.owner() != self.owner {
            return Err(DomError::ForeignOwner.into());
        }
        let key = usize::try_from(id.stamp.revision().get())
            .map_err(|_| DomError::RepresentationExhausted)?;
        let entry = self
            .pending_associations
            .entries
            .get(key)
            .filter(|e| e.id == id)
            .ok_or(crate::DomParsedHtmlAssociationAdmissionError::NotPending {
                contribution: id,
            })?;
        Ok(DomParsedHtmlStylePrefixView {
            state: lock(entry)?,
        })
    }
    pub fn begin_parsed_html_style(
        &mut self,
        id: DomParsedHtmlAssociationId,
        limits: DomParsedHtmlAssociationLimits,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStyleRuntimeHold<'_>, DomParsedHtmlStyleError> {
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
        meter.retain(size_of::<DomParsedHtmlStyleRuntimeHold<'_>>(), 0, 0, cancel)?;
        // A leaked read guard is Busy, even though the former command ended.
        drop(lock(meter.entry())?);
        meter.entry().control.enter();
        Ok(DomParsedHtmlStyleRuntimeHold { dom: self, meter })
    }
}
/// Temporary exclusive command custody; durable graph/limits/work live on the
/// original entry. Drop releases this header, never accepted binding or request.
///
/// First finish the read loan, then bind an explicitly supplied policy to its
/// authentic awaiting qualification. This does not form or evaluate a request:
/// ```no_run
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits};
/// use surgeist_html::HtmlCspAssociation;
/// use std::sync::atomic::AtomicBool;
/// fn bind(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///         limits: DomParsedHtmlAssociationLimits, input: &mut Option<HtmlCspAssociation>,
///         cancel: &AtomicBool) {
///     let view = dom.parsed_html_style_prefix(id).unwrap();
///     let qualification = view.initial_policy_request().unwrap().clone();
///     drop(view);
///     let mut hold = dom.begin_parsed_html_style(id, limits, cancel).unwrap();
///     hold.bind_initial_policy(&qualification, input, cancel).unwrap();
/// }
/// ```
/// A command hold excludes a concurrent canonical request loan:
/// ```compile_fail,E0502
/// use surgeist_dom::{Dom, DomParsedHtmlAssociationId, DomParsedHtmlAssociationLimits};
/// use std::sync::atomic::AtomicBool;
/// fn overlap(dom: &mut Dom, id: DomParsedHtmlAssociationId,
///            limits: DomParsedHtmlAssociationLimits, cancel: &AtomicBool) {
///     let hold = dom.begin_parsed_html_style(id, limits, cancel).unwrap();
///     let _ = dom.parsed_html_style_prefix(id);
///     let _ = hold.step(cancel);
/// }
/// ```
pub struct DomParsedHtmlStyleRuntimeHold<'dom> {
    dom: &'dom mut Dom,
    meter: Meter,
}
impl Drop for DomParsedHtmlStyleRuntimeHold<'_> {
    fn drop(&mut self) {
        self.meter.entry().control.leave();
    }
}
impl DomParsedHtmlStyleRuntimeHold<'_> {
    pub fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.meter.step(cancel)
    }
    pub fn spent_work(&self) -> u64 {
        self.meter.entry().control.spent_work()
    }
    pub fn limits(&self) -> DomParsedHtmlAssociationLimits {
        self.meter.entry().control.limits()
    }
    pub fn admit_runtime(
        &mut self,
        usage: DomParsedHtmlAssociationRuntimeUsage,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.meter.runtime(usage, cancel)
    }
    pub fn advance(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let mut state = lock(self.meter.entry())?;
        match state.as_ref().map(|s| s.value().as_ref()) {
            Some(Graph::Inline(_)) => {
                return Ok(state
                    .as_ref()
                    .expect("stored inline request")
                    .value()
                    .phase());
            }
            Some(Graph::Awaiting { policy: None, .. }) => {
                return Ok(DomParsedHtmlStylePhase::AwaitingInitialPolicy);
            }
            Some(Graph::Awaiting {
                policy: Some(_), ..
            }) => {
                // Binding already succeeded. Late work/output refusal keeps it
                // intact, and no cursor action or policy input is repeated.
                let formed = self
                    .meter
                    .fork()
                    .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
                let qualification = state
                    .as_ref()
                    .expect("awaiting phase")
                    .value()
                    .qualification();
                let (text, nonce) = form_text_nonce(&formed, qualification.occurrence(), cancel)?;
                let usage = text_usage(&text, nonce.as_ref())?;
                let next = self.dom.pending_associations.usage.checked_add(usage)?;
                Bank::admit_usage(next, self.dom.limits)?;
                self.meter.retain(0, 0, 1, cancel)?;
                self.meter.step(cancel)?;
                let incoming = formed.detach(usage);
                let held = state.as_mut().expect("qualified awaiting phase");
                held.debit_mut()
                    .absorb(incoming)
                    .expect("one original pending ledger");
                self.meter.entry().control.add_persistent(usage);
                self.dom.pending_associations.usage = next;
                let graph = held.value_mut();
                let old = std::mem::replace(graph.as_mut(), Graph::Transition);
                let Graph::Awaiting {
                    selection,
                    policy: Some(policy),
                } = old
                else {
                    unreachable!("qualified awaiting phase")
                };
                **graph = Graph::Inline(DomParsedHtmlStyleInlinePolicyRequest {
                    qualification: selection.qualification,
                    text,
                    nonce,
                    state: selection.state,
                    policy,
                    cursor: selection.cursor,
                });
                return Ok(DomParsedHtmlStylePhase::InlinePolicyPending);
            }
            Some(Graph::Transition) => unreachable!("private infallible transition"),
            None => (),
        }
        let formed = self
            .meter
            .fork()
            .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
        let selection = select(self.dom, &formed, cancel)?;
        formed.retain(size_of::<Graph>(), 0, 0, cancel)?;
        let graph = Box::new(Graph::Awaiting {
            selection,
            policy: None,
        });
        let usage = custody::Usage::new(size_of::<Graph>(), 0);
        let next = self.dom.pending_associations.usage.checked_add(usage)?;
        Bank::admit_usage(next, self.dom.limits)?;
        self.meter.step(cancel)?;
        let debit = formed.detach(usage);
        // Only canonical usage is mutable. Active Draft/source loans cannot
        // coexist with this exclusive idle owner; old retired banks only Drop.
        // Future Drafts clone this updated cache, and prepare/remove sum each
        // entry's base payload plus its persistent extent.
        self.meter.entry().control.add_persistent(usage);
        self.dom.pending_associations.usage = next;
        *state = Some(custody::Held::new(graph, debit));
        Ok(DomParsedHtmlStylePhase::AwaitingInitialPolicy)
    }
    /// Accept one actual external observation. Refusal retains the exact input;
    /// no evaluator is called while this exclusive command is live.
    pub fn accept_inline_policy(
        &mut self,
        input: &mut Option<DomParsedHtmlStyleInlinePolicyObservation>,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let mut state = lock(self.meter.entry())?;
        let Some(held) = state.as_mut() else {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        };
        let Graph::Inline(request) = held.value_mut().as_mut() else {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        };
        if request.cursor.action() != Action::CheckInlineCsp {
            return Err(DomParsedHtmlStyleError::DecisionAlreadyAccepted {
                contribution: self.meter.entry().id,
            });
        }
        let observation = input
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::MissingObservation)?;
        // The immutable one-request/no-reset phase binds these exact original
        // identities to its text and nonce. The observation cannot supply or
        // replace either backing, policy, cursor or historical qualification.
        if observation.qualification != request.qualification
            || observation.document != *request.policy.document()
            || observation.generation != *request.policy.generation()
            || observation.policy != *request.policy.policy().id()
            || observation.revision != request.policy.policy().revision()
            || observation.profile.as_ref() != request.policy.policy().profile()
        {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        self.meter.step(cancel)?;
        // This is the one newly durable selected-occurrence decision. All
        // fallible work ends before its same-cursor transition and input take.
        self.meter.retain(0, 0, 1, cancel)?;
        request
            .cursor
            .reply(Reply::InlineAllowed(observation.allowed))?;
        let _accepted = input.take();
        Ok(held.value().phase())
    }
    /// Consume the supported execution owner's explicit script-bookkeeping
    /// exclusion for this exact original operation. No source predicate is
    /// evaluated and no sheet is recreated. The render phase keeps its own
    /// environment/context requirements and the contribution remains Pending.
    pub fn exclude_script_bookkeeping(
        &mut self,
        qualification: &DomParsedHtmlStyleInitialPolicyRequest,
        profile: HtmlStyleExecutionProfile,
        cancel: &AtomicBool,
    ) -> Result<DomParsedHtmlStylePhase, DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let mut stored = lock(self.meter.entry())?;
        let Some(held) = stored.as_mut() else {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        };
        if held.value().qualification() != qualification {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        let Graph::Inline(request) = held.value_mut().as_mut() else {
            return Err(DomParsedHtmlStyleError::WrongPhase {
                phase: held.value().phase(),
            });
        };
        if request.cursor.action() != Action::ReadScriptBlocking {
            return Err(DomParsedHtmlStyleError::WrongPhase {
                phase: held.value().phase(),
            });
        }
        let document = self.dom.snapshot();
        let current = document.document(qualification.document())?;
        if self
            .dom
            .pending_associations
            .document(qualification.document())
            != Some(self.meter.entry().id)
        {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        // The public profile declaration is not authority by itself. Qualify
        // it against this admitted original report and canonical native owner,
        // whose no-engine contract is independent of its browsing context.
        if profile != HtmlStyleExecutionProfile::NoExecutableJavaScript
            || request.cursor.profile() != profile
            || current.metadata().document_environment().scripting()
                != surgeist_html_semantics::HtmlScriptingStatus::Disabled
            || self
                .meter
                .entry()
                .report
                .value()
                .value
                .effective_scripting_mode()
                != surgeist_html::HtmlParserScriptingMode::Disabled
        {
            return Err(DomParsedHtmlStyleError::WrongExecutionProfile { profile });
        }
        let mut successor = request.cursor;
        successor.reply(Reply::ScriptBookkeepingExcluded)?;
        // This is one new durable disposition. All refusal is before changing
        // the existing cursor; graph, policy and prepaid cleanup stay in Entry.
        self.meter.retain(0, 0, 1, cancel)?;
        request.cursor = successor;
        Ok(held.value().phase())
    }
    pub fn bind_initial_policy(
        &mut self,
        request: &DomParsedHtmlStyleInitialPolicyRequest,
        input: &mut Option<HtmlCspAssociation>,
        cancel: &AtomicBool,
    ) -> Result<(), DomParsedHtmlStyleError> {
        self.meter.step(cancel)?;
        let mut state = lock(self.meter.entry())?;
        let Some(held) = state.as_mut() else {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        };
        if held.value().qualification() != request {
            return Err(DomParsedHtmlStyleError::WrongRequest);
        }
        let Graph::Awaiting { policy, .. } = held.value().as_ref() else {
            return Err(DomParsedHtmlStyleError::AlreadyBound);
        };
        if policy.is_some() {
            return Err(DomParsedHtmlStyleError::AlreadyBound);
        }
        let policy = input
            .as_ref()
            .ok_or(DomParsedHtmlStyleError::MissingPolicy)?;
        let layout = policy.retained_layout()?;
        let bytes = layout
            .storage()
            .bytes()
            .checked_sub(size_of::<HtmlCspAssociation>())
            .ok_or(DomError::RepresentationExhausted)?;
        let usage = custody::Usage::new(bytes, layout.storage().utf16_units());
        let formed = self
            .meter
            .fork()
            .constrain_pending(self.dom.pending_associations.usage, self.dom.limits);
        formed.check_peak(custody::Usage::new(
            bytes
                .checked_add(layout.disposal().atomic_bytes())
                .ok_or(DomError::RepresentationExhausted)?,
            usage.units(),
        ))?;
        formed.spend(layout.disposal().work_credits(), cancel)?;
        formed.retain(bytes, usage.units(), 0, cancel)?;
        let next = self.dom.pending_associations.usage.checked_add(usage)?;
        Bank::admit_usage(next, self.dom.limits)?;
        self.meter.step(cancel)?;
        let debit = formed.detach(usage);
        held.debit_mut()
            .absorb(debit)
            .expect("one original pending ledger");
        let Graph::Awaiting { policy, .. } = held.value_mut().as_mut() else {
            unreachable!("qualified awaiting phase")
        };
        *policy = input.take();
        self.meter
            .entry()
            .control
            .reserve_cleanup(layout.disposal().atomic_bytes());
        self.meter.entry().control.add_persistent(usage);
        self.dom.pending_associations.usage = next;
        Ok(())
    }
}
fn append_group(
    report: &HtmlParseReport,
    group: HtmlParseOriginGroup<'_>,
    units: &mut Vec<u16>,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<(), DomParsedHtmlStyleError> {
    let scratch = meter.fork();
    let history = report.history_view();
    let mut frames = Vec::new();
    scratch.reserve(&mut frames, history.len(), cancel)?;
    for _ in 0..history.len() {
        scratch.step(cancel)?;
        frames.push(HtmlParseHistoryFrame::default());
    }
    let mut reader = history
        .cursor(group, &mut frames, None)
        .map_err(crate::DomParsedHtmlAssociationAdmissionError::History)?;
    loop {
        meter.step(cancel)?;
        match reader.advance() {
            HtmlParseHistoryStep::Leaf(leaf) => {
                meter.reserve_units(units, leaf.units().len(), cancel)?;
                for &unit in leaf.units() {
                    meter.step(cancel)?;
                    units.push(unit);
                }
            }
            HtmlParseHistoryStep::Progress => (),
            HtmlParseHistoryStep::Complete => return Ok(()),
            HtmlParseHistoryStep::Cancelled => return Err(DomError::Cancelled.into()),
        }
    }
}
fn select(
    dom: &Dom,
    meter: &Meter,
    cancel: &AtomicBool,
) -> Result<Selection, DomParsedHtmlStyleError> {
    let entry = meter.entry();
    let unsupported = || DomParsedHtmlStyleError::UnsupportedProfile {
        contribution: entry.id,
    };
    let report = &entry.report.value().value;
    let original = report.document().ok_or_else(unsupported)?.root().id();
    if entry.context_basis.is_some() {
        return Err(unsupported());
    }
    let document = DocumentId(entry.id.root);
    dom.snapshot().document(document)?;
    if dom.pending_associations.document(document) != Some(entry.id)
        || dom.style_cssom_roots.get(document.node()).is_some()
    {
        return Err(unsupported());
    }
    let original_document = crate::DomParsedHtmlHistoricalDocument::Constructed(original.clone());
    let mut mapped = false;
    for row in &entry.documents {
        meter.step(cancel)?;
        if row.source == original_document && row.document == document {
            mapped = true;
            break;
        }
    }
    if !mapped {
        return Err(unsupported());
    }
    let mut complete = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match complete.advance() {
            Step::Document(row) => {
                if let surgeist_html_parser::HtmlParseAssociationCaptureDisposition::Unavailable(
                    missing,
                ) = row.disposition()
                {
                    return Err(crate::DomParsedHtmlAssociationAdmissionError::Incomplete {
                        contribution: entry.id,
                        missing,
                    }
                    .into());
                }
            }
            Step::Complete => break,
            _ => (),
        }
    }
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Document(row) => {
                if let surgeist_html_parser::HtmlParseAssociationCaptureDisposition::Unavailable(
                    missing,
                ) = row.disposition()
                {
                    return Err(crate::DomParsedHtmlAssociationAdmissionError::Incomplete {
                        contribution: entry.id,
                        missing,
                    }
                    .into());
                }
            }
            Step::Occurrence(row) if row.requires_processing() => {
                let Kind::StyleUpdate(_) = row.kind() else {
                    return Err(unsupported());
                };
                let HtmlParseMetadataDocument::Owned(source) = row.document() else {
                    return Err(unsupported());
                };
                if source.id() != original {
                    return Err(unsupported());
                }
                let inputs = row.style().ok_or_else(unsupported)?;
                if !inputs.connected() || !inputs.type_eligible() {
                    return Err(unsupported());
                }
                let context = row.context();
                let requirement = context.requirement(Role::InitialPolicy);
                if requirement != Requirement::OfflineUnbound {
                    return Err(DomParsedHtmlStyleError::PolicyContext { requirement });
                }
                let slot = context.slot(Role::InitialPolicy).ok_or_else(unsupported)?;
                // This ordinary admitted owner has no executable-JavaScript engine.
                // Profile selection is independent of browsing context/media;
                // it does not assert the excluded source predicate is false.
                let mut html =
                    HtmlStyleUpdateCursor::new(HtmlStyleExecutionProfile::NoExecutableJavaScript);
                // Proven fresh consumer association state: no prior influencing
                // original operation/publication and no canonical producer binding.
                // HtmlStyleState and a native sheet=None are not this proof.
                for reply in [
                    Reply::Applied,
                    Reply::Connected(inputs.connected()),
                    Reply::TypeEligible(inputs.type_eligible()),
                ] {
                    meter.step(cancel)?;
                    html.reply(reply)?;
                }
                if html.action() != Action::CheckInlineCsp {
                    return Err(unsupported());
                }
                return Ok(Selection {
                    qualification: DomParsedHtmlStyleInitialPolicyRequest {
                        contribution: entry.id,
                        occurrence: row.id(),
                        subject: row.subject().id(),
                        original_document: crate::DomParsedHtmlHistoricalDocument::Constructed(
                            original,
                        ),
                        document,
                        context_epoch: context.epoch(),
                        slot,
                    },
                    state: inputs.state(),
                    cursor: html,
                });
            }
            Step::Occurrence(_) | Step::Skipped => (),
            Step::Complete => return Err(unsupported()),
        }
    }
}

fn text_usage(
    text: &DocumentString,
    nonce: Option<&DocumentString>,
) -> Result<custody::Usage, DomError> {
    let mut bytes = 0usize;
    let mut units = 0usize;
    for text in [Some(text), nonce].into_iter().flatten() {
        let n = text.shared_code_units().capacity();
        bytes = bytes
            .checked_add(size_of::<Vec<u16>>())
            .and_then(|v| v.checked_add(n.checked_mul(2)?))
            .ok_or(DomError::RepresentationExhausted)?;
        units = units
            .checked_add(n)
            .ok_or(DomError::RepresentationExhausted)?;
    }
    Ok(custody::Usage::new(bytes, units))
}
fn form_text_nonce(
    meter: &Meter,
    id: &surgeist_html_parser::HtmlParseAssociationOccurrenceId,
    cancel: &AtomicBool,
) -> Result<(DocumentString, Option<DocumentString>), DomParsedHtmlStyleError> {
    let report = &meter.entry().report.value().value;
    let mut cursor = report.pending_associations();
    loop {
        meter.step(cancel)?;
        match cursor.advance() {
            Step::Occurrence(row) if row.id() == *id => {
                let inputs = row.style().expect("selected original Style");
                let mut units = Vec::new();
                for i in 0..inputs.text_len() {
                    meter.step(cancel)?;
                    append_group(
                        report,
                        inputs.text(i).expect("retained text index"),
                        &mut units,
                        meter,
                        cancel,
                    )?;
                }
                meter.retain(size_of::<Vec<u16>>(), 0, 0, cancel)?;
                let text = DocumentString::from_utf16(units);
                let nonce = if let Some(group) =
                    inputs.attribute(surgeist_html::KnownHtmlAttribute::Nonce)
                {
                    let mut units = Vec::new();
                    append_group(report, group, &mut units, meter, cancel)?;
                    meter.retain(size_of::<Vec<u16>>(), 0, 0, cancel)?;
                    Some(DocumentString::from_utf16(units))
                } else {
                    None
                };
                return Ok((text, nonce));
            }
            Step::Complete => return Err(DomParsedHtmlStyleError::WrongRequest),
            _ => (),
        }
    }
}

#[cfg(test)]
pub(super) mod tests;
