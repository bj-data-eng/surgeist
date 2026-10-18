use super::*;
use crate::Attribute;
use std::{mem::size_of, sync::Arc};

fn native(local: &str) -> DomElement {
    DomElement::native_components(
        &QualifiedName::new(Namespace::html(), None, local),
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
fn attributes(value: &str, second: bool) -> OrderedAttributes {
    let mut attributes = Vec::with_capacity(12);
    attributes.push(Attribute::new(
        QualifiedName::new(Namespace::none(), None, "data-x"),
        if value.is_empty() {
            string(&[0x61, 0xd800, 0x62], 9)
        } else {
            string(&value.encode_utf16().collect::<Vec<_>>(), 7)
        },
    ));
    if second {
        attributes.push(Attribute::new(
            QualifiedName::new(Namespace::from_uri("urn:a"), Some("p".into()), "flag"),
            "xy",
        ));
    }
    OrderedAttributes::new(attributes).unwrap()
}
/// Independent enumeration uses only the immutable producer's actual backing.
fn enumerate(attributes: &OrderedAttributes) -> (usize, usize, usize) {
    let mut logical = 0;
    let mut units = 0;
    let mut bytes = size_of::<Vec<Attribute>>()
        + attributes.shared_attributes().capacity() * size_of::<Attribute>();
    for attribute in attributes.iter() {
        let name = attribute.name();
        for value in name
            .namespace()
            .uri()
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
    let (logical, bytes, units) = enumerate(element.attributes());
    assert_eq!(logical, expected_logical);
    assert_eq!(element.attribute_receipt.logical_units, Some(logical));
    let actual = element.attributes_payload().unwrap();
    assert_eq!((actual.bytes, actual.units), (bytes, units));
}
fn changed(element: &DomElement, attributes: OrderedAttributes) -> DomElement {
    element
        .with_attributes_formed(
            attributes,
            &QualifiedName::new(Namespace::none(), None, "data-x"),
            element.nonce().cloned(),
            None,
            &AtomicBool::new(false),
        )
        .unwrap()
}

#[test]
fn new_attribute_backings_keep_exact_receipts_through_add_replace_and_remove() {
    let empty = native("div");
    check(&empty, 0);
    let added = changed(&empty, attributes("", true));
    check(&added, 21);
    let replaced = changed(&added, attributes("four", true));
    check(&replaced, 22);
    let removed = changed(&replaced, attributes("four", false));
    check(&removed, 10);
    let cleared = changed(&removed, OrderedAttributes::default());
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
    let source = native("div");
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
    let roomy = parse(attributes("", true));
    let tight = parse(
        OrderedAttributes::new(roomy.attributes().iter().cloned().collect::<Vec<_>>()).unwrap(),
    );
    check(&roomy, 21);
    check(&tight, 21);
    assert_ne!(
        roomy.attributes_payload().unwrap().bytes,
        tight.attributes_payload().unwrap().bytes
    );
    assert_eq!(roomy, tight);
}

#[test]
fn scalar_family_clones_and_refused_mutations_preserve_the_original_receipt() {
    let original = changed(&native("marquee"), attributes("", true));
    let mut candidate = original.clone();
    candidate.set_marquee_running(true).unwrap();
    candidate.set_captured_in_view_transition(true);
    check(&candidate, 21);
    check(&original, 21);
    assert!(Arc::ptr_eq(
        candidate.attributes().shared_attributes(),
        original.attributes().shared_attributes()
    ));
    assert_ne!(candidate, original);
    let rejected = candidate.with_attributes_formed(
        attributes("four", false),
        &QualifiedName::new(Namespace::none(), None, "data-x"),
        candidate.nonce().cloned(),
        None,
        &AtomicBool::new(true),
    );
    assert_eq!(rejected, Err(DomError::Cancelled));
    check(&candidate, 21);
    let mut common = changed(&native("div"), attributes("four", false));
    assert_eq!(
        common.set_marquee_running(true),
        Err(DomError::MarqueeInapplicable)
    );
    check(&common, 10);
}

#[test]
fn unchanged_attribute_payload_observation_has_fixed_work() {
    let empty = native("div");
    let attributes = OrderedAttributes::new((0..128).map(|index| {
        Attribute::new(
            QualifiedName::new(Namespace::none(), None, format!("data-{index}")),
            "payload",
        )
    }))
    .unwrap();
    let populated = changed(&empty, attributes);
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
    let (_, bytes, units) = enumerate(populated.attributes());
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
