//! Ordinary-driver-owned Meta CSP service obligations and retained policy values.
use super::*;
use crate::meta_csp::{Account, Caller, Outcome, Service, Storage};
use surgeist_html::security::csp::{
    HtmlCspAssociation, HtmlCspOwner, HtmlCspPolicyId, HtmlCspPolicyRevision, HtmlMetaCspPrepared,
    HtmlMetaCspReceipt, HtmlMetaCspRequest,
};
use surgeist_html::values::DocumentString;

pub(super) struct Policy {
    pub value: Option<HtmlCspAssociation>,
}
/// The position of a policy in the parse's log of them: assigned when the policy is pushed, and
/// valid for as long as the parse retains the log.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PolicyId(usize);
impl PolicyId {
    #[cfg(test)]
    pub(super) fn at_for_test(position: usize) -> Self {
        Self(position)
    }
}
/// The Meta CSP owner's state, the policies it has retained and the transitions that produced
/// them. The parse holds one when the Meta CSP service is enabled, and it is not a registry row:
/// no node owns it, and the registry's rows are not the policies or transitions the owner reads.
pub(super) struct CspLog {
    /// `None` while [`SessionStorage::with_csp`] has taken it out for a use.
    state: Option<Box<State>>,
    policies: RowLog<Box<Policy>>,
    transitions: RowLog<Box<Transition>>,
}
impl CspLog {
    fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        let policies = RowLog::new(control)?;
        let transitions = match RowLog::new(control) {
            Ok(transitions) => transitions,
            Err(error) => {
                policies.release(control, &control.identity(), |_, _| Ok(()))?;
                return Err(error);
            }
        };
        Ok(Self {
            state: None,
            policies,
            transitions,
        })
    }
    pub(super) fn policy(&self, id: PolicyId) -> Option<&Policy> {
        self.policies.row(id.0).map(|policy| &**policy)
    }
    /// The policies the log holds.
    #[cfg(test)]
    pub(super) fn policy_count(&self) -> usize {
        self.policies.len()
    }
    /// Releases the banks and entries of the log, and returns what the owner's state still held:
    /// the storage the account of the owner's graph has acquired, which the registry's release
    /// returns with its own.
    pub(super) fn release(
        self,
        control: &mut ParseControl,
        owner: &ControlIdentity,
    ) -> (Storage, Result<(), SessionError>) {
        let Self {
            state,
            policies,
            transitions,
        } = self;
        let held = state.map_or(Storage::ZERO, |state| state.account.live);
        let mut refused = None;
        keep_first(
            &mut refused,
            policies.release(control, owner, |_, _| Ok(())),
        );
        keep_first(
            &mut refused,
            transitions.release(control, owner, |_, _| Ok(())),
        );
        (held, refused.map_or(Ok(()), Err))
    }
}
pub(super) struct Transition {
    pub value: Option<(
        HtmlMetaCspReceipt,
        HtmlCspAssociation,
        HtmlMetaCspRequest,
        HtmlMetaCspPrepared,
    )>,
    pub _cause: Option<provenance::ConstructionCauseRef>,
    pub _document: usize,
    pub _subject: usize,
}
pub(super) struct Pending {
    document: usize,
    subject: usize,
    content: DocumentString,
    cause: Option<provenance::ConstructionCauseRef>,
}
pub(super) struct State {
    owner: Option<HtmlCspOwner>,
    pub account: Account,
    pending: Option<Pending>,
    initial_base: bool,
    initial: Option<(HtmlCspPolicyId, HtmlCspPolicyRevision)>,
    owner_usage: Storage,
    /// The policies already summed into `rows_usage`. Rows are appended and
    /// never removed or reordered while the log is live, and a policy or
    /// transition is complete when the call that pushed it returns, so a row
    /// below this count never changes its usage.
    policies_scanned: usize,
    /// The transitions already summed into `rows_usage`, as `policies_scanned`.
    transitions_scanned: usize,
    /// What the policies and transitions below the counts retain.
    rows_usage: Storage,
}
fn protocol(
    c: &mut ParseControl,
    e: surgeist_html::security::csp::HtmlCspError<HtmlParseStop>,
) -> SessionError {
    match e {
        surgeist_html::security::csp::HtmlCspError::Interrupted(s) => s.into(),
        surgeist_html::security::csp::HtmlCspError::Protocol(p) => {
            c.csp_protocol_stop(p);
            SessionError::CspProtocol(p)
        }
    }
}
fn native_storage(
    c: &mut ParseControl,
    account: &mut Account,
    bytes: usize,
) -> Result<(), SessionError> {
    let s = Storage {
        bytes,
        units: 0,
        entries: 1,
    };
    let next = account.live.add(s).ok_or(SessionError::WrongToken)?;
    for (category, credits) in [
        (HtmlParseWorkCategory::ReserveByte, bytes as u64),
        (HtmlParseWorkCategory::CopyByte, bytes as u64),
        (HtmlParseWorkCategory::RecordWrite, 1),
        (HtmlParseWorkCategory::ProviderCall, 2),
    ] {
        c.charge(category, credits)?;
    }
    c.admit_atomic_layout(bytes)?;
    s.acquire(c)?;
    account.live = next;
    Ok(())
}
impl SessionStorage {
    pub(super) fn csp_enabled(&self) -> bool {
        self.payloads.facts.csp.is_some()
    }
    fn csp_log(&self, c: &mut ParseControl) -> Result<&CspLog, SessionError> {
        self.payloads
            .facts
            .csp
            .as_ref()
            .ok_or_else(|| latch_registry_row(c))
    }
    fn csp_log_mut(&mut self, c: &mut ParseControl) -> Result<&mut CspLog, SessionError> {
        self.payloads
            .facts
            .csp
            .as_mut()
            .ok_or_else(|| latch_registry_row(c))
    }
    fn with_csp<T>(
        &mut self,
        c: &mut ParseControl,
        f: impl FnOnce(&mut Self, &mut ParseControl, &mut State) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        let mut value = self
            .payloads
            .facts
            .csp
            .as_mut()
            .and_then(|log| log.state.take())
            .ok_or(SessionError::WrongToken)?;
        let result = f(self, c, &mut value);
        let settled = value.account.settle(c);
        let result = match (result, settled) {
            (Ok(out), Ok(())) => Ok(out),
            (Err(error), _) => Err(error),
            (Ok(_), Err(stop)) => Err(stop.into()),
        };
        // Native temporaries have now actually dropped. Reconcile the receipt
        // to the complete graph still retained by original payloads.
        let result = result.and_then(|out| {
            self.reconcile_csp(c, &mut value)?;
            Ok(out)
        });
        match self.payloads.facts.csp.as_mut() {
            Some(log) => {
                log.state = Some(value);
                result
            }
            None => {
                // The state has no log left to return to: credit what it
                // retained, as the release of the log would have.
                let invariant = latch_registry_row(c);
                let _ = value.account.live.release(c);
                Err(invariant)
            }
        }
    }
    /// What the policy retains beyond the log's own accounting of its row.
    fn policy_usage(policy: &Policy) -> Result<Storage, SessionError> {
        let mut usage = Storage {
            bytes: size_of::<Policy>(),
            units: 0,
            entries: 1,
        };
        if let Some(value) = &policy.value {
            let native = Storage::from(
                value
                    .retained_layout()
                    .map_err(SessionError::CspProtocol)?
                    .storage(),
            );
            usage = usage
                .add(
                    native
                        .sub(Storage {
                            bytes: size_of::<HtmlCspAssociation>(),
                            units: 0,
                            entries: 1,
                        })
                        .ok_or(SessionError::WrongToken)?,
                )
                .ok_or(SessionError::WrongToken)?;
        }
        Ok(usage)
    }
    /// What the transition retains beyond the log's own accounting of its row.
    fn transition_usage(transition: &Transition) -> Result<Storage, SessionError> {
        let mut usage = Storage {
            bytes: size_of::<Transition>(),
            units: 0,
            entries: 1,
        };
        if let Some((receipt, retired, request, prepared)) = &transition.value {
            for (layout, inline) in [
                (receipt.retained_layout(), size_of::<HtmlMetaCspReceipt>()),
                (retired.retained_layout(), size_of::<HtmlCspAssociation>()),
                (request.retained_layout(), size_of::<HtmlMetaCspRequest>()),
                (prepared.retained_layout(), size_of::<HtmlMetaCspPrepared>()),
            ] {
                let native = Storage::from(layout.map_err(SessionError::CspProtocol)?.storage());
                usage = usage
                    .add(
                        native
                            .sub(Storage {
                                bytes: inline,
                                units: 0,
                                entries: 1,
                            })
                            .ok_or(SessionError::WrongToken)?,
                    )
                    .ok_or(SessionError::WrongToken)?;
            }
        }
        Ok(usage)
    }
    fn reconcile_csp(&self, c: &mut ParseControl, state: &mut State) -> Result<(), SessionError> {
        let base = state
            .owner_usage
            .add(Storage {
                bytes: size_of::<State>(),
                units: 0,
                entries: 1,
            })
            .ok_or(SessionError::WrongToken)?;
        let log = self.csp_log(c)?;
        let mut rows = state.rows_usage;
        for policy in log.policies.rows_from(c, state.policies_scanned)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            rows = rows
                .add(Self::policy_usage(policy)?)
                .ok_or(SessionError::WrongToken)?;
        }
        for transition in log.transitions.rows_from(c, state.transitions_scanned)? {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            rows = rows
                .add(Self::transition_usage(transition)?)
                .ok_or(SessionError::WrongToken)?;
        }
        let mut live = base.add(rows).ok_or(SessionError::WrongToken)?;
        if let Some(p) = &state.pending {
            let units = p.content.retained_units();
            live = live
                .add(Storage {
                    bytes: units
                        .checked_mul(2)
                        .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                        .ok_or(SessionError::WrongToken)?,
                    units,
                    entries: 1,
                })
                .ok_or(SessionError::WrongToken)?;
        }
        // The account holds less than the graph the policies retain only if
        // the processor released storage it never held.
        let retired = state
            .account
            .live
            .sub(live)
            .ok_or_else(|| crate::meta_csp::accounting_violation(c))?;
        retired.release(c)?;
        state.account.rebaseline(live);
        #[cfg(test)]
        {
            // The incremental sum equals a sum over every row.
            let mut all = Storage::ZERO;
            for (_, policy) in log.policies.iter() {
                all = all
                    .add(Self::policy_usage(policy)?)
                    .ok_or(SessionError::WrongToken)?;
            }
            for (_, transition) in log.transitions.iter() {
                all = all
                    .add(Self::transition_usage(transition)?)
                    .ok_or(SessionError::WrongToken)?;
            }
            assert_eq!(
                rows, all,
                "the incremental policy row usage equals a full sum"
            );
        }
        state.policies_scanned = log.policies.len();
        state.transitions_scanned = log.transitions.len();
        state.rows_usage = rows;
        Ok(())
    }
    pub(super) fn csp_policy(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<&HtmlCspAssociation>, SessionError> {
        let Some(id) = self.csp_policy_id(c, document)? else {
            return Ok(None);
        };
        let policy = self
            .payloads
            .facts
            .csp
            .as_ref()
            .and_then(|log| log.policy(id))
            .ok_or_else(|| latch_registry_row(c))?;
        Ok(policy.value.as_ref())
    }
    pub(super) fn csp_policy_id(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<PolicyId>, SessionError> {
        Ok(self
            .payloads
            .facts
            .documents
            .get(c, document)?
            .and_then(|document| document.meta.csp))
    }
    pub(super) fn csp_pending(&self) -> bool {
        self.payloads
            .facts
            .csp
            .as_ref()
            .and_then(|log| log.state.as_ref())
            .is_some_and(|state| state.pending.is_some())
    }
    pub(super) fn ensure_csp_policy(
        &mut self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<(), SessionError> {
        if !self.csp_enabled() || self.csp_policy(c, document)?.is_some() {
            return Ok(());
        }
        self.with_csp(c, |this, c, state| {
            let association = state
                .owner
                .as_mut()
                .ok_or(SessionError::WrongToken)?
                .initial_document_with(&mut |a| state.account.admit(c, a, Caller::Parser))
                .map_err(|e| protocol(c, e))?;
            state.account.settle(c)?;
            if document == 0 {
                payload_write::<Option<(HtmlCspPolicyId, HtmlCspPolicyRevision)>>(c, 1)?;
                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                state.initial = Some((
                    association.policy().id().clone(),
                    association.policy().revision(),
                ));
            }
            this.csp_log_mut(c)?.policies.admit(c, 1)?;
            native_storage(c, &mut state.account, size_of::<Policy>())?;
            let index = PolicyId(
                this.csp_log_mut(c)?
                    .policies
                    .push(Box::new(Policy {
                        value: Some(association),
                    }))
                    .map_err(|_| latch_registry_row(c))?,
            );
            let doc = this.document_slot(c, document)?;
            payload_write::<Option<PolicyId>>(
                c,
                if state.initial_base && document == 0 {
                    2
                } else {
                    1
                },
            )?;
            this.meta_history_mut(doc)
                .map_err(|_| latch_registry_row(c))?
                .csp = Some(index);
            if state.initial_base && document == 0 {
                this.meta_history_mut(doc)
                    .map_err(|_| latch_registry_row(c))?
                    .base_csp = Some(index);
            }
            Ok(())
        })
    }
    pub(super) fn defer_csp(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        document: usize,
        content: DocumentString,
    ) -> Result<bool, SessionError> {
        if !self.csp_enabled() {
            return Ok(false);
        }
        if self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .first_child()
            .is_some()
        {
            return Err(SessionError::WrongToken);
        }
        if self.meta_policy_missing(c, document)?.is_some() {
            return Ok(false);
        }
        self.ensure_csp_policy(c, document)?;
        let cause = self.associated_cause(c)?;
        self.with_csp(c, |_, c, state| {
            if state.pending.is_some() {
                return Err(SessionError::WrongToken);
            }
            let (content, bytes, units) = c.csp_document_string(&content)?;
            state.account.live = state
                .account
                .live
                .add(Storage {
                    bytes,
                    units,
                    entries: 1,
                })
                .ok_or(SessionError::WrongToken)?;
            payload_write::<Option<Pending>>(c, 1)?;
            state.pending = Some(Pending {
                document,
                subject: node,
                content,
                cause,
            });
            Ok(true)
        })
    }
    pub(super) fn csp_base_current(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<bool, SessionError> {
        let Some(log) = self.payloads.facts.csp.as_ref() else {
            return Ok(true);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(state) = log.state.as_ref() else {
            return Err(SessionError::WrongToken);
        };
        Ok(state.initial_base
            && self.csp_policy(c, document)?.is_some_and(|p| {
                state
                    .initial
                    .as_ref()
                    .is_some_and(|(id, r)| p.policy().id() == id && p.policy().revision() == *r)
            }))
    }
}
impl ParserArenaSession {
    pub(crate) fn enable_meta_csp(
        &mut self,
        c: &mut ParseControl,
        initial_base: bool,
    ) -> Result<(), SessionError> {
        let s = self.active_mut()?;
        s.require(c)?;
        if s.csp_enabled() {
            return Err(SessionError::WrongToken);
        }
        // The log is installed before the state is allocated, so a refusal of the
        // allocation leaves the log to the release of the parse.
        let log = CspLog::new(c)?;
        s.payloads.facts.csp = Some(log);
        let mut account = Account::default();
        payload_write::<Option<Box<State>>>(c, 1)?;
        native_storage(c, &mut account, size_of::<State>())?;
        s.csp_log_mut(c)?.state = Some(Box::new(State {
            owner: None,
            account,
            pending: None,
            initial_base,
            initial: None,
            owner_usage: Storage::ZERO,
            policies_scanned: 0,
            transitions_scanned: 0,
            rows_usage: Storage::ZERO,
        }));
        s.with_csp(c, |_, c, state| {
            let owner = HtmlCspOwner::new_with(&mut |a| state.account.admit(c, a, Caller::Parser))
                .map_err(|e| protocol(c, e))?;
            state.owner_usage = state.account.last;
            state.owner = Some(owner);
            Ok(())
        })?;
        s.ensure_csp_policy(c, 0)
    }
}
impl SessionStorage {
    pub(super) fn service_meta_csp(
        &mut self,
        c: &mut ParseControl,
        service: &mut dyn Service,
    ) -> Result<(), SessionError> {
        self.with_csp(c, |s, c, state| {
            let pending = state.pending.take().ok_or(SessionError::WrongToken)?;
            let result = (|| {
                s.csp_policy_id(c, pending.document)?
                    .ok_or(SessionError::WrongToken)?;
                let original = s
                    .csp_policy(c, pending.document)?
                    .ok_or(SessionError::WrongToken)?;
                // The original retained policy remains history; this protocol
                // occurrence owns its own conservatively admitted association.
                let layout = original
                    .retained_layout()
                    .map_err(|_| SessionError::WrongToken)?;
                let storage = Storage::from(layout.storage());
                layout
                    .disposal()
                    .charge_with_calls(1, c)
                    .map_err(|error| match error {
                        surgeist_html::accounting::HtmlChargedError::Stopped(stop) => {
                            SessionError::from(stop)
                        }
                        surgeist_html::accounting::HtmlChargedError::Faulted(_) => {
                            SessionError::from(c.charge_overflow_stop())
                        }
                    })?;
                storage.acquire(c)?;
                state.account.live.bytes = state
                    .account
                    .live
                    .bytes
                    .checked_add(storage.bytes)
                    .ok_or(SessionError::WrongToken)?;
                state.account.live.units = state
                    .account
                    .live
                    .units
                    .checked_add(storage.units)
                    .ok_or(SessionError::WrongToken)?;
                state.account.live.entries = state
                    .account
                    .live
                    .entries
                    .checked_add(storage.entries)
                    .ok_or(SessionError::WrongToken)?;
                let mut current = original.clone();
                let profile = service.profile();
                let request: HtmlMetaCspRequest = state
                    .owner
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .request_meta_insertion_with(
                        &current,
                        profile,
                        pending.content.clone(),
                        &mut |a| state.account.admit(c, a, Caller::Parser),
                    )
                    .map_err(|e| protocol(c, e.error))?;
                let prepared = match service.call(c, &mut state.account, &request)? {
                    Outcome::Completed(p) => p,
                    Outcome::Unavailable(reason) => {
                        s.meta_missing(
                            c,
                            pending.document,
                            pending.subject,
                            meta::HtmlParseMetaMissingKind::CspProcessing(reason),
                        )?;
                        return Ok(());
                    }
                    Outcome::Terminal => return Err(SessionError::CspService),
                };
                // All receivers and payload backing precede the policy fence.
                s.csp_log_mut(c)?.policies.admit(c, 1)?;
                native_storage(c, &mut state.account, size_of::<Policy>())?;
                let next_policy = PolicyId(
                    s.csp_log_mut(c)?
                        .policies
                        .push(Box::new(Policy { value: None }))
                        .map_err(|_| latch_registry_row(c))?,
                );
                s.csp_log_mut(c)?.transitions.admit(c, 1)?;
                native_storage(c, &mut state.account, size_of::<Transition>())?;
                let transition_cause = pending
                    .cause
                    .as_ref()
                    .map(|cause| {
                        s.evidence
                            .as_mut()
                            .ok_or(SessionError::WrongToken)?
                            .copy_cause_ref(c, cause)
                    })
                    .transpose()?;
                let transition = s
                    .csp_log_mut(c)?
                    .transitions
                    .push(Box::new(Transition {
                        value: None,
                        _cause: transition_cause,
                        _document: pending.document,
                        _subject: pending.subject,
                    }))
                    .map_err(|_| latch_registry_row(c))?;
                let doc = s.document_slot(c, pending.document)?;
                payload_write::<Option<PolicyId>>(c, 1)?;
                let publication = state
                    .owner
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .prepare_publication_with(&mut current, request, prepared, &mut |a| {
                        state.account.admit(c, a, Caller::Parser)
                    })
                    .map_err(|e| protocol(c, e.error))?;
                let completion = publication.publish();
                let log = s.csp_log_mut(c)?;
                let Some(policy) = log.policies.row_mut(next_policy.0) else {
                    return Err(latch_registry_row(c));
                };
                policy.value = Some(current);
                let Some(transition) = log.transitions.row_mut(transition) else {
                    return Err(latch_registry_row(c));
                };
                transition.value = Some(completion.into_parts());
                s.meta_history_mut(doc)
                    .map_err(|_| latch_registry_row(c))?
                    .csp = Some(next_policy);
                Ok(())
            })();
            if result.is_err() {
                state.pending = Some(pending)
            }
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::construction::effects::tests::{graph, request};
    use crate::construction::tests::storage_zero;
    use surgeist_html::security::csp::HtmlCspOwner;

    #[test]
    fn a_meta_csp_log_dropped_during_its_use_reports_the_registry_row() {
        let request = request("<body><div id=a>");
        let (mut c, mut session, _proof, _nodes) = graph(&request);
        let storage = session.active_mut().unwrap();
        storage.payloads.facts.csp = Some(CspLog::new(&mut c).unwrap());
        let mut account = Account::default();
        native_storage(&mut c, &mut account, size_of::<State>()).unwrap();
        storage.payloads.facts.csp.as_mut().unwrap().state = Some(Box::new(State {
            owner: None,
            account,
            pending: None,
            initial_base: false,
            initial: None,
            owner_usage: Storage::ZERO,
            policies_scanned: 0,
            transitions_scanned: 0,
            rows_usage: Storage::ZERO,
        }));
        let result = storage.with_csp(&mut c, |this, c, _| {
            // The state is out for its use, and the log it returns to is released.
            let owner = c.identity();
            let log = this.payloads.facts.csp.take().unwrap();
            let (held, released) = log.release(c, &owner);
            released.unwrap();
            assert_eq!(held, Storage::ZERO);
            Ok(())
        });
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(result.err(), Some(SessionError::Stop(invariant)));
        assert_eq!(c.stop(), Some(invariant));
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    /// Installs an empty log and a state that has retained nothing, as the service enables them.
    fn install_log(storage: &mut SessionStorage, c: &mut ParseControl) {
        storage.payloads.facts.csp = Some(CspLog::new(c).unwrap());
        let mut account = Account::default();
        native_storage(c, &mut account, size_of::<State>()).unwrap();
        storage.payloads.facts.csp.as_mut().unwrap().state = Some(Box::new(State {
            owner: None,
            account,
            pending: None,
            initial_base: false,
            initial: None,
            owner_usage: Storage::ZERO,
            policies_scanned: 0,
            transitions_scanned: 0,
            rows_usage: Storage::ZERO,
        }));
    }

    /// The reconciliation that ends every use of the owner reads the rows pushed since the last
    /// one. A count of rows already read that is past the end of the log names rows the log never
    /// held, so it is the registry-row defect and not an empty read.
    #[test]
    fn a_scanned_count_past_the_log_is_the_registry_row_defect() {
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        for transitions in [false, true] {
            let request = request("<body><div id=a>");
            let (mut c, mut session, _proof, _nodes) = graph(&request);
            let storage = session.active_mut().unwrap();
            install_log(storage, &mut c);
            // The log holds no row, so a count of one is past its end.
            let result = storage.with_csp(&mut c, |_, _, state| {
                if transitions {
                    state.transitions_scanned = 1;
                } else {
                    state.policies_scanned = 1;
                }
                Ok(())
            });
            assert_eq!(
                result.err(),
                Some(SessionError::Stop(invariant)),
                "transitions: {transitions}"
            );
            assert_eq!(c.stop(), Some(invariant));
            session.release(&mut c).unwrap();
            storage_zero(&c);
        }
    }

    /// A reconciliation reads one row for each policy and each transition pushed since the last,
    /// and no other: after one policy and one transition, two rows. The count is taken inside
    /// the use, after the admissions, so only the reconciliation's own visits are counted.
    #[test]
    fn a_reconciliation_visits_exactly_the_policies_and_transitions_pushed_since_the_last() {
        let request = request("<body><div id=a>");
        let (mut c, mut session, _proof, _nodes) = graph(&request);
        let storage = session.active_mut().unwrap();
        install_log(storage, &mut c);
        let visits = |c: &ParseControl| c.usage().work(HtmlParseWorkCategory::RecordVisit);
        let mut at_end_of_use = 0;
        storage
            .with_csp(&mut c, |this, c, state| {
                let log = this.csp_log_mut(c)?;
                log.policies.admit(c, 1)?;
                native_storage(c, &mut state.account, size_of::<Policy>())?;
                log.policies
                    .push(Box::new(Policy { value: None }))
                    .map_err(|_| latch_registry_row(c))?;
                let log = this.csp_log_mut(c)?;
                log.transitions.admit(c, 1)?;
                native_storage(c, &mut state.account, size_of::<Transition>())?;
                log.transitions
                    .push(Box::new(Transition {
                        value: None,
                        _cause: None,
                        _document: 0,
                        _subject: 0,
                    }))
                    .map_err(|_| latch_registry_row(c))?;
                at_end_of_use = visits(c);
                Ok(())
            })
            .unwrap();
        assert_eq!(visits(&c) - at_end_of_use, 2);
        // The rows are now read: a use that pushes nothing visits none.
        let before = visits(&c);
        storage.with_csp(&mut c, |_, _, _| Ok(())).unwrap();
        assert_eq!(visits(&c), before);
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    /// The plain elements of a parse own registry rows, and the policies and transitions of the
    /// Meta CSP owner are not among them: after `elements` plain elements are created, the
    /// reconciliation of the owner's storage that ends every use of it has nothing of its own to
    /// read. The oracle is zero record visits at every count, because the owner's own rows are
    /// the only ones it reads and none was added. A reconciliation that walks the registry reads
    /// every row the elements added; the fixture creates seven per element.
    #[test]
    fn csp_reconciliation_visits_no_row_for_the_plain_elements_of_the_parse() {
        use crate::parser::tokenizer::{
            states::HtmlTokenizerState, tests::LexerFixture, token::Event,
        };
        let visits = |elements: usize| {
            let request =
                crate::construction::base::tests::request(&"<span>".repeat(elements.max(1)));
            let (mut c, mut session, _proof) =
                crate::construction::tests::fresh_script_owner(&request);
            session.enable_meta_csp(&mut c, false).unwrap();
            let mut lexer =
                LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c)
                    .unwrap();
            for _ in 0..elements {
                let mut incoming = loop {
                    match lexer.next(&mut c, false).unwrap().unwrap() {
                        Event::Token(event) => {
                            let token = event.into_token(&mut c).unwrap();
                            if let crate::parser::tokenizer::token::TokenKind::Tag(tag) = token.kind
                            {
                                break Some(tag);
                            }
                            crate::construction::release_token_kind(&mut Some(token.kind), &mut c)
                                .unwrap();
                        }
                        event => event.release(&mut c).unwrap(),
                    }
                };
                session.allocate_html_tag(&mut c, &mut incoming).unwrap();
            }
            let before = c.usage().work(HtmlParseWorkCategory::RecordVisit);
            session
                .active_mut()
                .unwrap()
                .with_csp(&mut c, |_, _, _| Ok(()))
                .unwrap();
            let visits = c.usage().work(HtmlParseWorkCategory::RecordVisit) - before;
            lexer.release(&mut c).unwrap();
            session.release(&mut c).unwrap();
            storage_zero(&c);
            visits
        };
        for elements in [0, 16, 64] {
            assert_eq!(visits(elements), 0, "{elements} plain elements");
        }
    }

    /// The stop for an owner call that the account cannot cover, with the
    /// account an association was admitted to replaced by an empty one.
    fn uncovered_owner_call() -> Option<HtmlParseStop> {
        let request = request("<body><div id=a>");
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut allow =
            |_: surgeist_html::security::csp::HtmlCspLayout| Ok::<(), std::convert::Infallible>(());
        let authority =
            surgeist_html::security::csp::HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(17, &mut allow).unwrap();
        let mut covered = Account::default();
        let mut owner =
            HtmlCspOwner::new_with(&mut |a| covered.admit(&mut c, a, Caller::Parser)).unwrap();
        let association = owner
            .initial_document_with(&mut |a| covered.admit(&mut c, a, Caller::Parser))
            .unwrap();
        // An account that never held the association cannot be asked to
        // transfer its storage into the request.
        let mut short = Account::default();
        let refused = owner.request_meta_insertion_with(
            &association,
            &profile,
            "img-src 'none'".into(),
            &mut |a| short.admit(&mut c, a, Caller::Parser),
        );
        assert!(refused.is_err());
        c.stop()
    }

    /// An owner call the parser itself makes, that its own account cannot
    /// cover, disagrees with what the parser charged. The processor is not
    /// involved, so it is not a failed processing.
    #[test]
    fn an_owner_call_the_parser_account_cannot_cover_is_a_capacity_mismatch() {
        assert_eq!(
            uncovered_owner_call(),
            Some(HtmlParseStop::InternalInvariant(
                crate::parser::control::HtmlParseInvariant::CapacityMismatch
            ))
        );
    }

    /// The same call made through the processor's admission callback is the
    /// processor's accounting failing.
    #[test]
    fn a_processor_call_the_account_cannot_cover_is_a_failed_processing() {
        let request = request("<body><div id=a>");
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut allow =
            |_: surgeist_html::security::csp::HtmlCspLayout| Ok::<(), std::convert::Infallible>(());
        let authority =
            surgeist_html::security::csp::HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(17, &mut allow).unwrap();
        let mut covered = Account::default();
        let mut owner =
            HtmlCspOwner::new_with(&mut |a| covered.admit(&mut c, a, Caller::Parser)).unwrap();
        let association = owner
            .initial_document_with(&mut |a| covered.admit(&mut c, a, Caller::Parser))
            .unwrap();
        let mut short = Account::default();
        let refused = owner.request_meta_insertion_with(
            &association,
            &profile,
            "img-src 'none'".into(),
            &mut |a| short.admit(&mut c, a, Caller::Processor),
        );
        assert!(refused.is_err());
        assert_eq!(c.stop(), Some(HtmlParseStop::CspProcessing));
    }

    /// Once the processor has admitted or released through the account, a gap
    /// that the parser's next owner call finds is the processor's: the account
    /// differs from the graph only by what the processor did.
    #[test]
    fn a_parser_call_after_the_processor_touched_the_account_fails_the_processing() {
        let request = request("<body><div id=a>");
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut allow =
            |_: surgeist_html::security::csp::HtmlCspLayout| Ok::<(), std::convert::Infallible>(());
        let authority =
            surgeist_html::security::csp::HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
        let profile = authority.profile_with(17, &mut allow).unwrap();
        let mut covered = Account::default();
        let mut owner =
            HtmlCspOwner::new_with(&mut |a| covered.admit(&mut c, a, Caller::Parser)).unwrap();
        let association = owner
            .initial_document_with(&mut |a| covered.admit(&mut c, a, Caller::Parser))
            .unwrap();
        let mut short = Account::default();
        short
            .admit(
                &mut c,
                surgeist_html::security::csp::HtmlCspLayout::processing(
                    0,
                    surgeist_html::security::csp::HtmlCspStorage::default(),
                    surgeist_html::security::csp::HtmlCspDisposalWork::default(),
                    0,
                )
                .unwrap(),
                Caller::Processor,
            )
            .unwrap();
        let refused = owner.request_meta_insertion_with(
            &association,
            &profile,
            "img-src 'none'".into(),
            &mut |a| short.admit(&mut c, a, Caller::Parser),
        );
        assert!(refused.is_err());
        assert_eq!(c.stop(), Some(HtmlParseStop::CspProcessing));
    }
}
