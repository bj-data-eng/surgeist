//! Optional immutable sharing through public DOM construction and edits.
//! Pointer observations accompany exact content, native hooks and snapshots.
use std::sync::{Arc, atomic::AtomicBool};

use surgeist_dom::{
    DocumentId, DocumentString, Dom, DomBatch, DomChange, DomCommit, DomLimits, DomNameLimits,
    DomSnapshot, Namespace, NodeId,
};

const RAW: &[u16] = &[107, 0xd800, 0, 0xdc00];

fn units(value: &[u16]) -> DocumentString {
    DocumentString::from_utf16(value.to_vec())
}
fn owner() -> Dom {
    Dom::new(
        DomLimits::new(
            32,
            65_536,
            128,
            0,
            0,
            64,
            1_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_input_processing(65_536, 1_000_000)
        .unwrap(),
    )
    .unwrap()
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}
fn document(dom: &mut Dom) -> DocumentId {
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    apply(dom, batch).document(&document).unwrap()
}
fn assert_input_attributes(snapshot: &DomSnapshot, input: NodeId, value: &[u16]) {
    let node = snapshot.node(input).unwrap();
    let attributes = node.as_element().unwrap().attributes();
    assert_eq!(attributes.len(), 3);
    for (attribute, local, expected) in [
        (
            &attributes[0],
            &[116, 121, 112, 101][..],
            &[116, 101, 120, 116][..],
        ),
        (&attributes[1], &[118, 97, 108, 117, 101][..], value),
    ] {
        assert!(attribute.name().namespace_uri().is_none());
        assert!(attribute.name().prefix().is_none());
        assert_eq!(attribute.name().local_name().code_units(), local);
        assert_eq!(attribute.value().code_units(), expected);
    }
    let label = &attributes[2];
    assert_eq!(
        label.name().namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 115, 104, 97, 114, 101]
    );
    assert_eq!(label.name().prefix().unwrap().code_units(), [112]);
    assert_eq!(
        label.name().local_name().code_units(),
        [108, 97, 98, 101, 108]
    );
    assert_eq!(label.value().code_units(), RAW);
}

#[test]
fn equal_native_and_imported_banks_share_without_merging_input_state() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let first = batch.create_element_ns(document, Namespace::html(), "input");
    batch.set_attribute(&first, "type", units(&[116, 101, 120, 116]));
    batch.set_attribute(&first, "value", units(&[115, 101, 101, 100]));
    batch.set_attribute_ns(
        &first,
        Namespace::from_uri("urn:share"),
        "p:label",
        units(RAW),
    );
    let second = batch.create_element_ns(document, Namespace::html(), "input");
    batch.set_attribute(&second, "type", units(&[116, 101, 120, 116]));
    batch.set_attribute(&second, "value", units(&[115, 101, 101, 100]));
    batch.set_attribute_ns(
        &second,
        Namespace::from_uri("urn:share"),
        "p:label",
        units(RAW),
    );
    let done = apply(&mut dom, batch);
    let first = done.created(&first).unwrap().unwrap();
    let second = done.created(&second).unwrap().unwrap();
    assert_ne!(first, second);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&before, first, document, true);
    let imported = apply(&mut dom, batch).created(&imported).unwrap().unwrap();
    let copied = dom.snapshot();
    assert_ne!(first, imported);
    for input in [first, second, imported] {
        assert_input_attributes(&copied, input, &[115, 101, 101, 100]);
        let node = copied.node(input).unwrap();
        assert_eq!(node.as_input().unwrap().value(), [115, 101, 101, 100]);
        assert!(!node.as_input().unwrap().state().flags().dirty_value());
    }

    let mut batch = DomBatch::new();
    batch.set_attribute(first, "value", "changed");
    apply(&mut dom, batch);
    let changed = dom.snapshot();
    assert_input_attributes(&changed, first, &[99, 104, 97, 110, 103, 101, 100]);
    assert_eq!(
        changed.node(first).unwrap().as_input().unwrap().value(),
        [99, 104, 97, 110, 103, 101, 100]
    );
    let mut batch = DomBatch::new();
    batch.set_input_value(first, "private");
    apply(&mut dom, batch);
    let current = dom.snapshot();
    assert_eq!(
        current.node(first).unwrap().as_input().unwrap().value(),
        [112, 114, 105, 118, 97, 116, 101]
    );
    assert!(
        current
            .node(first)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    for input in [second, imported] {
        assert_input_attributes(&current, input, &[115, 101, 101, 100]);
        let node = current.node(input).unwrap();
        assert_eq!(node.as_input().unwrap().value(), [115, 101, 101, 100]);
        assert!(!node.as_input().unwrap().state().flags().dirty_value());
    }
    assert_input_attributes(&before, first, &[115, 101, 101, 100]);
    assert_eq!(
        before.node(first).unwrap().as_input().unwrap().value(),
        [115, 101, 101, 100]
    );
    assert!(
        !before
            .node(first)
            .unwrap()
            .as_input()
            .unwrap()
            .state()
            .flags()
            .dirty_value()
    );
    let first_bank = copied
        .node(first)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    let second_bank = copied
        .node(second)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    let imported_bank = copied
        .node(imported)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    assert!(
        std::ptr::eq(first_bank, second_bank),
        "separately authored equal canonical banks must share"
    );
    assert!(
        std::ptr::eq(first_bank, imported_bank),
        "same-owner import must reuse the equal canonical bank"
    );
}

