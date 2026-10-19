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
        self,
        registry: &bank::CustodyRegistry,
        admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Sealed<T>, DomError> {
        self.seal_with_children(registry, None, admit)
    }
    pub fn seal_with_children(
        self,
        registry: &bank::CustodyRegistry,
        dependency: Option<bank::Dependency>,
        mut admit: impl FnMut(custody::Usage) -> Result<(), DomError>,
    ) -> Result<Sealed<T>, DomError> {
        struct Sealing<T> {
            pending: Pending<T>,
            dependency: Option<bank::Dependency>,
        }
        let mut sealing = Sealing {
            pending: self,
            dependency,
        };
        sealing.pending.shared = Some(sealing.pending.debit.share(&mut admit)?);
        let custody = registry.adopt_object_retained_with_children(
            sealing.pending.value.identity,
            sealing
                .pending
                .shared
                .as_ref()
                .expect("shared pending receipt"),
            sealing.dependency.clone(),
            &mut admit,
        )?;
        Ok(Sealed {
            value: sealing.pending.value,
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

struct NameChildren {
    _namespace: Option<bank::ObjectCustody>,
    _local: bank::ObjectCustody,
}
impl NameChildren {
    fn adopt(
        name: &crate::DomQualifiedName,
        registry: &bank::CustodyRegistry,
        names: Option<&super::name_registry::Operation>,
        admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
    ) -> Result<Self, DomError> {
        let mut atom = |value: &crate::DomNameAtom| {
            if let Some(names) = names {
                names.step(&mut || Ok(()))?;
            }
            registry.adopt_object(
                value.identity(),
                crate::name::NameRecord::usage(value.text())?,
                &mut *admit,
            )
        };
        Ok(Self {
            _namespace: name.namespace().map(&mut atom).transpose()?,
            _local: atom(name.local_name_atom())?,
        })
    }
}
#[cfg(test)]
pub(in crate::store) mod attribute_child_observation {
    use std::{cell::Cell, marker::PhantomData, rc::Rc};
    thread_local! { static READS: Cell<Option<usize>> = const { Cell::new(None) }; }
    pub struct Observation(PhantomData<Rc<()>>);
    impl Observation {
        pub fn begin() -> Self {
            READS.with(|value| {
                assert!(value.get().is_none(), "one child observation per thread");
                value.set(Some(0));
            });
            Self(PhantomData)
        }
        pub fn reads(&self) -> usize {
            READS.with(|value| value.get().expect("active child observation"))
        }
    }
    impl Drop for Observation {
        fn drop(&mut self) {
            READS.with(|value| value.set(None));
        }
    }
    pub(super) fn read() {
        READS.with(|value| {
            if let Some(before) = value.get() {
                value.set(Some(before.checked_add(1).expect("observed child reads")));
            }
        });
    }
}
struct AttributeChildren {
    _names: Vec<NameChildren>,
    // The one original dependency debit owns the global extent. This inline
    // receipt constrains that same measured extent locally and drops last.
    _attribution: Option<super::name_registry::Attribution>,
}
pub(in crate::store) fn attribute_dependency(
    value: &crate::name::AttributeValues,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    bank_reserved: bool,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    // This function receives an actual present bank, including a reserved
    // empty bank. Absence never enters it and allocates no child capsule.
    let entries = value.entries();
    let bytes = entries
        .len()
        .checked_mul(size_of::<NameChildren>())
        .and_then(|bytes| bytes.checked_add(size_of::<AttributeChildren>()))
        .ok_or(DomError::RepresentationExhausted)?;
    let receipt = custody::SharedDebit::administration_usage().bytes();
    let local = bytes
        .checked_add(receipt)
        .and_then(|bytes| {
            if bank_reserved {
                Some(bytes)
            } else {
                bytes
                    .checked_add(value.shallow_usage().ok()?.bytes())?
                    .checked_add(receipt)
            }
        })
        .ok_or(DomError::RepresentationExhausted)?;
    let mut attribution = names.map(|names| names.attribute(local)).transpose()?;
    let debit = registry.ledger().retain(Usage::new(bytes, 0), &mut admit)?;
    let mut values = custody::Held::new(Vec::new(), debit);
    values
        .value_mut()
        .try_reserve_exact(entries.len())
        .map_err(|_| DomError::AllocationFailed)?;
    let actual = values
        .value()
        .capacity()
        .checked_mul(size_of::<NameChildren>())
        .and_then(|bytes| bytes.checked_add(size_of::<AttributeChildren>()))
        .ok_or(DomError::RepresentationExhausted)?;
    let extra = Usage::new(actual, 0).checked_sub(values.debit().usage())?;
    if let Some(attribution) = &mut attribution {
        attribution.grow(extra.bytes())?;
    }
    values.debit_mut().grow(extra, &mut admit)?;
    #[cfg(test)]
    if super::scale_metrics::ENABLED && values.value().capacity() != 0 {
        super::scale_metrics::record(super::scale_metrics::Event::StructuralAllocation {
            bytes: values.value().capacity() * size_of::<NameChildren>(),
        });
    }
    #[cfg(test)]
    let mut copied = super::CopyObservation::default();
    for entry in entries {
        #[cfg(test)]
        if super::scale_metrics::ENABLED {
            attribute_child_observation::read();
        }
        values.value_mut().push(NameChildren::adopt(
            entry.name(),
            registry,
            names,
            &mut admit,
        )?);
        #[cfg(test)]
        copied.pushed::<NameChildren>();
    }
    let (names, debit) = values.into_parts();
    bank::Dependency::from_retained(
        custody::Held::new(
            AttributeChildren {
                _names: names,
                _attribution: attribution,
            },
            debit,
        ),
        Usage::new(actual, 0),
        admit,
    )
    .map(Some)
}
pub(in crate::store) fn attach_attribute_extent(
    dependency: &mut bank::Dependency,
    bank: super::name_registry::Attribution,
) {
    dependency
        .value_mut::<AttributeChildren>()
        ._attribution
        .as_mut()
        .expect("new bank has explicit name-operation custody")
        .absorb(bank);
}
struct SlotChildren {
    _form_names: Option<bank::ObjectCustody>,
    _element_name: Option<NameChildren>,
    _attributes: Option<bank::ObjectCustody>,
    _attribution: Option<super::name_registry::Attribution>,
}

#[cfg(test)]
pub(in crate::store) fn inspect_slot_child_extent() -> usize {
    size_of::<SlotChildren>() + size_of::<custody::Debit>()
}

#[cfg(test)]
pub(in crate::store) fn inspect_slot_form_custody(
    slots: &bank::BankCustody<super::Slot>,
) -> Vec<(custody::MaterializationId, Usage)> {
    slots
        .inspect_dependencies::<SlotChildren>()
        .iter()
        .filter_map(|dependency| {
            dependency
                ._form_names
                .as_ref()
                .map(|names| (names.identity(), names.inspect_payload_usage()))
        })
        .collect()
}
#[cfg(test)]
pub(in crate::store) fn inspect_slot_name_custody(
    slots: &bank::BankCustody<super::Slot>,
) -> Vec<(custody::MaterializationId, Usage)> {
    fn names(value: &NameChildren, out: &mut Vec<(custody::MaterializationId, Usage)>) {
        for child in value._namespace.iter().chain([&value._local]) {
            out.push((child.identity(), child.inspect_payload_usage()));
        }
    }
    let mut out = Vec::new();
    for slot in slots.inspect_dependencies::<SlotChildren>() {
        if let Some(value) = &slot._element_name {
            names(value, &mut out);
        }
        if let Some(value) = &slot._attributes {
            out.push((value.identity(), value.inspect_payload_usage()));
            if let Some(children) = value.inspect_dependency::<AttributeChildren>() {
                for value in &children._names {
                    names(value, &mut out);
                }
            }
        }
    }
    out
}
#[cfg(test)]
pub(in crate::store) fn inspect_attribute_child_allocations(
    slots: &bank::BankCustody<super::Slot>,
    graph: &mut super::compact_tests::AllocationGraph,
) {
    for slot in slots.inspect_dependencies::<SlotChildren>() {
        if let Some(value) = &slot._attributes
            && let Some(children) = value.inspect_dependency::<AttributeChildren>()
        {
            graph.allocation(
                &*children as *const AttributeChildren as usize,
                "control.dependency",
                size_of::<AttributeChildren>(),
            );
            graph.buffer(&children._names, "attributes.children.buffer");
        }
    }
}

pub(in crate::store) fn element_dependency(
    kind: &crate::DomNodeKind,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let element = match kind {
        crate::DomNodeKind::Element(element) => Some(element),
        _ => None,
    };
    children_dependency(None, element, registry, names, admit)
}

pub(in crate::store) fn slot_dependency(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let Some(node) = &slot.node else {
        return Ok(None);
    };
    let form = node
        .form_past_names
        .as_ref()
        .and_then(crate::form::FormPastNames::payload)
        .map(|value| registry.adopt_object(value.identity, value.own_usage, &mut admit))
        .transpose()?;
    let element = match &node.kind {
        crate::DomNodeKind::Element(element) => Some(element),
        _ => None,
    };
    children_dependency(form, element, registry, names, admit)
}

fn children_dependency(
    form: Option<bank::ObjectCustody>,
    element: Option<&crate::DomElement>,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    if form.is_none() && element.is_none() {
        return Ok(None);
    }
    let attribution = if element.is_some() {
        let bytes = size_of::<SlotChildren>()
            .checked_add(custody::SharedDebit::administration_usage().bytes())
            .ok_or(DomError::RepresentationExhausted)?;
        names.map(|names| names.attribute(bytes)).transpose()?
    } else {
        None
    };
    let element_name = element
        .map(|element| NameChildren::adopt(element.name(), registry, names, &mut admit))
        .transpose()?;
    let attributes = element
        .and_then(|element| {
            element
                .attributes_backing()
                .payload()
                .map(|value| (element.attributes_backing(), value))
        })
        .map(|(storage, value)| {
            registry.adopt_object_with_children(
                value.identity,
                storage.own_usage(),
                |admit| attribute_dependency(&value.value, registry, names, false, admit),
                &mut admit,
            )
        })
        .transpose()?;
    bank::Dependency::new(
        SlotChildren {
            _form_names: form,
            _element_name: element_name,
            _attributes: attributes,
            _attribution: attribution,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn describe_slot(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<bank::RecordDescription, DomError> {
    let usage = Usage::new(size_of::<super::Slot>(), 0).checked_add(
        slot.node
            .as_ref()
            .map_or(Usage::ZERO, |node| node.receipts.payload),
    )?;
    Ok(match slot_dependency(slot, registry, names, &mut admit)? {
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

    #[test]
    fn attribute_capsule_aliases_skip_children_and_historical_adoption_recreates_exact_children() {
        use super::super::*;
        if !scale_metrics::ENABLED {
            return;
        }
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(
            DomLimits::new(
                32,
                65_536,
                64,
                0,
                0,
                16,
                4_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
        let mut setup = crate::DomBatch::new();
        let document = setup.create_document();
        let element = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.set_attribute(&element, "data-a", "first");
        setup.set_attribute(&element, "data-b", "second");
        let observed = attribute_child_observation::Observation::begin();
        let element = dom
            .apply(setup, &cancel)
            .unwrap()
            .created(&element)
            .unwrap()
            .unwrap();
        assert!(
            observed.reads() >= 2,
            "positive construction reads its actual new attribute children"
        );
        drop(observed);
        let before = dom.snapshot();
        let bank = before
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes_backing()
            .clone();
        let mut edit = crate::DomBatch::new();
        edit.set_nonce(element, "ordinary");
        let observed = attribute_child_observation::Observation::begin();
        dom.apply(edit, &cancel).unwrap();
        assert_eq!(
            observed.reads(),
            0,
            "scalar Slot replacement aliases the original bank capsule"
        );
        drop(observed);
        let observed = attribute_child_observation::Observation::begin();
        let mut source = dom
            .begin_source_edit(
                crate::DomSourceEdit::set_nonce(element, "source"),
                crate::DomSourceLimits::new(16_000_000, 100_000, 2_000_000, 16, 64, 64, 1),
                &cancel,
            )
            .unwrap();
        assert!(matches!(
            source.advance(&cancel).unwrap(),
            crate::DomSourceProgress::Complete(_)
        ));
        drop(source);
        assert_eq!(
            observed.reads(),
            0,
            "Source preflight/restore/replay retain the same attribute capsule"
        );
        drop(observed);
        assert!(
            bank.same_storage(
                dom.snapshot()
                    .node(element)
                    .unwrap()
                    .as_element()
                    .unwrap()
                    .attributes_backing()
            )
        );
        let snapshot = dom.snapshot();
        let slot = snapshot.state.slots.get(element.slot).unwrap().clone();
        drop(dom);
        let ledger = custody::Ledger::default();
        let registry = bank::CustodyRegistry::new(&ledger, unlimited).unwrap();
        let baseline = ledger.total();
        let observed = attribute_child_observation::Observation::begin();
        let dependency = slot_dependency(&slot, &registry, None, unlimited)
            .unwrap()
            .unwrap();
        assert_eq!(
            observed.reads(),
            2,
            "historical re-adoption visits exactly the selected bank children"
        );
        drop(observed);
        assert!(ledger.total().bytes() > baseline.bytes());
        drop(dependency);
        registry.reconcile_releases();
        assert_eq!(
            ledger.total(),
            baseline,
            "last controlled release ends bank/name capsules despite external values"
        );
        assert_eq!(bank.entries()[1].value(), &"second".into());
        assert_eq!(
            snapshot
                .node(element)
                .unwrap()
                .as_element()
                .unwrap()
                .name()
                .local_name(),
            &"div".into()
        );
    }
    #[test]
    fn reserved_empty_attribute_bank_has_exact_local_extent_and_last_alias_release() {
        use super::super::*;
        let cancel = AtomicBool::new(false);
        for source in [false, true] {
            let mut dom = Dom::new(
                DomLimits::new(
                    32,
                    65_536,
                    64,
                    0,
                    0,
                    16,
                    4_000_000,
                    crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
                )
                .unwrap(),
            )
            .unwrap();
            let mut setup = crate::DomBatch::new();
            let document = setup.create_document();
            let document = dom
                .apply(setup, &cancel)
                .unwrap()
                .created(&document)
                .unwrap()
                .unwrap();
            let mut batch = crate::DomBatch::new();
            batch.create_comment(document, "unchanged");
            let draft = if source {
                Draft::new_source(
                    &dom,
                    batch.owner.clone(),
                    &batch.commands,
                    crate::DomSourceLimits::new(32_000_000, 100_000, 4_000_000, 16, 64, 64, 1),
                    &cancel,
                )
                .unwrap()
                .0
            } else {
                Draft::new(&dom, batch.owner.clone(), &cancel).unwrap()
            };
            let baseline = dom.names.inspect_usage();
            // Independent actual layouts, not the production bank receipt or
            // the name-local collector, determine this exact object boundary.
            let bank = size_of::<Payload<crate::name::AttributeValues>>()
                + 7 * size_of::<crate::DomAttribute>();
            let receipt = size_of::<custody::Debit>();
            let extent = bank + size_of::<AttributeChildren>() + 2 * receipt;
            for short in [true, false] {
                dom.names
                    .set_retained_limits(
                        baseline.bytes() + extent - usize::from(short),
                        dom.limits.names().entries(),
                    )
                    .unwrap();
                let formation = source::formation::Formation::new(&draft, &cancel).unwrap();
                let input = draft
                    .attribute_input::<DomError>(7, formation.as_ref(), &cancel, |entries| {
                        source::formation::reserve(entries, 7, formation.as_ref(), &cancel)
                    })
                    .unwrap();
                assert_eq!(input.entries.capacity(), 7, "owning reserve_exact premise");
                assert_eq!(
                    dom.names.inspect_usage().bytes(),
                    baseline.bytes() + bank + receipt,
                    "bank reserve is locally owned before initialization"
                );
                let formed = draft.form_attributes(input, formation.as_ref(), &cancel);
                if short {
                    assert!(matches!(
                        formed,
                        Err(DomError::ResourceLimit {
                            resource: DomResource::NameRetainedBytes,
                            ..
                        })
                    ));
                    assert_eq!(dom.names.inspect_usage(), baseline);
                } else {
                    let formed = formed.unwrap();
                    assert!(formed.value.entries().is_empty());
                    assert_eq!(formed.value.capacity(), 7);
                    assert_eq!(dom.names.inspect_usage().bytes(), baseline.bytes() + extent);
                    let external = formed.value.clone();
                    let alias = formed.custody.clone();
                    drop(formed);
                    assert_eq!(dom.names.inspect_usage().bytes(), baseline.bytes() + extent);
                    drop(alias);
                    dom.custody_registry.reconcile_releases();
                    assert_eq!(
                        dom.names.inspect_usage(),
                        baseline,
                        "only last controlled release ends local custody"
                    );
                    assert_eq!(
                        external.capacity(),
                        7,
                        "external immutable bank remains readable"
                    );
                }
                drop(formation);
            }
        }
    }
}
