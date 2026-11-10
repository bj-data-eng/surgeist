//! DOM3071 normalize: descendant-only traversal, stable Text and Range transfers.
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(units: usize) -> DomLimits {
    DomLimits::new(
        256,
        units,
        512,
        1024,
        512,
        64,
        1_000_000,
        DomNameLimits::new(16_777_216, 16384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_ranges(64)
    .unwrap()
}
fn policy() -> DomSourceLimits {
    DomSourceLimits::new(64_000_000, 1_000_000, 20_000_000, 128, 4096, 4096, 8)
}
fn apply(dom: &mut Dom, batch: DomBatch) -> DomCommit {
    dom.apply(batch, &cancel()).unwrap()
}
fn range(
    dom: &mut Dom,
    doc: DocumentId,
    start: (NodeId, usize),
    end: (NodeId, usize),
) -> DomRangeId {
    let mut batch = DomBatch::new();
    let ticket = batch.create_range(doc);
    batch.set_range_start(&ticket, start.0, start.1);
    batch.set_range_end(&ticket, end.0, end.1);
    apply(dom, batch).created_range(&ticket).unwrap().unwrap()
}
fn points(snapshot: &DomSnapshot, id: DomRangeId) -> ((NodeId, usize), (NodeId, usize)) {
    let r = snapshot.range(id).unwrap();
    (
        (r.start().container(), r.start().offset()),
        (r.end().container(), r.end().offset()),
    )
}

#[test]
fn normalize_merges_into_first_nonempty_identity_and_transfers_text_and_parent_ranges() {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let values: Vec<DocumentString> = vec![
        "".into(),
        "A".into(),
        DocumentString::from_utf16(vec![0xd800, 0, 0xdc00]),
        "".into(),
        "Z".into(),
    ];
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element(&doc, "root");
    batch.insert_before(&root, &doc, None);
    let texts: Vec<_> = values
        .iter()
        .map(|value| {
            let text = batch.create_text(&doc, value.clone());
            batch.insert_before(&text, &root, None);
            text
        })
        .collect();
    let comment = batch.create_comment(&doc, "stop");
    batch.insert_before(&comment, &root, None);
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&doc).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let comment = commit.created(&comment).unwrap().unwrap();
    let texts: Vec<_> = texts
        .iter()
        .map(|text| commit.created(text).unwrap().unwrap())
        .collect();
    let survivor = texts[1];
    let ids: Vec<_> = texts
        .iter()
        .zip(&values)
        .map(|(id, value)| range(&mut dom, doc, (*id, 0), (*id, value.len())))
        .collect();
    let boundaries: Vec<_> = (0..=6)
        .map(|offset| range(&mut dom, doc, (root, offset), (root, offset)))
        .collect();
    let mut batch = DomBatch::new();
    batch.set_active_range(doc, Some(ids[2].into()));
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let nodes = dom.usage().managed_nodes();
    let mut batch = DomBatch::new();
    batch.normalize(root);
    let commit = apply(&mut dom, batch);
    let after = dom.snapshot();
    assert_eq!(after.node(root).unwrap().children(), [survivor, comment]);
    assert_eq!(
        after.node(survivor).unwrap().data().unwrap().code_units(),
        [0x41, 0xd800, 0, 0xdc00, 0x5a]
    );
    assert_eq!(points(&after, ids[0]), ((root, 0), (root, 0)));
    for (id, expected) in ids[1..].iter().zip([(0, 1), (1, 4), (4, 4), (4, 5)]) {
        assert_eq!(
            points(&after, *id),
            ((survivor, expected.0), (survivor, expected.1))
        );
    }
    for (id, expected) in boundaries.iter().zip([
        (root, 0),
        (root, 0),
        (survivor, 1),
        (survivor, 4),
        (survivor, 4),
        (root, 1),
        (root, 2),
    ]) {
        assert_eq!(points(&after, *id), (expected, expected));
    }
    for (index, id) in texts.iter().enumerate().filter(|(index, _)| *index != 1) {
        assert_eq!(after.node(*id).unwrap().parent(), None);
        assert_eq!(after.node(*id).unwrap().data(), Some(&values[index]));
    }
    assert_eq!(
        after.document(doc).unwrap().active_range(),
        DomActiveRangeState::Known(Some(ids[2]))
    );
    assert_eq!(dom.usage().managed_nodes(), nodes);
    assert!(
        !commit
            .changes()
            .changes()
            .iter()
            .any(|change| matches!(change, DomChange::Created(_) | DomChange::Destroyed(_)))
    );
    assert_eq!(before.node(root).unwrap().children().len(), 6);
    assert_eq!(points(&before, ids[2]), ((texts[2], 0), (texts[2], 3)));
}

#[test]
fn normalize_visits_ordinary_descendants_in_order_and_respects_comment_pi_and_owned_tree_boundaries()
 {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let root = batch.create_element_ns(&doc, Namespace::html(), "div");
    batch.insert_before(&root, &doc, None);
    let nested = batch.create_element(&doc, "nested");
    let template = batch.create_element_ns(&doc, Namespace::html(), "template");
    let shadow = batch.attach_shadow(&root, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let comment = batch.create_comment(&doc, "stop");
    let pi = batch.create_processing_instruction(&doc, "stop", "data");
    let a = batch.create_text(&doc, "a");
    let b = batch.create_text(&doc, "b");
    let c = batch.create_text(&doc, "c");
    let n1 = batch.create_text(&doc, "n");
    let n2 = batch.create_text(&doc, "2");
    let s1 = batch.create_text(&doc, "s");
    let s2 = batch.create_text(&doc, "2");
    for text in [&n1, &n2] {
        batch.insert_before(text, &nested, None);
    }
    for text in [&s1, &s2] {
        batch.insert_before(text, &shadow, None);
    }
    for child in [&a, &nested, &b, &comment, &c, &pi, &template] {
        batch.insert_before(child, &root, None);
    }
    let commit = apply(&mut dom, batch);
    let document = commit.document(&doc).unwrap();
    let root = commit.created(&root).unwrap().unwrap();
    let (nested, n1, n2, shadow, s1, s2, a, b, c) = (
        commit.created(&nested).unwrap().unwrap(),
        commit.created(&n1).unwrap().unwrap(),
        commit.created(&n2).unwrap().unwrap(),
        commit.created(&shadow).unwrap().unwrap(),
        commit.created(&s1).unwrap().unwrap(),
        commit.created(&s2).unwrap().unwrap(),
        commit.created(&a).unwrap().unwrap(),
        commit.created(&b).unwrap().unwrap(),
        commit.created(&c).unwrap().unwrap(),
    );
    let template = commit.created(&template).unwrap().unwrap();
    let contents = dom
        .snapshot()
        .node(template)
        .unwrap()
        .as_template()
        .unwrap()
        .contents()
        .unwrap();
    let owner = dom
        .snapshot()
        .node(contents)
        .unwrap()
        .owner_document()
        .unwrap();
    let mut batch = DomBatch::new();
    let t1 = batch.create_text(owner, "t");
    let t2 = batch.create_text(owner, "2");
    batch.insert_before(&t1, contents, None);
    batch.insert_before(&t2, contents, None);
    let commit = apply(&mut dom, batch);
    let t1 = commit.created(&t1).unwrap().unwrap();
    let t2 = commit.created(&t2).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.normalize(document);
    apply(&mut dom, batch);
    let snapshot = dom.snapshot();
    assert_eq!(snapshot.node(root).unwrap().children().len(), 7);
    assert_eq!(snapshot.node(nested).unwrap().children(), [n1]);
    assert_eq!(snapshot.node(n1).unwrap().data(), Some(&"n2".into()));
    assert_eq!(snapshot.node(n2).unwrap().parent(), None);
    for (node, value) in [(a, "a"), (b, "b"), (c, "c")] {
        assert_eq!(snapshot.node(node).unwrap().data(), Some(&value.into()));
    }
    assert_eq!(snapshot.node(shadow).unwrap().children(), [s1, s2]);
    assert_eq!(snapshot.node(contents).unwrap().children(), [t1, t2]);
    let mut batch = DomBatch::new();
    batch.normalize(shadow);
    batch.normalize(contents);
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().node(shadow).unwrap().children(), [s1]);
    assert_eq!(dom.snapshot().node(s1).unwrap().data(), Some(&"s2".into()));
    assert_eq!(dom.snapshot().node(contents).unwrap().children(), [t1]);
    assert_eq!(dom.snapshot().node(t1).unwrap().data(), Some(&"t2".into()));
}

#[test]
fn normalize_text_receiver_excludes_itself_and_returns_no_creation_result() {
    for value in ["", "abc"] {
        let mut dom = Dom::new(limits(65536)).unwrap();
        let mut batch = DomBatch::new();
        let doc = batch.create_document();
        let text = batch.create_text(&doc, value);
        batch.normalize(&text);
        let commit = apply(&mut dom, batch);
        let text = commit.created(&text).unwrap().unwrap();
        let before = dom.snapshot();
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::normalize(text), policy(), &cancel())
            .unwrap();
        let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
            panic!("Text receiver has no descendants")
        };
        assert_eq!(done.output(), None);
        assert_eq!(done.published_node(), None);
        assert!(done.commit().changes().changes().is_empty());
        drop(lease);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(
            dom.snapshot().node(text).unwrap().data(),
            Some(&value.into())
        );
    }
}

