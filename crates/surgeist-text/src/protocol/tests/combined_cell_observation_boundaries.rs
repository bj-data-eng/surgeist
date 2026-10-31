//! Native Cells must not claim unsupported Range or placed-AX observations.
//! Their intrinsic horizontal payload is distinct from the executed outer square;
//! global placement bounds still describe that actual one-em square.

use super::*;

fn prepared_cell(
    text: &str,
) -> (
    Arc<ProcessedSource>,
    AnalyzedSource,
    PreparedText,
    FinalizedLine,
    Vec<u8>,
) {
    assert!(matches!(text, "A" | "AA"));
    let font = qualified_font();
    let (processed, analyzed) = captured(text, WhiteSpace::Preserve, 0.0);
    source_custody(
        &processed,
        &analyzed,
        &(0..text.len()).collect::<Vec<_>>(),
        0.0,
    );
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one authentic prepared native Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let line = complete_line(&prepared);
    one_em(&line);
    let finalized_cells = line.cell_views().collect::<Vec<_>>();
    let [finalized_cell] = finalized_cells.as_slice() else {
        panic!("one actual finalized square");
    };
    assert_eq!(finalized_cell.square().origin.x, 0.0);
    assert_eq!(finalized_cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(finalized_cell.scope_displacement().get(), 0.0);
    assert_eq!(
        finalized_cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    let actual = line
        .run_views()
        .flat_map(|run| {
            assert!(run.cell().is_some());
            assert_eq!(run.face().bytes(), font.as_slice());
            assert_eq!(run.font_size(), 10.0);
            assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
            run.glyphs()
                .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        actual,
        if text == "A" {
            vec![(35, 0.0, 10.0)]
        } else {
            vec![(35, 0.0, 10.0), (35, 10.0, 10.0)]
        }
    );
    drop(system);
    (processed, analyzed, prepared, line, font)
}

fn placed_cell(prepared: &PreparedText, line: FinalizedLine, font: &[u8], scale: f64) -> Layout {
    let layout = placed_vertical_cell(prepared, line);
    assert_eq!(layout.lines().len(), 1);
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    for packet in layout.glyph_runs() {
        assert_eq!(packet.font().bytes(), font);
        assert_eq!(packet.font_size(), 10.0);
        assert_eq!(packet.glyph_basis(), GlyphBasis::Unrotated);
        let frame = packet.glyph_frame();
        close(frame.outline_x().x, scale);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        close(frame.origin().y, 28.0);
    }
    layout
}

#[test]
fn visible_native_cell_range_observation_refuses_instead_of_claiming_removed_source() {
    let (processed, _analyzed, prepared, line, font) = prepared_cell("A");
    let capture_origin = processed.captured_origin(0).unwrap();
    let origin = prepared
        .origin_views()
        .find(|origin| origin.capture_position().as_ref() == Some(&capture_origin))
        .expect("the actual captured original occurrence is retained")
        .handle();
    assert_eq!(origin.view().source(), processed.source());
    assert_eq!(origin.view().local_range(), Range::new(0, 1));
    let layout = placed_cell(&prepared, line, &font, 1.0);
    assert_eq!(layout.source(), processed.source());
    let error = layout.source_range_fragments(&origin, Range::new(0, 1))
        .expect_err("visible provider-backed Cell source cannot masquerade as an empty removed-source observation");
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(
        matches!(
            error.detail(),
            Some(ErrorDetail::SourceRangeFragments { .. })
        ),
        "the Range observation must own its Cell capability refusal: {error:?}"
    );
}

#[test]
fn compressed_cell_global_bounds_use_the_executed_one_em_square() {
    let (processed, _analyzed, prepared, line, font) = prepared_cell("AA");
    // Ahem normal2x10 plus complete negative width receipts gives scale.5.
    // Those intrinsic horizontal payload rows do not create a vertical20 box.
    let layout = placed_cell(&prepared, line, &font, 0.5);
    assert_eq!(layout.source(), processed.source());
    assert_eq!(layout.bounds(), Rect::new(95.0, 20.0, 10.0, 10.0));
    assert_eq!(layout.bounds(), layout.lines()[0].bounds());
}

#[cfg(feature = "text-accessibility")]
#[test]
fn placed_cell_ax_refusal_preserves_the_successful_source_only_binding() {
    use accesskit::NodeId;
    let (processed, analyzed, prepared, line, font) = prepared_cell("AA");
    let mut state = Accessibility::new(10_000);
    let parent = NodeId(1);
    let focus = NodeId(1);
    let source_update = analyzed
        .accessibility_update(&mut state, parent, focus)
        .expect("source-only host text has no placed Cell geometry claim");
    assert!(!source_update.nodes.is_empty());
    let start = Cursor::new(0, Affinity::After);
    let end = Cursor::new(2, Affinity::Before);
    let selection = Selection::new(start, end);
    let before_start = analyzed
        .access_position(&state, start)
        .unwrap()
        .expect("source start AX position");
    let before_end = analyzed
        .access_position(&state, end)
        .unwrap()
        .expect("source end AX position");
    let before_selection = analyzed
        .access_selection(&state, selection)
        .unwrap()
        .expect("full original AA selection");
    let layout = placed_cell(&prepared, line, &font, 0.5);
    assert_eq!(layout.source(), processed.source());
    let error = layout
        .accessibility_update(&mut state, parent, focus, Point::new(0.0, 0.0))
        .expect_err("unimplemented placed Cell AX cannot publish legacy inner-cluster geometry");
    assert_eq!(error.code, ErrorCode::UnsupportedFeature);
    assert!(
        error.detail().is_some(),
        "placed Cell AX refusal must retain a typed diagnostic"
    );
    assert_eq!(
        analyzed.access_position(&state, start).unwrap(),
        Some(before_start)
    );
    assert_eq!(
        analyzed.access_position(&state, end).unwrap(),
        Some(before_end)
    );
    assert_eq!(
        analyzed.access_selection(&state, selection).unwrap(),
        Some(before_selection)
    );
}
