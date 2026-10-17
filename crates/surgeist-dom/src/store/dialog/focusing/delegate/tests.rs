//! Controlled called-algorithm fixtures over real canonical nodes. These tests
//! do not manufacture a Window, focusability fact, receipt or completed method.
//! HTML877d60113–60159 supplies both pass order and ordinary-descendant order.
use super::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn limits(bytes: usize, work: u64) -> crate::DomSourceLimits {
    crate::DomSourceLimits::new(bytes, 1_000_000, work, 64, 4096, 4096, 2)
}
fn dom() -> Dom {
    Dom::new(DomLimits::new(512, 1_000_000, 1024, 128, 128, 32, 4_000_000).unwrap()).unwrap()
}
fn draft(dom: &Dom) -> Draft {
    let batch = DomBatch::new();
    Draft::new_source(
        dom,
        batch.owner.clone(),
        &batch.commands,
        limits(64_000_000, 100_000_000),
        &cancel(),
    )
    .unwrap()
    .0
}
fn fixture(autofocus: bool) -> (Dom, NodeId, Vec<NodeId>, NodeId) {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let document = b.create_document();
    let root = b.create_element_ns(&document, crate::Namespace::html(), "dialog");
    let text = b.create_text(&document, "ordinary text descendant");
    let svg = b.create_element_ns(
        &document,
        crate::Namespace::from_uri("http://www.w3.org/2000/svg"),
        "g",
    );
    let host = b.create_element_ns(&document, crate::Namespace::html(), "div");
    let light = b.create_comment(&document, "ordinary light descendant");
    let tail = b.create_element_ns(&document, crate::Namespace::html(), "span");
    let shadow = b.attach_shadow(
        &host,
        crate::DomShadowRootInit::new(surgeist_html::HtmlShadowRootMode::Closed),
    );
    let hidden = b.create_element_ns(&document, crate::Namespace::html(), "button");
    b.set_attribute(&hidden, "autofocus", "");
    if autofocus {
        b.set_attribute(&svg, "autofocus", "false");
    } else {
        b.set_attribute_ns(
            &svg,
            crate::Namespace::from_uri("urn:test"),
            "q:autofocus",
            "",
        );
    }
    for child in [&text, &svg, &host, &tail] {
        b.insert_before(child, &root, None);
    }
    b.insert_before(&light, &host, None);
    b.insert_before(&hidden, &shadow, None);
    let c = dom.apply(b, &cancel()).unwrap();
    let root = c.created(&root).unwrap().unwrap();
    let descendants = [&text, &svg, &host, &light, &tail]
        .map(|ticket| c.created(ticket).unwrap().unwrap())
        .to_vec();
    (dom, root, descendants, c.created(&hidden).unwrap().unwrap())
}
fn pending(call: &DelegateFrame) -> (Key, Key, crate::DomSourceFocusDelegateStep) {
    let p = call.pending().expect("actual descendant query");
    (p.root, p.queried_node, p.step)
}

#[test]
fn ordinary_preorder_excludes_shadow_edges_and_restarts_a_distinct_second_pass() {
    let (dom, root, expected, hidden) = fixture(false);
    let d = draft(&dom);
    let root = d.resolve(root.into()).unwrap();
    let mut walk = Walk::new(&d);
    for _ in 0..2 {
        walk.begin(&d, root, &cancel()).unwrap();
        let mut actual = Vec::new();
        while let Some(key) = walk.next(&d, &cancel()).unwrap() {
            actual.push(d.published_id(key).unwrap());
        }
        assert_eq!(actual, expected);
        assert!(!actual.contains(&hidden));
        assert_eq!(walk.next(&d, &cancel()).unwrap(), None);
    }
    drop(walk);
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
}

