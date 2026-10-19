use super::*;
use crate::name::NameRecord;
use crate::store::MaterializationId;
use crate::{DomNameAtom, QualifiedName};
use std::collections::HashMap;
use std::{mem::size_of, sync::Arc};

// These receipt fixtures construct valid canonical values with one record per
// owner/spelling. Actual registry admission/lifecycle is tested by its owner.
struct Names {
    owner: crate::DomOwnerId,
    atoms: HashMap<DocumentString, DomNameAtom>,
    registry: crate::store::bank::CustodyRegistry,
    controls: Vec<crate::store::bank::ObjectCustody>,
    children: Vec<ElementChildren>,
}
impl Names {
    fn new() -> Self {
        let ledger = crate::store::custody::Ledger::default();
        let registry = crate::store::bank::CustodyRegistry::new(&ledger, |_| Ok(())).unwrap();
        Self {
            registry,
            controls: Vec::new(),
            children: Vec::new(),
            owner: crate::DomOwnerId::allocate().unwrap(),
            atoms: HashMap::new(),
        }
    }
    fn atom(&mut self, text: &DocumentString) -> DomNameAtom {
        let row = self.atoms.len();
        self.atoms
            .entry(text.clone())
            .or_insert_with(|| {
                DomNameAtom::from_record(Arc::new(NameRecord::new(
                    self.owner,
                    text.clone(),
                    MaterializationId::fresh().unwrap(),
                    row,
                )))
            })
            .clone()
    }
    fn name(&mut self, raw: &QualifiedName) -> DomQualifiedName {
        DomQualifiedName::from_atoms(
            raw.namespace().uri().map(|text| self.atom(text)),
            raw.prefix().cloned(),
            self.atom(raw.local_name()),
        )
        .unwrap()
    }
    fn name_target(&mut self, name: DomQualifiedName) -> Arc<Payload<ElementName>> {
        let text = name.retained_payload().unwrap();
        let usage = crate::store::custody::Usage::new(
            size_of::<Payload<ElementName>>() + text.bytes,
            text.units,
        );
        let formed = crate::store::payload::form_group(
            &self.registry,
            None,
            crate::store::source::storage::Payload::default(),
            usage,
            &mut |_| Ok(()),
            || Ok(ElementName { value: name }),
        )
        .unwrap();
        self.controls.push(formed.custody);
        formed.value
    }
    fn binding(&mut self, storage: AttributeStorage) -> Option<Arc<Payload<AttributeBinding>>> {
        storage.payload()?;
        let text = storage.text_payload();
        let usage = crate::store::custody::Usage::new(
            size_of::<Payload<AttributeBinding>>() + text.bytes,
            text.units,
        );
        let formed = crate::store::payload::form_group(
            &self.registry,
            None,
            crate::store::source::storage::Payload::default(),
            usage,
            &mut |_| Ok(()),
            || Ok(AttributeBinding { storage }),
        )
        .unwrap();
        self.controls.push(formed.custody);
        Some(formed.value)
    }
}
fn native(names: &mut Names, local: &str) -> DomElement {
    let raw = QualifiedName::new(Namespace::html(), None, local);
    crate::names::validate_element_components(&raw, &mut || Ok(())).unwrap();
    let name = names.name(&raw);
    let name = names.name_target(name);
    let cancel = AtomicBool::new(false);
    let formed = DomElement::native(
        name,
        crate::DomDocumentEnvironment::new(crate::DomNativeInputPlatform::headless()),
        ElementFormation::new(&names.registry, None, &cancel, &mut |_| Ok(())),
    )
    .unwrap();
    names.children.push(formed.children);
    formed.value
}
fn string(units: &[u16], spare: usize) -> DocumentString {
    let mut backing = Vec::with_capacity(units.len() + spare);
    backing.extend_from_slice(units);
    DocumentString::from_utf16(backing)
}
fn attributes(names: &mut Names, value: &str, second: bool) -> AttributeStorage {
    let mut attributes = Vec::with_capacity(12);
    attributes.push(DomAttribute::from_parts(
        names.name(&QualifiedName::new(Namespace::none(), None, "data-x")),
        if value.is_empty() {
            string(&[0x61, 0xd800, 0x62], 9)
        } else {
            string(&value.encode_utf16().collect::<Vec<_>>(), 7)
        },
    ));
    if second {
        attributes.push(DomAttribute::from_parts(
            names.name(&QualifiedName::new(
                Namespace::from_uri("urn:a"),
                Some("p".into()),
                "flag",
            )),
            "xy".into(),
        ));
    }
    AttributeStorage::from_entries(attributes)
}
/// Independent enumeration uses only the immutable producer's actual backing.
fn enumerate(attributes: Option<&AttributeStorage>) -> (usize, usize, usize) {
    let Some(attributes) = attributes else {
        return (0, 0, 0);
    };
    let mut logical = 0;
    let mut units = 0;
    let mut bytes = 0;
    for attribute in attributes.iter() {
        let name = attribute.name();
        for value in name
            .namespace_uri()
            .into_iter()
            .chain(name.prefix())
            .chain([name.local_name(), attribute.value()])
        {
            logical += value.len();
            let capacity = value.shared_code_units().capacity();
            units += capacity;
            bytes += size_of::<Vec<u16>>() + capacity * size_of::<u16>();
        }
    }
    (logical, bytes, units)
}
fn check(element: &DomElement, expected_logical: usize) {
    let (logical, bytes, units) = enumerate(element.attributes_backing());
    assert_eq!(logical, expected_logical);
    assert_eq!(
        element
            .attributes_backing()
            .map_or(0, AttributeStorage::logical_units),
        logical
    );
    let expected_shallow = element
        .attributes_backing()
        .and_then(AttributeStorage::payload)
        .map_or(0, |target| {
            std::mem::size_of_val(&**target)
                + element
                    .attributes_backing()
                    .map_or(0, AttributeStorage::capacity)
                    * size_of::<DomAttribute>()
        });
    assert_eq!(
        element
            .attributes_backing()
            .map_or(0, |value| value.own_usage().bytes()),
        expected_shallow
    );
    assert_eq!(
        element
            .attributes_backing()
            .map_or(0, |value| value.own_usage().units()),
        0
    );
    let actual = element.attributes_payload().unwrap();
    assert_eq!((actual.bytes, actual.units), (bytes, units));
}
fn changed(names: &mut Names, element: &DomElement, attributes: AttributeStorage) -> DomElement {
    let binding = names.binding(attributes);
    let changed = names.name(&QualifiedName::new(Namespace::none(), None, "data-x"));
    let cancel = AtomicBool::new(false);
    let formed = element
        .with_attributes_formed(
            binding,
            &changed,
            NonceUpdate::Unchanged,
            ElementFormation::new(&names.registry, None, &cancel, &mut |_| Ok(())),
        )
        .unwrap();
    names.children.push(formed.children);
    formed.value
}

