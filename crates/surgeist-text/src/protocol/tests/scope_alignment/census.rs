//! Allocation identity, related-result coexistence and bounded census scratch.
//! New API functional qualification; no allocator capacity is an oracle.
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn fixture() -> (
    System,
    PreparedText,
    SelectedLine,
    CapturedScope,
    ScopeAlignmentLimits,
) {
    let captured = Arc::new(process_contextual_source(super::prepared::two_leaves()).unwrap());
    let scope = captured.captured_scope(1).unwrap();
    let caps = scope.scope_alignment_limits().unwrap();
    let analysis = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analysis = crate::test_wrapping::bind_missing(&analysis, Wrap::Word);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    let mut request = super::super::request(&prepared);
    request.context.available_inline = None;
    request.context.spacing = SpacingPolicy::None;
    request.context.alignment = Alignment::Start;
    let selected = prepared.select(request).unwrap();
    (system, prepared, selected, scope, caps)
}
fn finalized(selected: &SelectedLine, displacement: f64) -> FinalizedLine {
    assert_eq!(selected.scopes().len(), 1);
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                scope: selected.scopes().next().unwrap(),
                displacement: BaselineShift::try_new(displacement).unwrap(),
            }]),
        })
        .unwrap()
}
fn full(caps: ScopeAlignmentLimits) -> ScopeAlignmentAllowance {
    ScopeAlignmentAllowance::new(caps.records(), caps.payload_bytes())
}
fn census(
    input: ScopeAlignmentProduct<'_>,
    related: &[ScopeAlignmentProduct<'_>],
    caps: ScopeAlignmentLimits,
) -> ScopeAlignmentCensus {
    scope_alignment_census(
        ScopeAlignmentCensusInput::Product(input),
        ScopeAlignmentProducts::Mixed(related),
        full(caps),
    )
    .unwrap()
}
fn error(value: Result<ScopeAlignmentCensus>) -> Error {
    match value {
        Err(error) => error,
        Ok(_) => panic!("insufficient census allowance was accepted"),
    }
}
fn reason(error: &Error) -> ScopeAlignmentError {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    let Some(ErrorDetail::ScopeAlignment { reason }) = error.detail() else {
        panic!("expected exact scope resource refusal: {error:?}")
    };
    *reason
}
fn refused(error: Error, kind: ScopeAlignmentLimitKind, bound: usize) {
    let ScopeAlignmentError::Limit {
        kind: actual,
        required,
        limit,
    } = reason(&error)
    else {
        panic!("expected limit refusal: {error:?}")
    };
    assert_eq!(actual, kind);
    assert_eq!(limit, bound);
    assert!(required > bound);
}
fn returned(
    result: &ScopeAlignmentCensus,
    input: ScopeAlignmentRetention,
    external: ScopeAlignmentRetention,
    caps: ScopeAlignmentLimits,
) {
    assert_eq!(result.input_retention(), input);
    assert_eq!(result.external_retention(), external);
    let records = input.records().checked_add(external.records()).unwrap();
    let bytes = input
        .payload_bytes()
        .checked_add(external.payload_bytes())
        .unwrap();
    let usage = result.usage();
    // Seen-vector storage is transaction scratch, not retained census output.
    // Rows stay materialized in usage after scratch byte custody is released.
    assert_eq!(usage.payload_bytes(), bytes);
    assert!(usage.records() > records);
    assert!(usage.peak_payload_bytes() > bytes);
    assert!(usage.structural_work() > 0);
    assert_eq!(
        usage.font_queries(),
        0,
        "identity traversal has no provider work"
    );
    assert!(usage.records() <= caps.records());
    assert!(usage.structural_work() <= caps.structural_work());
    assert!(usage.peak_payload_bytes() <= caps.payload_bytes());
}
fn placed(prepared: &PreparedText, line: &FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 0.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Horizontal,
            atomics: vec![],
        })],
    })
    .unwrap()
}
fn geometry(prepared: &PreparedText, line: &FinalizedLine, displacement: f64) {
    // Genuine Ahem X58 advance1000, ascent800/descent200 at10px. Outside8/2
    // and inside(8+d)/(2-d) yield common(8+d)/2 for the positive d used here.
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (20.0, 8.0 + displacement, 10.0 + displacement)
    );
    assert_eq!(line.scope_allocations().len(), 1);
    assert_eq!(line.scope_allocations()[0].displacement.get(), displacement);
    let layout = placed(prepared, line);
    assert_eq!(
        layout.bounds(),
        Rect::new(0.0, 92.0 - displacement, 20.0, 10.0 + displacement)
    );
    assert_eq!(
        layout
            .run_views()
            .map(|run| (run.origin(), run.range()))
            .collect::<Vec<_>>(),
        [
            (Point::new(0.0, 100.0), Range::new(0, 1)),
            (Point::new(10.0, 100.0 - displacement), Range::new(1, 2))
        ]
    );
    assert_eq!(layout.source().text().code_units(), [88, 88]);
}

