//! Physical signed execution and original-logical prefix receipts are distinct.
//! Mount as a child of text_layout::tests::inline_boundary_edges::parent_geometry.
use super::*;

const HUGE: f64 = 18_014_398_509_481_984.0; // 2^54, exactly representable.
// The following literals are independently derived f64 coordinates, not readback.
const AFTER_NEGATIVE_EDGE_AND_A: f64 = -18_014_398_509_481_974.0; // -2^54 + 10
const AFTER_POSITIVE_EDGE_AND_A: f64 = 18_014_398_509_481_992.0; // round(2^54 + 10)

fn purpose_advances(line: &text::FinalizedLine) -> [f64; 4] {
    let measurements = line.measurements();
    [
        measurements.fit().advance(),
        measurements.alignment().advance(),
        measurements.min_content_contribution().advance(),
        measurements.max_content_contribution().advance(),
    ]
}

fn flow_and_strip(
    line: &text::FinalizedLine,
    fixture: &Fixture,
    node: u32,
) -> ((f64, f64), (f64, f64)) {
    let occurrence = control(fixture, node);
    let slot = line
        .control_slots()
        .find(|slot| slot.control() == occurrence)
        .unwrap();
    assert_eq!(slot.level().get(), 0, "original Source levels stay LTR");
    (slot.flow_interval(), slot.strip_interval())
}

