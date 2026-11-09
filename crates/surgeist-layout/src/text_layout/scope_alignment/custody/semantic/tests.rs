//! Resource observations come from real Text transactions and exact root unions.
use super::*;
use crate::text_layout::{InlineBoxFragmentationDemand, TextParagraphBinding};
use surgeist_text as text;

const LIMITS: ScopeAlignmentLimits = ScopeAlignmentLimits::new(1_000_000, 100, 1_000_000, 16 << 20);

fn captured(raw: &str) -> PreparedText {
    use text::SourceProcessingEvent as E;
    let source = text::Source::new(raw);
    let end = raw.encode_utf16().count();
    let style = crate::test_support::text::ahem_style(10.0, 10.0);
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: text::Range::new(0, end),
                aggregate: text::Range::new(0, end),
                kind: text::ParagraphOriginKind::Authored,
            }],
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: end,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            connections: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            joint_flows: vec![],
            scope_alignment_limits: Some(LIMITS),
            scopes: vec![text::SourceProcessingScope::Block {
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: text::ParagraphBaseDirection::LeftToRight,
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
            }],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Control {
                    control: 0,
                    affinity: text::Affinity::After,
                },
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::Control {
                    control: 1,
                    affinity: text::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analyzed = crate::test_support::text::in_flat_wrapping_context(analyzed, text::Wrap::Word);
    let mut system = text::System::default();
    system
        .register_fonts(
            include_bytes!("../../../../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf")
                .to_vec(),
            surgeist_test::fixtures::FONT_LIMITS,
        )
        .unwrap();
    system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap()
}

fn owner(prepared: &PreparedText) -> ScopeAlignmentResources<f64> {
    let binding = TextParagraphBinding {
        prepared: prepared.clone(),
        inline_box_fragments: InlineBoxFragmentationDemand::NotRequested,
        container: 0_u32,
        nodes: vec![],
        origins: vec![],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    let snapshot = ScopeAlignmentSnapshot::admit(vec![], &[binding], false)
        .unwrap()
        .freeze();
    ScopeAlignmentResources::new(
        snapshot,
        crate::text_layout::fragmentation::InlineBoxFamilies::default(),
    )
}

fn template(prepared: &PreparedText) -> SemanticWindowRequest {
    SemanticWindowRequest::new(prepared.source_segments().next().unwrap())
}

fn funded(
    resources: &ScopeAlignmentResources<f64>,
    prepared: &PreparedText,
) -> FundedSemanticRequest {
    let controls = prepared.control_views().count();
    let mut request = resources
        .fund_semantic_request(template(prepared), 0, controls, 0)
        .unwrap();
    for view in prepared.control_views() {
        request
            .push_control(SemanticControlAllocation {
                occurrence: view.handle(),
                boundary: Some(text::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
            })
            .unwrap();
    }
    request
}

fn input(prepared: &PreparedText) -> ScopeAlignmentCensusInput<'_> {
    ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(prepared))
}

fn external_roots<const N: usize>(
    prepared: &PreparedText,
    roots: [&SemanticWindowEvidence; N],
) -> text::ScopeAlignmentRetention {
    let products = roots.map(ScopeAlignmentProduct::SemanticWindow);
    text::scope_alignment_census(
        input(prepared),
        ScopeAlignmentProducts::Mixed(&products),
        text::ScopeAlignmentAllowance::new(LIMITS.records(), LIMITS.payload_bytes()),
    )
    .unwrap()
    .external_retention()
}

