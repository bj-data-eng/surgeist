//! Operation allowances bound actual retained inputs, not just new output.
//! Geometry remains the independently derived Ahem10 +3 scope witness.
use crate::*;
use std::sync::Arc;

const AHEM: &[u8] = include_bytes!("../../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn fixture() -> (System, PreparedText, CapturedScope, ScopeAlignmentLimits) {
    fixture_from(super::prepared::two_leaves())
}
fn fixture_from(
    input: ContextualSourceInput,
) -> (System, PreparedText, CapturedScope, ScopeAlignmentLimits) {
    let captured = Arc::new(process_contextual_source(input).unwrap());
    let scope = captured.captured_scope(1).unwrap();
    let limits = scope.scope_alignment_limits().unwrap();
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
    (system, prepared, scope, limits)
}
fn request(p: &PreparedText) -> LineSelectionRequest {
    let mut value = super::super::request(p);
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::None;
    value.context.alignment = Alignment::Start;
    value
}
fn allocated(selected: &SelectedLine) -> LineAllocations {
    assert_eq!(selected.scopes().len(), 1);
    LineAllocations {
        spacing: SpacingAllocations::None,
        scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
            scope: selected.scopes().next().unwrap(),
            displacement: BaselineShift::try_new(3.0).unwrap(),
        }]),
    }
}
fn input_footprint(scopes: &ScopeAllocations, spacing: &SpacingAllocations) -> (usize, usize) {
    fn vector<T>(rows: &Vec<T>) -> (usize, usize) {
        (
            rows.len(),
            rows.capacity()
                .checked_mul(std::mem::size_of::<T>())
                .unwrap(),
        )
    }
    let scope = match scopes {
        ScopeAllocations::None => (0, 0),
        ScopeAllocations::Complete(rows) => vector(rows),
    };
    let spacing = match spacing {
        SpacingAllocations::None => (0, 0),
        SpacingAllocations::Complete(rows) => vector(rows),
    };
    // The adopted gate seeds the WHOLE typed input atomically before scratch:
    // retained product + actual incoming rows/backing allocations. Vec/enum
    // headers here are stack arguments, not extra heap custody. Referenced
    // selected/prepared owners are already present in the product receipt.
    (
        scope.0.checked_add(spacing.0).unwrap(),
        scope.1.checked_add(spacing.1).unwrap(),
    )
}
fn limit(error: Error, kind: ScopeAlignmentLimitKind, required: usize, bound: usize) {
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::Limit {
                kind,
                required,
                limit: bound
            },
        })
    );
}
fn full(limits: ScopeAlignmentLimits) -> ScopeAlignmentAllowance {
    ScopeAlignmentAllowance::new(limits.records(), limits.payload_bytes())
}
fn minus_record(
    seed: ScopeAlignmentRetention,
    limits: ScopeAlignmentLimits,
) -> ScopeAlignmentAllowance {
    assert!(
        seed.records() > 0,
        "active operation must retain real feature rows"
    );
    ScopeAlignmentAllowance::new(seed.records() - 1, limits.payload_bytes())
}
fn minus_byte(
    seed: ScopeAlignmentRetention,
    limits: ScopeAlignmentLimits,
) -> ScopeAlignmentAllowance {
    assert!(
        seed.payload_bytes() > 0,
        "active operation must retain real feature payload"
    );
    ScopeAlignmentAllowance::new(limits.records(), seed.payload_bytes() - 1)
}
fn sufficient(usage: ScopeAlignmentUsage, limits: ScopeAlignmentLimits) -> ScopeAlignmentAllowance {
    // Live retention alone cannot bound materialized scratch or its peak. Use
    // this same operation's real completed materialization/peak observations,
    // never allocator capacities or geometry sampled as an expected result.
    assert!(usage.records() < limits.records());
    assert!(usage.peak_payload_bytes() < limits.payload_bytes());
    ScopeAlignmentAllowance::new(usage.records(), usage.peak_payload_bytes())
}
fn layout(p: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: p.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
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
fn geometry(p: &PreparedText, scope: &CapturedScope, line: &FinalizedLine) {
    // Two actual X58/1000-unit glyphs at10px. The outside8/2 leaf and the
    // shifted11/-1 leaf give common11/2, height13, inline advance20.
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (20.0, 11.0, 13.0)
    );
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|g| (g.id(), g.x(), g.y(), g.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(58, 0.0, 0.0, 10.0), (58, 0.0, 0.0, 10.0)]
    );
    assert_eq!(line.scope_allocations().len(), 1);
    let frame = &line.scope_allocations()[0];
    assert_eq!(&frame.scope, scope);
    assert_eq!(frame.displacement.get(), 3.0);
    assert_eq!(frame.cumulative_displacement.get(), 3.0);
    let placed = layout(p, line.clone());
    assert_eq!(placed.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    assert_eq!(
        placed
            .run_views()
            .map(|run| (run.origin(), run.range()))
            .collect::<Vec<_>>(),
        [
            (Point::new(0.0, 100.0), Range::new(0, 1)),
            (Point::new(10.0, 97.0), Range::new(1, 2))
        ]
    );
    assert_eq!(
        placed
            .cursor(Cursor::new(1, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(10.0, 92.0, 1.0, 10.0)
    );
    let scoped = placed
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(1, Affinity::After))
        .unwrap();
    assert_eq!(
        placed.position_geometry(&scoped).unwrap().rect(),
        Rect::new(10.0, 89.0, 1.0, 10.0)
    );
    assert_eq!(placed.source().text().code_units(), [88, 88]);
}

#[test]
fn selection_allowance_must_admit_the_actual_retained_preparation_before_new_output() {
    let (_system, p, scope, caps) = fixture();
    let prepared_usage = p.scope_alignment_usage();
    let seed = p.scope_alignment_retention().unwrap();
    let accepted = p.select(request(&p)).unwrap();
    let old = accepted.finalize(allocated(&accepted)).unwrap();
    let published = layout(&p, old.clone());
    let records = minus_record(seed, caps);
    limit(
        p.select_with_scope_allowance(request(&p), records)
            .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
        seed.records(),
        seed.records() - 1,
    );
    let bytes = minus_byte(seed, caps);
    limit(
        p.select_with_scope_allowance(request(&p), bytes)
            .unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
        seed.payload_bytes(),
        seed.payload_bytes() - 1,
    );
    assert_eq!(p.scope_alignment_usage(), prepared_usage);
    assert_eq!(p.scope_alignment_retention(), Some(seed));
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    geometry(&p, &scope, &old);
    assert_eq!(published.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    let retry = p.select(request(&p)).unwrap();
    geometry(&p, &scope, &retry.finalize(allocated(&retry)).unwrap());
}

#[test]
fn finalization_allowance_must_admit_the_exact_selected_input_and_preserve_an_old_result() {
    let (_system, p, scope, caps) = fixture();
    let selected = p.select(request(&p)).unwrap();
    let seed = selected.scope_alignment_retention().unwrap();
    let selected_usage = selected.scope_alignment_usage();
    let old = selected.finalize(allocated(&selected)).unwrap();
    let published = layout(&p, old.clone());
    let record_input = allocated(&selected);
    let record_footprint = input_footprint(&record_input.scopes, &record_input.spacing);
    limit(
        selected
            .finalize_with_scope_allowance(record_input, minus_record(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
        seed.records().checked_add(record_footprint.0).unwrap(),
        seed.records() - 1,
    );
    let byte_input = allocated(&selected);
    let byte_footprint = input_footprint(&byte_input.scopes, &byte_input.spacing);
    limit(
        selected
            .finalize_with_scope_allowance(byte_input, minus_byte(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
        seed.payload_bytes().checked_add(byte_footprint.1).unwrap(),
        seed.payload_bytes() - 1,
    );
    assert_eq!(selected.scope_alignment_usage(), selected_usage);
    assert_eq!(selected.scope_alignment_retention(), Some(seed));
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    geometry(&p, &scope, &old);
    assert_eq!(published.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    geometry(
        &p,
        &scope,
        &selected.finalize(allocated(&selected)).unwrap(),
    );
}

#[test]
fn replay_allowance_includes_the_live_old_finalization_without_double_counting_shared_inputs() {
    let (_system, p, scope, caps) = fixture();
    let selected = p.select(request(&p)).unwrap();
    let old = selected.finalize(allocated(&selected)).unwrap();
    let seed = old.scope_alignment_retention().unwrap();
    let selected_seed = selected.scope_alignment_retention().unwrap();
    // Finalized custody includes selected/prepared input once PLUS actual old
    // scope/occurrence geometry. A selected-only seed omits that real input.
    assert!(seed.records() > selected_seed.records());
    assert!(seed.payload_bytes() > selected_seed.payload_bytes());
    let old_usage = old.scope_alignment_usage();
    let published = layout(&p, old.clone());
    limit(
        old.with_spacing_with_scope_allowance(SpacingAllocations::None, minus_record(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
        seed.records(),
        seed.records() - 1,
    );
    limit(
        old.with_spacing_with_scope_allowance(SpacingAllocations::None, minus_byte(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
        seed.payload_bytes(),
        seed.payload_bytes() - 1,
    );
    // Keep both actual old inputs alive through each failed operation. The
    // bounds are still independently derived, not copied from a new result.
    geometry(&p, &scope, &old);
    assert_eq!(old.scope_alignment_usage(), old_usage);
    assert_eq!(old.scope_alignment_retention(), Some(seed));
    assert_eq!(published.bounds(), Rect::new(0.0, 89.0, 20.0, 13.0));
    geometry(
        &p,
        &scope,
        &old.with_spacing(SpacingAllocations::None).unwrap(),
    );
}

#[test]
fn allowances_above_either_immutable_capture_bound_refuse_instead_of_clamping() {
    let (_system, p, scope, caps) = fixture();
    let selected = p.select(request(&p)).unwrap();
    let old = selected.finalize(allocated(&selected)).unwrap();
    let too_many_records = caps.records().checked_add(1).unwrap();
    let too_many_bytes = caps.payload_bytes().checked_add(1).unwrap();
    for (allowance, kind, requested, captured) in [
        (
            ScopeAlignmentAllowance::new(too_many_records, caps.payload_bytes()),
            ScopeAlignmentLimitKind::Records,
            too_many_records,
            caps.records(),
        ),
        (
            ScopeAlignmentAllowance::new(caps.records(), too_many_bytes),
            ScopeAlignmentLimitKind::PayloadBytes,
            too_many_bytes,
            caps.payload_bytes(),
        ),
    ] {
        limit(
            p.select_with_scope_allowance(request(&p), allowance)
                .unwrap_err(),
            kind,
            requested,
            captured,
        );
        limit(
            selected
                .finalize_with_scope_allowance(allocated(&selected), allowance)
                .unwrap_err(),
            kind,
            requested,
            captured,
        );
        limit(
            old.with_spacing_with_scope_allowance(SpacingAllocations::None, allowance)
                .unwrap_err(),
            kind,
            requested,
            captured,
        );
    }
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    geometry(&p, &scope, &old);
}

#[test]
fn sufficient_reduced_allowances_preserve_real_geometry_custody_and_nonsticky_request_identity() {
    let (system, p, scope, caps) = fixture();
    let capture_usage = scope.scope_alignment_usage();
    let normal = p.select(request(&p)).unwrap();
    let reduced_selection = sufficient(normal.scope_alignment_usage().unwrap(), caps);
    let reduced = p
        .select_with_scope_allowance(request(&p), reduced_selection)
        .unwrap();
    assert_eq!(normal.request(), reduced.request());
    assert_eq!(reduced.scopes().next().unwrap().captured_scope(), scope);
    let normal_line = reduced.finalize(allocated(&reduced)).unwrap();
    let reduced_finalization = sufficient(normal_line.scope_alignment_usage().unwrap(), caps);
    let stats = system.stats();
    let constrained = reduced
        .finalize_with_scope_allowance(allocated(&reduced), reduced_finalization)
        .unwrap();
    geometry(&p, &scope, &constrained);
    assert_eq!(constrained.selected_line(), reduced);
    let normal_replay = constrained.with_spacing(SpacingAllocations::None).unwrap();
    let reduced_replay = sufficient(normal_replay.scope_alignment_usage().unwrap(), caps);
    // The old constrained FinalizedLine remains the actual borrowed input and
    // stays live while its independently allocated replay is checked.
    let replay = constrained
        .with_spacing_with_scope_allowance(SpacingAllocations::None, reduced_replay)
        .unwrap();
    assert_eq!(replay.selected_line(), constrained.selected_line());
    assert_eq!(replay.scope_allocations(), constrained.scope_allocations());
    geometry(&p, &scope, &replay);
    geometry(&p, &scope, &constrained);
    assert_eq!(
        system.stats(),
        stats,
        "finalization/replay use retained resources"
    );
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    assert_eq!(scope.scope_alignment_usage(), capture_usage);
    let ordinary_again = p.select(request(&p)).unwrap();
    assert_eq!(ordinary_again.request(), normal.request());
    geometry(
        &p,
        &scope,
        &ordinary_again.finalize(allocated(&ordinary_again)).unwrap(),
    );
    // Equal-to-capture allowance is also valid and remains per-operation.
    let equal = p
        .select_with_scope_allowance(request(&p), full(caps))
        .unwrap();
    geometry(
        &p,
        &scope,
        &equal
            .finalize_with_scope_allowance(allocated(&equal), full(caps))
            .unwrap(),
    );
}

#[test]
fn a_real_spacing_change_keeps_old_published_geometry_alive_inside_the_replay_seed() {
    let mut input = super::prepared::two_leaves();
    let source = Source::identified("X X", Id::from_u64(1578022), 31);
    input.source = source.clone();
    input.origins = [(0, 2), (2, 3)]
        .into_iter()
        .map(|(start, end)| ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: Range::new(start, end),
            aggregate: Range::new(start, end),
            kind: ParagraphOriginKind::Authored,
        })
        .collect();
    // The existing genuine event topology places X+space outside and the final
    // X inside scope1. Preserved Ahem space is1000 font units, just like X.
    let (system, p, scope, caps) = fixture_from(input);
    let mut req = request(&p);
    req.context.spacing = SpacingPolicy::Spaces;
    let selected = p.select(req).unwrap();
    let slots = selected.spacing_slots().collect::<Vec<_>>();
    assert_eq!(slots.len(), 1);
    let extra = || {
        SpacingAllocations::Complete(vec![SpacingAllocation {
            slot: slots[0].clone(),
            amount: 20.0,
        }])
    };
    let old = selected.finalize(allocated(&selected)).unwrap();
    let old_layout = layout(&p, old.clone());
    let seed = old.scope_alignment_retention().unwrap();
    let old_usage = old.scope_alignment_usage();
    let stats = system.stats();
    let record_input = extra();
    let record_footprint = input_footprint(&ScopeAllocations::None, &record_input);
    limit(
        old.with_spacing_with_scope_allowance(record_input, minus_record(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::Records,
        seed.records().checked_add(record_footprint.0).unwrap(),
        seed.records() - 1,
    );
    let byte_input = extra();
    let byte_footprint = input_footprint(&ScopeAllocations::None, &byte_input);
    limit(
        old.with_spacing_with_scope_allowance(byte_input, minus_byte(seed, caps))
            .unwrap_err(),
        ScopeAlignmentLimitKind::PayloadBytes,
        seed.payload_bytes().checked_add(byte_footprint.1).unwrap(),
        seed.payload_bytes() - 1,
    );
    let ordinary = old.with_spacing(extra()).unwrap();
    let allowance = sufficient(ordinary.scope_alignment_usage().unwrap(), caps);
    let replay = old
        .with_spacing_with_scope_allowance(extra(), allowance)
        .unwrap();
    // Three10px cells originally advance30; exactly20px extra at the actual
    // space moves only the inside X from x20 to40. The +3 over displacement
    // and the11/2 common envelope are independent of this inline expansion.
    assert_eq!(
        (
            old.feedback().advance,
            old.feedback().baseline,
            old.feedback().line_height
        ),
        (30.0, 11.0, 13.0)
    );
    assert_eq!(
        (
            replay.feedback().advance,
            replay.feedback().baseline,
            replay.feedback().line_height
        ),
        (50.0, 11.0, 13.0)
    );
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 89.0, 30.0, 13.0));
    assert_eq!(
        old_layout
            .run_views()
            .filter(|run| run.range() == Range::new(2, 3))
            .map(|run| run.origin())
            .collect::<Vec<_>>(),
        [Point::new(20.0, 97.0)]
    );
    let replay_layout = layout(&p, replay.clone());
    assert_eq!(replay_layout.bounds(), Rect::new(0.0, 89.0, 50.0, 13.0));
    assert_eq!(
        replay_layout
            .run_views()
            .filter(|run| run.range() == Range::new(2, 3))
            .map(|run| run.origin())
            .collect::<Vec<_>>(),
        [Point::new(40.0, 97.0)]
    );
    assert_eq!(old_layout.source().text().code_units(), [88, 32, 88]);
    assert_eq!(replay_layout.source().text().code_units(), [88, 32, 88]);
    assert_eq!(replay.selected_line(), selected);
    assert_eq!(replay.scope_allocations(), old.scope_allocations());
    assert_eq!(old.scope_alignment_retention(), Some(seed));
    assert_eq!(old.scope_alignment_usage(), old_usage);
    assert_eq!(scope.scope_alignment_limits(), Some(caps));
    assert_eq!(system.stats(), stats);
    // Read the already published old Layout again after successful new output.
    assert_eq!(old_layout.bounds(), Rect::new(0.0, 89.0, 30.0, 13.0));
    assert_eq!(
        old_layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(0.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(
        replay_layout
            .cursor(Cursor::new(0, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(0.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(
        old_layout
            .cursor(Cursor::new(2, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(20.0, 92.0, 1.0, 10.0)
    );
    assert_eq!(
        replay_layout
            .cursor(Cursor::new(2, Affinity::After))
            .unwrap()
            .rect(),
        Rect::new(40.0, 92.0, 1.0, 10.0)
    );
    let old_scoped = old_layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::After))
        .unwrap();
    let new_scoped = replay_layout
        .positions()
        .find(|p| p.source_cursor() == Cursor::new(2, Affinity::After))
        .unwrap();
    assert_eq!(
        old_layout.position_geometry(&old_scoped).unwrap().rect(),
        Rect::new(20.0, 89.0, 1.0, 10.0)
    );
    assert_eq!(
        replay_layout.position_geometry(&new_scoped).unwrap().rect(),
        Rect::new(40.0, 89.0, 1.0, 10.0)
    );
}
