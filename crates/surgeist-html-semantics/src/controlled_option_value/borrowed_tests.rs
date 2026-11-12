use super::*;
use surgeist_html::{
    construction::HtmlDocumentMode,
    values::{Attribute, DocumentString, Namespace, QualifiedName},
};

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
    let cap = value.retained_units();
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
        owner.control.temporary_usage(),
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
    assert!(
        element.attributes().as_slice()[0]
            .value()
            .shares_storage_with(source[0].value())
    );
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
    let strong = attrs[0].value().block().holders();
    let failure = owner
        .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
        .unwrap_err();
    assert_eq!(
        failure.kind(),
        &HtmlControlledOptionValueError::ElementValues(HtmlCaptureElementError::Attributes(
            surgeist_html::bounded::values::HtmlAttributeValidationError::Duplicate {
                first: 0,
                duplicate: 1
            }
        ))
    );
    assert_eq!(owner.storage_usage(), before);
    assert_eq!(owner.capture_usage().subjects(), 1);
    assert_eq!(
        owner.control.temporary_usage(),
        HtmlOptionValueStorageUsage::default()
    );
    assert_eq!(attrs[0].value().block().holders(), strong);
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
        let strong = attrs[0].value().block().holders();
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
            owner.control.temporary_usage(),
            HtmlOptionValueStorageUsage::default()
        );
        assert_eq!(attrs[0].value().block().holders(), strong);
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
        owner.control.temporary_usage(),
        HtmlOptionValueStorageUsage::default()
    );
    owner.control.cancel_after.set(u64::MAX);
    cancel.store(false, Ordering::Relaxed);
    owner
        .add_element(&doc, &local, attrs.as_slice(), None, generous(), &cancel)
        .unwrap();
}

#[test]
fn values_and_saved_is_that_borrow_a_block_are_admitted_by_their_length_and_the_block_once() {
    let mut units = Vec::with_capacity(512);
    units.extend("abcd".encode_utf16());
    let block = surgeist_html::values::TextBlock::from_utf16(units);
    assert!(block.capacity() >= 512);
    let exact = |text: &str| {
        let mut units = Vec::new();
        units.try_reserve_exact(text.len()).unwrap();
        units.extend(text.encode_utf16());
        assert_eq!(units.capacity(), text.len());
        DocumentString::from_utf16(units)
    };
    let name = qname("x");
    let owners = [Attribute::new(qname("a"), exact("ab"))];
    let saved_owner = exact("bc");
    let borrowed = [Attribute::new(qname("a"), block.string(0..2).unwrap())];
    let saved = block.string(1..3).unwrap();
    // The envelope that admits an element of owners of the same texts admits the borrowers and the
    // block they borrow, once, at its capacity.
    let (owner_bytes, owner_units) = raw_payload(&name, &owners, Some(&saved_owner));
    let payload_bytes = owner_bytes + block.retained_bytes();
    let payload_units = owner_units + block.capacity();
    let (mut owner, doc) = setup();
    let cancel = AtomicBool::new(false);
    let before = owner.storage_usage();
    let bank = 2 * match &owner.phase {
        Phase::Building(builder) => builder.inspect_controlled_records().2,
        _ => unreachable!(),
    };
    let peak = before.bytes + payload_bytes + bank;
    owner
        .add_element(
            &doc,
            &name,
            borrowed.as_slice(),
            Some(&saved),
            HtmlOptionValueEnvelope::new(peak, payload_units, 100_000),
            &cancel,
        )
        .unwrap();
    assert_eq!(owner.storage_usage().units, payload_units);
    assert_eq!(owner.usage().peak_storage().bytes, peak);
}

/// A block of text with spare capacity, and an exact owner of a text, for the tests that compare
/// elements of borrowers with elements of owners.
fn spare_block() -> surgeist_html::values::TextBlock {
    let mut units = Vec::with_capacity(512);
    units.extend("abcdefgh".encode_utf16());
    surgeist_html::values::TextBlock::from_utf16(units)
}
fn exact_owner(text: &str) -> DocumentString {
    let mut units = Vec::new();
    units.try_reserve_exact(text.len()).unwrap();
    units.extend(text.encode_utf16());
    assert_eq!(units.capacity(), text.len());
    DocumentString::from_utf16(units)
}
/// Adds an element named `x` whose saved `is` is `saved`.
#[expect(
    clippy::result_large_err,
    reason = "The failure is the public error of the add operation under test, returned unchanged."
)]
fn add_with_saved_is(
    owner: &mut HtmlControlledOptionValueCapture,
    doc: &HtmlCaptureBuilderSubject,
    saved: &DocumentString,
    envelope: HtmlOptionValueEnvelope,
    cancel: &AtomicBool,
) -> Result<HtmlCaptureBuilderSubject, HtmlControlledOptionValueFailure> {
    owner.add_element(
        doc,
        &qname("x"),
        &[] as &[Attribute],
        Some(saved),
        envelope,
        cancel,
    )
}