#[test]
fn independently_created_text_shares_without_changing_siblings_or_snapshots() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let first = batch.create_text(document, units(RAW));
    let second = batch.create_text(document, units(RAW));
    let done = apply(&mut dom, batch);
    let first = done.created(&first).unwrap().unwrap();
    let second = done.created(&second).unwrap().unwrap();
    assert_ne!(first, second);
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(first, 1, 1, "B");
    let done = apply(&mut dom, batch);
    assert_eq!(done.changes().changes(), [DomChange::Data(first)]);
    let after = dom.snapshot();
    assert_eq!(
        after.node(first).unwrap().data().unwrap().code_units(),
        [107, 66, 0, 0xdc00]
    );
    assert_eq!(
        after.node(second).unwrap().data().unwrap().code_units(),
        RAW
    );
    for id in [first, second] {
        assert_eq!(before.node(id).unwrap().data().unwrap().code_units(), RAW);
    }
    assert!(
        Arc::ptr_eq(
            before
                .node(first)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units(),
            before
                .node(second)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units()
        ),
        "independent equal Text values must share"
    );
}

#[test]
fn completed_text_replacement_reuses_an_existing_equal_value() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let edited = batch.create_text(document, "old");
    let existing = batch.create_text(document, units(RAW));
    let done = apply(&mut dom, batch);
    let edited = done.created(&edited).unwrap().unwrap();
    let existing = done.created(&existing).unwrap().unwrap();
    let before = dom.snapshot();
    let mut batch = DomBatch::new();
    batch.replace_data(edited, 0, usize::MAX, units(RAW));
    let done = apply(&mut dom, batch);
    assert_eq!(done.changes().changes(), [DomChange::Data(edited)]);
    let after = dom.snapshot();
    assert_eq!(
        before.node(edited).unwrap().data().unwrap().code_units(),
        [111, 108, 100]
    );
    for id in [edited, existing] {
        assert_eq!(after.node(id).unwrap().data().unwrap().code_units(), RAW);
    }
    assert!(
        Arc::ptr_eq(
            after
                .node(edited)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units(),
            after
                .node(existing)
                .unwrap()
                .data()
                .unwrap()
                .shared_code_units()
        ),
        "completed Text replacement must select the existing immutable value"
    );
}