#[test]
fn normalize_removes_an_all_empty_descendant_chain_without_retiring_its_nodes() {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let texts: Vec<_> = (0..3)
        .map(|_| {
            let text = batch.create_text(&document, "");
            batch.insert_before(&text, &fragment, None);
            text
        })
        .collect();
    let commit = apply(&mut dom, batch);
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let texts: Vec<_> = texts
        .iter()
        .map(|text| commit.created(text).unwrap().unwrap())
        .collect();
    let nodes = dom.usage().managed_nodes();
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::normalize(fragment), policy(), &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("empty chain completes")
    };
    assert_eq!(done.output(), None);
    drop(lease);
    assert!(dom.snapshot().node(fragment).unwrap().children().is_empty());
    assert_eq!(dom.usage().managed_nodes(), nodes);
    for text in texts {
        assert_eq!(dom.snapshot().node(text).unwrap().parent(), None);
        assert_eq!(dom.snapshot().node(text).unwrap().data(), Some(&"".into()));
    }
}

fn deny(request: &DomStyleRequest) -> DomStyleInlineCspObservation {
    DomStyleInlineCspObservation::evaluated(
        request.source(),
        Arc::new(vec!["style-src 'none'".into()]),
        19,
        false,
    )
}

#[test]
fn normalize_source_preserves_append_transfer_and_each_removal_hook_across_drop_and_cancel() {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let style = batch.create_element_ns(&doc, Namespace::html(), "style");
    let texts: Vec<_> = ["ab", "cd", "", "ef"]
        .into_iter()
        .map(|value| {
            let node = batch.create_text(&doc, value);
            batch.insert_before(&node, &style, None);
            node
        })
        .collect();
    let commit = apply(&mut dom, batch);
    let doc = commit.document(&doc).unwrap();
    let style = commit.created(&style).unwrap().unwrap();
    let texts: Vec<_> = texts
        .iter()
        .map(|text| commit.created(text).unwrap().unwrap())
        .collect();
    {
        let mut lease = dom
            .begin_source_edit(
                DomSourceEdit::insert_before(style, doc.node(), None),
                policy(),
                &cancel(),
            )
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("connected Style policy")
        };
        lease
            .accept_inline_csp(&request, &deny(&request), &cancel())
            .unwrap();
        assert!(matches!(
            lease.advance(&cancel()).unwrap(),
            DomSourceProgress::Complete(_)
        ));
    }
    let r1 = range(&mut dom, doc, (texts[1], 1), (texts[1], 2));
    let r2 = range(&mut dom, doc, (style, 1), (texts[3], 2));
    let before = dom.snapshot();
    let nodes = dom.usage().managed_nodes();
    let mut batch = DomBatch::new();
    batch.normalize(style);
    assert!(matches!(
        dom.apply(batch, &cancel()),
        Err(DomError::StyleCompositionRequired(_))
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    let operation;
    let mut previous_work = 0;
    {
        let mut lease = dom
            .begin_source_edit(DomSourceEdit::normalize(style), policy(), &cancel())
            .unwrap();
        operation = lease.operation();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("append hook before transfers")
        };
        assert_eq!(
            request.source().text().as_ref(),
            &"abcdefcdef".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            points(&lease.snapshot(), r1),
            ((texts[1], 1), (texts[1], 2))
        );
    }
    for (ordinal, expected) in ["abcdefcdef", "abcdefef", "abcdefef", "abcdef"]
        .into_iter()
        .enumerate()
    {
        let mut lease = dom
            .resume_source_edit(operation, policy(), &cancel())
            .unwrap();
        let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
            lease.advance(&cancel()).unwrap()
        else {
            panic!("actual ordered normalize hook")
        };
        assert_eq!(request.source().ordinal(), ordinal as u64);
        assert_eq!(
            request.source().text().as_ref(),
            &expected.encode_utf16().collect::<Vec<_>>()
        );
        assert!(lease.status().spent_work() >= previous_work);
        previous_work = lease.status().spent_work();
        if ordinal > 0 {
            assert_eq!(
                points(&lease.snapshot(), r1),
                ((texts[0], 3), (texts[0], 4))
            );
            assert_eq!(
                points(&lease.snapshot(), r2),
                ((texts[0], 2), (texts[0], 6))
            );
        }
        lease.request_cancel();
        lease
            .accept_inline_csp(&request, &deny(&request), &cancel())
            .unwrap();
    }
    let mut lease = dom
        .resume_source_edit(operation, policy(), &cancel())
        .unwrap();
    let DomSourceProgress::Complete(done) = lease.advance(&cancel()).unwrap() else {
        panic!("original normalize completes")
    };
    assert_eq!(done.operation(), operation);
    assert_eq!(done.output(), None);
    assert_eq!(done.published_node(), None);
    assert!(done.cancellation_requested());
    drop(lease);
    assert_eq!(dom.snapshot().node(style).unwrap().children(), [texts[0]]);
    assert_eq!(
        dom.snapshot().node(texts[0]).unwrap().data(),
        Some(&"abcdef".into())
    );
    assert_eq!(dom.usage().managed_nodes(), nodes);
    assert!(dom.pending_source_edit().is_none());
    for text in &texts[1..] {
        assert_eq!(dom.snapshot().node(*text).unwrap().parent(), None);
    }
    assert_eq!(before.node(style).unwrap().children(), texts);
    // Even an already single nonempty descendant runs replace-data with empty
    // appended data, so its actual children-changed hook still executes.
    let mut lease = dom
        .begin_source_edit(DomSourceEdit::normalize(style), policy(), &cancel())
        .unwrap();
    assert_ne!(lease.operation(), operation);
    let DomSourceProgress::Request(DomSourceRequest::Style(request)) =
        lease.advance(&cancel()).unwrap()
    else {
        panic!("single Text still has the real append hook")
    };
    assert_eq!(request.source().ordinal(), 0);
    assert_eq!(
        request.source().text().as_ref(),
        &"abcdef".encode_utf16().collect::<Vec<_>>()
    );
    lease
        .accept_inline_csp(&request, &deny(&request), &cancel())
        .unwrap();
    assert!(matches!(
        lease.advance(&cancel()).unwrap(),
        DomSourceProgress::Complete(_)
    ));
}

