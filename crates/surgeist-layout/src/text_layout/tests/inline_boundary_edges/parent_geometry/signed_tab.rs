//! One canonical signed pen drives CSS TAB execution and original-logical receipts.
//! A fresh partial recipe executes its own grid without borrowing a closing margin.
use super::*;

fn execute<S: LayoutScalar>(fixture: &Fixture) -> CompletedLayoutBatchOf<u32, S> {
    let mut tree = native_tree::<S>(fixture, false, 40.0, false);
    // Real P parent is B LTR. Real C parent is P RTL; all OLD formatter fields
    // remain B. PStart m2/b2/p1=>F5. CEnd m-4/b1/p2=>F-1 on the LEFT.
    // CStart m-5/b2/p3=>F0 on the RIGHT, real5px strip. PEnd is explicit zero.
    for (node, [m, b, p]) in [
        (20, [2.0, 2.0, 1.0]),
        (23, [-4.0, 1.0, 2.0]),
        (22, [-5.0, 2.0, 3.0]),
        (21, [0.0, 0.0, 0.0]),
    ] {
        let old = tree.layout_input(node).as_inline_boundary().unwrap();
        let edge =
            InlineBoundaryEdgeOf::try_new(S::from_f64(m), S::from_f64(b), S::from_f64(p)).unwrap();
        let input = InlineBoundaryInputOf::box_edge(
            old.kind(),
            old.metrics(),
            edge,
            axes(matches!(node, 22 | 23)),
        )
        .with_writing_mode(old.writing_mode())
        .with_direction(old.direction())
        .with_vertical_align(old.vertical_align());
        tree = tree.input(node, LayoutInputOf::inline_boundary(input));
    }
    let input = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        wrapping(fixture, true),
        Vec::new(),
    )
    .unwrap();
    compute_layout_with_flow(&input, 0, request(40.0)).unwrap()
}
fn interval(line: &text::FinalizedLine, fixture: &Fixture, node: u32) -> ((f64, f64), (f64, f64)) {
    let occurrence = control(fixture, node);
    let slot = line
        .control_slots()
        .find(|slot| slot.control() == occurrence)
        .unwrap();
    (slot.flow_interval(), slot.strip_interval())
}
fn lane<S: LayoutScalar>() {
    // A real qualification of DIGIT ZERO advance10 in this SAME pinned Ahem
    // at size10: absolute period20 has half-ch threshold5, not a TAB glyph.
    let digit = Fixture::new("0", (0, 1), text::WhiteSpace::Preserve, false, false);
    let digit_layout = digit.compute(
        InlineBoundaryEdgeOf::ZERO,
        InlineBoundaryEdgeOf::ZERO,
        false,
        Some(10.0),
        false,
        AvailableOf::definite(S::from_f64(10.0)),
    );
    let digit_line = finalized_lines(&digit_layout).remove(0);
    assert_eq!(glyphs(&digit_line), [(18, 0.0, 10.0)]);

    let fixture = Fixture::nested_parent("A\tB", false, true, true);
    source_premises(&fixture, "A\tB", false, true);
    let ordinary = compute::<S>(&fixture, false, 30.0, true, false);
    let prior = finalized_lines(&ordinary).remove(0);
    assert_eq!(glyphs(&prior), [(35, 0.0, 10.0), (36, 20.0, 10.0)]);
    assert_eq!(prior.feedback().advance, 30.0);
    let origin = fixture.binding.origins[0].origin.clone();
    let seam = text::TextSeam::Origin {
        origin: origin.clone(),
        local: text::SourcePosition::try_new(origin.view().source().text(), 2).unwrap(),
    };
    assert_eq!(prior.prefix_at(&seam).unwrap().advance(), 20.0);

    let batch = execute::<S>(&fixture);
    let full = finalized_lines(&batch).remove(0);
    // CANONICAL physical execution: Ps F5, Ce F-1 ->4, A10->14,
    // TAB6->20 (6>=threshold5), B10->30, Cs F0, Pe0. No recomposed grid.
    assert_eq!(full.feedback().advance, 30.0);
    assert_eq!(
        full.feedback().occupied_end,
        35.0,
        "Cs has a real strip30..35 despite net flow0"
    );
    assert_eq!(glyphs(&full), [(35, 4.0, 10.0), (36, 20.0, 10.0)]);
    assert_eq!(interval(&full, &fixture, 20), ((0.0, 5.0), (2.0, 5.0)));
    assert_eq!(interval(&full, &fixture, 23), ((5.0, 4.0), (1.0, 4.0)));
    assert_eq!(interval(&full, &fixture, 22), ((30.0, 30.0), (30.0, 35.0)));
    assert_eq!(interval(&full, &fixture, 21), ((30.0, 30.0), (30.0, 30.0)));
    assert_eq!(
        full.measurements().margin_edge_fit().required_inline_size(),
        30.0
    );
    assert_eq!(
        full.prefix_at(&text::TextSeam::AfterParticipant(
            text::PreparedParticipant::Control(control(&fixture, 22))
        ))
        .unwrap()
        .advance(),
        5.0
    );
    // ORIGINAL source order Ps5,Cs0,A10,TAB6,B10,Ce-1,Pe0. This is the
    // authentic frozen full receipt21, NOT physical TAB end20 or a re-execution.
    assert_eq!(full.prefix_at(&seam).unwrap().advance(), 21.0);

    let prepared = &fixture.binding.prepared;
    let offer = prepared
        .opportunity_views()
        .find(|offer| {
            offer.kind() == text::PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 2
                && offer.boundary().view().source_after().get() == 2
        })
        .unwrap()
        .handle();
    let mut partial = full.selected_line().request().clone();
    partial.end = offer.view().boundary();
    partial.termination = text::PreparedTermination::Ordinary(offer);
    partial.context.end = text::LineEnd::Soft;
    partial.participants = prepared
        .participants()
        .take(partial.end.view().participant_offset())
        .collect();
    partial.controls.retain(|row| {
        partial.participants.iter().any(|participant|
        matches!(participant,text::PreparedParticipant::Control(c) if c==&row.occurrence))
    });
    partial.boundary_context.clear();
    assert_eq!(partial.controls.len(), 2);
    while let Some(needed) = prepared.boundary_context_demand(&partial).unwrap() {
        assert!(
            !partial
                .boundary_context
                .iter()
                .any(|r| r.occurrence == needed)
        );
        let allocation = full
            .selected_line()
            .request()
            .controls
            .iter()
            .find(|r| r.occurrence == needed)
            .unwrap()
            .boundary
            .clone()
            .unwrap();
        partial
            .boundary_context
            .push(text::InlineBoundaryContextAllocation {
                occurrence: needed,
                boundary: allocation,
            });
    }
    let selected = prepared.select(partial).unwrap();
    let partial = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap();
    // New recipe executes Ps5,A10->15,TAB5->20, then selected Cs F0. Gap5
    // EQUALS the real half-ch threshold, so it does not bump to the40px stop.
    // Offselection Ce F-1 supplies shaping facts but contributes NO flow.
    assert_eq!(partial.feedback().advance, 20.0);
    assert_eq!(glyphs(&partial), [(35, 5.0, 10.0)]);
    assert_eq!(partial.control_slots().count(), 2);
    assert_eq!(
        interval(&partial, &fixture, 22),
        ((20.0, 20.0), (20.0, 25.0))
    );
    assert_eq!(partial.prefix_at(&seam).unwrap().advance(), 20.0);
    assert_eq!(full.prefix_at(&seam).unwrap().advance(), 21.0);
    let replay = full.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.prefix_at(&seam).unwrap().advance(), 21.0);
    assert_eq!(glyphs(&replay), glyphs(&full));
    for node in [20, 22, 23, 21] {
        assert_eq!(
            interval(&replay, &fixture, node),
            interval(&full, &fixture, node)
        );
    }
    assert_eq!(prior.prefix_at(&seam).unwrap().advance(), 20.0);
    assert_eq!(glyphs(&prior), [(35, 0.0, 10.0), (36, 20.0, 10.0)]);
}
#[test]
fn signed_canonical_pen_drives_real_tab_frozen_logical_prefix_and_new_partial_recipe() {
    lane::<f32>();
    lane::<f64>();
}
// Scope: actual nominated nested owners, Htb/LTR formatter and actual parent
// opposition, both concrete Layout lanes, real CSS TAB threshold, raw strip/flow,
// original-logical receipts, partial/no borrowed end margin, query-free replay.
