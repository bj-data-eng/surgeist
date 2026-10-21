//! Real original ParserPopped progress after created and blocked predecessors.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, report};
use super::style_decision::{installed, observe, request_ready};
use super::style_sheet::{bind, create, pending, ready};
use surgeist_css::CssUtf16InputError;
use surgeist_dom::*;
use surgeist_html::{HtmlCspAssociation, HtmlCspProcessorProfile, HtmlStyleTrigger};
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
};
mod frontier;
mod text_policy;

fn next_parser_pop(
    session: &DomSession,
    old: &DomParsedHtmlStyleInitialPolicyRequest,
) -> surgeist_html_parser::HtmlParseAssociationOccurrenceId {
    let entry = session
        .pending_parsed_html_associations()
        .find(|entry| entry.id() == old.contribution())
        .unwrap();
    let mut inventory = entry.report().pending_associations();
    let mut seen = false;
    loop {
        match inventory.advance() {
            Step::Occurrence(row) if row.id() == *old.occurrence() => seen = true,
            Step::Occurrence(row) if seen && row.requires_processing() => {
                assert_eq!(
                    row.kind(),
                    Kind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
                );
                assert_eq!(row.prior_style().as_ref(), Some(old.occurrence()));
                return row.id();
            }
            Step::Complete => panic!("literal source retains its actual ParserPopped update"),
            _ => (),
        }
    }
}

pub(super) fn finish_rendering(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleInitialPolicyRequest,
) {
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), q, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        session
            .style_execution()
            .process_parsed_style_rendering(&mut session.dom(), q, envelope(), &cancel(),)
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
}
pub(super) fn decide(
    session: &mut DomSession,
    q: &DomParsedHtmlStyleInitialPolicyRequest,
    allowed: bool,
) {
    let mut input = observe(session, q.contribution(), &mut |request| {
        assert_eq!(request.qualification(), q);
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, allowed,
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
                &cancel(),
            )
            .unwrap(),
        if allowed {
            DomParsedHtmlStylePhase::CreateSheetPending
        } else {
            DomParsedHtmlStylePhase::InlinePolicyBlocked
        }
    );
    assert!(input.is_none());
}
fn created_first(
    text: &str,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleInitialPolicyRequest,
    DomStyleSheet,
) {
    let (mut session, installation) = ready(text, envelope());
    let id = installation.pending_association().unwrap();
    bind(&mut session, installation.document(), Default::default());
    let first = create(&mut session, id, envelope()).unwrap();
    let sheet = first.completion().sheet().clone();
    drop(first);
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    finish_rendering(&mut session, &q);
    (session, installation, q, sheet)
}
fn blocked_first(
    text: &str,
    policy: HtmlCspAssociation,
    profile: &HtmlCspProcessorProfile,
) -> (
    DomSession,
    DomParsedHtmlInstallation,
    DomParsedHtmlStyleInitialPolicyRequest,
    Option<DomParsedHtmlStyleInlinePolicyObservation>,
) {
    let (mut session, installation) = installed(report(text));
    let id = installation.pending_association().unwrap();
    request_ready(&mut session, id, policy, envelope());
    let q = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let old_decision = observe(&session, id, &mut |request| {
        Some(text_policy::decision(request, profile))
    });
    let mut input = observe(&session, id, &mut |request| {
        Some(text_policy::decision(request, profile))
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
                &cancel(),
            )
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(input.is_none());
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    (session, installation, q, old_decision)
}

#[test]
fn fresh_parser_pop_allow_creates_distinct_nonempty_sheet_and_processing_receipt() {
    let (mut session, installation) = ready(
        "<!doctype html><style title='chosen' nonce='original'>p{color:red}</style>",
        envelope(),
    );
    let id = installation.pending_association().unwrap();
    bind(&mut session, installation.document(), Default::default());
    let first = create(&mut session, id, envelope()).unwrap();
    let old_sheet = first.completion().sheet().clone();
    let old = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let expected = next_parser_pop(&session, &old);
    drop(first);
    session
        .style_execution()
        .exclude_parsed_style_script_bookkeeping(&mut session.dom(), &old, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        session
            .style_execution()
            .process_parsed_style_rendering(&mut session.dom(), &old, envelope(), &cancel(),)
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    let successor = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .unwrap();
    assert!(matches!(
        successor.removal(),
        DomParsedHtmlStyleSuccessorRemoval::Removed { .. }
    ));
    let new = successor.qualification().clone();
    drop(successor);
    assert_eq!(new.occurrence(), &expected);
    assert_ne!(new.occurrence(), old.occurrence());
    let mut decision = observe(&session, id, &mut |request| {
        assert_eq!(request.qualification(), &new);
        assert_eq!(request.text(), &DocumentString::from("p{color:red}"));
        assert_eq!(request.nonce(), Some(&DocumentString::from("original")));
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, true,
        ))
    });
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut decision,
                envelope(),
                &cancel(),
            )
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert!(decision.is_none());
    let recreated = create(&mut session, id, envelope())
        .expect("a fresh allowed ParserPopped invocation creates its own original sheet");
    let new_sheet = recreated.completion().sheet();
    assert_ne!(new_sheet.id(), old_sheet.id());
    assert_ne!(new_sheet.association(), old_sheet.association());
    assert_eq!(
        new_sheet.association().element(),
        old_sheet.association().element()
    );
    let actual = session
        .style_execution()
        .snapshot(installation.document().node())
        .unwrap()
        .unwrap();
    assert_eq!(actual.sheets(), &[new_sheet.id().clone()]);
    assert_eq!(
        actual
            .rules(actual.sheet(new_sheet.id()).unwrap().rules())
            .unwrap()
            .len(),
        1
    );
    assert_eq!(actual.sheet(old_sheet.id()).unwrap().inputs().owner, None);
    let state = session.state.borrow();
    assert_eq!(state.completions.len(), 2);
    let old_record = state.completions[0].processing().unwrap();
    let new_record = state.completions[1].processing().unwrap();
    assert_eq!(old_record.qualification, old);
    assert_eq!(old_record.sheet.as_ref(), Some(&old_sheet));
    assert_eq!(new_record.qualification, new);
    assert_eq!(new_record.sheet.as_ref(), Some(new_sheet));
    assert_eq!(new_record.dependencies, ImportDependencies::NoImports);
    assert_eq!(state.next_task, 1, "creation issues no networking task");
    drop(state);
    pending(&session, id);
}