#[test]
fn normalize_admits_original_plus_concatenated_data_before_removing_any_text() {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let mut batch = DomBatch::new();
    let doc = batch.create_document();
    let fragment = batch.create_fragment(&doc);
    let a = batch.create_text(&doc, "ab");
    let b = batch.create_text(&doc, "cde");
    batch.insert_before(&a, &fragment, None);
    batch.insert_before(&b, &fragment, None);
    let commit = apply(&mut dom, batch);
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let a = commit.created(&a).unwrap().unwrap();
    let b = commit.created(&b).unwrap().unwrap();
    let before = dom.snapshot();
    let usage = dom.usage();
    let units = usage.string_code_units();
    dom.set_limits(limits(units + 2)).unwrap();
    let mut batch = DomBatch::new();
    batch.normalize(fragment);
    assert!(matches!(
        dom.apply(batch, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::StringCodeUnits,
            ..
        })
    ));
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    dom.set_limits(limits(units + 3)).unwrap();
    let mut batch = DomBatch::new();
    batch.normalize(fragment);
    apply(&mut dom, batch);
    assert_eq!(dom.snapshot().node(fragment).unwrap().children(), [a]);
    assert_eq!(
        dom.snapshot().node(a).unwrap().data(),
        Some(&"abcde".into())
    );
    assert_eq!(dom.snapshot().node(b).unwrap().parent(), None);
    assert_eq!(dom.usage().string_code_units(), units + 3);
}