#[test]
fn elements_that_borrow_one_block_charge_it_once_for_the_capture() {
    let block = spare_block();
    assert!(block.capacity() >= 512);
    let cancel = AtomicBool::new(false);
    let add_two = |first: DocumentString, second: DocumentString| {
        let (mut owner, doc) = setup();
        for saved in [first, second] {
            add_with_saved_is(&mut owner, &doc, &saved, generous(), &cancel).unwrap();
        }
        owner.storage_usage()
    };
    let shared = add_two(block.string(0..2).unwrap(), block.string(4..6).unwrap());
    let owned = add_two(exact_owner("ab"), exact_owner("ef"));
    // Both values are charged by their lengths and the block is charged once, at its capacity.
    assert_eq!(shared.units, owned.units + block.capacity());
    assert_eq!(shared.bytes, owned.bytes + block.retained_bytes());
}

/// The storage of a capture that adds one element whose saved `is` is `saved`, after `refuse` has
/// tried and failed to add it.
fn storage_after_retry(
    saved: &DocumentString,
    refuse: impl FnOnce(&mut HtmlControlledOptionValueCapture, &HtmlCaptureBuilderSubject),
) -> HtmlOptionValueStorageUsage {
    let (mut owner, doc) = setup();
    refuse(&mut owner, &doc);
    add_with_saved_is(&mut owner, &doc, saved, generous(), &AtomicBool::new(false)).unwrap();
    owner.storage_usage()
}

#[test]
fn an_addition_the_envelope_refuses_leaves_its_block_unlisted_for_the_retry() {
    let block = spare_block();
    let saved = block.string(0..2).unwrap();
    let cancel = AtomicBool::new(false);
    let fresh = storage_after_retry(&saved, |_, _| {});
    let retried = storage_after_retry(&saved, |owner, doc| {
        let before = owner.storage_usage();
        // The value fits by its length and the block does not: the envelope refuses the addition
        // after the capture listed the block.
        let (_, units) = raw_payload(&qname("x"), &[], Some(&exact_owner("ab")));
        let failure = add_with_saved_is(
            owner,
            doc,
            &saved,
            HtmlOptionValueEnvelope::new(16_000_000, units + block.capacity() - 1, 100_000),
            &cancel,
        )
        .unwrap_err();
        assert!(matches!(
            failure.kind(),
            HtmlControlledOptionValueError::Control(HtmlOptionValueControlError::LimitExceeded {
                resource: HtmlOptionValueResource::LiveUtf16,
                ..
            })
        ));
        assert_eq!(owner.storage_usage(), before);
    });
    // The retry charges the block once, as an addition that was never refused does.
    assert_eq!(retried, fresh);
    assert!(fresh.units >= block.capacity());
}

#[test]
fn an_addition_the_capture_refuses_part_way_leaves_its_block_unlisted_for_the_retry() {
    let block = spare_block();
    let saved = block.string(0..2).unwrap();
    let fresh = storage_after_retry(&saved, |_, _| {});
    // Cancel at each work step of the addition in turn, until the addition finishes before the stop.
    let mut refused = 0;
    for after in 0..10_000 {
        let mut stopped = false;
        let retried = storage_after_retry(&saved, |owner, doc| {
            let cancel = AtomicBool::new(false);
            let before = owner.storage_usage();
            owner.control.cancel_after.set(owner.usage().work() + after);
            let outcome = add_with_saved_is(owner, doc, &saved, generous(), &cancel);
            owner.control.cancel_after.set(u64::MAX);
            if outcome.is_err() {
                stopped = true;
                assert_eq!(owner.storage_usage(), before);
                assert_eq!(
                    owner.control.temporary_usage(),
                    HtmlOptionValueStorageUsage::default()
                );
            }
        });
        if !stopped {
            break;
        }
        refused += 1;
        assert_eq!(retried, fresh, "stopped after {after} steps");
    }
    assert!(refused > 3, "the sweep refused {refused} stops");
}
