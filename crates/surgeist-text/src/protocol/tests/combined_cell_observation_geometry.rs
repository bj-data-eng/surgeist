//! Actual Cell allocation, outer-edge and logical-outline observations.
//! This child reuses licensed registered fonts and authentic completed captures;
//! no inner glyph row is promoted into a substitute outer Cell occurrence.

use super::*;
use crate::protocol::prepared::PresentationUnitKind;
use skrifa::{
    MetadataProvider,
    instance::{LocationRef, Size as FontSize},
};

fn horizontal_metric_premises(processed: &ProcessedSource, font: &[u8]) {
    glyph_frame_font_premises(font);
    let parsed = skrifa::FontRef::new(font).unwrap();
    let metrics = parsed.metrics(FontSize::new(10.0), LocationRef::default());
    assert_eq!(
        (metrics.ascent, metrics.descent, metrics.leading),
        (8.0, -2.0, 0.0)
    );
    let scope = processed.captured_scope(1).unwrap();
    let style = scope.style().unwrap();
    assert_eq!(style.size, 10.0);
    assert_eq!(style.line_height, LineHeight::Absolute(10.0));
}

fn canonical_outline_premises(line: &FinalizedLine, font: &[u8], glyphs: &[u32]) {
    let runs = line
        .run_views()
        .filter(|run| run.glyphs().next().is_some())
        .collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("one actual retained horizontal font instance");
    };
    assert!(run.cell().is_some());
    assert_eq!(run.face().bytes(), font);
    assert_eq!(run.font_size(), 10.0);
    let instance = run.effective_instance().unwrap();
    let face = instance.face();
    for &glyph in glyphs {
        // Ahem's source glyf bounds are [0,-200,1000,800] at UPEM1000.
        // The independent canonical qualifier is baseline-relative y-down;
        // changing hmtx(B) to500 does not change that real outline width1000.
        let outline = instance
            .glyph_outline(glyph, face.catalogue_identity())
            .unwrap();
        assert_eq!(outline.glyph_id(), glyph);
        assert_eq!(outline.instance().face().bytes(), font);
        assert_eq!(outline.bounds(), Some(Rect::new(0.0, -8.0, 10.0, 10.0)));
    }
}

#[test]
fn placed_inner_cluster_metric_rows_follow_the_real_horizontal_cell_frame() {
    for (text, scale, expected) in [
        (
            "AA",
            0.5,
            vec![
                (Range::new(0, 1), Rect::new(95.0, 20.0, 5.0, 10.0)),
                (Range::new(1, 2), Rect::new(100.0, 20.0, 5.0, 10.0)),
            ],
        ),
        (
            "A",
            1.0,
            vec![(Range::new(0, 1), Rect::new(95.0, 20.0, 10.0, 10.0))],
        ),
    ] {
        let (processed, _, prepared, line, font) = prepared_cell(text);
        horizontal_metric_premises(&processed, &font);
        // Inner ascent8/descent2 and line-height10 give metric y0..10 about
        // the Cell's inner baseline8. Raw advances0..10/10..20 are scaled
        // horizontally inside E10, then placed at square origin(95,20).
        let layout = placed_cell(&prepared, line, &font, scale);
        assert_eq!(layout.source(), processed.source());
        let mut actual = layout
            .clusters()
            .into_iter()
            .map(|row| (row.range(), row.bounds()))
            .collect::<Vec<_>>();
        actual.sort_by_key(|(range, _)| range.start);
        assert_eq!(actual, expected);
    }
}

fn with_tall_line_band(prepared: &PreparedText, line: &FinalizedLine) -> FinalizedLine {
    let mut request = line.selected_line().request().clone();
    // An actual caller-supplied line strut dominates the genuine Cell's5/5
    // contribution. It is line allocation, not invented glyph/font geometry.
    request.line_strut = Some(LineStrut {
        baseline: 8.0,
        line_extent: 24.0,
    });
    prepared
        .select(request)
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap()
}

