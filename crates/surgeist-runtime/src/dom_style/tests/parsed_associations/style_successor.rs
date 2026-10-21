//! The next real ParserPopped prefix removes its predecessor without waiting for tasks.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, report};
use super::style_decision::{empty_policy, installed, observe, request_ready};
use super::style_sheet::{bind, create, pending};
use surgeist_dom::*;
use surgeist_html::HtmlStyleTrigger;
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
};

fn removed(
    result: &DomParsedHtmlStyleSuccessorCompletion,
) -> (&DomCommit, &surgeist_cssom::CssomCommit) {
    match result.removal() {
        DomParsedHtmlStyleSuccessorRemoval::Removed { dom, cssom } => (dom, cssom),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet => panic!("real created predecessor"),
    }
}

fn rendered(
    limits: DomParsedHtmlAssociationLimits,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleQualification,
    DomStyleSheet,
    Option<DomParsedHtmlStyleInlinePolicyObservation>,
) {
    let (mut session, installation) = installed(report(
        "<!doctype html><style title='chosen' nonce='original'>p{color:red}</style>",
    ));
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, empty_policy(), limits);
    let mut old_decision = None;
    let mut input = observe(&session, id, &mut |request| {
        assert!(request.text().is_empty());
        old_decision = Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ));
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    session
        .style_execution()
        .accept_parsed_style_inline_policy(&mut session.dom(), id, &mut input, limits, &cancel())
        .unwrap();
    bind(&mut session, installation.document(), Default::default());
    let sheet = create(&mut session, id, limits)
        .unwrap()
        .completion()
        .sheet()
        .clone();
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .qualification()
        .unwrap()
        .clone();
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), &q, limits, &cancel())
        .unwrap();
    assert!(matches!(
        session
            .style_execution()
            .process_parsed_style_rendering(&mut session.dom(), &q, limits, &cancel(),)
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    (session, installation, q, sheet, old_decision)
}
fn owner(session: &DomSession) -> std::rc::Rc<DomStyleCssomRoot> {
    session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.label == "original-root")
        .unwrap()
        .store
        .clone()
}
fn next(
    session: &DomSession,
    q: &DomParsedHtmlStyleQualification,
) -> surgeist_html_parser::HtmlParseAssociationOccurrenceId {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == q.contribution())
        .unwrap();
    let mut cursor = entry.report().pending_associations();
    let mut seen = false;
    loop {
        match cursor.advance() {
            Step::Occurrence(row) if row.id() == *q.occurrence() => seen = true,
            Step::Occurrence(row) if seen && row.requires_processing() => {
                assert_eq!(
                    row.kind(),
                    Kind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
                );
                assert_eq!(row.prior_style().as_ref(), Some(q.occurrence()));
                return row.id();
            }
            Step::Complete => panic!("literal source retains parser-pop invocation"),
            _ => (),
        }
    }
}
fn assert_pending_request(session: &DomSession, q: &DomParsedHtmlStyleQualification) {
    let view = session.parsed_html_style_prefix(q.contribution()).unwrap();
    assert_eq!(view.phase(), DomParsedHtmlStylePhase::InlinePolicyPending);
    let request = view.inline_policy_request().unwrap();
    assert_eq!(request.qualification(), q);
    assert_eq!(request.text(), &DocumentString::from("p{color:red}"));
    assert_eq!(request.nonce(), Some(&DocumentString::from("original")));
    pending(session, q.contribution());
}

