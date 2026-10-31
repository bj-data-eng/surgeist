//! Signed inner word geometry stays horizontal inside either sideways outer
//! line. Actual Native controls precede the existing public Cell prepare guard.
use super::*;
use skrifa::raw::{TableProvider, tables::os2::SelectionFlags};

fn horizontal_reference() -> ScopeFontReference {
    ScopeFontReference {
        script: FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        axis: FontBaselineAxis::HorizontalY,
        baseline: FontBaseline::Alphabetic,
    }
}

#[derive(Clone, Copy)]
struct WordCase {
    word: f64,
    pen: f64,
    terminal: f64,
    occupied: (f64, f64),
    scale: f64,
    offset: f64,
}
const CASES: [WordCase; 4] = [
    WordCase {
        word: 4.0,
        pen: 2.0,
        terminal: 14.0,
        occupied: (0.0, 14.0),
        scale: 5.0 / 7.0,
        offset: 0.0,
    },
    WordCase {
        word: -2.0,
        pen: -1.0,
        terminal: 8.0,
        occupied: (-1.0, 9.0),
        scale: 1.0,
        offset: 1.0,
    },
    WordCase {
        word: -10.0,
        pen: -5.0,
        terminal: 0.0,
        occupied: (-5.0, 5.0),
        scale: 1.0,
        offset: 5.0,
    },
    WordCase {
        word: -30.0,
        pen: -15.0,
        terminal: -20.0,
        occupied: (-20.0, 0.0),
        scale: 0.5,
        offset: 10.0,
    },
];

fn no_cell_control(
    system: &mut System,
    font: &[u8],
    orientation: LineOrientation,
    word: f64,
    pen: f64,
    terminal: f64,
) {
    let (processed, analyzed) = captured_with_context(
        " ",
        WhiteSpace::Preserve,
        word,
        AnalysisTypography::HorizontalComposition,
        horizontal_reference(),
        TextCombineUpright::None,
    );
    assert_eq!(
        processed.captured_scope(1).unwrap().writing_axis(),
        Some(SourceWritingAxis::Vertical)
    );
    assert_eq!(
        analyzed.typography(),
        AnalysisTypography::HorizontalComposition
    );
    assert_eq!(analyzed.combined_cells().len(), 0);
    let options = PreparationOptions::default()
        .with_orientation(orientation)
        .with_baseline_reference(FontBaseline::Alphabetic);
    assert_eq!(
        options.requested_baseline_reference(),
        Some(FontBaseline::Alphabetic)
    );
    let prepared = system.prepare(&analyzed, options).unwrap();
    assert_eq!(prepared.cell_views().len(), 0);
    let line = complete_line(&prepared);
    close(line.feedback().advance, terminal);
    let runs = line.run_views().collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("one real ordinary NBSP run");
    };
    assert!(run.cell().is_none());
    assert_eq!(run.face().bytes(), font);
    assert_eq!(
        run.glyph_basis(),
        match orientation {
            LineOrientation::SidewaysClockwise => GlyphBasis::Clockwise,
            LineOrientation::SidewaysCounterclockwise => GlyphBasis::Counterclockwise,
            _ => unreachable!(),
        }
    );
    let instance = run.effective_instance().unwrap();
    let metrics = instance
        .metrics(instance.face().catalogue_identity())
        .unwrap();
    assert_eq!(metrics.units_per_em(), 1000);
    assert_eq!(metrics.line_metric_source(), FontLineMetricSource::Hhea);
    assert_eq!((metrics.ascent(), metrics.descent()), (8.0, -2.0));
    let reference = horizontal_reference();
    let baseline = instance
        .baseline(
            instance.face().catalogue_identity(),
            reference.script,
            reference.axis,
            reference.baseline,
        )
        .unwrap();
    let FontBaselineOutcome::Available(value) = baseline.outcome() else {
        panic!("real horizontal Alphabetic coordinate");
    };
    assert_eq!(value.coordinate(), 0.0);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (glyph.id(), run.origin().x + glyph.pen(), glyph.advance()))
            .collect::<Vec<_>>(),
        [(153, pen, 10.0)]
    );
}