#[test]
fn semantic_roots_are_strong_unique_per_observation_and_deduplicated_across_case_clones() {
    let p = captured("X");
    let q = captured("Y");
    let base = owner(&p);
    let initial_live = base.census();
    let resources = base.entry_view(&p).unwrap();
    let trial = resources.clone();
    let a = resources.observe_semantic(funded(&resources, &p)).unwrap();
    let a_clone = a.clone();
    let one = external_roots(&p, [a.evidence()]);
    assert_eq!(one, a.evidence().scope_alignment_retention());
    assert_eq!(one, external_roots(&p, [a.evidence(), a_clone.evidence()]));
    let b = trial.observe_semantic(funded(&trial, &q)).unwrap();
    let c = resources.observe_semantic(funded(&resources, &p)).unwrap();
    let fresh = external_roots(&p, [a.evidence(), c.evidence()]);
    assert_eq!(
        fresh.records(),
        one.records() + c.evidence().scope_alignment_retention().records()
    );
    assert_eq!(
        fresh.payload_bytes(),
        one.payload_bytes() + c.evidence().scope_alignment_retention().payload_bytes()
    );
    for proof in [&a, &b, &c] {
        assert!(proof.evidence().partitions()[0].has_in_flow_content());
        assert_eq!(
            proof
                .evidence()
                .scope_alignment_usage()
                .unwrap()
                .font_queries(),
            0
        );
    }
    let expected = text::scope_alignment_census(
        input(&q),
        ScopeAlignmentProducts::Mixed(&[
            ScopeAlignmentProduct::Prepared(&p),
            ScopeAlignmentProduct::SemanticWindow(a.evidence()),
            ScopeAlignmentProduct::SemanticWindow(a_clone.evidence()),
            ScopeAlignmentProduct::SemanticWindow(b.evidence()),
            ScopeAlignmentProduct::SemanticWindow(c.evidence()),
        ]),
        text::ScopeAlignmentAllowance::new(LIMITS.records(), LIMITS.payload_bytes()),
    )
    .unwrap();
    let operation = trial.begin(input(&q), 2).unwrap().unwrap();
    assert_eq!(
        operation.input_union().unwrap(),
        ScopeCensus::from_retention(expected.input_retention())
            .add(ScopeCensus::from_retention(expected.external_retention()))
            .unwrap()
    );
    assert_eq!(
        trial.semantic_credit(),
        [&a, &b, &c]
            .iter()
            .map(|p| p.evidence().scope_alignment_retention().records())
            .sum::<usize>()
    );
    drop(operation);
    let prior_work = base.active_owner().unwrap().state.lock().work;
    let prior_materialized = base.active_owner().unwrap().state.lock().materialized;
    drop((a, a_clone, b, c, trial, resources));
    // Persistent finalized guard slots are an existing owner allocation. They
    // outlive this operation, while all opt-in roots/wrappers have been released.
    assert!(base.census().bytes >= initial_live.bytes);
    assert_eq!(base.active_owner().unwrap().state.lock().work, prior_work);
    assert_eq!(
        base.active_owner().unwrap().state.lock().materialized,
        prior_materialized
    );
}

#[test]
fn request_and_root_live_leases_release_without_refunding_completed_costs() {
    let p = captured("X");
    let base = owner(&p);
    let before_view = base.census();
    let resources = base.entry_view(&p).unwrap();
    let request = funded(&resources, &p);
    assert!(
        request
            .leases
            .iter()
            .flatten()
            .map(|l| l.census.bytes)
            .sum::<usize>()
            > 0
    );
    let materialized = base.active_owner().unwrap().state.lock().materialized;
    let live = base.census();
    let request_bytes = request
        .leases
        .iter()
        .flatten()
        .map(|l| l.census.bytes)
        .sum::<usize>();
    drop(request);
    assert_eq!(base.census().bytes + request_bytes, live.bytes);
    assert_eq!(
        base.active_owner().unwrap().state.lock().materialized,
        materialized
    );
    let proof = resources.observe_semantic(funded(&resources, &p)).unwrap();
    assert!(base.active_owner().unwrap().state.lock().materialized > materialized);
    drop(resources);
    assert!(
        base.census().bytes > before_view.bytes,
        "the retained proof pins the real store and its live Layout leases"
    );
    drop(proof);
    assert_eq!(base.census(), before_view);
    assert!(base.active_owner().unwrap().state.lock().materialized > materialized);
}

#[test]
fn funded_failure_is_debited_before_its_original_error_even_after_late_exhaustion() {
    let p = captured("X");
    let resources = owner(&p).entry_view(&p).unwrap();
    // Missing actual controls is a real producer validation error after funding.
    let request = template(&p);
    let operation = resources
        .begin_with_families(
            ScopeAlignmentCensusInput::SemanticWindow {
                prepared: &p,
                request: &request,
            },
            0,
            &[],
            0,
        )
        .unwrap()
        .unwrap();
    let failure = p
        .semantic_window_with_scope_allowance(&request, operation.allowance().unwrap())
        .unwrap_err();
    let code = failure.error().code;
    let detail = failure.error().detail().cloned();
    let usage = failure.scope_alignment_usage().unwrap();
    let admitted = failure.input_retention();
    assert!(usage.structural_work() > 0);
    assert!(usage.peak_payload_bytes() > usage.payload_bytes());
    assert_eq!(usage.font_queries(), 0);
    let materialized = resources.active_owner().unwrap().state.lock().materialized;
    let remaining = LIMITS.structural_work() - resources.active_owner().unwrap().state.lock().work;
    resources.visit(LIMITS, remaining).unwrap();
    let error = resources
        .admit_semantic(Some(&operation), 0, Err(failure))
        .unwrap_err();
    assert_eq!(error.code, code);
    assert_eq!(error.detail(), detail.as_ref());
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized - materialized,
        usage.records() - admitted.records()
    );
    let later = resources.visit(LIMITS, 0).unwrap_err();
    assert!(
        matches!(later.detail(), Some(text::ErrorDetail::ScopeAlignment { reason: ScopeAlignmentError::Limit { kind: ScopeAlignmentLimitKind::StructuralWork, required, limit } }) if *required == LIMITS.structural_work() + usage.structural_work() && *limit == LIMITS.structural_work())
    );
}

