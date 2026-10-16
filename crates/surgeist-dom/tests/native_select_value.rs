//! Literal native outcomes from HTML877d #dom-select-value,
//! #set-the-selected-index, #dom-option-selected and #get-html-aware-text-content.
//! These tests exercise new public commands, without preimplementation RED.

use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

fn limits() -> DomLimits {
    DomLimits::new(512, 32768, 2048, 2048, 256, 16, 1_000_000)
        .unwrap()
        .with_select_processing(128, 4_000_000, 10_000_000)
        .unwrap()
        .with_select_value_queries(
            HtmlCaptureLimits::new(512, 4096, 512, 32768, 512, 0, 0),
            HtmlSemanticLimits::new(1_000_000, 16, 2048, 512, 4_000_000, 2048),
        )
}

fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &AtomicBool::new(false)).unwrap()
}

fn flags(snapshot: &DomSnapshot, option: NodeId) -> (bool, bool) {
    let node = snapshot.node(option).unwrap();
    let option = node.as_option().unwrap();
    let flags = option.state().flags();
    (flags.selectedness(), flags.dirtiness())
}

struct Fixture {
    dom: Dom,
    select: NodeId,
    options: [NodeId; 3],
}

fn fixture(multiple: bool) -> Fixture {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    if multiple {
        batch.set_attribute(&select, "multiple", "");
    }
    // Creation order differs from tree order, so identity order cannot be used.
    let second = batch.create_element_ns(&document, Namespace::html(), "option");
    let first = batch.create_element_ns(&document, Namespace::html(), "option");
    let third = batch.create_element_ns(&document, Namespace::html(), "option");
    for option in [&first, &second, &third] {
        batch.set_attribute(option, "selected", "");
        batch.insert_before(option, &select, None);
    }
    batch.set_attribute(&first, "value", "duplicate");
    batch.set_attribute(&second, "value", "duplicate");
    batch.set_attribute(&third, "value", "tail");
    let commit = apply(&mut dom, batch);
    Fixture {
        select: commit.created(&select).unwrap().unwrap(),
        options: [first, second, third].map(|ticket| commit.created(&ticket).unwrap().unwrap()),
        dom,
    }
}

fn assert_flags(fixture: &Fixture, expected: [(bool, bool); 3]) {
    let snapshot = fixture.dom.snapshot();
    assert_eq!(fixture.options.map(|id| flags(&snapshot, id)), expected);
    for id in fixture.options {
        assert_eq!(
            snapshot
                .node(id)
                .unwrap()
                .as_option()
                .unwrap()
                .cached_select()
                .unwrap(),
            Some(fixture.select)
        );
    }
}

#[test]
fn multiple_value_selects_first_tree_order_duplicate_and_preserves_other_dirtiness() {
    let mut f = fixture(true);
    assert_flags(&f, [(true, false), (true, false), (true, false)]);
    let mut batch = DomBatch::new();
    batch.set_option_selected(f.options[2], true);
    apply(&mut f.dom, batch);
    assert_flags(&f, [(true, false), (true, false), (true, true)]);

    let mut batch = DomBatch::new();
    batch.set_select_value(f.select, "duplicate");
    apply(&mut f.dom, batch);
    assert_flags(&f, [(true, true), (false, false), (false, true)]);

    let mut batch = DomBatch::new();
    batch.set_select_value(f.select, "absent");
    apply(&mut f.dom, batch);
    assert_flags(&f, [(false, true), (false, false), (false, true)]);
}

#[test]
fn selected_index_uses_tree_order_and_invalid_indices_leave_every_option_unselected() {
    for multiple in [false, true] {
        let mut f = fixture(multiple);
        let mut batch = DomBatch::new();
        batch.set_select_selected_index(f.select, 1);
        apply(&mut f.dom, batch);
        assert_flags(&f, [(false, false), (true, true), (false, false)]);

        for index in [-1, -2, i64::MIN, 3, i64::MAX] {
            let mut batch = DomBatch::new();
            batch.set_select_selected_index(f.select, 1);
            batch.set_select_selected_index(f.select, index);
            apply(&mut f.dom, batch);
            assert_flags(&f, [(false, false), (false, true), (false, false)]);
        }
        let mut batch = DomBatch::new();
        batch.set_select_selected_index(f.select, 2);
        apply(&mut f.dom, batch);
        assert_flags(&f, [(false, false), (false, true), (true, true)]);
    }
}

