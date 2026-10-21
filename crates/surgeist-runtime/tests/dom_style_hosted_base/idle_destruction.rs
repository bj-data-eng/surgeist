//! An idle Style owner remains a real hosted-destruction obligation.
use super::*;
use surgeist_cssom::CssomEditGuard;

struct IdleDocument {
    session: DomSession,
    hosted: HostedRuntime,
    styles: DomStyleRuntime,
    top: DomHostedTopLevel,
    style: NodeId,
    sheet: DomStyleSheet,
    associated_dom: DomSnapshot,
    associated_cssom: surgeist_cssom::CssomSnapshot,
    idle_dom: DomSnapshot,
    idle_cssom: surgeist_cssom::CssomSnapshot,
    idle_names: DomStyleSheetSetObservation,
    capability: DomStyleCssomRoot,
    notifications: Notifications,
    dispatcher: DomEventDispatcher<Notifications>,
    dispatch_limits: DomEventDispatchLimits,
    receipt: DomEventDispatchReceipt,
    insertion: DomSourceCompletion,
    policy_calls: Vec<DomSourceOperationId>,
    settings: DomHostedSettings,
    base: surgeist_html_semantics::HtmlBaseUrlRecord,
    _initial: DomSnapshot,
    _empty_cssom: surgeist_cssom::CssomSnapshot,
    _preference_outputs: (DomStyleSheetSetCommit, DomStyleSheetSetCommit),
}

