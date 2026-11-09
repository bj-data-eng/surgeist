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

#[test]
fn entry_funding_requires_the_live_captured_owner_and_keeps_native_inactive() {
    use crate::text_layout::scope_alignment::EntryFunding;
    let native = crate::test_support::text::PreparedParagraph::plain(
        10,
        text::Source::new("X"),
        crate::test_support::text::ahem_style(10.0, 10.0),
    )
    .prepared;
    let inactive = owner(&native);
    assert!(
        EntryFunding::new(inactive.clone(), native)
            .unwrap()
            .is_none()
    );
    let captured = captured("Y");
    let error = match EntryFunding::new(inactive, captured.clone()) {
        Ok(_) => panic!("captured entry work requires actual live resource custody"),
        Err(error) => error,
    };
    assert!(matches!(
        error.detail(),
        Some(text::ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::MissingCapture,
        })
    ));
    assert!(
        EntryFunding::new(owner(&captured), captured)
            .unwrap()
            .is_some()
    );
}

#[test]
fn entry_backing_adopts_funded_rows_and_rejects_unrepresented_storage() {
    use crate::text_layout::scope_alignment::{EntryBacking, EntryFunding};
    let prepared = captured("X");
    let resources = owner(&prepared);
    let funding = EntryFunding::new(resources.clone(), prepared.clone())
        .unwrap()
        .unwrap();
    let (lease, mut values) = funding
        .buffer::<u32>(4)
        .map(|(values, lease)| (lease, values))
        .unwrap();
    funding.visit(2).unwrap();
    values.extend([1, 2]);
    let before = resources.active_owner().unwrap().state.lock().materialized;
    let work = resources.active_owner().unwrap().state.lock().work;
    let live = resources.census();
    let (mut values, mut backing) = EntryBacking::from_buffer(values, 4, lease, &funding).unwrap();
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized,
        before
    );
    assert_eq!(resources.active_owner().unwrap().state.lock().work, work);
    assert_eq!(resources.census(), live);
    values.pop();
    backing.reserve_push(&mut values, Some(&funding)).unwrap();
    values.push(9);
    assert!(resources.active_owner().unwrap().state.lock().materialized > before);
    assert_eq!(resources.census(), live);
    let before_spare = resources.active_owner().unwrap().state.lock().materialized;
    backing.reserve_push(&mut values, Some(&funding)).unwrap();
    values.push(3);
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized,
        before_spare
    );
    assert_eq!(values, [1, 9, 3]);
    drop((values, backing));

    for invalid_case in 0..3 {
        let baseline = resources.census();
        let (lease, mut values) = funding
            .buffer::<u32>(4)
            .map(|(values, lease)| (lease, values))
            .unwrap();
        funding.visit(2).unwrap();
        values.extend([1, 2]);
        let admitted = match invalid_case {
            0 => 1,
            1 => values.capacity() + 1,
            _ => 3,
        };
        let records = resources.active_owner().unwrap().state.lock().materialized;
        let work = resources.active_owner().unwrap().state.lock().work;
        let error = match EntryBacking::from_buffer(values, admitted, lease, &funding) {
            Ok(_) => panic!("unrepresented buffer rows cannot acquire entry custody"),
            Err(error) => error,
        };
        assert!(
            matches!(error.detail(), Some(text::ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::UnrepresentedOwnership { scope },
        }) if *scope == prepared.captured_owner().unwrap().domain().capture_ordinal())
        );
        assert_eq!(resources.census(), baseline);
        assert_eq!(
            resources.active_owner().unwrap().state.lock().materialized,
            records
        );
        assert_eq!(resources.active_owner().unwrap().state.lock().work, work);
    }
}

