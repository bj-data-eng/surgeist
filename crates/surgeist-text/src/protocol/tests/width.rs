use super::*;
use crate::{CaseTransform, LineHeight, Range, TextTransform};

fn width_style(transform: TextTransform, white_space: WhiteSpace) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space,
        text_transform: transform,
        overflow_wrap: crate::OverflowWrap::Anywhere,
        ..Style::default()
    }
}

fn capture(source: Source, style: Style) -> PreparedText {
    capture_with_breaks(source, style, crate::DocumentSegmentBreaks::LineFeedDefault)
}

fn capture_with_breaks(
    source: Source,
    style: Style,
    segment_breaks: crate::DocumentSegmentBreaks,
) -> PreparedText {
    let len = source.text().len();
    let analyzed = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: style,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks,
            base_direction: ParagraphBaseDirection::LeftToRight,
            origins: vec![paragraph::ParagraphOrigin {
                text_node: None,
                source,
                local: Range::new(0, len),
                aggregate: Range::new(0, len),
                kind: paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    system
        .prepare(
            &crate::test_wrapping::bind_missing(&analyzed, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}

#[test]
fn preserved_document_delimiter_maps_one_blank_unit_but_retains_both_original_units() {
    let source = Source::new("A\r\nA");
    let p = capture_with_breaks(
        source.clone(),
        width_style(
            TextTransform::Set(crate::TextTransformSet::none().with_full_width(true)),
            WhiteSpace::PreserveSpaces,
        ),
        crate::DocumentSegmentBreaks::Defined(vec![Range::new(1, 3)]),
    );
    assert_eq!(
        p.analyzed_source().processed_source().processed_text(),
        &crate::Text::from("A A")
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "Ａ\u{3000}Ａ");
    let middle = &p.analyzed_source().inner.projection.atoms[1];
    assert_eq!(middle.source, Range::new(1, 3));
    assert_eq!(middle.processed, Range::new(1, 2));
    assert_eq!(middle.presentation, Range::new(1, 2));
    let origin = p.origin_views().next().unwrap().handle();
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(36, 20.0), (1, 20.0), (36, 20.0)]);
    assert_eq!(line.feedback().advance, 60.0);
    let placed = place(p, line, 0.0, 8.0).unwrap();
    for range in [Range::new(1, 2), Range::new(2, 3), Range::new(1, 3)] {
        assert_eq!(
            placed
                .source_range_fragments(&origin, range)
                .unwrap()
                .units()
                .len(),
            1
        );
    }
    let positions: Vec<_> = placed.positions().collect();
    let before = positions
        .iter()
        .find(|p| p.source_cursor() == crate::Cursor::new(1, crate::Affinity::After))
        .unwrap();
    let after = positions
        .iter()
        .find(|p| p.source_cursor() == crate::Cursor::new(3, crate::Affinity::Before))
        .unwrap();
    let selected = placed.position_selection(before, after).unwrap();
    assert_eq!(selected.source_text().code_units(), &[0x0d, 0x0a]);
    assert_eq!(selected.plaintext().code_units(), &[0x20]);
    assert_eq!(selected.geometry().rects()[0].rect().size.width, 20.0);
    assert_eq!(placed.source(), &source);
}

fn whole(p: &PreparedText) -> FinalizedLine {
    p.select(request(p))
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap()
}

fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| (glyph.id(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn width_uses_actual_target_fonts_metrics_and_canonical_shaping_after_raw_mapping() {
    let width_set = crate::TextTransformSet::none().with_full_width(true);
    let width = TextTransform::Set(width_set);
    for (input, transform, semantic, expected, advance) in [
        ("A", TextTransform::none(), "A", vec![(35, 10.0)], 10.0),
        ("A", width, "Ａ", vec![(36, 20.0)], 20.0),
        (
            "ß",
            TextTransform::Set(width_set.with_case(CaseTransform::Uppercase)),
            "ＳＳ",
            vec![(58, 15.0), (58, 15.0)],
            30.0,
        ),
        (
            "ß",
            TextTransform::Set(width_set.with_case(CaseTransform::Capitalize)),
            "Ｓｓ",
            vec![(58, 15.0), (86, 12.5)],
            27.5,
        ),
        ("ｧ", width, "ァ", vec![(67, 8.0)], 8.0),
        (
            "ｧ",
            TextTransform::Set(width_set.with_full_size_kana(true)),
            "ア",
            vec![(36, 20.0)],
            20.0,
        ),
        ("ｶﾞ", width, "カ\u{3099}", vec![(37, 18.0)], 18.0),
        ("\u{ffa0}", width, "\u{3164}", vec![(70, 17.0)], 17.0),
        ("\u{ffa1}", width, "\u{3131}", vec![(71, 19.0)], 19.0),
        ("!", width, "！", vec![(72, 14.0)], 14.0),
        ("¢", width, "￠", vec![(73, 16.0)], 16.0),
    ] {
        let source = Source::new(input);
        let p = capture(source.clone(), width_style(transform, WhiteSpace::Preserve));
        assert_eq!(p.analyzed_source().inner.projection.text, semantic);
        assert_eq!(
            p.analyzed_source().processed_source().processed_text(),
            source.text()
        );
        let line = whole(&p);
        assert_eq!(glyphs(&line), expected, "{input:?}/{transform:?}");
        assert_eq!(line.feedback().advance, advance);
        let bytes = crate::shaping_test_fonts::width_ahem();
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), bytes.as_slice());
            assert_eq!(run.font_size(), 10.0);
        }
        let placed = place(p, line, 0.0, 8.0).unwrap();
        assert_eq!(placed.source(), &source);
        let positions: Vec<_> = placed.positions().collect();
        assert_eq!(
            placed
                .position_selection(positions.first().unwrap(), positions.last().unwrap())
                .unwrap()
                .source_text(),
            source.text()
        );
    }
}

#[test]
fn actual_preserved_space_range_and_exact_selection_use_blank_twenty_pixel_target() {
    let width = TextTransform::Set(crate::TextTransformSet::none().with_full_width(true));
    for (mode, expected, advance) in [
        (
            WhiteSpace::Collapse,
            vec![(36, 20.0), (3, 5.0), (36, 20.0)],
            45.0,
        ),
        (
            WhiteSpace::Preserve,
            vec![(36, 20.0), (1, 20.0), (36, 20.0)],
            60.0,
        ),
    ] {
        let source = Source::new("A A");
        let p = capture(source.clone(), width_style(width, mode));
        let origin = p.origin_views().next().unwrap().handle();
        let line = whole(&p);
        assert_eq!(glyphs(&line), expected);
        assert_eq!(line.feedback().advance, advance);
        let placed = place(p, line, 0.0, 8.0).unwrap();
        let query = placed
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap();
        assert_eq!(query.units().len(), 1);
        let positions: Vec<_> = placed.positions().collect();
        let before = positions
            .iter()
            .find(|p| p.source_cursor() == crate::Cursor::new(1, crate::Affinity::After))
            .unwrap();
        let after = positions
            .iter()
            .find(|p| p.source_cursor() == crate::Cursor::new(2, crate::Affinity::Before))
            .unwrap();
        let selection = placed.position_selection(before, after).unwrap();
        assert_eq!(selection.source_text().code_units(), &[0x20]);
        assert_eq!(selection.plaintext().code_units(), &[0x20]);
        assert_eq!(
            selection.geometry().rects()[0].rect().size.width,
            advance - 40.0
        );
        assert_eq!(
            placed.position_geometry(before).unwrap().rect().origin.x,
            20.0
        );
        assert_eq!(
            placed.position_geometry(after).unwrap().rect().origin.x,
            advance - 20.0
        );
        assert_eq!(placed.source(), &source);
    }
}

#[test]
fn width_style_hashing_distinguishes_warm_and_fresh_raw_layout_results() {
    let mut warm = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    warm.register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    let source = Source::identified("A A", crate::Id::from_u64(6321), 2);
    for (enabled, expected, advance) in [
        (false, vec![(35, 10.0), (3, 5.0), (35, 10.0)], 25.0),
        (true, vec![(36, 20.0), (1, 20.0), (36, 20.0)], 60.0),
        (false, vec![(35, 10.0), (3, 5.0), (35, 10.0)], 25.0),
    ] {
        let style = width_style(
            TextTransform::Set(crate::TextTransformSet::none().with_full_width(enabled)),
            WhiteSpace::Preserve,
        );
        let options = crate::Options {
            quantize: false,
            ..crate::Options::default()
        };
        let actual = warm.layout(source.clone(), style.clone(), options).unwrap();
        let mut fresh = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        fresh
            .register_fonts(crate::shaping_test_fonts::width_ahem())
            .unwrap();
        let other = fresh.layout(source.clone(), style, options).unwrap();
        let observe = |layout: &crate::Layout| {
            layout
                .run_views()
                .flat_map(|run| {
                    run.glyphs()
                        .map(|g| (g.id(), g.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(observe(&actual), expected);
        assert_eq!(observe(&actual), observe(&other));
        assert_eq!(actual.lines()[0].bounds().size.width, advance);
        assert_eq!(actual.source(), &source);
    }
}

#[test]
fn fullwidth_semantics_create_real_normal_opportunities_while_narrow_letters_need_conditional_permission()
 {
    let neutral = capture(
        Source::new("AA"),
        width_style(TextTransform::none(), WhiteSpace::Preserve),
    );
    assert!(
        !neutral
            .opportunity_views()
            .any(|row| row.kind() == PreparedBreakKind::Allowed
                && row.boundary().view().participant_offset() == 1)
    );
    let query = request(&neutral);
    let PreparedTermination::Ordinary(ceiling) = query.termination else {
        unreachable!()
    };
    let conditional = neutral
        .source_segments()
        .next()
        .unwrap()
        .conditional_search(
            &query.start,
            None,
            &ceiling,
            ConditionalPurpose::Overflow,
            &[],
            crate::ConditionalLineContext::Unframed { inline_boxes: None },
        )
        .and_then(|search| search.begin(&[]))
        .map(|batch| batch.opportunities().to_vec())
        .unwrap();
    assert!(
        conditional
            .iter()
            .any(|offer| offer.boundary().view().participant_offset() == 1)
    );
    let wide = capture(
        Source::new("AA"),
        width_style(
            TextTransform::Set(crate::TextTransformSet::none().with_full_width(true)),
            WhiteSpace::Preserve,
        ),
    );
    assert_eq!(wide.analyzed_source().inner.projection.text, "ＡＡ");
    let offer = wide
        .opportunity_views()
        .find(|row| {
            row.kind() == PreparedBreakKind::Allowed
                && row.boundary().view().participant_offset() == 1
        })
        .unwrap()
        .handle();
    let mut first = request(&wide);
    first.end = offer.view().boundary();
    first.termination = PreparedTermination::Ordinary(offer);
    first.participants.truncate(1);
    first.context.end = LineEnd::Soft;
    first.context.available_inline = Some(20.0);
    let finalized = wide
        .select(first)
        .unwrap()
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(glyphs(&finalized), [(36, 20.0)]);
    assert_eq!(finalized.feedback().advance, 20.0);
}

#[test]
#[cfg(feature = "text-accessibility")]
fn width_accessibility_keeps_original_source_and_actual_twenty_pixel_geometry() {
    let p = capture(
        Source::new("A"),
        width_style(
            TextTransform::Set(crate::TextTransformSet::none().with_full_width(true)),
            WhiteSpace::Preserve,
        ),
    );
    let line = whole(&p);
    assert_eq!(glyphs(&line), [(36, 20.0)]);
    let placed = place(p, line, 0.0, 8.0).unwrap();
    let mut state = crate::Accessibility::new(21);
    let parent = 1001_u64.into();
    let update = placed
        .accessibility_update(&mut state, parent, parent, crate::Point::new(0.0, 0.0))
        .unwrap();
    let at = placed
        .access_position(&state, crate::Cursor::new(0, crate::Affinity::After))
        .unwrap()
        .unwrap();
    let node = &update
        .nodes
        .iter()
        .find(|(id, _)| *id == at.node)
        .unwrap()
        .1;
    assert_eq!(node.value(), Some("A"));
    assert_eq!(node.character_lengths(), [1]);
    let bounds = node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (0.0, 0.0, 20.0, 10.0)
    );
}
