//! Once-shifted Cell metric frames and independent outer line bands.
use super::*;

fn range_frame_case(orientation: LineOrientation) -> (ContainerFixture, PreparedText) {
    let container_context = if orientation == LineOrientation::Vertical {
        ContainerContext::vertical()
    } else {
        context()
    };
    let mut actual = fixture(
        ContainerShape::Inline,
        "A",
        ContainerContext {
            alignment: Some(ScopeAlignmentBoundary::NonInitial),
            ..container_context
        },
    );
    source_custody(&actual, false);
    glyph_frame_font_premises(&actual.body);
    let prepared = actual
        .system
        .prepare(&actual.analyzed, options(orientation, None))
        .unwrap();
    assert_eq!(
        prepared.analyzed_source().identity(),
        actual.analyzed.identity()
    );
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual captured Cell")
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    assert_eq!(
        (cell.before(), cell.after()),
        if orientation == LineOrientation::Vertical {
            (5.0, 5.0)
        } else {
            (8.0, 2.0)
        },
    );
    let container = actual.processed.captured_scope(actual.container).unwrap();
    let metrics = prepared.scope_metrics(&container).unwrap();
    assert_eq!(metrics.captured_scope(), container);
    assert_eq!(metrics.orientation(), orientation);
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        actual.body.as_slice()
    );
    // Same actual C-family font has H8/-2, no variation/USE_TYPO override,
    // actual Horizontal Alphabetic zero, and a genuine Vertical BASE reference.
    font_premises(metrics.effective_instance().unwrap());
    assert_eq!(
        metrics.font_reference(),
        if orientation == LineOrientation::Vertical {
            font_reference()
        } else {
            horizontal_reference()
        },
    );
    (actual, prepared)
}

fn scoped_range_line(prepared: &PreparedText, displacement: f64, tall: bool) -> FinalizedLine {
    let selected = select(prepared);
    let selected = if tall {
        let mut request = selected.request().clone();
        // Existing scope-effects B20/H40 allocation: it dominates the actual
        // Cell's signed sides even after d3. This is a real caller line strut,
        // not a font, glyph, parent donation or reconstructed Cell metric.
        request.line_strut = Some(LineStrut {
            baseline: 20.0,
            line_extent: 40.0,
        });
        prepared.select(request).unwrap()
    } else {
        selected
    };
    let scopes = selected.scopes().collect::<Vec<_>>();
    let [scope] = scopes.as_slice() else {
        panic!("one genuine NonInitial Inline target")
    };
    assert_eq!(scope.metrics().font_size(), 10.0);
    assert_eq!(scope.parent_metrics().unwrap().font_size(), 20.0);
    selected
        .finalize(LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: ScopeAllocations::Complete(vec![ScopeAllocation {
                scope: scope.clone(),
                displacement: BaselineShift::try_new(displacement).unwrap(),
            }]),
        })
        .unwrap()
}

fn issued_range_origin(prepared: &PreparedText, processed: &ProcessedSource) -> PreparedOrigin {
    let positions = prepared.origin_views().collect::<Vec<_>>();
    let [view] = positions.as_slice() else {
        panic!("one real authored A original")
    };
    assert_eq!(view.source(), processed.source());
    assert_eq!(view.local_range(), Range::new(0, 1));
    assert_eq!(view.aggregate_range(), Range::new(0, 1));
    assert_eq!(
        view.capture_position(),
        Some(processed.captured_origin(0).unwrap())
    );
    view.handle()
}

fn expected_metric_rect(orientation: LineOrientation, d: f64) -> Rect {
    // Anchor inline20/common100. Vertical's native Cell sides5/5 give x95;
    // CW's genuine horizontal8/2 gives x98; CCW gives x92 and y10.
    // Positive-over scope displacement moves physical X right for V/CW and
    // left for CCW. Inner font axes remain horizontal/right and down in all3.
    match orientation {
        LineOrientation::Vertical => Rect::new(95.0 + d, 20.0, 10.0, 10.0),
        CW => Rect::new(98.0 + d, 20.0, 10.0, 10.0),
        CCW => Rect::new(92.0 - d, 10.0, 10.0, 10.0),
        LineOrientation::Horizontal => panic!("this fixture has a genuine vertical Source"),
    }
}