#[test]
fn input_only_census_matches_each_actual_product_and_releases_identity_scratch() {
    let (system, prepared, selected, scope, caps) = fixture();
    let line = finalized(&selected, 3.0);
    let prepared_usage = prepared.scope_alignment_usage();
    let selected_usage = selected.scope_alignment_usage();
    let final_usage = line.scope_alignment_usage();
    let stats = system.stats();
    for (input, expected) in [
        (
            ScopeAlignmentProduct::Prepared(&prepared),
            prepared.scope_alignment_retention().unwrap(),
        ),
        (
            ScopeAlignmentProduct::Selected(&selected),
            selected.scope_alignment_retention().unwrap(),
        ),
        (
            ScopeAlignmentProduct::Finalized(&line),
            line.scope_alignment_retention().unwrap(),
        ),
    ] {
        let result = census(input, &[], caps);
        returned(&result, expected, ScopeAlignmentRetention::default(), caps);
    }
    assert_eq!(prepared.scope_alignment_usage(), prepared_usage);
    assert_eq!(selected.scope_alignment_usage(), selected_usage);
    assert_eq!(line.scope_alignment_usage(), final_usage);
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    geometry(&prepared, &line, 3.0);
    assert_eq!(system.stats(), stats);
}

#[test]
fn finalized_input_owns_its_prepared_selected_and_cloned_handles_before_related_traversal() {
    let (system, prepared, selected, _scope, caps) = fixture();
    let line = finalized(&selected, 3.0);
    let p_clone = prepared.clone();
    let s_clone = selected.clone();
    let f_clone = line.clone();
    let stats = system.stats();
    let related = [
        ScopeAlignmentProduct::Prepared(&p_clone),
        ScopeAlignmentProduct::Selected(&s_clone),
        ScopeAlignmentProduct::Finalized(&f_clone),
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
        ScopeAlignmentProduct::Finalized(&line),
    ];
    let result = census(ScopeAlignmentProduct::Finalized(&line), &related, caps);
    returned(
        &result,
        line.scope_alignment_retention().unwrap(),
        ScopeAlignmentRetention::default(),
        caps,
    );
    assert_eq!(system.stats(), stats);
    geometry(&prepared, &line, 3.0);
}

#[test]
fn independent_finalizations_add_only_their_own_output_custody_and_duplicates_add_nothing() {
    let (system, prepared, selected, _scope, caps) = fixture();
    let first = finalized(&selected, 3.0);
    let second = finalized(&selected, 5.0);
    let second_clone = second.clone();
    let first_clone = first.clone();
    let prepared_clone = prepared.clone();
    let selected_clone = selected.clone();
    let parent = selected.scope_alignment_retention().unwrap();
    let a = first.scope_alignment_retention().unwrap();
    let b = second.scope_alignment_retention().unwrap();
    let stats = system.stats();
    assert_eq!(first.selected_line(), selected);
    assert_eq!(second.selected_line(), selected);
    geometry(&prepared, &first, 3.0);
    geometry(&prepared, &second, 5.0);
    let one = census(
        ScopeAlignmentProduct::Finalized(&first),
        &[ScopeAlignmentProduct::Finalized(&second)],
        caps,
    );
    let external = one.external_retention();
    // The KNOWN shared subtree is the very same SelectedLine, not the first
    // finalized result. The second result's actual output is a fresh addition
    // even when the two finalizations happen to have equal byte/record counts.
    assert_eq!(
        external.records(),
        b.records().checked_sub(parent.records()).unwrap()
    );
    assert_eq!(
        external.payload_bytes(),
        b.payload_bytes()
            .checked_sub(parent.payload_bytes())
            .unwrap()
    );
    assert!(external.records() > 0);
    assert!(external.payload_bytes() > 0);
    returned(&one, a, external, caps);
    let repeated = census(
        ScopeAlignmentProduct::Finalized(&first_clone),
        &[
            ScopeAlignmentProduct::Selected(&selected),
            ScopeAlignmentProduct::Finalized(&second_clone),
            ScopeAlignmentProduct::Prepared(&prepared_clone),
            ScopeAlignmentProduct::Finalized(&first),
            ScopeAlignmentProduct::Finalized(&second),
            ScopeAlignmentProduct::Selected(&selected_clone),
            ScopeAlignmentProduct::Finalized(&second_clone),
        ],
        caps,
    );
    returned(&repeated, a, external, caps);
    let reversed = census(
        ScopeAlignmentProduct::Finalized(&second),
        &[ScopeAlignmentProduct::Finalized(&first)],
        caps,
    );
    assert_eq!(
        reversed.external_retention().records(),
        a.records() - parent.records()
    );
    assert_eq!(
        reversed.external_retention().payload_bytes(),
        a.payload_bytes() - parent.payload_bytes()
    );
    assert_eq!(
        a.records() + external.records(),
        b.records() + reversed.external_retention().records()
    );
    assert_eq!(
        a.payload_bytes() + external.payload_bytes(),
        b.payload_bytes() + reversed.external_retention().payload_bytes()
    );
    assert_eq!(system.stats(), stats);
}