#[test]
fn blocked_first_without_cssom_owner_advances_to_fresh_original_parser_pop_request() {
    let (mut session, installation) = installed(report(
        "<!doctype html><style nonce='original'>p{color:red}</style>",
    ));
    let id = installation.pending_association().unwrap();
    let (policy, profile) = text_policy::policy(true);
    request_ready(&mut session, id, policy, envelope());
    let old = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let expected = next_parser_pop(&session, &old);
    let mut decision = observe(&session, id, &mut |request| {
        assert!(request.text().is_empty());
        Some(text_policy::decision(request, &profile))
    });
    assert_eq!(
        session
            .style_execution()
            .accept_parsed_style_inline_policy(
                &mut session.dom(),
                id,
                &mut decision,
                envelope(),
                &cancel(),
            )
            .unwrap(),
        DomParsedHtmlStylePhase::InlinePolicyBlocked
    );
    assert!(decision.is_none());
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    assert!(session.state.borrow().completions.is_empty());
    let successor = session
        .style_execution()
        .advance_parsed_style_successor(&mut session.dom(), id, envelope(), &cancel())
        .expect("a genuine blocked predecessor needs no CSSOM owner to advance ParserPopped");
    assert!(matches!(
        successor.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    let new = successor.qualification();
    assert_eq!(new.occurrence(), &expected);
    assert_ne!(new.occurrence(), old.occurrence());
    assert_eq!(new.slot(), old.slot());
    let view = session.parsed_html_style_prefix(id).unwrap();
    assert_eq!(view.phase(), DomParsedHtmlStylePhase::InlinePolicyPending);
    let request = view.inline_policy_request().unwrap();
    assert_eq!(request.qualification(), new);
    assert_eq!(request.text(), &DocumentString::from("p{color:red}"));
    assert_eq!(request.nonce(), Some(&DocumentString::from("original")));
    assert!(
        session
            .style_execution()
            .snapshot(installation.document().node())
            .unwrap()
            .is_none()
    );
    assert!(session.state.borrow().completions.is_empty());
    assert_eq!(session.state.borrow().next_task, 1);
}

#[test]
fn blocked_predecessor_preserves_old_decision_qualification_and_independent_allowed_or_blocked_successor()
 {
    for allowed in [false, true] {
        let (policy, profile) = text_policy::policy(allowed);
        let (mut session, installation, old, mut old_decision) = blocked_first(
            "<!doctype html><style nonce=original>p{color:red}</style>",
            policy,
            &profile,
        );
        let stamp = session.snapshot().stamp();
        let outcome = session
            .style_execution()
            .advance_parsed_style_successor(
                &mut session.dom(),
                old.contribution(),
                envelope(),
                &cancel(),
            )
            .unwrap();
        assert!(matches!(
            outcome.removal(),
            DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
        ));
        let q = outcome.qualification().clone();
        drop(outcome);
        assert_eq!(
            session.snapshot().stamp(),
            stamp,
            "no live sheet or tree publication was required"
        );
        assert_eq!(q.slot(), old.slot());
        assert_ne!(q.occurrence(), old.occurrence());
        assert!(
            session
                .style_execution()
                .snapshot(installation.document().node())
                .unwrap()
                .is_none()
        );
        assert!(matches!(
            session.style_execution().accept_parsed_style_inline_policy(
                &mut session.dom(),
                q.contribution(),
                &mut old_decision,
                envelope(),
                &cancel(),
            ),
            Err(DomStyleParsedPrefixError::Style(
                DomParsedHtmlStyleError::WrongRequest
            ))
        ));
        assert!(old_decision.is_some());
        let mut decision = observe(&session, q.contribution(), &mut |request| {
            Some(text_policy::decision(request, &profile))
        });
        assert_eq!(decision.as_ref().unwrap().allowed(), allowed);
        assert_eq!(
            session
                .style_execution()
                .accept_parsed_style_inline_policy(
                    &mut session.dom(),
                    q.contribution(),
                    &mut decision,
                    envelope(),
                    &cancel(),
                )
                .unwrap(),
            if allowed {
                DomParsedHtmlStylePhase::CreateSheetPending
            } else {
                DomParsedHtmlStylePhase::InlinePolicyBlocked
            }
        );
        assert!(decision.is_none());
        if allowed {
            // Creation, rather than known-none advancement, first needs an owner.
            assert!(matches!(create(&mut session,q.contribution(),envelope()),
                Err(DomStyleParsedSheetError::Runtime(DomStyleRuntimeError::SheetSetRootUnavailable{root})) if root==installation.document().node()));
            bind(&mut session, installation.document(), Default::default());
            let created = create(&mut session, q.contribution(), envelope()).unwrap();
            let sheet = created.completion().sheet().clone();
            drop(created);
            finish_rendering(&mut session, &q);
            let state = session.state.borrow();
            assert_eq!(state.completions.len(), 1);
            assert_eq!(state.completions[0].processing().unwrap().qualification, q);
            assert_eq!(
                state.completions[0].processing().unwrap().sheet.as_ref(),
                Some(&sheet)
            );
            assert_eq!(
                state
                    .roots
                    .iter()
                    .find(|root| root.node == installation.document().node())
                    .unwrap()
                    .serial,
                1
            );
        } else {
            assert!(
                session
                    .style_execution()
                    .snapshot(installation.document().node())
                    .unwrap()
                    .is_none()
            );
            assert!(session.state.borrow().completions.is_empty());
            assert_eq!(
                session
                    .parsed_html_style_prefix(q.contribution())
                    .unwrap()
                    .phase(),
                DomParsedHtmlStylePhase::InlinePolicyBlocked
            );
        }
        assert_eq!(session.state.borrow().next_task, 1);
        pending(&session, q.contribution());
    }
}

#[test]
fn actual_successor_imports_stay_pending_without_fabricated_resource_or_task_completion() {
    let (mut session, installation, old, _) =
        created_first("<!doctype html><style>@import 'required.css';p{color:red}</style>");
    let outcome = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = outcome.qualification().clone();
    drop(outcome);
    decide(&mut session, &q, true);
    let created = create(&mut session, q.contribution(), envelope()).unwrap();
    let sheet = created.completion().sheet().clone();
    drop(created);
    finish_rendering(&mut session, &q);
    assert_eq!(
        session.state.borrow().completions[1]
            .processing()
            .unwrap()
            .dependencies,
        ImportDependencies::Imports
    );
    assert!(matches!(
        session
            .style_execution()
            .queue_parsed_style_notification(&mut session.dom(), &old, envelope(), &cancel())
            .unwrap(),
        DomStyleParsedNotificationProgress::OwnerAssociationRemoved { .. }
    ));
    assert!(
        matches!(session.style_execution().queue_parsed_style_notification(&mut session.dom(),&q,envelope(),&cancel()).unwrap(),DomStyleParsedNotificationProgress::PendingImports{association} if association==sheet.association())
    );
    assert_eq!(session.state.borrow().completions.len(), 1);
    assert_eq!(session.state.borrow().next_task, 1);
    assert_eq!(
        session.style_execution().waiting_resources().unwrap(),
        0,
        "pre-task original processing is not a native resource wait"
    );
    let DomShutdownOutcome::Pending { session, reason } = session.try_shutdown() else {
        panic!("original history and import proof remain retained")
    };
    assert_eq!(
        reason,
        DomShutdownReason::ParsedAssociations(q.contribution())
    );
    assert_eq!(
        session.state.borrow().completions[0]
            .processing()
            .unwrap()
            .qualification,
        q
    );
    assert_eq!(
        session.state.borrow().completions[0]
            .processing()
            .unwrap()
            .dependencies,
        ImportDependencies::Imports
    );
    assert_eq!(installation.pending_association(), Some(q.contribution()));
}

#[test]
fn queued_old_and_new_invocations_preserve_fifo_names_and_precise_later_meta_requirement() {
    let (mut session, installation, old, old_sheet) = created_first(
        "<!doctype html><style title=chosen>p{color:red}</style><meta http-equiv=default-style content=later>",
    );
    let names = session
        .style_execution()
        .sheet_sets(&session.dom(), installation.document().node(), &cancel())
        .unwrap();
    let DomStyleParsedNotificationProgress::Queued { task: old_task } = session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), &old, envelope(), &cancel())
        .unwrap()
    else {
        panic!("empty first sheet NoImports")
    };
    let outcome = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = outcome.qualification().clone();
    drop(outcome);
    decide(&mut session, &q, true);
    let created = create(&mut session, q.contribution(), envelope()).unwrap();
    let new_sheet = created.completion().sheet().clone();
    drop(created);
    finish_rendering(&mut session, &q);
    let DomStyleParsedNotificationProgress::Queued { task: new_task } = session
        .style_execution()
        .queue_parsed_style_notification(&mut session.dom(), &q, envelope(), &cancel())
        .unwrap()
    else {
        panic!("actual new NoImports")
    };
    assert_ne!(old_task, new_task);
    let after = session
        .style_execution()
        .sheet_sets(&session.dom(), installation.document().node(), &cancel())
        .unwrap();
    assert_eq!(after.preferred_name(), names.preferred_name());
    assert_eq!(after.last_selected_name(), names.last_selected_name());
    assert_eq!(after.revision(), names.revision());
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
    for _ in 0..2 {
        assert!(
            matches!(DomStyleRuntime::run_networking(&mut session.dom(),&mut dispatcher,&mut context,dispatch_limits(),&cancel()).unwrap(),
            DomStyleNetworkingProgress::OriginalSynchronousPending{task,requirement:DomParsedHtmlStyleNotificationRequirement::Occurrence{kind:Kind::ChangePreferred,..}} if task==old_task)
        );
    }
    assert!(context.events.is_empty());
    let state = session.state.borrow();
    assert_eq!(state.completions[0].task().id(), old_task);
    assert_eq!(state.completions[0].task().sheet, *old_sheet.id());
    assert_eq!(
        state.completions[0].processing().unwrap().qualification,
        old
    );
    assert_eq!(state.completions[1].task().id(), new_task);
    assert_eq!(state.completions[1].task().sheet, *new_sheet.id());
    assert_eq!(state.completions[1].processing().unwrap().qualification, q);
    assert!(
        state
            .completions
            .iter()
            .all(|row| matches!(row.task().delivery, Delivery::NotStarted) && !row.task().running)
    );
    assert_eq!(state.next_task, 3);
}