fn assert_scoped_range(
    layout: &Layout,
    origin: &PreparedOrigin,
    processed: &ProcessedSource,
    bytes: &[u8],
    expected: Rect,
) {
    assert_eq!(layout.source(), processed.source());
    let result = layout
        .source_range_fragments(origin, Range::new(0, 1))
        .unwrap();
    let units = result.units().collect::<Vec<_>>();
    let [unit] = units.as_slice() else {
        panic!("one real semantic A EGC")
    };
    assert_eq!(unit.line_index(), 0);
    let pieces = unit.pieces().collect::<Vec<_>>();
    let [SourceRangeGeometry::CellFont(piece)] = pieces.as_slice() else {
        panic!("actual horizontal provider metrics, not an outer-line rectangle");
    };
    assert_eq!(
        piece.cell_unit().kind(),
        SourceRangeCellUnitKind::Semantic { ordinal: 0 }
    );
    assert_eq!(piece.cell_unit().cell_index(), 0);
    assert_eq!(
        piece.cell_unit().source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    assert_eq!(piece.effective_instance().face().bytes(), bytes);
    assert_eq!(piece.inner_advance_edges(), (0.0, 10.0));
    assert_eq!((piece.font_ascent(), piece.font_descent()), (8.0, 2.0));
    assert_eq!(piece.rect(), expected);
    let reference = Point::new(expected.origin.x, expected.origin.y + 8.0);
    assert_eq!(piece.metric_frame().origin(), reference);
    assert_eq!(piece.metric_frame().x_scale(), 1.0);
    for intersections in [
        unit.selected_intersections().collect::<Vec<_>>(),
        unit.expanded_intersections().collect::<Vec<_>>(),
    ] {
        assert_eq!(intersections.len(), 1);
        assert_eq!(intersections[0].origin(), *origin);
        assert_eq!(intersections[0].local_range(), Range::new(0, 1));
        assert_eq!(intersections[0].aggregate_range(), Range::new(0, 1));
    }
    let packets = layout.glyph_runs();
    let [packet] = packets.as_slice() else {
        panic!("one authentic unrotated A packet")
    };
    assert_eq!(packet.font().bytes(), bytes);
    assert_eq!(packet.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(packet.glyph_frame().origin(), reference); // Qualified b0.
    assert_eq!(packet.glyph_frame().outline_x(), Point::new(1.0, 0.0));
    assert_eq!(packet.glyph_frame().outline_y(), Point::new(0.0, 1.0));
    assert_eq!(
        packet
            .glyphs()
            .iter()
            .map(|glyph| (glyph.id(), glyph.pen(), glyph.advance()))
            .collect::<Vec<_>>(),
        [(35, 0.0, 10.0)],
    );
}

#[test]
fn cell_range_uses_once_shifted_horizontal_metric_frames_in_vertical_cw_and_ccw() {
    for orientation in [LineOrientation::Vertical, CW, CCW] {
        for d in [-3.0, 3.0] {
            let (actual, prepared) = range_frame_case(orientation);
            let origin = issued_range_origin(&prepared, &actual.processed);
            let line = scoped_range_line(&prepared, d, false);
            one_em(&line);
            assert_eq!(
                line.cell_views().next().unwrap().scope_displacement().get(),
                d
            );
            drop(actual.system);
            let replay = line.with_spacing(SpacingAllocations::None).unwrap();
            for retained in [line, replay] {
                let layout = place(&prepared, retained);
                assert_scoped_range(
                    &layout,
                    &origin,
                    &actual.processed,
                    &actual.body,
                    expected_metric_rect(orientation, d),
                );
            }
        }
    }
}

#[test]
fn real_tall_line_strut_does_not_replace_shifted_cell_font_metric_band() {
    for orientation in [LineOrientation::Vertical, CW, CCW] {
        let (actual, prepared) = range_frame_case(orientation);
        let origin = issued_range_origin(&prepared, &actual.processed);
        let line = scoped_range_line(&prepared, 3.0, true);
        one_em(&line);
        assert_eq!(line.metric_bounds(), Rect::new(0.0, 0.0, 10.0, 40.0));
        let edges = line.edge_units().collect::<Vec<_>>();
        let [edge] = edges.as_slice() else {
            panic!("one actual outer Cell edge")
        };
        assert_eq!(edge.bounds(), Rect::new(0.0, 0.0, 10.0, 40.0));
        drop(edges);
        drop(actual.system);
        let replay = line.with_spacing(SpacingAllocations::None).unwrap();
        for retained in [line, replay] {
            let layout = place(&prepared, retained);
            // Real line band40 about B20 maps to physical x80..120. Its full
            // bounds remain distinct from the actual E10/font10 query piece.
            let y = if orientation == CCW { 10.0 } else { 20.0 };
            assert_eq!(layout.lines()[0].bounds(), Rect::new(80.0, y, 40.0, 10.0));
            assert_scoped_range(
                &layout,
                &origin,
                &actual.processed,
                &actual.body,
                expected_metric_rect(orientation, 3.0),
            );
        }
    }
}