#[test]
fn related_traversal_order_cannot_charge_the_shared_parent_more_than_once() {
    let (system, prepared, selected, _scope, caps) = fixture();
    let first = finalized(&selected, 3.0);
    let second = finalized(&selected, 5.0);
    let p = prepared.scope_alignment_retention().unwrap();
    let s = selected.scope_alignment_retention().unwrap();
    let a = first.scope_alignment_retention().unwrap();
    let b = second.scope_alignment_retention().unwrap();
    let stats = system.stats();
    let forward = census(
        ScopeAlignmentProduct::Prepared(&prepared),
        &[
            ScopeAlignmentProduct::Selected(&selected),
            ScopeAlignmentProduct::Finalized(&first),
            ScopeAlignmentProduct::Finalized(&second),
        ],
        caps,
    );
    let reverse = census(
        ScopeAlignmentProduct::Prepared(&prepared),
        &[
            ScopeAlignmentProduct::Finalized(&second),
            ScopeAlignmentProduct::Finalized(&first),
            ScopeAlignmentProduct::Selected(&selected),
            ScopeAlignmentProduct::Prepared(&prepared),
        ],
        caps,
    );
    let external = forward.external_retention();
    // One selected subtree beyond Prepared, plus each actual output subtree.
    assert_eq!(
        external.records(),
        (s.records() - p.records()) + (a.records() - s.records()) + (b.records() - s.records())
    );
    assert_eq!(
        external.payload_bytes(),
        (s.payload_bytes() - p.payload_bytes())
            + (a.payload_bytes() - s.payload_bytes())
            + (b.payload_bytes() - s.payload_bytes())
    );
    returned(&forward, p, external, caps);
    returned(&reverse, p, external, caps);
    assert_eq!(system.stats(), stats);
}

