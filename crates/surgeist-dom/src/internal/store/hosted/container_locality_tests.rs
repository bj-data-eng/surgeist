//! Fixed actual container participants; unrelated nodes share the active Document.
use super::*;
use crate::internal::shadow::DomShadowRootInit;
use surgeist_html::construction::HtmlShadowRootMode;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
struct Fixture {
    inner: history::tests::Fixture,
    top: DomHostedCreation,
    root: NodeId,
    nested: DomHostedCreation,
    light: DomHostedCreation,
    first: DomHostedCreation,
    first_operation: DomHostedOperationId,
    work: Arc<Work>,
    containers: Vec<NodeId>,
    expected: Vec<DomHostedCreation>,
}
fn fixture(unrelated: usize, populated: bool) -> Fixture {
    build_fixture(unrelated, populated, false)
}

fn build_fixture(unrelated: usize, populated: bool, document_tree: bool) -> Fixture {
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
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let (top, _, preparation_work) = f.issue(0);
    drop(preparation_work);
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "div",
    );
    batch.insert_before(&root, top.document, None);
    for _ in 0..unrelated {
        let ordinary = batch.create_element_ns(
            top.document,
            surgeist_html::values::Namespace::html(),
            "span",
        );
        batch.insert_before(&ordinary, &root, None);
    }
    let preceding = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "frame",
    );
    batch.insert_before(&preceding, &root, None);
    let host = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "div",
    );
    batch.insert_before(&host, &root, None);
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Closed));
    let shadow_child = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "iframe",
    );
    batch.insert_before(&shadow_child, &shadow, None);
    let light_child = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "iframe",
    );
    batch.insert_before(&light_child, &host, None);
    let null = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "iframe",
    );
    batch.insert_before(&null, &root, None);
    let mismatch = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "iframe",
    );
    batch.insert_before(&mismatch, &root, None);
    let template = document_tree.then(|| {
        let template = batch.create_element_ns(
            top.document,
            surgeist_html::values::Namespace::html(),
            "template",
        );
        batch.insert_before(&template, &root, None);
        template
    });
    let commit = f.dom.apply(batch, &cancel()).unwrap();
    let containers = [preceding, shadow_child, light_child, null, mismatch]
        .map(|key| commit.created(&key).unwrap().unwrap())
        .to_vec();
    let root = commit.created(&root).unwrap().unwrap();
    let template = template.map(|key| commit.created(&key).unwrap().unwrap());
    drop(commit);
    if let Some(template) = template {
        let snapshot = f.dom.snapshot();
        let contents = snapshot
            .node(template)
            .unwrap()
            .as_template()
            .unwrap()
            .contents()
            .unwrap();
        let document = snapshot.node(contents).unwrap().owner_document().unwrap();
        let mut batch = DomBatch::new();
        let container =
            batch.create_element_ns(document, surgeist_html::values::Namespace::html(), "iframe");
        batch.insert_before(&container, contents, None);
        f.dom.apply(batch, &cancel()).unwrap();
    }
    // Issue in a different order from shadow-including DOM order.
    let (light, _, _) = f.issue(1);
    let (in_shadow, _, _) = f.issue(2);
    let (first, first_operation, _) = f.issue(3);
    let (other_parent, _, _) = f.issue(4);
    let nested_container = if document_tree {
        let mut batch = DomBatch::new();
        let container = batch.create_element_ns(
            first.document,
            surgeist_html::values::Namespace::html(),
            "iframe",
        );
        batch.insert_before(&container, first.document, None);
        f.dom
            .apply(batch, &cancel())
            .unwrap()
            .created(&container)
            .unwrap()
            .unwrap()
    } else {
        containers[4]
    };
    assert!(light.navigable.slot < in_shadow.navigable.slot);
    assert!(in_shadow.navigable.slot < first.navigable.slot);
    let work = f.source.work(&cancel()).unwrap();
    let context_cancel = cancel();
    let context = history::Context::new(&f.source, &work, &context_cancel, 0);
    let mut graph = history::Graph::initial(None, f.input(top, None), &context).unwrap();
    let children = [
        (light, containers[2], Some(top)),
        (in_shadow, containers[1], Some(top)),
        (first, containers[0], Some(top)),
        (
            other_parent,
            nested_container,
            document_tree.then_some(first),
        ),
    ];
    let mut draft = Draft::new(&f.dom, DomBatch::new().owner, &cancel()).unwrap();
    if document_tree {
        draft
            .set_hosted_history(Some(graph.clone()), None, &cancel())
            .unwrap();
        initial_association(&mut draft, &f.source, top);
    }
    // This graph-locality fixture establishes its issuer rows directly. Qualify
    // the pre-content participants in the same canonical structural order.
    if document_tree {
        draft
            .record_hosted_child_insertions(Key::Existing(root), &cancel())
            .unwrap();
    }
    if populated || !document_tree {
        for (child, container, parent) in children {
            if document_tree {
                initial_association(&mut draft, &f.source, child);
                if let Some(parent) = parent
                    && parent.document != top.document
                {
                    draft
                        .record_hosted_child_insertions(Key::Existing(container), &cancel())
                        .unwrap();
                }
            }
            let mut input = f.input(child, parent);
            input.parent = parent.map(|parent| (parent.navigable, container));
            if populated {
                let insertion = fixture_insertion(
                    &mut draft,
                    parent.map_or(top.document, |p| p.document),
                    container,
                );
                input.insertion = parent.map(|_| insertion);
            }
            graph = history::Graph::initial(Some(&graph), input, &context).unwrap();
        }
    }
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    if populated {
        for (container, child) in [
            (containers[0], first),
            (containers[1], in_shadow),
            (containers[2], light),
            (nested_container, other_parent),
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
        root,
        nested: other_parent,
        light,
        first,
        first_operation,
        work,
        containers,
        expected: if populated {
            vec![first, in_shadow, light]
        } else {
            vec![]
        },
    }
}

