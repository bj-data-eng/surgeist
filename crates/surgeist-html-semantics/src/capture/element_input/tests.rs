use super::*;
use std::{cell::Cell, convert::Infallible};

fn limits(bytes: usize) -> HtmlCaptureLimits {
    HtmlCaptureLimits::new(20, 100, 100, 10_000, 20, 0, 0, bytes)
}
fn builder(bytes: usize) -> (HtmlCaptureBuilder, HtmlCaptureBuilderSubject) {
    let mut builder = HtmlCaptureBuilder::new(limits(bytes), HtmlHostFactsRevision::fresh());
    let doc = builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    (builder, doc)
}
fn qname(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::none(), None, local)
}
fn string_bytes(value: &DocumentString) -> usize {
    size_of::<Vec<u16>>() + value.shared_code_units().capacity() * size_of::<u16>()
}
fn name_bytes(value: &QualifiedName) -> usize {
    value.namespace().uri().map_or(0, string_bytes)
        + value.prefix().map_or(0, string_bytes)
        + string_bytes(value.local_name())
}
fn raw_element_bytes(
    name: &QualifiedName,
    values: &[Attribute],
    capacity: usize,
    saved: Option<&DocumentString>,
) -> usize {
    name_bytes(name)
        + size_of::<Vec<Attribute>>()
        + capacity * size_of::<Attribute>()
        + values
            .iter()
            .map(|a| name_bytes(a.name()) + string_bytes(a.value()))
            .sum::<usize>()
        + saved.map_or(0, string_bytes)
}

