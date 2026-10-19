use super::*;
use std::sync::Arc;

fn generous() -> HtmlOptionValueEnvelope {
    HtmlOptionValueEnvelope::new(16_000_000, 1_000_000, 1_000_000)
}
fn setup() -> (HtmlControlledOptionValueCapture, HtmlCaptureBuilderSubject) {
    let cancel = AtomicBool::new(false);
    let mut owner = HtmlControlledOptionValueCapture::new(
        HtmlCaptureLimits::new(16, 64, 16, 10_000, 16, 0, 0, 1_000_000),
        HtmlHostFactsRevision::fresh(),
        generous(),
        &cancel,
    )
    .unwrap();
    let doc = owner
        .add_document(HtmlDocumentMode::NoQuirks, generous(), &cancel)
        .unwrap();
    (owner, doc)
}
fn qname(local: &str) -> QualifiedName {
    QualifiedName::new(Namespace::none(), None, local)
}
fn padded(local: &str, capacity: usize) -> DocumentString {
    let mut units = Vec::with_capacity(capacity);
    units.extend(local.encode_utf16());
    DocumentString::from_utf16(units)
}
fn raw_string(value: &DocumentString) -> (usize, usize) {
    let cap = value.shared_code_units().capacity();
    (size_of::<Vec<u16>>() + cap * size_of::<u16>(), cap)
}
fn raw_name(value: &QualifiedName) -> (usize, usize) {
    let mut out = raw_string(value.local_name());
    for value in [value.namespace().uri(), value.prefix()]
        .into_iter()
        .flatten()
    {
        let next = raw_string(value);
        out.0 += next.0;
        out.1 += next.1;
    }
    out
}
fn raw_payload(
    name: &QualifiedName,
    attrs: &[Attribute],
    saved: Option<&DocumentString>,
) -> (usize, usize) {
    let mut out = raw_name(name);
    out.0 += size_of::<Vec<Attribute>>() + std::mem::size_of_val(attrs);
    for attr in attrs {
        for next in [raw_name(attr.name()), raw_string(attr.value())] {
            out.0 += next.0;
            out.1 += next.1;
        }
    }
    if let Some(value) = saved {
        let next = raw_string(value);
        out.0 += next.0;
        out.1 += next.1;
    }
    out
}
fn raw_builder_bytes(owner: &HtmlControlledOptionValueCapture) -> usize {
    let Phase::Building(builder) = &owner.phase else {
        panic!("building")
    };
    // Document-only setup owns no nested values, dependents or children bank.
    size_of::<HtmlControlledOptionValueCapture>()
        + builder.inspect_controlled_records().1 * builder.inspect_controlled_records().2
}

#[test]
fn borrowed_destination_and_old_record_bank_fit_exactly_and_refuse_one_short() {
    let (mut owner, doc) = setup();
    let cancel = AtomicBool::new(false);
    let name = QualifiedName::new(Namespace::none(), None, padded("x", 71));
    let mut source = Vec::with_capacity(47);
    source.push(Attribute::new(qname("a"), padded("v", 129)));
    let saved = padded("s", 31);
    let (payload_bytes, payload_units) = raw_payload(&name, &source, Some(&saved));
    let before = owner.storage_usage();
    assert_eq!(before.bytes, raw_builder_bytes(&owner));
    assert_eq!(before.units, 0);
    let new_bank = 2 * match &owner.phase {
        Phase::Building(builder) => builder.inspect_controlled_records().2,
        _ => unreachable!(),
    };
    let exact_peak = before.bytes + payload_bytes + new_bank;
    let failure = owner
        .add_element(
            &doc,
            &name,
            source.as_slice(),
            Some(&saved),
            HtmlOptionValueEnvelope::new(exact_peak - 1, payload_units, 100_000),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded { resource: HtmlOptionValueResource::LiveBytes, limit, required }) if *limit == (exact_peak - 1) as u64 && *required == exact_peak as u64)
    );
    assert_eq!(owner.storage_usage(), before);
    assert_eq!(owner.capture_usage().subjects(), 1);
    assert_eq!(
        owner.control.temporary.get(),
        HtmlOptionValueStorageUsage::default()
    );
    let subject = owner
        .add_element(
            &doc,
            &name,
            source.as_slice(),
            Some(&saved),
            HtmlOptionValueEnvelope::new(exact_peak, payload_units, 100_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.storage_usage().units, payload_units);
    assert_eq!(
        owner.storage_usage().bytes,
        raw_builder_bytes(&owner) + payload_bytes
    );
    assert_eq!(owner.usage().peak_storage().bytes, exact_peak);
    owner.finish(generous(), &cancel).unwrap();
    let Phase::Finished { capture, .. } = &owner.phase else {
        panic!("finished")
    };
    let id = capture.subject_id(&subject).unwrap();
    let node = capture.node(&id).unwrap();
    let HtmlCapturedNodeKind::Element(element) = node.kind() else {
        panic!("element")
    };
    assert_eq!(element.attributes().as_slice(), source.as_slice());
    assert_eq!(element.attributes().shared_attributes().capacity(), 1);
    assert!(source.capacity() > element.attributes().shared_attributes().capacity());
    assert!(Arc::ptr_eq(
        element.attributes().as_slice()[0]
            .value()
            .shared_code_units(),
        source[0].value().shared_code_units()
    ));
}

