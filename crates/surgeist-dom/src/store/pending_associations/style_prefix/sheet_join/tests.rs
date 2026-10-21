//! Real owning fence, original admission and forgotten-capability evidence.
use super::super::tests::{begin_await, binding, envelope, flag, install, parse, qualifier, setup};
use super::*;
use crate::{DomParsedHtmlAssociationAdmissionError, DomStyleCssomRootError};
use surgeist_css::{CssParserContext, CssUtf16InputLimits};
use surgeist_cssom::*;
use surgeist_html::{HtmlCspLayout, HtmlCspOwner};
fn ready() -> (Dom, DocumentId, DomParsedHtmlAssociationId) {
    ready_with("<!doctype html><style title=A media=screen nonce=n>p{color:red}</style>")
}
fn ready_with(source: &str) -> (Dom, DocumentId, DomParsedHtmlAssociationId) {
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    begin_await(&mut dom, id, envelope());
    let q = binding(&dom, id);
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut policy = Some(owner.initial_document_with(&mut allow).unwrap());
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        hold.bind_initial_policy(&q, &mut policy, &flag()).unwrap();
        hold.advance(&flag()).unwrap();
    }
    let mut observed = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observed, &flag())
        .unwrap();
    (dom, document, id)
}
fn root(document: DocumentId) -> DomStyleCssomRoot {
    DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    )
}
fn input(
    prep: &DomParsedHtmlStyleSheetPreparation<'_, '_, '_>,
) -> Option<DomParsedHtmlStyleSheetInput> {
    let parser = CssParserContext::default();
    let sheet = parser
        .parse_sheet_utf16_with_limits(
            prep.request().text().shared_code_units().clone(),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    let media = parser
        .parse_media_query_list_utf16_with_limits(
            prep.request().media().unwrap().shared_code_units().clone(),
            CssUtf16InputLimits::default(),
            || prep.step(&flag()),
        )
        .unwrap();
    Some(DomParsedHtmlStyleSheetInput::new(
        sheet,
        media,
        CssomInputVersion {
            role: CssomInputRole::Owner,
            identity: "original".into(),
            revision: 1,
        },
        "owner".into(),
        false,
    ))
}
#[test]
fn original_work_refuses_after_real_producer_prepare_and_raised_retry_publishes_once() {
    let (mut dom, document, id) = ready();
    let cap = root(document);
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    // Existing policy is raise-only: ready used the generous envelope. Create a
    // finite frontier by spending real steps up to its already-bound limit.
    let spent;
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        let mut supplied = input(&prep);
        prep.stage_sheet(&mut supplied, &flag()).unwrap();
        let control = &prep.meter.entry().control;
        // Compared with the former single-target array, actual target Vec
        // reservation adds three polls and its one placement adds one.
        assert_eq!(prep.order.len(), 1);
        while control.limits().work() - control.spent_work() > 9 {
            prep.step(&flag()).unwrap();
        }
        spent = control.spent_work();
        let before = PREPARED_FRONTIER.with(|n| n.get());
        assert!(matches!(
            prep.commit(&flag()),
            Err(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        ..
                    }
                ))
            ))
        ));
        assert_eq!(PREPARED_FRONTIER.with(|n| n.get()), before + 1);
    }
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert!(cap.snapshot().unwrap().sheets().is_empty());
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let raised = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 100_000,
        envelope().outputs(),
    )
    .unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() > spent);
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    let mut supplied = input(&prep);
    prep.stage_sheet(&mut supplied, &flag()).unwrap();
    let (completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
    assert_eq!(
        completion.phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
    drop(retirement);
    drop(hold);
    assert_eq!(
        cap.snapshot().unwrap().sheets(),
        &[completion.sheet().id().clone()]
    );
}
#[test]
fn post_construction_refusal_closes_the_same_editor_without_orphan_publication() {
    for source in [
        "<!doctype html><style title=A media=screen nonce=n>later</style>",
        "<!doctype html><style title=A media='screen and (' nonce=n>later</style>",
    ] {
        let (mut dom, document, id) = ready_with(source);
        let cap = root(document);
        let stamp = dom.snapshot().stamp();
        let before = cap.snapshot().unwrap().revision().clone();
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        let mut supplied = input(&prep);
        let mut next = input(&prep);
        // Admit a real temporary buffer before allocation, crowding out the
        // collector's first live scratch allocation after actual construction.
        let receipt = prep
            .meter
            .admission(prep.dom.custody_registry.total(), &flag())
            .unwrap();
        let remaining = receipt.remaining_bytes().unwrap();
        let scratch = prep.meter.fork();
        scratch.retain(remaining, 0, 0, &flag()).unwrap();
        let mut buffer = Vec::<u8>::new();
        buffer.try_reserve_exact(remaining).unwrap();
        assert!(matches!(
            prep.stage_sheet(&mut supplied, &flag()),
            Err(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::ResourceLimit {
                        resource: DomResource::ParsedAssociationProcessingRetainedBytes,
                        ..
                    }
                ))
            ))
        ));
        assert!(supplied.is_none());
        // Restore actual headroom, so rejection now proves the attempted editor
        // was closed instead of accidentally relying on an exhausted envelope.
        drop(buffer);
        drop(scratch);
        assert!(matches!(
            prep.stage_sheet(&mut next, &flag()),
            Err(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::InvalidSheetInput
            ))
        ));
        assert!(next.is_some());
        assert!(matches!(
            prep.commit(&flag()),
            Err(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::InvalidSheetInput
            ))
        ));
        drop(receipt);
        drop(hold);
        assert_eq!(dom.snapshot().stamp(), stamp);
        assert_eq!(cap.snapshot().unwrap().revision(), &before);
        assert!(cap.snapshot().unwrap().sheets().is_empty());
        assert_eq!(
            dom.parsed_html_style_prefix(id).unwrap().phase(),
            DomParsedHtmlStylePhase::CreateSheetPending
        );
    }
}
#[test]
fn actual_history_last_owner_survives_join_until_retirement_is_disposed() {
    let (mut dom, document, id) = ready();
    let mut limits = dom.limits();
    limits.history_entries = 1;
    dom.set_limits(limits).unwrap();
    let old = dom.history.oldest_changes_weak();
    assert_eq!(
        old.strong_count(),
        1,
        "installation output is gone; history owns the changes"
    );
    let ledger = dom.custody_registry.ledger().clone();
    let cap = root(document);
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    let mut supplied = input(&prep);
    prep.stage_sheet(&mut supplied, &flag()).unwrap();
    let published = prep.commit(&flag()).unwrap();
    assert!(old.upgrade().is_some());
    let (completion, retirement) = published.into_parts();
    drop(hold);
    assert!(old.upgrade().is_some());
    let retained = ledger.total();
    drop(retirement);
    assert!(old.upgrade().is_none());
    assert!(ledger.total().bytes() < retained.bytes());
    assert_eq!(
        completion.phase(),
        DomParsedHtmlStylePhase::ReadScriptBlockingPending
    );
}
#[test]
fn authentic_busy_poisoned_and_forgotten_editors_preserve_pending_qualification() {
    let (mut dom, document, id) = ready();
    let cap = root(document);
    let guard = cap.acquire().unwrap();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_create_sheet(&cap,&flag()),Err(DomParsedHtmlStyleSheetError::OwnerBusy {root}) if root==document.node())
    );
    drop(hold);
    drop(guard);
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = cap.acquire().unwrap();
        panic!("real owner poisoning");
    }));
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(hold.prepare_create_sheet(&cap,&flag()),Err(DomParsedHtmlStyleSheetError::OwnerPoisoned {root}) if root==document.node())
    );
    drop(hold);
    let cap = root(document);
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    let mut supplied = input(&prep);
    prep.stage_sheet(&mut supplied, &flag()).unwrap();
    std::mem::forget(prep);
    drop(hold);
    assert!(ledger.total().bytes() > before.bytes());
    assert!(
        matches!(dom.parsed_html_style_prefix(id),Err(DomParsedHtmlStyleError::Busy {contribution}) if contribution==id)
    );
    assert!(matches!(cap.snapshot(), Err(DomStyleCssomRootError::Busy)));
    assert!(
        matches!(dom.begin_parsed_html_style(id,envelope(),&flag()),Err(DomParsedHtmlStyleError::Busy {contribution}) if contribution==id)
    );
}

