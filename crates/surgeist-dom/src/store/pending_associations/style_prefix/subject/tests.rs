//! Actual same-entry handoff custody, not a fabricated completion registry.
use super::super::rendering::tests::{ready, ready_source};
use super::super::tests::{envelope, flag};
use super::*;
use crate::*;
use surgeist_css::query::media::CssMediaQueryList;
use surgeist_css::source::CssUtf16InputLimits;
use surgeist_css::{CssParser, CssParserContext};
use surgeist_cssom::*;

fn render(dom: &mut Dom, id: DomParsedHtmlAssociationId, q: &DomParsedHtmlStyleQualification) {
    drop(
        dom.begin_parsed_html_style(id, envelope(), &flag())
            .unwrap()
            .process_rendering(q, DomParsedHtmlStyleNotificationHandoff::Retain, &flag())
            .unwrap(),
    );
}
fn create_and_render(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    q: &DomParsedHtmlStyleQualification,
    cap: &DomStyleCssomRoot,
) {
    let mut input = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    hold.accept_inline_policy(&mut input, &flag()).unwrap();
    let mut prep = hold.prepare_create_sheet(cap, &flag()).unwrap();
    let parser = CssParserContext::default();
    let sheet = CssParser::new(parser, Default::default())
        .parse_sheet_utf16(
            prep.request().text().shared_code_units().clone(),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    let media = CssMediaQueryList::parse_utf16(
        &CssParser::new(parser, Default::default()),
        prep.request().media().unwrap().shared_code_units().clone(),
        CssUtf16InputLimits::default(),
        || prep.step(&flag()),
    )
    .unwrap();
    let mut input = Some(DomParsedHtmlStyleSheetInput::new(
        sheet,
        media,
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "original".into(),
            revision: 1,
        },
        "owner".into(),
        false,
    ));
    prep.stage_sheet(&mut input, &flag()).unwrap();
    drop(prep.commit(&flag()).unwrap().into_parts());
    hold.exclude_script_bookkeeping(
        q,
        surgeist_html::HtmlStyleExecutionProfile::NoExecutableJavaScript,
        &flag(),
    )
    .unwrap();
    drop(
        hold.process_rendering(q, DomParsedHtmlStyleNotificationHandoff::Retain, &flag())
            .unwrap(),
    );
}
#[test]
fn historical_output_drop_preserves_work_and_only_explicit_transfer_abandons_it() {
    let (mut dom, _, id, q, _, _) = ready("", 1024);
    render(&mut dom, id, &q);
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .notification_handoff(),
        Some(DomParsedHtmlStyleNotificationHandoffStatus::Retained)
    );
    let ledger = dom.custody_registry.ledger().clone();
    let mut receiver = None;
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.transfer_notification(&q, &mut receiver, &flag())
            .unwrap();
        assert_eq!(receiver.as_ref().unwrap().qualification(), &q);
        assert!(matches!(
            hold.transfer_notification(&q, &mut receiver, &flag()),
            Err(DomParsedHtmlStyleError::ReceiverOccupied)
        ));
    }
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .notification_handoff(),
        Some(DomParsedHtmlStyleNotificationHandoffStatus::Transferred)
    );
    let before = ledger.total().bytes();
    drop(receiver);
    assert!(ledger.total().bytes() < before);
    let mut empty = None;
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(matches!(
        hold.transfer_notification(&q, &mut empty, &flag()),
        Err(DomParsedHtmlStyleError::NotificationTransferred)
    ));
    assert!(matches!(
        hold.complete_synchronous_execution(&flag()),
        Err(DomParsedHtmlFrontierError::Remaining(_))
    ));
}
#[test]
fn subject_transition_requires_both_real_handoffs_and_retires_request_only_after_fence() {
    let (mut dom, document, id, q0, _, cap) = ready_source(
        "<!doctype html><head><style media=''>A{}</style><style media='' nonce=c>C{}</style><style media=''>D{}</style>",
        1024,
    );
    render(&mut dom, id, &q0);
    let q1 = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let (next, retired) = hold
            .prepare_successor(Some(&cap), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
        next.qualification().clone()
    };
    create_and_render(&mut dom, id, &q1, &cap);
    let next = {
        let entry = dom
            .pending_associations
            .entries
            .get(usize::try_from(id.stamp().revision().get()).unwrap())
            .unwrap();
        let controlled = dom.pending_associations.controlled_entry(entry.id).unwrap();
        let meter = Meter::new(controlled, dom.custody_registry.ledger().clone());
        frontier::scan(&dom, &meter, None, &flag())
            .unwrap()
            .next
            .unwrap()
            .id()
    };
    let before = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let mut a0 = None;
    let mut a1 = None;
    let q2 = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.advance_subject(&next, &flag()),
            Err(Error::Style(
                DomParsedHtmlStyleError::HandoffRequired { .. }
            ))
        ));
        hold.transfer_notification(&q1, &mut a1, &flag()).unwrap();
        assert!(matches!(
            hold.advance_subject(&next, &flag()),
            Err(Error::Style(
                DomParsedHtmlStyleError::HandoffRequired { .. }
            ))
        ));
        hold.transfer_notification(&q0, &mut a0, &flag()).unwrap();
        assert!(matches!(
            hold.advance_subject(&next, &AtomicBool::new(true)),
            Err(Error::Style(DomParsedHtmlStyleError::Admission(_)))
        ));
        let output = hold.advance_subject(&next, &flag()).unwrap();
        assert_ne!(output.qualification().subject(), q1.subject());
        assert_eq!(output.qualification().document(), document);
        output.qualification().clone()
    };
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert_eq!(a0.as_ref().unwrap().qualification(), &q0);
    assert_eq!(a1.as_ref().unwrap().qualification(), &q1);
    let view = dom.parsed_html_style_prefix(id).unwrap();
    assert_eq!(view.inline_policy_request().unwrap().qualification(), &q2);
    assert_eq!(
        view.inline_policy_request().unwrap().text().code_units(),
        []
    );
    assert_eq!(
        view.inline_policy_request()
            .unwrap()
            .nonce()
            .unwrap()
            .code_units(),
        [99]
    );
}

