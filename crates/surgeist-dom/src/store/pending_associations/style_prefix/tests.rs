// Real original prefix and first creation.
use super::*;
use crate::*;
use std::sync::{Arc, atomic::AtomicBool};
use surgeist_html_parser::{
    HtmlDocumentParseRequest, HtmlParseLimits, HtmlParseReport, HtmlSource,
};
use surgeist_html_semantics::{
    HtmlBaseEncoding, HtmlBaseFactsRevision, HtmlBaseProducerId, HtmlBaseUrlRecord,
    HtmlBaseUrlScheme,
};

pub(in crate::store::pending_associations) fn flag() -> AtomicBool {
    AtomicBool::new(false)
}
pub(in crate::store::pending_associations) fn envelope() -> DomParsedHtmlAssociationLimits {
    DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1_000_000, 1024).unwrap()
}
pub(in crate::store::pending_associations) fn parse(text: &str) -> Arc<HtmlParseReport> {
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
pub(in crate::store::pending_associations) fn setup() -> (Dom, DomParsedDocumentEnvironment) {
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
pub(in crate::store::pending_associations) fn install(
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
pub(in crate::store::pending_associations) fn qualifier(
    dom: &Dom,
    id: DomParsedHtmlAssociationId,
) -> DomParsedHtmlStyleInitialPolicyRequest {
    dom.parsed_html_style_prefix(id)
        .unwrap()
        .initial_policy_request()
        .unwrap()
        .clone()
}
use std::sync::atomic::{AtomicUsize, Ordering};
use surgeist_html::*;
#[derive(Debug)]
struct NativePolicy {
    clauses: [u8; 8],
    atomic: usize,
    drops: Arc<AtomicUsize>,
    descriptors: Arc<AtomicUsize>,
}
impl HtmlCspPolicyArtifact for NativePolicy {
    fn retained_layout(&self) -> HtmlCspArtifactLayout {
        self.descriptors.fetch_add(1, Ordering::Relaxed);
        HtmlCspArtifactLayout::try_new(
            HtmlCspStorage::try_new(size_of::<Self>() + size_of::<AtomicUsize>() * 2, 0, 3)
                .unwrap(),
            HtmlCspDisposalWork::try_new(64, self.atomic).unwrap(),
        )
        .unwrap()
    }
}
impl Drop for NativePolicy {
    fn drop(&mut self) {
        let _ = self.clauses;
        self.drops.fetch_add(1, Ordering::Relaxed);
    }
}
fn native_policy(atomic: usize) -> (HtmlCspAssociation, Arc<AtomicUsize>, Arc<AtomicUsize>) {
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut current = owner.initial_document_with(&mut allow).unwrap();
    let mut backend = HtmlCspProcessorAuthority::new_with(&mut allow).unwrap();
    let profile = backend.profile_with(1, &mut allow).unwrap();
    let request = owner
        .request_meta_insertion_with(
            &current,
            &profile,
            "fixture opaque enforced policy".into(),
            &mut allow,
        )
        .unwrap();
    let drops = Arc::new(AtomicUsize::new(0));
    let descriptors = Arc::new(AtomicUsize::new(0));
    let artifact = Arc::new(NativePolicy {
        clauses: [7; 8],
        atomic,
        drops: drops.clone(),
        descriptors: descriptors.clone(),
    });
    let prepared = backend
        .attest_changed_with(&request, artifact, &mut allow)
        .unwrap();
    let completion = owner
        .prepare_publication_with(&mut current, request, prepared, &mut allow)
        .unwrap()
        .publish();
    drop(completion);
    (current, drops, descriptors)
}
pub(in crate::store::pending_associations) fn begin_await(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    limits: DomParsedHtmlAssociationLimits,
) {
    let mut h = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    assert_eq!(
        h.advance(&flag()).unwrap(),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
}
#[test]
fn native_cleanup_headroom_stays_reserved_during_real_nonce_formation_and_retry() {
    for atomic in [1024, 131072] {
        let (mut dom, env) = setup();
        let nonce = "q".repeat(4096);
        let source = format!("<!doctype html><style nonce='{nonce}'>later text</style>");
        let (_, id) = install(&mut dom, parse(&source), env);
        let (policy, drops, descriptors) = native_policy(atomic);
        let layout = policy.retained_layout().unwrap();
        let allowance = size_of::<DomParsedHtmlStyleRuntimeHold<'_>>()
            + size_of::<Graph>()
            + layout.storage().bytes()
            - size_of::<HtmlCspAssociation>()
            + 131072
            + 4096;
        let limits =
            DomParsedHtmlAssociationLimits::new(allowance, 1_000_000, 1_000_000, 1024).unwrap();
        begin_await(&mut dom, id, limits);
        let q = qualifier(&dom, id);
        let policyid = policy.policy().id().clone();
        let mut input = Some(policy);
        let spent;
        {
            let mut h = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
            h.bind_initial_policy(&q, &mut input, &flag()).unwrap();
            spent = h.spent_work();
        }
        assert!(input.is_none());
        assert_eq!(descriptors.load(Ordering::Relaxed), 1);
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        {
            let mut h = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
            let result = h.advance(&flag());
            if atomic == 1024 {
                assert_eq!(
                    result.unwrap(),
                    DomParsedHtmlStylePhase::InlinePolicyPending
                );
            } else {
                assert!(matches!(
                    result,
                    Err(DomParsedHtmlStyleError::Admission(
                        crate::DomParsedHtmlAssociationAdmissionError::Dom(
                            DomError::ResourceLimit {
                                resource: DomResource::ParsedAssociationProcessingRetainedBytes,
                                ..
                            }
                        )
                    ))
                ));
                assert!(h.spent_work() > spent);
            }
        }
        if atomic != 1024 {
            assert_eq!(ledger.total(), before);
            let view = dom.parsed_html_style_prefix(id).unwrap();
            assert!(view.initial_policy_bound());
            assert_eq!(view.phase(), DomParsedHtmlStylePhase::AwaitingInitialPolicy);
            drop(view);
            let mut h = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            h.advance(&flag()).unwrap();
        }
        let view = dom.parsed_html_style_prefix(id).unwrap();
        let req = view.inline_policy_request().unwrap();
        assert!(req.text().is_empty());
        assert_eq!(req.nonce().unwrap().len(), 4096);
        assert_eq!(req.policy().policy().id(), &policyid);
        drop(view);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        assert_eq!(descriptors.load(Ordering::Relaxed), 1);
        drop(dom);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
    }
}
#[test]
fn persistent_bank_usage_survives_old_retirement_and_later_real_install_remove() {
    let (mut dom, env) = setup();
    let (old_doc, old_id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=A>"),
        env.clone(),
    );
    let old_root = crate::DomStyleCssomRoot::new(
        old_doc.node(),
        surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let caps = [&old_root];
    let (_, style) = install(
        &mut dom,
        parse("<!doctype html><style nonce='persistent'>later text</style>"),
        env.clone(),
    );
    let old_retirement = {
        let mut h = dom
            .begin_parsed_html_meta(old_id, envelope(), &flag())
            .unwrap();
        let p = h.prepare(&caps, &flag()).unwrap();
        p.commit(&flag()).unwrap().into_parts().1
    };
    let before = dom.usage().parsed_association_bytes();
    begin_await(&mut dom, style, envelope());
    assert_eq!(
        dom.usage().parsed_association_bytes(),
        before + size_of::<Graph>()
    );
    let q = qualifier(&dom, style);
    let (policy, drops, _) = native_policy(1024);
    let policy_bytes =
        policy.retained_layout().unwrap().storage().bytes() - size_of::<HtmlCspAssociation>();
    let mut input = Some(policy);
    {
        let mut h = dom
            .begin_parsed_html_style(style, envelope(), &flag())
            .unwrap();
        h.bind_initial_policy(&q, &mut input, &flag()).unwrap();
        h.advance(&flag()).unwrap();
    }
    let native_persistent = dom.usage().parsed_association_bytes();
    assert!(native_persistent >= before + size_of::<Graph>() + policy_bytes);
    drop(old_retirement);
    assert_eq!(dom.usage().parsed_association_bytes(), native_persistent);
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    let (third, id) = install(
        &mut dom,
        parse("<!doctype html><meta http-equiv=default-style content=B>"),
        env,
    );
    let root = crate::DomStyleCssomRoot::new(
        third.node(),
        surgeist_cssom::CssomStore::new(Default::default(), Default::default()).unwrap(),
    );
    let caps = [&root];
    {
        let mut h = dom.begin_parsed_html_meta(id, envelope(), &flag()).unwrap();
        let p = h.prepare(&caps, &flag()).unwrap();
        let (_, dead) = p.commit(&flag()).unwrap().into_parts();
        drop(dead);
    }
    assert_eq!(dom.usage().parsed_association_bytes(), native_persistent);
    assert!(
        dom.parsed_html_style_prefix(style)
            .unwrap()
            .inline_policy_request()
            .is_some()
    );
    let ledger = dom.custody_registry.ledger().clone();
    drop(dom);
    assert_eq!(drops.load(Ordering::Relaxed), 1);
    assert_eq!(ledger.total(), custody::Usage::ZERO);
}
#[test]
fn finite_canonical_pending_quota_refuses_prefix_before_any_new_ledger_backing() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>x{}</style>"), env);
    let bytes = dom.usage().parsed_association_bytes();
    let units = dom.usage().parsed_association_utf16_units();
    dom.set_limits(dom.limits().with_parsed_associations(bytes, units).unwrap())
        .unwrap();
    let ledger = dom.custody_registry.ledger().clone();
    let baseline = ledger.total();
    {
        let mut h = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            h.advance(&flag()),
            Err(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationBytes,
                    ..
                })
            ))
        ));
    }
    assert_eq!(ledger.total(), baseline);
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::Unstarted
    );
    dom.set_limits(
        dom.limits()
            .with_parsed_associations(bytes + 1_000_000, units + 1_000_000)
            .unwrap(),
    )
    .unwrap();
    begin_await(&mut dom, id, envelope());
}
#[test]
fn forgetting_command_keeps_actual_header_debit_and_active_original_control() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>x{}</style>"), env);
    let ledger = dom.custody_registry.ledger().clone();
    let baseline = ledger.total();
    let mut h = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    h.advance(&flag()).unwrap();
    let held = ledger.total();
    assert!(held.bytes() > baseline.bytes());
    std::mem::forget(h);
    assert!(
        matches!(dom.begin_parsed_html_style(id,envelope(),&flag()),Err(DomParsedHtmlStyleError::Admission(crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::ParsedAssociationOperationPending{pending}))) if pending==id)
    );
    assert_eq!(ledger.total(), held);
    drop(dom);
    assert!(ledger.total().bytes() > 0);
}

