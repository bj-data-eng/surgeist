//! Real original Style prefix; no policy evaluation or sheet completion.
#[path = "parsed_style/decision.rs"]
mod decision;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_dom::*;
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
fn envelope() -> DomParsedHtmlAssociationLimits {
    DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap()
}
fn parse(text: &str) -> Arc<HtmlParseReport> {
    let limits = HtmlParseLimits::builder()
        .source_units(100_000)
        .buffered_utf16_units(100_000)
        .token_attributes(10_000)
        .auxiliary_entries(1_000_000)
        .managed_nodes(100_000)
        .managed_edges(100_000)
        .managed_attributes(100_000)
        .managed_utf16_units(100_000)
        .source_map_entries(100_000)
        .source_map_payload_units(100_000)
        .diagnostics(100_000)
        .diagnostic_payload_units(100_000)
        .context_effects(100_000)
        .open_elements(100)
        .tree_depth(100_000)
        .managed_capacity_bytes(100_000_000)
        .peak_transient_bytes(100_000_000)
        .atomic_layout_bytes(100_000_000)
        .work_credits(100_000_000)
        .finish()
        .unwrap();
    Arc::new(
        HtmlDocumentParseRequest::offline(HtmlSource::utf8(text), limits)
            .parse()
            .unwrap(),
    )
}
fn setup() -> (Dom, DomParsedDocumentEnvironment) {
    let policy = DomLimits::new(
        1024,
        1_000_000,
        1024,
        4096,
        4096,
        32,
        4_000_000,
        DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
    )
    .unwrap()
    .with_meta_processing_work(1_000_000)
    .with_meta_theme_color_scratch_bytes(262144)
    .unwrap();
    let mut dom = Dom::new(policy).unwrap();
    let mut batch = DomBatch::new();
    let document = batch.create_document();
    let commit = dom.apply(batch, &flag()).unwrap();
    let origin = dom
        .snapshot()
        .document(commit.document(&document).unwrap())
        .unwrap()
        .metadata()
        .origin()
        .clone();
    let url = HtmlBaseUrlRecord::observed(
        HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3),
        "https://example.test/parsed/".into(),
        HtmlBaseUrlScheme::Other,
        false,
        None,
    )
    .unwrap();
    let environment = DomParsedDocumentEnvironment::headless(
        origin,
        url,
        HtmlBaseEncoding::fresh(),
        "UTF-8".into(),
        DomNativeInputPlatform::headless(),
    );
    (dom, environment)
}
fn install(
    dom: &mut Dom,
    report: Arc<HtmlParseReport>,
    environment: DomParsedDocumentEnvironment,
) -> (DocumentId, DomParsedHtmlAssociationId) {
    let installed = dom
        .install_parsed_html_document(
            dom.snapshot().stamp(),
            report,
            environment,
            DomParsedHtmlInstallLimits::new(1024, 10_000_000, 16_000_000, 100_000_000),
            &flag(),
        )
        .unwrap();
    (
        installed.document(),
        installed.pending_association().unwrap(),
    )
}
fn empty_policy() -> surgeist_html::HtmlCspAssociation {
    let mut admit = |_: surgeist_html::HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = surgeist_html::HtmlCspOwner::new_with(&mut admit).unwrap();
    owner.initial_document_with(&mut admit).unwrap()
}
fn qualifier(dom: &Dom, id: DomParsedHtmlAssociationId) -> DomParsedHtmlStyleInitialPolicyRequest {
    dom.parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone()
}
#[test]
fn original_prefix_keeps_spent_admission_work_then_binds_exact_policy_and_request_once() {
    let (mut dom, env) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><style nonce='raw&#x20;nonce'>a { color: red }</style>"),
        env,
    );
    let stamp = dom.snapshot().stamp();
    {
        let mut meta = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
        assert!(meta.validate_meta(&flag()).is_err());
    }
    let spent;
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        spent = hold.spent_work();
        assert!(spent > 0);
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::AwaitingInitialPolicy
        );
    }
    let q = qualifier(&dom, id);
    assert_eq!(q.document(), document);
    assert_eq!(q.contribution(), id);
    let policy = empty_policy();
    let expected = policy.policy().id().clone();
    let mut input = Some(policy);
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(hold.spent_work() > spent);
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::AwaitingInitialPolicy
        );
        hold.bind_initial_policy(&q, &mut input, &flag()).unwrap();
    }
    assert!(input.is_none());
    assert!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .initial_policy_bound()
    );
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
    }
    let address;
    {
        let view = dom.parsed_html_style_prefix(id).unwrap();
        let req = view.inline_policy_request().unwrap();
        address = req as *const _ as usize;
        assert_eq!(req.qualification(), &q);
        // Insertion is the first influencing original update. Its Text is
        // empty even though the completed parser tree later contains CSS.
        assert!(req.text().is_empty());
        assert_eq!(
            req.nonce().unwrap().code_units(),
            &"raw nonce".encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            req.action(),
            surgeist_html::HtmlStyleUpdateAction::CheckInlineCsp
        );
        assert_eq!(req.policy().policy().id(), &expected);
    }
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
    }
    assert_eq!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .unwrap() as *const _ as usize,
        address
    );
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(
        dom.pending_parsed_html_association(document).unwrap(),
        Some(id)
    );
    assert!(dom.style_cssom_root(document.node()).unwrap().is_none());
}
#[test]
fn independent_destinations_bind_distinct_explicit_policies_and_reject_foreign_request() {
    let (mut dom, env) = setup();
    let report = parse("<!doctype html><style>x{}</style>");
    let (_, a) = install(&mut dom, report.clone(), env.clone());
    let (_, b) = install(&mut dom, report, env);
    for id in [a, b] {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        h.advance(&flag()).unwrap();
    }
    let qa = qualifier(&dom, a);
    let qb = qualifier(&dom, b);
    assert_eq!(qa.occurrence(), qb.occurrence());
    assert_ne!(qa.contribution(), qb.contribution());
    let mut input = Some(empty_policy());
    let original = input.as_ref().unwrap().policy().id().clone();
    {
        let mut h = dom.begin_parsed_html_style(a, envelope(), &flag()).unwrap();
        assert!(matches!(
            h.bind_initial_policy(&qb, &mut input, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert_eq!(input.as_ref().unwrap().policy().id(), &original);
        h.bind_initial_policy(&qa, &mut input, &flag()).unwrap();
    }
    let mut other = Some(empty_policy());
    let otherid = other.as_ref().unwrap().policy().id().clone();
    {
        let mut h = dom.begin_parsed_html_style(b, envelope(), &flag()).unwrap();
        h.bind_initial_policy(&qb, &mut other, &flag()).unwrap();
        h.advance(&flag()).unwrap();
    }
    {
        let mut h = dom.begin_parsed_html_style(a, envelope(), &flag()).unwrap();
        h.advance(&flag()).unwrap();
        let mut replacement = Some(empty_policy());
        assert!(matches!(
            h.bind_initial_policy(&qa, &mut replacement, &flag()),
            Err(DomParsedHtmlStyleError::AlreadyBound)
        ));
        assert!(replacement.is_some());
    }
    assert_eq!(
        dom.parsed_html_style_prefix(a)
            .unwrap()
            .inline_policy_request()
            .unwrap()
            .policy()
            .policy()
            .id(),
        &original
    );
    assert_eq!(
        dom.parsed_html_style_prefix(b)
            .unwrap()
            .inline_policy_request()
            .unwrap()
            .policy()
            .policy()
            .id(),
        &otherid
    );
}
#[test]
fn accepted_binding_survives_request_output_refusal_and_raised_retry() {
    let (mut dom, env) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style>real text</style>"),
        env,
    );
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 0).unwrap();
    {
        let mut h = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        h.advance(&flag()).unwrap();
    }
    let q = qualifier(&dom, id);
    let mut input = Some(empty_policy());
    let spent;
    {
        let mut h = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        h.bind_initial_policy(&q, &mut input, &flag()).unwrap();
        assert!(h.advance(&flag()).is_err());
        spent = h.spent_work();
    }
    let view = dom.parsed_html_style_prefix(id).unwrap();
    assert_eq!(view.phase(), DomParsedHtmlStylePhase::AwaitingInitialPolicy);
    assert!(view.initial_policy_bound());
    drop(view);
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(h.spent_work() > spent);
        h.advance(&flag()).unwrap();
    }
    assert!(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .is_some()
    );
}
#[test]
fn earlier_influencing_meta_never_gains_fresh_style_authority() {
    let (mut dom, env) = setup();
    let (doc, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=A><style>x{}</style>"),
        env,
    );
    let mut h = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(
        matches!(h.advance(&flag()),Err(DomParsedHtmlStyleError::UnsupportedProfile{contribution}) if contribution==id)
    );
    drop(h);
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::Unstarted
    );
    assert_eq!(dom.pending_parsed_html_association(doc).unwrap(), Some(id));
}
#[test]
fn forgotten_canonical_view_stays_busy_instead_of_reopening_the_original_prefix() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>x{}</style>"), env);
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        h.advance(&flag()).unwrap();
    }
    std::mem::forget(dom.parsed_html_style_prefix(id).unwrap());
    assert!(
        matches!(dom.parsed_html_style_prefix(id),Err(DomParsedHtmlStyleError::Busy{contribution}) if contribution==id)
    );
    assert!(
        matches!(dom.begin_parsed_html_style(id,envelope(),&flag()),Err(DomParsedHtmlStyleError::Busy{contribution}) if contribution==id)
    );
}
