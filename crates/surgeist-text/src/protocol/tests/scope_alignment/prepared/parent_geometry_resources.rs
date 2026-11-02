//! BoxEdge request-buffer retention and shared preparation accounting.
//!
//! The moved ControlAllocation Vec supplies an independent input oracle:
//! logical rows and actual capacity times the public element size. These limits
//! qualify retained request ownership, not isolated execution-plan scratch.

use super::*;
use crate::protocol::InlineBoxSource;

#[path = "parent_geometry_resources/work_admission.rs"]
mod work_admission;

struct Funded {
    capture: Arc<ProcessedSource>,
    system: System,
    prepared: PreparedText,
    boxed: LineSelectionRequest,
    contribution: LineSelectionRequest,
    caps: ScopeAlignmentLimits,
}
fn funded() -> Funded {
    // Existing actual parent Hani/child Latn BASE references, NonInitial child,
    // declared fonts and immutable generous Scope limits. No new font/factory.
    let mut input = super::active();
    super::author_child_body(&mut input);
    let SourceProcessingScope::Inline { boundaries, .. } = &mut input.scopes[1] else {
        unreachable!()
    };
    *boundaries = Some(SourceProcessingScopeBoundaries { start: 1, end: 2 });
    input.controls = vec![
        // Genuine unpaired Native endpoint outside the nominated Inline. Its
        // explicit Contribution claims no pair/box/parent; it is NOT a dummy row.
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 0,
            kind: ParagraphControlKind::InlineBoundaryStart,
        },
        ParagraphControl {
            at: 1,
            kind: ParagraphControlKind::InlineBoundaryEnd,
        },
    ];
    use SourceProcessingEvent as E;
    input.events = vec![
        E::EnterScope(0),
        E::EnterParagraph(0),
        E::Control {
            control: 0,
            affinity: Affinity::After,
        },
        E::EnterScope(1),
        E::Control {
            control: 1,
            affinity: Affinity::After,
        },
        E::Origin {
            origin: 0,
            affinity: Affinity::After,
        },
        E::Control {
            control: 2,
            affinity: Affinity::After,
        },
        E::ExitScope(1),
        E::ExitParagraph(0),
        E::ExitScope(0),
    ];
    let (capture, analyzed) = super::analyze(input);
    assert_eq!(capture.source().text(), &Text::from("X"));
    assert_eq!(capture.processed_text(), &Text::from("X"));
    let owner = capture.captured_scope(1).unwrap();
    assert_eq!(owner.parent(), Some(capture.captured_scope(0).unwrap()));
    assert_eq!(owner.entry_control(), capture.captured_control(1));
    assert_eq!(owner.exit_control(), capture.captured_control(2));
    assert_eq!(
        capture.captured_control(0).unwrap().scope(),
        capture.captured_scope(0).unwrap()
    );
    let caps = owner.scope_alignment_limits().unwrap();
    let (parent_bytes, child_bytes) = super::scoped_fonts();
    let mut system = super::system(&parent_bytes, &child_bytes);
    let prepared = system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap();
    assert!(prepared.scope_alignment_retention().is_some());
    assert!(prepared.scope_alignment_usage().is_some());
    let context = prepared
        .inline_box_edge_context(
            InlineBoxSource::captured(owner),
            InlineBoxSource::captured(capture.captured_scope(0).unwrap()),
            LineEdgeSide::Start,
        )
        .unwrap();
    let segment = prepared.source_segments().next().unwrap();
    let offer = prepared
        .opportunity_views()
        .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let mut boxed = super::selected_request(&prepared, segment.start(), offer);
    let controls = prepared
        .control_views()
        .map(|view| view.handle())
        .collect::<Vec<_>>();
    assert_eq!(controls.len(), 3);
    for (index, control) in controls.iter().enumerate() {
        assert_eq!(control.captured_control(), capture.captured_control(index));
    }
    boxed.controls = vec![
        ControlAllocation {
            occurrence: controls[0].clone(),
            strut: None,
            boundary: Some(InlineBoundaryAllocation::contribution(5.0).unwrap()),
        },
        ControlAllocation {
            occurrence: controls[1].clone(),
            strut: None,
            boundary: Some(InlineBoundaryAllocation::box_edge(
                context.clone(),
                crate::InlineBoxEdgeComponents::from_f64(0.0, 1.0, 0.0).unwrap(),
            )),
        },
        ControlAllocation {
            occurrence: controls[2].clone(),
            strut: None,
            boundary: Some(InlineBoundaryAllocation::box_edge(
                context,
                crate::InlineBoxEdgeComponents::from_f64(0.0, 2.0, 0.0).unwrap(),
            )),
        },
    ];
    // Small REAL spare caller capacity distinguishes retained backing from len.
    // No quota is guessed and no large allocation is needed.
    boxed.controls.try_reserve_exact(2).unwrap();
    assert!(boxed.boundary_context.is_empty());
    let mut contribution = boxed.clone();
    for row in &mut contribution.controls {
        let amount = row.boundary.as_ref().unwrap().advance();
        row.boundary = Some(InlineBoundaryAllocation::contribution(amount).unwrap());
    }
    assert_ne!(boxed, contribution);
    Funded {
        capture,
        system,
        prepared,
        boxed,
        contribution,
        caps,
    }
}
fn full(caps: ScopeAlignmentLimits) -> ScopeAlignmentAllowance {
    ScopeAlignmentAllowance::new(caps.records(), caps.payload_bytes())
}
fn census_prepared(f: &Funded) -> ScopeAlignmentCensus {
    scope_alignment_census(
        ScopeAlignmentCensusInput::Product(ScopeAlignmentProduct::Prepared(&f.prepared)),
        ScopeAlignmentProducts::Mixed(&[]),
        full(f.caps),
    )
    .unwrap()
}
fn census_request(f: &Funded, request: &LineSelectionRequest) -> ScopeAlignmentCensus {
    scope_alignment_census(
        ScopeAlignmentCensusInput::Selection {
            prepared: &f.prepared,
            request,
        },
        ScopeAlignmentProducts::Mixed(&[]),
        full(f.caps),
    )
    .unwrap()
}
fn request_buffer(request: &LineSelectionRequest) -> (usize, usize) {
    // Caller-owned PUBLIC buffers only; headers belong to the actual stack or
    // SelectedData owner. Strong Source references alias the same preparation.
    let records = request
        .controls
        .len()
        .checked_add(request.boundary_context.len())
        .unwrap();
    let bytes = request
        .controls
        .capacity()
        .checked_mul(std::mem::size_of::<ControlAllocation>())
        .unwrap()
        .checked_add(
            request
                .boundary_context
                .capacity()
                .checked_mul(std::mem::size_of::<InlineBoundaryContextAllocation>())
                .unwrap(),
        )
        .unwrap();
    (records, bytes)
}
fn finish(selected: &SelectedLine) -> FinalizedLine {
    assert_eq!(selected.scopes().len(), 1);
    // The existing genuine NonInitial target requires its actual allocation;
    // zero displacement here is a supplied real frame, not invented geometry.
    selected
        .finalize(super::allocations(selected.scopes().next().unwrap(), 0.0))
        .unwrap()
}
fn preserved(f: &Funded, line: &FinalizedLine, request: &LineSelectionRequest) {
    assert_eq!(line.prepared_text(), f.prepared);
    assert_eq!(line.selected_line().request(), request);
    assert_eq!(line.feedback().advance, 18.0); // Native5 + BoxStart1 + real X10 + BoxEnd2.
    assert_eq!(f.capture.source().text(), &Text::from("X"));
    assert_eq!(
        line.run_views()
            .flat_map(|run| run
                .glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [(58, 10.0)]
    );
    for (row, expected) in [(0, (0.0, 5.0)), (1, (5.0, 6.0)), (2, (16.0, 18.0))] {
        let slot = line
            .control_slots()
            .find(|slot| slot.control() == request.controls[row].occurrence)
            .unwrap();
        assert_eq!(slot.strip_interval(), expected);
    }
}
fn refused<T>(result: Result<T>, kind: ScopeAlignmentLimitKind, required: usize, limit: usize) {
    let error = match result {
        Err(error) => error,
        Ok(_) => panic!("whole real retained BoxEdge request must be admitted before new output"),
    };
    assert_eq!(error.code, ErrorCode::ScopeAlignmentRefused);
    assert_eq!(
        error.detail(),
        Some(&ErrorDetail::ScopeAlignment {
            reason: ScopeAlignmentError::Limit {
                kind,
                required,
                limit
            },
        })
    );
}

#[test]
fn box_bearing_selection_census_owns_the_whole_mixed_request_backing_once() {
    let f = funded();
    let prepared = census_prepared(&f).input_retention();
    let boxed = census_request(&f, &f.boxed).input_retention();
    let native = census_request(&f, &f.contribution).input_retention();
    let (rows, bytes) = request_buffer(&f.boxed);
    assert_eq!(rows, 3); // TWO BoxEdge rows plus the legitimate unpaired Contribution.
    assert_eq!(
        boxed.records(),
        prepared.records().checked_add(rows).unwrap()
    );
    assert_eq!(
        boxed.payload_bytes(),
        prepared.payload_bytes().checked_add(bytes).unwrap()
    );
    // Merely supplying the same Native scalar rows does not turn their existing
    // allocation route into the new BoxEdge request owner/global Scope quota.
    assert_eq!(native, prepared);
    let selected = f.prepared.select(f.boxed.clone()).unwrap();
    preserved(&f, &finish(&selected), &f.boxed);
}

#[test]
fn selected_request_aliases_deduplicate_but_a_distinct_live_caller_buffer_counts() {
    let f = funded();
    let selected = f.prepared.select(f.boxed.clone()).unwrap();
    let products = [ScopeAlignmentProduct::Selected(&selected)];
    let alias = scope_alignment_census(
        ScopeAlignmentCensusInput::Selection {
            prepared: &f.prepared,
            request: selected.request(),
        },
        ScopeAlignmentProducts::Mixed(&products),
        full(f.caps),
    )
    .unwrap();
    let separate = scope_alignment_census(
        ScopeAlignmentCensusInput::Selection {
            prepared: &f.prepared,
            request: &f.boxed,
        },
        ScopeAlignmentProducts::Mixed(&products),
        full(f.caps),
    )
    .unwrap();
    let (rows, bytes) = request_buffer(&f.boxed);
    let a = alias.input_retention();
    let b = alias.external_retention();
    let c = separate.input_retention();
    let d = separate.external_retention();
    assert_eq!(
        c.records().checked_add(d.records()).unwrap(),
        a.records()
            .checked_add(b.records())
            .unwrap()
            .checked_add(rows)
            .unwrap()
    );
    assert_eq!(
        c.payload_bytes().checked_add(d.payload_bytes()).unwrap(),
        a.payload_bytes()
            .checked_add(b.payload_bytes())
            .unwrap()
            .checked_add(bytes)
            .unwrap()
    );
    // Actual existing capture/font/preparation ancestors are shared in BOTH
    // named snapshots. No duplicate source graph or per-box Arc is charged.
    preserved(&f, &finish(&selected), &f.boxed);
}

#[test]
fn insufficient_whole_request_records_or_backing_refuse_without_poisoning_prior_output() {
    let f = funded();
    let prepared = census_prepared(&f).input_retention();
    let (rows, bytes) = request_buffer(&f.boxed);
    let required_rows = prepared.records().checked_add(rows).unwrap();
    let required_bytes = prepared.payload_bytes().checked_add(bytes).unwrap();
    assert!(required_rows <= f.caps.records() && required_bytes <= f.caps.payload_bytes());
    let old_selected = f.prepared.select(f.contribution.clone()).unwrap();
    let old = finish(&old_selected);
    preserved(&f, &old, &f.contribution);
    let usage = f.prepared.scope_alignment_usage();
    let records = ScopeAlignmentAllowance::new(required_rows - 1, f.caps.payload_bytes());
    // The independently known input footprint exceeds each allowance BEFORE
    // seen-set growth, shaping or any BoxEdge execution-plan allocation.
    refused(
        f.prepared
            .select_with_scope_allowance(f.boxed.clone(), records),
        ScopeAlignmentLimitKind::Records,
        required_rows,
        required_rows - 1,
    );
    // Clone can lawfully shrink spare capacity. Qualify the actual newly owned
    // INPUT backing that this call moves, not a guessed clone growth factor.
    let moved = f.boxed.clone();
    let (_, moved_bytes) = request_buffer(&moved);
    // Allocator-visible capacity is the actual input; no assumed growth factor.
    let moved_required = prepared.payload_bytes().checked_add(moved_bytes).unwrap();
    let moved_payload = ScopeAlignmentAllowance::new(f.caps.records(), moved_required - 1);
    refused(
        f.prepared.select_with_scope_allowance(moved, moved_payload),
        ScopeAlignmentLimitKind::PayloadBytes,
        moved_required,
        moved_required - 1,
    );
    assert_eq!(f.prepared.scope_alignment_usage(), usage);
    preserved(&f, &old, &f.contribution);
    let retry = f
        .prepared
        .select_with_scope_allowance(f.boxed.clone(), full(f.caps))
        .unwrap();
    let line = finish(&retry);
    preserved(&f, &line, &f.boxed);
    let stats = f.system.stats();
    let replay = old.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(f.system.stats(), stats);
    preserved(&f, &replay, &f.contribution);
    preserved(&f, &old, &f.contribution);
}

// Input-buffer seed limits include common census, cloning and finalization work.
// These assertions isolate retained request rows/backings; an observed total
// minus one would not independently identify temporary execution-plan scratch.