#[test]
fn original_work_refusal_keeps_unstarted_graph_and_raised_retry_keeps_spent_work() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>x{}</style>"), env);
    let ledger = dom.custody_registry.ledger().clone();
    let before = ledger.total();
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 2, 1024).unwrap();
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.advance(&flag()),
            Err(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 2
                })
            ))
        ));
        assert_eq!(hold.spent_work(), 2);
    }
    assert_eq!(ledger.total(), before);
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::Unstarted
    );
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert_eq!(hold.spent_work(), 3);
    assert_eq!(
        hold.advance(&flag()).unwrap(),
        DomParsedHtmlStylePhase::AwaitingInitialPolicy
    );
}

fn inline_with_policy(dom: &mut Dom, id: DomParsedHtmlAssociationId, policy: HtmlCspAssociation) {
    inline_with_policy_under(dom, id, policy, envelope());
}
fn inline_with_policy_under(
    dom: &mut Dom,
    id: DomParsedHtmlAssociationId,
    policy: HtmlCspAssociation,
    limits: DomParsedHtmlAssociationLimits,
) {
    begin_await(dom, id, limits);
    let q = qualifier(dom, id);
    let mut input = Some(policy);
    let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
    hold.bind_initial_policy(&q, &mut input, &flag()).unwrap();
    hold.advance(&flag()).unwrap();
}
fn decision(
    dom: &Dom,
    id: DomParsedHtmlAssociationId,
    allowed: bool,
) -> DomParsedHtmlStyleInlinePolicyObservation {
    DomParsedHtmlStyleInlinePolicyObservation::evaluated(
        dom.parsed_html_style_prefix(id)
            .unwrap()
            .inline_policy_request()
            .unwrap(),
        allowed,
    )
}
#[test]
fn decision_rejects_each_foreign_bound_policy_stamp_before_accepting_one_output() {
    let (mut dom, env) = setup();
    let (_, id) = install(
        &mut dom,
        parse("<!doctype html><style nonce='n'>later</style>"),
        env,
    );
    let (policy, _, _) = native_policy(1024);
    inline_with_policy(&mut dom, id, policy);
    let (foreign, _, _) = native_policy(1024);
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let empty = owner.initial_document_with(&mut allow).unwrap();
    for field in 0..5 {
        // Start with an actual observation, then corrupt one private stamp
        // component to test validation rather than inventing a policy artifact.
        let mut observation = decision(&dom, id, true);
        match field {
            0 => observation.document = foreign.document().clone(),
            1 => observation.generation = foreign.generation().clone(),
            2 => observation.policy = foreign.policy().id().clone(),
            3 => observation.revision = empty.policy().revision(),
            4 => observation.profile = foreign.policy().profile().cloned(),
            _ => unreachable!(),
        }
        let mut input = Some(observation);
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut input, &flag()),
            Err(DomParsedHtmlStyleError::WrongRequest)
        ));
        assert!(input.is_some());
        assert_eq!(
            hold.advance(&flag()).unwrap(),
            DomParsedHtmlStylePhase::InlinePolicyPending
        );
    }
    let mut input = Some(decision(&dom, id, true));
    assert_eq!(
        dom.begin_parsed_html_style(id, envelope(), &flag())
            .unwrap()
            .accept_inline_policy(&mut input, &flag())
            .unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
}
#[test]
fn selected_decision_retains_exact_graph_debit_and_native_cleanup_until_entry_disposal() {
    for allowed in [false, true] {
        let (mut dom, env) = setup();
        let (_, id) = install(
            &mut dom,
            parse("<!doctype html><style nonce='n'>later</style>"),
            env,
        );
        let (policy, drops, descriptors) = native_policy(131072);
        inline_with_policy(&mut dom, id, policy);
        let ledger = dom.custody_registry.ledger().clone();
        let before = ledger.total();
        let usage = dom.pending_associations.usage;
        let record = dom.pending_associations.controlled_entry(id).unwrap();
        let graph = {
            let view = dom.parsed_html_style_prefix(id).unwrap();
            view.inline_policy_request().unwrap() as *const _ as usize
        };
        let mut input = Some(decision(&dom, id, allowed));
        {
            let mut hold = dom
                .begin_parsed_html_style(id, envelope(), &flag())
                .unwrap();
            hold.accept_inline_policy(&mut input, &flag()).unwrap();
        }
        assert!(input.is_none());
        assert_eq!(ledger.total(), before);
        assert_eq!(dom.pending_associations.usage, usage);
        assert_eq!(drops.load(Ordering::Relaxed), 0);
        assert_eq!(descriptors.load(Ordering::Relaxed), 1);
        {
            let guard = lock(record.value()).unwrap();
            let Graph::Inline(request) = guard.as_ref().unwrap().value().as_ref() else {
                panic!("actual selected request retained")
            };
            assert_eq!(request as *const _ as usize, graph);
            assert_eq!(
                request.action(),
                if allowed {
                    Action::CreateSheet
                } else {
                    Action::Complete
                }
            );
        }
        drop(record);
        drop(dom);
        assert_eq!(drops.load(Ordering::Relaxed), 1);
        assert_eq!(ledger.total(), custody::Usage::ZERO);
    }
}
#[test]
fn decision_work_frontier_refusal_preserves_original_input_and_same_cursor_for_raised_retry() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<!doctype html><style>later</style>"), env);
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let limits = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1024, 1024).unwrap();
    inline_with_policy_under(
        &mut dom,
        id,
        owner.initial_document_with(&mut allow).unwrap(),
        limits,
    );
    let mut input = Some(decision(&dom, id, true));
    let record = dom.pending_associations.controlled_entry(id).unwrap();
    let before = record.value().control.spent_work();
    // The existing original policy is raised only. Exhaust its current work
    // allowance by real steps, leaving exactly the begin admission plus the
    // first acceptance step. The final acceptance step must then refuse.
    {
        let hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        while hold.spent_work() < limits.work() - 2 {
            hold.step(&flag()).unwrap();
        }
    }
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut input, &flag()),
            Err(DomParsedHtmlStyleError::Admission(
                crate::DomParsedHtmlAssociationAdmissionError::Dom(DomError::WorkLimit {
                    resource: DomResource::ParsedAssociationProcessingWork,
                    limit: 1024
                })
            ))
        ));
    }
    assert!(record.value().control.spent_work() > before);
    assert!(input.is_some());
    assert_eq!(
        dom.parsed_html_style_prefix(id).unwrap().phase(),
        DomParsedHtmlStylePhase::InlinePolicyPending
    );
    let raised = DomParsedHtmlAssociationLimits::new(16_000_000, 1_000_000, 1034, 1024).unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert_eq!(
        hold.accept_inline_policy(&mut input, &flag()).unwrap(),
        DomParsedHtmlStylePhase::CreateSheetPending
    );
}
#[test]
fn blocked_successor_final_work_refusal_preserves_policy_and_real_null_removal_retry() {
    let (mut dom, env) = setup();
    let (document, id) = install(
        &mut dom,
        parse("<!doctype html><style nonce=n>later</style>"),
        env,
    );
    let (policy, drops, descriptors) = native_policy(131072);
    inline_with_policy(&mut dom, id, policy);
    let mut supplied = Some(decision(&dom, id, false));
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut supplied, &flag())
        .unwrap();
    let old_q = qualifier(&dom, id);
    let state = dom.state.clone();
    let stamp = dom.snapshot().stamp();
    let record = dom.pending_associations.controlled_entry(id).unwrap();
    let spent = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let prep = hold.prepare_successor(None, &flag()).unwrap();
        let control = &record.value().control;
        while control.limits().work() - control.spent_work() > 1 {
            prep.step(&flag()).unwrap();
        }
        assert!(matches!(
            prep.commit(&flag()),
            Err(DomParsedHtmlStyleSuccessorError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        ..
                    }
                ))
            ))
        ));
        hold.spent_work()
    };
    assert!(Arc::ptr_eq(&state, &dom.state));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(qualifier(&dom, id), old_q);
    assert!(dom.style_cssom_roots.get(document.node()).is_none());
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    assert_eq!(descriptors.load(Ordering::Relaxed), 1);
    let raised = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 100_000,
        envelope().outputs(),
    )
    .unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() >= spent);
    let (completion, retired) = hold
        .prepare_successor(None, &flag())
        .unwrap()
        .commit(&flag())
        .unwrap()
        .into_parts();
    assert!(matches!(
        completion.removal(),
        DomParsedHtmlStyleSuccessorRemoval::NoAssociatedSheet
    ));
    assert_ne!(completion.qualification(), &old_q);
    drop(retired);
    drop(hold);
    assert!(Arc::ptr_eq(&state, &dom.state));
    assert_eq!(dom.snapshot().stamp(), stamp);
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    assert_eq!(
        descriptors.load(Ordering::Relaxed),
        1,
        "policy was moved, not rebound/re-described"
    );
}

