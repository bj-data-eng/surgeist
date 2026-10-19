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

#[test]
fn later_values_and_text_reuse_existing_canonical_name_text_without_rebinding_atoms() {
    const URI: &[u16] = &[117, 114, 110, 58, 115, 104, 97, 114, 101, 100];
    const LOCAL: &[u16] = &[115, 104, 97, 114, 101, 100];
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(document, Namespace::from_uri(units(URI)), units(LOCAL));
    let element = apply(&mut dom, batch).created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    let old_node = before.node(element).unwrap();
    let old_name = old_node.as_element().unwrap().name();
    let namespace = old_name.namespace().unwrap().clone();
    let local = old_name.local_name_atom().clone();
    assert_eq!(namespace.text().code_units(), URI);
    assert_eq!(local.text().code_units(), LOCAL);
    assert!(old_name.prefix().is_none());
    assert!(old_node.as_element().unwrap().attributes().is_empty());

    let mut batch = DomBatch::new();
    batch.set_attribute(element, "namespace-value", units(URI));
    batch.set_attribute(element, "local-value", units(LOCAL));
    let namespace_text = batch.create_text(document, units(URI));
    let local_text = batch.create_text(document, units(LOCAL));
    let done = apply(&mut dom, batch);
    let namespace_text = done.created(&namespace_text).unwrap().unwrap();
    let local_text = done.created(&local_text).unwrap().unwrap();
    let after = dom.snapshot();
    let node = after.node(element).unwrap();
    let value = node.as_element().unwrap();
    assert_eq!(value.name(), old_name);
    assert_eq!(
        value.name().namespace().unwrap().same_identity(&namespace),
        Ok(true)
    );
    assert_eq!(
        value.name().local_name_atom().same_identity(&local),
        Ok(true)
    );
    assert_eq!(value.name().namespace_uri().unwrap().code_units(), URI);
    assert_eq!(value.name().local_name().code_units(), LOCAL);
    assert!(value.name().prefix().is_none());
    let attributes = value.attributes();
    assert_eq!(attributes.len(), 2);
    assert!(
        attributes[0]
            .name()
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq("namespace-value".encode_utf16())
    );
    assert!(
        attributes[1]
            .name()
            .local_name()
            .code_units()
            .iter()
            .copied()
            .eq("local-value".encode_utf16())
    );
    assert_eq!(attributes[0].value().code_units(), URI);
    assert_eq!(attributes[1].value().code_units(), LOCAL);
    assert_eq!(
        after
            .node(namespace_text)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        URI
    );
    assert_eq!(
        after.node(local_text).unwrap().data().unwrap().code_units(),
        LOCAL
    );
    assert!(
        before
            .node(element)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
            .is_empty()
    );
    assert!(
        Arc::ptr_eq(
            namespace.text().shared_code_units(),
            attributes[0].value().shared_code_units()
        ),
        "a later value must reuse the actual canonical namespace text"
    );
    assert!(
        Arc::ptr_eq(
            local.text().shared_code_units(),
            attributes[1].value().shared_code_units()
        ),
        "a later value must reuse the actual canonical local-name text"
    );
    assert!(Arc::ptr_eq(
        namespace.text().shared_code_units(),
        after
            .node(namespace_text)
            .unwrap()
            .data()
            .unwrap()
            .shared_code_units()
    ));
    assert!(Arc::ptr_eq(
        local.text().shared_code_units(),
        after
            .node(local_text)
            .unwrap()
            .data()
            .unwrap()
            .shared_code_units()
    ));
}

fn prefix_seed(dom: &mut Dom, document: DocumentId) -> (NodeId, NodeId) {
    let mut batch = DomBatch::new();
    let element = batch.create_element(document, "seed");
    batch.set_attribute(&element, "data", units(&[112]));
    let text = batch.create_text(document, units(&[112]));
    let done = apply(dom, batch);
    (
        done.created(&element).unwrap().unwrap(),
        done.created(&text).unwrap().unwrap(),
    )
}