#[test]
fn option_false_runs_reset_normalization_but_select_setters_do_not() {
    let mut f = fixture(false);
    let mut batch = DomBatch::new();
    batch.set_select_selected_index(f.select, 0);
    apply(&mut f.dom, batch);
    let mut batch = DomBatch::new();
    batch.set_option_selected(f.options[0], false);
    apply(&mut f.dom, batch);
    // A size-one single select restores its first enabled option, preserving dirtiness.
    assert_flags(&f, [(true, true), (false, false), (false, false)]);

    for value_setter in [false, true] {
        let mut batch = DomBatch::new();
        if value_setter {
            batch.set_select_value(f.select, "no match");
        } else {
            batch.set_select_selected_index(f.select, -1);
        }
        apply(&mut f.dom, batch);
        assert_flags(&f, [(false, true), (false, false), (false, false)]);
    }
    let mut batch = DomBatch::new();
    batch.reset_select(f.select);
    apply(&mut f.dom, batch);
    // Reset restores authored selectedness, clears dirtiness, then retains the last selection.
    assert_flags(&f, [(false, false), (false, false), (true, false)]);
}

#[test]
fn option_value_prunes_html_and_svg_scripts_but_includes_foreign_names_and_exact_utf16() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let text = batch.create_text(&document, "\t A\r");
    batch.insert_before(&text, &option, None);
    let span = batch.create_element_ns(&document, Namespace::html(), "span");
    let text = batch.create_text(&document, "\n B\u{000c}");
    batch.insert_before(&text, &span, None);
    batch.insert_before(&span, &option, None);
    for (namespace, name) in [(Namespace::html(), "script"), (Namespace::svg(), "script")] {
        let pruned = batch.create_element_ns(&document, namespace, name);
        batch.set_attribute(&pruned, "alt", "excluded alt");
        let nested = batch.create_element_ns(&document, Namespace::html(), "span");
        let text = batch.create_text(&document, "excluded descendant");
        batch.insert_before(&text, &nested, None);
        batch.insert_before(&nested, &pruned, None);
        batch.insert_before(&pruned, &option, None);
    }
    for (name, value) in [("script", " C"), ("img", " D")] {
        let foreign = batch.create_element_ns(&document, Namespace::from_uri("urn:test"), name);
        let text = batch.create_text(&document, value);
        batch.insert_before(&text, &foreign, None);
        batch.insert_before(&foreign, &option, None);
    }
    let tail = batch.create_text(
        &document,
        DocumentString::from_utf16(vec![9, 0x00a0, 0xd800, 9, 0xdfff, 13]),
    );
    batch.insert_before(&tail, &option, None);
    batch.insert_before(&option, &select, None);
    let commit = apply(&mut dom, batch);
    let select = commit.created(&select).unwrap().unwrap();
    let option = commit.created(&option).unwrap().unwrap();

    // ASCII whitespace alone collapses; NBSP and unpaired UTF-16 units survive exactly.
    let expected = DocumentString::from_utf16(vec![
        65, 32, 66, 32, 67, 32, 68, 32, 0x00a0, 0xd800, 32, 0xdfff,
    ]);
    let mut batch = DomBatch::new();
    batch.set_select_value(select, expected);
    apply(&mut dom, batch);
    assert_eq!(flags(&dom.snapshot(), option), (true, true));
    let mut batch = DomBatch::new();
    batch.set_select_value(select, "A B excluded descendant C D");
    apply(&mut dom, batch);
    assert_eq!(flags(&dom.snapshot(), option), (false, true));
}

#[test]
fn explicit_value_attribute_wins_without_whitespace_or_utf16_normalization() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let select = batch.create_element_ns(&document, Namespace::html(), "select");
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let text = batch.create_text(&document, "fallback text");
    batch.insert_before(&text, &option, None);
    batch.insert_before(&option, &select, None);
    let exact = DocumentString::from_utf16(vec![9, 65, 13, 10, 0xd800, 32]);
    batch.set_attribute(&option, "value", exact.clone());
    let commit = apply(&mut dom, batch);
    let select = commit.created(&select).unwrap().unwrap();
    let option = commit.created(&option).unwrap().unwrap();

    for mismatch in [
        DocumentString::from("fallback text"),
        DocumentString::from_utf16(vec![65, 32, 0xd800]),
        DocumentString::from_utf16(vec![9, 65, 10, 0xd800, 32]),
    ] {
        let mut batch = DomBatch::new();
        batch.set_select_value(select, mismatch);
        apply(&mut dom, batch);
        assert_eq!(flags(&dom.snapshot(), option), (false, false));
    }
    let mut batch = DomBatch::new();
    batch.set_select_value(select, exact);
    apply(&mut dom, batch);
    assert_eq!(flags(&dom.snapshot(), option), (true, true));
}