#[test]
fn census_allowances_admit_identity_scratch_and_failures_preserve_both_published_lines() {
    let (system, prepared, selected, scope, caps) = fixture();
    let first = finalized(&selected, 3.0);
    let second = finalized(&selected, 5.0);
    let old_a = placed(&prepared, &first);
    let old_b = placed(&prepared, &second);
    let a_ret = first.scope_alignment_retention();
    let b_ret = second.scope_alignment_retention();
    let a_usage = first.scope_alignment_usage();
    let b_usage = second.scope_alignment_usage();
    let stats = system.stats();
    let baseline = census(
        ScopeAlignmentProduct::Finalized(&first),
        &[ScopeAlignmentProduct::Finalized(&second)],
        caps,
    );
    let input = baseline.input_retention();
    let external = baseline.external_retention();
    let records = input.records().checked_add(external.records()).unwrap();
    let bytes = input
        .payload_bytes()
        .checked_add(external.payload_bytes())
        .unwrap();
    // These caps admit all actual retained products but no seen-vector rows or
    // bytes. Refusal thus qualifies scratch accounting without guessing its
    // allocation capacity or assuming a particular growth strategy.
    refused(
        error(scope_alignment_census(
            ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&first)),
            ScopeAlignmentProducts::Mixed(&[ScopeAlignmentProduct::Finalized(&second)]),
            ScopeAlignmentAllowance::new(records, caps.payload_bytes()),
        )),
        ScopeAlignmentLimitKind::Records,
        records,
    );
    refused(
        error(scope_alignment_census(
            ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&first)),
            ScopeAlignmentProducts::Mixed(&[ScopeAlignmentProduct::Finalized(&second)]),
            ScopeAlignmentAllowance::new(caps.records(), bytes),
        )),
        ScopeAlignmentLimitKind::PayloadBytes,
        bytes,
    );
    let sufficient = ScopeAlignmentAllowance::new(
        baseline.usage().records(),
        baseline.usage().peak_payload_bytes(),
    );
    assert!(sufficient.records() < caps.records());
    assert!(sufficient.payload_bytes() < caps.payload_bytes());
    let reduced = scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&first)),
        ScopeAlignmentProducts::Mixed(&[ScopeAlignmentProduct::Finalized(&second)]),
        sufficient,
    )
    .unwrap();
    returned(&reduced, input, external, caps);
    assert_eq!(first.scope_alignment_retention(), a_ret);
    assert_eq!(second.scope_alignment_retention(), b_ret);
    assert_eq!(first.scope_alignment_usage(), a_usage);
    assert_eq!(second.scope_alignment_usage(), b_usage);
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    assert_eq!(old_a.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    assert_eq!(old_b.bounds(), Rect::new(0.0, 87.0, 20.0, 15.0));
    geometry(&prepared, &first, 3.0);
    geometry(&prepared, &second, 5.0);
    let retry = census(
        ScopeAlignmentProduct::Finalized(&first),
        &[ScopeAlignmentProduct::Finalized(&second)],
        caps,
    );
    returned(&retry, input, external, caps);
    assert_eq!(system.stats(), stats);
}

#[test]
fn census_allowances_cannot_raise_either_immutable_input_capture_bound() {
    let (system, prepared, selected, scope, caps) = fixture();
    let line = finalized(&selected, 3.0);
    let stats = system.stats();
    for (allowance, kind, requested, captured) in [
        (
            ScopeAlignmentAllowance::new(
                caps.records().checked_add(1).unwrap(),
                caps.payload_bytes(),
            ),
            ScopeAlignmentLimitKind::Records,
            caps.records() + 1,
            caps.records(),
        ),
        (
            ScopeAlignmentAllowance::new(
                caps.records(),
                caps.payload_bytes().checked_add(1).unwrap(),
            ),
            ScopeAlignmentLimitKind::PayloadBytes,
            caps.payload_bytes() + 1,
            caps.payload_bytes(),
        ),
    ] {
        for input in [
            ScopeAlignmentProduct::Prepared(&prepared),
            ScopeAlignmentProduct::Selected(&selected),
            ScopeAlignmentProduct::Finalized(&line),
        ] {
            let error = error(scope_alignment_census(
                ScopeAlignmentCensusInput::Product(input),
                ScopeAlignmentProducts::Mixed(&[]),
                allowance,
            ));
            assert_eq!(
                reason(&error),
                ScopeAlignmentError::Limit {
                    kind,
                    required: requested,
                    limit: captured
                }
            );
        }
    }
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    geometry(&prepared, &line, 3.0);
    assert_eq!(system.stats(), stats);
}

