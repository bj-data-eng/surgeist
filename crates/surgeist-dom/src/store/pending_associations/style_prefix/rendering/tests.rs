//! Real canonical relation, original output admission and retirement lifetimes.
use super::super::tests::{begin_await, binding, envelope, flag, install, parse, setup};
use super::*;
use crate::*;
use surgeist_css::{CssMediaQueryList, CssParser, CssParserContext, CssUtf16InputLimits};
use surgeist_cssom::*;
use surgeist_html::HtmlCspOwner;
pub(in crate::store::pending_associations::style_prefix) fn ready(
    media: &str,
    outputs: usize,
) -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    DomParsedHtmlStyleQualification,
    NodeId,
    DomStyleCssomRoot,
) {
    ready_source(
        &format!("<!doctype html><head><style media='{media}'>later</style></head><body>final"),
        outputs,
    )
}
pub(in crate::store::pending_associations::style_prefix) fn ready_source(
    source: &str,
    outputs: usize,
) -> (
    Dom,
    DocumentId,
    DomParsedHtmlAssociationId,
    DomParsedHtmlStyleQualification,
    NodeId,
    DomStyleCssomRoot,
) {
    let limits =
        DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, outputs).unwrap();
    let (mut dom, env) = setup();
    let (document, id) = install(&mut dom, parse(source), env);
    begin_await(&mut dom, id, limits);
    let binding = binding(&dom, id);
    let q = binding.qualification().clone();
    let mut admit = |_| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut admit).unwrap();
    let mut policy = Some(owner.initial_document_with(&mut admit).unwrap());
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.bind_initial_policy(&binding, &mut policy, &flag())
            .unwrap();
        hold.advance(&flag()).unwrap();
    }
    let mut observed = {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        Some(DomParsedHtmlStyleInlinePolicyObservation::evaluated(
            view.inline_policy_request().unwrap(),
            true,
        ))
    };
    let cap = DomStyleCssomRoot::new(
        document.node(),
        CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let element = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.accept_inline_policy(&mut observed, &flag()).unwrap();
        let mut prep = hold.prepare_create_sheet(&cap, &flag()).unwrap();
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
        let element = prep.request().element();
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
        let (_completion, retirement) = prep.commit(&flag()).unwrap().into_parts();
        drop(retirement);
        element
    };
    dom.begin_parsed_html_style(id, limits, &flag())
        .unwrap()
        .exclude_script_bookkeeping(
            &q,
            HtmlStyleExecutionProfile::NoExecutableJavaScript,
            &flag(),
        )
        .unwrap();
    (dom, document, id, q, element, cap)
}
#[test]
fn completed_output_keeps_real_retired_state_history_and_original_frame_debit() {
    let (mut dom, document, id, q, element, _cap) = ready("", 1024);
    let mut limits = dom.limits();
    limits.history_entries = 1;
    dom.set_limits(limits).unwrap();
    let old_state = Arc::downgrade(&dom.state);
    let old_history = dom.history.oldest_changes_weak();
    let ledger = dom.custody_registry.ledger().clone();
    let before = dom.snapshot().stamp();
    let progress = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap();
    let DomParsedHtmlStyleRenderingProgress::Complete(done) = progress else {
        panic!("unconditional original head Style")
    };
    assert!(done.render_blocking_member());
    assert!(old_state.upgrade().is_some());
    assert!(old_history.upgrade().is_some());
    assert_eq!(done.dom().changes().from(), before);
    let snapshot = dom.snapshot();
    assert_eq!(
        snapshot
            .node(document.node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking
            .iter()
            .filter(|&&item| item == element)
            .count(),
        1
    );
    drop(snapshot);
    let retained = ledger.total();
    drop(done);
    assert!(old_state.upgrade().is_none());
    assert!(old_history.upgrade().is_none());
    assert!(ledger.total().bytes() < retained.bytes());
}
#[test]
fn final_output_refusal_discards_prepared_relation_and_retries_without_duplicate_membership() {
    let (mut dom, document, id, q, element, cap) = ready("", 6);
    let policy = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 6).unwrap();
    let before = dom.snapshot().stamp();
    let revision = cap.snapshot().unwrap().revision().clone();
    let spent = {
        let mut hold = dom.begin_parsed_html_style(id, policy, &flag()).unwrap();
        assert!(matches!(
            hold.process_rendering(
                &q,
                crate::DomParsedHtmlStyleNotificationHandoff::Retain,
                &flag()
            ),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 6
                })
            ))
        ));
        hold.spent_work()
    };
    assert_eq!(dom.snapshot().stamp(), before);
    assert!(
        dom.snapshot()
            .node(document.node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking
            .is_empty()
    );
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
    let progress = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(hold.spent_work() > spent);
        hold.process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        )
        .unwrap()
    };
    assert!(matches!(
        progress,
        DomParsedHtmlStyleRenderingProgress::Complete(_)
    ));
    assert_eq!(
        dom.snapshot()
            .node(document.node())
            .unwrap()
            .node
            .base_document()
            .unwrap()
            .render_blocking,
        &[element]
    );
    assert_eq!(cap.snapshot().unwrap().revision(), &revision);
}
#[test]
fn repeated_pending_outputs_consume_one_original_finite_allowance_without_cursor_progress() {
    let (mut dom, _document, id, q, _element, _cap) = ready("screen", 8);
    let policy = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 8).unwrap();
    let before = dom.snapshot().stamp();
    let mut emitted = 0;
    let mut refused = false;
    // The declared total allowance bounds this loop independently of the
    // implementation's earlier prefix-output count. Each retry reacquires the
    // same original entry; dropping a hold cannot reset its output allowance.
    for _ in 0..=policy.outputs() {
        let mut hold = dom.begin_parsed_html_style(id, policy, &flag()).unwrap();
        match hold.process_rendering(
            &q,
            crate::DomParsedHtmlStyleNotificationHandoff::Retain,
            &flag(),
        ) {
            Ok(DomParsedHtmlStyleRenderingProgress::Pending(Required::MediaEnvironment)) => {
                emitted += 1
            }
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 8,
                }),
            )) => {
                refused = true;
                break;
            }
            other => panic!("unexpected original media result: {other:?}"),
        }
    }
    assert!(emitted >= 2 && emitted <= policy.outputs());
    assert!(
        refused,
        "Drop/rebegin must not reset original finite outputs"
    );
    assert_eq!(dom.snapshot().stamp(), before);
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::ReadRenderBlockingPending
    );
}
