//! Actual descendant queries retain their original Source leaf without supplied focus facts.
use super::*;
use surgeist_html::{HtmlAutofocusDelegateAction, HtmlFocusDelegateAction};

fn edit(session: &mut DomSession, edit: DomSourceEdit) {
    let result = session
        .source_execution()
        .apply(
            &mut session.dom(),
            edit,
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(
        matches!(result, DomSourceDriveProgress::Complete(_)),
        "real setup: {result:?}"
    );
}
fn pending(progress: DomSourceDriveProgress) -> DomSourceFocusDelegateRequest {
    match progress {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::FocusDelegate(request),
            reason: DomSourceUnavailable::FocusDelegate(step),
        } => {
            assert_eq!(step, request.step());
            request
        }
        other => panic!("actual unsupported delegate query required: {other:?}"),
    }
}
fn stable_query(
    session: &mut DomSession,
    top: DomHostedTopLevel,
    subject: NodeId,
    query: NodeId,
    expected: DomSourceFocusDelegateStep,
    ready: &DomSnapshot,
) {
    let mut calls = Vec::new();
    let original = pending(drive(
        session,
        DomSourceEdit::show_modal_dialog(subject),
        true,
        &mut calls,
    ));
    assert_eq!(calls, [true]);
    assert_eq!(original.subject(), subject);
    assert_eq!(original.root(), subject);
    assert_eq!(original.queried_node(), query);
    assert_eq!(original.step(), expected);
    assert_eq!(original.command(), 0, "one original ShowModal command");
    let status = session.dom().pending_source_edit().unwrap();
    assert_eq!(status.operation(), original.operation());
    assert_eq!(status.phase(), DomSourcePhase::DialogService);
    assert_eq!(status.cleanup_cause(), None);
    assert!(status.has_published());
    let prefix = session.snapshot();
    assert_eq!(
        prefix.incomplete_source_operation(),
        Some(original.operation())
    );
    let node = prefix.node(subject).unwrap();
    assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"inert".into()), Some(&"".into()));
    assert_eq!(node.attribute(&"autofocus".into()), None);
    let dialog = node.as_dialog().unwrap();
    assert!(dialog.is_modal());
    let watcher = dialog.close_watcher().unwrap();
    assert_eq!(watcher.global(), top.global());
    assert_eq!(watcher.subject(), subject);
    assert_eq!(
        node.as_element()
            .unwrap()
            .common_state()
            .html()
            .unwrap()
            .previously_focused_element(),
        None
    );
    assert!(matches!(
        prefix.document(top.document()).unwrap().top_layer(),
        Err(DomError::IncompleteSourceOperation)
    ));
    assert!(matches!(
        prefix.document(top.document()).unwrap().autofocus(),
        Err(DomError::IncompleteSourceOperation)
    ));
    let tasks = session.element_task_execution();
    let queued = tasks
        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
        .unwrap()
        .unwrap();
    let toggle = queued.dialog_toggle().unwrap();
    assert_eq!(toggle.id().target(), subject);
    assert_eq!(
        toggle.queue_association().unwrap().document(),
        top.document()
    );
    assert_eq!(toggle.queue_association().unwrap().global(), top.global());
    assert_eq!(dialog.toggle_task_tracker().unwrap().task(), toggle.id());
    let retained_occurrence = original.current_occurrence();
    for _ in 0..2 {
        // Reissue a real execution permission. It owns neither the retained
        // delegate cursor nor the already published watcher/queue prefix.
        let mut source = session.source_execution();
        let next = pending(
            source
                .resume(
                    &mut session.dom(),
                    original.operation(),
                    source_limits(),
                    &mut DomSourceServices {
                        meta: None,
                        execution_scope: None,
                        style_policy: &mut |_| None,
                        url: None,
                    },
                    &cancel(),
                )
                .unwrap(),
        );
        assert_eq!(next, original);
        assert_eq!(next.command(), 0);
        assert_eq!(next.current_occurrence(), retained_occurrence);
        drop(source);
        assert_eq!(session.snapshot().stamp(), prefix.stamp());
        assert_eq!(
            session
                .dom()
                .pending_source_edit()
                .unwrap()
                .published_phases(),
            status.published_phases()
        );
        assert_eq!(tasks.pending_total_tasks().unwrap(), 1);
        assert_eq!(
            tasks
                .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap(),
            Some(queued)
        );
        assert_eq!(
            session
                .snapshot()
                .node(subject)
                .unwrap()
                .as_dialog()
                .unwrap()
                .close_watcher(),
            Some(watcher)
        );
    }
    assert_eq!(
        calls,
        [true],
        "reaching the same leaf cannot replay beforetoggle"
    );
    assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
    assert!(!ready.node(subject).unwrap().as_dialog().unwrap().is_modal());
    assert!(
        !ready
            .node(subject)
            .unwrap()
            .as_dialog()
            .unwrap()
            .has_close_watcher()
    );
    assert!(
        ready
            .document(top.document())
            .unwrap()
            .top_layer()
            .unwrap()
            .raw_members()
            .is_empty()
    );
    let history = ready.document(top.document()).unwrap().autofocus().unwrap();
    assert!(history.candidates().is_empty());
    assert!(!history.processed());
    assert_eq!(
        policy(ready, top.document()).inherited_focus_without_user_activation(),
        DomPermissionsPolicyValue::Enabled
    );
}