#[test]
fn original_control_refuses_the_actual_final_css_postgrammar_poll_then_raised_retry_succeeds() {
    use surgeist_css::CssUtf16InputError;
    let (mut dom, document, id) = ready();
    let cap = root(document);
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let text = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        prep.request().text().shared_code_units().clone()
    };
    let polls = std::cell::Cell::new(0u64);
    {
        let hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let report = CssParserContext::default()
            .parse_sheet_utf16_with_limits(text.clone(), Default::default(), || {
                polls.set(polls.get() + 1);
                hold.step(&flag())
            })
            .unwrap();
        assert!(report.syntax().rules().is_empty());
        drop(report);
        // The CSS UTF16 contract's last public poll is immediately after its
        // real grammar. Position finite original work, not a fake parser hook.
        while hold.limits().work() - hold.spent_work() > polls.get() - 1 {
            hold.step(&flag()).unwrap();
        }
        let calls = std::cell::Cell::new(0u64);
        let error = CssParserContext::default()
            .parse_sheet_utf16_with_limits(text.clone(), Default::default(), || {
                calls.set(calls.get() + 1);
                hold.step(&flag())
            })
            .unwrap_err();
        assert!(matches!(
            error,
            CssUtf16InputError::Interrupted(DomError::WorkLimit {
                resource: DomResource::ParsedAssociationProcessingWork,
                ..
            })
        ));
        assert_eq!(calls.get(), polls.get());
        assert_eq!(hold.spent_work(), hold.limits().work());
    }
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert!(cap.snapshot().unwrap().sheets().is_empty());
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
    let limits = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 10_000,
        envelope().outputs(),
    )
    .unwrap();
    let hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    assert!(hold.spent_work() >= envelope().work());
    let report = CssParserContext::default()
        .parse_sheet_utf16_with_limits(text, Default::default(), || hold.step(&flag()))
        .unwrap();
    assert!(report.syntax().rules().is_empty());
}
#[test]
fn successor_creation_late_refusal_preserves_removed_predecessor_then_retries_same_invocation() {
    let (mut dom, document, id, q, element, cap) = super::super::rendering::tests::ready("", 1024);
    let old = dom
        .snapshot()
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .sheet
        .clone()
        .unwrap();
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let (_completion, retired) = hold
            .prepare_successor(Some(&cap), &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
    }
    let mut observed = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut observed, &flag())
        .unwrap();
    let current = qualifier(&dom, id);
    let stamp = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let serial = dom
        .snapshot()
        .node(element)
        .unwrap()
        .as_style()
        .unwrap()
        .state()
        .next_association;
    let spent = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
        let mut report = input(&prep);
        prep.stage_sheet(&mut report, &flag()).unwrap();
        let control = &prep.meter.entry().control;
        // Compared with the former single-target array, actual target Vec
        // reservation adds three polls and its one placement adds one.
        assert_eq!(prep.order.len(), 1);
        while control.limits().work() - control.spent_work() > 9 {
            prep.step(&flag()).unwrap();
        }
        let count = PREPARED_FRONTIER.with(|value| value.get());
        assert!(matches!(
            prep.commit(&flag()),
            Err(DomParsedHtmlStyleSheetError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        ..
                    }
                ))
            ))
        ));
        assert_eq!(
            PREPARED_FRONTIER.with(|value| value.get()),
            count + 1,
            "actual producer preparation succeeded before late DOM refusal"
        );
        hold.spent_work()
    };
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
    assert_eq!(qualifier(&dom, id), current);
    assert_eq!(
        dom.snapshot()
            .node(element)
            .unwrap()
            .as_style()
            .unwrap()
            .state()
            .next_association,
        serial
    );
    assert!(cap.snapshot().unwrap().sheets().is_empty());
    assert!(
        cap.snapshot()
            .unwrap()
            .sheet(old.id())
            .unwrap()
            .inputs()
            .owner
            .is_none()
    );
    let raised = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 100_000,
        envelope().outputs(),
    )
    .unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() >= spent);
    let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
    let mut report = input(&prep);
    prep.stage_sheet(&mut report, &flag()).unwrap();
    let (created, retired) = prep.commit(&flag()).unwrap().into_parts();
    drop(retired);
    assert_ne!(created.sheet().id(), old.id());
    assert_ne!(created.sheet().association(), old.association());
    assert_eq!(created.sheet().root(), document.node());
}
