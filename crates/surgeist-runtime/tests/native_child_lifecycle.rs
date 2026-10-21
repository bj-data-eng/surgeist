//! HTML877d Frame/Iframe post-connection creates a real child navigable before
//! the shared absent-src initial-about:blank ancestor guard returns null.
//! Source completion covers the insertion's mandatory hooks, not queued history.
#![cfg(any(
    target_os = "macos",
    target_os = "linux",
    target_os = "windows",
    target_os = "android",
    target_os = "ios"
))]

use std::sync::atomic::AtomicBool;
use surgeist_css::CssUtf16InputLimits;
use surgeist_dom::*;
use surgeist_runtime::*;

fn cancel() -> AtomicBool {
    AtomicBool::new(false)
}
fn session() -> DomSession {
    let dom = Dom::new(
        DomLimits::new(
            256,
            200_000,
            1024,
            1024,
            1024,
            32,
            4_000_000,
            surgeist_dom::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
        )
        .unwrap()
        .with_dispatch_custody(
            surgeist_dom::DomDispatchCustodyLimits::new(
                64, 4096, 16_000_000, 16_000_000, 4_000_000,
            )
            .unwrap(),
        )
        .with_details_toggle_intentions(32)
        .unwrap(),
    )
    .unwrap();
    DomSession::new(
        dom,
        DomSessionLimits {
            source_events: surgeist_runtime::DomSourceEventRuntimeLimits {
                jobs: 0,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 4,
                completions: 32,
                reports: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
                css_input: CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 16_000_000,
                work: 1_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    16_000_000,
                    16_000_000,
                    1_000_000,
                ),
                tasks: 32,
                storage_bytes: 16_000_000,
                scratch_bytes: 16_000_000,
                work: 1_000_000,
            },
        },
    )
    .unwrap()
}
fn absent_src_insertion_creates_child(local: &str) {
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let top = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
    };
    let snapshot = session.snapshot();
    let html = snapshot.node(top.document().node()).unwrap().children()[0];
    let parent = snapshot.node(html).unwrap().children()[1];
    assert_eq!(
        snapshot
            .node(parent)
            .unwrap()
            .as_element()
            .unwrap()
            .name()
            .local_name(),
        &DocumentString::from("body")
    );
    assert!(
        snapshot
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let (closed, container) = {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        // Native initialization in a genuinely closed associated Document is
        // supported independently of the later hosted insertion hook.
        let closed = batch.create_html_document(top.document(), None);
        let container = batch.create_element_ns(&closed, Namespace::html(), local);
        let commit = dom.apply(batch, &cancel()).unwrap();
        (
            commit.document(&closed).unwrap(),
            commit.created(&container).unwrap().unwrap(),
        )
    };
    let before = session.snapshot();
    let old_node = before.node(container).unwrap();
    assert_eq!(old_node.owner_document(), Some(closed));
    assert_eq!(old_node.parent(), None);
    let old_element = old_node.as_element().unwrap();
    assert_eq!(old_element.child_navigable(), Some(None));
    assert_eq!(old_element.attribute(&DocumentString::from("src")), None);
    assert_eq!(old_element.attribute(&DocumentString::from("srcdoc")), None);
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut policy,
        url: None,
    };
    let completion = {
        let mut dom = session.dom();
        let progress = source
            .apply(
                &mut dom,
                DomSourceEdit::insert_before(container, parent, None),
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
                &mut services,
                &cancel(),
            )
            .expect("absent-src insertion must fulfill the actual child-creation hook");
        let DomSourceDriveProgress::Complete(completion) = progress else {
            panic!("the admitted initial child must finish its mandatory insertion hooks")
        };
        completion
    };
    let current = session.snapshot();
    assert_eq!(completion.commit().changes().to(), current.stamp());
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(current.incomplete_hosted_operation(), None);
    let node = current.node(container).unwrap();
    assert_eq!(node.parent(), Some(parent));
    assert_eq!(node.owner_document(), Some(top.document()));
    assert!(node.is_connected());
    let child = node
        .as_element()
        .unwrap()
        .child_navigable()
        .expect("actual Frame/Iframe family")
        .expect("connection must create its canonical child before attribute processing");
    assert_ne!(child, top.navigable());
    assert!(
        current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(before.node(container).unwrap().parent(), None);
    assert_eq!(
        before.node(container).unwrap().owner_document(),
        Some(closed)
    );
    assert_eq!(
        before
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
}

#[test]
fn absent_src_frame_source_insertion_creates_actual_child_and_completes() {
    absent_src_insertion_creates_child("frame");
}

#[test]
fn absent_src_iframe_source_insertion_creates_actual_child_and_completes() {
    absent_src_insertion_creates_child("iframe");
}

fn top_level(session: &mut DomSession) -> DomHostedTopLevel {
    let mut hosted = session.hosted_execution();
    let mut dom = session.dom();
    let host = hosted.create_host(&mut dom, &cancel()).unwrap();
    hosted.create_top_level(&mut dom, host, &cancel()).unwrap()
}
fn body(snapshot: &DomSnapshot, document: DocumentId) -> NodeId {
    let root = snapshot.node(document.node()).unwrap().children()[0];
    snapshot.node(root).unwrap().children()[1]
}
fn detached_siblings(session: &mut DomSession, associated: DocumentId) -> (NodeId, [NodeId; 2]) {
    let mut dom = session.dom();
    let mut batch = DomBatch::new();
    let document = batch.create_html_document(associated, None);
    let root = batch.create_element_ns(&document, Namespace::html(), "div");
    let frame = batch.create_element_ns(&document, Namespace::html(), "frame");
    let iframe = batch.create_element_ns(&document, Namespace::html(), "iframe");
    batch.insert_before(&frame, &root, None);
    batch.insert_before(&iframe, &root, None);
    let commit = dom.apply(batch, &cancel()).unwrap();
    (
        commit.created(&root).unwrap().unwrap(),
        [
            commit.created(&frame).unwrap().unwrap(),
            commit.created(&iframe).unwrap().unwrap(),
        ],
    )
}
fn assert_sibling_children(snapshot: &DomSnapshot, top: DomHostedTopLevel, siblings: [NodeId; 2]) {
    let document = top.document();
    let first = snapshot.node(siblings[0]).unwrap();
    let second = snapshot.node(siblings[1]).unwrap();
    assert_eq!(first.owner_document(), Some(document));
    assert_eq!(second.owner_document(), Some(document));
    assert!(first.is_connected());
    assert!(second.is_connected());
    let first_child = first
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let second_child = second
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    assert_ne!(first_child, second_child);
    let first_document = assert_initial_child(snapshot, top, siblings[0]);
    let second_document = assert_initial_child(snapshot, top, siblings[1]);
    assert_ne!(first_document, second_document);
}

#[test]
fn single_subtree_insertion_creates_both_sibling_children_before_source_completion() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (root, siblings) = detached_siblings(&mut session, top.document());
    let before = session.snapshot();
    for node in siblings {
        assert_eq!(
            before
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }
    let mut source = session.source_execution();
    {
        let mut dom = session.dom();
        let mut policy = |_: &DomStyleSource| None;
        let mut services = DomSourceServices {
            meta: None,
            execution_scope: None,
            style_policy: &mut policy,
            url: None,
        };
        assert!(matches!(
            source
                .apply(
                    &mut dom,
                    DomSourceEdit::insert_before(root, parent, None),
                    DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
                    &mut services,
                    &cancel()
                )
                .unwrap(),
            DomSourceDriveProgress::Complete(_)
        ));
    }
    let current = session.snapshot();
    assert_sibling_children(&current, top, siblings);
    assert_eq!(current.node(root).unwrap().children(), &siblings);
    assert_eq!(current.incomplete_source_operation(), None);
    assert_eq!(current.incomplete_hosted_operation(), None);
    for node in siblings {
        assert_eq!(
            before
                .node(node)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }
}

#[test]
fn details_callback_inserts_both_siblings_without_waiting_on_its_own_window_loop() {
    struct Context {
        element_tasks: DomElementTaskRuntime,
        source: DomSourceRuntime,
        root: NodeId,
        parent: NodeId,
        returned: Vec<Result<DomSourceDriveProgress, DomSourceDriveError>>,
    }
    impl DomElementTaskEventContext for Context {
        fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
            &mut self.element_tasks
        }
    }
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (root, siblings) = detached_siblings(&mut session, top.document());
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        source: session.source_execution(),
        root,
        parent,
        returned: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<Context>::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 8,
        nested_dispatches: 8,
    })
    .unwrap();
    {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(top.document(), Namespace::html(), "details");
        batch.insert_before(&details, parent, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let details = commit.created(&details).unwrap().unwrap();
        registry
            .add_listener(
                &mut dom,
                details,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, _| {
                    let mut policy = |_: &DomStyleSource| None;
                    let mut services = DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut policy,
                        url: None,
                    };
                    let result = context.source.apply(
                        dom,
                        DomSourceEdit::insert_before(context.root, context.parent, None),
                        DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
                        &mut services,
                        &cancel(),
                    );
                    context.returned.push(result);
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let progress = DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: DomEventDispatchLimits {
                    path_items: 128,
                    snapshot_listeners: 128,
                    failure_records: 32,
                    callbacks: 128,
                    work: 100_000,
                },
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
        .unwrap();
        match progress {
            DomElementTaskProgress::Complete { receipt, .. } => {
                assert_eq!(receipt.callbacks(), 1);
                assert!(receipt.failures().is_empty());
            }
            other => {
                panic!("the parent Details task must complete before child history jobs: {other:?}")
            }
        }
    }
    assert_eq!(context.returned.len(), 1);
    assert!(matches!(
        &context.returned[0],
        Ok(DomSourceDriveProgress::Complete(_))
    ));
    assert_sibling_children(&session.snapshot(), top, siblings);
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );
    assert_eq!(session.snapshot().incomplete_source_operation(), None);
    assert_eq!(session.snapshot().incomplete_hosted_operation(), None);
}

