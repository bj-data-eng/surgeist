//! Independent DOM3071 order, identity, Range and failure oracles.
use super::*;
use crate::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits() -> DomLimits {
    DomLimits::new(
        256,
        65_536,
        512,
        1024,
        512,
        64,
        1_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(64)
    .unwrap()
}
fn source_limits() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 2_000_000, 20_000_000, 128, 4096, 4096, 8)
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn fixture() -> (Dom, DocumentId, NodeId, [NodeId; 4]) {
    let mut dom = Dom::new(limits()).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element(&doc, "root");
    batch.insert_before(&root, &doc, None);
    let children = ["a", "b", "c", "d"].map(|name| {
        let node = batch.create_element(&doc, name);
        batch.insert_before(&node, &root, None);
        node
    });
    let commit = apply(&mut dom, batch);
    (
        dom,
        commit.document(&doc).unwrap(),
        commit.created(&root).unwrap().unwrap(),
        children.map(|node| commit.created(&node).unwrap().unwrap()),
    )
}
fn run(dom: &mut Dom, edit: DomSourceEdit) -> DomSourceProgress {
    let mut lease = dom
        .begin_source_edit(edit, source_limits(), &cancel())
        .unwrap();
    lease.advance(&cancel()).unwrap()
}
fn complete(dom: &mut Dom, edit: DomSourceEdit) {
    assert!(matches!(run(dom, edit), DomSourceProgress::Complete(_)));
}
fn children(dom: &Dom, node: NodeId) -> Vec<NodeId> {
    dom.snapshot().node(node).unwrap().children().to_vec()
}

#[test]
fn append_and_prepend_preserve_duplicate_node_order_fragments_and_exact_utf16() {
    let (mut dom, doc, root, [a, b, c, d]) = fixture();
    let mut batch = DomBatch::new();
    let fragment = batch.create_fragment(doc);
    let x = batch.create_comment(doc, "x");
    let y = batch.create_comment(doc, "y");
    batch.insert_before(&x, &fragment, None);
    batch.insert_before(&y, &fragment, None);
    let commit = apply(&mut dom, batch);
    let (fragment, x, y) = (
        commit.created(&fragment).unwrap().unwrap(),
        commit.created(&x).unwrap().unwrap(),
        commit.created(&y).unwrap().unwrap(),
    );
    let exact = DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]);
    complete(
        &mut dom,
        DomSourceEdit::append(
            root,
            vec![a.into(), exact.clone().into(), fragment.into(), a.into()],
        ),
    );
    let order = children(&dom, root);
    assert_eq!([order[0], order[1], order[2]], [b, c, d]);
    assert_eq!(&order[4..], &[x, y, a]);
    assert_eq!(dom.snapshot().node(order[3]).unwrap().data(), Some(&exact));
    assert!(children(&dom, fragment).is_empty());
    complete(
        &mut dom,
        DomSourceEdit::prepend(root, vec![c.into(), a.into(), c.into()]),
    );
    assert_eq!(children(&dom, root), vec![a, c, b, d, order[3], x, y]);
    complete(
        &mut dom,
        DomSourceEdit::prepend(root, vec![a.into(), DocumentString::from("p").into()]),
    );
    let prepended = children(&dom, root);
    assert_eq!(prepended[0], a);
    assert_eq!(&prepended[2..], &[c, b, d, order[3], x, y]);
    assert_eq!(
        dom.snapshot().node(prepended[1]).unwrap().data(),
        Some(&DocumentString::from("p"))
    );
}