#[test]
fn different_ordered_banks_share_equal_strings_but_keep_order_prefixes_and_ids() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let first = batch.create_element(document, "cell");
    batch.set_attribute(&first, "id", "one");
    batch.set_attribute(&first, "class", units(RAW));
    batch.set_attribute_ns(
        &first,
        Namespace::from_uri("urn:share"),
        "p:label",
        units(RAW),
    );
    let second = batch.create_element(document, "cell");
    batch.set_attribute(&second, "class", units(RAW));
    batch.set_attribute(&second, "id", "two");
    batch.set_attribute_ns(
        &second,
        Namespace::from_uri("urn:share"),
        "q:label",
        units(RAW),
    );
    let done = apply(&mut dom, batch);
    let first = done.created(&first).unwrap().unwrap();
    let second = done.created(&second).unwrap().unwrap();
    let snapshot = dom.snapshot();
    let left = snapshot
        .node(first)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    let right = snapshot
        .node(second)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    assert_eq!((left.len(), right.len()), (3, 3));
    assert_eq!(left[0].name().local_name().code_units(), [105, 100]);
    assert_eq!(left[0].value().code_units(), [111, 110, 101]);
    assert_eq!(
        right[0].name().local_name().code_units(),
        [99, 108, 97, 115, 115]
    );
    assert_eq!(right[1].name().local_name().code_units(), [105, 100]);
    assert_eq!(right[1].value().code_units(), [116, 119, 111]);
    assert_eq!(left[2].name().prefix().unwrap().code_units(), [112]);
    assert_eq!(right[2].name().prefix().unwrap().code_units(), [113]);
    for attribute in [&left[1], &left[2], &right[0], &right[2]] {
        assert_eq!(attribute.value().code_units(), RAW);
    }
    assert!(!std::ptr::eq(left, right));
    assert!(
        Arc::ptr_eq(
            left[1].value().shared_code_units(),
            right[0].value().shared_code_units()
        ),
        "different banks must still reuse repeated value strings"
    );
    assert!(Arc::ptr_eq(
        left[2].value().shared_code_units(),
        right[2].value().shared_code_units()
    ));
}

fn padded(value: &str, capacity: usize) -> DocumentString {
    let mut data = Vec::with_capacity(capacity);
    data.extend(value.encode_utf16());
    DocumentString::from_utf16(data)
}