#[test]
fn successor_removes_real_sheet_preserves_names_and_requires_a_new_full_text_decision() {
    let (mut session, _installation, old, sheet, mut old_decision) = rendered(envelope());
    let expected = next(&session, &old);
    let owner = owner(&session);
    let before = owner.snapshot().unwrap();
    let mut edit = owner.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
    edit.insert_sheet_rule(sheet.id(), "x{}", 0).unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    let names = session
        .style_execution()
        .sheet_sets(&session.dom(), old.document().node(), &cancel())
        .unwrap();
    let serials = {
        let state = session.state.borrow();
        (
            state
                .roots
                .iter()
                .map(|root| root.serial)
                .collect::<Vec<_>>(),
            state.next_task,
        )
    };
    let stamp = session.snapshot().stamp();
    let result = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = result.qualification().clone();
    assert_eq!(q.occurrence(), &expected);
    assert_ne!(q.occurrence(), old.occurrence());
    assert_ne!(q.context_epoch(), old.context_epoch());
    assert_eq!(q.original_document(), old.original_document());
    assert_eq!(result.phase(), DomParsedHtmlStylePhase::InlinePolicyPending);
    let (dom, _) = removed(&result);
    assert_eq!(dom.changes().from(), stamp);
    assert_ne!(session.snapshot().stamp(), stamp);
    assert!(matches!(
        removed(&result).1.publication(),
        surgeist_cssom::CssomPublication::Changed(_)
    ));
    let current = owner.snapshot().unwrap();
    assert!(current.sheets().is_empty());
    assert_eq!(current.sheet(sheet.id()).unwrap().inputs().owner, None);
    assert_eq!(current.sheet(sheet.id()).unwrap().owner_rule(), None);
    assert_eq!(
        current
            .rules(current.sheet(sheet.id()).unwrap().rules())
            .unwrap()
            .len(),
        1
    );
    assert!(
        dom.changes()
            .changes()
            .contains(&DomChange::ElementState(sheet.association().element()))
    );
    assert!(matches!(
        session
            .snapshot()
            .node(sheet.association().element())
            .unwrap()
            .as_style()
            .unwrap()
            .sheet(),
        Err(DomError::StyleProcessingUnavailable)
    ));
    let after = session
        .style_execution()
        .sheet_sets(&session.dom(), old.document().node(), &cancel())
        .unwrap();
    assert_eq!(after.preferred_name(), names.preferred_name());
    assert_eq!(after.last_selected_name(), names.last_selected_name());
    assert_eq!(after.revision(), names.revision());
    let state = session.state.borrow();
    assert_eq!(
        state
            .roots
            .iter()
            .map(|root| root.serial)
            .collect::<Vec<_>>(),
        serials.0
    );
    assert_eq!(state.next_task, serials.1);
    assert_eq!(state.completions.len(), 1);
    assert_eq!(
        state.completions[0].processing().unwrap().qualification,
        old
    );
    drop(state);
    drop(result);
    assert_pending_request(&session, &q);
    assert!(matches!(
        session.style_execution().accept_parsed_style_inline_policy(
            &mut session.dom(),
            old.contribution(),
            &mut old_decision,
            envelope(),
            &cancel(),
        ),
        Err(DomStyleParsedPrefixError::Style(
            DomParsedHtmlStyleError::WrongRequest
        ))
    ));
    assert!(old_decision.is_some());
    assert_pending_request(&session, &q);
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(
        &mut session.dom(), &old, envelope(), &cancel(),
    ).unwrap(), DomStyleParsedNotificationProgress::OwnerAssociationRemoved{association} if association==sheet.association())
    );
    assert!(session.state.borrow().completions.is_empty());
    assert_pending_request(&session, &q);
}

#[test]
fn queued_predecessor_keeps_exact_task_and_blocks_on_the_new_original_invocation() {
    let (mut session, _installation, old, sheet, _) = rendered(envelope());
    let DomStyleParsedNotificationProgress::Queued { task } = session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), &old, envelope(), &cancel())
        .unwrap()
    else {
        panic!("actual original NoImports")
    };
    let result = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = result.qualification().clone();
    drop(result);
    {
        let state = session.state.borrow();
        let row = state.completions[0].task();
        assert_eq!(row.id(), task);
        assert_eq!(row.sheet, *sheet.id());
        assert_eq!(row.parsed.as_ref().unwrap().qualification, old);
        assert_eq!(state.next_task, 2);
    }
    let mut context = Context {
        styles: session.style_execution(),
        source: session.source_execution(),
        events: Vec::new(),
    };
    let mut dispatcher = DomEventDispatcher::new(DomEventRegistryLimits {
        registrations: 16,
        pending_events: 16,
        nested_dispatches: 8,
    })
    .unwrap();
    assert!(
        matches!(DomStyleRuntime::run_networking(&mut session.dom(), &mut dispatcher, &mut context, dispatch_limits(), &cancel()).unwrap(),
        DomStyleNetworkingProgress::OriginalSynchronousPending {task:actual,requirement:DomParsedHtmlStyleNotificationRequirement::Invocation{contribution,occurrence,phase,..}}
        if actual==task && contribution==q.contribution() && occurrence==*q.occurrence() && phase==DomParsedHtmlStylePhase::InlinePolicyPending)
    );
    assert!(context.events.is_empty());
    assert!(matches!(
        session.state.borrow().completions[0].task().delivery,
        Delivery::NotStarted
    ));
    assert_pending_request(&session, &q);
}

