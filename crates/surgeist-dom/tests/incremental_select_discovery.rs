//! HTML-owned pruning and disabled-state observations through public operations.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;

fn limits() -> DomLimits {
    DomLimits::new(
        256,
        32768,
        1024,
        256,
        128,
        16,
        1_000_000,
        surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_select_processing(128, 4_000_000, 10_000_000)
    .unwrap()
}
fn option(
    batch: &mut DomBatch,
    document: &CreatedNode,
    parent: &CreatedNode,
    value: &str,
) -> CreatedNode {
    let option = batch.create_element_ns(document, Namespace::html(), "option");
    batch.set_attribute(&option, "value", value);
    batch.insert_before(&option, parent, None);
    option
}
fn selected(dom: &Dom, option: NodeId) -> bool {
    dom.snapshot()
        .node(option)
        .unwrap()
        .as_option()
        .unwrap()
        .state()
        .flags()
        .selectedness()
}

#[test]
fn option_membership_keeps_html_pruning_and_foreign_wrappers_distinct() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let select = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&select, "multiple", "");
    let first = option(&mut setup, &document, &select, "first");
    let pruned_inside_option = option(&mut setup, &document, &first, "inside-option");
    let foreign = setup.create_element_ns(&document, Namespace::from_uri("urn:foreign"), "select");
    setup.insert_before(&foreign, &select, None);
    let second = option(&mut setup, &document, &foreign, "second");
    let group = setup.create_element_ns(&document, Namespace::html(), "optgroup");
    setup.insert_before(&group, &select, None);
    let third = option(&mut setup, &document, &group, "third");
    let nested_group = setup.create_element_ns(&document, Namespace::html(), "optgroup");
    setup.insert_before(&nested_group, &group, None);
    let pruned_nested_group = option(&mut setup, &document, &nested_group, "nested-group");
    let mut pruned = vec![pruned_inside_option, pruned_nested_group];
    for tag in ["hr", "datalist", "select"] {
        let barrier = setup.create_element_ns(&document, Namespace::html(), tag);
        setup.insert_before(&barrier, &select, None);
        pruned.push(option(&mut setup, &document, &barrier, tag));
    }
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let select = id(&select);
    let actual = [id(&first), id(&second), id(&third)];
    let pruned = pruned.iter().map(id).collect::<Vec<_>>();
    let original_pruned = pruned
        .iter()
        .map(|node| selected(&dom, *node))
        .collect::<Vec<_>>();
    for ordinal in 0..3 {
        let mut edit = DomBatch::new();
        edit.set_select_selected_index(select, ordinal as i64);
        dom.apply(edit, &cancel).unwrap();
        assert_eq!(
            actual.map(|node| selected(&dom, node)),
            [ordinal == 0, ordinal == 1, ordinal == 2]
        );
        assert_eq!(
            pruned
                .iter()
                .map(|node| selected(&dom, *node))
                .collect::<Vec<_>>(),
            original_pruned
        );
    }
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(select, 3);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(actual.map(|node| selected(&dom, node)), [false; 3]);
}

#[test]
fn moved_option_membership_and_cached_select_restore_after_a_late_failure() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let source = setup.create_element_ns(&document, Namespace::html(), "select");
    let destination = setup.create_element_ns(&document, Namespace::html(), "select");
    setup.set_attribute(&source, "multiple", "");
    setup.set_attribute(&destination, "multiple", "");
    let moved = option(&mut setup, &document, &source, "moved");
    let retained = option(&mut setup, &document, &destination, "retained");
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let source = id(&source);
    let destination = id(&destination);
    let moved = id(&moved);
    let retained = id(&retained);
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(source, 0);
    edit.set_select_selected_index(destination, 0);
    dom.apply(edit, &cancel).unwrap();
    let before = dom.snapshot();
    let mut refused = DomBatch::new();
    refused.insert_before(moved, destination, None);
    refused.insert_before(source, source, None);
    assert_eq!(
        dom.apply(refused, &cancel).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(
        dom.snapshot()
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(source)
    );
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(source, -1);
    dom.apply(edit, &cancel).unwrap();
    assert!(!selected(&dom, moved));
    assert!(selected(&dom, retained));
    let mut edit = DomBatch::new();
    edit.insert_before(moved, destination, None);
    edit.set_select_selected_index(destination, 1);
    dom.apply(edit, &cancel).unwrap();
    assert!(selected(&dom, moved));
    assert!(!selected(&dom, retained));
    assert_eq!(
        dom.snapshot()
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(destination)
    );
    assert_eq!(
        before
            .node(moved)
            .unwrap()
            .as_option()
            .unwrap()
            .cached_select()
            .unwrap(),
        Some(source)
    );
}