fn place(prepared: &PreparedText, line: FinalizedLine, orientation: LineOrientation) -> Layout {
    Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation,
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn output(
    prepared: &PreparedText,
    line: &FinalizedLine,
    processed: &ProcessedSource,
    font: &[u8],
    orientation: LineOrientation,
    expected: WordCase,
) {
    // These are model operands, not invented public interval/terminal getters.
    // Signed flow14/8/0/-20 differs from occupied width14/10/10/20.
    assert_eq!(expected.pen, expected.word / 2.0);
    assert_eq!(expected.terminal, 10.0 + expected.word);
    assert_eq!(
        expected.occupied,
        (
            0.0_f64.min(expected.terminal).min(expected.pen),
            0.0_f64.max(expected.terminal).max(expected.pen + 10.0),
        )
    );
    let width = expected.occupied.1 - expected.occupied.0;
    close(
        expected.scale,
        if width > 10.0 { 10.0 / width } else { 1.0 },
    );
    close(
        expected.offset,
        (10.0 - width * expected.scale) / 2.0 - expected.scale * expected.occupied.0,
    );
    assert_eq!(line.feedback().advance, 10.0); // One outer Cell flow_end, not inner terminal.
    assert_eq!(line.measurements().fit().advance(), 10.0);
    let cells = line.cell_views().collect::<Vec<_>>();
    let [cell] = cells.as_slice() else {
        panic!("one actual sideways outer Cell");
    };
    assert_eq!(
        cell.source_cell().scope(),
        processed.captured_scope(1).unwrap()
    );
    assert_eq!(cell.square().size, Size::new(10.0, 10.0));
    assert_eq!(cell.square().origin.x, 0.0);
    assert_eq!(cell.square().origin.y, cell.reference() - 8.0);
    assert_eq!(cell.scope_displacement().get(), 0.0);
    let runs = line.run_views().collect::<Vec<_>>();
    let [run] = runs.as_slice() else {
        panic!("real retained NBSP provider packet despite blank ink");
    };
    assert!(run.cell().is_some());
    assert_eq!(run.face().bytes(), font);
    assert_eq!(run.font_size(), 10.0);
    assert_eq!(run.glyph_basis(), GlyphBasis::Unrotated);
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.range(),
                glyph.id(),
                run.origin().x + glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(Range::new(0, 1), 153, expected.pen, 0.0, 0.0, 10.0)]
    );
    let layout = place(prepared, line.clone(), orientation);
    assert_eq!(layout.source(), processed.source());
    let square = match orientation {
        LineOrientation::SidewaysClockwise => Rect::new(98.0, 20.0, 10.0, 10.0),
        LineOrientation::SidewaysCounterclockwise => Rect::new(92.0, 10.0, 10.0, 10.0),
        _ => unreachable!(),
    };
    assert_eq!(layout.lines()[0].bounds(), square);
    assert_eq!(layout.bounds(), square);
    let packets = layout
        .glyph_runs()
        .into_iter()
        .filter(|run| !run.glyphs().is_empty())
        .collect::<Vec<_>>();
    let [packet] = packets.as_slice() else {
        panic!("one real owned NBSP packet");
    };
    assert_eq!(packet.font().bytes(), font);
    assert_eq!(packet.font_size(), 10.0);
    let frame = packet.glyph_frame();
    assert_eq!(frame.pen_direction(), Point::new(1.0, 0.0));
    close(frame.outline_x().x, expected.scale);
    assert_eq!(frame.outline_x().y, 0.0);
    assert_eq!(frame.outline_y(), Point::new(0.0, 1.0));
    close(
        frame.origin().x,
        square.origin.x + expected.offset + expected.scale * run.origin().x,
    );
    close(frame.origin().y, square.origin.y + 8.0);
    let [glyph] = packet.glyphs() else {
        panic!("one actual provider scalar, never an empty-input proxy");
    };
    assert_eq!(
        (
            glyph.range(),
            glyph.id(),
            glyph.x(),
            glyph.y(),
            glyph.advance()
        ),
        (Range::new(0, 1), 153, 0.0, 0.0, 10.0)
    );
    close(
        frame.origin().x + expected.scale * glyph.pen(),
        square.origin.x + expected.offset + expected.scale * expected.pen,
    );
}