#[test]
fn new_attribute_backings_keep_exact_receipts_through_add_replace_and_remove() {
    let mut names = Names::new();
    let empty = native(&mut names, "div");
    check(&empty, 0);
    let attrs = attributes(&mut names, "", true);
    let added = changed(&mut names, &empty, attrs);
    check(&added, 21);
    let attrs = attributes(&mut names, "four", true);
    let replaced = changed(&mut names, &added, attrs);
    check(&replaced, 22);
    let attrs = attributes(&mut names, "four", false);
    let removed = changed(&mut names, &replaced, attrs);
    check(&removed, 10);
    let cleared = changed(&mut names, &removed, AttributeStorage::default());
    check(&cleared, 0);
    assert_eq!(added.attributes().len(), 2);
    assert_eq!(replaced.attributes().len(), 2);
    assert_eq!(removed.attributes().len(), 1);
    assert_eq!(cleared.attributes().len(), 0);
    assert_eq!(added.units().unwrap(), empty.units().unwrap() + 21);
    assert_eq!(replaced.units().unwrap(), empty.units().unwrap() + 22);
}

#[test]
fn parsed_receipts_and_semantic_equality_preserve_actual_producer_capacity() {
    let mut names = Names::new();
    let source = native(&mut names, "div");
    let parse = |names: &mut Names, attributes| {
        let attributes = names.binding(attributes);
        let name = names.name_target(source.name().clone());
        let cancel = AtomicBool::new(false);
        let formed = DomElement::from_parsed(
            name,
            attributes,
            source.custom_element_state(),
            source.saved_is().cloned(),
            source.nonce().cloned(),
            source.family.clone(),
            source.common_state(),
            ElementFormation::new(&names.registry, None, &cancel, &mut |_| Ok(())),
        )
        .unwrap();
        names.children.push(formed.children);
        formed.value
    };
    let attrs = attributes(&mut names, "", true);
    let roomy = parse(&mut names, attrs);
    let tight = parse(
        &mut names,
        AttributeStorage::from_entries(roomy.attributes().to_vec()),
    );
    check(&roomy, 21);
    check(&tight, 21);
    assert_ne!(
        roomy.attributes_backing().unwrap().own_usage().bytes(),
        tight.attributes_backing().unwrap().own_usage().bytes()
    );
    assert_eq!(roomy, tight);
}