#[test]
fn actual_utf16_capacity_has_its_own_exact_pending_value_gate() {
    let (mut owner, doc) = setup();
    let cancel = AtomicBool::new(false);
    let name = QualifiedName::new(Namespace::none(), None, padded("x", 61));
    let attrs = [Attribute::new(qname("a"), padded("v", 89))];
    let (_, units) = raw_payload(&name, &attrs, None);
    let before = owner.storage_usage();
    let failure = owner
        .add_element(
            &doc,
            &name,
            attrs.as_slice(),
            None,
            HtmlOptionValueEnvelope::new(1_000_000, units - 1, 100_000),
            &cancel,
        )
        .unwrap_err();
    assert!(
        matches!(failure.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded { resource: HtmlOptionValueResource::LiveUtf16, limit, required }) if *limit == (units - 1) as u64 && *required == units as u64)
    );
    assert_eq!(owner.storage_usage(), before);
    owner
        .add_element(
            &doc,
            &name,
            attrs.as_slice(),
            None,
            HtmlOptionValueEnvelope::new(1_000_000, units, 100_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.storage_usage().units, units);
    assert_eq!(owner.capture_usage().utf16_units(), 3);
}

#[test]
fn borrowed_duplicate_refusal_drops_the_destination_and_can_retry_on_original_owner() {
    let (mut owner, doc) = setup();
    let cancel = AtomicBool::new(false);
    let local = qname("x");
    let attrs = [
        Attribute::new(qname("value"), "first"),
        Attribute::new(qname("value"), "second"),
    ];
    let before = owner.storage_usage();
    let strong = Arc::strong_count(attrs[0].value().shared_code_units());
    let failure = owner
        .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlControlledOptionValueError::ElementValues(HtmlCaptureElementError::Attributes(
            surgeist_html::bounded::HtmlAttributeValidationError::Duplicate {
                first: 0,
                duplicate: 1
            }
        ))
    );
    assert_eq!(owner.storage_usage(), before);
    assert_eq!(owner.capture_usage().subjects(), 1);
    assert_eq!(
        owner.control.temporary.get(),
        HtmlOptionValueStorageUsage::default()
    );
    assert_eq!(
        Arc::strong_count(attrs[0].value().shared_code_units()),
        strong
    );
    owner
        .add_element(&doc, &local, &attrs[..1], None, generous(), &cancel)
        .unwrap();
    assert_eq!(owner.capture_usage().subjects(), 2);
}

#[test]
fn work_and_cancellation_refusals_drop_every_unpublished_borrowed_payload() {
    let local = qname("option");
    let attrs = [
        Attribute::new(qname("long-a"), "first"),
        Attribute::new(qname("long-b"), "second"),
    ];
    for budget in [0, 8, 32, 64] {
        let (mut owner, doc) = setup();
        let cancel = AtomicBool::new(false);
        let before = owner.storage_usage();
        let strong = Arc::strong_count(attrs[0].value().shared_code_units());
        let failure = owner
            .add_element(
                &doc,
                &local,
                attrs.as_slice(),
                None,
                HtmlOptionValueEnvelope::new(1_000_000, 10_000, budget),
                &cancel,
            )
            .unwrap_err();
        assert!(
            matches!(failure.kind(), HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded { resource: HtmlOptionValueResource::Work, limit, .. }) if *limit == budget)
        );
        assert_eq!(owner.storage_usage(), before);
        assert_eq!(owner.capture_usage().subjects(), 1);
        assert_eq!(
            owner.control.temporary.get(),
            HtmlOptionValueStorageUsage::default()
        );
        assert_eq!(
            Arc::strong_count(attrs[0].value().shared_code_units()),
            strong
        );
        owner
            .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
            .unwrap();
    }
    let (mut owner, doc) = setup();
    let cancel = AtomicBool::new(false);
    let before = owner.storage_usage();
    owner.control.cancel_after.set(owner.usage().work() + 32);
    let failure = owner
        .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::Cancelled)
    );
    assert_eq!(owner.storage_usage(), before);
    assert_eq!(
        owner.control.temporary.get(),
        HtmlOptionValueStorageUsage::default()
    );
    owner.control.cancel_after.set(u64::MAX);
    cancel.store(false, Ordering::Relaxed);
    owner
        .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
        .unwrap();
}
