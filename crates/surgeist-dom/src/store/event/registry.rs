//! Callback-free authority for the canonical owner's one live listener list.
use super::*;
use crate::DomEventRegistryAssociation;
use crate::event::RegistryBinding;

impl Dom {
    /// Associate one listener-registry owner. Snapshots cannot issue or retain
    /// this authority. A second live association is refused even when its
    /// proposed listener list is empty. Source observation admits the actual
    /// fixed binding storage and work before allocation. `payload_bytes` is the
    /// trusted registry owner's actual fixed out-of-line control backing; it
    /// remains charged until that owner withdraws its association.
    pub fn bind_event_registry(
        &mut self,
        payload_bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<DomEventRegistryAssociation, DomError> {
        self.require_source_entry()?;
        check_cancel(cancel)?;
        if self.event_registry.upgrade().is_some() {
            return Err(DomError::EventRegistryAlreadyBound);
        }
        let formation = self.source_observation_formation(cancel)?;
        source::formation::step(formation.as_ref(), cancel)?;
        let bytes = DomEventRegistryAssociation::binding_payload_bytes()
            .checked_add(payload_bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        let storage =
            self.custody_registry
                .ledger()
                .retain(custody::Usage::new(bytes, 0), |usage| match &formation {
                    Some(formation) => formation.admit_custody_usage(usage, cancel),
                    None => check_cancel(cancel),
                })?;
        let binding = source::formation::allocated_arc(RegistryBinding {
            owner: self.owner,
            _storage: storage,
        });
        self.event_registry = Arc::downgrade(&binding);
        Ok(DomEventRegistryAssociation { binding })
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
            .upgrade()
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
    fn registry_target_backing(dom: &Dom) -> usize {
        // A nonempty Weak still owns this allocation's target area at strong
        // zero. This observes backing independently of the claimed debit.
        if dom.event_registry.ptr_eq(&std::sync::Weak::new()) {
            0
        } else {
            DomEventRegistryAssociation::binding_payload_bytes()
        }
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
    fn affine_claim_charges_exact_backing_and_refuses_another_live_owner() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let baseline = dom.custody_registry.ledger().total();
        let association = dom.bind_event_registry(31, &cancel).unwrap();
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
            dom.bind_event_registry(17, &cancel),
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
        let fresh = dom.bind_event_registry(17, &cancel).unwrap();
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
            first.bind_event_registry(0, &AtomicBool::new(true)),
            Err(DomError::Cancelled)
        ));
        assert!(matches!(
            first.bind_event_registry(usize::MAX, &AtomicBool::new(false)),
            Err(DomError::RepresentationExhausted)
        ));
        assert_eq!(first.custody_registry.ledger().total(), before);
        let association = first
            .bind_event_registry(0, &AtomicBool::new(false))
            .unwrap();
        assert_eq!(association.validate(&second), Err(DomError::ForeignOwner));
        second
            .bind_event_registry(0, &AtomicBool::new(false))
            .unwrap();
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
            dom.bind_event_registry(payload + 1, &cancel),
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
        let claim = dom.bind_event_registry(payload, &cancel).unwrap();
        claim.validate(&dom).unwrap();
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
        drop(claim);
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
            dom.bind_event_registry(0, &cancel),
            Err(DomError::SourceOperationPending)
        ));
    }
    #[test]
    fn withdrawn_registry_body_stays_charged_until_its_last_weak_observation_is_removed() {
        let cancel = AtomicBool::new(false);
        let mut dom = dom();
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let association = dom.bind_event_registry(0, &cancel).unwrap();
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
        let replacement = dom.bind_event_registry(0, &cancel).unwrap();
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