#[test]
fn weak_product_follows_canonical_clones_and_owned_fragments_without_retaining_the_line() {
    let (system, prepared, selected, _scope, _caps) = fixture();
    let line = finalized(&selected, 3.0);
    let owned_clone = line.clone();
    let weak = line
        .scope_alignment_weak_product()
        .expect("active finalization has feature custody");
    let from_clone = owned_clone.scope_alignment_weak_product().unwrap();
    let other = finalized(&selected, 5.0);
    let other_weak = other.scope_alignment_weak_product().unwrap();
    let stats = system.stats();
    assert!(weak.same_product(&from_clone));
    assert!(from_clone.same_product(&weak));
    assert!(!weak.same_product(&other_weak));
    assert_eq!(weak.upgrade().unwrap(), line);
    geometry(&prepared, &line, 3.0);
    geometry(&prepared, &other, 5.0);
    drop(line);
    // A raw public FinalizedLine clone is an actual strong canonical owner.
    // No registry wrapper or private identity token is involved.
    assert_eq!(weak.upgrade().unwrap(), owned_clone);
    let fragment = owned_clone
        .fragments()
        .find(|fragment| fragment.view().aggregate_range() == Some(Range::new(1, 2)))
        .expect("actual inside-source fragment");
    assert!(matches!(
        fragment.view().owner(),
        FinalizedFragmentOwner::Origin(_)
    ));
    let fragment_clone = fragment.clone();
    {
        let actual_line = fragment.finalized_line();
        assert!(weak.same_product(&actual_line.scope_alignment_weak_product().unwrap()));
        assert_eq!(actual_line, owned_clone);
    }
    drop(owned_clone);
    // Only the issued owned source fragments now retain this finalized Arc.
    {
        let upgraded = weak
            .upgrade()
            .expect("owned fragment keeps the canonical product live");
        assert_eq!(upgraded, fragment.finalized_line());
        geometry(&prepared, &upgraded, 3.0);
    }
    drop(fragment);
    {
        let upgraded = weak
            .upgrade()
            .expect("owned fragment clone is also a strong owner");
        assert_eq!(upgraded, fragment_clone.finalized_line());
        geometry(&prepared, &upgraded, 3.0);
    }
    drop(fragment_clone);
    assert!(weak.upgrade().is_none());
    assert!(from_clone.upgrade().is_none());
    assert!(
        weak.same_product(&from_clone),
        "weak identity survives expiration"
    );
    assert!(!weak.same_product(&other_weak));
    // Prepared, Selected and a different finalized result stay usable/live;
    // they share parent custody without retaining this specific final product.
    assert_eq!(other.selected_line(), selected);
    assert_eq!(selected.prepared_text(), prepared);
    geometry(&prepared, &other, 5.0);
    assert_eq!(other_weak.upgrade().unwrap(), other);
    assert!(weak.upgrade().is_none());
    assert_eq!(system.stats(), stats);
}

#[test]
fn an_ordinary_no_feature_finalization_issues_no_scope_weak_product() {
    let mut system = super::super::system();
    let prepared = super::super::prepared(&mut system, "XX", WhiteSpace::Preserve);
    assert_eq!(prepared.scope_alignment_retention(), None);
    let selected = prepared.select(super::super::request(&prepared)).unwrap();
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    let stats = system.stats();
    assert_eq!(
        line.feedback().advance,
        40.0,
        "two real Ahem X58/1000 cells at20px"
    );
    assert!(line.scope_alignment_weak_product().is_none());
    let cloned = line.clone();
    assert!(cloned.scope_alignment_weak_product().is_none());
    let fragment = line
        .fragments()
        .next()
        .expect("ordinary actual source fragment");
    assert_eq!(fragment.view().aggregate_range(), Some(Range::new(0, 1)));
    // One origin intersects two actual selected text units; a fragment is an
    // occurrence intersection, not an implicit whole-origin aggregation.
    assert_eq!(
        line.fragments()
            .map(|f| f.view().aggregate_range())
            .collect::<Vec<_>>(),
        [Some(Range::new(0, 1)), Some(Range::new(1, 2))]
    );
    assert_eq!(
        line.prepared_text()
            .analyzed_source()
            .source()
            .text()
            .code_units(),
        [88, 88]
    );
    assert!(
        fragment
            .finalized_line()
            .scope_alignment_weak_product()
            .is_none()
    );
    assert_eq!(system.stats(), stats);
}