#[test]
fn diagnostic_mapping_preserves_actual_original_work_refusal() {
    let (mut dom, _, id, q, _, _) = ready("", 1024);
    let hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let mut rows = hold
        .meter
        .entry()
        .report
        .value()
        .value
        .pending_associations();
    let row = loop {
        match rows.advance() {
            Step::Occurrence(row) if row.id() == *q.occurrence() => break row,
            Step::Complete => panic!("authentic original occurrence"),
            _ => (),
        }
    };
    while hold.meter.entry().control.spent_work() < hold.limits().work() {
        hold.step(&flag()).unwrap();
    }
    assert!(matches!(
        frontier::diagnostic(&hold.meter, &row, None, &flag()),
        Err(DomError::WorkLimit {
            resource: DomResource::ParsedAssociationProcessingWork,
            ..
        })
    ));
}

#[test]
fn late_transfer_output_refusal_keeps_canonical_work_for_actual_raised_retry() {
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 8).unwrap();
    let (mut dom, _, id, q, _, _) = ready("", 8);
    drop(
        dom.begin_parsed_html_style(id, limits, &flag())
            .unwrap()
            .process_rendering(&q, DomParsedHtmlStyleNotificationHandoff::Retain, &flag())
            .unwrap(),
    );
    let mut input = None;
    let spent = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.transfer_notification(&q, &mut input, &flag()),
            Err(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 8
                })
            ))
        ));
        assert!(input.is_none());
        hold.spent_work()
    };
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .notification_handoff(),
        Some(DomParsedHtmlStyleNotificationHandoffStatus::Retained)
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() > spent);
    hold.transfer_notification(&q, &mut input, &flag()).unwrap();
    assert_eq!(input.as_ref().unwrap().qualification(), &q);
}