fn idle_native_document() -> IdleDocument {
    let mut session = new_session();
    let mut hosted = session.hosted_execution();
    let host = hosted.create_host(&mut session.dom(), &cancel()).unwrap();
    let top = hosted
        .create_top_level(&mut session.dom(), host, &cancel())
        .unwrap();
    let document = top.document();
    let initial = session.snapshot();
    assert!(
        initial
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    let base = initial
        .document(document)
        .unwrap()
        .document_base_record()
        .unwrap()
        .clone();
    assert_eq!(base.serialization(), &DocumentString::from("about:blank"));
    let settings = hosted
        .settings(&session.dom(), top.global(), &cancel())
        .unwrap();
    assert_eq!(settings.document().document_base_record().unwrap(), &base);

    let mut styles = session.style_execution();
    let mut input = Some(DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    styles
        .bind_root(
            &session.dom(),
            "idle-destruction".into(),
            &mut input,
            &cancel(),
        )
        .unwrap();
    assert!(input.is_none());
    let empty_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert!(empty_cssom.sheets().is_empty());
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    batch.set_attribute(&style, "title", "A");
    let text = batch.create_text(document, "p { color: red }");
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let mut policy_calls = Vec::new();
    let insertion = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, body(&initial, document), None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |source: &DomStyleSource| {
                    assert_eq!(source.element(), style);
                    assert_eq!(source.document(), document);
                    assert_eq!(source.root(), document.node());
                    assert_eq!(source.base_url(), base.serialization());
                    assert_eq!(
                        source.text().as_slice(),
                        "p { color: red }".encode_utf16().collect::<Vec<_>>()
                    );
                    policy_calls.push(source.operation());
                    Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                },
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    let DomSourceDriveProgress::Complete(insertion) = insertion else {
        panic!("real constructor-qualified Style creation must finish")
    };
    assert_eq!(policy_calls, [insertion.operation()]);
    assert!(!insertion.cancellation_requested());
    let associated_dom = session.snapshot();
    let sheet = associated_dom
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .clone();
    let associated_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert_eq!(sheet.root(), document.node());
    assert_eq!(associated_cssom.sheets(), std::slice::from_ref(sheet.id()));
    let created = associated_cssom.sheet(sheet.id()).unwrap();
    assert_eq!(created.authored().rules().len(), 1);
    assert!(created.diagnostics().is_empty());
    assert_eq!(
        created.inputs().owner.as_deref(),
        Some("idle-destruction:1")
    );
    assert_eq!(created.inputs().version.identity, "idle-destruction:1");
    assert_eq!(created.inputs().version.revision, 1);
    assert_eq!(created.inputs().title.as_slice(), &[65]);
    assert!(!created.inputs().disabled);

    let first_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(first_names.preferred_name(), &DocumentString::from("A"));
    assert_eq!(first_names.last_selected_name(), None);
    let selected = styles
        .select_sheet_set(&session.dom(), &first_names, &"A".into(), 1, &cancel())
        .unwrap();
    let preferred = styles
        .change_preferred_sheet_set_name(
            &session.dom(),
            selected.observation(),
            &"B".into(),
            1,
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        preferred.observation().preferred_name(),
        &DocumentString::from("B")
    );
    assert_eq!(
        preferred.observation().last_selected_name(),
        Some(&DocumentString::from("A"))
    );
    assert!(preferred.observation().revision() > selected.observation().revision());
    assert!(selected.observation().revision() > first_names.revision());

    let removal = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::detach(style),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(removal, DomSourceDriveProgress::Complete(_)));
    assert_eq!(policy_calls, [insertion.operation()]);
    let mut notifications = Notifications {
        styles: session.style_execution(),
        delivered: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 1,
        pending_events: 1,
        nested_dispatches: 1,
    })
    .unwrap();
    let listener = dispatcher
        .add_listener(
            &mut session.dom(),
            style,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Notifications, event| {
                context.delivered.push(event.target().unwrap());
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: true,
            },
        )
        .unwrap();
    let dispatch_limits = DomEventDispatchLimits {
        path_items: 32,
        snapshot_listeners: 8,
        failure_records: 8,
        callbacks: 8,
        work: 100_000,
    };
    let delivery = DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut notifications,
        dispatch_limits,
        &cancel(),
    )
    .unwrap();
    let DomStyleNetworkingProgress::Complete { receipt, .. } = delivery else {
        panic!("real no-import Load must dispatch and finish cleanup")
    };
    assert_eq!(receipt.event_type(), DomEventType::Load);
    assert_eq!(receipt.callbacks(), 1);
    assert!(receipt.failures().is_empty());
    assert_eq!(notifications.delivered, [style]);
    assert!(!dispatcher.remove_listener(listener));
    assert!(matches!(
        DomStyleRuntime::run_networking(
            &mut session.dom(),
            &mut dispatcher,
            &mut notifications,
            dispatch_limits,
            &cancel()
        )
        .unwrap(),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(notifications.delivered, [style]);
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(styles.pending_sheet(0).unwrap().is_none());
    let idle_dom = session.snapshot();
    assert_eq!(idle_dom.incomplete_source_operation(), None);
    assert_eq!(idle_dom.incomplete_hosted_operation(), None);
    assert!(session.dom().pending_source_edit().is_none());
    assert!(idle_dom.node(style).unwrap().parent().is_none());
    assert!(
        idle_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .is_none()
    );
    assert!(
        !idle_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .is_render_blocking()
            .unwrap()
    );
    let idle_cssom = styles.snapshot(document.node()).unwrap().unwrap();
    assert!(idle_cssom.sheets().is_empty());
    assert!(
        idle_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    let idle_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(
        idle_names.preferred_name(),
        preferred.observation().preferred_name()
    );
    assert_eq!(
        idle_names.last_selected_name(),
        preferred.observation().last_selected_name()
    );
    assert_eq!(idle_names.revision(), preferred.observation().revision());
    let capability = session
        .dom()
        .style_cssom_root(document.node())
        .unwrap()
        .unwrap();
    assert_eq!(
        capability.snapshot().unwrap().revision(),
        idle_cssom.revision()
    );

    IdleDocument {
        session,
        hosted,
        styles,
        top,
        style,
        sheet,
        associated_dom,
        associated_cssom,
        idle_dom,
        idle_cssom,
        idle_names,
        capability,
        notifications,
        dispatcher,
        dispatch_limits,
        receipt,
        insertion,
        policy_calls,
        settings,
        base,
        _initial: initial,
        _empty_cssom: empty_cssom,
        _preference_outputs: (selected, preferred),
    }
}

fn reached_cleanup(fixture: &mut IdleDocument) -> DomHostedDestructionQueued {
    let queued = fixture
        .hosted
        .destroy_top_level(
            &mut fixture.session.dom(),
            fixture.top.navigable(),
            &cancel(),
        )
        .unwrap();
    for _ in 0..256 {
        match fixture
            .hosted
            .run_next(&mut fixture.session.dom(), queued.traversable(), &cancel())
        {
            Err(DomHostedRuntimeError::Dom(DomHostedError::PendingDocumentObligations)) => {
                return queued;
            }
            Ok(DomHostedTaskProgress::Idle | DomHostedTaskProgress::Complete { .. }) => {
                panic!("retained custody must prevent destruction completion")
            }
            Ok(_) => (),
            Err(error) => panic!("must reach actual stylesheet cleanup: {error:?}"),
        }
    }
    panic!("actual destruction did not reach its custody boundary")
}

fn finish_destruction(fixture: &mut IdleDocument, queued: DomHostedDestructionQueued) {
    for _ in 0..256 {
        match fixture
            .hosted
            .run_next(&mut fixture.session.dom(), queued.traversable(), &cancel())
            .unwrap()
        {
            DomHostedTaskProgress::Complete { operation } if operation == queued.operation() => {
                return;
            }
            DomHostedTaskProgress::Idle => {
                panic!("the original destruction disappeared before completion")
            }
            _ => (),
        }
    }
    panic!("same retained destruction did not complete")
}

fn append_native_style(
    session: &mut DomSession,
    document: DocumentId,
    parent: NodeId,
    css: &str,
) -> NodeId {
    let mut batch = DomBatch::new();
    let style = batch.create_element_ns(document, Namespace::html(), "style");
    let text = batch.create_text(document, css);
    batch.insert_before(&text, &style, None);
    let style = session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&style)
        .unwrap()
        .unwrap();
    let outcome = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::insert_before(style, parent, None),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |source: &DomStyleSource| {
                    Some(DomStyleInlineCspObservation::complete_empty(source, 1))
                },
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(outcome, DomSourceDriveProgress::Complete(_)));
    style
}