#[test]
fn selectedcontent_scope_uses_actual_disabled_hooks_and_move_rollback() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(limits()).unwrap();
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let outer = setup.create_element_ns(&document, Namespace::html(), "select");
    let chosen = option(&mut setup, &document, &outer, "outer");
    let text = setup.create_text(&document, "outer");
    setup.insert_before(&text, &chosen, None);
    let datalist = setup.create_element_ns(&document, Namespace::html(), "datalist");
    setup.insert_before(&datalist, &outer, None);
    let enabled = setup.create_element_ns(&document, Namespace::html(), "selectedcontent");
    setup.insert_before(&enabled, &datalist, None);
    let nested = setup.create_element_ns(&document, Namespace::html(), "select");
    let nested_option = option(&mut setup, &document, &nested, "inner");
    let text = setup.create_text(&document, "inner");
    setup.insert_before(&text, &nested_option, None);
    let nested_target = setup.create_element_ns(&document, Namespace::html(), "selectedcontent");
    let sentinel = setup.create_text(&document, "not refreshed while disabled");
    setup.insert_before(&sentinel, &nested_target, None);
    setup.insert_before(&nested_target, &nested, None);
    setup.insert_before(&nested, &outer, None);
    let destination = setup.create_element_ns(&document, Namespace::html(), "div");
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let outer = id(&outer);
    let nested = id(&nested);
    let enabled = id(&enabled);
    let nested_target = id(&nested_target);
    let sentinel = id(&sentinel);
    let destination = id(&destination);
    assert!(
        dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let text_at = |dom: &Dom, target| {
        let snapshot = dom.snapshot();
        let children = snapshot.node(target).unwrap().children();
        assert_eq!(children.len(), 1);
        snapshot.node(children[0]).unwrap().data().unwrap().clone()
    };
    let mut edit = DomBatch::new();
    edit.set_select_selected_index(outer, 0);
    dom.apply(edit, &cancel).unwrap();
    assert_eq!(text_at(&dom, enabled), DocumentString::from("outer"));
    assert_eq!(
        dom.snapshot().node(nested_target).unwrap().children(),
        &[sentinel]
    );
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.insert_before(nested, destination, None);
    edit.set_select_selected_index(nested, 0);
    dom.apply(edit, &cancel).unwrap();
    assert!(
        !dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    assert_eq!(text_at(&dom, nested_target), DocumentString::from("inner"));
    assert!(
        old.node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    assert_eq!(old.node(nested_target).unwrap().children(), &[sentinel]);
    let before = dom.snapshot();
    let mut refused = DomBatch::new();
    refused.insert_before(nested, outer, None);
    refused.insert_before(destination, destination, None);
    assert_eq!(
        dom.apply(refused, &cancel).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert!(
        !dom.snapshot()
            .node(nested_target)
            .unwrap()
            .as_selectedcontent()
            .unwrap()
            .disabled()
    );
    let mut retry = DomBatch::new();
    retry.set_select_selected_index(nested, 0);
    dom.apply(retry, &cancel).unwrap();
    assert_eq!(text_at(&dom, nested_target), DocumentString::from("inner"));
}