fn assert_initial_child(
    snapshot: &DomSnapshot,
    top: DomHostedTopLevel,
    container: NodeId,
) -> DocumentId {
    let child = snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap();
    let navigable = snapshot.hosted_navigable(child).unwrap();
    assert_eq!(navigable.id(), child);
    assert_eq!(navigable.parent(), Some(top.navigable()));
    assert_eq!(navigable.container(), Some(container));
    assert_eq!(navigable.traversable(), top.navigable());
    assert_eq!(navigable.host(), top.host());
    assert_eq!(navigable.current_entry(), navigable.active_entry());
    assert!(!navigable.is_retiring());
    assert!(!navigable.is_retired());
    let document = navigable
        .active_document()
        .expect("real active initial child Document");
    assert_ne!(document, top.document());
    let entry = snapshot.hosted_entry(navigable.active_entry()).unwrap();
    assert_eq!(entry.document(), Some(document));
    assert_eq!(entry.step(), None);
    assert_eq!(entry.history(), Some((0, 0)));
    assert!(entry.nested_histories().is_empty());
    let parent_entry = snapshot.hosted_entry(top.entry()).unwrap();
    assert!(parent_entry.nested_histories().is_empty());
    assert_eq!(parent_entry.history(), Some((1, 0)));
    let child_document = snapshot.document(document).unwrap();
    assert!(child_document.is_fully_active().unwrap());
    assert_eq!(
        child_document.metadata().url(),
        &DocumentString::from("about:blank")
    );
    assert_eq!(
        child_document.metadata().encoding(),
        &DocumentString::from("UTF-8")
    );
    assert_eq!(
        child_document.metadata().origin(),
        snapshot
            .document(top.document())
            .unwrap()
            .metadata()
            .origin()
    );
    assert_eq!(
        snapshot.global_document(navigable.global()).unwrap(),
        document
    );
    assert_ne!(navigable.global(), top.global());
    let registry = child_document.metadata().custom_element_registry();
    assert_ne!(
        registry,
        snapshot
            .document(top.document())
            .unwrap()
            .metadata()
            .custom_element_registry()
    );
    let DomRegistryBinding::Registry(registry) = registry else {
        panic!("a fresh actual child Global supplies its own non-null registry")
    };
    assert_eq!(snapshot.registry(registry).unwrap().definition_count(), 0);
    document
}