fn context_only(
    limits: Option<ScopeAlignmentLimits>,
) -> (System, PreparedText, SelectedLine, CapturedScope) {
    let mut input = super::capture::captured_input(0);
    input.scope_alignment_limits = limits;
    if limits.is_some() {
        let SourceProcessingScope::Block { font_reference, .. } = &mut input.scopes[0] else {
            unreachable!()
        };
        *font_reference = Some(super::capture::reference());
    }
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let domain = captured.captured_scope(0).unwrap();
    let analysis = AnalyzedSource::from_processed_paragraph(
        captured.paragraphs().next().unwrap(),
        AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let analysis = crate::test_wrapping::bind_missing(&analysis, Wrap::Word);
    let mut system = super::super::system();
    let prepared = system
        .prepare(&analysis, PreparationOptions::default())
        .unwrap();
    let selected = prepared.select(super::super::request(&prepared)).unwrap();
    (system, prepared, selected, domain)
}

#[test]
fn census_without_actual_contextual_topology_refuses_missing_capture() {
    let mut system = super::super::system();
    let prepared = super::super::prepared(&mut system, "XX", WhiteSpace::Preserve);
    let selected = prepared.select(super::super::request(&prepared)).unwrap();
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    let stats = system.stats();
    for input in [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
        ScopeAlignmentProduct::Finalized(&line),
    ] {
        assert_eq!(
            reason(&error(scope_alignment_census(
                ScopeAlignmentCensusInput::Product(input),
                ScopeAlignmentProducts::Mixed(&[]),
                ScopeAlignmentAllowance::new(0, 0)
            ))),
            ScopeAlignmentError::MissingCapture
        );
    }
    assert_eq!(line.feedback().advance, 40.0);
    assert_eq!(system.stats(), stats);
}

#[test]
fn census_of_a_real_capture_without_alignment_limits_refuses_limits_required() {
    let (system, prepared, selected, domain) = context_only(None);
    assert_eq!(domain.scope_alignment_limits(), None);
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    let stats = system.stats();
    for input in [
        ScopeAlignmentProduct::Prepared(&prepared),
        ScopeAlignmentProduct::Selected(&selected),
        ScopeAlignmentProduct::Finalized(&line),
    ] {
        assert_eq!(
            reason(&error(scope_alignment_census(
                ScopeAlignmentCensusInput::Product(input),
                ScopeAlignmentProducts::Mixed(&[]),
                ScopeAlignmentAllowance::new(0, 0)
            ))),
            ScopeAlignmentError::LimitsRequired
        );
    }
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(system.stats(), stats);
}

#[test]
fn explicit_capture_limits_admit_a_domain_font_context_with_zero_alignment_targets() {
    let caps = ScopeAlignmentLimits::new(4096, 0, 1_000_000, 16_777_216);
    let (system, prepared, selected, domain) = context_only(Some(caps));
    assert_eq!(domain.scope_alignment_limits(), Some(caps));
    assert_eq!(prepared.scope_alignment_usage().unwrap().font_queries(), 0);
    assert_eq!(selected.scopes().len(), 0);
    let line = selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    assert!(line.scope_allocations().is_empty());
    let stats = system.stats();
    // The real domain context is captured but is not itself an alignment
    // target or a scope-font query. Census admission is still meaningful.
    assert_eq!(
        reason(&prepared.scope_metrics(&domain).unwrap_err()),
        ScopeAlignmentError::MissingMetrics { scope: 0 }
    );
    for (input, expected) in [
        (
            ScopeAlignmentProduct::Prepared(&prepared),
            prepared.scope_alignment_retention().unwrap(),
        ),
        (
            ScopeAlignmentProduct::Selected(&selected),
            selected.scope_alignment_retention().unwrap(),
        ),
        (
            ScopeAlignmentProduct::Finalized(&line),
            line.scope_alignment_retention().unwrap(),
        ),
    ] {
        let result = census(input, &[], caps);
        returned(&result, expected, ScopeAlignmentRetention::default(), caps);
    }
    assert_eq!(line.feedback().advance, 20.0);
    assert_eq!(domain.scope_alignment_limits(), Some(caps));
    assert_eq!(system.stats(), stats);
}

#[test]
fn sparse_finalized_slots_and_the_same_mixed_products_have_identical_unique_custody() {
    let (system, prepared, selected, _scope, caps) = fixture();
    let first = finalized(&selected, 3.0);
    let second = finalized(&selected, 5.0);
    let slots = [
        None,
        Some(second.clone()),
        None,
        Some(first.clone()),
        Some(second.clone()),
        None,
    ];
    let stats = system.stats();
    let mixed = census(
        ScopeAlignmentProduct::Finalized(&first),
        &[
            ScopeAlignmentProduct::Finalized(&second),
            ScopeAlignmentProduct::Finalized(&first),
            ScopeAlignmentProduct::Finalized(&second),
        ],
        caps,
    );
    let sparse = scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&first)),
        ScopeAlignmentProducts::FinalizedSlots(&slots),
        full(caps),
    )
    .unwrap();
    returned(
        &sparse,
        mixed.input_retention(),
        mixed.external_retention(),
        caps,
    );
    assert!(sparse.external_retention().records() > 0);
    assert!(sparse.external_retention().payload_bytes() > 0);
    let empty = [None, None, None];
    let empty = scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Finalized(&first)),
        ScopeAlignmentProducts::FinalizedSlots(&empty),
        full(caps),
    )
    .unwrap();
    returned(
        &empty,
        first.scope_alignment_retention().unwrap(),
        ScopeAlignmentRetention::default(),
        caps,
    );
    geometry(&prepared, &first, 3.0);
    geometry(&prepared, &second, 5.0);
    assert_eq!(system.stats(), stats);
}
