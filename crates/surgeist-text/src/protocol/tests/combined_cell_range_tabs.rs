//! Genuine native TAB spans remain separate from font metric geometry.
use super::*;
use crate::engine::layout::data::{ClusterContent, FinalizedRunFrame};
use crate::{SourceRangeCellUnitKind, SourceRangeGeometry, SourceRangeUnit};

fn place(prepared: &PreparedText, line: FinalizedLine) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn issued(prepared: &PreparedText, capture: &Captured, index: usize) -> PreparedOrigin {
    let position = capture.processed.captured_origin(index).unwrap();
    let view = prepared
        .origin_views()
        .find(|view| view.capture_position().as_ref() == Some(&position))
        .unwrap();
    assert_eq!(view.source(), capture.processed.source());
    assert_eq!(view.local_range(), Range::new(index, index + 1));
    assert_eq!(view.aggregate_range(), Range::new(index, index + 1));
    view.handle()
}

fn exact_source(unit: SourceRangeUnit<'_>, origin: &PreparedOrigin, ordinal: usize) {
    let range = Range::new(ordinal, ordinal + 1);
    for pieces in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        assert_eq!(pieces.len(), 1);
        assert_eq!(pieces[0].origin(), *origin);
        assert_eq!(pieces[0].local_range(), range);
        assert_eq!(pieces[0].aggregate_range(), range);
    }
    assert_eq!(
        (
            unit.presentation_range().start(),
            unit.presentation_range().end()
        ),
        (ordinal, ordinal + 1)
    );
    let owner = unit.cell_unit().unwrap();
    assert_eq!(owner.kind(), SourceRangeCellUnitKind::Semantic { ordinal });
    assert_eq!((owner.line_index(), owner.cell_index()), (0, 0));
    assert_eq!(owner.presentation_range(), unit.presentation_range());
}

fn native_incidence_before_query(
    layout: &Layout,
    atom: usize,
    before: f64,
    after: f64,
    amount: f64,
) {
    let data = &layout.inner.lines[0].finalized.data;
    let [cell] = data.cells.as_slice() else {
        panic!("one actual finalized Cell");
    };
    let [span] = cell.tabs.as_slice() else {
        panic!("one actual executed native TAB span");
    };
    assert_eq!(
        (span.atom, span.before, span.after, span.amount),
        (atom, before, after, amount)
    );
    let cluster = &data.clusters[span.cluster];
    assert_eq!(cluster.frame, FinalizedRunFrame::Cell(0));
    assert_eq!(cluster.source, Range::new(atom, atom + 1));
    assert!(matches!(cluster.content, ClusterContent::CssTab(_)));
    assert_eq!(cluster.content.font_run(), None);
    assert_eq!(
        (cluster.x, cluster.advance, cluster.shaped_advance),
        (before, amount, 0.0)
    );
    assert_eq!(cluster.logical_adjustments, [0.0; 3]);
    // This historical field retains native atom incidence too. It is NOT
    // evidence of a provider contribution: the authentic CssTab discriminator,
    // absent font-run and executed span above are required. Font shares below
    // may use ONLY the genuinely font-backed rows, never this native row.
    assert!(cluster.provider_atoms.contains(&atom));
    for provider in data
        .clusters
        .iter()
        .filter(|row| matches!(row.content, ClusterContent::Provider(_)))
    {
        assert!(
            !provider.provider_atoms.contains(&atom),
            "this ASCII fixture splits real provider requests at its native TAB"
        );
    }
}

fn em_reference(
    piece: crate::SourceRangeCellTabPiece<'_>,
    prepared: &PreparedText,
    capture: &Captured,
) {
    let expected = prepared
        .scope_metrics(&capture.processed.captured_scope(1).unwrap())
        .unwrap();
    let actual = piece.container_metrics();
    assert_eq!(
        actual.captured_scope(),
        capture.processed.captured_scope(1).unwrap()
    );
    assert_eq!(
        actual.font_reference(),
        reference(LineOrientation::Vertical)
    );
    assert_eq!(actual.orientation(), LineOrientation::Vertical);
    assert_eq!(actual.font_size(), 10.0);
    assert_eq!(actual.index, expected.index);
    assert!(std::ptr::eq(actual.prepared, prepared.inner.as_ref()));
    assert_eq!(piece.layout_frame().em_band_extent(), 10.0);
    // This is the true native container metric row, not a body provider run.
    // Its scope is Inline10; the lexical Block is explicitly20 in these cases.
}

