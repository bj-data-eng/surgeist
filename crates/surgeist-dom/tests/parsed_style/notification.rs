//! Genuine original creation/rendering then guarded queue permission, no dispatch witness.
use super::rendering::{Ready, ready};
use super::*;
use surgeist_html::HtmlStyleTrigger;
use surgeist_html_parser::HtmlParseAssociationOccurrenceKind;

fn rendered(outputs: usize) -> Ready {
    let limits =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, outputs).unwrap();
    let mut value = ready(
        "<!doctype html><head><style>later</style></head><body>final",
        limits,
    );
    assert!(matches!(
        value
            .dom
            .begin_parsed_html_style(value.id, limits, &flag())
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
    let prep = hold
        .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
        .unwrap();
    assert!(!prep.owner_association_removed());
    let mut published = prep.commit(&flag()).unwrap();
    assert!(matches!(
        value.root.snapshot(),
        Err(DomStyleCssomRootError::Busy)
    ));
    let DomParsedHtmlStyleNotificationDisposition::Token(token) =
        published.take_disposition().unwrap()
    else {
        panic!("actual live owner")
    };
    assert!(published.take_disposition().is_none());
    assert!(
        matches!(value.root.snapshot(), Err(DomStyleCssomRootError::Busy)),
        "taking does not unlock"
    );
    drop(published);
    token
}
fn assert_next(value: &Ready, token: &DomStyleNotificationToken) {
    let requirement = value
        .dom
        .parsed_style_notification_requirement(token, &flag())
        .unwrap()
        .expect("genuine ParserPopped remains");
    let DomParsedHtmlStyleNotificationRequirement::Occurrence {
        contribution,
        occurrence,
        document,
        kind,
    } = requirement
    else {
        panic!("real parser-pop synchronous work remains")
    };
    assert_eq!(contribution, value.id);
    assert_eq!(document, value.document);
    assert_ne!(&occurrence, value.q.occurrence());
    assert_eq!(
        kind,
        HtmlParseAssociationOccurrenceKind::StyleUpdate(HtmlStyleTrigger::ParserPopped)
    );
}
#[test]
fn actual_parsed_origin_is_once_only_and_remaining_parser_pop_blocks_direct_cleanup() {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<DomStyleNotificationToken>();
    let mut value = rendered(1024);
    let before = value.dom.snapshot().stamp();
    let mut token = issue(&mut value);
    assert!(
        matches!(token.origin(),DomStyleNotificationOrigin::ParsedOccurrence{contribution,occurrence} if *contribution==value.id && occurrence==value.q.occurrence())
    );
    assert_eq!(token.association(), value.association.association());
    assert_next(&value, &token);
    assert!(
        matches!(value.dom.finish_style_notification(&mut token, 1024, &flag()),Err(DomError::ParsedAssociationSynchronousPending{pending}) if pending==value.id)
    );
    assert_eq!(value.dom.snapshot().stamp(), before);
    assert_next(&value, &token);
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .notification_status(),
        Some(DomParsedHtmlStyleNotificationStatus::Issued)
    );
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_notification(&value.q,&value.association,"owner",&value.root,&flag()),Err(DomParsedHtmlStyleNotificationError::AlreadyIssued{contribution}) if contribution==value.id)
    );
}
#[test]
fn actual_prequeue_owner_removal_is_terminal_but_postqueue_removal_preserves_permission() {
    for queue_first in [false, true] {
        let mut value = rendered(1024);
        let token = queue_first.then(|| issue(&mut value));
        let snapshot = value.root.snapshot().unwrap();
        let mut batch = Some(
            value
                .root
                .batch(surgeist_cssom::CssomEditGuard::from_snapshot(&snapshot))
                .unwrap(),
        );
        assert!(batch.as_mut().unwrap().remove_sheet(&value.sheet).unwrap());
        value.root.commit(&mut batch).unwrap();
        if let Some(token) = token {
            assert_next(&value, &token);
            assert_eq!(
                value
                    .dom
                    .parsed_html_style_prefix(value.id)
                    .unwrap()
                    .notification_status(),
                Some(DomParsedHtmlStyleNotificationStatus::Issued)
            );
        } else {
            let mut hold = value
                .dom
                .begin_parsed_html_style(value.id, envelope(), &flag())
                .unwrap();
            let prep = hold
                .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
                .unwrap();
            assert!(prep.owner_association_removed());
            let mut published = prep.commit(&flag()).unwrap();
            assert!(
                matches!(published.take_disposition(),Some(DomParsedHtmlStyleNotificationDisposition::OwnerAssociationRemoved{association}) if association==value.association.association())
            );
            drop(published);
            drop(hold);
            assert_eq!(
                value
                    .dom
                    .parsed_html_style_prefix(value.id)
                    .unwrap()
                    .notification_status(),
                Some(DomParsedHtmlStyleNotificationStatus::OwnerAssociationRemoved)
            );
        }
    }
}
#[test]
fn wrong_label_foreign_owner_cancellation_and_dropped_preparation_do_not_issue() {
    let mut value = rendered(1024);
    let other = rendered(1024);
    let wrong = create::root(value.document);
    let before = value.dom.snapshot().stamp();
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.prepare_notification(&value.q, &value.association, "wrong", &value.root, &flag()),
            Err(DomParsedHtmlStyleNotificationError::MismatchedLabel { .. })
        ));
        assert!(matches!(
            hold.prepare_notification(&other.q, &value.association, "owner", &value.root, &flag()),
            Err(DomParsedHtmlStyleNotificationError::Style(
                DomParsedHtmlStyleError::WrongRequest
            ))
        ));
        assert!(matches!(
            hold.prepare_notification(&value.q, &value.association, "owner", &wrong, &flag()),
            Err(DomParsedHtmlStyleNotificationError::MismatchedOwner { .. })
        ));
        let prep = hold
            .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
            .unwrap();
        drop(prep);
        let prep = hold
            .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
            .unwrap();
        assert!(matches!(
            prep.commit(&AtomicBool::new(true)),
            Err(DomParsedHtmlStyleNotificationError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::Cancelled
                ))
            ))
        ));
    }
    assert_eq!(value.dom.snapshot().stamp(), before);
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .notification_status(),
        Some(DomParsedHtmlStyleNotificationStatus::NotIssued)
    );
    let mut token = issue(&mut value);
    assert!(matches!(
        other
            .dom
            .parsed_style_notification_requirement(&token, &flag()),
        Err(DomError::ForeignOwner)
    ));
    let mut foreign = other.dom;
    assert!(matches!(
        foreign.finish_style_notification(&mut token, 1024, &flag()),
        Err(DomError::ForeignOwner)
    ));
    assert_next(&value, &token);
}
#[test]
fn final_output_refusal_retains_original_issuance_for_raised_retry() {
    let mut value = rendered(8);
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 8).unwrap();
    let spent = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, limits, &flag())
            .unwrap();
        let prep = hold
            .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
            .unwrap();
        assert!(matches!(
            prep.commit(&flag()),
            Err(DomParsedHtmlStyleNotificationError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::ParsedAssociationProcessingOutputs,
                        limit: 8
                    }
                ))
            ))
        ));
        hold.spent_work()
    };
    assert_eq!(
        value
            .dom
            .parsed_html_style_prefix(value.id)
            .unwrap()
            .notification_status(),
        Some(DomParsedHtmlStyleNotificationStatus::NotIssued)
    );
    {
        let hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert!(hold.spent_work() > spent);
    }
    let token = issue(&mut value);
    assert_next(&value, &token);
}