#[test]
fn ordinary_descendant_queries_keep_the_exact_focus_or_autofocus_leaf_across_resume() {
    for autofocus in [false, true] {
        let (mut session, top) = headless_top();
        let subject = dialog(&mut session, top.document());
        edit(
            &mut session,
            DomSourceEdit::set_attribute(subject, "inert", ""),
        );
        let mut batch = DomBatch::new();
        let child = batch.create_element_ns(top.document(), Namespace::html(), "span");
        let child = session
            .dom()
            .apply(batch, &cancel())
            .unwrap()
            .created(&child)
            .unwrap()
            .unwrap();
        edit(
            &mut session,
            DomSourceEdit::insert_before(child, subject, None),
        );
        if autofocus {
            edit(
                &mut session,
                DomSourceEdit::set_attribute(child, "autofocus", ""),
            );
        }
        let ready = session.snapshot();
        assert_eq!(ready.node(subject).unwrap().children(), [child]);
        assert_eq!(ready.node(child).unwrap().parent(), Some(subject));
        assert_eq!(ready.node(subject).unwrap().hosted_shadow_root(), None);
        assert_eq!(
            ready
                .node(child)
                .unwrap()
                .attribute(&"autofocus".into())
                .is_some(),
            autofocus
        );
        // With no autofocus descendant the first pass returns null, but the
        // Dialog's second pass still needs actual sequential focusability.
        // With autofocus, the first pass needs the child's actual direct area.
        // Own inert supplies neither fact and cannot skip either query.
        let expected = if autofocus {
            DomSourceFocusDelegateStep::Autofocus(HtmlAutofocusDelegateAction::ReadFocusableArea)
        } else {
            DomSourceFocusDelegateStep::Focus(HtmlFocusDelegateAction::ReadSequentiallyFocusable)
        };
        stable_query(&mut session, top, subject, child, expected, &ready);
    }
}

#[test]
fn autofocus_in_shadow_descendants_does_not_enter_the_dialog_ordinary_passes() {
    let (mut session, top) = headless_top();
    let subject = dialog(&mut session, top.document());
    edit(
        &mut session,
        DomSourceEdit::set_attribute(subject, "inert", ""),
    );
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(top.document(), Namespace::html(), "div");
    let host = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&host)
        .unwrap()
        .unwrap();
    edit(
        &mut session,
        DomSourceEdit::insert_before(host, subject, None),
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
    let mut batch = DomBatch::new();
    let shadow_child = batch.create_element_ns(top.document(), Namespace::html(), "span");
    let shadow_child = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&shadow_child)
        .unwrap()
        .unwrap();
    edit(
        &mut session,
        DomSourceEdit::insert_before(shadow_child, root, None),
    );
    edit(
        &mut session,
        DomSourceEdit::set_attribute(shadow_child, "autofocus", ""),
    );
    let ready = session.snapshot();
    assert_eq!(ready.node(subject).unwrap().children(), [host]);
    assert!(ready.node(host).unwrap().children().is_empty());
    assert_eq!(ready.node(host).unwrap().hosted_shadow_root(), Some(root));
    assert_eq!(ready.node(root).unwrap().children(), [shadow_child]);
    assert_eq!(ready.shadow_root(root).unwrap().host(), host);
    assert_eq!(
        ready
            .node(shadow_child)
            .unwrap()
            .attribute(&"autofocus".into()),
        Some(&"".into())
    );
    // Ordinary autofocus iteration cannot see the shadow child. The fallback
    // iteration stops at the actual host's unknown sequential focusability;
    // no implicit shadow traversal or recursive get-area result is supplied.
    stable_query(
        &mut session,
        top,
        subject,
        host,
        DomSourceFocusDelegateStep::Focus(HtmlFocusDelegateAction::ReadSequentiallyFocusable),
        &ready,
    );
}

