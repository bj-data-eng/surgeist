//! Literal native outcomes from HTML877d #dom-select-value,
//! #set-the-selected-index, #dom-option-selected and #get-html-aware-text-content.
//! These tests exercise new public commands, without preimplementation RED.

use std::{cell::Cell, sync::atomic::AtomicBool};
use surgeist_dom::{
    Dom, DomBatch, DomCommit, DomError, DomSnapshot, identity::NodeId, limits::DomLimits,
};
use surgeist_html::values::{DocumentString, Namespace};
use surgeist_html_semantics::{HtmlCaptureLimits, HtmlSemanticLimits};

fn limits() -> DomLimits {
    DomLimits::new(
        512,
        32768,
        2048,
        2048,
        256,
        16,
        1_000_000,
        surgeist_dom::limits::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_select_processing(128, 4_000_000, 10_000_000)
    .unwrap()
    .with_select_value_queries(
        HtmlCaptureLimits::new(512, 4096, 512, 32768, 512, 0, 0, 16_777_216),
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
fn select_value_uses_first_selected_identity_even_when_its_value_is_empty() {
    let mut fixture = fixture(true);
    let [first, _, tail] = fixture.options;
    let mut batch = DomBatch::new();
    batch.set_attribute(first, "value", "");
    apply(&mut fixture.dom, batch);
    let original = fixture.dom.snapshot();

    let mut edit = DomBatch::new();
    edit.insert_before(tail, fixture.select, Some(first.into()));
    apply(&mut fixture.dom, edit);
    let reordered = fixture.dom.snapshot();
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(fixture.select, -1);
    apply(&mut fixture.dom, edit);
    let unselected = fixture.dom.snapshot();

    for (snapshot, expected) in [
        (&original, vec![vec![]]),
        (&reordered, vec![vec![116, 97, 105, 108]]),
        (&unselected, vec![]),
    ] {
        let view = snapshot.node(fixture.select).unwrap().as_select().unwrap();
        let mut output = Vec::new();
        view.visit_value_with(
            || Ok::<_, DomError>(()),
            |_| -> Result<(), _> { panic!("raw selected values need no traversal scratch") },
            |units| {
                output.push(units.to_vec());
                Ok(())
            },
        )
        .unwrap();
        assert_eq!(output, expected);
    }
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
fn fallback_strings_prune_html_images_and_html_and_svg_scripts_with_exact_utf16() {
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
    for (namespace, name) in [
        (Namespace::html(), "script"),
        (Namespace::html(), "img"),
        (Namespace::svg(), "prefixed:script"),
    ] {
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
    batch.set_attribute_ns(
        &option,
        Namespace::from_uri("urn:test"),
        "p:value",
        "foreign value",
    );
    batch.set_attribute_ns(
        &option,
        Namespace::from_uri("urn:test"),
        "p:label",
        "foreign label",
    );
    batch.insert_before(&option, &select, None);
    let commit = apply(&mut dom, batch);
    let select = commit.created(&select).unwrap().unwrap();
    let option = commit.created(&option).unwrap().unwrap();

    // ASCII whitespace alone collapses; NBSP and unpaired UTF-16 units survive exactly.
    let expected = DocumentString::from_utf16(vec![
        65, 32, 66, 32, 67, 32, 68, 32, 0x00a0, 0xd800, 32, 0xdfff,
    ]);
    let snapshot = dom.snapshot();
    let option_view = snapshot.node(option).unwrap().as_option().unwrap();
    let mut value = Vec::new();
    let mut label = Vec::new();
    let mut text = Vec::new();
    let step = || Ok::<_, DomError>(());
    let reserve = |_| Ok(());
    option_view
        .visit_value_with(step, reserve, |units| {
            value.extend_from_slice(units);
            Ok(())
        })
        .unwrap();
    option_view
        .visit_label_with(step, reserve, |units| {
            label.extend_from_slice(units);
            Ok(())
        })
        .unwrap();
    option_view
        .visit_text_with(step, reserve, |units| {
            text.extend_from_slice(units);
            Ok(())
        })
        .unwrap();
    assert_eq!(value, expected.code_units());
    assert_eq!(label, expected.code_units());
    assert_eq!(text, expected.code_units());
    let mut batch = DomBatch::new();
    batch.set_select_value(select, expected.clone());
    apply(&mut dom, batch);
    assert_eq!(flags(&dom.snapshot(), option), (true, true));
    let snapshot = dom.snapshot();
    let select_view = snapshot.node(select).unwrap().as_select().unwrap();
    value.clear();
    select_view
        .visit_value_with(step, reserve, |units| {
            value.extend_from_slice(units);
            Ok(())
        })
        .unwrap();
    assert_eq!(value, expected.code_units());
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

#[test]
fn option_value_and_label_borrow_distinct_raw_attributes_and_present_empty_stops_fallback() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let nested = batch.create_element_ns(&document, Namespace::html(), "span");
    let text = batch.create_text(&document, "fallback A");
    batch.insert_before(&text, &nested, None);
    batch.insert_before(&nested, &option, None);
    let text = batch.create_text(&document, " fallback B");
    batch.insert_before(&text, &option, None);
    let value = [9, 0xd800, 32];
    let label = [32, 0xdfff, 10];
    batch.set_attribute(&option, "value", DocumentString::from_utf16(value));
    batch.set_attribute(&option, "label", DocumentString::from_utf16(label));
    let commit = apply(&mut dom, batch);
    let option = commit.created(&option).unwrap().unwrap();
    let original = dom.snapshot();

    let mut batch = DomBatch::new();
    batch.set_attribute(option, "value", "");
    batch.set_attribute(option, "label", "");
    apply(&mut dom, batch);
    let current = dom.snapshot();
    for (snapshot, value, label) in [
        (&original, value.as_slice(), label.as_slice()),
        (&current, &[], &[]),
    ] {
        let node = snapshot.node(option).unwrap();
        let view = node.as_option().unwrap();
        let attrs = node.attributes().unwrap();
        let stored = |name: &str| {
            attrs
                .get_named_item(&name.into())
                .unwrap()
                .value()
                .code_units()
        };
        let step = || Ok::<_, DomError>(());
        let reserve = |_| -> Result<(), _> {
            panic!("a present attribute must never traverse fallback text")
        };
        let mut value_calls = 0;
        view.visit_value_with(step, reserve, |units| {
            value_calls += 1;
            assert_eq!(units, value);
            assert!(std::ptr::eq(units, stored("value")));
            Ok(())
        })
        .unwrap();
        let mut label_calls = 0;
        let refused = view.visit_label_with(step, reserve, |units| {
            label_calls += 1;
            assert_eq!(units, label);
            assert!(std::ptr::eq(units, stored("label")));
            Err(DomError::Cancelled)
        });
        assert_eq!(refused, Err(DomError::Cancelled));
        assert_eq!((value_calls, label_calls), (1, 1));
    }
}

#[test]
fn option_text_admits_the_exact_reached_work_and_refuses_one_step_short() {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let data = [9, 65, 32, 32, 0xd800];
    let text = batch.create_cdata_section(&document, DocumentString::from_utf16(data));
    batch.insert_before(&text, &option, None);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let child = commit.created(&text).unwrap().unwrap();
    let view = snapshot
        .node(commit.created(&option).unwrap().unwrap())
        .unwrap()
        .as_option()
        .unwrap();

    // One ordinary child edge, one canonical lookup, and every input UTF-16 unit.
    let mut lookup_work = 0;
    snapshot
        .node_with(child, || {
            lookup_work += 1;
            Ok::<_, DomError>(())
        })
        .unwrap();
    let exact = 1 + lookup_work + data.len();
    for (limit, expected, output) in [
        (exact - 1, Err(DomError::Cancelled), &[65][..]),
        (exact, Ok(()), &[65, 32, 0xd800][..]),
    ] {
        let mut spent = 0;
        let mut emitted = Vec::new();
        let outcome = view.visit_text_with(
            || {
                if spent == limit {
                    return Err(DomError::Cancelled);
                }
                spent += 1;
                Ok(())
            },
            |_| -> Result<(), _> { panic!("a leaf has no pending sibling branches") },
            |units| {
                emitted.extend_from_slice(units);
                Ok(())
            },
        );
        assert_eq!(outcome, expected);
        assert_eq!(spent, limit);
        assert_eq!(emitted, output);
    }
}

#[test]
fn option_text_admits_overlapping_scratch_and_releases_backing_on_first_refusal() {
    struct Scratch<'a> {
        held: &'a Cell<usize>,
        bytes: usize,
    }
    impl Drop for Scratch<'_> {
        fn drop(&mut self) {
            self.held.set(self.held.get() - self.bytes);
        }
    }

    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let option = batch.create_element_ns(&document, Namespace::html(), "option");
    let outer = batch.create_element_ns(&document, Namespace::html(), "span");
    let inner = batch.create_element_ns(&document, Namespace::html(), "span");
    let text = batch.create_text(&document, " A  B  ");
    let inner_tail = batch.create_text(&document, "C");
    let outer_tail = batch.create_text(&document, "D");
    batch.insert_before(&text, &inner, None);
    batch.insert_before(&inner, &outer, None);
    batch.insert_before(&inner_tail, &outer, None);
    batch.insert_before(&outer, &option, None);
    batch.insert_before(&outer_tail, &option, None);
    let commit = apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    let option = snapshot
        .node(commit.created(&option).unwrap().unwrap())
        .unwrap()
        .as_option()
        .unwrap();
    let frame = size_of::<&[NodeId]>();
    let exact = frame * 3;
    let held = Cell::new(0);
    let peak = Cell::new(0);
    let admission = |limit, bytes| {
        let next = held.get() + bytes;
        if next > limit {
            return Err(DomError::AllocationFailed);
        }
        held.set(next);
        peak.set(peak.get().max(next));
        Ok(Scratch { held: &held, bytes })
    };

    let outcome = option.visit_text_with(
        || Ok(()),
        |bytes| admission(exact - 1, bytes),
        |_| panic!("output before traversal scratch admission"),
    );
    assert_eq!(outcome, Err(DomError::AllocationFailed));
    assert_eq!(held.get(), 0);
    assert_eq!(peak.get(), frame);

    let outcome = option.visit_text_with(
        || {
            if held.get() == exact {
                return Err(DomError::Cancelled);
            }
            Ok(())
        },
        |bytes| admission(exact, bytes),
        |_| panic!("output after traversal refused a frame move"),
    );
    assert_eq!(outcome, Err(DomError::Cancelled));
    assert_eq!(held.get(), 0);
    assert_eq!(peak.get(), exact);

    let expected = [65, 32, 66, 32, 67, 68];
    for output_limit in [expected.len() - 1, expected.len()] {
        let mut output = Vec::new();
        let outcome = option.visit_text_with(
            || Ok(()),
            |bytes| admission(exact, bytes),
            |units| {
                assert_eq!(held.get(), frame * 2);
                if output.len() + units.len() > output_limit {
                    return Err(DomError::NotSupported);
                }
                output.extend_from_slice(units);
                Ok(())
            },
        );
        assert_eq!(held.get(), 0);
        assert_eq!(peak.get(), exact);
        assert_eq!(output, &expected[..output_limit]);
        assert_eq!(
            outcome,
            if output_limit == expected.len() {
                Ok(())
            } else {
                Err(DomError::NotSupported)
            }
        );
    }
    assert_eq!(dom.snapshot().stamp(), snapshot.stamp());
}
