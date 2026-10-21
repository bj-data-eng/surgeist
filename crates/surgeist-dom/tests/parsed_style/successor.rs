//! Actual ParserPopped continuation, independently of asynchronous notification work.
use super::rendering::{Ready, ready};
use super::*;
use surgeist_cssom::CssomSheetId;

fn removed(
    completion: &DomParsedHtmlStyleSuccessorCompletion,
) -> (&DomCommit, &surgeist_cssom::CssomCommit) {
    let DomParsedHtmlStyleSuccessorRemoval::Removed { dom, cssom } = completion.removal() else {
        panic!("actual old sheet removed")
    };
    (dom, cssom)
}
fn result_revision(commit: &surgeist_cssom::CssomCommit) -> &surgeist_cssom::CssomRevision {
    match commit.publication() {
        surgeist_cssom::CssomPublication::Changed(v) => v.to(),
        surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
    }
}
pub(super) fn rendered() -> Ready {
    let mut value = ready(
        "<!doctype html><head><style nonce='original'>a{color:red}</style></head><body>final",
        envelope(),
    );
    assert!(matches!(
        value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap()
            .process_rendering(&value.q, &flag())
            .unwrap(),
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    value
}
fn issue(value: &mut Ready) -> DomStyleNotificationToken {
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    let mut published = hold
        .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
        .unwrap()
        .commit(&flag())
        .unwrap();
    let DomParsedHtmlStyleNotificationDisposition::Token(token) =
        published.take_disposition().unwrap()
    else {
        panic!("live original owner")
    };
    token
}
fn advance(value: &mut Ready) -> DomParsedHtmlStyleSuccessorCompletion {
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert_eq!(
        hold.successor_owner_document(&flag()).unwrap(),
        Some(value.document)
    );
    let prep = hold.prepare_successor(Some(&value.root), &flag()).unwrap();
    assert_ne!(prep.qualification().occurrence(), value.q.occurrence());
    let published = prep.commit(&flag()).unwrap();
    assert!(matches!(
        value.root.snapshot(),
        Err(DomStyleCssomRootError::Busy)
    ));
    let (completion, retirement) = published.into_parts();
    drop(retirement);
    completion
}
fn old_is_readable_and_removed(value: &Ready, old: &CssomSheetId) {
    let snapshot = value.root.snapshot().unwrap();
    let sheet = snapshot.sheet(old).unwrap();
    assert!(sheet.inputs().owner.is_none());
    assert!(sheet.owner_rule().is_none());
    assert!(!snapshot.sheets().contains(old));
    assert_eq!(
        value
            .dom
            .pending_parsed_html_association(value.document)
            .unwrap(),
        Some(value.id)
    );
    assert!(
        value
            .dom
            .snapshot()
            .node(value.element)
            .unwrap()
            .as_style()
            .unwrap()
            .sheet()
            .is_err()
    );
}
#[test]
fn parser_pop_removes_real_sheet_and_forms_distinct_original_full_text_request() {
    let mut value = rendered();
    let before = value.dom.snapshot().stamp();
    let old = value.sheet.clone();
    assert!(
        value
            .root
            .snapshot()
            .unwrap()
            .sheet_rules(&old)
            .unwrap()
            .is_empty(),
        "first insertion parsed its authentic empty input"
    );
    let completion = advance(&mut value);
    assert_eq!(
        completion.phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    assert_eq!(removed(&completion).0.changes().from(), before);
    assert!(!removed(&completion).0.changes().changes().is_empty());
    assert_eq!(
        result_revision(removed(&completion).1),
        value.root.snapshot().unwrap().revision()
    );
    old_is_readable_and_removed(&value, &old);
    let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
    let request = view.inline_policy_request().unwrap();
    assert_eq!(
        request.text().code_units(),
        "a{color:red}".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(
        request.nonce().unwrap().code_units(),
        "original".encode_utf16().collect::<Vec<_>>()
    );
    assert_ne!(request.qualification().occurrence(), value.q.occurrence());
    assert_ne!(
        request.qualification().context_epoch(),
        value.q.context_epoch()
    );
    assert_eq!(request.qualification().slot(), value.q.slot());
    assert_eq!(request.qualification(), completion.qualification());
    assert!(
        view.script_bookkeeping_exclusion().is_none(),
        "the new cursor has not reached script bookkeeping"
    );
}
#[test]
fn queued_predecessor_keeps_execution_and_direct_cleanup_remains_blocked_by_new_request() {
    let mut value = rendered();
    let mut token = issue(&mut value);
    let completion = advance(&mut value);
    let requirement = value
        .dom
        .parsed_style_notification_requirement(&token, &flag())
        .unwrap()
        .expect("genuine new invocation remains");
    assert!(
        matches!(requirement, DomParsedHtmlStyleNotificationRequirement::Invocation { occurrence, phase: DomParsedHtmlStylePhase::InlinePolicyPending, .. } if &occurrence==completion.qualification().occurrence())
    );
    assert!(
        matches!(value.dom.finish_style_notification(&mut token, 1024, &flag()), Err(DomError::ParsedAssociationSynchronousPending{pending}) if pending==value.id)
    );
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.prepare_notification(&value.q, &value.association, "owner", &value.root, &flag()),
        Err(DomParsedHtmlStyleNotificationError::AlreadyIssued { .. })
    ));
}
#[test]
fn unqueued_old_receipt_can_terminate_after_successor_and_foreign_root_refuses() {
    let mut value = rendered();
    advance(&mut value);
    let other = rendered();
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.prepare_notification(&value.q, &other.association, "owner", &value.root, &flag()),
        Err(DomParsedHtmlStyleNotificationError::MismatchedOwner { root }) if root == value.document.node()
    ));
    let prep = hold
        .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
        .unwrap();
    assert!(prep.owner_association_removed());
    let mut published = prep.commit(&flag()).unwrap();
    assert!(
        matches!(published.take_disposition(), Some(DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved{association}) if association==value.association.association())
    );
}
#[test]
fn cancelled_and_dropped_preparations_leave_both_owners_and_current_request_unchanged() {
    let mut value = rendered();
    let before = value.dom.snapshot().stamp();
    let revision = value.root.snapshot().unwrap().revision().clone();
    let cancelled = std::sync::atomic::AtomicBool::new(true);
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert!(
            hold.prepare_successor(Some(&value.root), &cancelled)
                .is_err()
        );
        drop(hold.prepare_successor(Some(&value.root), &flag()).unwrap());
    }
    assert_eq!(value.dom.snapshot().stamp(), before);
    assert_eq!(value.root.snapshot().unwrap().revision(), &revision);
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .initial_policy_request(),
        Some(&value.q)
    );
    advance(&mut value);
}
#[test]
fn already_cleared_producer_still_clears_canonical_association_with_real_unchanged_commit() {
    let mut value = rendered();
    let snapshot = value.root.snapshot().unwrap();
    let mut batch = Some(
        value
            .root
            .batch(surgeist_cssom::CssomEditGuard::from_snapshot(&snapshot))
            .unwrap(),
    );
    batch.as_mut().unwrap().remove_sheet(&value.sheet).unwrap();
    value.root.commit(&mut batch).unwrap();
    let revision = value.root.snapshot().unwrap().revision().clone();
    let completion = advance(&mut value);
    assert_eq!(result_revision(removed(&completion).1), &revision);
    assert!(!removed(&completion).0.changes().changes().is_empty());
    old_is_readable_and_removed(&value, &value.sheet);
}
#[test]
fn new_policy_decision_is_required_before_authentic_successor_creation() {
    let mut value = rendered();
    advance(&mut value);
    let mut decision = {
        let view = value.dom.parsed_html_style_prefix(value.id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.prepare_successor(Some(&value.root), &flag()),
        Err(DomParsedHtmlStyleSuccessorError::Style(
            DomParsedHtmlStyleError::WrongPhase {
                phase: DomParsedHtmlStylePhase::InlinePolicyPending
            }
        ))
    ));
    hold.accept_inline_policy(&mut decision, &flag()).unwrap();
    let prepared = hold.prepare_create_sheet(&value.root, &flag()).unwrap();
    assert_eq!(
        prepared.request().text().code_units(),
        "a{color:red}".encode_utf16().collect::<Vec<_>>()
    );
}
#[test]
fn finite_successor_output_refusal_retains_original_work_and_raised_retry() {
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 8).unwrap();
    let mut value = ready(
        "<!doctype html><head><style>later</style></head><body>final",
        limits,
    );
    value
        .dom
        .begin_parsed_html_style(value.id, limits, &flag())
        .unwrap()
        .process_rendering(&value.q, &flag())
        .unwrap();
    let stamp = value.dom.snapshot().stamp();
    let revision = value.root.snapshot().unwrap().revision().clone();
    let spent = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, limits, &flag())
            .unwrap();
        let before = hold.spent_work();
        let result = hold
            .prepare_successor(Some(&value.root), &flag())
            .unwrap()
            .commit(&flag());
        assert!(matches!(
            result,
            Err(DomParsedHtmlStyleSuccessorError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::ParsedAssociationProcessingOutputs,
                        limit: 8
                    }
                ))
            ))
        ));
        assert!(hold.spent_work() > before);
        hold.spent_work()
    };
    assert_eq!(value.dom.snapshot().stamp(), stamp);
    assert_eq!(value.root.snapshot().unwrap().revision(), &revision);
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .initial_policy_request(),
        Some(&value.q)
    );
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() >= spent);
    let (completion, retirement) = hold
        .prepare_successor(Some(&value.root), &flag())
        .unwrap()
        .commit(&flag())
        .unwrap()
        .into_parts();
    drop(retirement);
    assert_ne!(completion.qualification(), &value.q);
}
