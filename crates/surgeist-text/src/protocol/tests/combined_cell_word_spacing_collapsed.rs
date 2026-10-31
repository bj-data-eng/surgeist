//! Interior collapsed-zero SPACE stays semantically used without a presentation
//! scalar/provider ordinal. The surviving material SPACE receives the sole WS4
//! term; actual Ahem A/SPACE/A therefore occupies34, not another four-unit term.

use super::*;

fn custody(processed: &ProcessedSource, analyzed: &AnalyzedSource) {
    assert_eq!(processed.source().text(), &Text::from("A  A"));
    assert_eq!(processed.processed_text(), &Text::from("A A"));
    assert_eq!(analyzed.source(), processed.source());
    let owner = analyzed.captured_owner().unwrap();
    assert_eq!(
        owner.members().collect::<Vec<_>>(),
        [processed.captured_paragraph(0).unwrap()]
    );
    assert!(owner.joint_flow().is_none());
    let cells = analyzed.combined_cells().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one genuine complete Inline All Cell");
    };
    assert_eq!(cell.scope(), processed.captured_scope(1).unwrap());
    assert_eq!(
        cell.parent_scope(),
        Some(processed.captured_scope(0).unwrap())
    );
    assert_eq!(cell.mode(), TextCombineUpright::All);
    assert_eq!(cell.inner_direction(), ParagraphBaseDirection::LeftToRight);
    // Source/used EGC cardinalities include both real interior SPACE occurrences.
    // Neither an invented scalar nor genuine earlier elimination represents the
    // second one: retained lexical SPACE is used but has no presentation emission.
    assert_eq!(cell.selection_unit_count(), 4);
    assert_eq!(cell.used_unit_count(), 4);
    assert_eq!(
        cell.used_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x20, 0x41]
    );
    assert_eq!(
        cell.presentation_code_points().collect::<Vec<_>>(),
        [0x41, 0x20, 0x41]
    );
    assert_eq!(
        cell.inner_bidi_levels()
            .map(BidiLevel::get)
            .collect::<Vec<_>>(),
        [0; 4]
    );
    let pieces = cell.source_pieces().collect::<Vec<_>>();
    assert_eq!(pieces.len(), 4);
    for (index, piece) in pieces.iter().enumerate() {
        assert_eq!(piece.source(), processed.source());
        assert_eq!(piece.local(), Range::new(index, index + 1));
        assert_eq!(piece.aggregate(), Range::new(index, index + 1));
        assert_eq!(
            piece.origin_position(),
            processed.captured_origin(index).unwrap()
        );
        assert_eq!(piece.paragraph(), processed.captured_paragraph(0).unwrap());
        assert_eq!(piece.kind(), ParagraphOriginKind::Authored);
        assert!(piece.is_used());
        assert_eq!(piece.has_presentation_emission(), index != 2);
    }
    assert_eq!(pieces[2].emission_subposition(), None);
    let inline = processed.captured_scope(1).unwrap();
    assert_eq!(inline.font_reference(), Some(font_reference()));
    assert_eq!(inline.style().unwrap().white_space, WhiteSpace::Collapse);
    assert_eq!(inline.style().unwrap().word_spacing, 4.0);
    assert_eq!(inline.style().unwrap().letter_spacing, 0.0);
}

fn output(prepared: &PreparedText, line: &FinalizedLine, processed: &ProcessedSource, font: &[u8]) {
    one_em(line);
    let mut raw = Vec::new();
    for run in line.run_views() {
        assert!(run.cell().is_some());
        assert_eq!(run.face().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
        for glyph in run.glyphs() {
            raw.push((
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
            ));
        }
    }
    // Original occurrences are asserted separately above; no provider ordinal
    // or incidental glyph-range envelope is inferred from their semantic count.
    assert_eq!(
        raw,
        [
            (35, 0.0, 0.0, 0.0, 10.0),
            (3, 12.0, 0.0, 0.0, 10.0),
            (35, 24.0, 0.0, 0.0, 10.0)
        ]
    );
    assert_eq!(
        raw.last().unwrap().1 + f64::from(raw.last().unwrap().4),
        34.0
    );
    let layout = placed_vertical_cell(prepared, line.clone());
    assert_eq!(layout.source(), processed.source());
    assert_eq!(
        layout.lines()[0].bounds(),
        Rect::new(95.0, 20.0, 10.0, 10.0)
    );
    let mut physical = Vec::new();
    for run in layout.glyph_runs() {
        assert_eq!(run.font().bytes(), font);
        assert_eq!(run.font_size(), 10.0);
        let frame = run.glyph_frame();
        close(frame.outline_x().x, 5.0 / 17.0);
        assert_eq!(frame.outline_x().y, 0.0);
        assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
        assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
        for glyph in run.glyphs() {
            assert_eq!((glyph.x(), glyph.y(), glyph.advance()), (0.0, 0.0, 10.0));
            physical.push((
                glyph.id(),
                frame.origin().x + frame.outline_x().x * glyph.pen(),
                frame.origin().y,
            ));
        }
    }
    assert_eq!(physical.len(), 3);
    for (actual, (glyph, pen)) in physical.iter().zip([(35, 0.0), (3, 12.0), (35, 24.0)]) {
        assert_eq!(actual.0, glyph);
        close(actual.1, 95.0 + (5.0 / 17.0) * pen);
        close(actual.2, 28.0);
    }
}

#[test]
fn interior_collapsed_zero_space_adds_neither_provider_glyph_nor_extra_word_term() {
    // Public Shaper qualification is over EXACT Ahem A/SPACE/A: three normal
    // advances1000, all demanded width receipts Complete/NoAction. At size10,
    // only the material SPACE has intrinsic10 and WS4 split2/2: pens0/12/24,
    // occupied0..34, scale10/34. The lexical zero is not a fourth provider input.
    let font = qualified_font();
    let (processed, analyzed) = captured("A  A", WhiteSpace::Collapse, 4.0);
    custody(&processed, &analyzed);
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analyzed.identity());
    custody(&processed, &prepared.analyzed_source());
    let cells = prepared.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual prepared Cell");
    };
    assert_eq!(cell.extent(), 10.0);
    assert!(cell.has_provider_input());
    assert_eq!(cell.source_cell().used_unit_count(), 4);
    assert_eq!(cell.source_cell().presentation_code_points().len(), 3);
    let metrics = prepared
        .scope_metrics(&processed.captured_scope(1).unwrap())
        .unwrap();
    assert_eq!(metrics.font_size(), 10.0);
    assert_eq!(
        metrics.effective_instance().unwrap().face().bytes(),
        font.as_slice()
    );
    let line = complete_line(&prepared);
    let finalized_cells = line.cell_views().collect::<Vec<_>>();
    let [finalized_cell] = finalized_cells.as_slice() else {
        panic!("one actual finalized Cell");
    };
    assert_eq!(finalized_cell.source_cell().used_unit_count(), 4);
    assert_eq!(
        finalized_cell
            .source_cell()
            .presentation_code_points()
            .len(),
        3
    );
    output(&prepared, &line, &processed, &font);
    drop(system);
    let replay = line.with_spacing(SpacingAllocations::None).unwrap();
    custody(&processed, &replay.prepared_text().analyzed_source());
    output(&prepared, &replay, &processed, &font);
}