#[test]
fn failed_decision_output_admission_stays_unstarted_and_raise_admits_the_same_input_once() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<style>x</style>"), env);
    let limits = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work(),
        1,
    )
    .unwrap();
    begin_await(&mut dom, id, limits);
    let q = qualifier(&dom, id);
    let mut allow = |_: HtmlCspLayout| Ok::<_, ()>(());
    let mut owner = HtmlCspOwner::new_with(&mut allow).unwrap();
    let mut policy = Some(owner.initial_document_with(&mut allow).unwrap());
    {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        hold.bind_initial_policy(&q, &mut policy, &flag()).unwrap();
        hold.advance(&flag()).unwrap();
    }
    let mut observation = Some(decision(&dom, id, true));
    let spent = {
        let mut hold = dom.begin_parsed_html_style(id, limits, &flag()).unwrap();
        assert!(matches!(
            hold.accept_inline_policy(&mut observation, &flag()),
            Err(DomParsedHtmlStyleError::Admission(
                DomParsedHtmlAssociationAdmissionError::Dom(DomError::ResourceLimit {
                    resource: DomResource::ParsedAssociationProcessingOutputs,
                    limit: 1
                })
            ))
        ));
        assert_eq!(
            hold.meter.entry().control.execution(),
            DomParsedHtmlExecutionState::Unstarted
        );
        hold.spent_work()
    };
    assert!(observation.is_some());
    let mut hold = dom
        .begin_parsed_html_style(id, envelope(), &flag())
        .unwrap();
    assert!(hold.spent_work() >= spent);
    hold.accept_inline_policy(&mut observation, &flag())
        .unwrap();
    assert!(observation.is_none());
    assert_eq!(
        hold.meter.entry().control.execution(),
        DomParsedHtmlExecutionState::Activated
    );
}

