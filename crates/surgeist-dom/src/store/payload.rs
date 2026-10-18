//! Immutable typed materializations. Semantics contain identity and shallow
//! receipts; controlled owners keep the matching custody separately.
#[cfg(test)]
mod encoding_tests;
use super::{DomError, bank, custody};
pub(crate) use custody::Usage;
use std::sync::Arc;

/// One nonreusable identity for a planned body. Projection reserves it before
/// acquiring its replacement extent or copying selected fields.
pub(crate) struct Reserved<T> {
    identity: custody::MaterializationId,
    value: std::marker::PhantomData<fn() -> T>,
}
impl<T> Reserved<T> {
    pub fn new() -> Result<Self, DomError> {
        #[cfg(test)]
        identity_refusal::before_reservation()?;
        Ok(Self {
            identity: custody::MaterializationId::fresh()?,
            value: std::marker::PhantomData,
        })
    }
    pub fn materialize(
        self,
        mut incoming: custody::Held<T>,
        own_usage: custody::Usage,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Pending<T>, DomError> {
        assert!(
            own_usage.bytes() >= size_of::<Payload<T>>(),
            "payload receipt covers its actual target body"
        );
        let extra = own_usage.checked_sub(incoming.debit().usage())?;
        incoming.debit_mut().grow(extra, &mut admit)?;
        let (value, debit) = incoming.into_parts();
        let value = Arc::new(Payload {
            identity: self.identity,
            own_usage,
            value,
        });
        #[cfg(test)]
        if super::scale_metrics::ENABLED {
            super::scale_metrics::record(super::scale_metrics::Event::StructuralAllocation {
                bytes: size_of::<Payload<T>>(),
            });
        }
        Ok(Pending {
            value,
            debit,
            shared: None,
        })
    }
}

#[derive(Debug)]
pub(crate) struct Payload<T> {
    pub identity: custody::MaterializationId,
    pub own_usage: custody::Usage,
    pub value: T,
}
impl<T: PartialEq> PartialEq for Payload<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}
impl<T: Eq> Eq for Payload<T> {}

#[cfg(test)]
impl<T> Drop for Payload<T> {
    fn drop(&mut self) {
        destruction_observation::entered(self.identity);
    }
}

#[cfg(test)]
pub(in crate::store) mod destruction_observation {
    use super::{Usage, custody};
    use std::{cell::Cell, cell::RefCell, marker::PhantomData, rc::Rc};

    struct Watch {
        identity: custody::MaterializationId,
        ledger: custody::Ledger,
        observed: Rc<Cell<Option<Usage>>>,
    }
    thread_local! { static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) }; }

    pub(in crate::store) struct Observation {
        observed: Rc<Cell<Option<Usage>>>,
        _thread: PhantomData<Rc<()>>,
    }
    impl Observation {
        pub fn begin(identity: custody::MaterializationId, ledger: &custody::Ledger) -> Self {
            let observed = Rc::new(Cell::new(None));
            WATCH.with(|slot| {
                let mut slot = slot.borrow_mut();
                assert!(
                    slot.is_none(),
                    "one payload destruction observation per thread"
                );
                *slot = Some(Watch {
                    identity,
                    ledger: ledger.clone(),
                    observed: observed.clone(),
                });
            });
            Self {
                observed,
                _thread: PhantomData,
            }
        }
        pub fn observed(&self) -> Option<Usage> {
            self.observed.get()
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            WATCH.with(|slot| *slot.borrow_mut() = None);
        }
    }
    pub(super) fn entered(identity: custody::MaterializationId) {
        WATCH.with(|slot| {
            if let Some(watch) = &*slot.borrow()
                && watch.identity == identity
            {
                watch.observed.set(Some(watch.ledger.total()));
            }
        });
    }
}

/// The only mutable phase. Its fresh identity is never copied from an alias.
/// Both the affine and shared receipt follow data in destruction order.
pub(crate) struct Pending<T> {
    value: Arc<Payload<T>>,
    debit: custody::Debit,
    shared: Option<custody::SharedDebit>,
}
impl<T> Pending<T> {
    pub fn new(
        incoming: custody::Held<T>,
        own_usage: custody::Usage,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        Reserved::new()?.materialize(incoming, own_usage, admit)
    }