fn assert_prefixed_values(snapshot: &DomSnapshot, element: NodeId) {
    let node = snapshot.node(element).unwrap();
    let value = node.as_element().unwrap();
    assert_eq!(
        value.name().namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 112, 114, 101, 102, 105, 120]
    );
    assert_eq!(value.name().prefix().unwrap().code_units(), [112]);
    assert_eq!(
        value.name().local_name().code_units(),
        [116, 97, 114, 103, 101, 116]
    );
    let attributes = value.attributes();
    assert_eq!(attributes.len(), 2);
    assert_eq!(
        attributes[0].name().namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 97, 116, 116, 114]
    );
    assert_eq!(attributes[0].name().prefix().unwrap().code_units(), [112]);
    assert_eq!(
        attributes[0].name().local_name().code_units(),
        [108, 97, 98, 101, 108]
    );
    assert_eq!(attributes[0].value().code_units(), [112]);
    assert!(attributes[1].name().namespace_uri().is_none());
    assert!(attributes[1].name().prefix().is_none());
    assert_eq!(
        attributes[1].name().local_name().code_units(),
        [111, 116, 104, 101, 114]
    );
    assert_eq!(attributes[1].value().code_units(), [107, 101, 112, 116]);
}

#[test]
fn new_and_imported_prefixes_reuse_earlier_values_without_merging_names_or_mutations() {
    let mut dom = owner();
    let source_document = document(&mut dom);
    let (seed, text) = prefix_seed(&mut dom, source_document);
    let seeded = dom.snapshot();
    let seed_node = seeded.node(seed).unwrap();
    let seeded_value = seed_node.as_element().unwrap().attributes()[0].value();
    let seeded_text = seeded.node(text).unwrap().data().unwrap();
    assert_eq!(seeded_value.code_units(), [112]);
    assert_eq!(seeded_text.code_units(), [112]);
    let namespace = dom
        .bind_name(
            &units(&[117, 114, 110, 58, 112, 114, 101, 102, 105, 120]),
            &AtomicBool::new(false),
        )
        .unwrap();
    let local = dom
        .bind_name(
            &units(&[116, 97, 114, 103, 101, 116]),
            &AtomicBool::new(false),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    let element = batch.create_element_ns(
        source_document,
        Namespace::from_uri("urn:prefix"),
        "p:target",
    );
    batch.set_attribute_ns(
        &element,
        Namespace::from_uri("urn:attr"),
        "p:label",
        units(&[112]),
    );
    batch.set_attribute(&element, "other", "kept");
    let element = apply(&mut dom, batch).created(&element).unwrap().unwrap();
    let before = dom.snapshot();
    assert_prefixed_values(&before, element);
    let mut batch = DomBatch::new();
    let copy = batch.import_node(&before, element, source_document, true);
    let copy = apply(&mut dom, batch).created(&copy).unwrap().unwrap();
    let copied = dom.snapshot();
    for id in [element, copy] {
        assert_prefixed_values(&copied, id);
        let node = copied.node(id).unwrap();
        let name = node.as_element().unwrap().name();
        assert_eq!(
            name.namespace().unwrap().same_identity(&namespace),
            Ok(true)
        );
        assert_eq!(name.local_name_atom().same_identity(&local), Ok(true));
        assert!(
            Arc::ptr_eq(
                name.prefix().unwrap().shared_code_units(),
                seeded_text.shared_code_units()
            ),
            "new native/imported prefix must reuse an earlier immutable Text value"
        );
    }
    assert!(Arc::ptr_eq(
        seeded_value.shared_code_units(),
        seeded_text.shared_code_units()
    ));

    let mut destination = owner();
    let target_document = document(&mut destination);
    let (target_seed, target_text) = prefix_seed(&mut destination, target_document);
    let target_namespace = destination
        .bind_name(
            &units(&[117, 114, 110, 58, 112, 114, 101, 102, 105, 120]),
            &AtomicBool::new(false),
        )
        .unwrap();
    let target_local = destination
        .bind_name(
            &units(&[116, 97, 114, 103, 101, 116]),
            &AtomicBool::new(false),
        )
        .unwrap();
    let mut batch = DomBatch::new();
    let imported = batch.import_node(&before, element, target_document, true);
    let imported = apply(&mut destination, batch)
        .created(&imported)
        .unwrap()
        .unwrap();
    let imported_snapshot = destination.snapshot();
    assert_prefixed_values(&imported_snapshot, imported);
    let imported_node = imported_snapshot.node(imported).unwrap();
    let imported_name = imported_node.as_element().unwrap().name();
    assert_eq!(
        imported_name
            .namespace()
            .unwrap()
            .same_identity(&target_namespace),
        Ok(true)
    );
    assert_eq!(
        imported_name.local_name_atom().same_identity(&target_local),
        Ok(true)
    );
    assert_eq!(
        namespace.same_identity(&target_namespace),
        Err(surgeist_dom::DomError::ForeignOwner)
    );
    assert_eq!(
        local.same_identity(&target_local),
        Err(surgeist_dom::DomError::ForeignOwner)
    );
    let target_text_value = imported_snapshot.node(target_text).unwrap().data().unwrap();
    assert_eq!(target_text_value.code_units(), [112]);
    assert!(
        Arc::ptr_eq(
            imported_name.prefix().unwrap().shared_code_units(),
            target_text_value.shared_code_units()
        ),
        "foreign import must select the destination owner's earlier prefix backing"
    );
    let target_seed_node = imported_snapshot.node(target_seed).unwrap();
    assert!(Arc::ptr_eq(
        target_seed_node.as_element().unwrap().attributes()[0]
            .value()
            .shared_code_units(),
        target_text_value.shared_code_units()
    ));

    // Public names are immutable. Removing/re-adding this attribute changes its
    // prefix and order through the actual attribute hooks, with a new value.
    let mut batch = DomBatch::new();
    batch.remove_attribute_ns(element, Namespace::from_uri("urn:attr"), "label");
    batch.set_attribute_ns(element, Namespace::from_uri("urn:attr"), "q:label", "q");
    apply(&mut dom, batch);
    let after = dom.snapshot();
    let changed_node = after.node(element).unwrap();
    let changed = changed_node.as_element().unwrap();
    assert_eq!(
        changed
            .name()
            .namespace()
            .unwrap()
            .same_identity(&namespace),
        Ok(true)
    );
    assert_eq!(
        changed.name().local_name_atom().same_identity(&local),
        Ok(true)
    );
    assert_eq!(changed.name().prefix().unwrap().code_units(), [112]);
    assert!(Arc::ptr_eq(
        changed.name().prefix().unwrap().shared_code_units(),
        seeded_text.shared_code_units()
    ));
    assert_eq!(changed.attributes().len(), 2);
    assert_eq!(
        changed.attributes()[0].name().local_name().code_units(),
        [111, 116, 104, 101, 114]
    );
    assert_eq!(
        changed.attributes()[0].value().code_units(),
        [107, 101, 112, 116]
    );
    assert_eq!(
        changed.attributes()[1]
            .name()
            .prefix()
            .unwrap()
            .code_units(),
        [113]
    );
    assert_eq!(changed.attributes()[1].value().code_units(), [113]);
    assert_prefixed_values(&before, element);
    assert_prefixed_values(&after, copy);
    assert_prefixed_values(&imported_snapshot, imported);
    assert_eq!(
        after.node(text).unwrap().data().unwrap().code_units(),
        [112]
    );
    assert_eq!(
        after.node(seed).unwrap().as_element().unwrap().attributes()[0]
            .value()
            .code_units(),
        [112]
    );
}

fn readoption_element(batch: &mut DomBatch, document: DocumentId) -> surgeist_dom::CreatedNode {
    let element = batch.create_element(document, "cell");
    batch.set_attribute(&element, "title", units(&[111, 108, 100]));
    batch.set_attribute_ns(
        &element,
        Namespace::from_uri("urn:readopt"),
        "p:label",
        units(RAW),
    );
    batch.set_attribute(&element, "class", units(&[107, 101, 112, 116]));
    element
}

fn assert_readoption_values(snapshot: &DomSnapshot, element: NodeId, title: &[u16]) {
    let node = snapshot.node(element).unwrap();
    let value = node.as_element().unwrap();
    assert!(value.name().namespace_uri().is_none());
    assert!(value.name().prefix().is_none());
    assert_eq!(value.name().local_name().code_units(), [99, 101, 108, 108]);
    let attributes = value.attributes();
    assert_eq!(attributes.len(), 3);
    assert!(attributes[0].name().namespace_uri().is_none());
    assert!(attributes[0].name().prefix().is_none());
    assert_eq!(
        attributes[0].name().local_name().code_units(),
        [116, 105, 116, 108, 101]
    );
    assert_eq!(attributes[0].value().code_units(), title);
    assert_eq!(
        attributes[1].name().namespace_uri().unwrap().code_units(),
        [117, 114, 110, 58, 114, 101, 97, 100, 111, 112, 116]
    );
    assert_eq!(attributes[1].name().prefix().unwrap().code_units(), [112]);
    assert_eq!(
        attributes[1].name().local_name().code_units(),
        [108, 97, 98, 101, 108]
    );
    assert_eq!(attributes[1].value().code_units(), RAW);
    assert!(attributes[2].name().namespace_uri().is_none());
    assert!(attributes[2].name().prefix().is_none());
    assert_eq!(
        attributes[2].name().local_name().code_units(),
        [99, 108, 97, 115, 115]
    );
    assert_eq!(attributes[2].value().code_units(), [107, 101, 112, 116]);
}

#[test]
fn historical_weak_bank_is_readopted_by_fresh_creation_before_snapshot_import() {
    let mut dom = owner();
    let document = document(&mut dom);
    let original = {
        let mut batch = DomBatch::new();
        let element = readoption_element(&mut batch, document);
        apply(&mut dom, batch).created(&element).unwrap().unwrap()
    };
    let historical = dom.snapshot();
    assert_readoption_values(&historical, original, &[111, 108, 100]);
    let mut batch = DomBatch::new();
    batch.destroy(original);
    apply(&mut dom, batch);
    assert_eq!(
        dom.snapshot().node(original).unwrap_err(),
        surgeist_dom::DomError::StaleNode
    );

    // This independently authored creation receives no historical snapshot or
    // borrowed old attributes. Only that snapshot keeps the retired bank alive.
    let fresh = {
        let mut batch = DomBatch::new();
        let element = readoption_element(&mut batch, document);
        apply(&mut dom, batch).created(&element).unwrap().unwrap()
    };
    assert_ne!(original, fresh);
    let recreated = dom.snapshot();
    assert_readoption_values(&recreated, fresh, &[111, 108, 100]);
    let original_node = historical.node(original).unwrap();
    let fresh_node = recreated.node(fresh).unwrap();
    let old = original_node.as_element().unwrap();
    let new = fresh_node.as_element().unwrap();
    assert_eq!(old.name().same_expanded_identity(new.name()), Ok(true));
    for (old, new) in old.attributes().iter().zip(new.attributes()) {
        assert_eq!(old.name().same_expanded_identity(new.name()), Ok(true));
    }
    assert!(
        std::ptr::eq(old.attributes(), new.attributes()),
        "fresh independent creation must readopt the externally surviving equal bank"
    );

    // Snapshot ingress is deliberately later: it must not preadmit the old
    // bank/name graph before the fresh-creation readoption above.
    let imported = {
        let mut batch = DomBatch::new();
        let element = batch.import_node(&historical, original, document, true);
        apply(&mut dom, batch).created(&element).unwrap().unwrap()
    };
    assert_ne!(fresh, imported);
    let imported_snapshot = dom.snapshot();
    assert_readoption_values(&imported_snapshot, imported, &[111, 108, 100]);
    let imported_node = imported_snapshot.node(imported).unwrap();
    let imported_value = imported_node.as_element().unwrap();
    assert_eq!(
        old.name().same_expanded_identity(imported_value.name()),
        Ok(true)
    );
    for (old, imported) in old.attributes().iter().zip(imported_value.attributes()) {
        assert_eq!(old.name().same_expanded_identity(imported.name()), Ok(true));
    }
    assert!(std::ptr::eq(old.attributes(), imported_value.attributes()));

    let mut batch = DomBatch::new();
    batch.set_attribute(fresh, "title", "changed");
    apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_readoption_values(&after, fresh, &[99, 104, 97, 110, 103, 101, 100]);
    assert_readoption_values(&after, imported, &[111, 108, 100]);
    assert_readoption_values(&historical, original, &[111, 108, 100]);
    assert_readoption_values(&recreated, fresh, &[111, 108, 100]);
    assert!(std::ptr::eq(
        old.attributes(),
        after
            .node(imported)
            .unwrap()
            .as_element()
            .unwrap()
            .attributes()
    ));

    let mut batch = DomBatch::new();
    batch.destroy(fresh);
    batch.destroy(imported);
    apply(&mut dom, batch);
    let mut batch = DomBatch::new();
    let alive = batch.create_text(document, "alive");
    let alive = apply(&mut dom, batch).created(&alive).unwrap().unwrap();
    assert_eq!(
        dom.snapshot()
            .node(alive)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [97, 108, 105, 118, 101]
    );
    drop(dom);
    assert_readoption_values(&historical, original, &[111, 108, 100]);
    assert_readoption_values(&imported_snapshot, imported, &[111, 108, 100]);
}

#[test]
fn distinct_rows_keep_parentage_and_ids_while_equal_child_values_share() {
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let table = batch.create_element(document, "table");
    batch.insert_before(&table, document, None);
    let mut rows = Vec::new();
    let mut cells = Vec::new();
    let mut texts = Vec::new();
    for row_id in ["row-a", "row-b"] {
        let row = batch.create_element(document, "row");
        batch.set_attribute(&row, "id", row_id);
        batch.insert_before(&row, &table, None);
        for _ in 0..2 {
            let cell = batch.create_element(document, "cell");
            batch.set_attribute(&cell, "class", "repeat");
            batch.set_attribute_ns(&cell, Namespace::from_uri("urn:row"), "p:label", units(RAW));
            let text = batch.create_text(document, units(RAW));
            batch.insert_before(&text, &cell, None);
            batch.insert_before(&cell, &row, None);
            cells.push(cell);
            texts.push(text);
        }
        rows.push(row);
    }
    let done = apply(&mut dom, batch);
    let table = done.created(&table).unwrap().unwrap();
    let rows: Vec<_> = rows
        .iter()
        .map(|node| done.created(node).unwrap().unwrap())
        .collect();
    let cells: Vec<_> = cells
        .iter()
        .map(|node| done.created(node).unwrap().unwrap())
        .collect();
    let texts: Vec<_> = texts
        .iter()
        .map(|node| done.created(node).unwrap().unwrap())
        .collect();
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(table).unwrap().children(), rows);
    let all_ids: std::collections::HashSet<_> =
        rows.iter().chain(&cells).chain(&texts).copied().collect();
    assert_eq!(all_ids.len(), 10);
    for (index, row) in rows.iter().copied().enumerate() {
        let node = snapshot.node(row).unwrap();
        assert_eq!(node.parent(), Some(table));
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.children(), &cells[index * 2..index * 2 + 2]);
        let element = node.as_element().unwrap();
        assert_eq!(element.name().local_name().code_units(), [114, 111, 119]);
        let attributes = element.attributes();
        assert_eq!(attributes.len(), 1);
        assert!(attributes[0].name().namespace_uri().is_none());
        assert!(attributes[0].name().prefix().is_none());
        assert_eq!(attributes[0].name().local_name().code_units(), [105, 100]);
        assert_eq!(
            attributes[0].value().code_units(),
            if index == 0 {
                &[114, 111, 119, 45, 97][..]
            } else {
                &[114, 111, 119, 45, 98][..]
            }
        );
    }
    for (index, cell) in cells.iter().copied().enumerate() {
        let node = snapshot.node(cell).unwrap();
        assert_eq!(node.parent(), Some(rows[index / 2]));
        assert_eq!(node.owner_document(), Some(document));
        assert_eq!(node.children(), [texts[index]]);
        let element = node.as_element().unwrap();
        assert!(element.name().namespace_uri().is_none());
        assert!(element.name().prefix().is_none());
        assert_eq!(
            element.name().local_name().code_units(),
            [99, 101, 108, 108]
        );
        let attributes = element.attributes();
        assert_eq!(attributes.len(), 2);
        assert!(attributes[0].name().namespace_uri().is_none());
        assert!(attributes[0].name().prefix().is_none());
        assert_eq!(
            attributes[0].name().local_name().code_units(),
            [99, 108, 97, 115, 115]
        );
        assert_eq!(
            attributes[0].value().code_units(),
            [114, 101, 112, 101, 97, 116]
        );
        assert_eq!(
            attributes[1].name().namespace_uri().unwrap().code_units(),
            [117, 114, 110, 58, 114, 111, 119]
        );
        assert_eq!(attributes[1].name().prefix().unwrap().code_units(), [112]);
        assert_eq!(
            attributes[1].name().local_name().code_units(),
            [108, 97, 98, 101, 108]
        );
        assert_eq!(attributes[1].value().code_units(), RAW);
        let text = snapshot.node(texts[index]).unwrap();
        assert_eq!(text.parent(), Some(cell));
        assert_eq!(text.owner_document(), Some(document));
        assert_eq!(text.data().unwrap().code_units(), RAW);
    }
    let first_cell = snapshot.node(cells[0]).unwrap();
    let first_bank = first_cell.as_element().unwrap().attributes();
    let first_text = snapshot.node(texts[0]).unwrap().data().unwrap();
    for index in 1..4 {
        let cell = snapshot.node(cells[index]).unwrap();
        assert!(std::ptr::eq(
            first_bank,
            cell.as_element().unwrap().attributes()
        ));
        let text = snapshot.node(texts[index]).unwrap().data().unwrap();
        assert!(Arc::ptr_eq(
            first_text.shared_code_units(),
            text.shared_code_units()
        ));
    }
}