#[test]
fn scalar_family_clones_and_refused_mutations_preserve_the_original_receipt() {
    let mut names = Names::new();
    let base = native(&mut names, "marquee");
    let attrs = attributes(&mut names, "", true);
    let original = changed(&mut names, &base, attrs);
    let mut candidate = original.clone();
    candidate.set_marquee_running(true).unwrap();
    candidate.set_captured_in_view_transition(true);
    check(&candidate, 21);
    check(&original, 21);
    assert!(
        candidate
            .attributes_backing()
            .unwrap()
            .same_storage(original.attributes_backing().unwrap())
    );
    assert_ne!(candidate, original);
    let attrs = attributes(&mut names, "four", false);
    let binding = names.binding(attrs);
    let changed_name = names.name(&QualifiedName::new(Namespace::none(), None, "data-x"));
    let cancel = AtomicBool::new(true);
    let rejected = candidate.with_attributes_formed(
        binding,
        &changed_name,
        NonceUpdate::Unchanged,
        ElementFormation::new(&names.registry, None, &cancel, &mut |_| Ok(())),
    );
    assert!(matches!(rejected, Err(DomError::Cancelled)));
    check(&candidate, 21);
    let base = native(&mut names, "div");
    let attrs = attributes(&mut names, "four", false);
    let mut common = changed(&mut names, &base, attrs);
    assert_eq!(
        common.set_marquee_running(true),
        Err(DomError::MarqueeInapplicable)
    );
    check(&common, 10);
}

#[test]
fn unchanged_attribute_payload_observation_has_fixed_work() {
    let mut names = Names::new();
    let empty = native(&mut names, "div");
    let attributes = AttributeStorage::from_entries(
        (0..128)
            .map(|index| {
                DomAttribute::from_parts(
                    names.name(&QualifiedName::new(
                        Namespace::none(),
                        None,
                        format!("data-{index}"),
                    )),
                    "payload".into(),
                )
            })
            .collect::<Vec<_>>(),
    );
    let populated = changed(&mut names, &empty, attributes);
    let observe = |element: &DomElement| {
        let mut steps = 0;
        let payload = element
            .retained_payload(|| {
                steps += 1;
                Ok(())
            })
            .unwrap();
        (steps, payload)
    };
    let (empty_steps, _) = observe(&empty);
    let (populated_steps, payload) = observe(&populated);
    assert_eq!(populated_steps, empty_steps);
    // Independently enumerate the occurrence target, bank target and actual
    // destination capacity. The diagnostic aggregate includes these in
    // addition to each attribute's retained string occurrences.
    let raw_attributes = |element: &DomElement| {
        let (_, text_bytes, units) = enumerate(element.attributes_backing());
        let bytes = element.attribute_binding().map_or(0, |binding| {
            size_of::<crate::store::payload::Payload<AttributeBinding>>()
                + binding.value.storage.payload().map_or(0, |_| {
                    size_of::<crate::store::payload::Payload<crate::name::AttributeValues>>()
                        + binding.value.storage.capacity() * size_of::<DomAttribute>()
                })
                + text_bytes
        });
        (bytes, units)
    };
    let (bytes, units) = raw_attributes(&populated);
    let (empty_bytes, empty_units) = raw_attributes(&empty);
    let empty_payload = observe(&empty).1;
    assert_eq!(payload.bytes, empty_payload.bytes - empty_bytes + bytes);
    assert_eq!(payload.units, empty_payload.units - empty_units + units);
}

#[test]
fn unrelated_attributes_alias_the_name_and_exact_native_group_without_admission() {
    let mut names = Names::new();
    let original = native(&mut names, "input");
    let attrs = attributes(&mut names, "literal", false);
    let binding = names.binding(attrs);
    let changed = names.name(&QualifiedName::new(Namespace::none(), None, "data-x"));
    let cancel = AtomicBool::new(false);
    let formed = original
        .with_attributes_formed(
            binding,
            &changed,
            NonceUpdate::Unchanged,
            ElementFormation::new(&names.registry, None, &cancel, &mut |_| {
                panic!("unchanged semantic groups acquire no new payload or control")
            }),
        )
        .unwrap();
    let edited = &formed.value;
    assert!(Arc::ptr_eq(original.name_payload(), edited.name_payload()));
    assert!(Arc::ptr_eq(
        original.input_payload().unwrap(),
        edited.input_payload().unwrap()
    ));
    assert_eq!(original.input_state(), edited.input_state());
    assert_eq!(original.attributes(), &[]);
    assert_eq!(
        edited.attributes()[0].value().code_units(),
        "literal".encode_utf16().collect::<Vec<_>>()
    );
    assert!(formed.children.native.is_none());
    assert!(formed.children.nonce.is_none());
}