fn case(orientation: LineOrientation) {
    let font = qualified_font_for(" ");
    let parsed = skrifa::FontRef::new(&font).unwrap();
    // Qualify the final exact font's horizontal metric-source choice: hhea is
    // nonzero and OS/2 does not request typographic metrics. No variation delta.
    assert_eq!(parsed.hhea().unwrap().ascender().to_i16(), 800);
    assert_eq!(parsed.hhea().unwrap().descender().to_i16(), -200);
    assert!(
        !parsed
            .os2()
            .unwrap()
            .fs_selection()
            .contains(SelectionFlags::USE_TYPO_METRICS)
    );
    assert!(parsed.fvar().is_err());
    assert!(parsed.mvar().is_err());
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(font.clone()).unwrap();
    assert_eq!(system.registered_font_faces().count(), 1);
    // All of these existing ordinary sideways/None premises execute before the
    // Cell guard. The signed Cell expectations below are not claimed as run RED.
    native_control_with_orientation(
        &mut system,
        &font,
        4.0,
        [0.0, 12.0, 24.0],
        34.0,
        orientation,
    );
    native_control_with_orientation(
        &mut system,
        &font,
        -4.0,
        [0.0, 8.0, 16.0],
        26.0,
        orientation,
    );
    no_cell_control(&mut system, &font, orientation, 4.0, 2.0, 14.0);
    no_cell_control(&mut system, &font, orientation, -2.0, -1.0, 8.0);
    let mut completed = Vec::new();
    for expected in CASES {
        let (processed, analyzed) = captured_with_context(
            " ",
            WhiteSpace::Preserve,
            expected.word,
            AnalysisTypography::HorizontalComposition,
            horizontal_reference(),
            TextCombineUpright::All,
        );
        assert_eq!(processed.source().text(), &Text::from(" "));
        assert_eq!(
            processed.captured_scope(1).unwrap().writing_axis(),
            Some(SourceWritingAxis::Vertical)
        );
        assert_eq!(
            analyzed.typography(),
            AnalysisTypography::HorizontalComposition
        );
        source_custody_with_reference(
            &processed,
            &analyzed,
            &[0],
            expected.word,
            horizontal_reference(),
        );
        let options = PreparationOptions::default()
            .with_orientation(orientation)
            .with_baseline_reference(FontBaseline::Alphabetic);
        assert_eq!(
            options.requested_baseline_reference(),
            Some(FontBaseline::Alphabetic)
        );
        let prepared = system.prepare(&analyzed, options)
            .expect("genuine vertical-writing/horizontal-typographic Cell must admit this sideways orientation");
        assert_eq!(prepared.orientation(), orientation);
        let cell = prepared.cell_views().next().unwrap();
        assert_eq!(
            (cell.extent(), cell.before(), cell.after()),
            (10.0, 8.0, 2.0)
        );
        assert!(cell.has_provider_input());
        let metrics = prepared
            .scope_metrics(&processed.captured_scope(1).unwrap())
            .unwrap();
        assert_eq!(metrics.font_reference(), horizontal_reference());
        assert_eq!(
            metrics.common_baseline_reference(),
            Some(FontBaseline::Alphabetic)
        );
        assert_eq!(metrics.orientation(), orientation);
        assert_eq!(metrics.upright_reference(), None);
        assert_eq!(metrics.upright_provenance(), None);
        assert_eq!(
            (
                metrics.over(),
                metrics.under(),
                metrics.native_reference_coordinate()
            ),
            (8.0, 2.0, 0.0)
        );
        assert_eq!(
            metrics.effective_instance().unwrap().face().bytes(),
            font.as_slice()
        );
        let line = complete_line(&prepared);
        output(&prepared, &line, &processed, &font, orientation, expected);
        completed.push((processed, prepared, line, expected));
    }
    drop(system);
    for (processed, prepared, line, expected) in completed {
        let replay = line.with_spacing(SpacingAllocations::None).unwrap();
        source_custody_with_reference(
            &processed,
            &replay.prepared_text().analyzed_source(),
            &[0],
            expected.word,
            horizontal_reference(),
        );
        output(&prepared, &replay, &processed, &font, orientation, expected);
    }
}

#[test]
fn clockwise_cells_retain_signed_inner_word_coverage_and_horizontal_packet_on_replay() {
    case(LineOrientation::SidewaysClockwise);
}

#[test]
fn counterclockwise_cells_retain_signed_inner_word_coverage_and_horizontal_packet_on_replay() {
    case(LineOrientation::SidewaysCounterclockwise);
}