#[test]
fn empty_dialog_completes_both_called_producers_without_a_focusability_reply() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let root = b.create_element_ns(&doc, crate::Namespace::html(), "dialog");
    let c = dom.apply(b, &cancel()).unwrap();
    let d = draft(&dom);
    let root = d
        .resolve(c.created(&root).unwrap().unwrap().into())
        .unwrap();
    let mut call = DelegateFrame::new(&d, root);
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), Some(false));
    assert_eq!(
        call.cursor.action(),
        FA::Complete(HtmlFocusDelegateOutcome::Null)
    );
    assert!(call.autofocus.is_none());
    assert!(call.walk.current.is_none());
    assert!(call.walk.levels.is_empty());
    // Both real BeginDescendants calls reached the admitted root bank.
    assert!(call.walk.levels.capacity() >= 1);
    drop(call);
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert!(d.dialog_delegate_null_available(root, &cancel()).unwrap());
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(
        d.dialog_delegate_null_available(root, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
}

#[test]
fn autofocus_pass_uses_actual_svg_presence_before_any_dialog_sequential_query() {
    let (dom, root, descendants, _) = fixture(true);
    let d = draft(&dom);
    let root = d.resolve(root.into()).unwrap();
    let svg = d.resolve(descendants[1].into()).unwrap();
    let mut call = DelegateFrame::new(&d, root);
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), None);
    assert_eq!(
        pending(&call),
        (
            root,
            svg,
            crate::DomSourceFocusDelegateStep::Autofocus(AA::ReadFocusableArea)
        )
    );
    let original = pending(&call);
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), None);
    assert_eq!(pending(&call), original);
}

#[test]
fn fallback_retains_first_non_element_query_after_full_autofocus_pass_exhaustion() {
    let (dom, root, descendants, _) = fixture(false);
    let d = draft(&dom);
    let root = d.resolve(root.into()).unwrap();
    let text = d.resolve(descendants[0].into()).unwrap();
    assert!(!d.dialog_delegate_null_available(root, &cancel()).unwrap());
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    let mut call = DelegateFrame::new(&d, root);
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), None);
    assert_eq!(
        pending(&call),
        (
            root,
            text,
            crate::DomSourceFocusDelegateStep::Focus(FA::ReadSequentiallyFocusable)
        )
    );
    assert!(call.autofocus.is_none());
    let original = pending(&call);
    assert_eq!(
        call.advance(&d, root, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    );
    assert_eq!(pending(&call), original);
    let spent = d.source_work.load(Ordering::Relaxed);
    let mut d = d;
    d.source_limits = Some(limits(64_000_000, spent));
    assert_eq!(
        call.advance(&d, root, &cancel()),
        Err(DomError::SourceWorkLimit { limit: spent })
    );
    assert_eq!(pending(&call), original);
    d.source_limits = Some(limits(64_000_000, 100_000_000));
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), None);
    assert_eq!(pending(&call), original);
}

#[test]
fn admitted_wide_walk_is_linear_without_a_per_sibling_payload_rescan() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let root = b.create_element_ns(&doc, crate::Namespace::html(), "dialog");
    let mut tickets = Vec::new();
    for _ in 0..96 {
        let child = b.create_comment(&doc, "leaf");
        b.insert_before(&child, &root, None);
        tickets.push(child);
    }
    let c = dom.apply(b, &cancel()).unwrap();
    let expected: Vec<_> = tickets
        .iter()
        .map(|t| c.created(t).unwrap().unwrap())
        .collect();
    let d = draft(&dom);
    let root = d
        .resolve(c.created(&root).unwrap().unwrap().into())
        .unwrap();
    let mut walk = Walk::new(&d);
    walk.begin(&d, root, &cancel()).unwrap();
    while walk.next(&d, &cancel()).unwrap().is_some() {}
    assert!(
        walk.levels.capacity() <= 4,
        "capacity follows depth, not width"
    );
    let baseline = d.source_work.load(Ordering::Relaxed);
    walk.begin(&d, root, &cancel()).unwrap();
    let mut actual = Vec::new();
    while let Some(node) = walk.next(&d, &cancel()).unwrap() {
        actual.push(d.published_id(node).unwrap());
    }
    assert_eq!(actual, expected);
    // Once capacity is admitted, each indexed edge/pop/read has a constant
    // bound; this is not an oracle for the separate global admission tariff.
    assert!(
        d.source_work.load(Ordering::Relaxed) - baseline
            <= 8 * u64::try_from(expected.len()).unwrap() + 8
    );
}