fn complete_source_edit(session: &mut DomSession, edit: DomSourceEdit) {
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut policy,
        url: None,
    };
    let mut dom = session.dom();
    assert!(matches!(
        source
            .apply(
                &mut dom,
                edit,
                DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
                &mut services,
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
}

fn child_id(snapshot: &DomSnapshot, container: NodeId) -> DomHostedNavigableId {
    snapshot
        .node(container)
        .unwrap()
        .as_element()
        .unwrap()
        .child_navigable()
        .unwrap()
        .unwrap()
}

// A bounded sequence follows the real FIFO completion observation, rather than
// assuming a number of internal traversal, Global-job, or checkpoint phases.
fn pump_one_history(
    session: &mut DomSession,
    traversable: DomHostedNavigableId,
) -> (DomHostedOperationId, Vec<(DocumentId, DomHostedGlobalId)>) {
    let mut hosted = session.hosted_execution();
    let mut original = None;
    let mut executed = Vec::new();
    for _ in 0..256 {
        let progress = hosted
            .run_next(&mut session.dom(), traversable, &cancel())
            .unwrap();
        let operation = match progress {
            DomHostedTaskProgress::Idle => panic!("queued history must not disappear"),
            DomHostedTaskProgress::Quiescent
            | DomHostedTaskProgress::FullscreenExitPending { .. } => {
                panic!("history fixture has no parallel fullscreen invocation")
            }
            DomHostedTaskProgress::Pending { operation, .. }
            | DomHostedTaskProgress::CanceledDocumentDestroyed { operation, .. } => operation,
            DomHostedTaskProgress::Executed {
                operation,
                document,
                global,
                kind,
            } => {
                assert_eq!(kind, DomHostedTaskKind::History);
                executed.push((document, global));
                operation
            }
            DomHostedTaskProgress::Complete { operation } => {
                assert_eq!(original, Some(operation));
                return (operation, executed);
            }
        };
        match original {
            None => original = Some(operation),
            Some(first) => assert_eq!(operation, first, "later C must not overtake this C"),
        }
    }
    panic!("original queued history did not settle within the finite fixture bound")
}

#[test]
fn sibling_history_fifo_updates_both_initial_histories_before_later_creation() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (root, siblings) = detached_siblings(&mut session, top.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(root, parent, None),
    );
    let initial = session.snapshot();
    assert_sibling_children(&initial, top, siblings);
    let children = siblings.map(|node| child_id(&initial, node));
    let entries = children.map(|id| initial.hosted_navigable(id).unwrap().active_entry());
    let documents = entries.map(|id| initial.hosted_entry(id).unwrap().document().unwrap());
    let globals = children.map(|id| initial.hosted_navigable(id).unwrap().global());
    let settings_before = same_origin_settings(&mut session, top, &globals);

    let (first, jobs) = pump_one_history(&mut session, top.navigable());
    let after_first = session.snapshot();
    assert_eq!(
        after_first
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[children[0]]
    );
    assert_eq!(
        after_first.hosted_entry(entries[0]).unwrap().step(),
        Some(0)
    );
    assert_eq!(after_first.hosted_entry(entries[1]).unwrap().step(), None);
    for entry in entries {
        assert_eq!(
            after_first.hosted_entry(entry).unwrap().history(),
            Some((1, 0))
        );
    }
    for index in 0..2 {
        assert!(jobs.contains(&(documents[index], globals[index])));
    }
    let (second, _) = pump_one_history(&mut session, top.navigable());
    assert_ne!(first, second);
    let final_state = session.snapshot();
    assert_eq!(
        final_state
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &children
    );
    for entry in entries {
        assert_eq!(final_state.hosted_entry(entry).unwrap().step(), Some(0));
        assert_eq!(
            final_state.hosted_entry(entry).unwrap().history(),
            Some((1, 0))
        );
    }
    let settings_after = same_origin_settings(&mut session, top, &globals);
    for (before, after) in settings_before.iter().zip(&settings_after) {
        assert_eq!(before.top_level_origin(), after.top_level_origin());
        assert!(!before.cross_site());
    }
    assert_eq!(initial.hosted_entry(entries[0]).unwrap().step(), None);
    assert!(
        initial
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
}

#[test]
fn nested_initial_document_history_waits_for_its_actual_parent_creation() {
    let mut session = session();
    let top = top_level(&mut session);
    let top_body = body(&session.snapshot(), top.document());
    let (root, siblings) = detached_siblings(&mut session, top.document());
    let outer = siblings[0];
    // Insert just the Frame, leaving the unrelated sibling in its closed tree.
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(outer, top_body, None),
    );
    let initial_outer = session.snapshot();
    let outer_nav = child_id(&initial_outer, outer);
    let outer_view = initial_outer.hosted_navigable(outer_nav).unwrap();
    let outer_document = outer_view.active_document().unwrap();
    let outer_entry = outer_view.active_entry();
    let inner = siblings[1];
    let inner_parent = body(&initial_outer, outer_document);
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(inner, inner_parent, None),
    );
    let initial = session.snapshot();
    let inner_nav = child_id(&initial, inner);
    let inner_view = initial.hosted_navigable(inner_nav).unwrap();
    let inner_entry = inner_view.active_entry();
    let globals = [outer_view.global(), inner_view.global()];
    let settings_before = same_origin_settings(&mut session, top, &globals);
    assert_eq!(
        initial.node(inner).unwrap().owner_document(),
        Some(outer_document)
    );
    assert_eq!(inner_view.parent(), Some(outer_nav));
    assert_eq!(inner_view.traversable(), top.navigable());
    assert_eq!(initial.hosted_entry(outer_entry).unwrap().step(), None);
    assert_eq!(initial.hosted_entry(inner_entry).unwrap().step(), None);
    assert!(
        initial
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert!(
        initial
            .hosted_entry(outer_entry)
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert_eq!(initial.node(root).unwrap().children(), &[]);

    let (parent_c, _) = pump_one_history(&mut session, top.navigable());
    let parent_done = session.snapshot();
    assert_eq!(
        parent_done
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[outer_nav]
    );
    assert_eq!(
        parent_done.hosted_entry(outer_entry).unwrap().step(),
        Some(0)
    );
    assert_eq!(parent_done.hosted_entry(inner_entry).unwrap().step(), None);
    assert!(
        parent_done
            .hosted_entry(outer_entry)
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert_eq!(
        parent_done.hosted_entry(inner_entry).unwrap().history(),
        Some((1, 0))
    );
    let (child_c, _) = pump_one_history(&mut session, top.navigable());
    assert_ne!(parent_c, child_c);
    let current = session.snapshot();
    assert_eq!(
        current
            .hosted_entry(outer_entry)
            .unwrap()
            .nested_histories(),
        &[inner_nav]
    );
    assert_eq!(current.hosted_entry(inner_entry).unwrap().step(), Some(0));
    let settings_after = same_origin_settings(&mut session, top, &globals);
    for (before, after) in settings_before.iter().zip(&settings_after) {
        assert_eq!(before.top_level_origin(), after.top_level_origin());
        assert!(!before.cross_site());
    }
    assert_eq!(initial.hosted_entry(inner_entry).unwrap().step(), None);
}

#[test]
fn removal_before_creation_history_clears_live_edge_but_preserves_residual_history() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (_, siblings) = detached_siblings(&mut session, top.document());
    let container = siblings[0];
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(container, parent, None),
    );
    let before = session.snapshot();
    let child = child_id(&before, container);
    let nav = before.hosted_navigable(child).unwrap();
    let document = nav.active_document().unwrap();
    let entry = nav.active_entry();
    assert_eq!(before.hosted_entry(entry).unwrap().step(), None);
    complete_source_edit(&mut session, DomSourceEdit::detach(container));
    let removed = session.snapshot();
    assert_eq!(removed.node(container).unwrap().parent(), None);
    assert_eq!(
        removed
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert!(
        removed
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );

    let mut hosted = session.hosted_execution();
    let mut idle = false;
    for _ in 0..512 {
        let progress = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap();
        if matches!(progress, DomHostedTaskProgress::Idle) {
            idle = true;
            break;
        }
    }
    assert!(
        idle,
        "captured creation and destruction must both settle finitely"
    );
    let current = session.snapshot();
    assert_eq!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[child]
    );
    assert!(current.hosted_navigable(child).unwrap().is_retired());
    assert_eq!(current.hosted_entry(entry).unwrap().document(), None);
    assert!(
        current.node(document.node()).is_ok(),
        "destruction retains canonical node identity"
    );
    assert!(
        !current
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        current.node(container).unwrap().owner_document(),
        Some(top.document())
    );
    assert_eq!(child_id(&before, container), child);
    assert!(
        before
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        before
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
}

fn child_target_name_preserves_nullable_source_operand(local: &str) {
    let names = [
        None,
        Some(DocumentString::from("")),
        Some(DocumentString::from_utf16(vec![
            0x0041, 0xD800, 0x0000, 0x0062,
        ])),
    ];
    for name in names {
        let mut session = session();
        let top = top_level(&mut session);
        let parent = body(&session.snapshot(), top.document());
        let container = {
            let mut dom = session.dom();
            let mut batch = DomBatch::new();
            let closed = batch.create_html_document(top.document(), None);
            let element = batch.create_element_ns(&closed, Namespace::html(), local);
            if let Some(value) = &name {
                batch.set_attribute(&element, "name", value.clone());
            }
            dom.apply(batch, &cancel())
                .unwrap()
                .created(&element)
                .unwrap()
                .unwrap()
        };
        complete_source_edit(
            &mut session,
            DomSourceEdit::insert_before(container, parent, None),
        );
        let initial = session.snapshot();
        let child = child_id(&initial, container);
        let navigable = initial.hosted_navigable(child).unwrap();
        let entry_id = navigable.active_entry();
        let document_id = navigable.active_document().unwrap();
        let entry = initial.hosted_entry(entry_id).unwrap();
        let document = initial.document(document_id).unwrap();
        // HTML877d raw71115–71120 assigns the nullable attribute operand
        // directly after Document creation. Absent and supplied-empty differ.
        assert_eq!(entry.target_name(), name.as_ref());
        assert_eq!(entry.initiator_origin(), Some(document.metadata().origin()));
        assert_eq!(entry.origin(), document.metadata().origin());
        assert_eq!(
            entry.about_base_url(),
            document.base_document_facts().unwrap().about_base_url()
        );
        assert_eq!(entry.step(), None);
        let top_entry = initial.hosted_entry(top.entry()).unwrap();
        assert_eq!(top_entry.target_name(), Some(&DocumentString::from("")));
        assert_eq!(top_entry.initiator_origin(), None);
        assert!(matches!(
            top_entry.about_base_url(),
            surgeist_html_semantics::HtmlAboutBaseUrl::KnownNull
        ));

        pump_one_history(&mut session, top.navigable());
        let current = session.snapshot();
        let after = current.hosted_entry(entry_id).unwrap();
        assert_eq!(after.target_name(), name.as_ref());
        assert_eq!(after.origin(), entry.origin());
        assert_eq!(after.initiator_origin(), entry.initiator_origin());
        assert_eq!(after.about_base_url(), entry.about_base_url());
        assert_eq!(after.step(), Some(0));
        assert_eq!(initial.hosted_entry(entry_id).unwrap().step(), None);
        assert_eq!(
            initial.hosted_entry(entry_id).unwrap().target_name(),
            name.as_ref()
        );
    }
}

#[test]
fn frame_initial_target_name_distinguishes_absent_empty_and_exact_utf16() {
    child_target_name_preserves_nullable_source_operand("frame");
}

#[test]
fn iframe_initial_target_name_distinguishes_absent_empty_and_exact_utf16() {
    child_target_name_preserves_nullable_source_operand("iframe");
}

fn same_origin_settings(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    globals: &[DomHostedGlobalId],
) -> Vec<DomHostedSettings> {
    let snapshot = session.snapshot();
    let root_document = snapshot.document(top.document()).unwrap();
    let root_origin = root_document.metadata().origin();
    let hosted = session.hosted_execution();
    let dom = session.dom();
    globals
        .iter()
        .map(|global| {
            let settings = hosted.settings(&dom, *global, &cancel()).unwrap();
            // HTML877d raw66860–66865 follows the actual parent settings.
            // These unsandboxed child chains share the root's opaque origin.
            assert_eq!(settings.top_level_origin(), root_origin);
            assert_eq!(settings.document().metadata().origin(), root_origin);
            assert!(!settings.cross_site());
            settings
        })
        .collect()
}

fn nested_containers(session: &mut DomSession, top: DomHostedTopLevel) -> [NodeId; 2] {
    let parent = body(&session.snapshot(), top.document());
    let (_, containers) = detached_siblings(session, top.document());
    complete_source_edit(
        session,
        DomSourceEdit::insert_before(containers[0], parent, None),
    );
    let snapshot = session.snapshot();
    let outer = child_id(&snapshot, containers[0]);
    let document = snapshot
        .hosted_navigable(outer)
        .unwrap()
        .active_document()
        .unwrap();
    let parent = body(&snapshot, document);
    complete_source_edit(
        session,
        DomSourceEdit::insert_before(containers[1], parent, None),
    );
    containers
}

fn settle_queued_tasks(
    session: &mut DomSession,
    traversable: DomHostedNavigableId,
    generations: &[(DomHostedEntryId, DocumentId)],
) -> Vec<DomHostedTaskProgress> {
    let mut hosted = session.hosted_execution();
    let mut observed = Vec::new();
    for _ in 0..512 {
        let result = {
            let mut dom = session.dom();
            hosted.run_next(&mut dom, traversable, &cancel())
        };
        let progress = result.unwrap_or_else(|error| {
            let current = session.snapshot();
            let states = generations.iter().map(|(entry, document)| {
                let row = current.hosted_entry(*entry).unwrap();
                (*entry, *document, row.document(), row.step(), row.history(),
                    row.nested_histories().to_vec(), current.document(*document).unwrap().is_fully_active())
            }).collect::<Vec<_>>();
            panic!("queued execution refused: {error:?}; earlier progress={observed:?}; current generations={states:?}")
        });
        if let DomHostedTaskProgress::CanceledDocumentDestroyed { document, .. } = progress {
            let (entry, _) = generations
                .iter()
                .find(|(_, known)| *known == document)
                .expect("retirement must identify an actual captured Document generation");
            let current = session.snapshot();
            assert_eq!(current.hosted_entry(*entry).unwrap().document(), None);
            assert!(
                !current
                    .document(document)
                    .unwrap()
                    .is_fully_active()
                    .unwrap()
            );
        }
        observed.push(progress);
        if matches!(progress, DomHostedTaskProgress::Idle) {
            return observed;
        }
    }
    panic!("actual queued obligations did not reach their public Idle observation")
}

#[test]
fn overlapping_nested_removals_settle_captured_waiters_with_one_cleanup_per_document() {
    let mut session = session();
    let top = top_level(&mut session);
    let containers = nested_containers(&mut session, top);
    pump_one_history(&mut session, top.navigable());
    pump_one_history(&mut session, top.navigable());
    let before = session.snapshot();
    let children = containers.map(|container| child_id(&before, container));
    let entries = children.map(|child| before.hosted_navigable(child).unwrap().active_entry());
    let documents = entries.map(|entry| before.hosted_entry(entry).unwrap().document().unwrap());
    assert_eq!(
        before.hosted_entry(top.entry()).unwrap().nested_histories(),
        &[children[0]]
    );
    assert_eq!(
        before.hosted_entry(entries[0]).unwrap().nested_histories(),
        &[children[1]]
    );

    // The ancestor invocation captures its real child before either queued
    // wrapper can run. Direct child removal then starts a second invocation
    // for that same exact child generation, rather than replacing the waiter.
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[0]));
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[1]));
    let removed = session.snapshot();
    for container in containers {
        assert_eq!(removed.node(container).unwrap().parent(), None);
        assert_eq!(
            removed
                .node(container)
                .unwrap()
                .as_element()
                .unwrap()
                .child_navigable(),
            Some(None)
        );
    }
    for entry in entries {
        assert!(
            removed.hosted_entry(entry).unwrap().document().is_some(),
            "queueing or unlinking does not itself complete canonical destruction"
        );
    }
    let observations = settle_queued_tasks(
        &mut session,
        top.navigable(),
        &[(entries[0], documents[0]), (entries[1], documents[1])],
    );
    for document in documents {
        assert_eq!(observations.iter().filter(|progress| matches!(progress,
            DomHostedTaskProgress::Executed { document: actual, kind: DomHostedTaskKind::Destruction, .. } if *actual == document)).count(), 1,
            "each exact Document has one actual destructor body, even with multiple invocations");
    }
    let completed = observations
        .iter()
        .filter_map(|progress| match progress {
            DomHostedTaskProgress::Complete { operation } => Some(*operation),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(
        completed.len() >= 4,
        "both removal histories and both invoking wrappers must settle"
    );
    for (index, operation) in completed.iter().enumerate() {
        assert!(
            !completed[..index].contains(operation),
            "completion is once per actual operation"
        );
    }
    let current = session.snapshot();
    for (index, child) in children.into_iter().enumerate() {
        assert!(current.hosted_navigable(child).unwrap().is_retired());
        assert_eq!(
            current.hosted_entry(entries[index]).unwrap().document(),
            None
        );
        assert!(current.node(documents[index].node()).is_ok());
        assert!(
            before
                .document(documents[index])
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
    }
    assert!(
        current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert_eq!(
        before.hosted_entry(top.entry()).unwrap().nested_histories(),
        &[children[0]]
    );
}

#[test]
fn pending_child_creation_appends_to_removed_established_parent_without_restoring_root_history() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (_, containers) = detached_siblings(&mut session, top.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(containers[0], parent, None),
    );
    pump_one_history(&mut session, top.navigable());
    let established = session.snapshot();
    let outer = child_id(&established, containers[0]);
    let outer_view = established.hosted_navigable(outer).unwrap();
    let outer_entry = outer_view.active_entry();
    let outer_document = outer_view.active_document().unwrap();
    assert_eq!(
        established
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[outer]
    );
    assert_eq!(
        established.hosted_entry(outer_entry).unwrap().step(),
        Some(0)
    );
    let parent = body(&established, outer_document);
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(containers[1], parent, None),
    );
    let initial = session.snapshot();
    let inner = child_id(&initial, containers[1]);
    let inner_view = initial.hosted_navigable(inner).unwrap();
    let inner_entry = inner_view.active_entry();
    let inner_document = inner_view.active_document().unwrap();
    assert_eq!(initial.hosted_entry(inner_entry).unwrap().step(), None);
    assert!(
        initial
            .hosted_entry(outer_entry)
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[0]));
    let removed = session.snapshot();
    assert!(
        removed
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert_eq!(
        removed.hosted_navigable(outer).unwrap().active_entry(),
        outer_entry
    );
    assert_eq!(removed.hosted_entry(outer_entry).unwrap().step(), Some(0));

    settle_queued_tasks(
        &mut session,
        top.navigable(),
        &[(outer_entry, outer_document), (inner_entry, inner_document)],
    );
    let current = session.snapshot();
    assert!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
    assert_eq!(
        current.hosted_navigable(outer).unwrap().active_entry(),
        outer_entry
    );
    assert_eq!(
        current
            .hosted_entry(outer_entry)
            .unwrap()
            .nested_histories(),
        &[inner]
    );
    assert_eq!(current.hosted_entry(inner_entry).unwrap().step(), Some(0));
    assert_eq!(current.hosted_entry(outer_entry).unwrap().document(), None);
    assert_eq!(current.hosted_entry(inner_entry).unwrap().document(), None);
    assert!(current.hosted_navigable(outer).unwrap().is_retired());
    assert!(current.hosted_navigable(inner).unwrap().is_retired());
    assert_eq!(initial.hosted_entry(inner_entry).unwrap().step(), None);
    assert_eq!(
        established
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[outer]
    );
}

#[test]
fn captured_history_job_waits_for_actual_destruction_without_writing_inactive_document_history() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (root, containers) = detached_siblings(&mut session, top.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(root, parent, None),
    );
    let initial = session.snapshot();
    let children = containers.map(|node| child_id(&initial, node));
    let entries = children.map(|child| initial.hosted_navigable(child).unwrap().active_entry());
    let documents = entries.map(|entry| initial.hosted_entry(entry).unwrap().document().unwrap());
    let globals = children.map(|child| initial.hosted_navigable(child).unwrap().global());
    let mut hosted = session.hosted_execution();
    let mut first_creation = None;
    // A real first-update Global task proves that C has captured its update;
    // no private phase flag or fabricated task is used to place the removal.
    for _ in 0..256 {
        let progress = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap();
        match progress {
            DomHostedTaskProgress::Executed {
                operation,
                document,
                kind: DomHostedTaskKind::History,
                ..
            } if document == top.document() => {
                first_creation = Some(operation);
                break;
            }
            DomHostedTaskProgress::Complete { .. } | DomHostedTaskProgress::Idle => {
                panic!("the first real update must not disappear before its Global task")
            }
            _ => {}
        }
    }
    let first_creation = first_creation.expect("first update's actual top-Document History task");
    assert_eq!(
        session
            .snapshot()
            .hosted_entry(entries[1])
            .unwrap()
            .history(),
        Some((0, 0))
    );
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[1]));
    let inactive = session.snapshot();
    assert_eq!(
        inactive.hosted_entry(entries[1]).unwrap().document(),
        Some(documents[1])
    );
    assert!(
        !inactive
            .document(documents[1])
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(
        inactive.hosted_entry(entries[1]).unwrap().history(),
        Some((0, 0))
    );
    // Neither inactivity nor caller cancellation is completed destruction.
    assert!(matches!(
        hosted.run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(true)),
        Err(DomHostedRuntimeError::Cancelled)
    ));
    let retained = session.snapshot();
    assert_eq!(retained.stamp(), inactive.stamp());
    assert_eq!(
        retained.hosted_entry(entries[1]).unwrap().document(),
        Some(documents[1])
    );
    assert_eq!(
        retained.hosted_entry(entries[1]).unwrap().history(),
        Some((0, 0))
    );

    let observations = settle_queued_tasks(
        &mut session,
        top.navigable(),
        &[(entries[0], documents[0]), (entries[1], documents[1])],
    );
    assert!(
        observations.iter().any(|progress| matches!(progress,
        DomHostedTaskProgress::CanceledDocumentDestroyed { operation, document, global }
            if *operation == first_creation && *document == documents[1] && *global == globals[1]))
    );
    assert!(
        !observations.iter().any(|progress| matches!(progress,
        DomHostedTaskProgress::Executed { document, kind: DomHostedTaskKind::History, .. }
            if *document == documents[1])),
        "retired job must not execute or write History"
    );
    assert!(
        observations.iter().any(|progress| matches!(progress,
        DomHostedTaskProgress::Executed { document, kind: DomHostedTaskKind::History, .. }
            if *document == documents[0])),
        "surviving sibling still executes its real update"
    );
    let current = session.snapshot();
    assert_eq!(
        current.hosted_entry(entries[0]).unwrap().history(),
        Some((1, 0))
    );
    assert_eq!(
        current.hosted_entry(entries[1]).unwrap().history(),
        Some((0, 0))
    );
    assert_eq!(current.hosted_entry(entries[1]).unwrap().document(), None);
    assert_eq!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &children
    );
    assert_eq!(current.hosted_entry(entries[1]).unwrap().step(), Some(0));
    assert_eq!(initial.hosted_entry(entries[1]).unwrap().step(), None);
    assert!(
        initial
            .document(documents[1])
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

#[test]
fn direct_grandchild_removal_after_recursive_join_keeps_each_destructor_once() {
    let mut session = session();
    let top = top_level(&mut session);
    let containers = nested_containers(&mut session, top);
    pump_one_history(&mut session, top.navigable());
    pump_one_history(&mut session, top.navigable());
    let before = session.snapshot();
    let children = containers.map(|container| child_id(&before, container));
    let entries = children.map(|child| before.hosted_navigable(child).unwrap().active_entry());
    let documents = entries.map(|entry| before.hosted_entry(entry).unwrap().document().unwrap());
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[0]));
    let removed = session.snapshot();
    assert!(removed.hosted_navigable(children[0]).unwrap().is_retiring());
    assert!(!removed.hosted_navigable(children[1]).unwrap().is_retiring());
    let mut hosted = session.hosted_execution();
    let mut earlier = Vec::new();
    let mut joined = false;
    for _ in 0..256 {
        let progress = hosted
            .run_next(&mut session.dom(), top.navigable(), &cancel())
            .unwrap();
        earlier.push(progress);
        let current = session.snapshot();
        if matches!(
            progress,
            DomHostedTaskProgress::Pending {
                reason: DomHostedTaskWait::Descendants,
                ..
            }
        ) && current.hosted_navigable(children[1]).unwrap().is_retiring()
        {
            assert!(!current.hosted_navigable(children[1]).unwrap().is_retired());
            for (entry, document) in entries.into_iter().zip(documents) {
                assert_eq!(
                    current.hosted_entry(entry).unwrap().document(),
                    Some(document)
                );
            }
            joined = true;
            break;
        }
        assert!(
            !matches!(progress, DomHostedTaskProgress::Idle),
            "captured child cannot disappear before recursive join"
        );
    }
    assert!(
        joined,
        "actual descendant retirement must be registered before cleanup runs"
    );
    complete_source_edit(&mut session, DomSourceEdit::detach(containers[1]));
    earlier.extend(settle_queued_tasks(
        &mut session,
        top.navigable(),
        &[(entries[0], documents[0]), (entries[1], documents[1])],
    ));
    for document in documents {
        assert_eq!(earlier.iter().filter(|progress| matches!(progress,
            DomHostedTaskProgress::Executed { document: actual, kind: DomHostedTaskKind::Destruction, .. } if *actual == document)).count(), 1);
    }
    let current = session.snapshot();
    for (index, child) in children.into_iter().enumerate() {
        assert!(current.hosted_navigable(child).unwrap().is_retired());
        assert_eq!(
            current.hosted_entry(entries[index]).unwrap().document(),
            None
        );
        assert!(
            before
                .document(documents[index])
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
    }
    assert!(
        current
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories()
            .is_empty()
    );
}

