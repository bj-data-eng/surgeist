//! Actual native insertion histories and the same-origin Dialog consumer.
use super::*;
fn fixture(candidates: usize) -> (DomSession, DomHostedTopLevel) {
    let dom = Dom::new(
        DomLimits::new(512, 2_000_000, 2048, 2048, 2048, 64, 8_000_000)
            .unwrap()
            .with_autofocus_candidates(candidates)
            .unwrap()
            .with_dispatch_custody(
                DomDispatchCustodyLimits::new(64, 4096, 32_000_000, 32_000_000, 8_000_000).unwrap(),
            ),
    )
    .unwrap();
    let mut session = DomSession::new(
        dom,
        DomSessionLimits {
            source_events: DomSourceEventRuntimeLimits {
                jobs: 8,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            style: DomStyleRuntimeLimits {
                roots: 8,
                completions: 32,
                reports: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
                css_input: surgeist_css::CssUtf16InputLimits::default(),
            },
            element_tasks: DomElementTaskRuntimeLimits {
                tasks: 32,
                storage_bytes: 32_000_000,
                work: 4_000_000,
            },
            hosted: DomHostedRuntimeLimits {
                canonical: DomHostedLimits::new(
                    DomHostedRecordLimits::new(32, 32, 32, 32, 32, 32, 32),
                    32_000_000,
                    32_000_000,
                    4_000_000,
                ),
                tasks: 32,
                storage_bytes: 32_000_000,
                scratch_bytes: 32_000_000,
                work: 4_000_000,
            },
        },
    )
    .unwrap();
    let mut hosted = session.hosted_execution();
    let host = hosted
        .create_headless_host(&mut session.dom(), &cancel())
        .unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    (session, top)
}

fn edit(session: &mut DomSession, operation: DomSourceEdit) {
    let progress = session
        .source_execution()
        .apply(
            &mut session.dom(),
            operation,
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(progress, DomSourceDriveProgress::Complete(_)),
        "actual insertion/edit must complete: {progress:?}"
    );
}
fn detached(session: &mut DomSession, document: DocumentId, autofocus: bool) -> NodeId {
    let mut batch = DomBatch::new();
    let node = batch.create_element_ns(document, Namespace::html(), "input");
    if autofocus {
        batch.set_attribute(&node, "autofocus", "");
    }
    session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&node)
        .unwrap()
        .unwrap()
}
fn state(snapshot: &DomSnapshot, document: DocumentId, candidates: &[NodeId], processed: bool) {
    let view = snapshot.document(document).unwrap().autofocus().unwrap();
    assert_eq!(view.stamp(), snapshot.document(document).unwrap().stamp());
    assert_eq!(view.candidates(), candidates);
    assert_eq!(view.processed(), processed);
}

#[test]
fn insertion_enqueues_in_top_document_and_attribute_changes_alone_do_not_enqueue() {
    let (mut session, top) = fixture(8);
    let (_, child) = container(&mut session, top.document(), "iframe", None, None, false);
    let snapshot = session.snapshot();
    state(&snapshot, top.document(), &[], false);
    state(&snapshot, child, &[], false);
    let node = detached(&mut session, child, false);
    let destination = body(&snapshot, child);
    edit(
        &mut session,
        DomSourceEdit::insert_before(node, destination, None),
    );
    edit(
        &mut session,
        DomSourceEdit::set_attribute(node, "autofocus", ""),
    );
    state(&session.snapshot(), top.document(), &[], false);
    edit(&mut session, DomSourceEdit::detach(node));
    edit(
        &mut session,
        DomSourceEdit::insert_before(node, destination, None),
    );
    let first = session.snapshot();
    state(&first, top.document(), &[node], false);
    state(&first, child, &[], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 1);
    edit(
        &mut session,
        DomSourceEdit::remove_attribute(node, "autofocus"),
    );
    state(&session.snapshot(), top.document(), &[node], false);
    state(&snapshot, top.document(), &[], false);
    state(&first, top.document(), &[node], false);
}

#[test]
fn insertion_into_actual_connected_shadow_tree_is_not_document_tree_insertion() {
    let (mut session, top) = fixture(0);
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
    let host = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&host)
        .unwrap()
        .unwrap();
    let destination = body(&session.snapshot(), top.document());
    edit(
        &mut session,
        DomSourceEdit::insert_before(host, destination, None),
    );
    let mut batch = DomBatch::new();
    let root = batch.attach_shadow(host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    let root = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&root)
        .unwrap()
        .unwrap();
    let node = detached(&mut session, top.document(), true);
    edit(&mut session, DomSourceEdit::insert_before(node, root, None));
    let snapshot = session.snapshot();
    assert_eq!(snapshot.shadow_root(root).unwrap().host(), host);
    assert_eq!(snapshot.node(host).unwrap().parent(), Some(destination));
    assert_eq!(snapshot.node(node).unwrap().parent(), Some(root));
    state(&snapshot, top.document(), &[], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 0);
}