    #[cfg(test)]
    pub(in crate::store) fn inspect_target(
        &self,
        graph: &mut super::compact_tests::AllocationGraph,
    ) {
        graph.target(&self.value, "form.names.target");
    }
    #[cfg(test)]
    pub(in crate::store) fn value(&self) -> &Payload<T> {
        &self.value
    }

    pub fn seal(
        mut self,
        registry: &bank::CustodyRegistry,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Sealed<T>, DomError> {
        self.shared = Some(self.debit.share(&mut admit)?);
        let custody = registry.adopt_object_retained(
            self.value.identity,
            self.shared.as_ref().expect("shared pending receipt"),
            &mut admit,
        )?;
        Ok(Sealed {
            value: self.value,
            custody,
        })
    }
}

#[cfg(test)]
pub(in crate::store) mod identity_refusal {
    use super::DomError;
    use std::{cell::Cell, marker::PhantomData, rc::Rc};
    thread_local! { static NEXT: Cell<Option<bool>> = const { Cell::new(None) }; }
    pub(super) fn before_reservation() -> Result<(), DomError> {
        NEXT.with(|next| {
            if next.get() == Some(true) {
                next.set(Some(false));
                Err(DomError::RepresentationExhausted)
            } else {
                Ok(())
            }
        })
    }
    pub(in crate::store) struct Refusal(PhantomData<Rc<()>>);
    impl Refusal {
        pub fn begin() -> Self {
            NEXT.with(|next| {
                assert!(next.get().is_none(), "one isolated payload issuer refusal");
                next.set(Some(true));
            });
            Self(PhantomData)
        }
        pub fn reached(&self) -> bool {
            NEXT.with(|next| next.get() == Some(false))
        }
    }
    impl Drop for Refusal {
        fn drop(&mut self) {
            NEXT.with(|next| next.set(None));
        }
    }
}

pub(crate) struct Sealed<T> {
    pub value: Arc<Payload<T>>,
    pub custody: bank::ObjectCustody,
}

struct SlotChildren {
    _form_names: bank::ObjectCustody,
}

#[cfg(test)]
pub(in crate::store) fn inspect_slot_form_custody(
    slots: &bank::BankCustody<super::Slot>,
) -> Vec<(custody::MaterializationId, Usage)> {
    slots
        .inspect_dependencies::<SlotChildren>()
        .iter()
        .map(|dependency| {
            (
                dependency._form_names.identity(),
                dependency._form_names.inspect_payload_usage(),
            )
        })
        .collect()
}
pub(in crate::store) fn slot_dependency(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let Some(names) = slot
        .node
        .as_ref()
        .and_then(|node| node.form_past_names.as_ref())
        .and_then(crate::form::FormPastNames::payload)
    else {
        return Ok(None);
    };
    let names = registry.adopt_object(names.identity, names.own_usage, &mut admit)?;
    bank::Dependency::new(
        SlotChildren { _form_names: names },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn describe_slot(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<bank::RecordDescription, DomError> {
    let usage = Usage::new(size_of::<super::Slot>(), 0).checked_add(
        slot.node
            .as_ref()
            .map_or(Usage::ZERO, |node| node.receipts.payload),
    )?;
    Ok(match slot_dependency(slot, registry, &mut admit)? {
        Some(dependency) => bank::RecordDescription::with_dependency(usage, dependency),
        None => usage.into(),
    })
}

#[cfg(test)]
pub(crate) fn fixture_history<K>(
    entries: Vec<crate::form::FormPastName<K>>,
) -> Arc<Payload<crate::form::History<K>>> {
    let (own_usage, logical_units) =
        super::transaction::names::describe(&entries, || Ok(())).unwrap();
    Arc::new(Payload {
        identity: custody::MaterializationId::fresh().unwrap(),
        own_usage,
        value: crate::form::History {
            entries,
            logical_units,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::super::compact_tests::AllocationGraph;
    use super::*;

    fn unlimited(_: Usage) -> Result<(), DomError> {
        Ok(())
    }
    fn bounded(limit: usize) -> impl FnMut(Usage) -> Result<(), DomError> {
        move |usage| {
            if usage.bytes() <= limit {
                Ok(())
            } else {
                Err(DomError::ResourceLimit {
                    resource: crate::DomResource::SourceRetainedBytes,
                    limit,
                })
            }
        }
    }

    #[test]
    fn pending_body_admission_is_exact_and_refusal_releases_incoming_data() {
        // This is the new private pending-body gate, not a calibration of the
        // complete registry frontier. The expectation is the actual target.
        let ledger = custody::Ledger::default();
        let extent = Usage::new(size_of::<Payload<[u64; 3]>>(), 0);
        for short in [true, false] {
            let input = custody::Held::new(
                [11u64, 13, 17],
                ledger.retain(Usage::ZERO, unlimited).unwrap(),
            );
            let pending = Pending::new(input, extent, bounded(extent.bytes() - usize::from(short)));
            if short {
                assert!(matches!(pending, Err(DomError::ResourceLimit { .. })));
                assert_eq!(ledger.total(), Usage::ZERO);
            } else {
                let pending = pending.unwrap();
                assert_eq!(pending.value.value, [11u64, 13, 17]);
                assert_eq!(ledger.total(), extent);
                drop(pending);
                assert_eq!(ledger.total(), Usage::ZERO);
            }
        }
    }

    #[test]
    fn isolated_issuer_failure_precedes_payload_retention_and_allocation() {
        let ledger = custody::Ledger::default();
        let input = custody::Held::new(
            [11u64, 13, 17],
            ledger
                .retain(Usage::new(size_of::<[u64; 3]>(), 0), unlimited)
                .unwrap(),
        );
        let failure = identity_refusal::Refusal::begin();
        let result = Pending::new(input, Usage::new(size_of::<Payload<[u64; 3]>>(), 0), |_| {
            panic!("identity refusal precedes new retention")
        });
        assert!(matches!(result, Err(DomError::RepresentationExhausted)));
        assert!(failure.reached());
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn fresh_receipt_transfer_alias_and_historical_adoption_keep_one_payload() {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
        let baseline = ledger.total();
        let own = Usage::new(size_of::<Payload<[u64; 3]>>(), 0);
        let input = custody::Held::new([11u64, 13, 17], ledger.retain(own, unlimited).unwrap());
        let pending = Pending::new(input, own, unlimited).unwrap();
        assert_eq!(
            ledger.total(),
            baseline.checked_add(own).unwrap(),
            "original preadmission is transferred"
        );
        let sealed = pending.seal(&registry, unlimited).unwrap();
        let active = ledger.total();
        let alias = registry
            .adopt_object(sealed.value.identity, sealed.value.own_usage, unlimited)
            .unwrap();
        assert_eq!(
            ledger.total(),
            active,
            "active identity aliases do not readmit payload or control"
        );
        let value = sealed.value.clone();
        let identity = value.identity;
        let mut graph = AllocationGraph::default();
        graph.target(&value, "payload.body");
        registry.inspect_raw_allocations(&mut graph);
        eprintln!(
            "typed_payload active raw_body_and_control={}B/{}alloc conservative={active:?}",
            graph.bytes(),
            graph.allocations()
        );
        drop(alias);
        drop(sealed);
        registry.reconcile_releases();
        assert_eq!(
            ledger.total(),
            baseline,
            "external immutable Arc has no semantic lease"
        );
        let first_gate =
            baseline.bytes() + own.bytes() + custody::SharedDebit::administration_usage().bytes();
        assert!(
            registry
                .adopt_object(identity, own, bounded(first_gate - 1))
                .is_err()
        );
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        assert_eq!(value.identity, identity);
        let adopted = registry.adopt_object(identity, own, unlimited).unwrap();
        assert!(ledger.total().bytes() > first_gate);
        assert_eq!(
            value.identity, identity,
            "historical adoption never mints an identity"
        );
        drop(adopted);
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
        drop(value);
        drop(registry);
        assert_eq!(ledger.total(), Usage::ZERO);
    }

    #[test]
    fn equal_materializations_have_distinct_identities_and_sealing_refusal_cleans_up() {
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
        let baseline = ledger.total();
        let own = Usage::new(size_of::<Payload<[u64; 3]>>(), 0);
        let make = || {
            Pending::new(
                custody::Held::new([1u64, 2, 3], ledger.retain(Usage::ZERO, unlimited).unwrap()),
                own,
                unlimited,
            )
            .unwrap()
        };
        let first = make();
        let second = make();
        assert_ne!(first.value.identity, second.value.identity);
        assert_eq!(
            first.value, second.value,
            "derived identity and receipt are not semantic equality"
        );
        drop(second);
        assert!(first.seal(&registry, |_| Err(DomError::Cancelled)).is_err());
        registry.reconcile_releases();
        assert_eq!(ledger.total(), baseline);
    }
}
