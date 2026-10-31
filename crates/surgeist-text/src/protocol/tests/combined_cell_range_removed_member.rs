//! Removed interior source retains its authenticated marked-EGC closure.
use super::*;
use crate::protocol::prepared::SelectedRelationEvent;
use crate::transform::relation::{ComponentOwner, RelationEvent};

#[test]
fn an_actual_removed_forced_original_selects_itself_and_closes_the_retained_marked_egc() {
    // Reuse the maintained genuine three-origin Preserve/All capture, not a
    // synthetic removal, source-less ForcedBreak control, or flattened adjacency.
    let captured = capture("A\n\u{301}");
    assert_eq!(
        captured.processed.source().text(),
        &Text::from("A\n\u{301}")
    );
    assert_eq!(
        captured.processed.processed_text(),
        &Text::from("A\n\u{301}")
    );
    let cells = captured.analyzed.combined_cells().collect::<Vec<_>>();
    let [source_cell] = cells.as_slice() else {
        panic!("real uninterrupted All admits one source Cell");
    };
    assert_eq!(
        source_cell.scope(),
        captured.processed.captured_scope(1).unwrap()
    );
    assert_eq!(source_cell.mode(), TextCombineUpright::All);
    // GB4/GB5 break on each side of real LF before use; GB9 attaches acute to
    // A after LF is removed. This is not the already-covered ordinary A+mark case.
    assert_eq!(source_cell.selection_unit_count(), 3);
    assert_eq!(source_cell.used_unit_count(), 1);
    assert_eq!(
        source_cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x301]
    );
    assert_eq!(
        source_cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x301]
    );
    let originals = source_cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(originals.len(), 3);
    for (index, piece) in originals.iter().enumerate() {
        assert_eq!(piece.source(), captured.processed.source());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            captured.processed.captured_origin(index).unwrap()
        );
        assert_eq!(
            piece.paragraph(),
            captured.processed.captured_paragraph(0).unwrap()
        );
        assert_eq!(piece.is_used(), index != 1);
        assert_eq!(piece.has_presentation_emission(), index != 1);
    }
    assert!(captured.processed.captured_control(0).is_none());
    assert!(captured.processed.captured_origin(3).is_none());

    // Existing maintained mark fixture; exact same cmap/mark tables, raw font
    // metrics and outlines as the already qualified genuine A+acute Source case.
    let original_font = crate::shaping_test_fonts::mark_ahem();
    let bytes = native_metric_font(&original_font);
    for tag in [
        *b"cmap", *b"GDEF", *b"GPOS", *b"glyf", *b"loca", *b"hhea", *b"hmtx",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original_font, tag));
    }
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(face.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    assert!(face.charmap().map('\u{c1}').is_none());
    actual_provider(&bytes, "A\u{301}", &[(35, 1000, 0, 0), (100, 0, -700, 400)]);
    let (prepared, layout) = prepared_layout(&captured, &bytes);
    let origins = [
        issued(&prepared, &captured.processed, 0, Range::new(0, 1)),
        issued(&prepared, &captured.processed, 1, Range::new(1, 2)),
        issued(&prepared, &captured.processed, 2, Range::new(2, 3)),
    ];
    assert!(origins.windows(2).all(|pair| pair[0] != pair[1]));

    // Observe authentic SELECTED input facts, not the Range projection or a
    // glyph-source hull. The actual LF Removal must lie strictly between the
    // actual retained A and acute events in the selected Cell's own event range.
    let native = &layout.inner.lines[0].finalized;
    let [actual_cell] = native.data.cells.as_slice() else {
        panic!("one actual finalized Cell");
    };
    let selected = &native.selected;
    let relation = &selected.presentation.selected_relation;
    let map = &selected.prepared.analyzed.transform;
    let mut facts = Vec::new();
    for event_index in relation.unit_events[actual_cell.unit].clone() {
        let SelectedRelationEvent::Base { event } = relation.events[event_index] else {
            panic!("this fixture has only real original Source relation events");
        };
        match map.events()[event] {
            RelationEvent::Text { atom } => {
                assert!(matches!(
                    map.component_for_atom(atom).owner,
                    ComponentOwner::Original(_)
                ));
                facts.push((
                    event_index,
                    true,
                    map.emission_for_atom(atom).code_point,
                    map.component_for_atom(atom).source,
                ));
            }
            RelationEvent::Removal {
                row,
                selection_part: Some(_),
            } => {
                facts.push((
                    event_index,
                    false,
                    map.rows[row].code_point,
                    map.rows[row].source,
                ));
            }
            _ => (),
        }
    }
    assert_eq!(
        facts
            .iter()
            .map(|(_, retained, cp, source)| (*retained, *cp, *source))
            .collect::<Vec<_>>(),
        [
            (true, 0x41, Range::new(0, 1)),
            (false, 0x0a, Range::new(1, 2)),
            (true, 0x301, Range::new(2, 3)),
        ]
    );
    assert!(facts[0].0 < facts[1].0 && facts[1].0 < facts[2].0);
    let raw = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(raw, [(35, 10.0), (100, 0.0)]);

    // All premise checks above precede the operation under qualification.
    // The query selects ONLY the genuinely removed original; it must not return
    // a Removed-only pseudo-unit or borrow a nearest neighboring EGC by cut.
    let removed = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    let base = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let mark = layout
        .source_range_fragments(&origins[2], Range::new(2, 3))
        .unwrap();
    assert_eq!(
        (
            removed.units().len(),
            base.units().len(),
            mark.units().len()
        ),
        (1, 1, 1)
    );
    let units = [
        base.units().next().unwrap(),
        removed.units().next().unwrap(),
        mark.units().next().unwrap(),
    ];
    assert!(units[1].same_unit(&units[0]));
    assert!(units[1].same_unit(&units[2]));
    assert!(
        units[1]
            .cell_unit()
            .unwrap()
            .same_unit(&units[0].cell_unit().unwrap())
    );
    assert!(
        units[1]
            .cell_unit()
            .unwrap()
            .same_unit(&units[2].cell_unit().unwrap())
    );
    for (selected, unit) in units.iter().copied().enumerate() {
        let owner = unit.cell_unit().unwrap();
        assert_eq!(
            owner.kind(),
            SourceRangeCellUnitKind::Semantic { ordinal: 0 }
        );
        assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
        assert_eq!(
            (
                owner.source_cell().selection_unit_count(),
                owner.source_cell().used_unit_count()
            ),
            (3, 1)
        );
        assert_eq!(
            (
                unit.presentation_range().start(),
                unit.presentation_range().end()
            ),
            (0, 2)
        );
        assert_eq!(
            unit.selected_intersections()
                .map(|piece| (piece.origin(), piece.local_range(), piece.aggregate_range()))
                .collect::<Vec<_>>(),
            [(
                origins[selected].clone(),
                Range::new(selected, selected + 1),
                Range::new(selected, selected + 1)
            )]
        );
        let expanded = unit.expanded_intersections().collect::<Vec<_>>();
        assert_eq!(expanded.len(), 3);
        // Closure, not an incidental collector insertion order, is this oracle.
        for (index, origin) in origins.iter().enumerate() {
            let matching = expanded
                .iter()
                .filter(|piece| piece.origin() == *origin)
                .collect::<Vec<_>>();
            assert_eq!(matching.len(), 1);
            assert_eq!(matching[0].local_range(), Range::new(index, index + 1));
            assert_eq!(matching[0].aggregate_range(), Range::new(index, index + 1));
        }
        assert_eq!(unit.controls().len(), 0);
        assert_eq!(unit.virtual_separators().len(), 0);
        assert!(unit.pieces().len() > 0);
        // No provider-cluster partition assumption: all actual pieces must use
        // the same real font/reference; their independently derived metric hull
        // is A advance10 plus attached mark0, H8/+2 at baseline(95,28).
        let mut hull: Option<(f64, f64, f64, f64)> = None;
        for geometry in unit.pieces() {
            let SourceRangeGeometry::CellFont(piece) = geometry else {
                panic!("retained marked EGC is font-backed");
            };
            assert!(piece.cell_unit().same_unit(&owner));
            assert_eq!(piece.effective_instance().face().bytes(), bytes.as_slice());
            assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
            assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
            assert_eq!(piece.metric_frame().x_scale(), 1.0);
            assert!(piece.inner_cluster_indices().len() > 0);
            let rect = piece.rect();
            let corners = (
                rect.origin.x,
                rect.origin.y,
                rect.origin.x + rect.size.width,
                rect.origin.y + rect.size.height,
            );
            hull = Some(match hull {
                None => corners,
                Some((x0, y0, x1, y1)) => (
                    x0.min(corners.0),
                    y0.min(corners.1),
                    x1.max(corners.2),
                    y1.max(corners.3),
                ),
            });
        }
        assert_eq!(hull, Some((95.0, 20.0, 105.0, 30.0)));
    }
    assert_eq!(rectangles(&removed), rectangles(&base));
    assert_eq!(rectangles(&removed), rectangles(&mark));
}
