//! Range geometry follows the requested authentic Cell occurrence,
//! while ordinary source in the very same placed layout stays observable.

use super::*;

fn actual_origin(
    prepared: &PreparedText,
    processed: &ProcessedSource,
    occurrence: usize,
) -> PreparedOrigin {
    let capture = processed.captured_origin(occurrence).unwrap();
    prepared
        .origin_views()
        .find(|origin| origin.capture_position().as_ref() == Some(&capture))
        .expect("actual mixed capture occurrence has an issued prepared origin")
        .handle()
}

fn ordinary(layout: &Layout, origin: &PreparedOrigin, fixture: &Fixture) {
    let local = origin.view().local_range();
    let capture = origin.view().capture_position().unwrap();
    let fragments = layout
        .source_range_fragments(origin, local)
        .expect("ordinary nonempty B source stays observable alongside a Cell");
    assert_eq!(fragments.local_range(), local);
    assert_eq!(
        fragments.origin().view().capture_position(),
        Some(capture.clone())
    );
    assert_eq!(
        fragments.origin().view().source(),
        fixture.processed.source()
    );
    let units = fragments.units().collect::<Vec<_>>();
    assert!(!units.is_empty());
    let mut selected = Vec::new();
    let mut pieces = 0;
    for unit in units {
        for intersection in unit.selected_intersections() {
            assert_eq!(
                intersection.origin().view().capture_position(),
                Some(capture.clone())
            );
            assert_eq!(
                intersection.origin().view().source(),
                fixture.processed.source()
            );
            selected.push((intersection.local_range(), intersection.aggregate_range()));
        }
        for piece in unit.pieces() {
            let crate::SourceRangeGeometry::Line(piece) = piece else {
                panic!("ordinary Range fixture must retain Line geometry");
            };
            let instance = piece.effective_instance();
            assert!(instance.face().is_same_face(&fixture.parent.face()));
            assert_eq!(instance.font_size(), 10.0);
            assert!(piece.rect().size.width > 0.0 && piece.rect().size.height > 0.0);
            pieces += 1;
        }
    }
    assert_eq!(selected, [(local, local)]);
    assert!(pieces > 0);
}

#[test]
fn cell_and_ordinary_range_geometry_keep_distinct_frames_in_the_same_layout() {
    // This genuine fixture emits one leading B and one trailing B: BAB,
    // inner1..2. It creates a separate real origin for each source scalar,
    // with genuine Continuous connections and Inline enter/exit around A only.
    let (mut system, fixture) = fixture("A", 0.0, true, true, 10.0);
    assert_eq!(fixture.processed.source().text(), &Text::from("BAB"));
    assert_eq!(fixture.inner, Range::new(1, 2));
    let source_cell = fixture.analyzed.combined_cells().next().unwrap();
    assert_eq!(
        source_cell.scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    let originals = source_cell.source_pieces().collect::<Vec<_>>();
    let [original] = originals.as_slice() else {
        panic!("one genuine A occurrence");
    };
    assert_eq!(
        original.origin_position(),
        fixture.processed.captured_origin(1).unwrap()
    );
    assert_eq!(original.local(), Range::new(1, 2));
    assert!(original.is_used() && original.has_presentation_emission());
    let prepared = system
        .prepare(
            &fixture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.origin_views().len(), 3);
    let before = actual_origin(&prepared, &fixture.processed, 0);
    let inside = actual_origin(&prepared, &fixture.processed, 1);
    let after = actual_origin(&prepared, &fixture.processed, 2);
    assert_eq!(before.view().local_range(), Range::new(0, 1));
    assert_eq!(inside.view().local_range(), Range::new(1, 2));
    assert_eq!(after.view().local_range(), Range::new(2, 3));
    let prepared_cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = prepared_cells.as_slice() else {
        panic!("one actual Cell beside ordinary B text");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    let line = finalize(&prepared);
    assert_eq!(line.feedback().advance, 30.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual finalized square");
    };
    assert_eq!(cell.square().origin.x, 10.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    let mut inner = Vec::new();
    let mut outside = Vec::new();
    for run in line.run_views() {
        if run.cell().is_some() {
            assert!(run.face().is_same_face(&fixture.body.face()));
            inner.extend(run.glyphs().map(|glyph| (glyph.id(), glyph.advance())));
        } else {
            assert!(run.face().is_same_face(&fixture.parent.face()));
            outside.extend(run.glyphs().map(|glyph| (glyph.id(), glyph.advance())));
        }
    }
    assert_eq!(inner, [(35, 10.0)]);
    assert_eq!(outside, [(36, 10.0), (36, 10.0)]);
    drop(system);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), fixture.processed.source());
    ordinary(&layout, &before, &fixture);
    let range = layout
        .source_range_fragments(&inside, Range::new(1, 2))
        .unwrap();
    assert_eq!(range.origin(), &inside);
    assert_eq!(range.local_range(), Range::new(1, 2));
    let units = range.units().collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("one actual A EGC alongside ordinary B text");
    };
    let cell_unit = unit.cell_unit().unwrap();
    assert_eq!((cell_unit.line_index(), cell_unit.cell_index()), (0, 0));
    assert_eq!(
        cell_unit.kind(),
        SourceRangeCellUnitKind::Semantic { ordinal: 0 }
    );
    assert_eq!(
        cell_unit.source_cell().scope(),
        fixture.processed.captured_scope(1).unwrap()
    );
    for intersections in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        let [piece] = intersections.as_slice() else {
            panic!("one source-owned A contribution");
        };
        assert_eq!(piece.origin(), inside);
        assert_eq!(piece.local_range(), Range::new(1, 2));
        assert_eq!(piece.aggregate_range(), Range::new(1, 2));
    }
    let pieces = unit.pieces().collect::<Vec<_>>();
    let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
        panic!("A retains its own Cell font frame");
    };
    assert!(piece.cell_unit().same_unit(&cell_unit));
    assert!(
        piece
            .effective_instance()
            .face()
            .is_same_face(&fixture.body.face())
    );
    assert_eq!(piece.effective_instance().font_size(), 10.0);
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    assert_eq!(piece.inner_advance_edges(), (0.0, 10.0));
    assert_eq!(piece.rect(), Rect::new(95.0, 30.0, 10.0, 10.0));
    ordinary(&layout, &after, &fixture);
    ordinary(&layout, &before, &fixture);
}