#[test]
fn expectation_loans_end_before_publication_without_releasing_the_actual_root_guard() {
    let mut value = rendered(1024);
    let mut hold = value
        .dom
        .begin_parsed_html_style(value.id, envelope(), &flag())
        .unwrap();
    let prep = {
        let expected = value.association.clone();
        let label = String::from("owner");
        hold.prepare_notification(&value.q, &expected, &label, &value.root, &flag())
            .unwrap()
    };
    let mut published = prep.commit(&flag()).unwrap();
    assert!(matches!(
        published.take_disposition(),
        Some(DomParsedHtmlStyleNotificationDisposition::Token(_))
    ));
    assert!(matches!(
        value.root.snapshot(),
        Err(DomStyleCssomRootError::Busy)
    ));
}

#[test]
fn readback_output_refusal_keeps_the_same_token_for_raised_original_policy() {
    let mut value = rendered(9);
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 9).unwrap();
    let token = {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, limits, &flag())
            .unwrap();
        let mut published = hold
            .prepare_notification(&value.q, &value.association, "owner", &value.root, &flag())
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
    assert!(matches!(
        value
            .dom
            .parsed_style_notification_requirement(&token, &flag()),
        Err(DomError::ResourceLimit {
            resource: DomResource::ParsedAssociationProcessingOutputs,
            limit: 9
        })
    ));
    {
        let mut hold = value
            .dom
            .begin_parsed_html_style(value.id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::RenderingComplete
        );
    }
    assert_next(&value, &token);
}