#[test]
fn retiring_document_refuses_fresh_child_before_single_edit_publication() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (_, containers) = detached_siblings(&mut session, top.document());
    let container = containers[0];
    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let before = session.snapshot();
    assert!(
        before
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retiring()
    );
    assert!(
        !before
            .hosted_navigable(top.navigable())
            .unwrap()
            .is_retired()
    );
    assert_eq!(
        before.hosted_entry(top.entry()).unwrap().document(),
        Some(top.document())
    );
    let old_parent = before.node(container).unwrap().parent();
    let old_document = before.node(container).unwrap().owner_document();
    let mut source = session.source_execution();
    let mut policy = |_: &DomStyleSource| None;
    let mut services = DomSourceServices {
        meta: None,
        execution_scope: None,
        style_policy: &mut policy,
        url: None,
    };
    assert!(matches!(
        source.apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(container, parent, None),
            DomSourceLimits::new(16_000_000, 1_000_000, 10_000_000, 128, 4096, 4096, 1),
            &mut services,
            &cancel(),
        ),
        Err(surgeist_runtime::DomSourceDriveError::Source(
            DomSourceRuntimeError::Dom(DomError::HostedSourceUnavailable)
        ))
    ));
    let refused = session.snapshot();
    assert_eq!(refused.stamp(), before.stamp());
    assert_eq!(refused.node(container).unwrap().parent(), old_parent);
    assert_eq!(
        refused.node(container).unwrap().owner_document(),
        old_document
    );
    assert_eq!(
        refused
            .node(container)
            .unwrap()
            .as_element()
            .unwrap()
            .child_navigable(),
        Some(None)
    );
    assert_eq!(refused.incomplete_source_operation(), None);
    assert_eq!(
        refused.hosted_entry(top.entry()).unwrap().document(),
        Some(top.document())
    );

    let mut complete = false;
    let mut bodies = 0;
    for _ in 0..256 {
        let progress = hosted
            .run_next(&mut session.dom(), queued.traversable(), &cancel())
            .unwrap();
        if matches!(progress, DomHostedTaskProgress::Executed { document, kind: DomHostedTaskKind::Destruction, .. } if document == top.document())
        {
            bodies += 1;
        }
        if matches!(progress, DomHostedTaskProgress::Complete { operation } if operation == queued.operation())
        {
            complete = true;
            break;
        }
    }
    assert!(
        complete,
        "the original admitted zero-child destruction still completes"
    );
    assert_eq!(bodies, 1);
    assert_eq!(
        session
            .snapshot()
            .hosted_entry(top.entry())
            .unwrap()
            .document(),
        None
    );
    assert!(
        before
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

#[test]
fn history_job_task_admission_refusal_retains_original_fifo_for_raised_limit_retry() {
    let mut session = session();
    let top = top_level(&mut session);
    let parent = body(&session.snapshot(), top.document());
    let (root, containers) = detached_siblings(&mut session, top.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(root, parent, None),
    );
    let initial = session.snapshot();
    let children = containers.map(|node| child_id(&initial, node));
    let entries = children.map(|child| initial.hosted_navigable(child).unwrap().active_entry());
    let documents = entries.map(|entry| initial.hosted_entry(entry).unwrap().document().unwrap());
    let globals = children.map(|child| initial.hosted_navigable(child).unwrap().global());
    assert_sibling_children(&initial, top, containers);
    let policy = |tasks| DomHostedRuntimeLimits {
        canonical: DomHostedLimits::new(
            DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
            16_000_000,
            16_000_000,
            1_000_000,
        ),
        tasks,
        storage_bytes: 16_000_000,
        scratch_bytes: 16_000_000,
        work: 1_000_000,
    };
    let mut hosted = session.hosted_execution();
    // Both original C rows fit. Their first additional real Global task does
    // not. The cap concerns actual queue admission, not a fabricated service.
    hosted
        .set_limits(&mut session.dom(), policy(2), &cancel())
        .unwrap();
    for _ in 0..2 {
        assert!(matches!(
            hosted.run_next(&mut session.dom(), top.navigable(), &cancel()),
            Err(DomHostedRuntimeError::Admission)
        ));
        let retained = session.snapshot();
        assert_eq!(
            retained
                .hosted_entry(top.entry())
                .unwrap()
                .nested_histories(),
            &[children[0]]
        );
        assert_eq!(retained.hosted_entry(entries[0]).unwrap().step(), Some(0));
        assert_eq!(retained.hosted_entry(entries[1]).unwrap().step(), None);
        for index in 0..2 {
            let nav = retained.hosted_navigable(children[index]).unwrap();
            assert_eq!(nav.current_entry(), entries[index]);
            assert_eq!(nav.active_entry(), entries[index]);
            assert_eq!(nav.active_document(), Some(documents[index]));
            assert_eq!(nav.global(), globals[index]);
            assert_eq!(
                retained.hosted_entry(entries[index]).unwrap().history(),
                Some((0, 0))
            );
        }
        assert_eq!(retained.incomplete_source_operation(), None);
    }
    assert!(matches!(
        hosted.run_next(&mut session.dom(), top.navigable(), &AtomicBool::new(true)),
        Err(DomHostedRuntimeError::Cancelled)
    ));
    let cancelled = session.snapshot();
    assert_eq!(
        cancelled
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[children[0]]
    );
    assert_eq!(cancelled.hosted_entry(entries[0]).unwrap().step(), Some(0));
    assert_eq!(cancelled.hosted_entry(entries[1]).unwrap().step(), None);
    for entry in entries {
        assert_eq!(
            cancelled.hosted_entry(entry).unwrap().history(),
            Some((0, 0))
        );
    }

    // No Source edit is replayed: the same original owner gains room for one
    // actual Global task while preserving both captured creation closures.
    hosted
        .set_limits(&mut session.dom(), policy(3), &cancel())
        .unwrap();
    let (first_creation, jobs) = pump_one_history(&mut session, top.navigable());
    for expected in [
        (top.document(), top.global()),
        (documents[0], globals[0]),
        (documents[1], globals[1]),
    ] {
        assert_eq!(jobs.iter().filter(|job| **job == expected).count(), 1);
    }
    let first_done = session.snapshot();
    assert_eq!(
        first_done
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &[children[0]]
    );
    assert_eq!(first_done.hosted_entry(entries[1]).unwrap().step(), None);
    for entry in entries {
        assert_eq!(
            first_done.hosted_entry(entry).unwrap().history(),
            Some((1, 0))
        );
    }
    let (second_creation, _) = pump_one_history(&mut session, top.navigable());
    assert_ne!(second_creation, first_creation);
    let current = session.snapshot();
    assert_eq!(
        current
            .hosted_entry(top.entry())
            .unwrap()
            .nested_histories(),
        &children
    );
    for index in 0..2 {
        let nav = current.hosted_navigable(children[index]).unwrap();
        assert_eq!(nav.current_entry(), entries[index]);
        assert_eq!(nav.active_document(), Some(documents[index]));
        assert_eq!(nav.global(), globals[index]);
        assert_eq!(
            current.hosted_entry(entries[index]).unwrap().step(),
            Some(0)
        );
        assert_eq!(
            current.hosted_entry(entries[index]).unwrap().history(),
            Some((1, 0))
        );
        assert_eq!(child_id(&current, containers[index]), children[index]);
        assert_eq!(initial.hosted_entry(entries[index]).unwrap().step(), None);
        assert_eq!(
            initial.hosted_entry(entries[index]).unwrap().history(),
            Some((0, 0))
        );
    }
    assert_eq!(current.incomplete_source_operation(), None);
}

#[test]
fn child_details_blocks_shared_parent_loop_but_not_another_group_in_same_host() {
    struct Context {
        element_tasks: DomElementTaskRuntime,
        hosted: HostedRuntime,
        traversables: [DomHostedNavigableId; 2],
        pumped: Vec<Result<DomHostedTaskProgress, DomHostedRuntimeError>>,
    }
    impl DomElementTaskEventContext for Context {
        fn element_tasks(&mut self) -> &mut DomElementTaskRuntime {
            &mut self.element_tasks
        }
    }
    let mut session = session();
    let mut hosted = session.hosted_execution();
    let (first, second) = {
        let mut dom = session.dom();
        let host = hosted.create_host(&mut dom, &cancel()).unwrap();
        let first = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
        let second = hosted.create_top_level(&mut dom, host, &cancel()).unwrap();
        assert_eq!(first.host(), second.host());
        (first, second)
    };
    let first_body = body(&session.snapshot(), first.document());
    let second_body = body(&session.snapshot(), second.document());
    let (_, siblings) = detached_siblings(&mut session, first.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(siblings[0], first_body, None),
    );
    pump_one_history(&mut session, first.navigable());
    let established = session.snapshot();
    let first_child = child_id(&established, siblings[0]);
    let child = established.hosted_navigable(first_child).unwrap();
    let child_document = child.active_document().unwrap();
    let child_entry = child.active_entry();
    let child_body = body(&established, child_document);
    assert_eq!(
        established
            .document(child_document)
            .unwrap()
            .metadata()
            .origin(),
        established
            .document(first.document())
            .unwrap()
            .metadata()
            .origin()
    );
    assert_eq!(
        established.hosted_entry(child_entry).unwrap().history(),
        Some((1, 0))
    );

    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(siblings[1], first_body, None),
    );
    let (_, other_containers) = detached_siblings(&mut session, second.document());
    complete_source_edit(
        &mut session,
        DomSourceEdit::insert_before(other_containers[0], second_body, None),
    );
    let queued = session.snapshot();
    let sibling = child_id(&queued, siblings[1]);
    let other_child = child_id(&queued, other_containers[0]);
    let sibling_view = queued.hosted_navigable(sibling).unwrap();
    let other_view = queued.hosted_navigable(other_child).unwrap();
    let sibling_entry = sibling_view.active_entry();
    let other_entry = other_view.active_entry();
    let sibling_document = sibling_view.active_document().unwrap();
    let other_document = other_view.active_document().unwrap();
    for entry in [sibling_entry, other_entry] {
        assert_eq!(queued.hosted_entry(entry).unwrap().step(), None);
        assert_eq!(queued.hosted_entry(entry).unwrap().history(), Some((0, 0)));
    }
    let mut context = Context {
        element_tasks: session.element_task_execution(),
        hosted: session.hosted_execution(),
        traversables: [first.navigable(), second.navigable()],
        pumped: Vec::new(),
    };
    let mut registry = DomEventDispatcher::<Context>::new(DomEventRegistryLimits {
        registrations: 8,
        pending_events: 8,
        nested_dispatches: 8,
    })
    .unwrap();
    {
        let mut dom = session.dom();
        let mut batch = DomBatch::new();
        let details = batch.create_element_ns(child_document, Namespace::html(), "details");
        batch.insert_before(&details, child_body, None);
        batch.set_attribute(&details, "open", "");
        let commit = dom.apply(batch, &cancel()).unwrap();
        let details = commit.created(&details).unwrap().unwrap();
        registry
            .add_listener(
                &mut dom,
                details,
                DomEventType::Toggle,
                DomEventCallback::new(|dom, _, context: &mut Context, _| {
                    for traversable in context.traversables {
                        context
                            .pumped
                            .push(context.hosted.run_next(dom, traversable, &cancel()));
                    }
                    Ok(())
                }),
                DomEventListenerOptions {
                    capture: false,
                    once: true,
                },
            )
            .unwrap();
        assert_eq!(
            context.element_tasks.ingest(&mut dom, &cancel()).unwrap(),
            1
        );
        let progress = DomElementTaskRuntime::run_next(
            &mut dom,
            surgeist_dom::DomElementTaskSourceKind::DomManipulation,
            &mut registry,
            &mut context,
            crate::DomElementTaskRunLimits {
                dispatch: DomEventDispatchLimits {
                    path_items: 128,
                    snapshot_listeners: 128,
                    failure_records: 32,
                    callbacks: 128,
                    work: 100_000,
                },
                source: surgeist_dom::DomSourceLimits::new(
                    16_000_000, 16_000_000, 4_000_000, 64, 65_536, 65_536, 64,
                ),
            },
            &cancel(),
        )
        .unwrap();
        let DomElementTaskProgress::Complete { receipt, .. } = progress else {
            panic!("the real child Details task and its checkpoint must complete")
        };
        assert_eq!(receipt.callbacks(), 1);
        assert!(receipt.failures().is_empty());
    }
    assert_eq!(context.pumped.len(), 2);
    let first_operation = match &context.pumped[0] {
        Ok(DomHostedTaskProgress::Pending {
            operation,
            reason: DomHostedTaskWait::CurrentTask,
        }) => *operation,
        other => {
            panic!("same-origin child's running task must own its parent's actual loop: {other:?}")
        }
    };
    let other_operation = match &context.pumped[1] {
        Ok(DomHostedTaskProgress::Executed {
            operation,
            document,
            global,
            kind: DomHostedTaskKind::History,
        }) if *document == second.document() && *global == second.global() => *operation,
        other => panic!(
            "another group in the same Host must execute its eligible History job: {other:?}"
        ),
    };
    let after_callback = session.snapshot();
    assert_eq!(
        after_callback
            .hosted_entry(first.entry())
            .unwrap()
            .nested_histories(),
        &[first_child]
    );
    assert_eq!(
        after_callback.hosted_entry(sibling_entry).unwrap().step(),
        None
    );
    assert_eq!(
        after_callback
            .hosted_entry(sibling_entry)
            .unwrap()
            .history(),
        Some((0, 0))
    );
    assert_eq!(
        after_callback
            .hosted_entry(second.entry())
            .unwrap()
            .nested_histories(),
        &[other_child]
    );
    assert_eq!(
        after_callback.hosted_entry(other_entry).unwrap().step(),
        Some(0)
    );
    assert_eq!(
        after_callback.hosted_entry(other_entry).unwrap().history(),
        Some((0, 0))
    );
    assert_eq!(
        context
            .element_tasks
            .pending_tasks(surgeist_dom::DomElementTaskSourceKind::DomManipulation)
            .unwrap(),
        0
    );

    let (first_complete, first_jobs) = pump_one_history(&mut session, first.navigable());
    assert_eq!(first_complete, first_operation);
    for expected in [
        (first.document(), first.global()),
        (child_document, child.global()),
        (sibling_document, sibling_view.global()),
    ] {
        assert_eq!(first_jobs.iter().filter(|job| **job == expected).count(), 1);
    }
    let (other_complete, other_jobs) = pump_one_history(&mut session, second.navigable());
    assert_eq!(other_complete, other_operation);
    assert_eq!(
        other_jobs
            .iter()
            .filter(|job| **job == (other_document, other_view.global()))
            .count(),
        1
    );
    assert!(
        !other_jobs.contains(&(second.document(), second.global())),
        "callback's completed History body must not replay"
    );
    let current = session.snapshot();
    assert_eq!(
        current
            .hosted_entry(first.entry())
            .unwrap()
            .nested_histories(),
        &[first_child, sibling]
    );
    assert_eq!(
        current
            .hosted_entry(second.entry())
            .unwrap()
            .nested_histories(),
        &[other_child]
    );
    for entry in [child_entry, sibling_entry, other_entry] {
        assert_eq!(current.hosted_entry(entry).unwrap().history(), Some((1, 0)));
        assert_eq!(current.hosted_entry(entry).unwrap().step(), Some(0));
    }
    assert_eq!(queued.hosted_entry(sibling_entry).unwrap().step(), None);
    assert_eq!(queued.hosted_entry(other_entry).unwrap().step(), None);
    assert_eq!(current.incomplete_source_operation(), None);
}
