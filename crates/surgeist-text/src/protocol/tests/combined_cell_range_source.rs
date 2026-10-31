//! Exact original-source selection and semantic Cell closure.
#[cfg(feature = "text-accessibility")]
#[path = "combined_cell_ax_source.rs"]
mod accessibility;

use super::*;
use crate::protocol::tests::combined_cells::inner_tabs as tabs;
use crate::{SourceRangeCellUnitKind, SourceRangeGeometry, SourceRangeUnit};
use skrifa::{GlyphId, MetadataProvider};
use surgeist_shaper::{
    Direction as ShapeDirection, FontRef as ShapeFont, ShaperData, ShapingStatus, UnicodeBuffer,
    script,
};

fn actual_provider(bytes: &[u8], text: &str, expected: &[(u32, i32, i32, i32)]) {
    glyph_frame_font_premises(bytes);
    let font = ShapeFont::new(bytes).unwrap();
    let data = ShaperData::new(&font);
    let shaper = data.shaper(&font).build();
    assert_eq!(shaper.units_per_em(), 1000);
    let mut input = UnicodeBuffer::new();
    for (index, ch) in text.chars().enumerate() {
        input.add(ch, u32::try_from(index).unwrap());
    }
    input.set_direction(ShapeDirection::LeftToRight);
    input.set_script(script::LATIN);
    input.set_language("en".parse().unwrap());
    let output = shaper.shape(input, &[]);
    assert_eq!(output.shaping_status(), ShapingStatus::Complete);
    assert!(output.width_feature_receipt().unwrap().is_none());
    assert_eq!(
        output
            .glyph_infos()
            .iter()
            .zip(output.glyph_positions())
            .map(|(info, pos)| {
                assert_eq!(pos.y_advance, 0);
                (info.glyph_id, pos.x_advance, pos.x_offset, pos.y_offset)
            })
            .collect::<Vec<_>>(),
        expected
    );
}

fn capture(text: &str) -> tabs::Captured {
    tabs::captured(
        text,
        TabSize::spaces(2.0).unwrap(),
        10.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        "Ahem",
    )
}

fn prepared_layout(capture: &tabs::Captured, bytes: &[u8]) -> (PreparedText, Layout) {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let prepared = system
        .prepare(
            &capture.analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().identity(),
        capture.analyzed.identity()
    );
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual native source Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    let line = complete_line(&prepared);
    one_em(&line);
    let layout = placed_vertical_cell(&prepared, line);
    assert_eq!(layout.source(), capture.processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let fonts = &layout.inner.lines[0].finalized.data.font_runs;
    assert!(!fonts.is_empty());
    for run in fonts {
        // Actual retained b0, not inferred from rectangle/glyph equality.
        // These fixtures therefore have coincident metric-reference and
        // font-design-origin Y, while their public frame types stay distinct.
        assert_eq!(run.metrics.font_origin_offset(), 0.0);
    }
    (prepared, layout)
}

fn issued(
    prepared: &PreparedText,
    processed: &ProcessedSource,
    index: usize,
    range: Range,
) -> PreparedOrigin {
    let captured = processed.captured_origin(index).unwrap();
    let view = prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&captured))
        .unwrap();
    assert_eq!(view.source(), processed.source());
    assert_eq!(view.local_range(), range);
    assert_eq!(view.aggregate_range(), range);
    view.handle()
}

fn source_before_query(capture: &tabs::Captured, points: &[u32]) {
    let cell = capture.analyzed.combined_cells().next().unwrap();
    assert_eq!(capture.analyzed.combined_cells().len(), 1);
    assert_eq!(cell.scope(), capture.processed.captured_scope(1).unwrap());
    assert_eq!(cell.selection_unit_count(), 1);
    assert_eq!(cell.used_unit_count(), 1);
    assert_eq!(cell.used_code_points().collect::<Vec<_>>(), points);
    // These are real two issued original occurrences, NOT one broadened
    // original. Their same Source and same Inline do not merge their identity.
    assert_eq!(
        cell.source_pieces()
            .map(|piece| {
                assert!(piece.is_used());
                assert!(piece.has_presentation_emission());
                assert_eq!(piece.source(), capture.processed.source());
                (piece.origin_position(), piece.local(), piece.aggregate())
            })
            .collect::<Vec<_>>(),
        [
            (
                capture.processed.captured_origin(0).unwrap(),
                Range::new(0, 1),
                Range::new(0, 1)
            ),
            (
                capture.processed.captured_origin(1).unwrap(),
                Range::new(1, 2),
                Range::new(1, 2)
            ),
        ]
    );
}