#[test]
fn real_blocked_pair_end_work_refusal_keeps_controller_and_raise_does_not_rebind_policy() {
    let (mut dom, env) = setup();
    let (_, id) = install(&mut dom, parse("<style>blocked</style>"), env);
    let (policy, drops, descriptors) = native_policy(0);
    inline_with_policy(&mut dom, id, policy);
    let mut input = Some(decision(&dom, id, false));
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut input, &flag())
        .unwrap();
    {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        let (done, retired) = hold
            .prepare_successor(None, &flag())
            .unwrap()
            .commit(&flag())
            .unwrap()
            .into_parts();
        drop(retired);
        drop(done);
    }
    let mut input = Some(decision(&dom, id, false));
    dom.begin_parsed_html_style(id, envelope(), &flag())
        .unwrap()
        .accept_inline_policy(&mut input, &flag())
        .unwrap();
    let state = dom.state.clone();
    let spent = {
        let mut hold = dom
            .begin_parsed_html_style(id, envelope(), &flag())
            .unwrap();
        while hold.limits().work() - hold.spent_work() > 3 {
            hold.step(&flag()).unwrap();
        }
        assert!(matches!(
            hold.complete_synchronous_execution(&flag()),
            Err(DomParsedHtmlFrontierError::Style(
                DomParsedHtmlStyleError::Admission(DomParsedHtmlAssociationAdmissionError::Dom(
                    DomError::WorkLimit {
                        resource: DomResource::ParsedAssociationProcessingWork,
                        ..
                    }
                ))
            ))
        ));
        assert_eq!(
            hold.meter.entry().control.execution(),
            DomParsedHtmlExecutionState::Activated
        );
        hold.spent_work()
    };
    assert!(Arc::ptr_eq(&state, &dom.state));
    assert_eq!(drops.load(Ordering::Relaxed), 0);
    assert_eq!(descriptors.load(Ordering::Relaxed), 1);
    let raised = DomParsedHtmlAssociationLimits::new(
        envelope().retained_bytes(),
        envelope().retained_units(),
        envelope().work() + 100_000,
        envelope().outputs(),
    )
    .unwrap();
    let mut hold = dom.begin_parsed_html_style(id, raised, &flag()).unwrap();
    assert!(hold.spent_work() >= spent);
    let end = hold.complete_synchronous_execution(&flag()).unwrap();
    assert_eq!(end.contribution(), id);
    drop(hold);
    drop(end);
    assert!(Arc::ptr_eq(&state, &dom.state));
    assert_eq!(descriptors.load(Ordering::Relaxed), 1);
}