#[test]
fn a_native_tab_between_two_real_font_inputs_has_its_own_span_and_no_provider_share() {
    let bytes = qualified_font(); // exact public Shaper/SPACE/ZERO/negative-width receipts.
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    let size = TabSize::spaces(2.0).unwrap();
    // Genuine unchanged ordinary operation: its Block20 owns T40, so A0,
    // TAB10..40, B40..50. This is not an All Cell/source reinterpretation.
    let ordinary = captured(
        "A\tB",
        size,
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::None,
        LineOrientation::Vertical,
        "Ahem",
    );
    assert_eq!(ordinary.analyzed.combined_cells().len(), 0);
    let native = system
        .prepare(&ordinary.analyzed, options(LineOrientation::Vertical))
        .unwrap();
    let ordinary_line = complete(&native, TabContext::CssStops { origin: Some(0.0) });
    assert_eq!(ordinary_line.feedback().advance, 50.0);
    assert_eq!(
        raw_glyphs(&ordinary_line, &bytes, false),
        [(glyph_id(&bytes, 'A'), 0.0), (glyph_id(&bytes, 'B'), 40.0)]
    );

    let capture = captured(
        "A\tB",
        size,
        20.0,
        ParagraphBaseDirection::LeftToRight,
        TextCombineUpright::All,
        LineOrientation::Vertical,
        "Ahem",
    );
    source_premises(
        &capture,
        "A\tB",
        ParagraphBaseDirection::LeftToRight,
        &[0, 0, 0],
    );
    let prepared = system
        .prepare(&capture.analyzed, options(LineOrientation::Vertical))
        .unwrap();
    container_premises(&prepared, &capture, &bytes);
    let line = complete(&prepared, TabContext::NoTabs);
    assert_eq!(line.feedback().advance, 10.0);
    assert_eq!(
        raw_glyphs(&line, &bytes, true),
        [(glyph_id(&bytes, 'A'), 0.0), (glyph_id(&bytes, 'B'), 20.0)]
    );
    let layout = place(&prepared, line);
    assert_eq!(layout.bounds(), square(LineOrientation::Vertical));
    native_incidence_before_query(&layout, 1, 10.0, 20.0, 10.0);
    let data = &layout.inner.lines[0].finalized.data;
    for run in &data.font_runs {
        assert_eq!(run.metrics.font_origin_offset(), 0.0);
    }
    for atom in [0, 2] {
        let rows = data
            .clusters
            .iter()
            .filter(|row| {
                row.frame == FinalizedRunFrame::Cell(0) && row.source == Range::new(atom, atom + 1)
            })
            .collect::<Vec<_>>();
        let [row] = rows.as_slice() else {
            panic!("one genuine finer ASCII font contribution");
        };
        assert!(matches!(row.content, ClusterContent::Provider(_)));
        assert_eq!(row.provider_atoms, atom..atom + 1);
        assert_eq!(
            (row.x, row.advance),
            (if atom == 0 { 0.0 } else { 20.0 }, 10.0)
        );
    }
    let tab_origin = issued(&prepared, &capture, 1);
    let result = layout
        .source_range_fragments(&tab_origin, Range::new(1, 2))
        .unwrap();
    assert_eq!(result.units().count(), 1);
    let unit = result.units().next().unwrap();
    exact_source(unit, &tab_origin, 1);
    let geometries = unit.pieces().collect::<Vec<_>>();
    let [SourceRangeGeometry::CellTab(piece)] = geometries.as_slice() else {
        panic!("native TAB geometry cannot borrow either adjacent font or the whole square");
    };
    assert!(piece.cell_unit().same_unit(&unit.cell_unit().unwrap()));
    assert_eq!(piece.span_index(), 0);
    assert_eq!(piece.inner_advance_edges(), (10.0, 20.0));
    assert_eq!(piece.advance(), 10.0);
    let frame = piece.layout_frame();
    assert_eq!(frame.origin(), Point::new(95.0, 20.0)); // native em origin, NOT glyph baseline28.
    let scale = 10.0_f64 / 30.0;
    assert_eq!(frame.x_scale(), scale);
    em_reference(*piece, &prepared, &capture);
    // Native grid: SPACE10*2=T20, preceding A10, gap10>=half-ch5,
    // B20..30. All width features are complete negative, so normal W30
    // compresses by1/3. Preserve actual multiply, x-offset, anchor order;
    // rectangle width is rounded physical endpoint difference, not literal10/3.
    let before = 95.0 + (0.0 + scale * 10.0);
    let after = 95.0 + (0.0 + scale * 20.0);
    assert_eq!(piece.rect(), Rect::new(before, 20.0, after - before, 10.0));
    assert_ne!(piece.rect(), square(LineOrientation::Vertical));
    // Adjacent original queries retain their OWN font advances. The native
    // TAB may not become a third member of an invented shared-provider span.
    for (atom, start, end) in [(0, 0.0, 10.0), (2, 20.0, 30.0)] {
        let origin = issued(&prepared, &capture, atom);
        let query = layout
            .source_range_fragments(&origin, Range::new(atom, atom + 1))
            .unwrap();
        assert_eq!(query.units().count(), 1);
        let font_unit = query.units().next().unwrap();
        exact_source(font_unit, &origin, atom);
        let pieces = font_unit.pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellFont(font)] = pieces.as_slice() else {
            panic!("genuine adjacent ASCII font");
        };
        assert_eq!(font.inner_advance_edges(), (start, end));
        assert_eq!(font.effective_instance().face().bytes(), bytes.as_slice());
        assert_eq!(font.metric_frame().origin(), Point::new(95.0, 28.0));
        assert_eq!(font.metric_frame().x_scale(), scale);
        let left = 95.0 + (0.0 + scale * start);
        let right = 95.0 + (0.0 + scale * end);
        assert_eq!(font.rect(), Rect::new(left, 20.0, right - left, 10.0));
    }
}

