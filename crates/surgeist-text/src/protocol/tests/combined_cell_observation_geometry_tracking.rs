//! Actual executed outer Cell advances, independently of the fixed square or
//! rounded endpoint subtraction. Child of the genuine tracking numeric fixture.

use super::*;

fn executed(size: f32, tracking: f64, indent: f64) -> (PreparedText, FinalizedLine) {
    let (mut system, input) = fixture("AA", tracking, false, true, size);
    assert_eq!(input.processed.source().text(), &Text::from("AAB"));
    let scope = input.processed.captured_scope(1).unwrap();
    assert_eq!(scope.style().unwrap().size, size);
    assert_eq!(scope.style().unwrap().letter_spacing, tracking);
    assert_eq!(scope.style().unwrap().word_spacing, 0.0);
    let prepared = system
        .prepare(
            &input.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let cell = prepared.cell_views().next().unwrap();
    assert_eq!(cell.extent(), f64::from(size));
    assert_eq!(cell.source_cell().scope(), scope);
    let line = selected_at_indent(&prepared, indent)
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    // The existing real font qualifies hwid A35->B36 with hmtx500. Two
    // actual half-size glyphs fit E; outer tracking never changes this packet.
    let inner = line
        .run_views()
        .filter(|run| run.cell().is_some())
        .flat_map(|run| {
            assert!(run.face().is_same_face(&input.body.face()));
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(inner, [(36, size / 2.0), (36, size / 2.0)]);
    (prepared, line)
}

fn cell_participant(prepared: &PreparedText) -> PreparedParticipant {
    let cells = prepared
        .participants()
        .filter(|participant| matches!(participant, PreparedParticipant::Cell(_)))
        .collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual Cell participant beside ordinary B");
    };
    cell.clone()
}

#[test]
fn cell_edge_advance_keeps_the_executed_value_when_endpoint_subtraction_rounds_differently() {
    // At C=2^52, E2.5 + mean(LS2,LS0)=1 executes advance3.5.
    // The stored complete endpoint is C+4; subtracting C reconstructs4,
    // while using the selected natural Cell would instead report only2.5.
    assert_eq!(C + 2.5, C + 2.0);
    assert_eq!(C + 3.5, C + 4.0);
    assert_eq!((C + 3.5) - C, 4.0);
    let (prepared, line) = executed(2.5, 2.0, C);
    let participant = cell_participant(&prepared);
    let edges = line
        .edge_units()
        .filter(|edge| {
            edge.owners()
                .any(|owner| owner == LineEdgeOwner::Participant(participant.clone()))
        })
        .collect::<Vec<_>>();
    let [edge] = edges.as_slice() else {
        panic!("one authentic outer Cell edge");
    };
    assert_eq!(edge.advance(), 3.5);
    assert_ne!(edge.advance(), 2.5);
    // Allocation width retains the same executed scalar. Adding3.5 to C
    // rounds its endpoint to C+4, without changing the stored width to4.
    let band = line.metric_bounds();
    assert_eq!(
        edge.bounds(),
        Rect::new(C, band.origin.y, 3.5, band.size.height)
    );
    assert_eq!(
        edge.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
}

#[test]
fn negative_executed_cell_flow_remains_signed_while_its_square_stays_one_em() {
    // Cell E10 + mean(LS-24,following LS0)=-2. The actual fixed square
    // remains0..10, but the allocated flow interval is0..-2, with a2-wide hull.
    let (prepared, line) = executed(10.0, -24.0, 0.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine signed-flow Cell");
    };
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let participant = cell_participant(&prepared);
    let edges = line
        .edge_units()
        .filter(|edge| {
            edge.owners()
                .any(|owner| owner == LineEdgeOwner::Participant(participant.clone()))
        })
        .collect::<Vec<_>>();
    let [edge] = edges.as_slice() else {
        panic!("one authentic outer Cell edge");
    };
    assert_eq!(edge.advance(), -2.0);
    let band = line.metric_bounds();
    assert_eq!(
        edge.bounds(),
        Rect::new(-2.0, band.origin.y, 2.0, band.size.height)
    );
    assert_eq!(
        edge.source_pieces().collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2)]
    );
}