#[test]
fn equal_smaller_noop_candidates_keep_original_backings_usage_and_history() {
    let mut dom = owner();
    let document = document(&mut dom);
    let original_attribute = padded("repeat-value", 257);
    let original_text = padded("repeat-text", 511);
    let smaller_attribute = units(&[114, 101, 112, 101, 97, 116, 45, 118, 97, 108, 117, 101]);
    let smaller_text = units(&[114, 101, 112, 101, 97, 116, 45, 116, 101, 120, 116]);
    assert!(
        original_attribute.shared_code_units().capacity()
            > smaller_attribute.shared_code_units().capacity()
    );
    assert!(
        original_text.shared_code_units().capacity() > smaller_text.shared_code_units().capacity()
    );
    let mut batch = DomBatch::new();
    let element = batch.create_element(document, "cell");
    batch.set_attribute(&element, "data", original_attribute.clone());
    let text = batch.create_text(document, original_text.clone());
    let done = apply(&mut dom, batch);
    let element = done.created(&element).unwrap().unwrap();
    let text = done.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let mut batch = DomBatch::new();
    batch.set_attribute(element, "data", smaller_attribute);
    batch.replace_data(text, 0, usize::MAX, smaller_text);
    let done = apply(&mut dom, batch);
    let after = dom.snapshot();
    assert!(done.changes().changes().is_empty());
    assert_eq!(after.stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert!(
        dom.changes_since(before.stamp())
            .unwrap()
            .changes()
            .is_empty()
    );
    let old_bank = before
        .node(element)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    let new_bank = after
        .node(element)
        .unwrap()
        .as_element()
        .unwrap()
        .attributes();
    assert_eq!(
        new_bank[0].value().code_units(),
        [114, 101, 112, 101, 97, 116, 45, 118, 97, 108, 117, 101]
    );
    assert!(std::ptr::eq(old_bank, new_bank));
    assert!(Arc::ptr_eq(
        new_bank[0].value().shared_code_units(),
        original_attribute.shared_code_units()
    ));
    assert_eq!(
        after.node(text).unwrap().data().unwrap().code_units(),
        [114, 101, 112, 101, 97, 116, 45, 116, 101, 120, 116]
    );
    assert!(Arc::ptr_eq(
        after
            .node(text)
            .unwrap()
            .data()
            .unwrap()
            .shared_code_units(),
        original_text.shared_code_units()
    ));
}

#[test]
fn one_long_native_transaction_reuses_first_late_hot_patterns() {
    let mut dom = Dom::new(
        DomLimits::new(
            1_500,
            1_000_000,
            8_192,
            0,
            0,
            8,
            16_000_000,
            DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_input_processing(65_536, 1_000_000)
        .unwrap(),
    )
    .unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let mut cold = Vec::new();
    // These are authored fixture counts, not a pool capacity or turnover oracle.
    // Every distinct bank and Text remains live through the single publication.
    for index in 0..640 {
        let input = batch.create_element_ns(&document, Namespace::html(), "input");
        batch.set_attribute(&input, "type", "text");
        batch.set_attribute(&input, "id", format!("c{index:03}"));
        batch.set_attribute(&input, "value", "cold");
        let text = batch.create_text(&document, format!("row{index:03}"));
        cold.push((input, text));
    }
    let first = batch.create_element_ns(&document, Namespace::html(), "input");
    batch.set_attribute(&first, "type", "text");
    batch.set_attribute(&first, "id", "hot");
    batch.set_attribute(&first, "value", "seed");
    let first_text = batch.create_text(&document, units(RAW));
    let second = batch.create_element_ns(&document, Namespace::html(), "input");
    batch.set_attribute(&second, "type", "text");
    batch.set_attribute(&second, "id", "hot");
    batch.set_attribute(&second, "value", "seed");
    let second_text = batch.create_text(&document, units(RAW));
    let done = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    for (index, (input, text)) in cold.iter().enumerate() {
        let input = done.created(input).unwrap().unwrap();
        let text = done.created(text).unwrap().unwrap();
        let node = snapshot.node(input).unwrap();
        let attributes = node.as_element().unwrap().attributes();
        assert_eq!(attributes.len(), 3);
        assert_eq!(
            attributes[0].name().local_name().code_units(),
            [116, 121, 112, 101]
        );
        assert_eq!(attributes[0].value().code_units(), [116, 101, 120, 116]);
        assert_eq!(attributes[1].name().local_name().code_units(), [105, 100]);
        assert!(
            attributes[1]
                .value()
                .code_units()
                .iter()
                .copied()
                .eq(format!("c{index:03}").encode_utf16())
        );
        assert_eq!(
            attributes[2].name().local_name().code_units(),
            [118, 97, 108, 117, 101]
        );
        assert_eq!(attributes[2].value().code_units(), [99, 111, 108, 100]);
        assert_eq!(node.as_input().unwrap().value(), [99, 111, 108, 100]);
        assert!(!node.as_input().unwrap().state().flags().dirty_value());
        assert!(
            snapshot
                .node(text)
                .unwrap()
                .data()
                .unwrap()
                .code_units()
                .iter()
                .copied()
                .eq(format!("row{index:03}").encode_utf16())
        );
    }
    let first = done.created(&first).unwrap().unwrap();
    let second = done.created(&second).unwrap().unwrap();
    let first_text = done.created(&first_text).unwrap().unwrap();
    let second_text = done.created(&second_text).unwrap().unwrap();
    assert_ne!(first, second);
    assert_ne!(first_text, second_text);
    for input in [first, second] {
        let node = snapshot.node(input).unwrap();
        let attributes = node.as_element().unwrap().attributes();
        assert_eq!(attributes.len(), 3);
        assert_eq!(
            attributes[0].name().local_name().code_units(),
            [116, 121, 112, 101]
        );
        assert_eq!(attributes[0].value().code_units(), [116, 101, 120, 116]);
        assert_eq!(attributes[1].name().local_name().code_units(), [105, 100]);
        assert_eq!(attributes[1].value().code_units(), [104, 111, 116]);
        assert_eq!(
            attributes[2].name().local_name().code_units(),
            [118, 97, 108, 117, 101]
        );
        assert_eq!(attributes[2].value().code_units(), [115, 101, 101, 100]);
        assert_eq!(node.as_input().unwrap().value(), [115, 101, 101, 100]);
        assert!(!node.as_input().unwrap().state().flags().dirty_value());
    }
    for text in [first_text, second_text] {
        assert_eq!(
            snapshot.node(text).unwrap().data().unwrap().code_units(),
            RAW
        );
    }
    let bank_shared = std::ptr::eq(
        snapshot
            .node(first)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
        snapshot
            .node(second)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes(),
    );
    let text_shared = Arc::ptr_eq(
        snapshot
            .node(first_text)
            .unwrap()
            .data()
            .unwrap()
            .shared_code_units(),
        snapshot
            .node(second_text)
            .unwrap()
            .data()
            .unwrap()
            .shared_code_units(),
    );
    assert!(
        bank_shared && text_shared,
        "late hot patterns in one transaction must reuse backing: bank={bank_shared}, Text={text_shared}"
    );
}