fn actual_line(opposed_parent: bool) -> (text::FinalizedLine, text::TextSeam) {
    // B is LTR; actual captured C is nested inside actual captured P.
    // P's host-resolved used basis can oppose B without changing Source levels.
    // Reuse the already owning opposing-parent capture and Native adapter.
    let fixture = Fixture::nested_parent("A", false, false, true);
    source_premises(&fixture, "A", false, true);

    // Same captured source, real registered Ahem and ordinary affirmative-zero
    // Native allocations qualify the simple advance10 before the signed case.
    let ordinary = compute::<f64>(&fixture, false, 64.0, true, false);
    let ordinary_lines = finalized_lines(&ordinary);
    assert_eq!(ordinary_lines.len(), 1);
    let prior = &ordinary_lines[0];
    assert_eq!(glyphs(prior), [(35, 0.0, 10.0)]);
    assert_eq!(prior.feedback().advance, 10.0);
    assert_eq!(purpose_advances(prior), [10.0; 4]);

    // Obtain a PUBLIC, actually admitted small BoxEdge request first. Its
    // checked owners/parents, participants, struts and source segment are real.
    // This small placement is safe; the large-coordinate experiment below
    // stops at FinalizedLine and deliberately does not create caret rectangles.
    let small_batch = compute::<f64>(&fixture, false, 64.0, true, true);
    let small_lines = finalized_lines(&small_batch);
    assert_eq!(small_lines.len(), 1);
    let seed = &small_lines[0];
    assert_eq!(glyphs(seed), [(35, 4.0, 10.0)]);
    assert_eq!(seed.feedback().advance, 20.0);

    let prepared = &fixture.binding.prepared;
    let owner = control(&fixture, 22).captured_control().unwrap().scope();
    let parent = owner.parent().unwrap();
    assert_eq!(
        parent,
        control(&fixture, 20).captured_control().unwrap().scope()
    );
    let child_context = prepared
        .inline_box_edge_context(
            text::InlineBoxSource::captured(owner),
            text::InlineBoxSource::captured(parent),
            if opposed_parent {
                text::LineEdgeSide::End
            } else {
                text::LineEdgeSide::Start
            },
        )
        .unwrap();

    // Keep the genuine checked request/family. Only the four component receipts
    // and, for the same-direction control, C's requested physical side change.
    // P stays an actual checked zero-component BoxEdge, not a guessed parent.
    let mut recipe = seed.selected_line().request().clone();
    for (node, margin) in [(20, 0.0), (22, HUGE), (23, -HUGE), (21, 0.0)] {
        let occurrence = control(&fixture, node);
        let row = recipe
            .controls
            .iter_mut()
            .find(|row| row.occurrence == occurrence)
            .unwrap();
        let admitted_context = row
            .boundary
            .as_ref()
            .unwrap()
            .box_context()
            .expect("small real BoxEdge compute issued every endpoint context")
            .clone();
        let context = if matches!(node, 22 | 23) {
            child_context.clone()
        } else {
            admitted_context
        };
        row.boundary = Some(text::InlineBoundaryAllocation::box_edge(
            context,
            text::InlineBoxEdgeComponents::from_f64(margin, 0.0, 0.0).unwrap(),
        ));
    }
    let selected = prepared.select(recipe).unwrap();

    // Rebind actual seed frame values to the NEW selection's own SelectedScope
    // handles. Never reuse old selected-Arc handles or fabricate Scope::None.
    // This fixture may have no active alignment rows; None follows only from
    // the actual empty selected scope inventory, not from an assumed shortcut.
    let scope_rows = selected
        .scopes()
        .map(|scope| {
            let captured = scope.captured_scope();
            let actual = seed
                .scope_allocations()
                .iter()
                .find(|frame| frame.scope == captured)
                .expect("same genuine source/metrics retain the admitted frame demand");
            text::ScopeAllocation {
                scope,
                displacement: actual.displacement,
            }
        })
        .collect::<Vec<_>>();
    let scopes = if scope_rows.is_empty() {
        text::ScopeAllocations::None
    } else {
        text::ScopeAllocations::Complete(scope_rows)
    };
    let line = selected
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes,
        })
        .unwrap();
    assert_eq!(line.prepared_text(), fixture.binding.prepared);
    assert_eq!(line.control_slots().count(), 4);
    assert_eq!(
        line.run_views()
            .flat_map(|run| run.glyphs().map(|glyph| glyph.range()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [text::Range::new(0, 1)]
    );
    // A is not whitespace/punctuation, spacing is unchanged, and there is no
    // conditional hanging. These actual observations justify equal raw-purpose
    // advance oracles rather than confusing fit exclusion with physical flow.
    assert!(
        line.edge_units()
            .all(|edge| edge.excluded_contribution() == 0.0)
    );

    let expected_physical = if opposed_parent { 10.0 } else { 8.0 };
    if opposed_parent {
        // Real physical order: Pstart0, Cend(-H), A10, Cstart(+H), Pend0.
        // (-H + 10) is exactly representable (the toward-zero spacing is2);
        // adding H therefore yields10. Original Source order instead yields8.
        assert_eq!(glyphs(&line), [(35, -HUGE, 10.0)]);
        assert_eq!(
            flow_and_strip(&line, &fixture, 23),
            ((0.0, -HUGE), (-HUGE, -HUGE))
        );
        assert_eq!(
            flow_and_strip(&line, &fixture, 22),
            (
                (AFTER_NEGATIVE_EDGE_AND_A, 10.0),
                (AFTER_NEGATIVE_EDGE_AND_A, AFTER_NEGATIVE_EDGE_AND_A),
            )
        );
    } else {
        // Same-direction control: Pstart0, Cstart(+H), A10, Cend(-H), Pend0.
        // At +H the spacing is4, so +10 rounds to +8; the physical result is8.
        assert_eq!(glyphs(&line), [(35, HUGE, 10.0)]);
        assert_eq!(
            flow_and_strip(&line, &fixture, 22),
            ((0.0, HUGE), (HUGE, HUGE))
        );
        assert_eq!(
            flow_and_strip(&line, &fixture, 23),
            (
                (AFTER_POSITIVE_EDGE_AND_A, 8.0),
                (AFTER_POSITIVE_EDGE_AND_A, AFTER_POSITIVE_EDGE_AND_A),
            )
        );
    }
    assert_eq!(
        flow_and_strip(&line, &fixture, 20),
        ((0.0, 0.0), (0.0, 0.0))
    );
    assert_eq!(
        flow_and_strip(&line, &fixture, 21),
        (
            (expected_physical, expected_physical),
            (expected_physical, expected_physical),
        )
    );

    // The model deliberately keeps prefix_at in ORIGINAL-LOGICAL order.
    // Complete logical sequence: Pstart0, Cstart H, A10, Cend -H, Pend0.
    // (H + 10) rounds to H+8; subtracting H gives the independent receipt8.
    let seam =
        text::TextSeam::AfterParticipant(text::PreparedParticipant::Control(control(&fixture, 21)));
    assert_eq!(line.prefix_at(&seam).unwrap().advance(), 8.0);
    let admission = line.measurements().margin_edge_fit();
    assert_eq!(
        admission.authority(),
        text::MarginEdgeFitAuthority::CssComposed
    );
    assert_eq!(admission.native_occupied_bounds(), None);
    assert_eq!(admission.advance(), expected_physical);
    assert_eq!(admission.required_inline_size(), expected_physical);

    // These geometry/provenance/prefix guards execute before either target RED.
    assert_eq!(glyphs(prior), [(35, 0.0, 10.0)]);
    assert_eq!(prior.feedback().advance, 10.0);
    (line, seam)
}

#[test]
fn opposed_parent_feedback_advance_uses_actual_signed_execution() {
    let (line, _) = actual_line(true);
    assert_eq!(
        line.feedback().advance,
        10.0,
        "feedback uses physical End,A,Start; only prefix_at retains logical8"
    );
}

#[test]
fn opposed_parent_raw_purpose_advances_use_actual_signed_execution() {
    let (line, _) = actual_line(true);
    assert_eq!(
        purpose_advances(&line),
        [10.0; 4],
        "fit/alignment/min/max raw measures use physical End,A,Start without exclusions"
    );
}

#[test]
fn same_direction_raw_flow_and_logical_prefix_both_retain_rounded_eight() {
    let (line, _) = actual_line(false);
    assert_eq!(line.feedback().advance, 8.0);
    assert_eq!(purpose_advances(&line), [8.0; 4]);
}

#[test]
fn replay_keeps_physical_flow_separate_from_the_frozen_logical_prefix() {
    let (line, seam) = actual_line(true);
    let replay = line.with_spacing(text::SpacingAllocations::None).unwrap();
    assert_eq!(replay.prefix_at(&seam).unwrap().advance(), 8.0);
    assert_eq!(glyphs(&replay), glyphs(&line));
    assert_eq!(replay.measurements().margin_edge_fit().advance(), 10.0);
    assert_eq!(replay.feedback().advance, 10.0);
    assert_eq!(purpose_advances(&replay), [10.0; 4]);
    assert_eq!(line.prefix_at(&seam).unwrap().advance(), 8.0);
}
