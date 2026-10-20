//! Fixed actual container participants; unrelated nodes share the active Document.
use super::*;
use crate::{DomShadowRootInit, HtmlShadowRootMode};

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
struct Fixture {
    inner: history::tests::Fixture,
    top: DomHostedCreation,
    work: Arc<Work>,
    containers: Vec<NodeId>,
    expected: Vec<DomHostedCreation>,
}
fn fixture(unrelated: usize, populated: bool) -> Fixture {
    let mut f = history::tests::Fixture::new();
    f.dom
        .set_limits(
            DomLimits::new(
                4096,
                1_000_000,
                4096,
                0,
                0,
                32,
                128_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let (top, _, preparation_work) = f.issue(0);
    drop(preparation_work);
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(top.document, crate::Namespace::html(), "div");
    batch.insert_before(&root, top.document, None);
    for _ in 0..unrelated {
        let ordinary = batch.create_element_ns(top.document, crate::Namespace::html(), "span");
        batch.insert_before(&ordinary, &root, None);
    }
    let preceding = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
    batch.insert_before(&preceding, &root, None);
    let host = batch.create_element_ns(top.document, crate::Namespace::html(), "div");
    batch.insert_before(&host, &root, None);
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let shadow_child = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
    batch.insert_before(&shadow_child, &shadow, None);
    let light_child = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
    batch.insert_before(&light_child, &host, None);
    let null = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
    batch.insert_before(&null, &root, None);
    let mismatch = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
    batch.insert_before(&mismatch, &root, None);
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let containers = [preceding, shadow_child, light_child, null, mismatch]
        .map(|key| commit.created(&key).unwrap().unwrap())
        .to_vec();
    drop(commit);
    // Issue in a different order from shadow-including DOM order.
    let (light, _, _) = f.issue(1);
    let (in_shadow, _, _) = f.issue(2);
    let (first, _, _) = f.issue(3);
    let (other_parent, _, _) = f.issue(4);
    assert!(light.navigable.slot < in_shadow.navigable.slot);
    assert!(in_shadow.navigable.slot < first.navigable.slot);
    let work = f.source.work(&cancel()).unwrap();
    let context_cancel = cancel();
    let context = history::Context::new(&f.source, &work, &context_cancel, 0);
    let mut graph = history::Graph::initial(None, f.input(top, None), &context).unwrap();
    for (child, container, parent) in [
        (light, containers[2], Some(top)),
        (in_shadow, containers[1], Some(top)),
        (first, containers[0], Some(top)),
        (other_parent, containers[4], None),
    ] {
        let mut input = f.input(child, parent);
        input.parent = parent.map(|parent| (parent.navigable, container));
        graph = history::Graph::initial(Some(&graph), input, &context).unwrap();
    }
    let mut draft = Draft::new(&f.dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    if populated {
        for (container, child) in [
            (containers[0], first),
            (containers[1], in_shadow),
            (containers[2], light),
            (containers[4], other_parent),
        ] {
            draft
                .establish_child_navigable(
                    Key::Existing(container),
                    Some(child.navigable),
                    &cancel(),
                )
                .unwrap();
        }
    }
    let prepared = draft.finish(&f.dom, &cancel()).unwrap();
    f.dom.publish(prepared, &cancel()).unwrap();
    Fixture {
        inner: f,
        top,
        work,
        containers,
        expected: if populated {
            vec![first, in_shadow, light]
        } else {
            vec![]
        },
    }
}

#[test]
fn ordered_container_discovery_ignores_noncontainers_in_the_same_document() {
    fn sample(unrelated: usize, populated: bool) -> u64 {
        let f = fixture(unrelated, populated);
        let old = f.inner.dom.snapshot();
        let graph = f.inner.dom.state.hosted_history.as_ref().unwrap();
        let before = f.work.spent.load(Ordering::Relaxed);
        let captured = f
            .inner
            .dom
            .capture_hosted_children(&f.inner.source, f.top, &f.work, 0, &cancel())
            .unwrap();
        let steps = f.work.spent.load(Ordering::Relaxed) - before;
        assert_eq!(captured.values, f.expected);
        assert_eq!(
            old.node(f.containers[3])
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
        if populated {
            let mismatch = old
                .node(f.containers[4])
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable()
                .unwrap()
                .unwrap();
            assert_eq!(graph.navigable(mismatch).unwrap().parent, None);
            assert!(
                !captured
                    .values
                    .iter()
                    .any(|child| child.navigable == mismatch)
            );
        }
        assert_eq!(old.stamp(), f.inner.dom.snapshot().stamp());
        steps
    }
    for populated in [false, true] {
        let small = sample(0, populated);
        let large = sample(256, populated);
        assert!(
            large <= small + 96,
            "fixed five actual containers (populated={populated}) across 256 noncontainers: small={small}, large={large}"
        );
    }
}

#[test]
fn container_order_is_shadow_before_light_and_historical_views_keep_old_edges() {
    let mut f = fixture(0, true);
    let old = f.inner.dom.snapshot();
    let mut before = Vec::new();
    f.inner
        .dom
        .visit_hosted_children(
            f.inner.dom.state.hosted_history.as_ref().unwrap(),
            f.top.navigable,
            &f.work,
            &cancel(),
            |id| {
                before.push(id);
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(
        before,
        f.expected
            .iter()
            .map(|child| child.navigable)
            .collect::<Vec<_>>()
    );
    let mut draft = Draft::new(&f.inner.dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .establish_child_navigable(Key::Existing(f.containers[1]), None, &cancel())
        .unwrap();
    // The actual removal publication couples the cleared live container edge
    // with Graph retiring/unlinking; it is not a standalone field-only edit.
    let cancelled = cancel();
    let context = history::Context::new(&f.inner.source, &f.work, &cancelled, 0);
    let retiring = draft
        .hosted_history
        .as_ref()
        .unwrap()
        .mark_retiring(f.expected[1].navigable, &context)
        .unwrap();
    let graph = retiring
        .unlink_child(f.expected[1].navigable, f.top.navigable, &context)
        .unwrap();
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    let prepared = draft.finish(&f.inner.dom, &cancel()).unwrap();
    f.inner.dom.publish(prepared, &cancel()).unwrap();
    assert_eq!(
        old.node(f.containers[1])
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(Some(f.expected[1].navigable))
    );
    assert_eq!(
        old.state
            .hosted_history
            .as_ref()
            .unwrap()
            .navigable(f.expected[1].navigable)
            .unwrap()
            .parent,
        Some(f.top.navigable)
    );
    let current_graph = f.inner.dom.state.hosted_history.as_ref().unwrap();
    let removed = current_graph.navigable(f.expected[1].navigable).unwrap();
    assert!(removed.retiring);
    assert_eq!(removed.parent, None);
    assert_eq!(removed.former_parent, Some(f.top.navigable));
    let current = f
        .inner
        .dom
        .capture_hosted_children(&f.inner.source, f.top, &f.work, 0, &cancel())
        .unwrap();
    assert_eq!(current.values, vec![f.expected[0], f.expected[2]]);
    assert_eq!(
        f.inner
            .dom
            .snapshot()
            .node(f.containers[1])
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

fn empty_output_steps(unrelated: usize, containers: usize) -> u64 {
    let mut f = history::tests::Fixture::new();
    f.dom
        .set_limits(
            DomLimits::new(
                4096,
                1_000_000,
                4096,
                0,
                0,
                32,
                128_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let (top, _, preparation_work) = f.issue(0);
    drop(preparation_work);
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(top.document, crate::Namespace::html(), "div");
    batch.insert_before(&root, top.document, None);
    for _ in 0..unrelated {
        let node = batch.create_element_ns(top.document, crate::Namespace::html(), "span");
        batch.insert_before(&node, &root, None);
    }
    let mut nulls = Vec::new();
    for _ in 0..containers {
        let node = batch.create_element_ns(top.document, crate::Namespace::html(), "iframe");
        batch.insert_before(&node, &root, None);
        nulls.push(node);
    }
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let nulls = nulls
        .into_iter()
        .map(|node| commit.created(&node).unwrap().unwrap())
        .collect::<Vec<_>>();
    drop(commit);
    let work = f.source.work(&cancel()).unwrap();
    let cancelled = cancel();
    let context = history::Context::new(&f.source, &work, &cancelled, 0);
    let graph = history::Graph::initial(None, f.input(top, None), &context).unwrap();
    let mut draft = Draft::new(&f.dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    let prepared = draft.finish(&f.dom, &cancel()).unwrap();
    f.dom.publish(prepared, &cancel()).unwrap();
    let before = work.spent.load(Ordering::Relaxed);
    let captured = f
        .dom
        .capture_hosted_children(&f.source, top, &work, 0, &cancel())
        .unwrap();
    let steps = work.spent.load(Ordering::Relaxed) - before;
    assert!(captured.values.is_empty());
    for node in nulls {
        assert_eq!(
            f.dom
                .snapshot()
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }
    steps
}

#[test]
fn absent_container_discovery_ignores_noncontainers_in_the_same_document() {
    let small = empty_output_steps(0, 0);
    let large = empty_output_steps(256, 0);
    assert!(
        large <= small + 96,
        "zero actual containers: small={small}, large={large}"
    );
}

#[test]
fn one_null_container_keeps_its_semantic_check_without_visiting_noncontainers() {
    let small = empty_output_steps(0, 1);
    let large = empty_output_steps(256, 1);
    assert!(
        large <= small + 96,
        "one actual null container: small={small}, large={large}"
    );
}