fn one_outer_edge(prepared: &PreparedText, line: &FinalizedLine, source: &[Range]) {
    one_em(line);
    assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, 10.0, 24.0));
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one authentic outer Cell");
    };
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(
        cell.source_cell()
            .source_pieces()
            .map(|piece| piece.aggregate())
            .collect::<Vec<_>>(),
        source
    );
    let participants = prepared.participants().collect::<Vec<_>>();
    let [participant @ PreparedParticipant::Cell(_)] = participants.as_slice() else {
        panic!("one real prepared Cell participant");
    };
    let selected = line.selected_line();
    let units = selected
        .inner
        .presentation
        .units
        .iter()
        .filter(|unit| matches!(unit.kind, PresentationUnitKind::Cell { .. }))
        .collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("one authentic selected Cell occurrence");
    };
    let expected_presentation = PresentationRange {
        start: unit.presentation.start,
        end: unit.presentation.end,
    };
    let edges = line.edge_units().collect::<Vec<_>>();
    assert_eq!(
        edges.len(),
        1,
        "inner provider rows do not replace the outer Cell edge"
    );
    let edge = &edges[0];
    assert_eq!(edge.advance(), 10.0);
    assert_eq!(edge.bounds(), Rect::new(0.0, 0.0, 10.0, 24.0));
    assert_eq!(edge.presentation_range(), expected_presentation);
    assert_eq!(edge.source_pieces().collect::<Vec<_>>(), source);
    assert_eq!(
        edge.owners().collect::<Vec<_>>(),
        [LineEdgeOwner::Participant(participant.clone())]
    );
    assert_eq!(edge.disposition(), LineEdgeDisposition::Ordinary);
    assert_eq!(edge.excluded_contribution(), 0.0);
}

#[test]
fn provider_backed_cell_edge_keeps_whole_custody_and_the_full_line_band() {
    let (_, _, prepared, line, _) = prepared_cell("AA");
    let line = with_tall_line_band(&prepared, &line);
    one_outer_edge(&prepared, &line, &[Range::new(0, 1), Range::new(1, 2)]);
}

#[test]
fn provider_empty_source_cell_has_an_outer_edge_without_an_inner_cluster_or_ink_row() {
    let font = qualified_font();
    let (processed, analyzed) = captured(" ", WhiteSpace::Collapse, 0.0);
    source_custody(&processed, &analyzed, &[], 0.0);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert!(!prepared.cell_views().next().unwrap().has_provider_input());
    let line = with_tall_line_band(&prepared, &complete_line(&prepared));
    one_outer_edge(&prepared, &line, &[Range::new(0, 1)]);
    // This genuine source-nonempty Cell has E10 without any inner provider row.
    let placed = placed_vertical_cell(&prepared, line.clone());
    assert!(placed.clusters().is_empty());
    assert_eq!(line.outline_ink().unwrap(), LineOutlineInk::Empty);
}

fn logical_outline(line: &FinalizedLine) -> Rect {
    match line.outline_ink().unwrap() {
        LineOutlineInk::Bounds(bounds) => bounds,
        LineOutlineInk::Empty => panic!("actual Ahem outlines are nonempty"),
    }
}

#[test]
fn cell_outline_ink_uses_logical_line_axes_and_real_scaled_outline_overflow() {
    let (mut system, processed, analyzed, body) = source_with_full_coverage("AB");
    width_actions_cover_only_original_a(&body);
    assert_eq!(sfnt::u16_at(sfnt::table(&body, *b"hmtx"), 35 * 4), 1000);
    assert_eq!(sfnt::u16_at(sfnt::table(&body, *b"hmtx"), 36 * 4), 500);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let line = complete_line(&prepared);
    horizontal_metric_premises(&processed, &body);
    canonical_outline_premises(&line, &body, &[35, 36]);
    let prepared_cell_view = prepared.cell_views().next().unwrap();
    assert_eq!(
        (prepared_cell_view.before(), prepared_cell_view.after()),
        (5.0, 5.0)
    );
    assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, 10.0, 10.0));
    assert_eq!(
        line.run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), body.as_slice());
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.advance()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>(),
        [(35, 10.0), (36, 5.0)]
    );
    one_em(&line);
    // Original B has no width-feature action, so normal W15 compresses2/3.
    // Both actual outlines are width10: h0..20 becomes0..40/3, wider than E10.
    // Vertical inverse therefore yields inline0..10 and under-10/3..10.
    let bounds = logical_outline(&line);
    close(bounds.origin.x, 0.0);
    close(bounds.origin.y, -10.0 / 3.0);
    close(bounds.size.width, 10.0);
    close(bounds.size.height, 40.0 / 3.0);

    let (processed, _, _prepared, line, font) = prepared_cell("AA");
    horizontal_metric_premises(&processed, &font);
    canonical_outline_premises(&line, &font, &[35]);
    assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, 10.0, 10.0));
    // Horizontal h0..10/v0..10 maps through Vertical's actual inverse:
    // inline=I0+v, under=U1-h. This5/5 Cell-only line has I0=0/U1=10.
    assert_eq!(logical_outline(&line), Rect::new(0.0, 0.0, 10.0, 10.0));
}