fn detach_native_style(session: &mut DomSession, style: NodeId) {
    let outcome = session
        .source_execution()
        .apply(
            &mut session.dom(),
            DomSourceEdit::detach(style),
            source_limits(),
            &mut DomSourceServices {
                meta: None,
                execution_scope: None,
                style_policy: &mut |_: &DomStyleSource| None,
                url: None,
            },
            &cancel(),
        )
        .unwrap();
    assert!(matches!(outcome, DomSourceDriveProgress::Complete(_)));
}

fn independent_document_wait(
    fixture: &mut IdleDocument,
) -> (DomHostedTopLevel, NodeId, DomStyleSheet) {
    let top = fixture
        .hosted
        .create_top_level(&mut fixture.session.dom(), fixture.top.host(), &cancel())
        .unwrap();
    fixture
        .styles
        .bind_root(
            &fixture.session.dom(),
            "independent-document".into(),
            &mut Some(DomStyleCssomRoot::new(
                top.document().node(),
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
    let parent = body(&fixture.session.snapshot(), top.document());
    let style = append_native_style(
        &mut fixture.session,
        top.document(),
        parent,
        "@import 'still-required.css';",
    );
    let sheet = fixture
        .session
        .snapshot()
        .node(style)
        .unwrap()
        .as_style()
        .unwrap()
        .sheet()
        .unwrap()
        .unwrap()
        .clone();
    assert_eq!(fixture.styles.waiting_resources().unwrap(), 1);
    assert_eq!(
        fixture.styles.pending_sheet(0).unwrap(),
        Some((top.document().node(), sheet.id().clone()))
    );
    (top, style, sheet)
}

fn second_idle_root(fixture: &mut IdleDocument) -> NodeId {
    let document = fixture.top.document();
    let mut batch = DomBatch::new();
    let host = batch.create_element_ns(document, Namespace::html(), "div");
    let shadow = batch.attach_shadow(&host, DomShadowRootInit::new(HtmlShadowRootMode::Open));
    batch.insert_before(&host, body(&fixture.session.snapshot(), document), None);
    let shadow = fixture
        .session
        .dom()
        .apply(batch, &cancel())
        .unwrap()
        .created(&shadow)
        .unwrap()
        .unwrap();
    fixture
        .styles
        .bind_root(
            &fixture.session.dom(),
            "same-document-shadow".into(),
            &mut Some(DomStyleCssomRoot::new(
                shadow,
                CssomStore::new(Default::default(), Default::default()).unwrap(),
            )),
            &cancel(),
        )
        .unwrap();
    let style = append_native_style(&mut fixture.session, document, shadow, "s{color:blue}");
    detach_native_style(&mut fixture.session, style);
    deliver_load(&mut fixture.session, style);
    assert!(
        fixture
            .styles
            .snapshot(shadow)
            .unwrap()
            .unwrap()
            .sheets()
            .is_empty()
    );
    assert!(
        fixture
            .session
            .dom()
            .style_cssom_root(shadow)
            .unwrap()
            .is_some()
    );
    assert!(fixture.styles.pending_sheet(0).unwrap().is_none());
    shadow
}

fn withdraw(
    fixture: &mut IdleDocument,
    queued: DomHostedDestructionQueued,
    cancel: &AtomicBool,
) -> Result<DomStyleRootWithdrawal, DomStyleRootWithdrawalError> {
    fixture.styles.withdraw_root_for_hosted_destruction(
        &mut fixture.session.dom(),
        &fixture.hosted,
        queued.operation(),
        fixture.top.document().node(),
        cancel,
    )
}

fn assert_idle_custody(fixture: &mut IdleDocument) {
    let root = fixture.top.document().node();
    assert!(
        fixture
            .session
            .dom()
            .style_cssom_root(root)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fixture.styles.snapshot(root).unwrap().unwrap().revision(),
        fixture.idle_cssom.revision()
    );
    let names = fixture
        .styles
        .sheet_sets(&fixture.session.dom(), root, &cancel())
        .unwrap();
    assert_eq!(names.preferred_name(), fixture.idle_names.preferred_name());
    assert_eq!(
        names.last_selected_name(),
        fixture.idle_names.last_selected_name()
    );
    assert_eq!(names.revision(), fixture.idle_names.revision());
}

fn assert_released(fixture: &mut IdleDocument, cleanup: DomHostedOperationId) {
    let root = fixture.top.document().node();
    assert!(
        fixture
            .session
            .dom()
            .style_cssom_root(root)
            .unwrap()
            .is_none()
    );
    assert!(
        matches!(fixture.styles.snapshot(root), Err(DomStyleRuntimeError::RootReleasedForHostedDestruction { root: actual, cleanup: original, withdrawal_revision }) if actual == root && original == cleanup && &withdrawal_revision == fixture.idle_cssom.revision())
    );
    assert!(
        matches!(fixture.styles.sheet_sets(&fixture.session.dom(), root, &cancel()), Err(DomStyleRuntimeError::RootReleasedForHostedDestruction { root: actual, cleanup: original, withdrawal_revision }) if actual == root && original == cleanup && &withdrawal_revision == fixture.idle_cssom.revision())
    );
}

#[test]
fn withdrawal_preserves_real_history_and_escaped_store_then_resumes_the_same_destruction() {
    let mut fixture = idle_native_document();
    let queued = reached_cleanup(&mut fixture);
    let before = fixture.session.snapshot();
    let report = fixture.styles.processed_sheet(0).unwrap().unwrap();
    let outcome = withdraw(&mut fixture, queued, &cancel()).unwrap();
    let cleanup = outcome.cleanup_operation();
    assert_eq!(outcome.root(), fixture.top.document().node());
    assert_eq!(outcome.document(), fixture.top.document());
    assert_eq!(outcome.invocation(), queued.operation());
    assert_ne!(cleanup, queued.operation());
    assert_eq!(
        outcome.withdrawal_cssom_snapshot().revision(),
        fixture.idle_cssom.revision()
    );
    assert!(outcome.withdrawal_cssom_snapshot().sheets().is_empty());
    let history = outcome.document_sheet_sets().unwrap();
    assert_eq!(history.preferred(), &DocumentString::from("B"));
    assert_eq!(history.last_selected(), Some(&DocumentString::from("A")));
    assert_eq!(history.revision(), fixture.idle_names.revision());
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(fixture.top.document().node())
            .unwrap()
            .children(),
        before
            .node(fixture.top.document().node())
            .unwrap()
            .children()
    );
    assert!(
        fixture
            .session
            .snapshot()
            .node(fixture.style)
            .unwrap()
            .parent()
            .is_none()
    );
    assert_eq!(
        fixture.capability.snapshot().unwrap().revision(),
        fixture.idle_cssom.revision()
    );
    assert_eq!(fixture.notifications.delivered, [fixture.style]);
    assert_eq!(fixture.receipt.callbacks(), 1);
    assert_eq!(fixture.policy_calls, [fixture.insertion.operation()]);
    assert_eq!(report.root(), fixture.top.document().node());
    assert_eq!(report.sheet(), fixture.sheet.id());
    assert_released(&mut fixture, cleanup);
    assert!(
        matches!(fixture.styles.select_sheet_set(&fixture.session.dom(), &fixture.idle_names, &"A".into(), 1, &cancel()), Err(DomStyleRuntimeError::RootReleasedForHostedDestruction { root, cleanup: original, .. }) if root == fixture.top.document().node() && original == cleanup)
    );
    assert!(
        matches!(fixture.styles.change_preferred_sheet_set_name(&fixture.session.dom(), &fixture.idle_names, &"C".into(), 1, &cancel()), Err(DomStyleRuntimeError::RootReleasedForHostedDestruction { root, cleanup: original, .. }) if root == fixture.top.document().node() && original == cleanup)
    );
    let mut replacement = Some(DomStyleCssomRoot::new(
        fixture.top.document().node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    ));
    assert!(matches!(
        fixture.styles.bind_root(
            &fixture.session.dom(),
            "idle-destruction".into(),
            &mut replacement,
            &cancel()
        ),
        Err(DomStyleRuntimeError::Binding)
    ));
    assert!(replacement.is_some());
    assert!(
        matches!(withdraw(&mut fixture, queued, &cancel()), Err(DomStyleRootWithdrawalError::Style(DomStyleRuntimeError::RootReleasedForHostedDestruction { cleanup: original, .. })) if original == cleanup)
    );

    // A separately escaped writer still owns its real Store. The result is a
    // withdrawal-time observation, never its new current mutable authority.
    let captured = fixture.capability.snapshot().unwrap();
    assert!(
        !captured
            .sheet(fixture.sheet.id())
            .unwrap()
            .inputs()
            .disabled
    );
    let mut batch = fixture
        .capability
        .batch(CssomEditGuard::from_snapshot(&captured))
        .unwrap();
    assert!(batch.set_disabled(fixture.sheet.id(), true).unwrap());
    let changed = fixture.capability.commit(&mut Some(batch)).unwrap();
    let current = fixture.capability.snapshot().unwrap();
    assert_ne!(
        current.revision(),
        outcome.withdrawal_cssom_snapshot().revision()
    );
    assert!(current.sheet(fixture.sheet.id()).unwrap().inputs().disabled);
    assert!(
        !outcome
            .withdrawal_cssom_snapshot()
            .sheet(fixture.sheet.id())
            .unwrap()
            .inputs()
            .disabled
    );
    assert!(current.sheets().is_empty());
    assert_released(&mut fixture, cleanup);
    let disabled = fixture
        .styles
        .disabled(&fixture.session.dom(), fixture.style)
        .unwrap();
    assert_eq!(disabled.association(), None);
    assert!(!disabled.disabled());
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_eq!(
        outcome.document_sheet_sets().unwrap().revision(),
        fixture.idle_names.revision()
    );
    drop(changed);
    drop(outcome);

    // Recover/move the same pending Session without inventing another task.
    fixture.session = match fixture.session.try_shutdown() {
        DomShutdownOutcome::Pending { session, reason } => {
            assert_eq!(reason, DomShutdownReason::HostedLifecycle);
            session
        }
        other => panic!("original destruction is not acknowledged by withdrawal: {other:?}"),
    };
    fixture.styles = fixture.session.style_execution();
    fixture.hosted = fixture.session.hosted_execution();
    assert_released(&mut fixture, cleanup);
    assert_eq!(
        fixture
            .hosted
            .destroy_top_level(
                &mut fixture.session.dom(),
                fixture.top.navigable(),
                &cancel()
            )
            .unwrap(),
        queued
    );
    finish_destruction(&mut fixture, queued);
    let after = fixture.session.snapshot();
    assert!(after.node(fixture.top.document().node()).is_ok());
    assert!(
        !after
            .document(fixture.top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        fixture
            .idle_dom
            .document(fixture.top.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        fixture
            .hosted
            .top_level(&fixture.session.dom(), fixture.top.navigable(), &cancel())
            .is_err()
    );
    assert!(
        fixture
            .hosted
            .settings(&fixture.session.dom(), fixture.top.global(), &cancel())
            .is_err()
    );
    assert_eq!(
        fixture
            .associated_dom
            .node(fixture.style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        Some(&fixture.sheet)
    );
    assert!(
        fixture
            .associated_cssom
            .sheet(fixture.sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(fixture.notifications.delivered, [fixture.style]);
    assert_eq!(
        fixture.capability.snapshot().unwrap().revision(),
        current.revision()
    );
    assert_released(&mut fixture, cleanup);
    // The real Host remains a distinct lifecycle owner, even with no Top.
    assert!(matches!(
        fixture.session.try_shutdown(),
        DomShutdownOutcome::Pending {
            reason: DomShutdownReason::HostedLifecycle,
            ..
        }
    ));
}

#[test]
fn early_phase_and_cancellation_preserve_both_owners_for_the_original_cleanup_retry() {
    let mut fixture = idle_native_document();
    let queued = fixture
        .hosted
        .destroy_top_level(
            &mut fixture.session.dom(),
            fixture.top.navigable(),
            &cancel(),
        )
        .unwrap();
    let before = fixture.session.snapshot();
    assert!(matches!(
        withdraw(&mut fixture, queued, &cancel()),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::OperationPending
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_idle_custody(&mut fixture);
    assert_eq!(reached_cleanup(&mut fixture), queued);
    let before = fixture.session.snapshot();
    let stopped = AtomicBool::new(true);
    assert!(matches!(
        withdraw(&mut fixture, queued, &stopped),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::Cancelled
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_idle_custody(&mut fixture);
    assert_eq!(fixture.notifications.delivered, [fixture.style]);
    let outcome = withdraw(&mut fixture, queued, &cancel()).unwrap();
    assert_eq!(outcome.invocation(), queued.operation());
    assert_eq!(
        outcome.document_sheet_sets().unwrap().revision(),
        fixture.idle_names.revision()
    );
    finish_destruction(&mut fixture, queued);
}

#[test]
fn foreign_owner_invocation_and_wrong_document_cannot_withdraw_another_root() {
    let mut first = idle_native_document();
    let mut foreign = idle_native_document();
    let first_request = reached_cleanup(&mut first);
    let foreign_request = reached_cleanup(&mut foreign);
    let before = first.session.snapshot();
    assert!(matches!(
        first.styles.withdraw_root_for_hosted_destruction(
            &mut first.session.dom(),
            &foreign.hosted,
            first_request.operation(),
            first.top.document().node(),
            &cancel()
        ),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::Binding
        ))
    ));
    assert!(matches!(
        foreign.styles.withdraw_root_for_hosted_destruction(
            &mut first.session.dom(),
            &first.hosted,
            first_request.operation(),
            first.top.document().node(),
            &cancel()
        ),
        Err(DomStyleRootWithdrawalError::Style(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        first.styles.withdraw_root_for_hosted_destruction(
            &mut first.session.dom(),
            &first.hosted,
            foreign_request.operation(),
            first.top.document().node(),
            &cancel()
        ),
        Err(DomStyleRootWithdrawalError::Hosted(_))
    ));
    assert!(matches!(
        first.styles.withdraw_root_for_hosted_destruction(
            &mut first.session.dom(),
            &first.hosted,
            first_request.operation(),
            foreign.top.document().node(),
            &cancel()
        ),
        Err(DomStyleRootWithdrawalError::Style(_))
    ));
    assert_eq!(first.session.snapshot().stamp(), before.stamp());
    assert_idle_custody(&mut first);
    assert_idle_custody(&mut foreign);
    let outcome = withdraw(&mut first, first_request, &cancel()).unwrap();
    assert_eq!(outcome.document(), first.top.document());
    assert_idle_custody(&mut foreign);
    finish_destruction(&mut first, first_request);
}

#[test]
fn empty_list_with_a_real_import_wait_or_queued_load_is_not_idle_root_custody() {
    for (css, imports) in [("@import 'required.css';", true), ("q{color:blue}", false)] {
        let mut fixture = idle_native_document();
        let document = fixture.top.document();
        let parent = body(&fixture.session.snapshot(), document);
        let pending_style = append_native_style(&mut fixture.session, document, parent, css);
        let associated = fixture.session.snapshot();
        let sheet = associated
            .node(pending_style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap()
            .unwrap()
            .clone();
        detach_native_style(&mut fixture.session, pending_style);
        let removed = fixture.styles.snapshot(document.node()).unwrap().unwrap();
        assert!(removed.sheets().is_empty());
        assert!(removed.sheet(sheet.id()).unwrap().inputs().owner.is_none());
        assert_eq!(
            fixture.styles.waiting_resources().unwrap(),
            usize::from(imports)
        );
        assert_eq!(
            fixture.styles.pending_sheet(0).unwrap(),
            Some((document.node(), sheet.id().clone()))
        );
        let queued = reached_cleanup(&mut fixture);
        let before = fixture.session.snapshot();
        assert!(
            matches!(withdraw(&mut fixture, queued, &cancel()), Err(DomStyleRootWithdrawalError::Style(DomStyleRuntimeError::RootWorkPending { root })) if root == document.node())
        );
        assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
        assert!(
            fixture
                .session
                .dom()
                .style_cssom_root(document.node())
                .unwrap()
                .is_some()
        );
        assert_eq!(
            fixture
                .styles
                .snapshot(document.node())
                .unwrap()
                .unwrap()
                .revision(),
            removed.revision()
        );
        assert_eq!(
            fixture.styles.pending_sheet(0).unwrap(),
            Some((document.node(), sheet.id().clone()))
        );
        assert_eq!(
            fixture.styles.waiting_resources().unwrap(),
            usize::from(imports)
        );
        assert_eq!(fixture.notifications.delivered, [fixture.style]);
        assert_eq!(
            associated
                .node(pending_style)
                .unwrap()
                .as_style()
                .unwrap()
                .sheet()
                .unwrap(),
            Some(&sheet)
        );
    }
}

#[test]
fn another_registered_root_in_the_same_document_requires_its_own_real_cleanup() {
    let mut fixture = idle_native_document();
    let shadow = second_idle_root(&mut fixture);
    let shadow_cssom = fixture.styles.snapshot(shadow).unwrap().unwrap();
    let queued = reached_cleanup(&mut fixture);
    let before = fixture.session.snapshot();
    assert!(matches!(
        withdraw(&mut fixture, queued, &cancel()),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::Dom(DomHostedError::PendingDocumentObligations)
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    assert_idle_custody(&mut fixture);
    assert!(
        fixture
            .session
            .dom()
            .style_cssom_root(shadow)
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fixture.styles.snapshot(shadow).unwrap().unwrap().revision(),
        shadow_cssom.revision()
    );
    assert_eq!(fixture.styles.waiting_resources().unwrap(), 0);
    assert!(fixture.styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(
        fixture
            .hosted
            .destroy_top_level(
                &mut fixture.session.dom(),
                fixture.top.navigable(),
                &cancel()
            )
            .unwrap(),
        queued
    );
}

#[test]
fn another_documents_actual_import_wait_does_not_become_a_withdrawal_prerequisite() {
    let mut fixture = idle_native_document();
    let (other, style, sheet) = independent_document_wait(&mut fixture);
    let other_cssom = fixture
        .styles
        .snapshot(other.document().node())
        .unwrap()
        .unwrap();
    let other_names = fixture
        .styles
        .sheet_sets(&fixture.session.dom(), other.document().node(), &cancel())
        .unwrap();
    let queued = reached_cleanup(&mut fixture);
    let before = fixture.session.snapshot();
    assert!(matches!(
        fixture.styles.withdraw_root_for_hosted_destruction(
            &mut fixture.session.dom(),
            &fixture.hosted,
            queued.operation(),
            other.document().node(),
            &cancel()
        ),
        Err(DomStyleRootWithdrawalError::Hosted(
            DomHostedRuntimeError::Binding
        ))
    ));
    assert_eq!(fixture.session.snapshot().stamp(), before.stamp());
    let outcome = withdraw(&mut fixture, queued, &cancel()).unwrap();
    assert_eq!(outcome.document(), fixture.top.document());
    finish_destruction(&mut fixture, queued);
    assert!(
        fixture
            .session
            .snapshot()
            .document(other.document())
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert_eq!(fixture.styles.waiting_resources().unwrap(), 1);
    assert_eq!(
        fixture.styles.pending_sheet(0).unwrap(),
        Some((other.document().node(), sheet.id().clone()))
    );
    assert_eq!(
        fixture
            .session
            .snapshot()
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        Some(&sheet)
    );
    assert!(
        fixture
            .session
            .dom()
            .style_cssom_root(other.document().node())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        fixture
            .styles
            .snapshot(other.document().node())
            .unwrap()
            .unwrap()
            .revision(),
        other_cssom.revision()
    );
    let after_names = fixture
        .styles
        .sheet_sets(&fixture.session.dom(), other.document().node(), &cancel())
        .unwrap();
    assert_eq!(after_names.preferred_name(), other_names.preferred_name());
    assert_eq!(
        after_names.last_selected_name(),
        other_names.last_selected_name()
    );
    assert_eq!(after_names.revision(), other_names.revision());
    assert_eq!(fixture.notifications.delivered, [fixture.style]);
    assert!(matches!(
        fixture.session.try_shutdown(),
        DomShutdownOutcome::Pending {
            reason: DomShutdownReason::HostedLifecycle,
            ..
        }
    ));
}

#[test]
fn settled_native_style_retains_exact_hosted_destruction_until_root_custody_cleanup() {
    let IdleDocument {
        mut session,
        mut hosted,
        styles,
        top,
        style,
        sheet,
        associated_dom,
        associated_cssom,
        idle_dom,
        idle_cssom,
        idle_names,
        capability,
        notifications,
        receipt,
        insertion,
        policy_calls,
        settings,
        base,
        dispatcher: _dispatcher,
        dispatch_limits: _dispatch_limits,
        _initial,
        _empty_cssom,
        _preference_outputs,
    } = idle_native_document();
    let document = top.document();
    let queued = hosted
        .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
        .unwrap();
    let mut blocked = false;
    for _ in 0..256 {
        match hosted.run_next(&mut session.dom(), queued.traversable(), &cancel()) {
            Err(DomHostedRuntimeError::Dom(DomHostedError::PendingDocumentObligations)) => {
                blocked = true;
                break;
            }
            Ok(DomHostedTaskProgress::Complete { operation })
                if operation == queued.operation() =>
            {
                panic!("canonical stylesheet custody cannot be silently discarded")
            }
            Ok(DomHostedTaskProgress::Idle) => {
                panic!("the retained original destruction cannot disappear")
            }
            Ok(_) => (),
            Err(error) => panic!("wrong cleanup refusal: {error:?}"),
        }
    }
    assert!(
        blocked,
        "must reach the actual Document cleanup custody guard"
    );
    let guarded = session.snapshot();
    assert!(guarded.node(document.node()).is_ok());
    assert_eq!(
        guarded.document(document).unwrap().navigable(),
        Some(top.navigable())
    );
    assert!(
        guarded
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    assert!(
        idle_dom
            .document(document)
            .unwrap()
            .is_fully_active()
            .unwrap()
    );
    // This parallel destruction retains its own operation/task, rather than
    // publishing the creation protocol's global incomplete-hosted marker.
    assert_eq!(guarded.incomplete_hosted_operation(), None);
    assert_eq!(
        hosted
            .destroy_top_level(&mut session.dom(), top.navigable(), &cancel())
            .unwrap(),
        queued
    );
    assert!(matches!(
        hosted.run_next(&mut session.dom(), queued.traversable(), &cancel()),
        Err(DomHostedRuntimeError::Dom(
            DomHostedError::PendingDocumentObligations
        ))
    ));
    assert_eq!(session.snapshot().incomplete_hosted_operation(), None);
    assert_eq!(session.snapshot().stamp(), guarded.stamp());
    assert!(
        session
            .dom()
            .style_cssom_root(document.node())
            .unwrap()
            .is_some()
    );
    assert_eq!(
        capability.snapshot().unwrap().revision(),
        idle_cssom.revision()
    );
    assert_eq!(
        styles
            .snapshot(document.node())
            .unwrap()
            .unwrap()
            .revision(),
        idle_cssom.revision()
    );
    assert_eq!(styles.waiting_resources().unwrap(), 0);
    assert!(styles.pending_sheet(0).unwrap().is_none());
    assert_eq!(notifications.delivered, [style]);
    assert_eq!(policy_calls, [insertion.operation()]);
    let pending_names = styles
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap();
    assert_eq!(pending_names.preferred_name(), idle_names.preferred_name());
    assert_eq!(
        pending_names.last_selected_name(),
        idle_names.last_selected_name()
    );
    assert_eq!(pending_names.revision(), idle_names.revision());
    assert_eq!(receipt.callbacks(), 1);
    assert!(
        associated_cssom
            .sheet(sheet.id())
            .unwrap()
            .inputs()
            .owner
            .is_some()
    );
    assert_eq!(
        associated_dom
            .node(style)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .unwrap(),
        Some(&sheet)
    );
    assert_eq!(settings.document().document_base_record().unwrap(), &base);
    assert_eq!(idle_names.preferred_name(), &DocumentString::from("B"));
    assert_eq!(
        idle_names.last_selected_name(),
        Some(&DocumentString::from("A"))
    );
}