#[test]
fn partial_record_seed_is_borrowed_input_and_never_new_materialization() {
    let p = captured("X");
    for seed_records in [false, true] {
        let resources = owner(&p).entry_view(&p).unwrap();
        let request = template(&p);
        let operation = resources
            .begin_with_families(
                ScopeAlignmentCensusInput::SemanticWindow {
                    prepared: &p,
                    request: &request,
                },
                0,
                &[],
                0,
            )
            .unwrap()
            .unwrap();
        let allowance = text::ScopeAlignmentAllowance::new(
            if seed_records {
                operation.input.records
            } else {
                0
            },
            0,
        );
        let failure = p
            .semantic_window_with_scope_allowance(&request, allowance)
            .unwrap_err();
        let usage = failure.scope_alignment_usage().unwrap();
        assert_eq!(
            failure.input_retention(),
            text::ScopeAlignmentRetention::default()
        );
        assert_eq!(
            usage.records(),
            if seed_records {
                operation.input.records
            } else {
                0
            }
        );
        let prior = resources.active_owner().unwrap().state.lock().materialized;
        resources
            .admit_semantic(Some(&operation), 0, Err(failure))
            .unwrap_err();
        assert_eq!(
            resources.active_owner().unwrap().state.lock().materialized,
            prior
        );
    }
}

#[test]
fn request_capacity_refusal_preserves_funded_roots_and_later_valid_admission() {
    let p = captured("X");
    let resources = owner(&p).entry_view(&p).unwrap();
    let before = resources.active_owner().unwrap().state.lock().work;
    let error = match resources.fund_semantic_request(template(&p), 0, usize::MAX, 0) {
        Ok(_) => panic!("unrepresentable request rows must refuse before allocation"),
        Err(error) => error,
    };
    assert!(matches!(
        error.detail(),
        Some(text::ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::Overflow {
                kind: ScopeAlignmentLimitKind::Records
            }
        })
    ));
    assert!(resources.active_owner().unwrap().state.lock().work > before);
    let mut request = funded(&resources, &p);
    let extra = request.request.controls[0].clone();
    request.push_control(extra).unwrap_err();
    assert_eq!(request.request.controls.len(), p.control_views().count());
    resources.observe_semantic(request).unwrap();
}

#[test]
fn pure_native_semantic_observation_never_activates_scope_funding() {
    let p = crate::test_support::text::PreparedParagraph::plain(
        10,
        text::Source::new("X"),
        crate::test_support::text::ahem_style(10.0, 10.0),
    )
    .prepared;
    let base = owner(&p);
    assert_eq!(base.census(), ScopeCensus::default());
    let resources = base.entry_view(&p).unwrap();
    let a = resources.observe_semantic(funded(&resources, &p)).unwrap();
    let b = resources.observe_semantic(funded(&resources, &p)).unwrap();
    assert!(a.evidence().scope_alignment_usage().is_none());
    assert!(resources.begin(input(&p), 3).unwrap().is_none());
    assert_eq!(resources.census(), ScopeCensus::default());

    // A real captured sibling funds the shared Layout store only. The Native
    // Text transaction still has no usage; its actual store insertion costs
    // exactly one explicit Layout work unit after request construction.
    let anchor = captured("Y");
    // This independent captured census observes custody identity. It does not
    // activate either Native producer or the Native Layout resource owner.
    let one = external_roots(&anchor, [a.evidence()]);
    assert_eq!(
        one,
        external_roots(&anchor, [a.evidence(), a.clone().evidence()])
    );
    let fresh = external_roots(&anchor, [a.evidence(), b.evidence()]);
    assert_eq!(
        fresh.records(),
        one.records() + b.evidence().scope_alignment_retention().records()
    );
    assert_eq!(
        fresh.payload_bytes(),
        one.payload_bytes() + b.evidence().scope_alignment_retention().payload_bytes()
    );
    assert_eq!(resources.census(), ScopeCensus::default());
    let mixed = owner(&anchor).entry_view(&anchor).unwrap();
    let request = funded(&mixed, &p);
    let before = mixed.active_owner().unwrap().state.lock().work;
    let proof = mixed.observe_semantic(request).unwrap();
    assert!(proof.evidence().scope_alignment_usage().is_none());
    assert_eq!(mixed.active_owner().unwrap().state.lock().work, before + 1);
}
