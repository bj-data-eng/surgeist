//! Independent original-order, CSSOM selection and durable notification oracles.
use super::*;
use surgeist_cssom::{CssomError, CssomLimits, CssomSnapshot};
use surgeist_html_parser::HtmlParseAssociationOccurrenceId as Occurrence;

const SOURCE: &str = "<!doctype html><style title=A>p{color:red}</style><meta http-equiv=default-style content=B><style title=C>c{color:blue}</style><style title=B>d{color:green}</style>";

fn rows(session: &DomSession, id: DomParsedHtmlAssociationId) -> (Vec<Occurrence>, Occurrence) {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == id)
        .unwrap();
    let mut cursor = entry.report().pending_associations();
    let mut styles = Vec::new();
    let mut meta = None;
    loop {
        match cursor.advance() {
            Step::Occurrence(row) if row.requires_processing() => match row.kind() {
                Kind::StyleUpdate(trigger) => {
                    assert_eq!(
                        trigger == HtmlStyleTrigger::ParserPopped,
                        styles.len() % 2 == 1,
                        "each literal Style has its real insertion followed by ParserPopped"
                    );
                    styles.push(row.id());
                }
                Kind::ChangePreferred => {
                    assert!(meta.replace(row.id()).is_none());
                    assert_eq!(styles.len(), 2, "Meta follows the completed A history");
                }
                other => panic!("unexpected influencing fixture operation: {other:?}"),
            },
            Step::Complete => return (styles, meta.unwrap()),
            _ => (),
        }
    }
}
fn current(
    session: &DomSession,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleQualification {
    session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone()
}
fn snapshot(session: &DomSession, document: DocumentId) -> CssomSnapshot {
    session
        .style_execution()
        .snapshot(document.node())
        .unwrap()
        .unwrap()
}
fn names(session: &mut DomSession, document: DocumentId) -> DomStyleSheetSetObservation {
    session
        .style_execution()
        .sheet_sets(&session.dom(), document.node(), &cancel())
        .unwrap()
}
fn allow(session: &mut DomSession, q: &DomParsedHtmlStyleQualification, expected_text: &str) {
    let mut input = observe(session, q.contribution(), &mut |request| {
        assert_eq!(request.qualification(), q);
        assert_eq!(request.text(), &DocumentString::from(expected_text));
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                q.contribution(),
                &mut input,
                envelope(),
                &cancel()
            )
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(input.is_none());
}
fn created(session: &mut DomSession, q: &DomParsedHtmlStyleQualification) -> DomStyleSheet {
    let output = create(session, q.contribution(), envelope()).unwrap();
    let sheet = output.completion().sheet().clone();
    drop(output);
    finish_rendering(session, q);
    sheet
}
fn first(
    source: &str,
    producer: CssomLimits,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
) {
    let (mut session, installation) = ready(source, envelope());
    let id = installation.pending_association().unwrap();
    bind(&mut session, installation.document(), producer);
    let q = current(&session, id);
    let sheet = created(&mut session, &q);
    (session, installation, q, sheet)
}
fn pop(
    session: &mut DomSession,
    old: &DomParsedHtmlStyleQualification,
    expected: &Occurrence,
    text: &str,
) -> (DomParsedHtmlStyleQualification, DomStyleSheet) {
    let output = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        output.removal(),
        DomParsedHtmlStyleSuccessorRemoval::Removed { .. }
    ));
    let q = output.qualification().clone();
    assert_eq!(q.occurrence(), expected);
    assert_eq!(q.subject(), old.subject());
    drop(output);
    allow(session, &q, text);
    let sheet = created(session, &q);
    (q, sheet)
}
fn meta(session: &mut DomSession, id: DomParsedHtmlAssociationId, expected: &Occurrence) {
    let output = session
        .style_execution()
        .consume_parsed_meta_occurrence(&mut session.dom(), id, expected, envelope(), &cancel())
        .unwrap();
    assert_eq!(output.completion().occurrence().id(), expected);
    assert_eq!(
        output.preference().preferred_name(),
        &DocumentString::from("B")
    );
}
fn next(
    session: &mut DomSession,
    id: DomParsedHtmlAssociationId,
    expected: &Occurrence,
) -> DomParsedHtmlStyleQualification {
    let output = session
        .style_execution()
        .advance_parsed_style_subject(&mut session.dom(), id, expected, envelope(), &cancel())
        .unwrap();
    assert_eq!(output.phase(), DomParsedHtmlStylePhase::InlinePolicyPending);
    let q = output.qualification().clone();
    assert_eq!(q.occurrence(), expected);
    drop(output);
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert!(view.policy_ready());
    assert!(view.initial_policy_request().is_none());
    q
}
fn queue(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> DomStyleNetworkingTaskId {
    match session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel())
        .unwrap()
    {
        DomStyleParsedNotificationProgress::Queued { task } => task,
        other => panic!("genuine live NoImports invocation: {other:?}"),
    }
}
fn owner_removed(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleQualification,
    sheet: &DomStyleSheet,
) {
    let before = session.state.borrow().next_task;
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(
        &mut session.dom(), q, envelope(), &cancel()
    ).unwrap(), DomStyleParsedNotificationProgress::OwnerAssociationRemoved { association } if association == sheet.association())
    );
    assert_eq!(session.state.borrow().next_task, before);
}
fn end(session: &mut DomSession, id: DomParsedHtmlAssociationId) {
    let proof = session
        .style_execution()
        .complete_parsed_html_synchronous_execution(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert_eq!(proof.contribution(), id);
}
fn events(session: &DomSession) -> (DomEventDispatcher<Context>, Context) {
    (
        DomEventDispatcher::new(DomEventRegistryLimits {
            registrations: 16,
            pending_events: 16,
            nested_dispatches: 8,
        })
        .unwrap(),
        Context {
            styles: session.style_execution(),
            source: session.source_execution(),
            events: Vec::new(),
        },
    )
}
fn run(
    session: &mut DomSession,
    dispatcher: &mut DomEventDispatcher<Context>,
    context: &mut Context,
) -> DomStyleNetworkingProgress {
    DomStyleRuntime::run_networking(
        &mut session.dom(),
        dispatcher,
        context,
        dispatch_limits(),
        &cancel(),
    )
    .unwrap()
}
fn listen(session: &mut DomSession, dispatcher: &mut DomEventDispatcher<Context>, target: NodeId) {
    dispatcher
        .add_listener(
            &mut session.dom(),
            target,
            DomEventType::Load,
            DomEventCallback::new(|_, _, context: &mut Context, event| {
                context.events.push((
                    event.target().unwrap(),
                    event.current_target().unwrap().node().unwrap(),
                ));
                Ok(())
            }),
            DomEventListenerOptions {
                capture: false,
                once: false,
            },
        )
        .unwrap();
}
fn unchanged_names(a: &DomStyleSheetSetObservation, b: &DomStyleSheetSetObservation) {
    assert_eq!(a.document(), b.document());
    assert_eq!(a.preferred_name(), b.preferred_name());
    assert_eq!(a.last_selected_name(), b.last_selected_name());
    assert_eq!(a.revision(), b.revision());
}

#[test]
fn preferred_b_preserves_a_and_selects_each_new_title_through_a_third_subject() {
    for title in ["C", "B"] {
        let source = SOURCE.replace("title=C", &format!("title={title}"));
        let (mut session, installation, a0, old_a) = first(&source, Default::default());
        let id = a0.contribution();
        let document = installation.document();
        let (styles, m) = rows(&session, id);
        assert_eq!(styles.len(), 6);
        let (_, a) = pop(&mut session, &a0, &styles[1], "p{color:red}");
        meta(&mut session, id, &m);
        let basis = names(&mut session, document);
        assert_eq!(basis.last_selected_name(), None);
        let before = snapshot(&session, document);
        let a_owner = before.sheet(a.id()).unwrap().inputs().owner.clone();
        let a_creation = a.revision().clone();
        assert!(a_owner.is_some());
        assert!(before.sheet(a.id()).unwrap().inputs().disabled);
        let c0 = next(&mut session, id, &styles[2]);
        assert_ne!(c0.subject(), a0.subject());
        assert_eq!(snapshot(&session, document).revision(), before.revision());
        allow(&mut session, &c0, "");
        let old_c = created(&mut session, &c0);
        let inserted = snapshot(&session, document);
        assert_eq!(inserted.sheets(), &[a.id().clone(), old_c.id().clone()]);
        assert!(inserted.sheet_rules(old_c.id()).unwrap().is_empty());
        assert_eq!(
            inserted.sheet(old_c.id()).unwrap().inputs().disabled,
            title != "B"
        );
        let (c1, c) = pop(&mut session, &c0, &styles[3], "c{color:blue}");
        let after_c = snapshot(&session, document);
        assert_eq!(after_c.sheets(), &[a.id().clone(), c.id().clone()]);
        assert_eq!(after_c.sheet(old_c.id()).unwrap().inputs().owner, None);
        assert_eq!(after_c.sheet(old_a.id()).unwrap().inputs().owner, None);
        assert_eq!(after_c.sheet(a.id()).unwrap().inputs().owner, a_owner);
        assert_eq!(after_c.sheet_rules(a.id()).unwrap().len(), 1);
        assert_eq!(
            after_c.sheet_rules(a.id()).unwrap(),
            before.sheet_rules(a.id()).unwrap()
        );
        assert_eq!(after_c.sheet_rules(c.id()).unwrap().len(), 1);
        assert_eq!(
            after_c.sheet(c.id()).unwrap().inputs().disabled,
            title != "B"
        );
        assert_ne!(old_c.id(), c.id());
        assert_eq!(old_c.association().element(), c.association().element());
        unchanged_names(&names(&mut session, document), &basis);
        let c_owner = after_c.sheet(c.id()).unwrap().inputs().owner.clone();
        let d0 = next(&mut session, id, &styles[4]);
        assert_ne!(d0.subject(), a0.subject());
        assert_ne!(d0.subject(), c1.subject());
        allow(&mut session, &d0, "");
        let old_d = created(&mut session, &d0);
        let (_, d) = pop(&mut session, &d0, &styles[5], "d{color:green}");
        let final_state = snapshot(&session, document);
        assert_eq!(
            final_state.sheets(),
            &[a.id().clone(), c.id().clone(), d.id().clone()]
        );
        assert_eq!(final_state.sheet(a.id()).unwrap().inputs().owner, a_owner);
        assert_eq!(final_state.sheet(c.id()).unwrap().inputs().owner, c_owner);
        assert_eq!(
            final_state.sheet_rules(a.id()).unwrap(),
            before.sheet_rules(a.id()).unwrap()
        );
        assert_eq!(
            final_state.sheet_rules(c.id()).unwrap(),
            after_c.sheet_rules(c.id()).unwrap()
        );
        assert_eq!(final_state.sheet(old_d.id()).unwrap().inputs().owner, None);
        assert!(final_state.sheet(a.id()).unwrap().inputs().disabled);
        assert!(!final_state.sheet(d.id()).unwrap().inputs().disabled);
        assert_eq!(a.revision(), &a_creation);
        unchanged_names(&names(&mut session, document), &basis);
        end(&mut session, id);
        pending(&session, id);
    }
}

#[test]
fn queued_a_old_and_late_a_final_survive_output_drop_subject_retirement_and_session_move() {
    let (mut session, installation, a0, old_a) = first(SOURCE, Default::default());
    let id = a0.contribution();
    let document = installation.document();
    let (styles, m) = rows(&session, id);
    let first_task = queue(&mut session, &a0);
    let (a1, a) = pop(&mut session, &a0, &styles[1], "p{color:red}");
    meta(&mut session, id, &m);
    let c0 = next(&mut session, id, &styles[2]);
    allow(&mut session, &c0, "");
    let old_c = created(&mut session, &c0);
    let (c1, c) = pop(&mut session, &c0, &styles[3], "c{color:blue}");
    let d0 = next(&mut session, id, &styles[4]);
    allow(&mut session, &d0, "");
    let old_d = created(&mut session, &d0);
    let (d1, d) = pop(&mut session, &d0, &styles[5], "d{color:green}");
    drop(installation);
    let DomShutdownOutcome::Pending {
        session: recovered,
        reason,
    } = session.try_shutdown()
    else {
        panic!("pending original history")
    };
    assert_eq!(reason, DomShutdownReason::ParsedAssociations(id));
    session = recovered;
    let after_move = snapshot(&session, document);
    assert_eq!(
        after_move.sheets(),
        &[a.id().clone(), c.id().clone(), d.id().clone()]
    );
    assert_eq!(after_move.sheet(old_a.id()).unwrap().inputs().owner, None);
    let late_a = queue(&mut session, &a1);
    owner_removed(&mut session, &c0, &old_c);
    owner_removed(&mut session, &d0, &old_d);
    let late_c = queue(&mut session, &c1);
    let late_d = queue(&mut session, &d1);
    let (mut dispatcher, mut context) = events(&session);
    for sheet in [&a, &c, &d] {
        listen(&mut session, &mut dispatcher, sheet.association().element());
    }
    assert!(
        matches!(run(&mut session, &mut dispatcher, &mut context), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == first_task)
    );
    assert!(context.events.is_empty());
    end(&mut session, id);
    for (task, target) in [
        (first_task, a.association().element()),
        (late_a, a.association().element()),
        (late_c, c.association().element()),
        (late_d, d.association().element()),
    ] {
        let DomStyleNetworkingProgress::Complete {
            task: actual,
            receipt,
        } = run(&mut session, &mut dispatcher, &mut context)
        else {
            panic!("real end releases FIFO")
        };
        assert_eq!(actual, task);
        assert_eq!(receipt.original_target(), target);
        assert_eq!(receipt.event_type(), DomEventType::Load);
    }
    assert!(matches!(
        run(&mut session, &mut dispatcher, &mut context),
        DomStyleNetworkingProgress::Empty
    ));
    assert_eq!(
        context.events,
        vec![
            (a.association().element(), a.association().element()),
            (a.association().element(), a.association().element()),
            (c.association().element(), c.association().element()),
            (d.association().element(), d.association().element())
        ]
    );
    pending(&session, id);
}

#[test]
fn unissued_owner_cleared_a_old_is_not_fabricated_into_a_new_event_after_c() {
    let (mut session, installation, a0, old_a) = first(SOURCE, Default::default());
    let id = a0.contribution();
    let (styles, m) = rows(&session, id);
    let (a1, a) = pop(&mut session, &a0, &styles[1], "p{color:red}");
    meta(&mut session, id, &m);
    let c0 = next(&mut session, id, &styles[2]);
    allow(&mut session, &c0, "");
    drop(created(&mut session, &c0));
    let (_, c) = pop(&mut session, &c0, &styles[3], "c{color:blue}");
    owner_removed(&mut session, &a0, &old_a);
    let actual = snapshot(&session, installation.document());
    assert_eq!(actual.sheets(), &[a.id().clone(), c.id().clone()]);
    let live_task = queue(&mut session, &a1);
    assert!(matches!(
        session.style_execution().queue_parsed_style_notification(
            &mut session.dom(),
            &a0,
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedNotificationError::ProcessingUnavailable { .. })
    ));
    let (mut dispatcher, mut context) = events(&session);
    listen(&mut session, &mut dispatcher, a.association().element());
    assert!(
        matches!(run(&mut session, &mut dispatcher, &mut context), DomStyleNetworkingProgress::OriginalSynchronousPending { task, .. } if task == live_task)
    );
    assert!(context.events.is_empty());
}

#[test]
fn exact_next_order_foreign_decisions_cancellation_and_real_capacity_retry_preserve_a() {
    let (mut session, installation, a0, _) = first(SOURCE, Default::default());
    let id = a0.contribution();
    let document = installation.document();
    let (styles, m) = rows(&session, id);
    assert!(matches!(
        session.style_execution().advance_parsed_style_subject(
            &mut session.dom(),
            id,
            &styles[2],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSubjectError::Subject(
            DomParsedHtmlStyleSubjectError::Remaining(_)
        ))
    ));
    let (_, a) = pop(&mut session, &a0, &styles[1], "p{color:red}");
    assert!(
        matches!(session.style_execution().advance_parsed_style_subject(&mut session.dom(), id, &styles[2], envelope(), &cancel()), Err(DomStyleParsedSubjectError::Subject(DomParsedHtmlStyleSubjectError::Remaining(DomParsedHtmlStyleNotificationRequirement::Occurrence { kind: Kind::ChangePreferred, occurrence, .. }))) if occurrence == m)
    );
    meta(&mut session, id, &m);
    let before = snapshot(&session, document);
    let before_names = names(&mut session, document);
    assert!(
        session
            .style_execution()
            .advance_parsed_style_subject(
                &mut session.dom(),
                id,
                &styles[2],
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    assert!(matches!(
        session.style_execution().advance_parsed_style_subject(
            &mut session.dom(),
            id,
            &styles[4],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSubjectError::Subject(
            DomParsedHtmlStyleSubjectError::WrongOccurrence
        ))
    ));
    let (mut foreign, foreign_installation) = installed(report(SOURCE));
    let foreign_id = foreign_installation.pending_association().unwrap();
    let (foreign_rows, _) = rows(&foreign, foreign_id);
    assert!(matches!(
        session.style_execution().advance_parsed_style_subject(
            &mut session.dom(),
            id,
            &foreign_rows[2],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSubjectError::Subject(
            DomParsedHtmlStyleSubjectError::WrongOccurrence
        ))
    ));
    assert!(matches!(
        session.style_execution().advance_parsed_style_subject(
            &mut foreign.dom(),
            id,
            &styles[2],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSubjectError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert_eq!(snapshot(&session, document).revision(), before.revision());
    unchanged_names(&names(&mut session, document), &before_names);
    let c0 = next(&mut session, id, &styles[2]);
    request_ready(
        &mut foreign,
        foreign_id,
        super::super::style_decision::empty_policy(),
        envelope(),
    );
    let mut wrong = observe(&foreign, foreign_id, &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut wrong,
                envelope(),
                &cancel()
            )
            .is_err()
    );
    assert!(wrong.is_some());
    let mut genuine = observe(&session, id, &mut |request| {
        assert_eq!(request.qualification(), &c0);
        assert!(request.text().is_empty());
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut genuine,
                envelope(),
                &AtomicBool::new(true)
            )
            .is_err()
    );
    assert!(genuine.is_some());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut genuine,
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert!(genuine.is_none());
    let mut runtime = session.style_execution();
    let mut tight = limits();
    tight.completions = session.state.borrow().completions.len();
    runtime
        .set_limits(&session.dom(), tight, &cancel())
        .unwrap();
    let spent = session.state.borrow().work.get();
    assert!(matches!(
        create(&mut session, id, envelope()),
        Err(DomStyleParsedSheetError::Runtime(
            DomStyleRuntimeError::Admission
        ))
    ));
    assert!(session.state.borrow().work.get() > spent);
    assert_eq!(current(&session, id), c0);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert_eq!(snapshot(&session, document).revision(), before.revision());
    assert_eq!(
        snapshot(&session, document).sheets(),
        std::slice::from_ref(a.id())
    );
    unchanged_names(&names(&mut session, document), &before_names);
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    let c = created(&mut session, &c0);
    assert!(matches!(
        session.style_execution().advance_parsed_style_subject(
            &mut session.dom(),
            id,
            &styles[2],
            envelope(),
            &cancel()
        ),
        Err(DomStyleParsedSubjectError::Subject(
            DomParsedHtmlStyleSubjectError::AlreadySelected
        ))
    ));
    assert_eq!(
        snapshot(&session, document).sheets(),
        &[a.id().clone(), c.id().clone()]
    );
}

#[test]
fn fixed_cssom_revision_refusal_cannot_drop_a_or_publish_c_partial_state() {
    let (mut session, installation, a0, _) = first(
        SOURCE,
        CssomLimits {
            max_revision: 4,
            ..Default::default()
        },
    );
    let id = a0.contribution();
    let document = installation.document();
    let (styles, m) = rows(&session, id);
    let (_, a) = pop(&mut session, &a0, &styles[1], "p{color:red}");
    meta(&mut session, id, &m);
    let c0 = next(&mut session, id, &styles[2]);
    allow(&mut session, &c0, "");
    let before = snapshot(&session, document);
    assert_eq!(
        before.revision().value(),
        4,
        "real A create/remove/recreate and changed Meta reached the fixed limit"
    );
    let before_names = names(&mut session, document);
    let before_tasks = session.state.borrow().next_task;
    assert!(matches!(
        create(&mut session, id, envelope()),
        Err(DomStyleParsedSheetError::Sheet(
            DomParsedHtmlStyleSheetError::Cssom {
                error: CssomError::RevisionExhausted,
                ..
            }
        ))
    ));
    let after = snapshot(&session, document);
    assert_eq!(after.revision(), before.revision());
    assert_eq!(after.sheets(), std::slice::from_ref(a.id()));
    assert_eq!(
        after.sheet(a.id()).unwrap().inputs().owner,
        before.sheet(a.id()).unwrap().inputs().owner
    );
    assert!(after.sheet(a.id()).unwrap().inputs().disabled);
    unchanged_names(&names(&mut session, document), &before_names);
    assert_eq!(session.state.borrow().next_task, before_tasks);
    assert_eq!(current(&session, id), c0);
    assert_eq!(
        session.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
}

#[test]
fn original_native_policy_is_evaluated_independently_for_each_new_subject_text() {
    let (policy, profile) = text_policy::policy(true);
    let (mut session, installation, a0, _) = blocked_first(SOURCE, policy, &profile);
    let id = a0.contribution();
    let document = installation.document();
    let (styles, m) = rows(&session, id);
    let successor = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        successor.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    let a1 = successor.qualification().clone();
    drop(successor);
    let mut input = observe(&session, id, &mut |request| {
        Some(text_policy::decision(request, &profile))
    });
    assert!(input.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    bind(&mut session, document, Default::default());
    let a = created(&mut session, &a1);
    meta(&mut session, id, &m);
    let c0 = next(&mut session, id, &styles[2]);
    let mut input = observe(&session, id, &mut |request| {
        assert!(request.text().is_empty());
        assert_eq!(request.qualification(), &c0);
        Some(text_policy::decision(request, &profile))
    });
    assert!(!input.as_ref().unwrap().allowed());
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut input,
                envelope(),
                &cancel()
            )
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    let successor = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        successor.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    let c1 = successor.qualification().clone();
    drop(successor);
    assert_eq!(c1.occurrence(), &styles[3]);
    let mut input = observe(&session, id, &mut |request| {
        assert_eq!(request.text(), &DocumentString::from("c{color:blue}"));
        Some(text_policy::decision(request, &profile))
    });
    assert!(!input.as_ref().unwrap().allowed());
    session
        .style_execution()
        .accept_parsed_style_inline_policy(
            &mut session.dom(),
            id,
            &mut input,
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert_eq!(
        snapshot(&session, document).sheets(),
        std::slice::from_ref(a.id())
    );
    let d0 = next(&mut session, id, &styles[4]);
    assert_ne!(d0.subject(), c0.subject());
    assert_eq!(
        names(&mut session, document).preferred_name(),
        &DocumentString::from("B")
    );
}

#[test]
fn actual_imports_keep_their_rows_without_blocking_later_subjects_or_real_end() {
    let source = "<!doctype html><style title=A>@import 'a.css';p{color:red}</style><meta http-equiv=default-style content=B><style title=C>@import 'c.css';c{color:blue}</style><style title=B>d{color:green}</style>";
    let (mut session, installation, a0, _) = first(source, Default::default());
    let id = a0.contribution();
    let (styles, m) = rows(&session, id);
    let old_task = queue(&mut session, &a0);
    let (a1, a) = pop(
        &mut session,
        &a0,
        &styles[1],
        "@import 'a.css';p{color:red}",
    );
    meta(&mut session, id, &m);
    let c0 = next(&mut session, id, &styles[2]);
    allow(&mut session, &c0, "");
    drop(created(&mut session, &c0));
    let c_task = queue(&mut session, &c0);
    let (c1, c) = pop(
        &mut session,
        &c0,
        &styles[3],
        "@import 'c.css';c{color:blue}",
    );
    let d0 = next(&mut session, id, &styles[4]);
    allow(&mut session, &d0, "");
    let old_d = created(&mut session, &d0);
    let (d1, _) = pop(&mut session, &d0, &styles[5], "d{color:green}");
    for (q, sheet) in [(&a1, &a), (&c1, &c)] {
        let serial = session.state.borrow().next_task;
        assert!(
            matches!(session.style_execution().queue_parsed_style_notification(&mut session.dom(), q, envelope(), &cancel()).unwrap(), DomStyleParsedNotificationProgress::PendingImports { association } if association == sheet.association())
        );
        assert_eq!(session.state.borrow().next_task, serial);
    }
    owner_removed(&mut session, &d0, &old_d);
    let d_task = queue(&mut session, &d1);
    end(&mut session, id);
    let (mut dispatcher, mut context) = events(&session);
    for expected in [old_task, c_task, d_task] {
        assert!(
            matches!(run(&mut session, &mut dispatcher, &mut context), DomStyleNetworkingProgress::Complete { task, .. } if task == expected)
        );
    }
    assert!(matches!(
        run(&mut session, &mut dispatcher, &mut context),
        DomStyleNetworkingProgress::Empty
    ));
    let state = session.state.borrow();
    for q in [&a1, &c1] {
        let row = state
            .completions
            .iter()
            .find_map(|row| row.processing().filter(|p| p.qualification == *q))
            .unwrap();
        assert_eq!(row.dependencies, ImportDependencies::Imports);
    }
    drop(state);
    assert_eq!(session.style_execution().waiting_resources().unwrap(), 0);
    assert_eq!(
        snapshot(&session, installation.document()).sheets().len(),
        3
    );
    pending(&session, id);
}
