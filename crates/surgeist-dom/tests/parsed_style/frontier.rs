//! Authentic synchronous end, independent of task/resource delivery.
use super::*;
use surgeist_html::{HtmlStyleExecutionProfile, HtmlStyleTrigger};
use surgeist_html_parser::HtmlParseAssociationOccurrenceKind;

fn headers(dom: &Dom) -> Vec<(DomParsedHtmlAssociationId, DomParsedHtmlExecutionState)> {
    let mut cursor = dom.parsed_html_dispatch_cursor();
    let mut result = Vec::new();
    // This public test's independent read allowance is charged before every
    // actual advance, just as a composing dispatcher's existing work budget is.
    for _ in 0..100 {
        let Some(entry) = cursor.advance(&flag()).unwrap() else {
            return result;
        };
        result.push((entry.contribution(), entry.state()));
    }
    panic!("bounded header read exhausted")
}
fn blocked_pair(source: &str) -> (Dom, DomParsedHtmlAssociationId) {
    let (mut dom, _, id, _, _) = recreation::blocked(source, envelope());
    recreation::no_sheet(&mut dom, id, envelope());
    let mut observation = Some(recreation::observed(&dom, id));
    assert!(!observation.as_ref().unwrap().allowed());
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    (dom, id)
}
fn next_sheet(dom: &mut Dom, id: DomParsedHtmlAssociationId, root: &DomStyleCssomRoot) {
    let q = qualifier(dom, id);
    let mut observation = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    hold.accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(root, &flag()).unwrap();
    create::stage(&mut prep);
    let (created, retired) = prep.commit(&flag()).unwrap().into_parts();
    drop(retired);
    assert_eq!(
        created.phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    hold.exclude_script_bookkeeping(
        &q,
        HtmlStyleExecutionProfile::NoExecutableJavaScript,
        &flag(),
    )
    .unwrap();
    assert!(matches!(
        hold.process_rendering(
            &q,
            surgeist_dom::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag()
        )
        .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
}
#[test]
fn preparation_and_binding_are_unstarted_even_when_an_immutable_request_loan_is_forgotten() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<style>p{color:red}</style>"), env);
    decision::ready(&mut dom, id, envelope());
    assert_eq!(
        headers(&dom),
        [(id, DomParsedHtmlExecutionState::Unstarted)]
    );
    assert!(dom.parsed_html_synchronous_end(id).unwrap().is_none());
    let view = dom.parsed_html_style_prefix(id).unwrap();
    std::mem::forget(view);
    let mut cursor = dom.parsed_html_dispatch_cursor();
    let entry = cursor.advance(&flag()).unwrap().unwrap();
    assert_eq!(entry.state(), DomParsedHtmlExecutionState::Unstarted);
    assert!(entry.requirement(&flag()).unwrap().is_none());
}
#[test]
fn accepted_blocked_decision_activates_until_both_real_invocations_complete() {
    let (mut dom, _, id, _, _) = recreation::blocked("<style>blocked</style>", envelope());
    assert_eq!(
        headers(&dom),
        [(id, DomParsedHtmlExecutionState::Activated)]
    );
    let remaining = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag());
    assert!(matches!(
        remaining,
        Err(DomParsedHtmlFrontierError::Remaining(
            DomParsedHtmlStyleNotificationRequirement::Occurrence {
                kind: HtmlParseAssociationOccurrenceKind::StyleUpdate(
                    HtmlStyleTrigger::ParserPopped
                ),
                ..
            }
        ))
    ));
    recreation::no_sheet(&mut dom, id, envelope());
    let mut observation = Some(recreation::observed(&dom, id));
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    let before = dom.snapshot().stamp();
    let end = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
        .unwrap();
    assert_eq!(end.contribution(), id);
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        headers(&dom),
        [(id, DomParsedHtmlExecutionState::SynchronousEnd)]
    );
    assert_eq!(
        dom.parsed_html_synchronous_end(id)
            .unwrap()
            .unwrap()
            .contribution(),
        id
    );
    assert_eq!(
        dom.pending_parsed_html_association(qualifier(&dom, id).document())
            .unwrap(),
        Some(id)
    );
    drop(end);
    // End and unstarted header reads do not borrow the original graph.
    let view = dom.parsed_html_style_prefix(id).unwrap();
    std::mem::forget(view);
    assert!(dom.parsed_html_synchronous_end(id).unwrap().is_some());
    let mut cursor = dom.parsed_html_dispatch_cursor();
    let entry = cursor.advance(&flag()).unwrap().unwrap();
    assert!(entry.requirement(&flag()).unwrap().is_none());
}
#[test]
fn a_later_real_meta_remains_exact_work_after_two_blocked_style_returns() {
    let (mut dom, id) =
        blocked_pair("<style>blocked</style><meta http-equiv=default-style content=A>");
    let result = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag());
    assert!(matches!(
        result,
        Err(DomParsedHtmlFrontierError::Remaining(
            DomParsedHtmlStyleNotificationRequirement::Occurrence {
                kind: HtmlParseAssociationOccurrenceKind::ChangePreferred,
                ..
            }
        ))
    ));
    assert_eq!(
        headers(&dom),
        [(id, DomParsedHtmlExecutionState::Activated)]
    );
    assert!(dom.parsed_html_synchronous_end(id).unwrap().is_none());
}
#[test]
fn queued_predecessor_reaches_end_before_delivery_and_real_cleanup_unblocks_current_document_once()
{
    let mut value = rendering::ready(
        "<!doctype html><head><style>a{color:red}</style></head><body>x",
        envelope(),
    );
    value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap()
        .process_rendering(
            &value.q,
            surgeist_dom::DomParsedHtmlStyleNotificationHandoff::Transfer(&mut value.notification),
            &flag(),
        )
        .unwrap();
    let mut token = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        let mut published = hold
            .prepare_notification(&mut value.notification, "owner", &value.root, &flag())
            .unwrap()
            .commit(&flag())
            .unwrap();
        let DomParsedHtmlStyleNotificationDisposition::Token(token) =
            published.take_disposition().unwrap()
        else {
            panic!("actual owner")
        };
        token
    };
    assert!(
        value
            .dom
            .parsed_style_notification_requirement(&token, &flag())
            .unwrap()
            .is_some()
    );
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        let (next, retired) = hold
            .prepare_successor(Some(&value.root), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
        drop(next);
    }
    next_sheet(&mut value.dom, value.id, &value.root);
    let before = value.root.snapshot().unwrap().revision().clone();
    value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
        .unwrap();
    assert!(
        value
            .dom
            .parsed_style_notification_requirement(&token, &flag())
            .unwrap()
            .is_none()
    );
    assert!(matches!(
        value.dom.finish_style_notification(&mut token, 0, &flag()),
        Err(DomError::WorkLimit {
            resource: DomResource::ParsedAssociationDispatchWork,
            limit: 0
        })
    ));
    // An authentic second Document starts after this task's own End. Its
    // unfinished synchronous execution blocks the whole owner, not just itself.
    let (_, env) = setup();
    let (_, other) = install(&mut value.dom, parse("<style>blocked</style>"), env);
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(other, envelope(), &flag())
            .unwrap();
        hold.advance(&flag()).unwrap();
    }
    let q = binding(&value.dom, other);
    {
        let mut policy = Some(recreation::policy());
        let mut hold = value
            .dom
            .begin_parsed_html_style(other, envelope(), &flag())
            .unwrap();
        hold.bind_initial_policy(&q, &mut policy, &flag()).unwrap();
        hold.advance(&flag()).unwrap();
    }
    let mut observation = Some(recreation::observed(&value.dom, other));
    value
        .dom
        .begin_parsed_html_style(other, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    assert!(
        value
            .dom
            .parsed_style_notification_requirement(&token, &flag())
            .unwrap()
            .is_none(),
        "own end is insufficient for owner-wide dispatch"
    );
    assert!(
        matches!(value.dom.finish_style_notification(&mut token,1024,&flag()),Err(DomError::ParsedAssociationSynchronousPending{pending}) if pending==other)
    );
    recreation::no_sheet(&mut value.dom, other, envelope());
    let mut observation = Some(recreation::observed(&value.dom, other));
    value
        .dom
        .begin_parsed_html_style(other, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    value
        .dom
        .begin_parsed_html_style(other, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
        .unwrap();
    // Two actual canonical headers plus the final end advance, all prepaid.
    assert!(matches!(
        value.dom.finish_style_notification(&mut token, 2, &flag()),
        Err(DomError::WorkLimit {
            resource: DomResource::ParsedAssociationDispatchWork,
            limit: 2
        })
    ));
    let cleanup = value
        .dom
        .finish_style_notification(&mut token, 3, &flag())
        .unwrap();
    let DomStyleNotificationCompletion::ParsedComplete(cleanup) = cleanup else {
        panic!("real parsed custody")
    };
    assert_eq!(cleanup.dom().changes().to(), value.dom.snapshot().stamp());
    assert_eq!(
        value.root.snapshot().unwrap().revision(),
        &before,
        "cleanup never republishes a producer"
    );
    assert!(matches!(
        value
            .dom
            .finish_style_notification(&mut token, 1024, &flag()),
        Err(DomError::StyleProtocol)
    ));
}
#[test]
fn end_readback_rejects_foreign_identity_and_cancelled_end_keeps_activation() {
    let (mut dom, id) = blocked_pair("<style>blocked</style>");
    let (other, _) = setup();
    assert!(matches!(
        other.parsed_html_synchronous_end(id),
        Err(DomParsedHtmlAssociationAdmissionError::Dom(
            DomError::ForeignOwner
        ))
    ));
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let before = hold.spent_work();
    assert!(matches!(
        hold.complete_synchronous_execution(&AtomicBool::new(true)),
        Err(DomParsedHtmlFrontierError::Style(
            DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                DomError::Cancelled
            ))
        ))
    ));
    assert_eq!(hold.spent_work(), before);
    drop(hold);
    assert_eq!(
        headers(&dom),
        [(id, DomParsedHtmlExecutionState::Activated)]
    );
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .complete_synchronous_execution(&flag())
        .unwrap();
}
#[test]
fn final_end_output_refusal_keeps_active_then_raise_seals_without_repeating_decisions() {
    let limits = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work(),
        4,
    )
    .unwrap();
    let (mut dom, _, id, _, _) = recreation::blocked("<style>blocked</style>", limits);
    recreation::no_sheet(&mut dom, id, limits);
    let mut observation = Some(recreation::observed(&dom, id));
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .accept_inline_policy(&mut observation, &flag())
        .unwrap();
    assert!(observation.is_none());
    let before = dom.snapshot().stamp();
    let spent = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.complete_synchronous_execution(&flag()),
            Err(DomParsedHtmlFrontierError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::ParsedAssociationProcessingOutputs,
                        limit: 4
                    }
                ))
            ))
        ));
        hold.spent_work()
    };
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(dom.parsed_html_synchronous_end(id).unwrap().is_none());
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() >= spent);
    assert_eq!(
        hold.complete_synchronous_execution(&flag())
            .unwrap()
            .contribution(),
        id
    );
}