fn formatted_input() -> crate::ComputeInputOf<f64> {
    crate::ComputeInputOf::leaf_layout(
        crate::Size::NONE,
        crate::Size::NONE,
        crate::ContainingLayoutContext::new(
            crate::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            crate::ParentFormattingContext::BlockFlow,
        ),
        crate::SizingRequestOf::new(
            crate::Size::new(
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Definite(
                        crate::NonNegativeFiniteOf::new(40.0).unwrap(),
                    ),
                    crate::SizingOperation::Normal,
                ),
                crate::AxisSizingRequestOf::new(
                    crate::AvailableSpaceOf::Indefinite,
                    crate::SizingOperation::Normal,
                ),
            ),
            crate::OrthogonalFallbackOf::new(
                crate::Size::NONE,
                crate::NearestScrollportOf::Unknown,
                crate::Size::splat(crate::OrthogonalFallbackLimitOf::Unknown),
            ),
        ),
    )
    .unwrap()
}

#[test]
fn funded_formatted_fork_growth_replacement_and_shared_basis_keep_custody() {
    use crate::text_layout::formatted::{
        BoundAssignment, ComputeScope, FormattedFlow, FormattedState, FormattedTailAssignment,
        FormattedTailEndpoint, FormattedTailUses, TailPredicate,
    };
    use crate::text_layout::scope_alignment::EntryFunding;
    let prepared = captured("X");
    let resources = owner(&prepared);
    let funding = EntryFunding::new(resources.clone(), prepared.clone())
        .unwrap()
        .unwrap();
    let baseline = resources.census();
    let work_before = resources.active_owner().unwrap().state.lock().work;
    let records_before = resources.active_owner().unwrap().state.lock().materialized;
    let input = formatted_input();
    let scope = ComputeScope {
        node: 0_u32,
        input,
        anonymous: false,
        frame: None,
    };
    let segment = prepared.source_segments().next().unwrap();
    let mode = crate::AxisSizingRequestOf::new(
        crate::AvailableSpaceOf::Definite(crate::NonNegativeFiniteOf::new(40.0).unwrap()),
        crate::SizingOperation::Normal,
    );
    let mut original = FormattedState::new_entry(None, Arc::from([]), None).unwrap();
    original.push_scope(scope.clone()).unwrap();
    let binding = original.binding(0, &[1, 2], &segment, input, mode).unwrap();
    let endpoint = FormattedTailEndpoint::new(segment.end(), None);
    original.assignments.push(BoundAssignment {
        binding,
        assignment: FormattedTailAssignment {
            endpoint,
            predicate: TailPredicate::AcceptedEndpoint,
            last: true,
        },
    });
    original.demanded.push(0);
    let mut fork = original.fork_entry(Some(&funding), true).unwrap();
    assert!(resources.census().bytes > baseline.bytes);
    for node in 1..10 {
        fork.push_scope(ComputeScope {
            node,
            ..scope.clone()
        })
        .unwrap();
    }
    let item = fork
        .merge_run(
            0,
            &[1, 2],
            &segment,
            input,
            mode,
            FormattedTailUses::default(),
        )
        .unwrap();
    let mut flow = FormattedFlow::new(None, Some(&funding));
    flow.push(item).unwrap();
    fork.record(flow).unwrap();
    let live_after_record = resources.census();
    let before_replacement = resources.active_owner().unwrap().state.lock().materialized;
    let item = fork
        .merge_run(
            0,
            &[1, 2],
            &segment,
            input,
            mode,
            FormattedTailUses::default(),
        )
        .unwrap();
    let mut flow = FormattedFlow::new(None, Some(&funding));
    flow.push(item).unwrap();
    fork.record(flow).unwrap();
    assert_eq!(resources.census(), live_after_record);
    assert!(resources.active_owner().unwrap().state.lock().materialized > before_replacement);
    let basis = fork.entry_basis(Some(&funding), true).unwrap();
    let mut accepted =
        FormattedState::new_entry(Some(&funding), original.frames.clone(), None).unwrap();
    accepted.accept(&fork).unwrap();
    let spent_work = resources.active_owner().unwrap().state.lock().work;
    let spent_records = resources.active_owner().unwrap().state.lock().materialized;
    assert!(spent_work > work_before);
    assert!(spent_records > records_before);
    drop(fork);
    assert!(resources.census().bytes > baseline.bytes);
    drop(accepted);
    assert!(resources.census().bytes > baseline.bytes);
    drop(basis);
    assert_eq!(resources.census(), baseline);
    assert_eq!(
        resources.active_owner().unwrap().state.lock().work,
        spent_work
    );
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized,
        spent_records
    );
}

