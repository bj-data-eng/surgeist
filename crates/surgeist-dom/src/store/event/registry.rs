//! Callback-free authority for the canonical owner's one live listener list.
use super::*;
use crate::event::{RegistryBinding, RegistryObservation};
use crate::{DomEventRegistryAssociation, DomEventRegistryPreparation};

impl Dom {
    /// Prepare one listener-registry owner. Snapshots cannot issue or retain
    /// this authority. A second live association is refused even when its
    /// proposed listener list is empty. Source observation admits the actual
    /// fixed binding storage and work before allocation. `payload_bytes` is the
    /// trusted registry owner's actual fixed out-of-line control backing; it
    /// remains charged until that owner withdraws its association.
    pub fn prepare_event_registry(
        &mut self,
        payload_bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<DomEventRegistryPreparation, DomError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if self
            .event_registry
            .as_ref()
            .and_then(|entry| entry.binding.upgrade())
            .is_some()
        {
            return Err(DomError::EventRegistryAlreadyBound);
        }
        let formation = self.source_observation_formation(cancel)?;
        source::formation::step(formation.as_ref(), cancel)?;
        DomEventRegistryAssociation::binding_payload_bytes()
            .checked_add(payload_bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let body = self.custody_registry.ledger().retain_shared(
            custody::Usage::new(size_of::<RegistryBinding>(), 0),
            |usage| match &formation {
                Some(formation) => formation.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
        )?;
        let payload = self.custody_registry.ledger().retain(
            custody::Usage::new(payload_bytes, 0),
            |usage| match &formation {
                Some(formation) => formation.admit_custody_usage(usage, cancel),
                None => check_cancel(cancel),
            },
        )?;
        let binding = source::formation::allocated_arc(RegistryBinding { owner: self.owner });
        Ok(DomEventRegistryPreparation(DomEventRegistryAssociation {
            binding,
            body,
            payload,
        }))
    }
}
impl DomEventRegistryPreparation {
    /// Publish after consumer preflight. Refusal returns the original funding
    /// capability: free its represented buffers before dropping it. Publication
    /// rechecks current Source admission and the exact canonical owner.
    #[expect(
        clippy::result_large_err,
        reason = "Refusal returns the original funding without allocation so represented buffers can be freed before releasing custody"
    )]
    pub fn commit(
        self,
        dom: &mut Dom,
        cancel: &AtomicBool,
    ) -> Result<DomEventRegistryAssociation, (DomError, Self)> {
        let validate = || -> Result<(), DomError> {
            dom.require_source_entry()?;
            check_cancel(cancel)?;
            if self.0.binding.owner != dom.owner {
                return Err(DomError::ForeignOwner);
            }
            if dom
                .event_registry
                .as_ref()
                .and_then(|entry| entry.binding.upgrade())
                .is_some()
            {
                return Err(DomError::EventRegistryAlreadyBound);
            }
            let formation = dom.source_observation_formation(cancel)?;
            source::formation::step(formation.as_ref(), cancel)?;
            if let Some(formation) = formation {
                formation.admit_custody_usage(dom.custody_registry.ledger().total(), cancel)?;
            }
            Ok(())
        };
        if let Err(error) = validate() {
            return Err((error, self));
        }
        dom.event_registry = Some(RegistryObservation {
            binding: Arc::downgrade(&self.0.binding),
            _body: self.0.body.clone(),
        });
        Ok(self.0)
    }
}
impl DomEventRegistryAssociation {
    /// Validate the exact current association. Historical identities and a
    /// foreign canonical owner cannot substitute for this affine authority.
    pub fn validate(&self, dom: &Dom) -> Result<(), DomError> {
        if self.binding.owner != dom.owner {
            return Err(DomError::ForeignOwner);
        }
        if !dom
            .event_registry
            .as_ref()
            .and_then(|entry| entry.binding.upgrade())
            .is_some_and(|current| Arc::ptr_eq(&current, &self.binding))
        {
            return Err(DomError::InvalidEventRegistry);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn claim(
        dom: &mut Dom,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<DomEventRegistryAssociation, DomError> {
        dom.prepare_event_registry(bytes, cancel)?
            .commit(dom, cancel)
            .map_err(|(error, _)| error)
    }
    fn registry_target_backing(dom: &Dom) -> usize {
        // A nonempty Weak still owns this allocation's target area at strong
        // zero. This observes backing independently of the claimed debit.
        dom.event_registry.as_ref().map_or(0, |_| {
            size_of::<RegistryBinding>() + size_of::<custody::Debit>()
        })
    }
    fn dom() -> Dom {
        Dom::new(
            DomLimits::new(
                16,
                1024,
                32,
                16,
                16,
                4,
                100_000,
                crate::DomNameLimits::new(1_000_000, 1024, 100_000, 100_000).unwrap(),
            )
            .unwrap()
            .with_dispatch_custody(
                crate::DomDispatchCustodyLimits::new(8, 32, 1_000_000, 1_000_000, 100_000).unwrap(),
            ),
        )
        .unwrap()
    }
    #[test]
    fn unpublished_preparation_and_failed_publication_leave_no_canonical_backing() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let preparation = dom.prepare_event_registry(64, &cancel).unwrap();
        let backing = vec![0_u8; 64];
        assert!(dom.event_registry.is_none());
        let charged = before
            .checked_add(custody::Usage::new(
                size_of::<RegistryBinding>() + size_of::<custody::Debit>() + backing.capacity(),
                0,
            ))
            .unwrap();
        assert_eq!(ledger.total(), charged);
        let mut foreign = self::dom();
        let foreign_before = foreign.custody_registry.ledger().total();
        let (error, preparation) = preparation.commit(&mut foreign, &cancel).unwrap_err();
        assert_eq!(error, DomError::ForeignOwner);
        assert_eq!(ledger.total(), charged);
        assert_eq!(foreign.custody_registry.ledger().total(), foreign_before);
        let (error, preparation) = preparation
            .commit(&mut dom, &AtomicBool::new(true))
            .unwrap_err();
        assert_eq!(error, DomError::Cancelled);
        assert!(dom.event_registry.is_none());
        assert_eq!(
            ledger.total(),
            charged,
            "refusal retains funding while actual buffer is live"
        );
        drop(backing);
        drop(preparation);
        assert_eq!(ledger.total(), before);
        let other_preparation = dom.prepare_event_registry(0, &cancel).unwrap();
        drop(other_preparation);
        assert!(dom.event_registry.is_none());
        assert_eq!(ledger.total(), before);
    }
    #[test]
    fn canonical_observer_can_drop_first_without_releasing_strong_body_custody() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let ledger = dom.custody_registry.ledger().clone();
        let association = claim(&mut dom, 19, &cancel).unwrap();
        drop(dom);
        assert_eq!(
            ledger.total(),
            custody::Usage::new(
                size_of::<RegistryBinding>() + size_of::<custody::Debit>() + 19,
                0,
            )
        );
        drop(association);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
    }
    #[test]
    fn separately_prepared_claim_loses_to_live_association_and_can_retry_after_withdrawal() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let body = size_of::<RegistryBinding>() + size_of::<custody::Debit>();
        let first = dom.prepare_event_registry(17, &cancel).unwrap();
        let first_backing = vec![0_u8; 17];
        let second = dom.prepare_event_registry(29, &cancel).unwrap();
        let second_backing = vec![0_u8; 29];
        let both = before
            .checked_add(custody::Usage::new(
                2 * body + first_backing.capacity() + second_backing.capacity(),
                0,
            ))
            .unwrap();
        assert_eq!(ledger.total(), both);
        assert!(dom.event_registry.is_none());
        let first = first.commit(&mut dom, &cancel).unwrap();
        let (error, second) = second.commit(&mut dom, &cancel).unwrap_err();
        assert_eq!(error, DomError::EventRegistryAlreadyBound);
        first.validate(&dom).unwrap();
        assert_eq!(ledger.total(), both);
        drop(first_backing);
        drop(first);
        let second = second.commit(&mut dom, &cancel).unwrap();
        second.validate(&dom).unwrap();
        assert_eq!(
            ledger.total(),
            before
                .checked_add(custody::Usage::new(body + second_backing.capacity(), 0))
                .unwrap()
        );
        drop(second_backing);
        drop(second);
        assert_eq!(
            ledger.total(),
            before.checked_add(custody::Usage::new(body, 0)).unwrap()
        );
        drop(dom);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
    }
    #[test]
    fn affine_claim_charges_exact_backing_and_refuses_another_live_owner() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let baseline = dom.custody_registry.ledger().total();
        let association = claim(&mut dom, 31, &cancel).unwrap();
        assert_eq!(association.owner(), dom.owner());
        association.validate(&dom).unwrap();
        let charged = baseline
            .checked_add(custody::Usage::new(
                DomEventRegistryAssociation::binding_payload_bytes() + 31,
                0,
            ))
            .unwrap();
        assert_eq!(dom.custody_registry.ledger().total(), charged);
        assert_eq!(
            association.retained_bytes(),
            DomEventRegistryAssociation::binding_payload_bytes() + 31
        );
        assert!(matches!(
            claim(&mut dom, 17, &cancel),
            Err(DomError::EventRegistryAlreadyBound)
        ));
        assert_eq!(dom.custody_registry.ledger().total(), charged);
        drop(association);
        assert_eq!(
            dom.custody_registry.ledger().total(),
            baseline
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
        let fresh = claim(&mut dom, 17, &cancel).unwrap();
        fresh.validate(&dom).unwrap();
        drop(fresh);
        assert_eq!(
            dom.custody_registry.ledger().total(),
            baseline
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
    }
    #[test]
    fn cancellation_overflow_and_foreign_validation_do_not_claim_or_charge() {
        let mut first = dom();
        let mut second = dom();
        let before = first.custody_registry.ledger().total();
        assert!(matches!(
            claim(&mut first, 0, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
        assert!(matches!(
            claim(&mut first, usize::MAX, &AtomicBool::new(false)),
            Err(DomError::RepresentationExhausted)
        ));
        assert_eq!(first.custody_registry.ledger().total(), before);
        let association = claim(&mut first, 0, &AtomicBool::new(false)).unwrap();
        assert_eq!(association.validate(&second), Err(DomError::ForeignOwner));
        claim(&mut second, 0, &AtomicBool::new(false)).unwrap();
        fn is_send<T: Send>() {}
        is_send::<Dom>();
        is_send::<DomEventRegistryAssociation>();
    }

    fn observation(
        bytes: usize,
    ) -> (
        Dom,
        crate::DomSourceEventExecutor,
        crate::DomSourceEventRequest,
        crate::DomSourceEventDispatchGuard,
    ) {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let mut batch = crate::DomBatch::new();
        let document = batch.create_document();
        let dialog = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let commit = dom.apply(batch, &cancel).unwrap();
        let target = commit.created(&dialog).unwrap().unwrap();
        let executor = dom.bind_source_event_executor(&cancel).unwrap();
        let limits = crate::DomSourceLimits::new(bytes, 1_000_000, 1_000_000, 16, 1024, 1024, 4);
        let request = {
            let mut lease = dom
                .begin_source_edit(crate::DomSourceEdit::show_dialog(target), limits, &cancel)
                .unwrap();
            let crate::DomSourceProgress::Request(crate::DomSourceRequest::Event(request)) =
                lease.advance(&cancel).unwrap()
            else {
                panic!("actual Show event observation required");
            };
            request
        };
        let permit = executor.begin_dispatch(&dom, &request, &cancel).unwrap();
        let guard = executor.enter_dispatch(&dom, &permit, &cancel).unwrap();
        (dom, executor, request, guard)
    }
    #[test]
    fn source_exact_and_one_short_claims_preserve_real_scope_and_custody() {
        // Below the independent 1MB dispatch-custody cap. This isolates the
        // original Source storage admission rather than another producer cap.
        const LIMIT: usize = 262_144;
        let cancel = AtomicBool::new(false);
        let (mut dom, _executor, request, guard) = observation(LIMIT);
        let stamp = dom.snapshot().stamp();
        let before = dom.custody_registry.ledger().total();
        // Place both requests against the SAME actual observation: fresh
        // owners can require different custody-index backing for their real
        // materialization IDs. The temporary formation owns no extra payload.
        let formation = dom.source_observation_formation(&cancel).unwrap().unwrap();
        assert_eq!(formation.owned_payload().bytes, 0);
        let payload = formation.remaining_bytes().unwrap()
            - DomEventRegistryAssociation::binding_payload_bytes();
        drop(formation);
        assert_eq!(dom.custody_registry.ledger().total(), before);
        assert!(matches!(
            claim(&mut dom, payload + 1, &cancel),
            Err(DomError::ResourceLimit {
                resource: DomResource::SourceRetainedBytes,
                limit: LIMIT
            })
        ));
        assert_eq!(dom.custody_registry.ledger().total(), before);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
        let association = claim(&mut dom, payload, &cancel).unwrap();
        association.validate(&dom).unwrap();
        let mut backing = Vec::<u8>::new();
        backing.try_reserve_exact(payload).unwrap();
        assert_eq!(backing.capacity(), payload);
        assert_eq!(
            dom.custody_registry.ledger().total(),
            before
                .checked_add(custody::Usage::new(
                    DomEventRegistryAssociation::binding_payload_bytes() + payload,
                    0
                ))
                .unwrap()
        );
        drop(backing);
        drop(association);
        assert_eq!(
            dom.custody_registry.ledger().total(),
            before
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
        drop(guard);
        assert!(matches!(
            claim(&mut dom, 0, &cancel),
            Err(DomError::SourceOperationPending)
        ));
    }
    #[test]
    fn registry_publication_revalidates_current_source_storage_and_work_policy() {
        const LIMIT: usize = 262_144;
        let cancel = AtomicBool::new(false);
        let (mut dom, _executor, request, _guard) = observation(LIMIT);
        let preparation = dom.prepare_event_registry(64, &cancel).unwrap();
        let backing = vec![0_u8; 64];
        let ledger = dom.custody_registry.ledger().clone();
        let charged = ledger.total();
        let stamp = dom.snapshot().stamp();
        let formation = dom.source_observation_formation(&cancel).unwrap().unwrap();
        let exact = LIMIT - formation.remaining_bytes().unwrap();
        drop(formation);
        let work_limit = request.work.work.load(Ordering::Relaxed) + 32;
        let policy =
            |bytes, work| crate::DomSourceLimits::new(bytes, 1_000_000, work, 16, 1024, 1024, 4);
        assert!(matches!(
            dom.resume_source_edit(request.operation(), policy(exact - 1, work_limit), &cancel),
            Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == exact - 1
        ));
        drop(
            dom.resume_source_edit(request.operation(), policy(exact, work_limit), &cancel)
                .unwrap(),
        );
        let remaining = work_limit - request.work.work.load(Ordering::Relaxed);
        for _ in 0..remaining {
            dom.source_event_step(&request, &cancel).unwrap();
        }
        let (error, preparation) = preparation.commit(&mut dom, &cancel).unwrap_err();
        assert_eq!(error, DomError::SourceWorkLimit { limit: work_limit });
        assert!(dom.event_registry.is_none());
        assert_eq!(ledger.total(), charged);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
        drop(
            dom.resume_source_edit(request.operation(), policy(exact, 1_000_000), &cancel)
                .unwrap(),
        );
        let association = preparation.commit(&mut dom, &cancel).unwrap();
        association.validate(&dom).unwrap();
        assert_eq!(
            ledger.total(),
            charged,
            "publication aliases pre-admitted body custody"
        );
        drop(backing);
        drop(association);
        assert_eq!(
            ledger.total(),
            charged.checked_sub(custody::Usage::new(64, 0)).unwrap()
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
    }
    #[test]
    fn registry_publication_refuses_closed_source_entry_without_discarding_funding() {
        let cancel = AtomicBool::new(false);
        let (mut dom, _executor, request, guard) = observation(262_144);
        let preparation = dom.prepare_event_registry(41, &cancel).unwrap();
        let backing = vec![0_u8; 41];
        drop(guard);
        let ledger = dom.custody_registry.ledger().clone();
        let charged = ledger.total();
        let stamp = dom.snapshot().stamp();
        let (error, preparation) = preparation.commit(&mut dom, &cancel).unwrap_err();
        assert_eq!(error, DomError::SourceOperationPending);
        assert!(dom.event_registry.is_none());
        assert_eq!(ledger.total(), charged);
        drop(backing);
        drop(preparation);
        assert_eq!(
            ledger.total(),
            charged
                .checked_sub(custody::Usage::new(
                    size_of::<RegistryBinding>() + size_of::<custody::Debit>() + 41,
                    0,
                ))
                .unwrap()
        );
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
    }
    #[test]
    fn source_storage_split_preserves_original_request_and_exact_resource_union() {
        let cancel = AtomicBool::new(false);
        let (dom, _executor, request, _guard) = observation(1_000_000);
        let stamp = dom.snapshot().stamp();
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let mut whole = dom
            .retain_source_event_payload(&request, 200, 100, &cancel)
            .unwrap();
        let charged = before.checked_add(custody::Usage::new(200, 100)).unwrap();
        assert_eq!(ledger.total(), charged);
        assert!(matches!(
            whole.split(201, 100),
            Err(DomError::RepresentationExhausted)
        ));
        assert_eq!(ledger.total(), charged);
        let part = whole.split(60, 30).unwrap();
        let parent_backing = vec![0_u16; 70];
        let child_backing = vec![0_u16; 30];
        assert_eq!(ledger.total(), charged);
        assert!(Arc::ptr_eq(&whole.work, &part.work));
        assert_eq!(
            (whole.bytes, whole.units, part.bytes, part.units),
            (140, 70, 60, 30)
        );
        drop(child_backing);
        drop(part);
        assert_eq!(
            ledger.total(),
            before.checked_add(custody::Usage::new(140, 70)).unwrap()
        );
        drop(parent_backing);
        drop(whole);
        assert_eq!(ledger.total(), before);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(
            dom.pending_source_edit().unwrap().operation(),
            request.operation()
        );
    }
    #[test]
    fn withdrawn_registry_body_stays_charged_until_its_last_weak_observation_is_removed() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let association = claim(&mut dom, 0, &cancel).unwrap();
        assert_eq!(
            ledger.total(),
            before
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
        drop(association);
        assert_eq!(
            ledger.total(),
            before
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap(),
            "withdrawn authority does not free a body still retained by the canonical weak slot"
        );
        // Withdrawing authority permits a fresh owner. It neither transfers
        // the old authority nor charges the retired body's backing twice.
        let replacement = claim(&mut dom, 0, &cancel).unwrap();
        replacement.validate(&dom).unwrap();
        assert_eq!(
            ledger.total(),
            before
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
        drop(replacement);
        assert_eq!(
            ledger.total(),
            before
                .checked_add(custody::Usage::new(registry_target_backing(&dom), 0))
                .unwrap()
        );
        drop(dom);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
    }
}
