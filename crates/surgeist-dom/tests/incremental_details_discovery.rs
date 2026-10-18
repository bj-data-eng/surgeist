//! Literal Details groups and immediate hooks; no index-layout expectations.
use std::sync::atomic::AtomicBool;
use surgeist_dom::*;
use surgeist_html::HtmlDetailsState::{Closed, Open};

#[test]
fn exact_details_names_stay_root_scoped_and_failed_renaming_restores_the_group() {
    let cancel = AtomicBool::new(false);
    let mut dom = Dom::new(
        DomLimits::new(128, 32768, 512, 0, 0, 16, 1_000_000)
            .unwrap()
            .with_details_toggle_intentions(32)
            .unwrap(),
    )
    .unwrap();
    let raw = DocumentString::from_utf16(vec![0xd800]);
    let mut setup = DomBatch::new();
    let document = setup.create_document();
    let root = setup.create_element_ns(&document, Namespace::html(), "div");
    let other_root = setup.create_element_ns(&document, Namespace::html(), "div");
    let peer = setup.create_element_ns(&document, Namespace::html(), "details");
    setup.set_attribute(&peer, "name", raw.clone());
    setup.set_attribute(&peer, "open", "");
    setup.insert_before(&peer, &root, None);
    let target = setup.create_element_ns(&document, Namespace::html(), "details");
    setup.set_attribute(&target, "name", raw.clone());
    setup.insert_before(&target, &root, None);
    let distinct = setup.create_element_ns(&document, Namespace::html(), "details");
    setup.set_attribute(&distinct, "name", "\u{fffd}");
    setup.set_attribute(&distinct, "open", "");
    setup.insert_before(&distinct, &root, None);
    let elsewhere = setup.create_element_ns(&document, Namespace::html(), "details");
    setup.set_attribute(&elsewhere, "name", raw.clone());
    setup.set_attribute(&elsewhere, "open", "");
    setup.insert_before(&elsewhere, &other_root, None);
    let mut empty_names = Vec::new();
    for _ in 0..2 {
        let node = setup.create_element_ns(&document, Namespace::html(), "details");
        setup.set_attribute(&node, "name", "");
        setup.set_attribute(&node, "open", "");
        setup.insert_before(&node, &root, None);
        empty_names.push(node);
    }
    let commit = dom.apply(setup, &cancel).unwrap();
    let id = |ticket: &CreatedNode| commit.created(ticket).unwrap().unwrap();
    let root = id(&root);
    let peer = id(&peer);
    let target = id(&target);
    let distinct = id(&distinct);
    let elsewhere = id(&elsewhere);
    let empty_names = empty_names.iter().map(id).collect::<Vec<_>>();
    dom.take_details_toggle_intentions(32, &cancel).unwrap();
    let old = dom.snapshot();
    let mut edit = DomBatch::new();
    edit.set_attribute(target, "open", "");
    dom.apply(edit, &cancel).unwrap();
    let state = |dom: &Dom, node| {
        dom.snapshot()
            .node(node)
            .unwrap()
            .as_details()
            .unwrap()
            .state()
    };
    assert_eq!((state(&dom, target), state(&dom, peer)), (Open, Closed));
    for node in [distinct, elsewhere].into_iter().chain(empty_names) {
        assert_eq!(state(&dom, node), Open);
    }
    let changes = dom.take_details_toggle_intentions(32, &cancel).unwrap();
    assert_eq!(
        changes
            .intentions()
            .iter()
            .map(|row| (row.target(), row.old_state(), row.new_state()))
            .collect::<Vec<_>>(),
        [(target, Closed, Open), (peer, Open, Closed)]
    );
    assert_eq!(old.node(peer).unwrap().as_details().unwrap().state(), Open);

    let mut edit = DomBatch::new();
    edit.set_attribute(target, "name", "");
    edit.set_attribute(peer, "open", "");
    dom.apply(edit, &cancel).unwrap();
    dom.take_details_toggle_intentions(32, &cancel).unwrap();
    let before = dom.snapshot();
    let mut refused = DomBatch::new();
    refused.set_attribute(target, "name", raw.clone());
    refused.insert_before(root, root, None);
    assert_eq!(
        dom.apply(refused, &cancel).unwrap_err(),
        DomError::HierarchyRequest
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!((state(&dom, target), state(&dom, peer)), (Open, Open));
    assert_eq!(dom.pending_details_toggle_intentions(), 0);
    let mut retry = DomBatch::new();
    retry.set_attribute(target, "name", raw);
    dom.apply(retry, &cancel).unwrap();
    assert_eq!((state(&dom, target), state(&dom, peer)), (Closed, Open));
    let changes = dom.take_details_toggle_intentions(32, &cancel).unwrap();
    assert_eq!(
        changes
            .intentions()
            .iter()
            .map(|row| (row.target(), row.old_state(), row.new_state()))
            .collect::<Vec<_>>(),
        [(target, Open, Closed)]
    );
}
