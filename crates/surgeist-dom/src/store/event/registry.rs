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
        assert_eq!(dom.custody_registry.ledger().total(), baseline);
        let fresh = dom.bind_event_registry(17, &cancel).unwrap();
        fresh.validate(&dom).unwrap();
        drop(fresh);
        assert_eq!(dom.custody_registry.ledger().total(), baseline);
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
        const LIMIT: usize = 32_000_000;
        const PAYLOAD: usize = 65_536;
        let cancel = AtomicBool::new(false);
        // This measurement places the finite admission boundary. Independent
        // assertions below specify ownership, cleanup and unchanged DOM effects.
        let (probe, _probe_executor, _probe_request, _probe_guard) = observation(LIMIT);
        let formation = probe
            .source_observation_formation(&cancel)
            .unwrap()
            .unwrap();
        let baseline = LIMIT - formation.remaining_bytes().unwrap();
        let exact = baseline + DomEventRegistryAssociation::binding_payload_bytes() + PAYLOAD;
        drop(formation);
        drop(probe);
        for bytes in [exact - 1, exact] {
            let (mut dom, _executor, request, guard) = observation(bytes);
            let stamp = dom.snapshot().stamp();
            let before = dom.custody_registry.ledger().total();
            let claim = dom.bind_event_registry(PAYLOAD, &cancel);
            if bytes < exact {
                assert!(
                    matches!(claim, Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == bytes)
                );
                assert_eq!(dom.custody_registry.ledger().total(), before);
                let retry = dom.bind_event_registry(0, &cancel).unwrap();
                retry.validate(&dom).unwrap();
                drop(retry);
            } else {
                let claim = claim.unwrap();
                let mut backing = Vec::<u8>::new();
                backing.try_reserve_exact(PAYLOAD).unwrap();
                assert_eq!(backing.capacity(), PAYLOAD);
                assert_eq!(
                    dom.custody_registry.ledger().total(),
                    before
                        .checked_add(custody::Usage::new(
                            DomEventRegistryAssociation::binding_payload_bytes() + PAYLOAD,
                            0
                        ))
                        .unwrap()
                );
                drop(backing);
                drop(claim);
            }
            assert_eq!(dom.custody_registry.ledger().total(), before);
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
    }
}