fn closure(unit: SourceRangeUnit<'_>, origins: &[PreparedOrigin; 2], selected: usize) {
    let range = Range::new(selected, selected + 1);
    assert_eq!(
        unit.selected_intersections()
            .map(|piece| (piece.origin(), piece.local_range(), piece.aggregate_range()))
            .collect::<Vec<_>>(),
        [(origins[selected].clone(), range, range)]
    );
    assert_eq!(
        unit.expanded_intersections()
            .map(|piece| (piece.origin(), piece.local_range(), piece.aggregate_range()))
            .collect::<Vec<_>>(),
        [
            (origins[0].clone(), Range::new(0, 1), Range::new(0, 1)),
            (origins[1].clone(), Range::new(1, 2), Range::new(1, 2)),
        ]
    );
    let owner = unit.cell_unit().unwrap();
    assert_eq!(
        owner.kind(),
        SourceRangeCellUnitKind::Semantic { ordinal: 0 }
    );
    assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
    assert_eq!(owner.source_cell().used_unit_count(), 1);
    assert_eq!(owner.presentation_range(), unit.presentation_range());
}

#[test]
fn each_genuine_surrogate_origin_selects_only_itself_and_closes_the_same_actual_scalar() {
    // Maintained source_custody fixture maps this real scalar to A35. The
    // native table wrapper is the same existing Cell metric owner, not a fake
    // provider or a claim that original Ahem covers arbitrary supplementary text.
    let bytes = crate::mixed_test_fonts::with_cmap(&parent_font(), &[(0x41, 35), (0x1f600, 35)]);
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('\u{1f600}'), Some(GlyphId::new(35)));
    actual_provider(&bytes, "\u{1f600}", &[(35, 1000, 0, 0)]);
    let capture = capture("\u{1f600}");
    assert_eq!(
        capture.processed.source().text().code_units(),
        [0xd83d, 0xde00]
    );
    source_before_query(&capture, &[0x1f600]);
    let (prepared, layout) = prepared_layout(&capture, &bytes);
    let origins = [
        issued(&prepared, &capture.processed, 0, Range::new(0, 1)),
        issued(&prepared, &capture.processed, 1, Range::new(1, 2)),
    ];
    assert_ne!(origins[0], origins[1]);
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("one actual scalar font packet");
    };
    assert_eq!(packet.font().bytes(), bytes.as_slice());
    assert_eq!(
        packet
            .glyphs()
            .iter()
            .map(|glyph| (glyph.id(), glyph.range(), glyph.pen(), glyph.advance()))
            .collect::<Vec<_>>(),
        [(35, Range::new(0, 2), 0.0, 10.0)]
    );
    let frame = packet.glyph_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 28.0));
    assert_eq!(frame.outline_x(), Point::new(1.0, 0.0));
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    // Query only each origin's actual issued interval; NEVER0..2 on either.
    let first = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let second = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    assert_eq!((first.units().count(), second.units().count()), (1, 1));
    let a = first.units().next().unwrap();
    let b = second.units().next().unwrap();
    closure(a, &origins, 0);
    closure(b, &origins, 1);
    assert!(a.same_unit(&b));
    assert!(a.cell_unit().unwrap().same_unit(&b.cell_unit().unwrap()));
    // Real normal glyph35 is10 wide; H8/-2 through the qualified frame gives
    // this metric rectangle. No half-surrogate rectangle or outer-band proxy.
    for unit in [a, b] {
        let pieces = unit.pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
            panic!("actual scalar font piece");
        };
        assert_eq!(piece.inner_advance_edges(), (0.0, 10.0));
        assert_eq!(piece.metric_frame().origin(), Point::new(95.0, 28.0));
        assert_eq!(piece.metric_frame().x_scale(), 1.0);
        assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        assert_eq!(piece.effective_instance().face().bytes(), bytes.as_slice());
        assert_eq!(piece.rect(), Rect::new(95.0, 20.0, 10.0, 10.0));
    }
}

