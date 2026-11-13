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
    /// The registry rows already summed into `rows_usage`. Rows are appended
    /// and never removed or reordered while the registry is live, and a
    /// policy or transition row is complete when the call that pushed it
    /// returns, so a row below this count never changes its usage.
    rows_scanned: usize,
    /// What the policy and transition rows below `rows_scanned` retain.
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
    fn csp_slot(&self) -> Option<usize> {
        self.csp_slot
    }
    fn with_csp<T>(
        &mut self,
        c: &mut ParseControl,
        f: impl FnOnce(&mut Self, &mut ParseControl, &mut State) -> Result<T, SessionError>,
    ) -> Result<T, SessionError> {
        let slot = self.csp_slot().ok_or(SessionError::WrongToken)?;
        let RetainedPayload::CspState(value) = self.payloads.row_mut(c, slot)? else {
            return Err(latch_registry_row(c));
        };
        let mut value = value.take().ok_or(SessionError::WrongToken)?;
        let result = f(self, c, &mut value);
        let settled = value.account.settle(c);
        let result = match (result, settled) {
            (Ok(out), Ok(())) => Ok(out),
            (Err(error), _) => Err(error),
            (Ok(_), Err(stop)) => Err(stop.into()),
        };
        // Native temporaries have now actually dropped. Reconcile the receipt
        // to the complete graph still retained by original payloads.
        // A row the closure replaced is a fault of the parser's own, found
        // below, and is not an account to reconcile.
        let row_kept = matches!(
            self.payloads.values.get(slot),
            Some(RetainedPayload::CspState(_))
        );
        let result = result.and_then(|out| {
            if row_kept {
                self.reconcile_csp(c, &mut value)?;
            }
            Ok(out)
        });
        match self.payloads.values.get_mut(slot) {
            Some(RetainedPayload::CspState(original)) => {
                *original = Some(value);
                result
            }
            _ => {
                // The state has no row left to return to: credit what it
                // retained, as the registry's own release would have.
                let invariant = latch_registry_row(c);
                let _ = value.account.live.release(c);
                Err(invariant)
            }
        }
    }
    /// What the policy or transition row retains beyond the registry's own
    /// accounting of its slot; zero for any other row.
    fn csp_row_usage(row: &RetainedPayload) -> Result<Storage, SessionError> {
        Ok(match row {
            RetainedPayload::CspPolicy(v) => {
                let mut usage = Storage {
                    bytes: size_of::<Policy>(),
                    units: 0,
                    entries: 1,
                };
                if let Some(value) = &v.value {
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
                usage
            }
            RetainedPayload::CspTransition(v) => {
                let mut usage = Storage {
                    bytes: size_of::<Transition>(),
                    units: 0,
                    entries: 1,
                };
                if let Some((receipt, retired, request, prepared)) = &v.value {
                    for (layout, inline) in [
                        (receipt.retained_layout(), size_of::<HtmlMetaCspReceipt>()),
                        (retired.retained_layout(), size_of::<HtmlCspAssociation>()),
                        (request.retained_layout(), size_of::<HtmlMetaCspRequest>()),
                        (prepared.retained_layout(), size_of::<HtmlMetaCspPrepared>()),
                    ] {
                        let native =
                            Storage::from(layout.map_err(SessionError::CspProtocol)?.storage());
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
                usage
            }
            _ => Storage::ZERO,
        })
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
        let mut rows = state.rows_usage;
        for row in self.payloads.values.iter().skip(state.rows_scanned) {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            let usage = Self::csp_row_usage(row)?;
            rows = rows.add(usage).ok_or(SessionError::WrongToken)?;
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
            for row in &self.payloads.values {
                all = all
                    .add(Self::csp_row_usage(row)?)
                    .ok_or(SessionError::WrongToken)?;
            }
            assert_eq!(
                rows, all,
                "the incremental policy row usage equals a full sum"
            );
        }
        state.rows_scanned = self.payloads.values.len();
        state.rows_usage = rows;
        Ok(())
    }
    pub(super) fn csp_policy(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<&HtmlCspAssociation>, SessionError> {
        let Some(index) = self.csp_policy_slot(c, document)? else {
            return Ok(None);
        };
        let RetainedPayload::CspPolicy(value) = self.payloads.row(c, index)? else {
            return Err(latch_registry_row(c));
        };
        Ok(value.value.as_ref())
    }
    pub(super) fn csp_policy_slot(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        for row in &self.payloads.values {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::Document {
                node: Some(node),
                meta,
                ..
            } = row
                && *node == document
            {
                return Ok(meta.csp);
            }
        }
        Ok(None)
    }
    pub(super) fn csp_pending(&self, c: &mut ParseControl) -> Result<bool, SessionError> {
        let Some(slot) = self.csp_slot() else {
            return Ok(false);
        };
        Ok(matches!(
            self.payloads.row(c, slot)?,
            RetainedPayload::CspState(Some(state)) if state.pending.is_some()
        ))
    }
    pub(super) fn ensure_csp_policy(
        &mut self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<(), SessionError> {
        if self.csp_slot().is_none() || self.csp_policy(c, document)?.is_some() {
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
            let prepared = this.prepare_payload_slot_owned(c)?;
            native_storage(c, &mut state.account, size_of::<Policy>())?;
            let index = this
                .payloads
                .push_row(
                    prepared,
                    RetainedPayload::CspPolicy(Box::new(Policy {
                        value: Some(association),
                    })),
                )
                .map_err(|_| latch_registry_row(c))?;
            let doc = this.document_slot(c, document)?;
            payload_write::<Option<usize>>(
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
        if self.csp_slot().is_none() {
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
        let Some(index) = self.csp_slot() else {
            return Ok(true);
        };
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let RetainedPayload::CspState(Some(state)) = self.payloads.row(c, index)? else {
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
        if s.csp_slot().is_some() {
            return Err(SessionError::WrongToken);
        }
        let prepared = s.prepare_payload_slot_owned(c)?;
        let mut account = Account::default();
        payload_write::<Option<usize>>(c, 1)?;
        native_storage(c, &mut account, size_of::<State>())?;
        let state_slot = s
            .payloads
            .push_row(
                prepared,
                RetainedPayload::CspState(Some(Box::new(State {
                    owner: None,
                    account,
                    pending: None,
                    initial_base,
                    initial: None,
                    owner_usage: Storage::ZERO,
                    rows_scanned: 0,
                    rows_usage: Storage::ZERO,
                }))),
            )
            .map_err(|_| latch_registry_row(c))?;
        s.csp_slot = Some(state_slot);
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
                let current_slot = s
                    .csp_policy_slot(c, pending.document)?
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
                let admission = s.prepare_payload_slot_owned(c)?;
                native_storage(c, &mut state.account, size_of::<Policy>())?;
                let next_slot = s
                    .payloads
                    .push_row(
                        admission,
                        RetainedPayload::CspPolicy(Box::new(Policy { value: None })),
                    )
                    .map_err(|_| latch_registry_row(c))?;
                let admission = s.prepare_payload_slot_owned(c)?;
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
                let transition_slot = s
                    .payloads
                    .push_row(
                        admission,
                        RetainedPayload::CspTransition(Box::new(Transition {
                            value: None,
                            _cause: transition_cause,
                            _document: pending.document,
                            _subject: pending.subject,
                        })),
                    )
                    .map_err(|_| latch_registry_row(c))?;
                let doc = s.document_slot(c, pending.document)?;
                payload_write::<Option<usize>>(c, 1)?;
                let publication = state
                    .owner
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .prepare_publication_with(&mut current, request, prepared, &mut |a| {
                        state.account.admit(c, a, Caller::Parser)
                    })
                    .map_err(|e| protocol(c, e.error))?;
                let completion = publication.publish();
                let RetainedPayload::CspPolicy(value) = s.payloads.row_mut(c, next_slot)? else {
                    return Err(latch_registry_row(c));
                };
                value.value = Some(current);
                let RetainedPayload::CspTransition(value) =
                    s.payloads.row_mut(c, transition_slot)?
                else {
                    return Err(latch_registry_row(c));
                };
                value.value = Some(completion.into_parts());
                s.meta_history_mut(doc)
                    .map_err(|_| latch_registry_row(c))?
                    .csp = Some(next_slot);
                let _ = current_slot;
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
    fn a_meta_csp_state_row_replaced_during_its_use_reports_the_registry_row() {
        let request = request("<body><div id=a>");
        let (mut c, mut session, _proof, _nodes) = graph(&request);
        let storage = session.active_mut().unwrap();
        let prepared = storage.prepare_payload_slot_owned(&mut c).unwrap();
        let mut account = Account::default();
        native_storage(&mut c, &mut account, size_of::<State>()).unwrap();
        let slot = storage
            .payloads
            .push_row(
                prepared,
                RetainedPayload::CspState(Some(Box::new(State {
                    owner: None,
                    account,
                    pending: None,
                    initial_base: false,
                    initial: None,
                    owner_usage: Storage::ZERO,
                    rows_scanned: 0,
                    rows_usage: Storage::ZERO,
                }))),
            )
            .unwrap();
        storage.csp_slot = Some(slot);
        let result = storage.with_csp(&mut c, |this, _, _| {
            this.payloads.values[slot] =
                RetainedPayload::CspPolicy(Box::new(Policy { value: None }));
            Ok(())
        });
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(result.err(), Some(SessionError::Stop(invariant)));
        assert_eq!(c.stop(), Some(invariant));
        storage.csp_slot = None;
        session.release(&mut c).unwrap();
        storage_zero(&c);
    }

    #[test]
    fn a_pending_query_with_a_state_slot_past_the_registry_reports_the_registry_row() {
        let request = request("<body><div id=a>");
        let (mut c, mut session, _proof, _nodes) = graph(&request);
        let storage = session.active_mut().unwrap();
        assert_eq!(storage.csp_pending(&mut c), Ok(false));
        storage.csp_slot = Some(storage.payloads.values.len());
        let result = storage.csp_pending(&mut c);
        let invariant = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(result.err(), Some(SessionError::Stop(invariant)));
        assert_eq!(c.stop(), Some(invariant));
        storage.csp_slot = None;
        session.release(&mut c).unwrap();
        storage_zero(&c);
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