fn fixture_insertion(
    draft: &mut Draft,
    document: DocumentId,
    container: NodeId,
) -> DomHostedChildInsertion {
    let key = Key::Existing(container);
    if let Some(insertion) = draft
        .node(key)
        .unwrap()
        .element()
        .unwrap()
        .language()
        .hosted_insertion()
    {
        return insertion;
    }
    let insertion = DomHostedChildInsertion {
        document,
        group: draft.next_hosted_insertion,
        preorder: 0,
    };
    draft.next_hosted_insertion += 1;
    draft
        .replace_hosted_content(
            key,
            Some(crate::internal::element::HostedContent {
                insertion,
                navigable: None,
            }),
            &cancel(),
        )
        .unwrap();
    insertion
}

fn initial_association(draft: &mut Draft, source: &DomHostedSource, creation: DomHostedCreation) {
    let moment = DomHostedMoment {
        source: source.id(),
        microseconds: 1_000,
    };
    associate(
        draft,
        source,
        creation,
        DocumentPhase::InitialEntry {
            entry: creation.entry,
            navigable: creation.navigable,
            load: HostedDocumentLoad::Loaded(moment),
        },
    );
}

fn associate(
    draft: &mut Draft,
    source: &DomHostedSource,
    creation: DomHostedCreation,
    phase: DocumentPhase,
) {
    let mut metadata = draft
        .document_metadata_mut(Key::Existing(creation.document.node()), &cancel())
        .unwrap();
    metadata.set_hosted_association(Arc::new(DocumentAssociation {
        source: source.observe(),
        host: creation.host,
        context: creation.context,
        global: creation.global,
        navigation_start: DomHostedMoment {
            source: source.id(),
            microseconds: 1_000,
        },
        policy: DomInitialPermissionsPolicy::new(
            DomPermissionsPolicyValue::Enabled,
            DomPermissionsPolicyValue::Enabled,
        ),
        phase,
        _retained: source
            .retain(size_of::<DocumentAssociation>(), &cancel())
            .unwrap(),
    }));
    metadata.commit(&cancel()).unwrap();
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
        // Metered ordering reads the deeper node and position banks. Allow
        // 320 page reads per extra level across the two passes over five
        // containers. The larger fixture adds two bank levels and enough
        // ordinary nodes that even one step per noncontainer fails the bound.
        let large = sample(1024, populated);
        assert!(
            large <= small + 2 * 320,
            "fixed five actual containers (populated={populated}) across 1024 noncontainers: small={small}, large={large}"
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
                crate::internal::name_limits::DomNameLimits::new(
                    16_777_216, 16_384, 4_000_000, 1_048_576,
                )
                .unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
    let (top, _, preparation_work) = f.issue(0);
    drop(preparation_work);
    let mut batch = DomBatch::new();
    let root = batch.create_element_ns(
        top.document,
        surgeist_html::values::Namespace::html(),
        "div",
    );
    batch.insert_before(&root, top.document, None);
    for _ in 0..unrelated {
        let node = batch.create_element_ns(
            top.document,
            surgeist_html::values::Namespace::html(),
            "span",
        );
        batch.insert_before(&node, &root, None);
    }
    let mut nulls = Vec::new();
    for _ in 0..containers {
        let node = batch.create_element_ns(
            top.document,
            surgeist_html::values::Namespace::html(),
            "iframe",
        );
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

type ChildIdentities = (DomHostedNavigableId, DomHostedBrowsingContextId);

fn current_children(dom: &Dom, document: DocumentId) -> Vec<ChildIdentities> {
    let mut found = Vec::new();
    let count = dom
        .visit_document_tree_child_navigables(
            document,
            10_000,
            || Ok::<_, DomHostedError>(()),
            &cancel(),
            |child| {
                found.push((child.id(), child.browsing_context()));
                Ok(())
            },
        )
        .unwrap();
    assert_eq!(count, found.len());
    found
}

fn identities(creation: DomHostedCreation) -> ChildIdentities {
    (creation.navigable(), creation.browsing_context())
}

#[test]
fn document_tree_children_use_ordinary_container_order_and_the_selected_document() {
    let f = build_fixture(20, true, true);
    let before = f.inner.dom.snapshot().stamp();
    assert_eq!(
        current_children(&f.inner.dom, f.top.document),
        [identities(f.expected[0]), identities(f.expected[2])]
    );
    assert_eq!(
        current_children(&f.inner.dom, f.expected[0].document),
        [identities(f.nested)]
    );
    assert_eq!(f.inner.dom.snapshot().stamp(), before);
}

#[test]
fn document_tree_children_follow_current_publication_reorder_and_retirement() {
    let mut f = build_fixture(0, false, true);
    assert!(current_children(&f.inner.dom, f.top.document).is_empty());

    let mut draft = Draft::new(&f.inner.dom, DomBatch::new().owner, &cancel()).unwrap();
    let cancelled = cancel();
    let context = history::Context::new(&f.inner.source, &f.work, &cancelled, 0);
    let mut graph = draft.hosted_history.clone().unwrap();
    for (container, child) in [(f.containers[2], f.light), (f.containers[0], f.first)] {
        let mut input = f.inner.input(child, Some(f.top));
        input.parent = Some((f.top.navigable, container));
        input.insertion = Some(fixture_insertion(&mut draft, f.top.document, container));
        graph = history::Graph::initial(Some(&graph), input, &context).unwrap();
        initial_association(&mut draft, &f.inner.source, child);
        draft
            .establish_child_navigable(Key::Existing(container), Some(child.navigable), &cancel())
            .unwrap();
    }
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    let prepared = draft.finish(&f.inner.dom, &cancel()).unwrap();
    f.inner.dom.publish(prepared, &cancel()).unwrap();
    assert_eq!(
        current_children(&f.inner.dom, f.top.document),
        [identities(f.first), identities(f.light)]
    );

    let mut edit = f
        .inner
        .dom
        .begin_source_edit(
            crate::internal::source::DomSourceEdit::move_before(
                f.containers[2],
                f.root,
                Some(f.containers[0]),
            ),
            crate::internal::source::DomSourceLimits::new(
                64_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 8,
            ),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        edit.advance(&cancel()).unwrap(),
        crate::internal::source::DomSourceProgress::Complete(_),
    ));
    drop(edit);
    assert_eq!(
        current_children(&f.inner.dom, f.top.document),
        [identities(f.light), identities(f.first)]
    );

    let mut draft = Draft::new(&f.inner.dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .establish_child_navigable(Key::Existing(f.containers[0]), None, &cancel())
        .unwrap();
    let cancelled = cancel();
    let context = history::Context::new(&f.inner.source, &f.work, &cancelled, 0);
    let graph = draft.hosted_history.as_ref().unwrap();
    let retiring = graph.mark_retiring(f.first.navigable, &context).unwrap();
    let graph = retiring
        .unlink_child(f.first.navigable, f.top.navigable, &context)
        .unwrap();
    let graph = graph.clear_document(f.first.document, &context).unwrap();
    associate(
        &mut draft,
        &f.inner.source,
        f.first,
        DocumentPhase::Retired {
            operation: f.first_operation,
        },
    );
    draft
        .set_hosted_history(Some(graph), None, &cancel())
        .unwrap();
    let prepared = draft.finish(&f.inner.dom, &cancel()).unwrap();
    f.inner.dom.publish(prepared, &cancel()).unwrap();
    assert_eq!(
        current_children(&f.inner.dom, f.top.document),
        [identities(f.light)]
    );
    assert!(current_children(&f.inner.dom, f.first.document).is_empty());
    let mut edit = DomBatch::new();
    edit.detach(f.containers[0]);
    f.inner.dom.apply(edit, &cancel()).unwrap();
    let snapshot = f.inner.dom.snapshot();
    assert_eq!(
        snapshot.node(f.containers[0]).unwrap().root(),
        f.containers[0]
    );
    assert_eq!(
        current_children(&f.inner.dom, f.top.document),
        [identities(f.light)]
    );
}

#[test]
fn document_tree_child_reads_distinguish_absence_from_invalid_authority() {
    let mut f = build_fixture(0, true, true);
    let mut batch = DomBatch::new();
    let plain = batch.create_document();
    let plain = f
        .inner
        .dom
        .apply(batch, &cancel())
        .unwrap()
        .document(&plain)
        .unwrap();
    let read = |dom: &Dom, document| {
        dom.visit_document_tree_child_navigables(
            document,
            10_000,
            || Ok::<_, DomHostedError>(()),
            &cancel(),
            |_| Ok(()),
        )
    };
    assert_eq!(read(&f.inner.dom, plain), Ok(0));
    let other = fixture(0, false);
    assert_eq!(
        read(&f.inner.dom, other.top.document),
        Err(DomError::ForeignOwner.into())
    );

    let stale = DocumentId(NodeId {
        generation: plain.node().generation + 1,
        ..plain.node()
    });
    assert_eq!(read(&f.inner.dom, stale), Err(DomError::StaleNode.into()));

    drop(f.inner.source);
    assert_eq!(
        read(&f.inner.dom, f.top.document),
        Err(DomHostedError::SourceExpired)
    );
}

#[derive(Debug, Eq, PartialEq)]
enum ReadRefusal {
    Hosted(DomHostedError),
    Admission,
    Visitor,
}
impl From<DomHostedError> for ReadRefusal {
    fn from(value: DomHostedError) -> Self {
        Self::Hosted(value)
    }
}

#[test]
fn document_tree_child_reads_meter_participants_and_stop_at_the_first_refusal() {
    let measure = |unrelated| {
        let f = build_fixture(unrelated, true, true);
        let mut work = 0;
        let count = f
            .inner
            .dom
            .visit_document_tree_child_navigables(
                f.top.document,
                10_000,
                || {
                    work += 1;
                    Ok::<_, DomHostedError>(())
                },
                &cancel(),
                |_| Ok(()),
            )
            .unwrap();
        assert_eq!(count, 2);
        assert_eq!(
            f.inner.dom.visit_document_tree_child_navigables(
                f.top.document,
                work - 1,
                || Ok::<_, DomHostedError>(()),
                &cancel(),
                |_| Ok(()),
            ),
            Err(DomHostedError::WorkLimit { limit: work - 1 })
        );
        work
    };
    let small = measure(0);
    let large = measure(256);
    assert!(large <= small + 32, "small={small}, large={large}");

    let f = build_fixture(0, true, true);
    let mut visited = Vec::new();
    assert_eq!(
        f.inner.dom.visit_document_tree_child_navigables(
            f.top.document,
            10_000,
            || Ok(()),
            &cancel(),
            |child| {
                visited.push(child.id());
                Err(ReadRefusal::Visitor)
            },
        ),
        Err(ReadRefusal::Visitor)
    );
    assert_eq!(visited, [f.first.navigable]);

    let cancelled = cancel();
    visited.clear();
    assert_eq!(
        f.inner.dom.visit_document_tree_child_navigables(
            f.top.document,
            10_000,
            || Ok(()),
            &cancelled,
            |child| {
                visited.push(child.id());
                cancelled.store(true, Ordering::Relaxed);
                Ok::<_, DomHostedError>(())
            },
        ),
        Err(DomError::Cancelled.into())
    );
    assert_eq!(visited, [f.first.navigable]);

    visited.clear();
    assert_eq!(
        f.inner.dom.visit_document_tree_child_navigables(
            f.top.document,
            10_000,
            || Err(ReadRefusal::Admission),
            &cancel(),
            |child| {
                visited.push(child.id());
                Ok(())
            },
        ),
        Err(ReadRefusal::Admission)
    );
    assert!(visited.is_empty());
}

#[test]
fn document_tree_child_read_refuses_an_inconsistent_insertion_projection() {
    let mut f = build_fixture(0, true, true);
    let container = f.containers[0];
    let before = f.inner.dom.snapshot();
    let child = before.hosted_navigable(f.first.navigable).unwrap();
    let original = child.container_insertion().unwrap();
    let mut draft = Draft::new(&f.inner.dom, DomBatch::new().owner, &cancel()).unwrap();
    draft
        .replace_hosted_content(
            Key::Existing(container),
            Some(crate::internal::element::HostedContent {
                insertion: DomHostedChildInsertion {
                    group: original.group() + 1,
                    ..original
                },
                navigable: Some(child.id()),
            }),
            &cancel(),
        )
        .unwrap();
    let prepared = draft.finish(&f.inner.dom, &cancel()).unwrap();
    f.inner.dom.publish(prepared, &cancel()).unwrap();
    let changes = f.inner.dom.changes_since(before.stamp()).unwrap();
    assert!(
        changes
            .changes()
            .contains(&DomChange::ElementState(container))
    );
    let mut visited = false;
    assert_eq!(
        f.inner.dom.visit_document_tree_child_navigables(
            f.top.document,
            10_000,
            || Ok::<_, DomHostedError>(()),
            &cancel(),
            |_| {
                visited = true;
                Ok(())
            }
        ),
        Err(DomHostedError::StaleBasis)
    );
    assert!(!visited);
}
