use super::*;
use std::convert::Infallible;
use surgeist_html::{
    construction::{HtmlDoctype, HtmlDocumentMode},
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

fn builder() -> HtmlCaptureBuilder {
    HtmlCaptureBuilder::new(
        HtmlCaptureLimits::new(100, 1000, 100, 10000, 100, 100, 100, 16 * 1024 * 1024),
        HtmlHostFactsRevision::fresh(),
    )
}
fn document(builder: &mut HtmlCaptureBuilder) -> HtmlCaptureBuilderSubject {
    builder
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap()
}
fn padded(units: &[u16], spare: usize) -> DocumentString {
    let mut backing = Vec::with_capacity(units.len() + spare);
    backing.extend_from_slice(units);
    DocumentString::from_utf16(backing)
}
fn payload(value: &DocumentString) -> (usize, usize) {
    let capacity = value.retained_units();
    (
        size_of::<Vec<u16>>() + capacity * size_of::<u16>(),
        capacity,
    )
}
fn observe(capture: &HtmlCapture) -> HtmlCaptureRetainedStorageUsage {
    capture
        .retained_storage_usage_with(|| Ok::<_, Infallible>(()))
        .unwrap()
}

#[test]
fn real_capture_counts_spare_banks_edges_attributes_and_every_string_occurrence() {
    let mut builder = builder();
    let document = document(&mut builder);
    let strings = [
        padded(
            &"http://www.w3.org/1999/xhtml"
                .encode_utf16()
                .collect::<Vec<_>>(),
            11,
        ),
        padded(&[], 7), // present empty prefix
        padded(&[100, 105, 118], 9),
        padded(&[97, 116, 116, 114], 8),
        padded(&[0xd800, 120, 0xdc00], 13),
        padded(&[105, 115], 10),
        padded(&[], 5), // present empty nonce
        padded(&[116], 17),
        padded(&[99], 19),
        padded(&[112, 105], 21),
        padded(&[100], 23),
        padded(&[104], 25),
        padded(&[], 29), // present empty public identifier
        padded(&[115], 31),
        padded(&[120, 45, 97], 33),
        padded(&[100, 105, 118], 35),
    ];
    let name = QualifiedName::new(
        Namespace::from_uri(strings[0].clone()),
        Some(strings[1].clone()),
        strings[2].clone(),
    );
    let mut attribute_bank = Vec::with_capacity(13);
    attribute_bank.push(surgeist_html::values::Attribute::new(
        QualifiedName::new(Namespace::none(), None, strings[3].clone()),
        strings[4].clone(),
    ));
    let attributes = OrderedAttributes::new(attribute_bank).unwrap();
    let attribute_capacity = attributes.shared_attributes().capacity();
    let element = builder
        .add_element(
            &document,
            name,
            attributes,
            HtmlCapturedIsValue::Value(strings[5].clone()),
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    builder
        .establish_element_nonce(&element, strings[6].clone())
        .unwrap();
    let text = builder.add_text(&document, strings[7].clone()).unwrap();
    let comment = builder.add_comment(&document, strings[8].clone()).unwrap();
    let pi = builder
        .add_processing_instruction(&document, strings[9].clone(), strings[10].clone())
        .unwrap();
    let doctype = builder
        .add_doctype(
            &document,
            HtmlDoctype::new(
                strings[11].clone(),
                Some(strings[12].clone()),
                Some(strings[13].clone()),
            ),
        )
        .unwrap();
    let fragment = builder.add_fragment(&document).unwrap();
    builder.establish_unhosted(&fragment).unwrap();
    builder.establish_children(&fragment, &[]).unwrap();
    builder
        .establish_children(&document, &[element.clone(), pi, doctype])
        .unwrap();
    builder
        .establish_children(&element, &[text, comment])
        .unwrap();
    let mut definitions = Vec::with_capacity(11);
    definitions.push(HtmlCustomElementDefinition::new(
        strings[14].clone(),
        strings[15].clone(),
        false,
        false,
    ));
    builder.add_registry(Some(definitions)).unwrap();
    builder.data.records.reserve_exact(23);
    builder.data.registries.reserve_exact(17);
    builder.data.records[document.index]
        .dependents
        .reserve_exact(19);
    builder.data.records[element.index]
        .children
        .as_mut()
        .unwrap()
        .reserve_exact(13);
    let capture = builder.finish().unwrap();
    let data = &capture.0;
    let edge_capacity: usize = data
        .records
        .iter()
        .map(|record| {
            record.children.as_ref().map_or(0, Vec::capacity) + record.dependents.capacity()
        })
        .sum();
    let string_bytes: usize = strings.iter().map(|value| payload(value).0).sum();
    let expected_bytes = size_of::<CaptureData>()
        + data.records.capacity() * size_of::<Record>()
        + data.registries.capacity() * size_of::<RegistryRecord>()
        + edge_capacity * size_of::<usize>()
        + size_of::<Vec<surgeist_html::values::Attribute>>()
        + attribute_capacity * size_of::<surgeist_html::values::Attribute>()
        + data.registries[0].definitions.capacity() * size_of::<HtmlCustomElementDefinition>()
        + string_bytes;
    let usage = observe(&capture);
    assert_eq!(usage.retained_bytes(), expected_bytes);
    assert_eq!(
        usage.utf16_extent(),
        strings.iter().map(|value| payload(value).1).sum::<usize>()
    );
    assert!(usage.utf16_extent() > capture.usage().utf16_units());
    assert_interruptions(&capture);
}

#[test]
fn present_empty_doctype_identifier_owns_a_payload_header_but_absence_does_not() {
    let build = |public| {
        let mut builder = builder();
        let document = document(&mut builder);
        builder
            .add_doctype(&document, HtmlDoctype::new("html", public, None))
            .unwrap();
        builder.finish().unwrap()
    };
    let absent = build(None);
    let present = build(Some(DocumentString::from_utf16(Vec::new())));
    assert_eq!(
        observe(&present).utf16_extent(),
        observe(&absent).utf16_extent()
    );
    assert_eq!(
        observe(&present).retained_bytes() - observe(&absent).retained_bytes(),
        size_of::<Vec<u16>>()
    );
    assert_eq!(present.usage(), absent.usage());
}

#[test]
fn shared_strings_count_per_occurrence_but_clone_preserves_whole_capture_identity() {
    let value = padded(&[0xd800, 65], 41);
    let build = || {
        let mut builder = builder();
        let document = document(&mut builder);
        builder.add_text(&document, value.clone()).unwrap();
        builder.add_comment(&document, value.clone()).unwrap();
        builder.finish().unwrap()
    };
    let capture = build();
    let clone = capture.clone();
    let independent = build();
    assert_eq!(capture.identity(), clone.identity());
    assert_ne!(capture.identity(), independent.identity());
    assert_eq!(observe(&capture), observe(&clone));
    assert_eq!(observe(&capture), observe(&independent));
    assert_eq!(observe(&capture).utf16_extent(), 2 * payload(&value).1);
}

#[test]
fn shared_attribute_payload_is_priced_for_each_retained_element_occurrence() {
    let local = padded(&[100, 105, 118], 11);
    let attribute_name = padded(&[120], 17);
    let value = padded(&[0xd800], 23);
    let mut backing = Vec::with_capacity(29);
    backing.push(surgeist_html::values::Attribute::new(
        QualifiedName::new(Namespace::none(), None, attribute_name.clone()),
        value.clone(),
    ));
    let attributes = OrderedAttributes::new(backing).unwrap();
    let build = |count| {
        let mut builder = builder();
        let document = document(&mut builder);
        for _ in 0..count {
            builder
                .add_element(
                    &document,
                    QualifiedName::new(Namespace::none(), None, local.clone()),
                    attributes.clone(),
                    HtmlCapturedIsValue::Absent,
                    HtmlCaptureRegistryInput::Null,
                )
                .unwrap();
        }
        builder.finish().unwrap()
    };
    let single = build(1);
    let double = build(2);
    let bank_growth =
        (double.0.records.capacity() - single.0.records.capacity()) * size_of::<Record>();
    assert_eq!(
        observe(&double).retained_bytes() - observe(&single).retained_bytes(),
        bank_growth
            + payload(&local).0
            + size_of::<Vec<surgeist_html::values::Attribute>>()
            + attributes.shared_attributes().capacity()
                * size_of::<surgeist_html::values::Attribute>()
            + payload(&attribute_name).0
            + payload(&value).0
    );
    assert_eq!(
        observe(&double).utf16_extent() - observe(&single).utf16_extent(),
        payload(&local).1 + payload(&attribute_name).1 + payload(&value).1
    );
}

#[test]
fn selected_hyperlink_adds_only_out_of_line_receipt_without_original_fact_bank() {
    use crate::*;
    use surgeist_html::values::{DocumentString, Namespace, OrderedAttributes, QualifiedName};
    let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
    let base = HtmlBaseUrlRecord::observed(
        revision.clone(),
        "https://example.test/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let raw: DocumentString = "https://[".into();
    let facts = HtmlEncodingUrlParseFacts::new(
        base,
        HtmlBaseEncoding::fresh(),
        revision,
        vec![
            HtmlEncodingUrlParseObservation::new(raw.clone(), HtmlEncodingUrlParseOutcome::Failure),
            HtmlEncodingUrlParseObservation::new(
                padded(&[120], 1000),
                HtmlEncodingUrlParseOutcome::Failure,
            ),
        ],
        HtmlBaseFactsLimits::new(10, 10000, 10000),
    )
    .unwrap();
    let snapshot = facts
        .resolve_hyperlink_with(
            &raw,
            facts.base(),
            facts.encoding(),
            facts.revision(),
            || Ok::<_, Infallible>(()),
            |_| Ok(()),
        )
        .unwrap();
    let build = |observed| {
        let mut builder = builder();
        let document = document(&mut builder);
        let attributes = OrderedAttributes::new(vec![surgeist_html::values::Attribute::new(
            QualifiedName::new(Namespace::none(), None, "href"),
            raw.clone(),
        )])
        .unwrap();
        let element = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::html(), None, "a"),
                attributes,
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        if observed {
            builder
                .establish_hyperlink_url_with(&element, Some((&document, snapshot.clone())), || {
                    Ok::<_, Infallible>(())
                })
                .unwrap();
        }
        builder.finish().unwrap()
    };
    let unknown = build(false);
    let observed = build(true);
    let nested = snapshot.retained_storage_usage();
    assert_eq!(
        observe(&observed).retained_bytes() - observe(&unknown).retained_bytes(),
        nested.retained_bytes() - size_of::<HtmlHyperlinkUrlSnapshot>()
    );
    assert_eq!(
        observe(&observed).utf16_extent() - observe(&unknown).utf16_extent(),
        nested.utf16_extent()
    );
    drop(facts);
    assert_eq!(
        observe(&observed).utf16_extent(),
        observe(&unknown).utf16_extent() + nested.utf16_extent()
    );
    assert_interruptions(&observed);
}

#[test]
fn every_interruption_preserves_exact_first_error_identity_and_capture_value() {
    let mut builder = builder();
    let document = document(&mut builder);
    let text = padded(&[0xd800, 65, 0xdc00], 71);
    builder.add_text(&document, text).unwrap();
    let capture = builder.finish().unwrap();
    assert_interruptions(&capture);
}

fn assert_interruptions(capture: &HtmlCapture) {
    let identity = capture.identity().clone();
    let logical = capture.usage();
    let kinds: Vec<_> = capture
        .0
        .records
        .iter()
        .map(|record| record.kind.clone())
        .collect();
    let expected = observe(capture);
    let mut complete_calls = 0;
    capture
        .retained_storage_usage_with(|| {
            complete_calls += 1;
            Ok::<_, Infallible>(())
        })
        .unwrap();
    #[derive(Debug, PartialEq)]
    struct Stop(usize);
    for allowance in 0..complete_calls {
        let mut calls = 0;
        let failure = capture
            .retained_storage_usage_with(|| {
                calls += 1;
                if calls > allowance {
                    Err(Stop(allowance))
                } else {
                    Ok(())
                }
            })
            .unwrap_err();
        assert_eq!(
            failure,
            HtmlCaptureStorageError::Interrupted(Stop(allowance))
        );
        assert_eq!(calls, allowance + 1);
        assert_eq!(capture.identity(), &identity);
        assert_eq!(capture.usage(), logical);
        assert_eq!(observe(capture), expected);
        assert!(
            capture
                .0
                .records
                .iter()
                .zip(&kinds)
                .all(|(record, kind)| &record.kind == kind)
        );
    }
}

#[test]
fn checked_arithmetic_refuses_without_constructing_impossible_allocations() {
    let mut usage = HtmlCaptureRetainedStorageUsage {
        bytes: usize::MAX,
        units: 0,
    };
    assert_eq!(
        usage.add_bytes::<Infallible>(1),
        Err(HtmlCaptureStorageError::RepresentationExhausted)
    );
    let mut usage = HtmlCaptureRetainedStorageUsage { bytes: 0, units: 0 };
    assert_eq!(
        usage.bank::<u16, Infallible>(usize::MAX),
        Err(HtmlCaptureStorageError::RepresentationExhausted)
    );
    let mut usage = HtmlCaptureRetainedStorageUsage {
        bytes: 0,
        units: usize::MAX,
    };
    assert_eq!(
        usage.add_units::<Infallible>(1),
        Err(HtmlCaptureStorageError::RepresentationExhausted)
    );
}

fn borrowers() -> (surgeist_html::values::TextBlock, [DocumentString; 2]) {
    let mut units = Vec::with_capacity(512);
    units.extend([97_u16, 98, 99, 100, 101, 102, 103, 104]);
    let block = surgeist_html::values::TextBlock::from_utf16(units);
    let first = block.string(0..4).unwrap();
    let second = block.string(4..8).unwrap();
    (block, [first, second])
}

fn exact(units: &[u16]) -> DocumentString {
    let mut backing = Vec::new();
    backing.try_reserve_exact(units.len()).unwrap();
    backing.extend_from_slice(units);
    assert_eq!(backing.capacity(), units.len());
    DocumentString::from_utf16(backing)
}

#[test]
fn borrowers_of_one_block_are_priced_by_length_and_the_block_once() {
    let build = |first: DocumentString, second: DocumentString| {
        let mut builder = builder();
        let document = document(&mut builder);
        builder.add_text(&document, first).unwrap();
        builder.add_comment(&document, second).unwrap();
        builder.finish().unwrap()
    };
    let owners = build(exact(&[97, 98, 99, 100]), exact(&[101, 102, 103, 104]));
    let (block, [first, second]) = borrowers();
    let borrowed = build(first, second);
    let block_bytes = size_of::<Vec<u16>>() + block.capacity() * size_of::<u16>();
    let (owned, shared) = (observe(&owners), observe(&borrowed));
    assert_eq!(
        shared.utf16_extent(),
        owned.utf16_extent() + block.capacity()
    );
    // The block is listed once however many strings borrow it, in a table the capture owns.
    assert_eq!(borrowed.0.blocks.len(), 1);
    assert!(owners.0.blocks.is_empty());
    let table_bytes = borrowed.0.blocks.capacity() * BlockTable::ENTRY_BYTES;
    assert_eq!(
        shared.retained_bytes(),
        owned.retained_bytes() + block_bytes + table_bytes
    );
}

#[test]
fn attribute_values_that_borrow_one_block_are_priced_by_length_and_the_block_once() {
    let build = |first: DocumentString, second: DocumentString| {
        let mut builder = builder();
        let document = document(&mut builder);
        let name = |local: &str| QualifiedName::new(Namespace::none(), None, local);
        let attributes = OrderedAttributes::new([
            surgeist_html::values::Attribute::new(name("a"), first),
            surgeist_html::values::Attribute::new(name("b"), second),
        ])
        .unwrap();
        builder
            .add_element(
                &document,
                name("div"),
                attributes,
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.finish().unwrap()
    };
    let owners = build(exact(&[97, 98, 99, 100]), exact(&[101, 102, 103, 104]));
    let (block, [first, second]) = borrowers();
    let borrowed = build(first, second);
    let (owned, shared) = (observe(&owners), observe(&borrowed));
    assert_eq!(
        shared.utf16_extent(),
        owned.utf16_extent() + block.capacity()
    );
    assert_eq!(borrowed.0.blocks.len(), 1);
    assert!(owners.0.blocks.is_empty());
}

fn listed_strings<'a>(
    strings: &'a [DocumentString],
) -> impl Fn(&mut dyn FnMut(&DocumentString)) + 'a {
    move |visit| strings.iter().for_each(visit)
}

#[test]
fn a_listing_adds_each_new_block_once_and_undoes_only_what_it_added() {
    let (first_block, [first, second]) = borrowers();
    let (second_block, [third, _]) = borrowers();
    let mut table = BlockTable::new();
    let mut sequence = 0;
    let earlier = [first.clone()];
    let kept = list_blocks(&mut table, &mut sequence, listed_strings(&earlier)).unwrap();
    assert_eq!(table.len(), 1);
    // The second listing meets the first block again and adds the second block once, however many
    // strings borrow it.
    let both = [second, third.clone(), third];
    let listing = list_blocks(&mut table, &mut sequence, listed_strings(&both)).unwrap();
    assert_eq!(table.len(), 2);
    assert_eq!(
        table.find(&first_block),
        Some(&(first_block.retained_bytes(), first_block.capacity(), 0))
    );
    listing.undo(&mut table);
    assert_eq!(table.len(), 1);
    assert!(table.find(&first_block).is_some());
    assert!(table.find(&second_block).is_none());
    kept.undo(&mut table);
    assert!(table.is_empty());
}

#[test]
fn an_owner_string_lists_no_block() {
    let mut table = BlockTable::new();
    let mut sequence = 0;
    let owners = [exact(&[97]), padded(&[98], 9)];
    list_blocks(&mut table, &mut sequence, listed_strings(&owners)).unwrap();
    assert!(table.is_empty());
}

#[test]
fn a_record_that_a_host_cap_refuses_lists_no_block() {
    let (block, [first, second]) = borrowers();
    let caps = surgeist_html::accounting::HtmlLimits::from_fn(
        |_| u64::MAX,
        |resource| match resource {
            surgeist_html::accounting::HtmlResource::CapturedUtf16Units => 4,
            _ => u64::MAX,
        },
    );
    let operation = surgeist_ledger::Operation::with_profile(
        "host",
        caps,
        4,
        surgeist_ledger::ProfileMode::Summary { timing: false },
    )
    .unwrap();
    let root = operation.root();
    let accounting = surgeist_html::accounting::HtmlHostScope::new(operation.share(), root);
    let mut hosted = HtmlCaptureBuilder::with_accounting(
        HtmlCaptureLimits::new(100, 1000, 100, 10000, 100, 100, 100, 16 * 1024 * 1024),
        HtmlHostFactsRevision::fresh(),
        &accounting,
    )
    .unwrap();
    let document = hosted
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    // Four units fit the cap; the next four do not.
    hosted.add_text(&document, first).unwrap();
    assert!(hosted.add_text(&document, second).is_err());
    assert_eq!(hosted.data.blocks.len(), 1);
    assert!(hosted.data.blocks.find(&block).is_some());
    let mut refused = HtmlCaptureBuilder::with_accounting(
        HtmlCaptureLimits::new(100, 1000, 100, 10000, 100, 100, 100, 16 * 1024 * 1024),
        HtmlHostFactsRevision::fresh(),
        &accounting,
    )
    .unwrap();
    let document = refused
        .add_document(
            HtmlDocumentMode::NoQuirks,
            HtmlScriptingStatus::Disabled,
            HtmlCaptureRegistryInput::Null,
        )
        .unwrap();
    let (other, [third, _]) = borrowers();
    assert!(refused.add_text(&document, third).is_err());
    assert!(refused.data.blocks.find(&other).is_none());
    assert!(refused.data.blocks.is_empty());
}
