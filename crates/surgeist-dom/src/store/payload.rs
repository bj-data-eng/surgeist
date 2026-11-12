//! Immutable typed materializations. Semantics contain identity and shallow
//! receipts; controlled owners keep the matching custody separately.
mod children;
pub(in crate::store) use children::{
    attr_data_dependency, data_dependency, document_dependency, element_node_dependency,
    form_dependency, fragment_dependency, image_association_dependency, shadow_dependency,
    text_dependency,
};
#[cfg(test)]
pub(in crate::store) use children::{form_dependency_refusal, inspect_slot_typed_custody};
#[cfg(test)]
mod encoding_tests;
#[cfg(test)]
mod group_tests;
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
        control: Option<Box<dyn Fn() -> bool>>,
        control_observed: Rc<Cell<Option<bool>>>,
    }
    thread_local! { static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) }; }

    pub(in crate::store) struct Observation {
        observed: Rc<Cell<Option<Usage>>>,
        control_observed: Rc<Cell<Option<bool>>>,
        _thread: PhantomData<Rc<()>>,
    }
    impl Observation {
        pub fn begin(identity: custody::MaterializationId, ledger: &custody::Ledger) -> Self {
            Self::start(identity, ledger, None)
        }
        pub fn begin_with_control(
            identity: custody::MaterializationId,
            ledger: &custody::Ledger,
            control: impl Fn() -> bool + 'static,
        ) -> Self {
            Self::start(identity, ledger, Some(Box::new(control)))
        }
        fn start(
            identity: custody::MaterializationId,
            ledger: &custody::Ledger,
            control: Option<Box<dyn Fn() -> bool>>,
        ) -> Self {
            let observed = Rc::new(Cell::new(None));
            let control_observed = Rc::new(Cell::new(None));
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
                    control,
                    control_observed: control_observed.clone(),
                });
            });
            Self {
                observed,
                control_observed,
                _thread: PhantomData,
            }
        }
        pub fn observed(&self) -> Option<Usage> {
            self.observed.get()
        }
        pub fn control_alive_at_destruction(&self) -> Option<bool> {
            self.control_observed.get()
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
                if let Some(control) = &watch.control {
                    watch.control_observed.set(Some(control()));
                }
            }
        });
    }
}