#[test]
fn child_methods_select_viable_siblings_before_self_and_duplicate_conversion() {
    let (mut dom, _doc, root, [a, b, c, d]) = fixture();
    complete(
        &mut dom,
        DomSourceEdit::before(c, vec![b.into(), c.into(), b.into()]),
    );
    assert_eq!(children(&dom, root), [a, c, b, d]);
    complete(
        &mut dom,
        DomSourceEdit::after(c, vec![c.into(), b.into(), c.into()]),
    );
    assert_eq!(children(&dom, root), [a, b, c, d]);
    complete(
        &mut dom,
        DomSourceEdit::replace_with(c, vec![c.into(), b.into()]),
    );
    assert_eq!(children(&dom, root), [a, c, b, d]);
    complete(&mut dom, DomSourceEdit::replace_with(c, vec![]));
    assert_eq!(children(&dom, root), [a, b, d]);
    let usage = dom.usage();
    complete(
        &mut dom,
        DomSourceEdit::before(c, vec![DocumentString::from("unallocated").into()]),
    );
    assert_eq!(dom.usage(), usage);
}

#[test]
fn replace_children_excludes_original_document_children_and_reclaims_unpublished_fragment() {
    let (mut dom, doc, root, _) = fixture();
    let mut batch = DomBatch::new();
    let replacement = batch.create_element(doc, "new");
    let commit = apply(&mut dom, batch);
    let replacement = commit.created(&replacement).unwrap().unwrap();
    complete(
        &mut dom,
        DomSourceEdit::replace_children(doc.node(), vec![replacement.into()]),
    );
    assert_eq!(children(&dom, doc.node()), [replacement]);
    assert_eq!(dom.snapshot().node(root).unwrap().parent(), None);
    let count = dom.usage().managed_nodes();
    complete(
        &mut dom,
        DomSourceEdit::replace_children(doc.node(), vec![]),
    );
    assert!(children(&dom, doc.node()).is_empty());
    assert_eq!(dom.usage().managed_nodes(), count);
}

#[test]
fn later_hierarchy_failure_publishes_conversion_detaches_but_simple_insertion_still_rolls_back() {
    let (mut dom, doc, root, _) = fixture();
    let before = dom.snapshot().stamp();
    let DomSourceProgress::Failed(failure) = run(
        &mut dom,
        DomSourceEdit::append(
            doc.node(),
            vec![root.into(), DocumentString::from("x").into()],
        ),
    ) else {
        panic!("ordinary hierarchy failure")
    };
    assert_eq!(failure.error(), &DomError::HierarchyRequest);
    assert_eq!(failure.commit().changes().from(), before);
    assert_eq!(failure.commit().changes().to(), dom.snapshot().stamp());
    assert_ne!(dom.snapshot().stamp(), before);
    assert!(children(&dom, doc.node()).is_empty());
    let fragment = dom.snapshot().node(root).unwrap().parent().unwrap();
    let converted = children(&dom, fragment);
    assert_eq!(converted[0], root);
    assert_eq!(
        dom.snapshot().node(converted[1]).unwrap().data(),
        Some(&DocumentString::from("x"))
    );
    assert!(dom.pending_source_edit().is_none());
    let mut batch = DomBatch::new();
    let text = batch.create_text(doc, "invalid");
    let text = apply(&mut dom, batch).created(&text).unwrap().unwrap();
    let before = dom.snapshot().stamp();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(text, doc.node(), None),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        assert_eq!(
            lease.advance(&cancel()).unwrap_err(),
            DomError::HierarchyRequest
        );
    }
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.snapshot().node(text).unwrap().parent(), None);
}

