//! Ordinary-driver-owned Meta CSP service obligations and retained policy values.
use super::*;
use crate::meta_csp::{Account, Outcome, Service, Storage};
use surgeist_html::{
    HtmlCspAssociation, HtmlCspOwner, HtmlCspPolicyId, HtmlCspPolicyRevision, HtmlMetaCspPrepared,
    HtmlMetaCspReceipt, HtmlMetaCspRequest,
};

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
}
fn protocol(c: &mut ParseControl, e: surgeist_html::HtmlCspError<ParseStop>) -> SessionError {
    match e {
        surgeist_html::HtmlCspError::Interrupted(s) => s.into(),
        surgeist_html::HtmlCspError::Protocol(p) => {
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
        (WorkCategory::ReserveByte, bytes as u64),
        (WorkCategory::CopyByte, bytes as u64),
        (WorkCategory::RecordWrite, 1),
        (WorkCategory::ProviderCall, 2),
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
        let RetainedPayload::CspState(value) = &mut self.payloads.values[slot] else {
            unreachable!()
        };
        let mut value = value.take().ok_or(SessionError::WrongToken)?;
        let result = f(self, c, &mut value);
        value.account.settle(c);
        // Native temporaries have now actually dropped. Reconcile the receipt
        // to the complete graph still retained by original payloads.
        let result = result.and_then(|out| {
            self.reconcile_csp(c, &mut value)?;
            Ok(out)
        });
        let RetainedPayload::CspState(original) = &mut self.payloads.values[slot] else {
            unreachable!()
        };
        *original = Some(value);
        result
    }
    fn reconcile_csp(&self, c: &mut ParseControl, state: &mut State) -> Result<(), SessionError> {
        let mut live = state
            .owner_usage
            .add(Storage {
                bytes: size_of::<State>(),
                units: 0,
                entries: 1,
            })
            .ok_or(SessionError::WrongToken)?;
        for row in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let usage = match row {
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
            };
            live = live.add(usage).ok_or(SessionError::WrongToken)?;
        }
        if let Some(p) = &state.pending {
            let units = p.content.shared_code_units().capacity();
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
        let retired = state
            .account
            .live
            .sub(live)
            .ok_or(SessionError::WrongToken)?;
        retired.release(c);
        state.account.live = live;
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
        let RetainedPayload::CspPolicy(value) = &self.payloads.values[index] else {
            unreachable!("qualified CSP policy")
        };
        Ok(value.value.as_ref())
    }
    pub(super) fn csp_policy_slot(
        &self,
        c: &mut ParseControl,
        document: usize,
    ) -> Result<Option<usize>, SessionError> {
        for row in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
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
    pub(super) fn csp_pending(&self) -> bool {
        self.csp_slot().is_some_and(|slot|matches!(&self.payloads.values[slot],RetainedPayload::CspState(Some(v))if v.pending.is_some()))
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
                .initial_document_with(&mut |a| state.account.admit(c, a))
                .map_err(|e| protocol(c, e))?;
            state.account.settle(c);
            if document == 0 {
                payload_write::<Option<(HtmlCspPolicyId, HtmlCspPolicyRevision)>>(c, 1)?;
                c.charge(WorkCategory::ProviderCall, 1)?;
                state.initial = Some((
                    association.policy().id().clone(),
                    association.policy().revision(),
                ));
            }
            this.prepare_payload_slot_owned(c)?;
            native_storage(c, &mut state.account, size_of::<Policy>())?;
            let index = this.payloads.values.len();
            this.payloads
                .values
                .push(RetainedPayload::CspPolicy(Box::new(Policy {
                    value: Some(association),
                })));
            let doc = this.document_slot(c, document)?;
            payload_write::<Option<usize>>(
                c,
                if state.initial_base && document == 0 {
                    2
                } else {
                    1
                },
            )?;
            this.meta_history_mut(doc).csp = Some(index);
            if state.initial_base && document == 0 {
                this.meta_history_mut(doc).base_csp = Some(index);
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
            .children()
            .len()
            != 0
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
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::CspState(Some(state)) = &self.payloads.values[index] else {
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
        s.prepare_payload_slot_owned(c)?;
        let mut account = Account::default();
        payload_write::<Option<usize>>(c, 1)?;
        native_storage(c, &mut account, size_of::<State>())?;
        s.csp_slot = Some(s.payloads.values.len());
        s.payloads
            .values
            .push(RetainedPayload::CspState(Some(Box::new(State {
                owner: None,
                account,
                pending: None,
                initial_base,
                initial: None,
                owner_usage: Storage::ZERO,
            }))));
        s.with_csp(c, |_, c, state| {
            let owner = HtmlCspOwner::new_with(&mut |a| state.account.admit(c, a))
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
                c.charge(
                    WorkCategory::ProviderCall,
                    layout
                        .disposal()
                        .work_credits()
                        .checked_add(1)
                        .ok_or(SessionError::WrongToken)?,
                )?;
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
                        &mut |a| state.account.admit(c, a),
                    )
                    .map_err(|e| protocol(c, e.error))?;
                let prepared = match service.call(c, &mut state.account, &request)? {
                    Outcome::Completed(p) => p,
                    Outcome::Unavailable(reason) => {
                        s.meta_missing(
                            c,
                            pending.document,
                            pending.subject,
                            meta::MetaMissingKind::CspProcessing(reason),
                        )?;
                        return Ok(());
                    }
                    Outcome::Terminal => return Err(SessionError::CspService),
                };
                // All receivers and payload backing precede the policy fence.
                s.prepare_payload_slot_owned(c)?;
                native_storage(c, &mut state.account, size_of::<Policy>())?;
                let next_slot = s.payloads.values.len();
                s.payloads
                    .values
                    .push(RetainedPayload::CspPolicy(Box::new(Policy { value: None })));
                s.prepare_payload_slot_owned(c)?;
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
                let transition_slot = s.payloads.values.len();
                s.payloads
                    .values
                    .push(RetainedPayload::CspTransition(Box::new(Transition {
                        value: None,
                        _cause: transition_cause,
                        _document: pending.document,
                        _subject: pending.subject,
                    })));
                let doc = s.document_slot(c, pending.document)?;
                payload_write::<Option<usize>>(c, 1)?;
                let publication = state
                    .owner
                    .as_mut()
                    .ok_or(SessionError::WrongToken)?
                    .prepare_publication_with(&mut current, request, prepared, &mut |a| {
                        state.account.admit(c, a)
                    })
                    .map_err(|e| protocol(c, e.error))?;
                let completion = publication.publish();
                let RetainedPayload::CspPolicy(value) = &mut s.payloads.values[next_slot] else {
                    unreachable!()
                };
                value.value = Some(current);
                let RetainedPayload::CspTransition(value) = &mut s.payloads.values[transition_slot]
                else {
                    unreachable!()
                };
                value.value = Some(completion.into_parts());
                s.meta_history_mut(doc).csp = Some(next_slot);
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