fn assert_large_pattern(value: &DocumentString) {
    assert_eq!(value.code_units().len(), 16_384);
    for chunk in value.code_units().chunks_exact(4) {
        assert_eq!(chunk, [65, 0xd800, 0, 0xdc00]);
    }
}

#[test]
fn large_caller_shared_values_keep_backing_capacity_and_snapshot_edit_isolation() {
    let mut data = Vec::with_capacity(32_768);
    for _ in 0..4096 {
        data.extend_from_slice(&[65, 0xd800, 0, 0xdc00]);
    }
    let caller = DocumentString::from_utf16(data);
    let capacity = caller.shared_code_units().capacity();
    assert_large_pattern(&caller);
    let mut dom = owner();
    let document = document(&mut dom);
    let mut batch = DomBatch::new();
    let element = batch.create_element(document, "large");
    batch.set_attribute(&element, "payload", caller.clone());
    let text = batch.create_text(document, caller.clone());
    let done = apply(&mut dom, batch);
    let element = done.created(&element).unwrap().unwrap();
    let text = done.created(&text).unwrap().unwrap();
    let before = dom.snapshot();
    let original_node = before.node(element).unwrap();
    let original_attributes = original_node.as_element().unwrap().attributes();
    assert_eq!(original_attributes.len(), 1);
    assert_eq!(
        original_attributes[0].name().local_name().code_units(),
        [112, 97, 121, 108, 111, 97, 100]
    );
    assert!(original_attributes[0].name().namespace_uri().is_none());
    assert!(original_attributes[0].name().prefix().is_none());
    let original_text = before.node(text).unwrap().data().unwrap();
    for value in [original_attributes[0].value(), original_text] {
        assert_large_pattern(value);
        assert_eq!(value.shared_code_units().capacity(), capacity);
        assert!(Arc::ptr_eq(
            caller.shared_code_units(),
            value.shared_code_units()
        ));
    }

    let mut batch = DomBatch::new();
    batch.set_attribute(element, "payload", "changed");
    apply(&mut dom, batch);
    let attribute_changed = dom.snapshot();
    let node = attribute_changed.node(element).unwrap();
    assert_eq!(
        node.as_element().unwrap().attributes()[0]
            .value()
            .code_units(),
        [99, 104, 97, 110, 103, 101, 100]
    );
    let unchanged_text = attribute_changed.node(text).unwrap().data().unwrap();
    assert_large_pattern(unchanged_text);
    assert_eq!(unchanged_text.shared_code_units().capacity(), capacity);
    assert!(Arc::ptr_eq(
        caller.shared_code_units(),
        unchanged_text.shared_code_units()
    ));
    assert_large_pattern(original_attributes[0].value());

    let mut batch = DomBatch::new();
    batch.replace_data(text, 0, 1, "B");
    apply(&mut dom, batch);
    let after = dom.snapshot();
    let changed_text = after.node(text).unwrap().data().unwrap();
    assert_eq!(changed_text.code_units().len(), 16_384);
    assert_eq!(changed_text.code_units()[0], 66);
    for (offset, unit) in changed_text
        .code_units()
        .iter()
        .copied()
        .enumerate()
        .skip(1)
    {
        assert_eq!(unit, [65, 0xd800, 0, 0xdc00][offset % 4]);
    }
    let node = after.node(element).unwrap();
    assert_eq!(
        node.as_element().unwrap().attributes()[0]
            .value()
            .code_units(),
        [99, 104, 97, 110, 103, 101, 100]
    );
    for value in [
        &caller,
        original_attributes[0].value(),
        original_text,
        unchanged_text,
    ] {
        assert_large_pattern(value);
        assert_eq!(value.shared_code_units().capacity(), capacity);
        assert!(Arc::ptr_eq(
            caller.shared_code_units(),
            value.shared_code_units()
        ));
    }
}
