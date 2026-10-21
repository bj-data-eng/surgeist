//! Real original ParserPopped progress after created and blocked predecessors.
use super::super::meta::cancel;
use super::super::*;
use super::consumption::{envelope, report};
use super::style_decision::{empty_policy, installed, observe, request_ready};
use super::style_sheet::{bind, create, pending, ready};
use surgeist_dom::*;
use surgeist_html::HtmlStyleTrigger;
use surgeist_html_parser::{
    HtmlParseAssociationOccurrenceKind as Kind, HtmlParsePendingAssociationStep as Step,
};

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
    request_ready(&mut session, id, empty_policy(), envelope());
    let old = session
        .parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone();
    let expected = next_parser_pop(&session, &old);
    let mut decision = observe(&session, id, &mut |request| {
        assert!(request.text().is_empty());
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            request, false,
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
