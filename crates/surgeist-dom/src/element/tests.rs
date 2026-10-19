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
}
impl Names {
    fn new() -> Self {
        Self {
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
}
fn native(names: &mut Names, local: &str) -> DomElement {
    let raw = QualifiedName::new(Namespace::html(), None, local);
    crate::names::validate_element_components(&raw, &mut || Ok(())).unwrap();
    DomElement::native(
        names.name(&raw),
        crate::DomDocumentEnvironment::new(crate::DomNativeInputPlatform::headless()),
        &AtomicBool::new(false),
        None,
    )
    .unwrap()
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
fn enumerate(attributes: &AttributeStorage) -> (usize, usize, usize) {
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
    assert_eq!(element.attributes_backing().logical_units(), logical);
    let expected_shallow = element.attributes_backing().payload().map_or(0, |target| {
        std::mem::size_of_val(&**target)
            + element.attributes_backing().capacity() * size_of::<DomAttribute>()
    });
    assert_eq!(
        element.attributes_backing().own_usage().bytes(),
        expected_shallow
    );
    assert_eq!(element.attributes_backing().own_usage().units(), 0);
    let actual = element.attributes_payload().unwrap();
    assert_eq!((actual.bytes, actual.units), (bytes, units));
}
fn changed(names: &mut Names, element: &DomElement, attributes: AttributeStorage) -> DomElement {
    element
        .with_attributes_formed(
            attributes,
            &names.name(&QualifiedName::new(Namespace::none(), None, "data-x")),
            element.nonce().cloned(),
            None,
            &AtomicBool::new(false),
        )
        .unwrap()
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
    let parse = |attributes| {
        DomElement::from_parsed(
            source.name().clone(),
            attributes,
            source.custom_element_state(),
            source.saved_is().cloned(),
            source.nonce().cloned(),
            source.family.clone(),
            source.common_state(),
        )
    };
    let roomy = parse(attributes(&mut names, "", true));
    let tight = parse(AttributeStorage::from_entries(roomy.attributes().to_vec()));
    check(&roomy, 21);
    check(&tight, 21);
    assert_ne!(
        roomy.attributes_backing().own_usage().bytes(),
        tight.attributes_backing().own_usage().bytes()
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
            .same_storage(original.attributes_backing())
    );
    assert_ne!(candidate, original);
    let rejected = candidate.with_attributes_formed(
        attributes(&mut names, "four", false),
        &names.name(&QualifiedName::new(Namespace::none(), None, "data-x")),
        candidate.nonce().cloned(),
        None,
        &AtomicBool::new(true),
    );
    assert_eq!(rejected, Err(DomError::Cancelled));
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
    let (_, bytes, units) = enumerate(populated.attributes_backing());
    let empty_attributes = empty.attributes_payload().unwrap();
    let empty_payload = observe(&empty).1;
    assert_eq!(
        payload.bytes,
        empty_payload.bytes - empty_attributes.bytes + bytes
    );
    assert_eq!(
        payload.units,
        empty_payload.units - empty_attributes.units + units
    );
}