#[test]
fn deep_walk_keeps_only_indexed_ancestors_and_releases_actual_capacity() {
    let mut dom = dom();
    let mut b = DomBatch::new();
    let doc = b.create_document();
    let root = b.create_element_ns(&doc, crate::Namespace::html(), "dialog");
    let mut parent = root.clone();
    let mut tickets = Vec::new();
    for _ in 0..48 {
        let child = b.create_element_ns(&doc, crate::Namespace::html(), "div");
        b.insert_before(&child, &parent, None);
        parent = child.clone();
        tickets.push(child);
    }
    let c = dom.apply(b, &cancel()).unwrap();
    let d = draft(&dom);
    let root = d
        .resolve(c.created(&root).unwrap().unwrap().into())
        .unwrap();
    let mut walk = Walk::new(&d);
    walk.begin(&d, root, &cancel()).unwrap();
    let mut actual = Vec::new();
    while let Some(node) = walk.next(&d, &cancel()).unwrap() {
        actual.push(d.published_id(node).unwrap());
    }
    assert_eq!(
        actual,
        tickets
            .iter()
            .map(|t| c.created(t).unwrap().unwrap())
            .collect::<Vec<_>>()
    );
    assert!(walk.levels.capacity() < 2 * (tickets.len() + 2));
    assert!(d.select_temporary_bytes.load(Ordering::Relaxed) > 0);
    drop(walk);
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
}

#[test]
fn query_checkpoint_refusal_and_cancellation_preserve_original_bank_and_called_cursor() {
    let (dom, root, _, _) = fixture(true);
    let mut d = draft(&dom);
    let root = d.resolve(root.into()).unwrap();
    let mut call = DelegateFrame::new(&d, root);
    assert_eq!(call.advance(&d, root, &cancel()).unwrap(), None);
    let original = pending(&call);
    let original_bytes = d.select_temporary_bytes.load(Ordering::Relaxed);
    let live = d.source_storage_bytes(&cancel()).unwrap();
    let minimum_copy = call.walk.levels.len() * size_of::<Level>();
    assert!(minimum_copy > 0);
    let bound = live + minimum_copy - 1;
    d.source_limits = Some(limits(bound, 100_000_000));
    let mut admission = source::checkpoint::Admission::new(&d, live).unwrap();
    assert!(
        matches!(call.checkpoint(&mut admission, &cancel()), Err(DomError::ResourceLimit { resource: DomResource::SourceRetainedBytes, limit }) if limit == bound)
    );
    drop(admission);
    assert_eq!(pending(&call), original);
    assert_eq!(
        d.select_temporary_bytes.load(Ordering::Relaxed),
        original_bytes
    );
    d.source_limits = Some(limits(64_000_000, 100_000_000));
    let mut admission = source::checkpoint::Admission::new(&d, live).unwrap();
    assert!(matches!(
        call.checkpoint(&mut admission, &AtomicBool::new(true)),
        Err(DomError::Cancelled)
    ));
    drop(admission);
    assert_eq!(pending(&call), original);
    assert_eq!(
        d.select_temporary_bytes.load(Ordering::Relaxed),
        original_bytes
    );
    let mut admission = source::checkpoint::Admission::new(&d, live).unwrap();
    let copy = call.checkpoint(&mut admission, &cancel()).unwrap();
    let charge = admission.finish();
    assert_eq!(pending(&copy), original);
    assert!(d.select_temporary_bytes.load(Ordering::Relaxed) > original_bytes);
    drop(copy);
    drop(charge);
    assert_eq!(
        d.select_temporary_bytes.load(Ordering::Relaxed),
        original_bytes
    );
    drop(call);
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
}

#[test]
fn reached_stack_allocation_refuses_one_short_without_spending_capacity() {
    let (dom, root, _, _) = fixture(false);
    let mut d = draft(&dom);
    let root = d.resolve(root.into()).unwrap();
    let mut walk = Walk::new(&d);
    let live = d.source_storage_bytes(&cancel()).unwrap() + d.source_other_bytes;
    let bound = live + size_of::<Level>() - 1;
    d.source_limits = Some(limits(bound, 100_000_000));
    assert_eq!(
        walk.begin(&d, root, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: bound
        })
    );
    assert!(walk.levels.is_empty());
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    assert_eq!(
        d.dialog_delegate_null_available(root, &cancel()),
        Err(DomError::ResourceLimit {
            resource: DomResource::SourceRetainedBytes,
            limit: bound
        })
    );
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    let spent = d.source_work.load(Ordering::Relaxed);
    d.source_limits = Some(limits(64_000_000, spent));
    assert_eq!(
        d.dialog_delegate_null_available(root, &cancel()),
        Err(DomError::SourceWorkLimit { limit: spent })
    );
    assert_eq!(d.select_temporary_bytes.load(Ordering::Relaxed), 0);
    d.source_limits = Some(limits(64_000_000, 100_000_000));
    walk.begin(&d, root, &cancel()).unwrap();
    assert!(walk.next(&d, &cancel()).unwrap().is_some());
}