#[test]
fn nonempty_recreation_css_input_refusal_and_cancellation_preserve_retry_and_both_receipts() {
    let (mut session, installation, old, old_sheet) =
        created_first("<!doctype html><style title=chosen>p{color:red}</style>");
    let outcome = session
        .style_execution()
        .advance_parsed_style_successor(
            &mut session.dom(),
            old.contribution(),
            envelope(),
            &cancel(),
        )
        .unwrap();
    let q = outcome.qualification().clone();
    drop(outcome);
    decide(&mut session, &q, true);
    let stamp = session.snapshot().stamp();
    let revision = session
        .style_execution()
        .snapshot(installation.document().node())
        .unwrap()
        .unwrap()
        .revision()
        .clone();
    let mut runtime = session.style_execution();
    assert!(matches!(
        runtime.create_parsed_style_sheet(
            &mut session.dom(),
            q.contribution(),
            envelope(),
            &AtomicBool::new(true)
        ),
        Err(DomStyleParsedSheetError::Sheet(
            DomParsedHtmlStyleSheetError::Style(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::Cancelled)
            ))
        ))
    ));
    let mut small = limits();
    small.css_input.max_input_units = 0;
    runtime
        .set_limits(&session.dom(), small, &cancel())
        .unwrap();
    let work = session.state.borrow().work.get();
    assert!(matches!(
        create(&mut session, q.contribution(), envelope()),
        Err(DomStyleParsedSheetError::Css(CssUtf16InputError::Limit(
            "original UTF16 units"
        )))
    ));
    assert!(session.state.borrow().work.get() > work);
    assert_eq!(session.snapshot().stamp(), stamp);
    let actual = runtime
        .snapshot(installation.document().node())
        .unwrap()
        .unwrap();
    assert_eq!(actual.revision(), &revision);
    assert!(actual.sheets().is_empty());
    assert_eq!(
        session
            .parsed_html_style_prefix(q.contribution())
            .unwrap()
            .phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    assert_eq!(session.state.borrow().completions.len(), 1);
    assert_eq!(
        session.state.borrow().completions[0]
            .processing()
            .unwrap()
            .sheet
            .as_ref(),
        Some(&old_sheet)
    );
    runtime
        .set_limits(&session.dom(), limits(), &cancel())
        .unwrap();
    let created = create(&mut session, q.contribution(), envelope()).unwrap();
    assert_ne!(created.completion().sheet().id(), old_sheet.id());
    assert_eq!(session.state.borrow().completions.len(), 2);
    assert_eq!(session.state.borrow().next_task, 1);
    assert_eq!(
        session
            .state
            .borrow()
            .roots
            .iter()
            .find(|root| root.node == installation.document().node())
            .unwrap()
            .serial,
        2
    );
}