#[test]
fn borrowed_entry_payload_clone_shares_live_header_and_keeps_completed_charges() {
    use crate::text_layout::formatted::{EntryCopies, SharedPayload};
    use crate::text_layout::scope_alignment::EntryFunding;
    let prepared = captured("X");
    let resources = owner(&prepared);
    let funding = EntryFunding::new(resources.clone(), prepared)
        .unwrap()
        .unwrap();
    let baseline = resources.census();
    let mut copies = EntryCopies::default();
    let mut rows = copies.rows_ref::<u32>(Some(funding.as_ref()), 3).unwrap();
    funding.as_ref().visit(3).unwrap();
    rows.extend([1, 2, 3]);
    let payload = SharedPayload::with_copies_ref(rows, copies, Some(funding.as_ref())).unwrap();
    let live = resources.census();
    let work = resources.active_owner().unwrap().state.lock().work;
    let records = resources.active_owner().unwrap().state.lock().materialized;
    let clone = payload.clone();
    assert_eq!(&**clone, &[1, 2, 3]);
    assert_eq!(resources.census(), live);
    drop(payload);
    assert_eq!(resources.census(), live);
    let clone = match clone.into_native() {
        Err(owner) => owner,
        Ok(_) => panic!("funded rows keep their original custody"),
    };
    assert_eq!(resources.census(), live);
    let mut rows = clone.into_rows();
    assert_eq!(rows.next(), Some(1));
    assert_eq!(rows.next(), Some(2));
    assert_eq!(rows.next(), Some(3));
    assert_eq!(rows.next(), None);
    assert_eq!(resources.census(), live);
    drop(rows);
    assert_eq!(resources.census(), baseline);
    assert_eq!(resources.active_owner().unwrap().state.lock().work, work);
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized,
        records
    );
    let ordinary = SharedPayload::new(vec![4_u32, 5], None).unwrap();
    assert_eq!(ordinary.into_native().unwrap(), [4, 5]);
    assert_eq!(resources.census(), baseline);
}

#[test]
fn funded_margin_fork_grows_and_replaces_without_refunding_history() {
    use crate::block::fragment_margins::State;
    use crate::engine::block_scope::ProductionId;
    use crate::text_layout::scope_alignment::EntryFunding;
    let prepared = captured("X");
    let resources = owner(&prepared);
    let funding = EntryFunding::new(resources.clone(), prepared)
        .unwrap()
        .unwrap();
    let baseline = resources.census();
    let mut source = State::<u32, f64>::new();
    source.bind(1, ProductionId(7)).unwrap();
    let mut fork = source.fork_entry(Some(&funding)).unwrap();
    for node in 2..20 {
        fork.bind(node, ProductionId(node as usize)).unwrap();
    }
    let live = resources.census();
    let records = resources.active_owner().unwrap().state.lock().materialized;
    fork.bind(1, ProductionId(9)).unwrap();
    assert_eq!(resources.census(), live);
    assert!(resources.active_owner().unwrap().state.lock().materialized > records);
    assert!(
        fork.reported(1, crate::PhysicalSide::Top)
            .unwrap()
            .is_empty()
    );
    let spent_work = resources.active_owner().unwrap().state.lock().work;
    let spent_records = resources.active_owner().unwrap().state.lock().materialized;
    drop(fork);
    assert_eq!(resources.census(), baseline);
    assert_eq!(
        resources.active_owner().unwrap().state.lock().work,
        spent_work
    );
    assert_eq!(
        resources.active_owner().unwrap().state.lock().materialized,
        spent_records
    );
}