#[test]
fn text_content_uses_character_data_and_replace_all_range_algorithms_and_explicit_noops() {
    let (mut dom, doc, root, [a, b, _, _]) = fixture();
    let mut batch = DomBatch::new();
    let text = batch.create_text(doc, "abcdef");
    batch.insert_before(&text, a, None);
    let range = batch.create_range(doc);
    batch.set_range_start(&range, &text, 2);
    batch.set_range_end(&range, root, 3);
    let commit = apply(&mut dom, batch);
    let text = commit.created(&text).unwrap().unwrap();
    let range = commit.created_range(&range).unwrap().unwrap();
    complete(&mut dom, DomSourceEdit::set_text_content(text, "z"));
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.range(range).unwrap().start().container(), text);
    assert_eq!(snapshot.range(range).unwrap().start().offset(), 0);
    complete(
        &mut dom,
        DomSourceEdit::set_text_content(root, DocumentString::from_utf16(vec![0xdfff, 0])),
    );
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(a).unwrap().parent(), None);
    assert_eq!(snapshot.node(b).unwrap().parent(), None);
    assert_eq!(
        (
            snapshot.range(range).unwrap().start().container(),
            snapshot.range(range).unwrap().start().offset()
        ),
        (root, 0)
    );
    assert_eq!(
        (
            snapshot.range(range).unwrap().end().container(),
            snapshot.range(range).unwrap().end().offset()
        ),
        (root, 0)
    );
    let child = children(&dom, root)[0];
    assert_eq!(
        snapshot.node(child).unwrap().data().unwrap().code_units(),
        [0xdfff, 0]
    );
    let before = snapshot.stamp();
    complete(
        &mut dom,
        DomSourceEdit::set_text_content(doc.node(), "ignored"),
    );
    assert_eq!(dom.snapshot().stamp(), before);
    complete(&mut dom, DomSourceEdit::set_text_content(root, ""));
    assert!(children(&dom, root).is_empty());
}

#[test]
fn adjacent_text_creates_exact_empty_text_and_interprets_ascii_positions_after_creation() {
    let (mut dom, doc, root, [a, b, c, d]) = fixture();
    complete(
        &mut dom,
        DomSourceEdit::insert_adjacent_text(b, "BeFoReBeGiN", ""),
    );
    let order = children(&dom, root);
    assert_eq!([order[0], order[2], order[3], order[4]], [a, b, c, d]);
    assert_eq!(
        dom.snapshot().node(order[1]).unwrap().data(),
        Some(&DocumentString::from(""))
    );
    complete(
        &mut dom,
        DomSourceEdit::insert_adjacent_text(
            a,
            "afterbegin",
            DocumentString::from_utf16(vec![0xd800]),
        ),
    );
    assert_eq!(
        dom.snapshot()
            .node(children(&dom, a)[0])
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [0xd800]
    );
    let usage = dom.usage();
    let before = dom.snapshot().stamp();
    let DomSourceProgress::Failed(failure) = run(
        &mut dom,
        DomSourceEdit::insert_adjacent_text(a, " beforeend", "x"),
    ) else {
        panic!("SyntaxError outcome")
    };
    assert_eq!(failure.error(), &DomError::InvalidAdjacentPosition);
    assert_eq!(dom.usage(), usage);
    assert_eq!(dom.snapshot().stamp(), before);
    let DomSourceProgress::Failed(failure) = run(
        &mut dom,
        DomSourceEdit::insert_adjacent_text(root, "afterend", "x"),
    ) else {
        panic!("Text cannot enter Document")
    };
    assert_eq!(failure.error(), &DomError::HierarchyRequest);
    assert_eq!(children(&dom, doc.node()), [root]);
    assert_eq!(dom.usage(), usage);
}

#[test]
fn conversion_admits_the_temporary_fragment_peak_and_exact_text_units() {
    for (nodes, units, succeeds) in [(8, 41, true), (7, 41, false), (8, 40, false)] {
        let (mut dom, _doc, root, [a, ..]) = fixture();
        assert_eq!(dom.usage().managed_nodes(), 6);
        assert_eq!(dom.usage().string_code_units(), 39);
        let mut bounds = limits();
        bounds.nodes = nodes;
        bounds.units = units;
        dom.set_limits(bounds).unwrap();
        let before = dom.snapshot().stamp();
        let result = {
            let mut lease = dom
                .begin_source_edit(
                    DomSourceEdit::append(root, vec![a.into(), DocumentString::from("xy").into()]),
                    source_limits(),
                    &cancel(),
                )
                .unwrap();
            lease.advance(&cancel())
        };
        if succeeds {
            assert!(matches!(result, Ok(DomSourceProgress::Complete(_))));
            assert_eq!(dom.usage().managed_nodes(), 7);
            assert_eq!(dom.usage().string_code_units(), 41);
        } else {
            assert!(matches!(
                result,
                Err(DomError::ResourceLimit {
                    resource: DomResource::ManagedNodes | DomResource::StringCodeUnits,
                    ..
                })
            ));
            assert_eq!(dom.snapshot().stamp(), before);
            assert_eq!(dom.usage().managed_nodes(), 6);
        }
    }
}