fn output_policy(outputs: usize) -> DomSourceLimits {
    let ordinary = source_limits();
    DomSourceLimits::new(
        ordinary.retained_bytes(),
        ordinary.retained_string_code_units(),
        ordinary.work(),
        5,
        outputs,
        ordinary.published_native_identities(),
        ordinary.nested_operations(),
    )
}
fn before_with_policy(
    session: &mut DomSession,
    subject: NodeId,
    limits: DomSourceLimits,
) -> (DomSourceEventRequest, usize) {
    let before = match session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::show_modal_dialog(subject),
            limits,
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap()
    {
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(event),
            reason: DomSourceUnavailable::Event,
        } => event,
        other => panic!("real beforetoggle precedes queue preadmission: {other:?}"),
    };
    assert!(before.cancelable());
    let mut registry = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let listener = registry
        .add_listener(
            &mut session.dom(),
            subject,
            DomEventType::BeforeToggle,
            DomEventCallback::new(|_, _, calls: &mut usize, event| {
                assert!(event.cancelable());
                assert_eq!(event.toggle_event().unwrap().source(), None);
                *calls += 1;
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let mut calls = 0;
    let result = session
        .source_execution()
        .service_event(
            &mut session.dom(),
            &before,
            &mut registry,
            &mut calls,
            DomSourceEventServiceLimits {
                source: limits,
                dispatch: dispatch_limits(),
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceEventServiceProgress::Serviced { receipt } = result else {
        panic!("actual event receipt: {result:?}")
    };
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert!(!receipt.default_prevented());
    assert!(!registry.remove_listener(listener));
    assert_eq!(calls, 1);
    (before, calls)
}

#[test]
fn empty_delegate_tail_preadmits_only_actual_changed_same_origin_autofocus_output_before_queue() {
    for case in 0..3 {
        for short in [true, false] {
            let (mut session, top) = headless_top();
            // Case1 establishes processed=true by an actual earlier complete
            // Dialog method. Its watcher, raw member and Toggle stay owned.
            let seed = if case == 1 {
                let seed = dialog(&mut session, top.document());
                for name in ["inert", "autofocus"] {
                    edit(&mut session, DomSourceEdit::set_attribute(seed, name, ""));
                }
                let mut calls = Vec::new();
                let result = drive(
                    &mut session,
                    DomSourceEdit::show_modal_dialog(seed),
                    true,
                    &mut calls,
                );
                assert!(matches!(result, DomSourceDriveProgress::Complete(_)));
                assert_eq!(calls, [true]);
                Some(seed)
            } else {
                None
            };
            let document = if case == 2 {
                container(
                    &mut session,
                    top.document(),
                    "iframe",
                    Some("focus-without-user-activation *"),
                    Some(""),
                    false,
                )
                .1
            } else {
                top.document()
            };
            let subject = dialog(&mut session, document);
            edit(
                &mut session,
                DomSourceEdit::set_attribute(subject, "inert", ""),
            );
            let ready = session.snapshot();
            let node = ready.node(subject).unwrap();
            assert!(node.children().is_empty());
            assert_eq!(node.hosted_shadow_root(), None);
            assert_eq!(node.attribute(&"autofocus".into()), None);
            assert_eq!(node.attribute(&"inert".into()), Some(&"".into()));
            assert_eq!(
                policy(&ready, document).inherited_focus_without_user_activation(),
                DomPermissionsPolicyValue::Enabled
            );
            assert_eq!(
                ready.document(document).unwrap().metadata().origin()
                    == ready.document(top.document()).unwrap().metadata().origin(),
                case != 2
            );
            let initial = ready.document(top.document()).unwrap().autofocus().unwrap();
            assert!(initial.candidates().is_empty());
            assert_eq!(initial.processed(), case == 1);
            let tasks = session.element_task_execution();
            let baseline = tasks
                .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                .unwrap();
            assert_eq!(
                tasks.pending_total_tasks().unwrap(),
                usize::from(seed.is_some())
            );
            if let Some(seed) = seed {
                assert_eq!(baseline.unwrap().id().target(), seed);
                assert_eq!(
                    ready
                        .document(top.document())
                        .unwrap()
                        .top_layer()
                        .unwrap()
                        .raw_members(),
                    [seed]
                );
            }
            // Queue tracker+marker2, AddOpen attribute+Document2, watcher1,
            // modal+raw-layer2, final marker1 = eight actual output rows.
            // Only dirty same-origin history adds DocumentAutofocusState1.
            // Remember(None) is unchanged; the already-empty candidate list
            // has no second row. The existing five phases do not grow.
            const BASE_OUTPUTS: usize = 2 + 2 + 1 + 2 + 1;
            let exact = BASE_OUTPUTS + usize::from(case == 0);
            let (before, calls) = before_with_policy(
                &mut session,
                subject,
                output_policy(exact - usize::from(short)),
            );
            let stamp = session.snapshot().stamp();
            let mut source = session.source_execution();
            let result = source.resume(
                &mut session.dom(),
                before.operation(),
                output_policy(exact - usize::from(short)),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut |_| None,
                    url: None,
                },
                &cancel(),
            );
            let complete = if short {
                assert!(
                    matches!(
                        result,
                        Err(surgeist_runtime::DomSourceDriveError::Source(
                            DomSourceRuntimeError::ElementTasks(DomElementTaskRuntimeError::Dom(
                                DomError::SourceOutputLimit
                            ))
                        ))
                    ),
                    "whole future output must refuse before QueueToggle publication: {result:?}"
                );
                let refused = session.snapshot();
                assert_eq!(refused.stamp(), stamp);
                let node = refused.node(subject).unwrap();
                assert_eq!(node.attribute(&"open".into()), None);
                let dialog = node.as_dialog().unwrap();
                assert!(!dialog.is_modal());
                assert!(!dialog.has_close_watcher());
                assert_eq!(dialog.toggle_task_tracker(), None);
                assert_eq!(
                    tasks.pending_total_tasks().unwrap(),
                    usize::from(seed.is_some())
                );
                assert_eq!(
                    tasks
                        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                        .unwrap(),
                    baseline
                );
                assert_eq!(
                    session.dom().pending_source_edit().unwrap().operation(),
                    before.operation()
                );
                source
                    .resume(
                        &mut session.dom(),
                        before.operation(),
                        output_policy(exact),
                        &mut DomSourceServices {
                            meta: None,
                            execution_scope: None,
                            style_policy: &mut |_| None,
                            url: None,
                        },
                        &cancel(),
                    )
                    .unwrap()
            } else {
                result.unwrap()
            };
            let DomSourceDriveProgress::Complete(done) = complete else {
                panic!("literal exact output allowance: {complete:?}")
            };
            assert_eq!(done.operation(), before.operation());
            assert!(!done.cancellation_requested());
            assert_eq!(calls, 1);
            assert!(session.dom().pending_source_edit().is_none());
            let shown = session.snapshot();
            assert_eq!(shown.incomplete_source_operation(), None);
            let final_history = shown.document(top.document()).unwrap().autofocus().unwrap();
            assert!(final_history.candidates().is_empty());
            assert_eq!(final_history.processed(), case != 2);
            assert_eq!(
                initial.processed(),
                case == 1,
                "historical history remains unchanged"
            );
            let node = shown.node(subject).unwrap();
            assert_eq!(node.attribute(&"open".into()), Some(&"".into()));
            let dialog = node.as_dialog().unwrap();
            assert!(dialog.is_modal());
            assert_eq!(
                dialog.close_watcher().unwrap().global(),
                shown.relevant_global(subject).unwrap().unwrap()
            );
            assert_eq!(
                tasks.pending_total_tasks().unwrap(),
                usize::from(seed.is_some()) + 1
            );
            let task = tasks
                .pending_task(
                    DomElementTaskSourceKind::DomManipulation,
                    usize::from(seed.is_some()),
                )
                .unwrap()
                .unwrap();
            assert_eq!(dialog.toggle_task_tracker().unwrap().task(), task.id());
            assert_eq!(task.queue_association().unwrap().document(), document);
            if let Some(baseline) = baseline {
                assert_eq!(
                    tasks
                        .pending_task(DomElementTaskSourceKind::DomManipulation, 0)
                        .unwrap(),
                    Some(baseline)
                );
            }
            let expected = seed.into_iter().chain([subject]).collect::<Vec<_>>();
            let layer = shown.document(document).unwrap().top_layer().unwrap();
            assert_eq!(layer.raw_members(), expected);
            assert!(layer.pending_removals().is_empty());
            assert_eq!(ready.node(subject).unwrap().attribute(&"open".into()), None);
        }
    }
}