#[test]
fn provider_free_positive_and_zero_tabs_expose_layout_spans_without_body_metrics_or_glyph_frames() {
    let bytes = qualified_font();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.clone()).unwrap();
    for (period, scale, offset) in [(20.0, 0.5, 0.0), (0.0, 1.0, 5.0)] {
        let capture = captured(
            "\t",
            TabSize::length(period).unwrap(),
            20.0,
            ParagraphBaseDirection::LeftToRight,
            TextCombineUpright::All,
            LineOrientation::Vertical,
            "Ahem",
        );
        source_premises(&capture, "\t", ParagraphBaseDirection::LeftToRight, &[0]);
        let prepared = system
            .prepare(&capture.analyzed, options(LineOrientation::Vertical))
            .unwrap();
        container_premises(&prepared, &capture, &bytes);
        let cell = prepared.cell_views().next().unwrap();
        assert_eq!(cell.extent(), 10.0);
        assert!(!cell.has_provider_input());
        let line = complete(&prepared, TabContext::NoTabs);
        assert_eq!(line.feedback().advance, 10.0);
        assert!(line.run_views().all(|run| run.glyphs().next().is_none()));
        let layout = place(&prepared, line);
        assert_eq!(layout.bounds(), square(LineOrientation::Vertical));
        assert!(layout.glyph_runs().is_empty());
        let data = &layout.inner.lines[0].finalized.data;
        assert!(data.font_runs.is_empty());
        assert!(data.runs.is_empty());
        assert!(data.cells[0].frame.is_none()); // no body-baseline/paint frame.
        assert!(data.cells[0].transform.is_some()); // real independent native layout.
        native_incidence_before_query(&layout, 0, 0.0, period, period);
        let origin = issued(&prepared, &capture, 0);
        let result = layout
            .source_range_fragments(&origin, Range::new(0, 1))
            .unwrap();
        assert_eq!(result.units().count(), 1);
        let unit = result.units().next().unwrap();
        exact_source(unit, &origin, 0);
        assert_eq!(unit.cell_unit().unwrap().source_cell().used_unit_count(), 1);
        let pieces = unit.pieces().collect::<Vec<_>>();
        let [SourceRangeGeometry::CellTab(piece)] = pieces.as_slice() else {
            panic!("retained native operation, including zero, has actual TAB geometry");
        };
        assert_eq!(piece.span_index(), 0);
        assert_eq!(piece.inner_advance_edges(), (0.0, period));
        assert_eq!(piece.advance(), period);
        assert_eq!(piece.layout_frame().x_scale(), scale);
        assert_eq!(
            piece.layout_frame().origin(),
            Point::new(95.0 + offset, 20.0)
        );
        em_reference(*piece, &prepared, &capture);
        // Positive20 compresses to10. Zero width stays scale1, centered by5;
        // it remains a genuine Semantic0 TAB, not Removed or an empty result.
        let left = 95.0 + (offset + scale * 0.0);
        let right = 95.0 + (offset + scale * period);
        assert_eq!(piece.rect(), Rect::new(left, 20.0, right - left, 10.0));
        assert!(piece.cell_unit().same_unit(&unit.cell_unit().unwrap()));
    }
}