#[test]
fn terminal_old_receipt_is_not_a_prerequisite_for_successor_and_removal_can_be_cssom_noop() {
    let (mut session, _installation, old, sheet, _) = rendered(envelope());
    let owner = owner(&session);
    let before = owner.snapshot().unwrap();
    let mut edit = owner.batch(CssomEditGuard::from_snapshot(&before)).unwrap();
    edit.remove_sheet(sheet.id()).unwrap();
    drop(owner.commit(&mut Some(edit)).unwrap());
    assert!(matches!(
        session
            .style_execution()
            .queue_parsed_style_notification(&mut session.dom(), &old, envelope(), &cancel(),)
            .unwrap(),
        DomStyleParsedNotificationProgress::OwnerAssociationRemoved { .. }
    ));
    assert!(session.state.borrow().completions.is_empty());
    let result = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    assert!(matches!(
        removed(&result).1.publication(),
        surgeist_cssom::CssomPublication::Unchanged { .. }
    ));
    let q = result.qualification().clone();
    drop(result);
    assert_pending_request(&session, &q);
    assert!(session.state.borrow().completions.is_empty());
    assert_eq!(session.state.borrow().next_task, 1);
}

#[test]
fn successor_wrong_session_cancellation_and_wrong_phase_preserve_later_recreation() {
    let (mut session, _installation, old, sheet, _) = rendered(envelope());
    let owner = owner(&session);
    let revision = owner.snapshot().unwrap().revision().clone();
    let stamp = session.snapshot().stamp();
    let runtime = session.style_execution();
    let (mut foreign, _) = bound_dispatch_session();
    assert!(matches!(
        runtime.advance_parsed_style_successor(
            &mut foreign.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        ),
        Err(DomStyleParsedSuccessorError::Runtime(
            DomStyleRuntimeError::WrongSession
        ))
    ));
    assert!(matches!(
        runtime.advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &AtomicBool::new(true),
        ),
        Err(DomStyleParsedSuccessorError::Successor(
            DomParsedHtmlStyleSuccessorError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert_eq!(owner.snapshot().unwrap().sheets(), &[sheet.id().clone()]);
    let result = runtime
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = result.qualification().clone();
    drop(result);
    let stamp = session.snapshot().stamp();
    let revision = owner.snapshot().unwrap().revision().clone();
    assert!(matches!(
        runtime.advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        ),
        Err(DomStyleParsedSuccessorError::Successor(
            DomParsedHtmlStyleSuccessorError::Style(DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::InlinePolicyPending
            })
        ))
    ));
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert_pending_request(&session, &q);
    let mut decision = observe(&session, old.contribution(), &mut |request| {
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        runtime
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                old.contribution(),
                &mut decision,
                envelope(),
                &cancel(),
            )
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let recreated = create(&mut session, old.contribution(), envelope()).unwrap();
    assert_ne!(recreated.completion().sheet().id(), sheet.id());
    assert_ne!(
        recreated.completion().sheet().association(),
        sheet.association()
    );
    drop(recreated);
    runtime
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), &q, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        runtime
            .process_parsed_style_rendering(&mut session.dom(), &q, envelope(), &cancel())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    assert!(
        matches!(runtime.advance_parsed_style_successor(&mut session.dom(), old.contribution(), envelope(), &cancel()),
        Err(DomStyleParsedSuccessorError::Successor(DomParsedHtmlStyleSuccessorError::Style(DomParsedHtmlStyleError::UnsupportedProfile { contribution }))) if contribution == old.contribution())
    );
    assert_eq!(session.state.borrow().next_task, 1);
    pending(&session, old.contribution());
}

#[test]
fn original_finite_output_refusal_preserves_sheet_and_spent_work_until_raised_retry() {
    let large = envelope();
    let finite = DomParsedHtmlAssociationLimits::new(
        large.retained_bytes(),
        large.retained_units(),
        large.work(),
        10,
    )
    .unwrap();
    let (mut session, _installation, old, sheet, _) = rendered(finite);
    let owner = owner(&session);
    let stamp = session.snapshot().stamp();
    let revision = owner.snapshot().unwrap().revision().clone();
    let spent = session.state.borrow().work.get();
    let refusal = session.style_execution().advance_parsed_style_successor(
        &mut session.dom(),
        old.contribution(),
        finite,
        &cancel(),
    );
    assert!(
        matches!(
            &refusal,
            Err(DomStyleParsedSuccessorError::Runtime(
                DomStyleRuntimeError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 10,
                })
            ))
        ),
        "{refusal:?}"
    );
    assert!(session.state.borrow().work.get() > spent);
    assert_eq!(session.snapshot().stamp(), stamp);
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert_eq!(owner.snapshot().unwrap().sheets(), &[sheet.id().clone()]);
    assert_eq!(
        session
            .parsed_html_style_prefix(old.contribution())
            .unwrap()
            .phase(),
        DomParsedHtmlStylePhase::RenderingComplete
    );
    assert_eq!(
        session.state.borrow().completions[0]
            .processing()
            .unwrap()
            .qualification,
        old
    );
    let result = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), old.contribution(), large, &cancel())
        .unwrap();
    let q = result.qualification().clone();
    drop(result);
    assert_pending_request(&session, &q);
    assert!(owner.snapshot().unwrap().sheets().is_empty());
}