#[test]
fn nonzero_outer_inline_offset_is_applied_once_to_cell_metrics_and_logical_ink() {
    let (processed, _, prepared, original, font) = prepared_cell("AA");
    horizontal_metric_premises(&processed, &font);
    let mut request = original.selected_line().request().clone();
    request.context.indent = 7.0;
    let line = prepared
        .select(request)
        .unwrap()
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::None,
        })
        .unwrap();
    canonical_outline_premises(&line, &font, &[35]);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine offset Cell");
    };
    assert_eq!(cell.square().origin.x, 7.0);
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    // The actual Cell starts at inline7. Its horizontal h0..10/v0..10
    // footprint maps to logical inline7..17/under0..10. Placement adds
    // the real inline20 anchor exactly once; the cross baseline stays100.
    assert_eq!(logical_outline(&line), Rect::new(7.0, 0.0, 10.0, 10.0));
    let layout = placed_vertical_cell(&prepared, line.clone());
    let mut rows = layout
        .clusters()
        .into_iter()
        .map(|row| (row.range(), row.bounds()))
        .collect::<Vec<_>>();
    rows.sort_by_key(|(range, _)| range.start);
    assert_eq!(
        rows,
        [
            (Range::new(0, 1), Rect::new(95.0, 27.0, 5.0, 10.0)),
            (Range::new(1, 2), Rect::new(100.0, 27.0, 5.0, 10.0)),
        ]
    );
    let edges = line.edge_units().collect::<Vec<_>>();
    let [edge] = edges.as_slice() else {
        panic!("one authentic offset outer edge");
    };
    assert_eq!(edge.advance(), 10.0);
    assert_eq!(edge.bounds(), Rect::new(7.0, 0.0, 10.0, 10.0));
}

#[test]
fn a_cell_metric_allocation_that_absorbs_at_a_finite_anchor_is_refused_before_publication() {
    let (processed, _, prepared, line, font) = prepared_cell("AA");
    horizontal_metric_premises(&processed, &font);
    let retained = placed_cell(&prepared, line.clone(), &font, 0.5);
    let retained_clusters = retained.clusters();
    let retained_bounds = retained.bounds();
    // At this exact binade boundary, the existing square path survives:
    // line_point(under0) rounds B+5 to B, then line_rect subtracts10 to B-8.
    // Its stored positive10 extent still has a strictly greater right edge.
    const B: f64 = 72_057_594_037_927_936.0; // exactly2^56
    assert!(B.is_finite());
    assert_eq!(B + 5.0, B);
    assert_eq!(B - 10.0, B - 8.0);
    const {
        assert!((B - 8.0) + 10.0 > B - 8.0);
    }
    // The actual placed Cell frame instead starts at line_point(under10),
    // B-5 -> B-8. Directly mapping AA's scaled horizontal h0/5/10 endpoints yields
    // B-8/B/B: the second real5-wide allocation has lost its entire span.
    assert_eq!(B - 5.0, B - 8.0);
    assert_eq!((B - 8.0) + 5.0, B);
    assert_eq!((B - 8.0) + 10.0, B);
    let error = Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line: line.clone(),
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: B,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .expect_err("a surviving outer square does not authorize a collapsed inner metric allocation");
    assert_eq!(error.code, ErrorCode::InvalidTextNumeric);
    assert!(
        matches!(error.detail(), Some(ErrorDetail::InvalidTextNumeric {
        field: "cell cluster horizontal metric span",
        phase: TextNumericPhase::Placement,
        category: TextNumericError::PositiveLost,
        value,
    }) if *value == 5.0),
        "unexpected refusal: {error:?}"
    );
    assert_eq!(retained.clusters(), retained_clusters);
    assert_eq!(retained.bounds(), retained_bounds);
    assert_eq!(retained.source(), processed.source());
}