#[test]
fn svg_and_mathml_elements_use_actual_null_namespace_autofocus_presence() {
    let (mut session, top) = fixture(2);
    let destination = body(&session.snapshot(), top.document());
    let mut inserted = Vec::new();
    for (namespace, name) in [(Namespace::svg(), "g"), (Namespace::mathml(), "mrow")] {
        let mut batch = DomBatch::new();
        let node = batch.create_element_ns(top.document(), namespace, name);
        batch.set_attribute(&node, "autofocus", "false");
        let node = session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&node)
            .unwrap()
            .unwrap();
        edit(
            &mut session,
            DomSourceEdit::insert_before(node, destination, None),
        );
        inserted.push(node);
        state(&session.snapshot(), top.document(), &inserted, false);
    }
    // A namespaced attribute with the same local spelling is not the global
    // null-namespace attribute, and cannot consume a third candidate slot.
    let mut batch = DomBatch::new();
    let node = batch.create_element_ns(top.document(), Namespace::svg(), "g");
    batch.set_attribute_ns(&node, Namespace::svg(), "s:autofocus", "");
    let node = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&node)
        .unwrap()
        .unwrap();
    edit(
        &mut session,
        DomSourceEdit::insert_before(node, destination, None),
    );
    state(&session.snapshot(), top.document(), &inserted, false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 2);
}

#[test]
fn reinsert_reorders_and_adoption_preserves_history_until_actual_retirement() {
    let (mut session, a) = fixture(8);
    let mut hosted = session.hosted_execution();
    let host = hosted
        .create_headless_host(&mut session.dom(), &cancel())
        .unwrap();
    let b = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    let snapshot = session.snapshot();
    let parent_a = body(&snapshot, a.document());
    let parent_b = body(&snapshot, b.document());
    let x = detached(&mut session, a.document(), true);
    let y = detached(&mut session, a.document(), true);
    edit(
        &mut session,
        DomSourceEdit::insert_before(x, parent_a, None),
    );
    edit(
        &mut session,
        DomSourceEdit::insert_before(y, parent_a, None),
    );
    edit(&mut session, DomSourceEdit::detach(x));
    state(&session.snapshot(), a.document(), &[x, y], false);
    edit(
        &mut session,
        DomSourceEdit::insert_before(x, parent_a, None),
    );
    state(&session.snapshot(), a.document(), &[y, x], false);
    edit(
        &mut session,
        DomSourceEdit::insert_before(x, parent_b, None),
    );
    let adopted = session.snapshot();
    state(&adopted, a.document(), &[y, x], false);
    state(&adopted, b.document(), &[x], false);
    assert_eq!(
        adopted.node(x).unwrap().owner_document(),
        Some(b.document())
    );
    assert_eq!(session.dom().usage().autofocus_candidates(), 3);
    edit(&mut session, DomSourceEdit::detach(x));
    edit(&mut session, DomSourceEdit::destroy(x));
    let retired = session.snapshot();
    assert!(retired.node(x).is_err());
    state(&retired, a.document(), &[y], false);
    state(&retired, b.document(), &[], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 1);
    state(&adopted, a.document(), &[y, x], false);
    state(&adopted, b.document(), &[x], false);
}

#[test]
fn sandbox_and_actual_permission_denial_do_not_create_candidate_references() {
    for (sandbox, allow, focus) in [
        (
            Some(""),
            "focus-without-user-activation *",
            DomPermissionsPolicyValue::Enabled,
        ),
        (
            None,
            "focus-without-user-activation 'none'",
            DomPermissionsPolicyValue::Disabled,
        ),
    ] {
        let (mut session, top) = fixture(0);
        let (_, child) = container(
            &mut session,
            top.document(),
            "iframe",
            Some(allow),
            sandbox,
            false,
        );
        assert_eq!(
            policy(&session.snapshot(), child).inherited_focus_without_user_activation(),
            focus
        );
        let node = detached(&mut session, child, true);
        let destination = body(&session.snapshot(), child);
        edit(
            &mut session,
            DomSourceEdit::insert_before(node, destination, None),
        );
        state(&session.snapshot(), top.document(), &[], false);
        state(&session.snapshot(), child, &[], false);
        assert_eq!(session.dom().usage().autofocus_candidates(), 0);
        assert!(session.dom().pending_source_edit().is_none());
    }
}