#[test]
fn ancestor_conversion_failure_preserves_original_ownership_and_stops_later_operands() {
    let (mut dom, doc, root, [a, b, c, d]) = fixture();
    let DomSourceProgress::Failed(failure) = run(
        &mut dom,
        DomSourceEdit::append(a, vec![root.into(), b.into()]),
    ) else {
        panic!("converted fragment contains the destination's ancestor")
    };
    assert_eq!(failure.error(), &DomError::HierarchyRequest);
    assert!(children(&dom, doc.node()).is_empty());
    let fragment = dom.snapshot().node(root).unwrap().parent().unwrap();
    assert_eq!(children(&dom, fragment), [root, b]);
    assert_eq!(children(&dom, root), [a, c, d]);
    assert_eq!(dom.snapshot().node(a).unwrap().owner_document(), Some(doc));
}

#[test]
fn conversion_failure_keeps_earlier_moves_stops_later_nodes_and_reclaims_unused_text() {
    let (mut dom, doc, root, [a, b, c, d]) = fixture();
    let DomSourceProgress::Failed(failure) = run(
        &mut dom,
        DomSourceEdit::append(
            a,
            vec![
                c.into(),
                doc.node().into(),
                b.into(),
                DocumentString::from("unused").into(),
            ],
        ),
    ) else {
        panic!("Document cannot be appended to the conversion fragment")
    };
    assert_eq!(failure.error(), &DomError::HierarchyRequest);
    assert_eq!(children(&dom, root), [a, b, d]);
    assert_eq!(children(&dom, doc.node()), [root]);
    assert!(children(&dom, a).is_empty());
    let fragment = dom.snapshot().node(c).unwrap().parent().unwrap();
    assert_eq!(children(&dom, fragment), [c]);
    assert_eq!(dom.snapshot().node(b).unwrap().parent(), Some(root));
    assert_eq!(dom.usage().managed_nodes(), 7);
}

#[test]
fn adjacent_text_creation_refusal_precedes_invalid_position_and_does_not_publish() {
    let (mut dom, _doc, root, _) = fixture();
    let mut bounds = limits();
    bounds.nodes = 6;
    dom.set_limits(bounds).unwrap();
    let before = dom.snapshot().stamp();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_adjacent_text(root, "invalid", "x"),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()),
            Err(DomError::ResourceLimit {
                resource: DomResource::ManagedNodes,
                limit: 6
            })
        ));
    }
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(dom.usage().managed_nodes(), 6);
}

#[test]
fn all_strings_are_created_before_an_earlier_invalid_node_operand_is_appended() {
    let (mut dom, doc, root, _) = fixture();
    let mut bounds = limits();
    bounds.nodes = 6;
    dom.set_limits(bounds).unwrap();
    let before = dom.snapshot().stamp();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::append(
                    root,
                    vec![doc.node().into(), DocumentString::from("x").into()],
                ),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()),
            Err(DomError::ResourceLimit {
                resource: DomResource::ManagedNodes,
                limit: 6
            })
        ));
    }
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(children(&dom, doc.node()), [root]);
}