#[test]
fn an_actual_gpos_attached_mark_and_base_keep_exact_selection_but_share_egc_closure() {
    let original = crate::shaping_test_fonts::mark_ahem();
    let bytes = native_metric_font(&original);
    for tag in [
        *b"cmap", *b"GDEF", *b"GPOS", *b"glyf", *b"loca", *b"hhea", *b"hmtx",
    ] {
        assert_eq!(sfnt::table(&bytes, tag), sfnt::table(&original, tag));
    }
    let font = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(font.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(font.charmap().map('\u{301}'), Some(GlyphId::new(100)));
    assert!(font.charmap().map('\u{c1}').is_none());
    // Maintained mark_ahem real GPOS: base anchor400/600, mark100/200.
    // With pen1000 the native mark offset-700/+400 gives logical(3,-4),
    // advance0. No absent-font/tofu or mark ink bound is assumed.
    actual_provider(&bytes, "A\u{301}", &[(35, 1000, 0, 0), (100, 0, -700, 400)]);
    let capture = capture("A\u{301}");
    source_before_query(&capture, &[0x41, 0x301]);
    let (prepared, layout) = prepared_layout(&capture, &bytes);
    let origins = [
        issued(&prepared, &capture.processed, 0, Range::new(0, 1)),
        issued(&prepared, &capture.processed, 1, Range::new(1, 2)),
    ];
    let glyphs = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.glyph_frame().outline_x(), Point::new(1.0, 0.0));
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(glyphs, [(35, 10.0), (100, 0.0)]);
    let base = layout
        .source_range_fragments(&origins[0], Range::new(0, 1))
        .unwrap();
    let mark = layout
        .source_range_fragments(&origins[1], Range::new(1, 2))
        .unwrap();
    assert_eq!((base.units().count(), mark.units().count()), (1, 1));
    let a = base.units().next().unwrap();
    let b = mark.units().next().unwrap();
    closure(a, &origins, 0);
    closure(b, &origins, 1);
    assert!(a.same_unit(&b));
    assert!(a.cell_unit().unwrap().same_unit(&b.cell_unit().unwrap()));
    // Same real EGC geometry regardless of which actual source member is
    // selected. Do not prescribe a provider-cluster partition or mark ink box.
    assert_eq!(rectangles(&base), rectangles(&mark));
    assert!(a.pieces().count() > 0);
    for unit in [a, b] {
        for geometry in unit.pieces() {
            let SourceRangeGeometry::CellFont(piece) = geometry else {
                panic!("font-backed marked EGC");
            };
            assert!(piece.cell_unit().same_unit(&unit.cell_unit().unwrap()));
            assert_eq!(piece.effective_instance().face().bytes(), bytes.as_slice());
            assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
        }
    }
}

#[test]
fn a_genuine_inner_trimmed_space_has_removed_identity_and_no_borrowed_rectangle() {
    // Existing source+font fixture already qualifies this exact provider-empty
    // Cell in allocated_empty. It remains a real E10 square, not an empty line.
    let bytes = qualified_font();
    let (processed, analyzed) = captured(" ", WhiteSpace::Collapse, 0.0);
    source_custody(&processed, &analyzed, &[], 0.0);
    let source = analyzed.combined_cells().next().unwrap();
    assert_eq!(
        (source.selection_unit_count(), source.used_unit_count()),
        (1, 0)
    );
    assert_eq!(source.presentation_code_points().len(), 0);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert!(!prepared.cell_views().next().unwrap().has_provider_input());
    let origin = issued(&prepared, &processed, 0, Range::new(0, 1));
    let line = complete_line(&prepared);
    one_em(&line);
    assert!(line.run_views().all(|run| run.glyphs().next().is_none()));
    let layout = placed_cell(&prepared, line, &bytes, 1.0);
    assert!(
        layout
            .glyph_runs()
            .iter()
            .all(|run| run.glyphs().is_empty())
    );
    let result = layout
        .source_range_fragments(&origin, Range::new(0, 1))
        .unwrap();
    assert_eq!(result.units().count(), 1);
    let unit = result.units().next().unwrap();
    let owner = unit.cell_unit().unwrap();
    assert_eq!(
        owner.kind(),
        SourceRangeCellUnitKind::Removed { ordinal: 0 }
    );
    assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
    assert_eq!(owner.source_cell().used_unit_count(), 0);
    assert_eq!(unit.pieces().count(), 0);
    assert!(rectangles(&result).is_empty());
    assert_eq!(
        (
            unit.presentation_range().start(),
            unit.presentation_range().end()
        ),
        (0, 0)
    );
    for pieces in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].origin(), origin);
        assert_eq!(pieces[0].local_range(), Range::new(0, 1));
        assert_eq!(pieces[0].aggregate_range(), Range::new(0, 1));
    }
}

#[path = "combined_cell_range_removed_member.rs"]
mod removed_member;
