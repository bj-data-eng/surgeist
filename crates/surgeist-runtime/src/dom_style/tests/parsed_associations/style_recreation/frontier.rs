//! Real sealed-end execution is independent of retained networking/import work.
use super::*;

fn end(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
) -> Result<DomParsedHtmlSynchronousEnd, DomStyleParsedFrontierError> {
    session
        .style_execution()
        .complete_parsed_html_synchronous_execution(&mut session.dom(), id, envelope(), &cancel())
}
fn successor(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInitialPolicyRequest {
    let output = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    output.qualification().clone()
}
fn queue(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleInitialPolicyRequest,
) -> DomStyleNetworkingTaskId {
    match session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    {
        DomStyleParsedNotificationProgress::Queued { task } => task,
        other => panic!("NoImports must genuinely queue: {other:?}"),
    }
}
fn networking(session: &mut DomSession) -> DomStyleNetworkingProgress {
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    DomStyleRuntime::run_networking(
        &mut session.dom(),
        &mut dispatcher,
        &mut context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
}

#[test]
fn real_end_releases_old_and_recreated_tasks_in_fifo_without_repeating_processing() {
    let (mut session, installed, old, old_sheet) =
        created_first("<!doctype html><style title=chosen>p{color:red}</style>");
    let old_task = queue(&mut session, &old);
    assert!(
        matches!(networking(&mut session), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == old_task)
    );
    let q = successor(&mut session, old.contribution());
    decide(&mut session, &q, true);
    let created = create(&mut session, q.contribution(), envelope()).unwrap();
    let new_sheet = created.completion().sheet().clone();
    drop(created);
    finish_rendering(&mut session, &q);
    let new_task = queue(&mut session, &q);
    assert_ne!(old_sheet.id(), new_sheet.id());
    assert!(
        matches!(networking(&mut session), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == old_task)
    );
    assert!(
        session
            .dom
            .parsed_html_synchronous_end(q.contribution())
            .unwrap()
            .is_none()
    );
    let names = session
        .style_execution()
        .sheet_sets(&session.dom(), installed.document().node(), &cancel())
        .unwrap();
    let serial = session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node == installed.document().node())
        .unwrap()
        .serial;
    let proof = end(&mut session, q.contribution()).unwrap();
    assert_eq!(proof.contribution(), q.contribution());
    drop(proof);
    let DomShutdownOutcome::Pending {
        session: recovered,
        reason,
    } = session.try_shutdown()
    else {
        panic!("whole original custody remains pending")
    };
    assert_eq!(
        reason,
        DomShutdownReason::ParsedAssociations(q.contribution())
    );
    session = recovered;
    assert_eq!(
        session
            .dom
            .parsed_html_synchronous_end(q.contribution())
            .unwrap()
            .unwrap()
            .contribution(),
        q.contribution()
    );
    for expected in [old_task, new_task] {
        let DomStyleNetworkingProgress::Complete { task, receipt } = networking(&mut session)
        else {
            panic!("actual end permits real delivery")
        };
        assert_eq!(task, expected);
        assert_eq!(receipt.event_type(), DomEventType::Load);
        assert_eq!(receipt.original_target(), new_sheet.association().element());
    }
    assert!(matches!(
        networking(&mut session),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(session.state.borrow().next_task, 3);
    assert_eq!(
        session
            .state
            .borrow()
            .roots
            .iter()
            .find(|root| root.node == installed.document().node())
            .unwrap()
            .serial,
        serial
    );
    let after = session
        .style_execution()
        .sheet_sets(&session.dom(), installed.document().node(), &cancel())
        .unwrap();
    assert_eq!(after.preferred_name(), names.preferred_name());
    assert_eq!(after.last_selected_name(), names.last_selected_name());
    pending(&session, q.contribution());
}

#[test]
fn blocked_first_and_allowed_or_blocked_pop_authenticate_end_without_fake_sheet() {
    for allow in [false, true] {
        let (policy, profile) = text_policy::policy(allow);
        let (mut session, installed, old, _) = blocked_first(
            "<!doctype html><style>p{color:red}</style>",
            policy,
            &profile,
        );
        assert!(matches!(
            end(&mut session, old.contribution()),
            Err(DomStyleParsedFrontierError::Frontier(
                DomParsedHtmlFrontierError::Remaining(_)
            ))
        ));
        let q = successor(&mut session, old.contribution());
        let mut input = observe(&session, q.contribution(), &mut |request| {
            Some(text_policy::decision(request, &profile))
        });
        assert_eq!(input.as_ref().unwrap().allowed(), allow);
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                q.contribution(),
                &mut input,
                envelope(),
                &cancel(),
            )
            .unwrap();
        if allow {
            bind(&mut session, installed.document(), Default::default());
            drop(create(&mut session, q.contribution(), envelope()).unwrap());
            finish_rendering(&mut session, &q);
        } else {
            assert!(
                session
                    .style_execution()
                    .snapshot(installed.document().node())
                    .unwrap()
                    .is_none()
            );
        }
        drop(end(&mut session, q.contribution()).unwrap());
        assert!(
            session
                .dom
                .parsed_html_synchronous_end(q.contribution())
                .unwrap()
                .is_some()
        );
        pending(&session, q.contribution());
    }
}

#[test]
fn actual_import_processing_remains_pending_after_genuine_synchronous_end() {
    let (mut session, installed, old, _) =
        created_first("<!doctype html><style>@import 'theme.css'; p{color:red}</style>");
    let q = successor(&mut session, old.contribution());
    decide(&mut session, &q, true);
    drop(create(&mut session, q.contribution(), envelope()).unwrap());
    finish_rendering(&mut session, &q);
    let before = session.state.borrow().next_task;
    assert!(matches!(
        session
            .style_execution()
            .queue_parsed_style_notification(&mut session.dom(), &q, envelope(), &cancel())
            .unwrap(),
        DomStyleParsedNotificationProgress::PendingImports { .. }
    ));
    drop(end(&mut session, q.contribution()).unwrap());
    assert_eq!(session.state.borrow().next_task, before);
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    assert_eq!(
        session.state.borrow().completions[1]
            .processing()
            .unwrap()
            .dependencies,
        ImportDependencies::Imports
    );
    assert_eq!(installed.pending_association(), Some(q.contribution()));
    pending(&session, q.contribution());
}

#[test]
fn later_meta_is_exact_remaining_and_end_never_uses_graph_rendered_alone() {
    let (mut session, _, old, _) = created_first(
        "<!doctype html><style>p{color:red}</style><meta http-equiv=default-style content=later>",
    );
    let q = successor(&mut session, old.contribution());
    decide(&mut session, &q, true);
    drop(create(&mut session, q.contribution(), envelope()).unwrap());
    finish_rendering(&mut session, &q);
    assert!(matches!(
        end(&mut session, q.contribution()),
        Err(DomStyleParsedFrontierError::Frontier(
            DomParsedHtmlFrontierError::Remaining(
                DomParsedHtmlStyleNotificationRequirement::Occurrence {
                    kind: Kind::ChangePreferred,
                    ..
                }
            )
        ))
    ));
    assert!(
        session
            .dom
            .parsed_html_synchronous_end(q.contribution())
            .unwrap()
            .is_none()
    );
}

#[test]
fn end_cancellation_runtime_refusal_and_raised_retry_preserve_actual_execution() {
    let (mut session, _, old, _) = created_first("<!doctype html><style>p{color:red}</style>");
    let q = successor(&mut session, old.contribution());
    decide(&mut session, &q, true);
    drop(create(&mut session, q.contribution(), envelope()).unwrap());
    finish_rendering(&mut session, &q);
    let mut runtime = session.style_execution();
    assert!(
        runtime
            .complete_parsed_html_synchronous_execution(
                &mut session.dom(),
                q.contribution(),
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    let stamp = session.snapshot().stamp();
    let work = session.state.borrow().work.get();
    let mut small = limits();
    small.storage_bytes = session
        .state
        .borrow()
        .storage(None, &cancel(), small.work)
        .unwrap();
    runtime
        .set_limits(&session.dom(), small, &cancel())
        .unwrap();
    assert!(matches!(
        end(&mut session, q.contribution()),
        Err(DomStyleParsedFrontierError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(
        session
            .dom
            .parsed_html_synchronous_end(q.contribution())
            .unwrap()
            .is_none()
    );
    assert_eq!(session.snapshot().stamp(), stamp);
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    drop(end(&mut session, q.contribution()).unwrap());
    let spent = session.state.borrow().work.get();
    assert!(spent > work);
    drop(end(&mut session, q.contribution()).unwrap());
    assert!(session.state.borrow().work.get() > spent);
    assert_eq!(session.snapshot().stamp(), stamp);
}