#[test]
fn owned_values_admit_actual_reserved_capacity_exactly_and_refuse_one_byte_short() {
    let name = qname("x");
    let mut values = Vec::with_capacity(17);
    values.push(Attribute::new(qname("a"), "value"));
    let attrs = OrderedAttributes::new(values).unwrap();
    let saved = DocumentString::from("saved");
    let expected = raw_element_bytes(
        &name,
        attrs.as_slice(),
        attrs.shared_attributes().capacity(),
        Some(&saved),
    );
    let (mut owner, doc) = builder(expected - 1);
    let before = owner.usage();
    assert_eq!(
        owner.add_element(
            &doc,
            name.clone(),
            attrs.clone(),
            HtmlCapturedIsValue::Value(saved.clone()),
            HtmlCaptureRegistryInput::Null
        ),
        Err(HtmlCaptureError::LimitExceeded {
            resource: HtmlCaptureResource::ElementValueBytes,
            limit: expected - 1,
            required: expected
        })
    );
    assert_eq!(owner.usage(), before);
    owner.limits = limits(expected);
    let subject = owner
        .add_element(
            &doc,
            name.clone(),
            attrs.clone(),
            HtmlCapturedIsValue::Value(saved),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    assert_eq!(owner.usage().element_value_bytes(), expected);
    let HtmlCapturedNodeKind::Element(value) = &owner.data.records[subject.index].kind else {
        panic!("element")
    };
    assert!(Arc::ptr_eq(
        value.attributes.shared_attributes(),
        attrs.shared_attributes()
    ));
    assert!(Arc::ptr_eq(
        value.name.local_name().shared_code_units(),
        name.local_name().shared_code_units()
    ));
    assert!(attrs.shared_attributes().capacity() > attrs.len());
}

#[test]
fn borrowed_empty_values_have_an_exact_same_builder_byte_gate_and_cumulative_receipts() {
    let name = qname("x");
    let attrs: &[Attribute] = &[];
    let bytes = raw_element_bytes(&name, attrs, 0, None);
    let (mut owner, doc) = builder(bytes - 1);
    let before = owner.usage();
    let add = |owner: &mut HtmlCaptureBuilder| {
        owner.add_element_from_inputs(
            &doc,
            &name,
            attrs,
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(()),
        )
    };
    assert!(
        matches!(add(&mut owner), Err(HtmlCaptureElementError::Capture(HtmlCaptureError::LimitExceeded { resource: HtmlCaptureResource::ElementValueBytes, limit, required })) if limit == bytes - 1 && required == bytes)
    );
    assert_eq!(owner.usage(), before);
    owner.limits = limits(bytes);
    let first = add(&mut owner).unwrap();
    assert_eq!(owner.usage().element_value_bytes(), bytes);
    let before = owner.usage();
    assert!(
        matches!(add(&mut owner), Err(HtmlCaptureElementError::Capture(HtmlCaptureError::LimitExceeded { resource: HtmlCaptureResource::ElementValueBytes, required, .. })) if required == 2 * bytes)
    );
    assert_eq!(owner.usage(), before);
    owner.limits = limits(2 * bytes);
    let second = add(&mut owner).unwrap();
    assert_ne!(first, second);
    assert_eq!(owner.usage().element_value_bytes(), 2 * bytes);
    assert_eq!(owner.usage().subjects(), 3);
}

#[test]
fn borrowed_values_preserve_order_text_backings_and_saved_is() {
    let name = QualifiedName::new(Namespace::html(), Some("p".into()), "input");
    let attrs = [
        Attribute::new(
            qname("z"),
            DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
        ),
        Attribute::new(qname("a"), "last"),
    ];
    let saved = DocumentString::from("custom");
    let bytes = raw_element_bytes(&name, &attrs, attrs.len(), Some(&saved));
    let (mut owner, doc) = builder(bytes);
    let subject = owner
        .add_element_from_inputs(
            &doc,
            &name,
            attrs.as_slice(),
            HtmlCapturedIsValue::Value(saved.clone()),
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(()),
        )
        .unwrap();
    let HtmlCapturedNodeKind::Element(value) = &owner.data.records[subject.index].kind else {
        panic!("element")
    };
    assert_eq!(value.name(), &name);
    assert_eq!(value.attributes().as_slice(), attrs);
    assert_eq!(value.is_value(), &HtmlCapturedIsValue::Value(saved));
    assert!(Arc::ptr_eq(
        value.attributes().as_slice()[0].value().shared_code_units(),
        attrs[0].value().shared_code_units()
    ));
    assert_eq!(owner.usage().element_value_bytes(), bytes);
}

#[test]
fn open_duplicate_expanded_names_are_rejected_before_any_builder_commit() {
    let attrs = [
        Attribute::new(
            QualifiedName::new(Namespace::none(), Some("one".into()), "a"),
            "first",
        ),
        Attribute::new(
            QualifiedName::new(Namespace::none(), Some("two".into()), "a"),
            "second",
        ),
    ];
    let (mut owner, doc) = builder(1_000_000);
    let before = owner.usage();
    assert_eq!(
        owner.add_element_from_inputs(
            &doc,
            &qname("x"),
            attrs.as_slice(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(())
        ),
        Err(HtmlCaptureElementError::Attributes(
            HtmlAttributeValidationError::Duplicate {
                first: 0,
                duplicate: 1
            }
        ))
    );
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.data.records.len(), 1);
    owner
        .add_element_from_inputs(
            &doc,
            &qname("x"),
            &attrs[..1],
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(()),
        )
        .unwrap();
    assert_eq!(owner.usage().subjects(), 2);
}

struct CountMismatch {
    value: Attribute,
    count: usize,
}
impl HtmlAttributesInput for CountMismatch {
    type Attribute = Attribute;
    fn len(&self) -> usize {
        self.count
    }
    fn get(&self, index: usize) -> Option<&Attribute> {
        (index == 0).then_some(&self.value)
    }
}
#[test]
fn open_attribute_count_is_not_a_proof_of_content() {
    for (count, index) in [(0, 0), (2, 1)] {
        let input = CountMismatch {
            value: Attribute::new(qname("a"), "value"),
            count,
        };
        let (mut owner, doc) = builder(1_000_000);
        let before = owner.usage();
        assert_eq!(
            owner.add_element_from_inputs(
                &doc,
                &qname("x"),
                &input,
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
                || Ok::<_, Infallible>(())
            ),
            Err(HtmlCaptureElementError::InconsistentAttributes { index })
        );
        assert_eq!(owner.usage(), before);
        assert_eq!(owner.data.records.len(), 1);
    }
}

struct ChangedCapacity {
    small: Attribute,
    large: Attribute,
    reads: Cell<usize>,
}

struct ChangingName {
    small: DocumentString,
    large: DocumentString,
    reads: Cell<usize>,
}
impl HtmlNameInput for ChangingName {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        None
    }
    fn prefix(&self) -> Option<&DocumentString> {
        None
    }
    fn local_name(&self) -> &DocumentString {
        let reads = self.reads.get();
        self.reads.set(reads + 1);
        if reads < 2 { &self.small } else { &self.large }
    }
}
#[test]
fn open_name_components_are_cloned_from_the_exact_borrowed_admitted_values() {
    let input = ChangingName {
        small: "x".into(),
        large: "unexpected-third-read".into(),
        reads: Cell::new(0),
    };
    let expected = QualifiedName::new(Namespace::none(), None, input.small.clone());
    let bytes = raw_element_bytes(&expected, &[], 0, None);
    let (mut owner, doc) = builder(bytes);
    let subject = owner
        .add_element_from_inputs(
            &doc,
            &input,
            &[] as &[Attribute],
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(()),
        )
        .unwrap();
    let HtmlCapturedNodeKind::Element(element) = &owner.data.records[subject.index].kind else {
        panic!("element")
    };
    assert_eq!(element.name(), &expected);
    assert_eq!(owner.usage().element_value_bytes(), bytes);
}