#[test]
fn original_empty_operand_capacity_is_admitted_before_conversion() {
    let (mut dom, _doc, root, _) = fixture();
    let policy = DomSourceLimits::new(1_048_576, 2_000_000, 20_000_000, 128, 4096, 4096, 8);
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::append(root, Vec::new()), policy, &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let before = dom.snapshot().stamp();
    let capacity = 1_048_576 / size_of::<DomNodeOrString>() + 1;
    let operands = Vec::with_capacity(capacity);
    assert!(matches!(
        dom.begin_source_edit(DomSourceEdit::append(root, operands), policy, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: 1_048_576
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.pending_source_edit().is_none());
}

#[test]
fn text_content_handles_comment_pi_fragment_shadow_and_doctype_by_actual_interface() {
    let (mut dom, doc, root, _) = fixture();
    let mut batch = DomBatch::new();
    let comment = batch.create_comment(doc, "old");
    let pi = batch.create_processing_instruction(doc, "go", "old");
    let doctype = batch.create_document_type(doc, "html", "", "");
    let fragment = batch.create_fragment(doc);
    let host = batch.create_element_ns(doc, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let commit = apply(&mut dom, batch);
    let [comment, pi, doctype, fragment] =
        [&comment, &pi, &doctype, &fragment].map(|ticket| commit.created(ticket).unwrap().unwrap());
    let shadow = commit.attached_shadow_root(&shadow).unwrap().unwrap();
    complete(
        &mut dom,
        DomSourceEdit::set_text_content(comment, DocumentString::from_utf16(vec![0xdc00, 0])),
    );
    complete(&mut dom, DomSourceEdit::set_text_content(pi, "?>"));
    assert_eq!(
        dom.snapshot()
            .node(comment)
            .unwrap()
            .data()
            .unwrap()
            .code_units(),
        [0xdc00, 0]
    );
    assert_eq!(
        dom.snapshot().node(pi).unwrap().data(),
        Some(&DocumentString::from("?>"))
    );
    for node in [fragment, shadow] {
        complete(&mut dom, DomSourceEdit::set_text_content(node, "text"));
        let child = children(&dom, node)[0];
        assert_eq!(
            dom.snapshot().node(child).unwrap().data(),
            Some(&DocumentString::from("text"))
        );
    }
    let before = dom.snapshot().stamp();
    complete(
        &mut dom,
        DomSourceEdit::set_text_content(doctype, "ignored"),
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(children(&dom, root).len(), 4);
}

#[test]
fn original_compound_style_request_survives_drop_and_cancellation_then_finishes_its_tail_once() {
    let (mut dom, doc, root, _) = fixture();
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(doc, Namespace::html(), "style");
    let style = apply(&mut dom, batch).created(&style).unwrap().unwrap();
    let (operation, request) = {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::append(
                    root,
                    vec![style.into(), DocumentString::from("tail").into()],
                ),
                source_limits(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("real Style CSP boundary")
        };
        let DomSourceProgress::Request(DomSourceRequest::Style(repeated)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("unaccepted request stays pending")
        };
        assert_eq!(request.source().ordinal(), repeated.source().ordinal());
        let order = lease.snapshot().node(root).unwrap().children().to_vec();
        // The first actual Style service interrupts fragment transfer before
        // the later Text insertion; the parent tail has not run early.
        assert_eq!(order.len(), 5);
        assert_eq!(order[4], style);
        (lease.operation(), request)
    };
    assert!(dom.pending_source_edit().unwrap().has_published());
    let interrupted = AtomicBool::new(true);
    assert!(matches!(
        dom.resume_source_edit(operation, source_limits(), &interrupted),
        Err(DomError::Cancelled)
    ));
    {
        let mut lease = dom
            .resume_source_edit(operation, source_limits(), &cancel())
            .unwrap();
        lease.request_cancel();
        lease
            .accept_inline_csp(
                &request,
                &DomStyleInlineCspObservation::evaluated(
                    request.source(),
                    Arc::new(vec!["style-src 'none'".into()]),
                    1,
                    false,
                ),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
            panic!("denied sheet keeps its actual native tail")
        };
        assert!(done.cancellation_requested());
    }
    assert!(dom.pending_source_edit().is_none());
    assert_eq!(children(&dom, root).len(), 6);
    let tail = children(&dom, root)[5];
    assert_eq!(
        dom.snapshot().node(tail).unwrap().data(),
        Some(&DocumentString::from("tail"))
    );
    assert!(
        dom.snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
}