#[test]
fn parked_real_source_operation_excludes_the_original_successor() {
    let (mut session, _installation, old, sheet, _) = rendered(envelope());
    let document = session
        .state
        .borrow()
        .roots
        .iter()
        .find(|root| root.node != old.document().node())
        .unwrap()
        .node;
    let mut batch = DomBatch::new();
    let dialog = batch.create_element_ns(document, Namespace::html(), "dialog");
    let parent = session.snapshot().node(document).unwrap().children()[0];
    batch.insert_before(&dialog, parent, None);
    let setup = session.dom().apply(batch, &cancel()).unwrap();
    let dialog = setup.created(&dialog).unwrap().unwrap();
    assert!(matches!(
        session
            .source_execution()
            .apply(
                &mut session.dom(),
                DomSourceEdit::show_dialog(dialog),
                source_limits(),
                &mut DomSourceServices {
                    meta: None,
                    execution_scope: None,
                    style_policy: &mut policy,
                    url: None
                },
                &cancel(),
            )
            .unwrap(),
        DomSourceDriveProgress::Pending {
            request: DomSourceRequest::Event(_),
            ..
        }
    ));
    let operation = session.dom().pending_source_edit().unwrap().operation();
    let owner = owner(&session);
    let revision = owner.snapshot().unwrap().revision().clone();
    assert!(
        matches!(session.style_execution().advance_parsed_style_successor(
        &mut session.dom(),old.contribution(),envelope(),&cancel(),
    ),Err(DomStyleParsedSuccessorError::Runtime(DomStyleRuntimeError::SourcePending(actual))) if actual==operation)
    );
    assert_eq!(
        session.dom().pending_source_edit().unwrap().operation(),
        operation
    );
    assert_eq!(owner.snapshot().unwrap().revision(), &revision);
    assert_eq!(owner.snapshot().unwrap().sheets(), &[sheet.id().clone()]);
    assert_eq!(
        session
            .parsed_html_style_prefix(old.contribution())
            .unwrap()
            .phase(),
        DomParsedHtmlStylePhase::RenderingComplete
    );
}
