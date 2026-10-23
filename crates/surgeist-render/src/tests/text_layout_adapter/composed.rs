//! Explicit selected-line placement reaches the existing Render owner directly.
//! These are new phase-API functional tests, not historical regression RED.

use super::*;

#[path = "composed_fonts.rs"]
mod fonts;

fn finalized(source: text::Source) -> (text::PreparedText, text::FinalizedLine) {
    let mut system = system(AHEM.to_vec());
    let analyzed = text::AnalyzedSource::new(source, style()).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let termination = prepared
        .opportunity_views()
        .find(|opportunity| opportunity.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let selected = prepared
        .select(text::LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: text::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                // The external selector deliberately retains this overflowing
                // line. Render cannot run a width-based paragraph wrapper.
                available_inline: Some(5.0),
                indent: 0.0,
                alignment: text::Alignment::Left,
                tabs: text::TabContext::NoTabs,
                hanging: text::HangingContext::None,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap();
    let stats = system.stats();
    let line = selected.finalize(text::SpacingAllocations::None).unwrap();
    assert_eq!(line.feedback().advance, 40.0);
    assert_eq!(system.stats(), stats);
    // Returned preparations/finalized rows own the actual font resources.
    drop(system);
    (prepared, line)
}

fn placed(
    prepared: text::PreparedText,
    line: text::FinalizedLine,
    x: f64,
    baseline: f64,
) -> text::Layout {
    text::Layout::from_finalized_lines(text::PlacementRequest {
        prepared,
        coverage: text::PlacementCoverage::WholeParagraph,
        lines: vec![text::LinePlacement {
            line,
            anchor: text::LineAnchor {
                inline_origin: x,
                baseline,
            },
            orientation: text::LineOrientation::Horizontal,
            atomics: vec![],
        }],
    })
    .unwrap()
}

#[test]
fn explicit_placement_keeps_selected_glyphs_and_fractional_outer_origin() {
    let (prepared, line) = finalized(text::Source::new("AX"));
    let x = 2.0_f64.powi(24) + 0.25;
    let baseline = 2.0_f64.powi(24) + 0.5;
    let input = placed(prepared.clone(), line.clone(), x, baseline);
    let scene =
        prepare_text_layout(&input, Transform::translation(7.0, 9.0).unwrap(), limits()).unwrap();
    assert!(std::ptr::eq(scene.layout(), &input));
    assert_eq!(input.lines().len(), 1);
    assert_eq!(input.lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(input.lines()[0].baseline(), baseline);
    assert_eq!(scene.scene().commands.len(), 1);
    let (paint, transform) = outline(&scene.scene().commands[0]);
    assert_eq!(
        transform,
        Transform::translation(x + 7.0, baseline + 9.0).unwrap()
    );
    assert_eq!(paint.font().bytes(), AHEM);
    assert_eq!(paint.instance().size(), 20.0);
    assert_eq!(
        paint.glyphs(),
        [
            TextGlyph::try_new(35, 0.0, 0.0, 20.0).unwrap(),
            TextGlyph::try_new(58, 20.0, 0.0, 20.0).unwrap(),
        ]
    );
    assert_eq!(
        paint.bounds().ink_rect(),
        Some(Rect::try_new(0.0, -16.0, 40.0, 20.0).unwrap())
    );
    let cursor = input
        .cursor(text::Cursor::new(1, text::Affinity::After))
        .unwrap()
        .rect();
    assert_eq!(cursor.origin, text::Point::new(x + 20.0, baseline - 16.0));
    assert_eq!(cursor.size.height, 20.0);

    // Replacing only the physical anchor keeps the exact finalized owner and
    // local glyph facts while changing the emitted outer transform once.
    let translated = placed(prepared, line, 3.25, 41.5);
    let translated_scene =
        prepare_text_layout(&translated, Transform::identity(), limits()).unwrap();
    let (translated_paint, transform) = outline(&translated_scene.scene().commands[0]);
    assert_eq!(transform, Transform::translation(3.25, 41.5).unwrap());
    assert_eq!(translated_paint.glyphs(), paint.glyphs());
}

#[test]
fn placed_line_refuses_a_backend_transform_that_collapses_positive_ink() {
    let (prepared, line) = finalized(text::Source::new("AX"));
    let input = placed(prepared, line, 2.0_f64.powi(40), 2.0_f64.powi(40));
    // The logical Text placement is valid. At this magnitude the backend's
    // f32 affine collapses the actual 40x20 outline; it must refuse paint.
    let error = refuse(&input, Transform::identity(), limits());
    let cause = item_render(&error, TextLayoutLocation::Run { line: 0, run: 0 });
    assert!(matches!(
        cause.glyph_preparation_diagnostic(),
        Some(GlyphPreparationError::Numeric {
            phase: GlyphNumericPhase::Placement,
            value: 0.0,
        })
    ));
    assert_eq!(input.lines().len(), 1);
    assert_eq!(input.lines()[0].bounds().size.width, 5.0);
    assert_eq!(
        input
            .run_views()
            .map(|run| {
                run.glyphs()
                    .map(|glyph| f64::from(glyph.advance()))
                    .sum::<f64>()
            })
            .sum::<f64>(),
        40.0
    );
}

#[test]
fn aggregate_placed_run_refusal_preserves_an_already_admitted_scene() {
    let (prepared, line) = finalized(split_source());
    let input = placed(prepared, line, 10.25, 30.5);
    let accepted = prepare_text_layout(&input, Transform::identity(), limits()).unwrap();
    assert_eq!(accepted.scene().commands.len(), 2);
    let before: Vec<_> = accepted
        .scene()
        .commands
        .iter()
        .map(|command| {
            let (outline, transform) = outline(command);
            (outline.glyphs().to_vec(), transform)
        })
        .collect();
    let error = refuse(
        &input,
        Transform::identity(),
        bounded(1, 1024, ROOM, 32, 32, 32),
    );
    assert_limit(&error, TextLayoutResource::Glyphs, 2, 1);
    assert_eq!(input.lines().len(), 1);
    for (command, (glyphs, expected_transform)) in accepted.scene().commands.iter().zip(before) {
        let (outline, transform) = outline(command);
        assert_eq!(outline.glyphs(), glyphs);
        assert_eq!(transform, expected_transform);
    }
}

#[test]
fn actual_gpos_mark_placement_reaches_admitted_outline_ink_without_advance_substitution() {
    let bytes = fonts::mark_ahem();
    let mut system = system(bytes.clone());
    let analyzed = text::AnalyzedSource::new(
        text::Source::new("A\u{301}"),
        text::Style {
            size: 12.5,
            line_height: text::LineHeight::Absolute(16.0),
            ..style()
        },
    )
    .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let termination = prepared
        .opportunity_views()
        .find(|opportunity| opportunity.kind() == text::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let finalized = prepared
        .select(text::LineSelectionRequest {
            start: prepared.boundary_views().next().unwrap().handle(),
            end: termination.view().boundary(),
            termination: text::PreparedTermination::Ordinary(termination),
            alternative: None,
            participants: prepared.participants().collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
            context: text::LineContext {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                orientation: text::LineOrientation::Horizontal,
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(100.0),
                indent: 0.0,
                alignment: text::Alignment::Left,
                tabs: text::TabContext::NoTabs,
                hanging: text::HangingContext::None,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::SpacingAllocations::None)
        .unwrap();
    assert_eq!(finalized.feedback().advance, 12.5);
    let input = placed(prepared, finalized, 3.25, 41.5);
    assert_eq!(input.source().text().code_units(), &[0x41, 0x301]);
    assert_eq!(input.lines().len(), 1);
    assert_eq!(input.lines()[0].range(), text::Range::new(0, 2));
    assert_eq!(input.lines()[0].baseline(), 41.5);
    assert_eq!(input.lines()[0].bounds().size.width, 100.0);
    let mut runs = input.run_views();
    let run = runs.next().unwrap();
    assert!(runs.next().is_none());
    assert_eq!(run.range(), text::Range::new(0, 2));
    assert_eq!(run.origin(), text::Point::new(3.25, 41.5));
    let instance = run.effective_instance().unwrap();
    assert!(instance.face().is_same_face(&run.face()));
    assert_eq!(instance.face().bytes(), bytes);
    assert_eq!(instance.font_size(), 12.5);
    assert_eq!(instance.axis_count(), 0);
    assert_eq!(instance.x_shear(), 0.0);
    assert_eq!(instance.embolden(), 0.0);
    // UPEM 1000 anchors (400,600)-(100,200) give local mark (3.75,-5).
    // Its pen follows the base's 12.5 advance, so the raw x offset is -8.75;
    // GDEF mark classification supplies zero mark advance independently of ink.
    assert_eq!(
        run.glyphs()
            .map(|glyph| (
                glyph.id(),
                glyph.pen(),
                glyph.x(),
                glyph.y(),
                glyph.advance()
            ))
            .collect::<Vec<_>>(),
        [(35, 0.0, 0.0, 0.0, 12.5), (100, 12.5, -8.75, -5.0, 0.0),]
    );
    let stats = system.stats();
    let placement = Transform::try_new([2.0, 0.0, 0.0, 3.0, 7.0, 9.0]).unwrap();
    let scene = prepare_text_layout(&input, placement, limits()).unwrap();
    assert!(std::ptr::eq(scene.layout(), &input));
    assert_eq!(system.stats(), stats);
    assert_eq!(scene.scene().commands.len(), 1);
    let (paint, transform) = outline(&scene.scene().commands[0]);
    // The already-placed run origin remains one outer transform, after local
    // GPOS geometry. No per-glyph split, second shaping, or inferred font occurs.
    assert_eq!(
        transform,
        Transform::try_new([2.0, 0.0, 0.0, 3.0, 13.5, 133.5]).unwrap()
    );
    assert_eq!(paint.font().bytes(), instance.face().bytes());
    assert_eq!(paint.font().index(), instance.face().index());
    assert_eq!(paint.instance().size(), 12.5);
    assert!(paint.instance().coordinates().is_empty());
    assert_eq!(paint.instance().synthesis(), GlyphSynthesis::none());
    assert_eq!(
        paint.glyphs(),
        [
            TextGlyph::try_new(35, 0.0, 0.0, 12.5).unwrap(),
            TextGlyph::try_new(100, 3.75, -5.0, 0.0).unwrap(),
        ]
    );
    // Unchanged Ahem glyph 35 has corners x=0..1000/y=-200..800;
    // glyph 100 has x=0..1000/y=0..800. At 12.5px after y reflection, base
    // ink is [0,-10,12.5,12.5], mark ink [3.75,-15,12.5,10]. Their actual
    // union overhangs the 12.5 advance on the right and above the base.
    assert_eq!(
        paint.bounds().ink_rect(),
        Some(Rect::try_new(0.0, -15.0, 16.25, 17.5).unwrap())
    );
    assert_eq!(
        paint
            .path_elements()
            .filter(|element| matches!(element, crate::PathElement::Close))
            .count(),
        2
    );
}