#[test]
fn genuinely_retired_target_document_exits_before_permission_and_candidate_insertion() {
    let (mut session, top) = fixture(0);
    let before = session.snapshot();
    let destination = body(&before, top.document());
    let node = detached(&mut session, top.document(), true);
    let mut hosted = session.hosted_execution();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut completed = false;
    for _ in 0..32 {
        if matches!(hosted.run_next(&mut session.dom(),queued.traversable(),&cancel()).unwrap(),DomHostedTaskProgress::Complete{operation} if operation==queued.operation())
        {
            completed = true;
            break;
        }
    }
    assert!(completed, "actual hosted retirement must complete");
    assert!(
        !session
            .snapshot()
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    edit(
        &mut session,
        DomSourceEdit::insert_before(node, destination, None),
    );
    state(&session.snapshot(), top.document(), &[], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 0);
    assert!(
        before
            .document(top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
}

#[test]
fn candidate_capacity_refuses_atomic_insertion_and_exact_one_reference_admits() {
    for capacity in [0, 1] {
        let (mut session, top) = fixture(capacity);
        let node = detached(&mut session, top.document(), true);
        let before = session.snapshot();
        let destination = body(&before, top.document());
        let result = session.source_execution().apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(node, destination, None),
            source_limits(),
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        );
        if capacity == 0 {
            assert!(
                matches!(
                    result,
                    Err(DomSourceRuntimeError::Dom(DomError::ResourceLimit {
                        resource: DomResource::AutofocusCandidates,
                        limit: 0
                    }))
                ),
                "wrong actual capacity refusal: {result:?}"
            );
            assert_eq!(session.snapshot().stamp(), before.stamp());
            assert_eq!(session.snapshot().node(node).unwrap().parent(), None);
            assert_eq!(session.dom().usage().autofocus_candidates(), 0);
            state(&before, top.document(), &[], false);
        } else {
            assert!(matches!(
                result.unwrap(),
                DomSourceDriveProgress::Complete(_)
            ));
            state(&session.snapshot(), top.document(), &[node], false);
            assert_eq!(session.dom().usage().autofocus_candidates(), 1);
        }
    }
}

#[test]
fn unpublished_source_refusal_and_fresh_retry_never_install_partial_candidate_history() {
    let (mut session, top) = fixture(1);
    let node = detached(&mut session, top.document(), true);
    let before = session.snapshot();
    let destination = body(&before, top.document());
    let ordinary = source_limits();
    let short = DomSourceLimits::new(
        ordinary.retained_bytes(),
        ordinary.retained_string_code_units(),
        ordinary.work(),
        ordinary.phase_publications(),
        0,
        ordinary.published_native_identities(),
        ordinary.nested_operations(),
    );
    let mut source = session.source_execution();
    let result = source.apply(
        &mut session.dom(),
        DomSourceEdit::insert_before(node, destination, None),
        short,
        &mut DomSourceServices {
            execution_scope: None,
            style_policy: &mut |_| None,
            url: None,
        },
        &cancel(),
    );
    assert!(
        matches!(
            result,
            Err(DomSourceRuntimeError::Dom(DomError::SourceOutputLimit))
        ),
        "actual source refusal required: {result:?}"
    );
    // No callback, producer receipt or earlier publication escaped this
    // positive local-only insertion. Lease drop discards that unpublished
    // command; retaining an artificial pending operation would be incorrect.
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert_eq!(session.dom().usage().autofocus_candidates(), 0);
    assert_eq!(session.snapshot().node(node).unwrap().parent(), None);
    drop(source);
    let mut replacement = session.source_execution();
    assert!(matches!(
        replacement.apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(node, destination, None),
            ordinary,
            &mut DomSourceServices {
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None
            },
            &AtomicBool::new(true)
        ),
        Err(DomSourceRuntimeError::Dom(DomError::Cancelled))
    ));
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(session.snapshot().stamp(), before.stamp());
    assert!(matches!(
        replacement
            .apply(
                &mut session.dom(),
                DomSourceEdit::insert_before(node, destination, None),
                ordinary,
                &mut DomSourceServices {
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None
                },
                &cancel()
            )
            .unwrap(),
        DomSourceDriveProgress::Complete(_)
    ));
    state(&session.snapshot(), top.document(), &[node], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 1);
    assert!(session.dom().pending_source_edit().is_none());
    assert_eq!(
        session
            .element_task_execution()
            .pending_total_tasks()
            .unwrap(),
        0
    );
}

#[test]
fn dialog_same_origin_tail_clears_actual_top_history_and_processed_blocks_later_insertions() {
    let (mut session, top) = fixture(8);
    let (_, child) = container(&mut session, top.document(), "iframe", None, None, false);
    let a = detached(&mut session, top.document(), true);
    let b = detached(&mut session, child, true);
    let snapshot = session.snapshot();
    edit(
        &mut session,
        DomSourceEdit::insert_before(a, body(&snapshot, top.document()), None),
    );
    edit(
        &mut session,
        DomSourceEdit::insert_before(b, body(&snapshot, child), None),
    );
    let populated = session.snapshot();
    state(&populated, top.document(), &[a, b], false);
    state(&populated, child, &[], false);
    let subject = dialog(&mut session, child);
    for name in ["inert", "autofocus"] {
        edit(
            &mut session,
            DomSourceEdit::set_attribute(subject, name, ""),
        );
    }
    let mut calls = Vec::new();
    completed(drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    let shown = session.snapshot();
    state(&shown, top.document(), &[], true);
    state(&shown, child, &[], false);
    state(&populated, top.document(), &[a, b], false);
    assert_eq!(calls, [true]);
    assert_eq!(session.dom().usage().autofocus_candidates(), 0);
    let next = detached(&mut session, child, true);
    edit(
        &mut session,
        DomSourceEdit::insert_before(next, body(&shown, child), None),
    );
    state(&session.snapshot(), top.document(), &[], true);
    assert_eq!(session.dom().usage().autofocus_candidates(), 0);
}

#[test]
fn cross_origin_dialog_return_preserves_actual_nonempty_top_candidate_history() {
    let (mut session, top) = fixture(8);
    let (_, child) = container(
        &mut session,
        top.document(),
        "iframe",
        Some("focus-without-user-activation *"),
        Some("allow-scripts"),
        false,
    );
    assert_ne!(
        session
            .snapshot()
            .document(child)
            .unwrap()
            .metadata()
            .origin(),
        session
            .snapshot()
            .document(top.document())
            .unwrap()
            .metadata()
            .origin()
    );
    let candidate = detached(&mut session, top.document(), true);
    let parent = body(&session.snapshot(), top.document());
    edit(
        &mut session,
        DomSourceEdit::insert_before(candidate, parent, None),
    );
    let populated = session.snapshot();
    state(&populated, top.document(), &[candidate], false);
    let subject = dialog(&mut session, child);
    for name in ["inert", "autofocus"] {
        edit(
            &mut session,
            DomSourceEdit::set_attribute(subject, name, ""),
        );
    }
    let mut calls = Vec::new();
    completed(drive(
        &mut session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    state(&session.snapshot(), top.document(), &[candidate], false);
    state(&session.snapshot(), child, &[], false);
    assert_eq!(session.dom().usage().autofocus_candidates(), 1);
    assert_eq!(calls, [true]);
    state(&populated, top.document(), &[candidate], false);
}

#[test]
fn undefined_element_namespaces_do_not_trigger_autofocus_insertion() {
    for namespace in [
        Namespace::none(),
        Namespace::from_uri("urn:surgeist:undefined-elements"),
    ] {
        let (mut session, top) = fixture(0);
        let before = session.snapshot();
        assert!(
            before
                .document(top.document())
                .unwrap()
                .is_fully_active()
                .unwrap()
        );
        assert_eq!(
            policy(&before, top.document()).inherited_focus_without_user_activation(),
            DomPermissionsPolicyValue::Enabled
        );
        assert_eq!(session.dom().limits().autofocus_candidates(), 0);
        let destination = body(&before, top.document());
        let mut batch = DomBatch::new();
        let node = batch.create_element_ns(top.document(), namespace.clone(), "widget");
        batch.set_attribute(&node, "autofocus", "");
        let node = session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&node)
            .unwrap()
            .unwrap();
        let detached = session.snapshot();
        let element = detached.node(node).unwrap().as_element().unwrap();
        assert_eq!(element.name().namespace(), &namespace);
        assert_eq!(
            element.attribute_ns(&Namespace::none(), &"autofocus".into()),
            Some(&"".into())
        );
        assert_eq!(
            detached.node(node).unwrap().owner_document(),
            Some(top.document())
        );
        assert_eq!(detached.node(node).unwrap().parent(), None);
        // HTML877d raw8898–8903 excludes namespaces not defined as having the
        // global attribute. The HTML/SVG/MathML mixin (8160–8167) does not give
        // either of these ordinary Elements autofocus insertion semantics.
        edit(
            &mut session,
            DomSourceEdit::insert_before(node, destination, None),
        );
        let current = session.snapshot();
        assert_eq!(current.node(node).unwrap().parent(), Some(destination));
        assert!(current.node(node).unwrap().is_connected());
        state(&current, top.document(), &[], false);
        state(&before, top.document(), &[], false);
        assert_eq!(session.dom().usage().autofocus_candidates(), 0);
        assert!(session.dom().pending_source_edit().is_none());
        assert_eq!(
            session
                .element_task_execution()
                .pending_total_tasks()
                .unwrap(),
            0
        );
    }
}