/// A freshly materialized body awaiting controlled registration. Its identity
/// is never copied from an alias. Both the affine and shared receipt follow
/// data in destruction order; selected mutation finishes before this phase.
pub(crate) struct Pending<T> {
    value: Arc<Payload<T>>,
    debit: custody::Debit,
    shared: Option<custody::SharedDebit>,
}
impl<T> Pending<T> {
    /// Transfer an unregistered target and its original affine receipt into
    /// the one enclosing Slot owner. This does not publish a semantic alias.
    pub(in crate::store) fn into_unregistered(self) -> custody::Held<Arc<Payload<T>>> {
        assert!(
            self.shared.is_none(),
            "unregistered target has no shared receipt"
        );
        let Self {
            value,
            debit,
            shared: _,
        } = self;
        custody::Held::new(value, debit)
    }

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

/// Construct one exact leaf under its original allocation credit. The caller
/// identifies already formed occurrences separately from new storage.
/// Both original and added credit stay live until the factory's owned inputs
/// have been destroyed, including refusal before the factory can run.
pub(crate) fn form_group<T>(
    registry: &bank::CustodyRegistry,
    budget: Option<&super::source::formation::Formation>,
    original: super::source::storage::Payload,
    own_usage: Usage,
    admit: &mut dyn FnMut(Usage) -> Result<(), DomError>,
    build: impl FnOnce() -> Result<T, DomError>,
) -> Result<Sealed<T>, DomError> {
    let reserved = Reserved::new()?;
    struct Building<F> {
        build: F,
        debit: custody::Debit,
    }
    let debit = match budget {
        Some(budget) => {
            let available = budget.owned_payload();
            Usage::new(available.bytes, available.units)
                .checked_sub(Usage::new(original.bytes, original.units))?;
            budget.split_payload(original).into_debit()
        }
        None => None,
    }
    .map_or_else(|| registry.ledger().retain(Usage::ZERO, &mut *admit), Ok)?;
    let mut building = Building { build, debit };
    assert!(
        building.debit.belongs_to(registry.ledger()),
        "formed leaf retains its original custody ledger"
    );
    let extra = own_usage.checked_sub(building.debit.usage())?;
    building.debit.grow(extra, &mut *admit)?;
    let Building { build, debit } = building;
    let incoming = custody::Held::new(build()?, debit);
    reserved
        .materialize(incoming, own_usage, &mut *admit)?
        .seal(registry, admit)
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
struct ElementNameChildren {
    _names: NameChildren,
    _attribution: Option<super::name_registry::Attribution>,
}
struct ElementLanguageChildren {
    _name: bank::ObjectCustody,
    _native: Option<bank::ObjectCustody>,
    _custom: Option<bank::ObjectCustody>,
    _nonce: Option<bank::ObjectCustody>,
    _focus: Option<bank::ObjectCustody>,
    _hosted: Option<bank::ObjectCustody>,
    // The actual dedicated language/name-containing capsule remains local;
    // generic exact-kind and Slot capsules are global storage only.
    _attribution: Option<super::name_registry::Attribution>,
}

fn containing_attribution<T, D>(
    names: Option<&super::name_registry::Operation>,
) -> Result<Option<super::name_registry::Attribution>, DomError> {
    let bytes = size_of::<Payload<T>>()
        .checked_add(size_of::<D>())
        .and_then(|bytes| bytes.checked_add(2 * size_of::<custody::Debit>()))
        .ok_or(DomError::RepresentationExhausted)?;
    // Conservative text occurrences stay in the global shallow receipt; the
    // local ceiling follows only actual dedicated containing target/metadata.
    names.map(|names| names.attribute(bytes)).transpose()
}

pub(in crate::store) fn element_name_dependency(
    value: &crate::element::ElementName,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    qualified_name_dependency::<crate::element::ElementName>(&value.value, registry, names, admit)
}

pub(in crate::store) fn attr_dependency(
    value: &crate::DomAttribute,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    qualified_name_dependency::<crate::DomAttribute>(value.name(), registry, names, admit)
}

fn qualified_name_dependency<T>(
    name: &crate::DomQualifiedName,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let attribution = containing_attribution::<T, ElementNameChildren>(names)?;
    let children = NameChildren::adopt(name, registry, names, &mut admit)?;
    bank::Dependency::new(
        ElementNameChildren {
            _names: children,
            _attribution: attribution,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn element_dependency(
    element: &crate::DomElement,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    let local = size_of::<ElementLanguageChildren>()
        .checked_add(custody::SharedDebit::administration_usage().bytes())
        .ok_or(DomError::RepresentationExhausted)?;
    let attribution = names.map(|names| names.attribute(local)).transpose()?;
    let value = element.name_payload();
    let name = registry.adopt_object_with_children(
        value.identity,
        value.own_usage,
        |admit| element_name_dependency(&value.value, registry, names, admit),
        &mut admit,
    )?;
    let native = match element.native_family() {
        crate::element::NativeFamily::Dialog(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Style(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Fieldset(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Button(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Input(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Textarea(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Output(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Select(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        crate::element::NativeFamily::Canvas(value) => {
            Some(children::leaf(value, registry, &mut admit)?)
        }
        _ => None,
    };
    let custom = element
        .custom_payload()
        .map(|value| children::leaf(value, registry, &mut admit))
        .transpose()?;
    let nonce = element
        .nonce_payload()
        .map(|value| children::leaf(value, registry, &mut admit))
        .transpose()?;
    let focus = element
        .focus_payload()
        .map(|value| children::leaf(value, registry, &mut admit))
        .transpose()?;
    let hosted = element
        .hosted_payload()
        .map(|value| children::leaf(value, registry, &mut admit))
        .transpose()?;
    bank::Dependency::new(
        ElementLanguageChildren {
            _name: name,
            _native: native,
            _custom: custom,
            _nonce: nonce,
            _focus: focus,
            _hosted: hosted,
            _attribution: attribution,
        },
        registry.ledger(),
        admit,
    )
    .map(Some)
}

pub(in crate::store) fn slot_dependency(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<Option<bank::Dependency>, DomError> {
    match slot.node.as_ref() {
        Some(node) => children::node_children(node, registry, names, &mut admit),
        None => Ok(None),
    }
}

#[cfg(test)]
pub(in crate::store) fn inspect_element_language_child_extent() -> usize {
    size_of::<ElementLanguageChildren>() + size_of::<custody::Debit>()
}

/// Raw concrete target layout for independent name-local quota witnesses.
/// Receipt metadata and the name payload target are enumerated by the caller.
#[cfg(test)]
pub(in crate::store) fn inspect_element_name_children_body_bytes() -> usize {
    size_of::<ElementNameChildren>()
}

#[cfg(test)]
pub(in crate::store) fn inspect_slot_form_custody(
    slots: &bank::BankCustody<super::Slot>,
) -> Vec<(custody::MaterializationId, Usage)> {
    let mut out = Vec::new();
    for dependency in children::inspect_slot_dependencies(slots) {
        if let children::SlotDependency::Element(element) = dependency
            && let Some(association) = &element._association
            && let Some(form) = association.inspect_dependency::<children::FormChildren>()
        {
            out.push((
                form._past_names.identity(),
                form._past_names.inspect_payload_usage(),
            ));
        }
    }
    out
}
#[cfg(test)]
fn inspect_element_children(
    slots: &bank::BankCustody<super::Slot>,
) -> Vec<bank::Retained<ElementLanguageChildren>> {
    let mut out = Vec::new();
    for dependency in children::inspect_slot_dependencies(slots) {
        if let children::SlotDependency::Element(element) = dependency
            && let Some(language) = element
                ._element
                .inspect_dependency::<ElementLanguageChildren>()
        {
            out.push(language);
        }
    }
    out
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
    for element in inspect_element_children(slots) {
        if let Some(value) = element._name.inspect_dependency::<ElementNameChildren>() {
            names(&value._names, &mut out);
        }
    }
    for dependency in children::inspect_slot_dependencies(slots) {
        if let children::SlotDependency::Data(data) = dependency
            && let Some(value) = data._data.inspect_dependency::<ElementNameChildren>()
        {
            names(&value._names, &mut out);
        }
    }
    out
}
pub(in crate::store) fn describe_slot(
    slot: &super::Slot,
    registry: &bank::CustodyRegistry,
    names: Option<&super::name_registry::Operation>,
    mut admit: impl FnMut(Usage) -> Result<(), DomError>,
) -> Result<bank::RecordDescription, DomError> {
    let usage = super::source::storage::slot_usage(slot)?;
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
}