#[test]
fn normalize_foreign_retired_and_precanceled_inputs_preserve_the_actual_tree() {
    let mut dom = Dom::new(limits(65536)).unwrap();
    let mut other = Dom::new(limits(65536)).unwrap();
    let mut batch = DomBatch::new();
    let foreign = batch.create_document();
    let foreign = apply(&mut other, batch).document(&foreign).unwrap().node();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let fragment = batch.create_fragment(&document);
    let a = batch.create_text(&document, "a");
    let b = batch.create_text(&document, "b");
    let retired = batch.create_text(&document, "retired");
    batch.insert_before(&a, &fragment, None);
    batch.insert_before(&b, &fragment, None);
    let commit = apply(&mut dom, batch);
    let fragment = commit.created(&fragment).unwrap().unwrap();
    let retired = commit.created(&retired).unwrap().unwrap();
    let mut batch = DomBatch::new();
    batch.destroy(retired);
    apply(&mut dom, batch);
    let before = dom.snapshot();
    let usage = dom.usage();
    for (node, expected) in [
        (foreign, DomError::ForeignOwner),
        (retired, DomError::StaleNode),
    ] {
        let mut batch = DomBatch::new();
        batch.normalize(node);
        assert_eq!(dom.apply(batch, &cancel()).unwrap_err(), expected);
    }
    let mut batch = DomBatch::new();
    batch.normalize(fragment);
    assert_eq!(
        dom.apply(batch, &AtomicBool::new(true)).unwrap_err(),
        DomError::Cancelled
    );
    assert_eq!(dom.snapshot().stamp(), before.stamp());
    assert_eq!(dom.usage(), usage);
    assert_eq!(
        dom.snapshot().node(fragment).unwrap().children(),
        before.node(fragment).unwrap().children()
    );
}