struct GrowingName {
    small: DocumentString,
    large: DocumentString,
    reads: Cell<usize>,
}
impl HtmlNameInput for GrowingName {
    fn namespace_uri(&self) -> Option<&DocumentString> {
        None
    }
    fn prefix(&self) -> Option<&DocumentString> {
        None
    }
    fn local_name(&self) -> &DocumentString {
        let reads = self.reads.get();
        self.reads.set(reads + 1);
        if reads == 0 { &self.small } else { &self.large }
    }
}

#[test]
fn actual_name_unit_limit_precedes_owned_cloning_and_later_interruption() {
    let input = GrowingName {
        small: "x".into(),
        large: "too-long".into(),
        reads: Cell::new(0),
    };
    let attrs = [Attribute::new(qname("a"), "v")];
    let (mut owner, doc) = builder(1_000_000);
    // The initial name, attribute name and value are one unit each. The actual
    // second-pass local name alone has eight units and must be refused before
    // an owned occurrence exists, regardless of the generous byte allowance.
    owner.limits.utf16_units = 3;
    let before = owner.usage();
    let original_strong = Arc::strong_count(input.large.shared_code_units());
    let observed_unadmitted_clone = Cell::new(false);
    let result = owner.add_element_from_inputs(
        &doc,
        &input,
        attrs.as_slice(),
        HtmlCapturedIsValue::Absent,
        HtmlCaptureRegistryInput::Null,
        || {
            if Arc::strong_count(input.large.shared_code_units()) > original_strong {
                observed_unadmitted_clone.set(true);
                Err("unadmitted clone reached a later callback")
            } else {
                Ok(())
            }
        },
    );
    assert_eq!(
        result,
        Err(HtmlCaptureElementError::Capture(
            HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::Utf16Units,
                limit: 3,
                required: 8
            }
        ))
    );
    assert!(!observed_unadmitted_clone.get());
    assert_eq!(
        Arc::strong_count(input.large.shared_code_units()),
        original_strong
    );
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.data.records.len(), 1);
}
impl HtmlAttributesInput for ChangedCapacity {
    type Attribute = Attribute;
    fn len(&self) -> usize {
        1
    }
    fn get(&self, index: usize) -> Option<&Attribute> {
        if index != 0 {
            return None;
        }
        let count = self.reads.get();
        self.reads.set(count + 1);
        Some(if count == 0 { &self.small } else { &self.large })
    }
}
#[test]
fn changed_open_backing_capacity_is_reconciled_before_publication() {
    let mut backing = Vec::with_capacity(91);
    backing.push(118);
    let input = ChangedCapacity {
        small: Attribute::new(qname("a"), "v"),
        large: Attribute::new(qname("a"), DocumentString::from_utf16(backing)),
        reads: Cell::new(0),
    };
    let name = qname("x");
    let bytes = raw_element_bytes(&name, std::slice::from_ref(&input.small), 1, None);
    let required = raw_element_bytes(&name, std::slice::from_ref(&input.large), 1, None);
    let (mut owner, doc) = builder(bytes);
    let before = owner.usage();
    assert_eq!(
        owner.add_element_from_inputs(
            &doc,
            &name,
            &input,
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || Ok::<_, Infallible>(())
        ),
        Err(HtmlCaptureElementError::Capture(
            HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::ElementValueBytes,
                limit: bytes,
                required
            }
        ))
    );
    assert_eq!(owner.usage(), before);
    assert_eq!(owner.data.records.len(), 1);
}

#[test]
fn every_admitted_read_refusal_keeps_the_original_builder_and_error() {
    let name = qname("input");
    let attrs = [
        Attribute::new(qname("long-a"), "1"),
        Attribute::new(qname("long-b"), "2"),
    ];
    let (mut owner, doc) = builder(1_000_000);
    let mut actions = 0;
    owner
        .add_element_from_inputs(
            &doc,
            &name,
            attrs.as_slice(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || {
                actions += 1;
                Ok::<_, usize>(())
            },
        )
        .unwrap();
    assert!(actions > 20, "includes real materialization and validation");
    for stop in 0..actions {
        let (mut owner, doc) = builder(1_000_000);
        let before = owner.usage();
        let mut called = 0;
        let result = owner.add_element_from_inputs(
            &doc,
            &name,
            attrs.as_slice(),
            HtmlCapturedIsValue::Absent,
            HtmlCaptureRegistryInput::Null,
            || {
                let at = called;
                called += 1;
                if at == stop { Err(stop) } else { Ok(()) }
            },
        );
        assert_eq!(result, Err(HtmlCaptureElementError::Interrupted(stop)));
        assert_eq!(called, stop + 1);
        assert_eq!(owner.usage(), before);
        assert_eq!(owner.data.records.len(), 1);
    }
}
