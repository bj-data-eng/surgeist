use super::*;
use crate::{LineHeight, Range, WordSpaceTransform};

fn separator_style(operation: WordSpaceTransform, mode: WhiteSpace) -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(10.0),
        white_space: mode,
        word_space_transform: operation,
        ..Style::default()
    }
}
fn catalogue() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::width_ahem())
        .unwrap();
    system
}
fn capture(source: Source, style: Style) -> PreparedText {
    let len = source.text().len();
    let a = AnalyzedSource::from_paragraph(
        ParagraphInput {
            source: source.clone(),
            base_style: style,
            paragraph_strut: ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
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
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        },
        crate::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    catalogue()
        .prepare(
            &crate::test_wrapping::bind_missing(&a, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
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
        .flat_map(|r| {
            r.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn genuine_separator_targets_use_real_metrics_ranges_and_original_extraction() {
    for (operation, id, width, total) in [
        (WordSpaceTransform::Space, 3, 5.0, 25.0),
        (WordSpaceTransform::IdeographicSpace, 1, 20.0, 40.0),
    ] {
        let source = Source::new("A\u{200b}A");
        let p = capture(
            source.clone(),
            separator_style(operation, WhiteSpace::Preserve),
        );
        let origin = p.origin_views().next().unwrap().handle();
        let line = whole(&p);
        assert_eq!(glyphs(&line), [(35, 10.0), (id, width), (35, 10.0)]);
        assert_eq!(line.feedback().advance, total);
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
        let selected = placed.position_selection(before, after).unwrap();
        assert_eq!(selected.source_text().code_units(), [0x200b]);
        assert_eq!(selected.plaintext().code_units(), [0x200b]);
        assert_eq!(selected.source_pieces().len(), 1);
        assert_eq!(
            selected.source_pieces()[0].aggregate_range(),
            Range::new(1, 2)
        );
        assert_eq!(
            selected.geometry().rects()[0].rect().size.width,
            f64::from(width)
        );
        assert_eq!(placed.source(), &source);
    }
}

#[test]
fn every_phase_i_mode_retains_generated_full_width_blank_metrics_without_coalescing() {
    for mode in [
        WhiteSpace::Preserve,
        WhiteSpace::Collapse,
        WhiteSpace::Discard,
        WhiteSpace::PreserveSpaces,
    ] {
        let p = capture(
            Source::new("A\u{200b}\u{200b}A"),
            Style {
                text_transform: crate::TextTransform::Set(
                    crate::TextTransformSet::none().with_full_width(true),
                ),
                ..separator_style(WordSpaceTransform::Space, mode)
            },
        );
        let line = whole(&p);
        assert_eq!(
            glyphs(&line),
            [(36, 20.0), (1, 20.0), (1, 20.0), (36, 20.0)],
            "{mode:?}"
        );
        assert_eq!(line.feedback().advance, 80.0);
    }
}

#[test]
fn separator_style_cache_identity_distinguishes_warm_and_fresh_raw_results() {
    let mut warm = catalogue();
    let source = Source::identified("A\u{200b}A", crate::Id::from_u64(6311), 3);
    for (operation, advance) in [
        (WordSpaceTransform::None, 20.0),
        (WordSpaceTransform::Space, 25.0),
        (WordSpaceTransform::IdeographicSpace, 40.0),
        (WordSpaceTransform::None, 20.0),
    ] {
        let style = separator_style(operation, WhiteSpace::Preserve);
        let options = crate::Options {
            quantize: false,
            ..crate::Options::default()
        };
        let actual = warm.layout(source.clone(), style.clone(), options).unwrap();
        let fresh = catalogue().layout(source.clone(), style, options).unwrap();
        let observe = |l: &crate::Layout| {
            l.run_views()
                .flat_map(|r| {
                    r.glyphs()
                        .map(|g| (g.id(), g.advance()))
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(observe(&actual), observe(&fresh));
        assert_eq!(actual.lines()[0].bounds().size.width, advance);
        assert_eq!(actual.source(), &source);
    }
}

#[test]
fn generated_collapse_space_still_uses_real_phase_two_line_edge_discard() {
    let p = capture(
        Source::new("A\u{200b}"),
        separator_style(WordSpaceTransform::Space, WhiteSpace::Collapse),
    );
    assert_eq!(p.analyzed_source().inner.projection.text, "A ");
    let line = whole(&p);
    assert_eq!(line.feedback().advance, 10.0);
    // The original separator remains source, while Phase II removes its
    // actual generated collapsible SPACE from the line-edge contribution.
    let placed = place(p, line, 0.0, 8.0).unwrap();
    assert_eq!(placed.source().text().code_units(), [0x41, 0x200b]);
}

#[test]
#[cfg(feature = "text-accessibility")]
fn separator_accessibility_uses_original_zwsp_and_actual_generated_geometry() {
    let p = capture(
        Source::new("A\u{200b}A"),
        separator_style(WordSpaceTransform::Space, WhiteSpace::Preserve),
    );
    let line = whole(&p);
    assert_eq!(line.feedback().advance, 25.0);
    let placed = place(p, line, 0.0, 8.0).unwrap();
    let mut state = crate::Accessibility::new(63);
    let parent = 1001_u64.into();
    let update = placed
        .accessibility_update(&mut state, parent, parent, crate::Point::new(0.0, 0.0))
        .unwrap();
    let descriptors: Vec<_> = update
        .nodes
        .iter()
        .filter_map(|(_, node)| {
            node.value().map(|value| {
                let bounds = node.bounds().unwrap();
                (
                    value,
                    node.character_lengths(),
                    (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
                )
            })
        })
        .collect();
    // AX retains original host-EGC descriptors, rather than joining them
    // merely because their target glyphs share a placed line.
    assert_eq!(
        descriptors,
        [
            ("A", &[1][..], (0.0, 0.0, 10.0, 10.0)),
            ("\u{200b}", &[3][..], (10.0, 0.0, 15.0, 10.0)),
            ("A", &[1][..], (15.0, 0.0, 25.0, 10.0)),
        ]
    );
    // The parent keeps the actual allocated line box from request(), while
    // the three source descriptors above retain the 25px text extent.
    let parent_node = &update.nodes.iter().find(|(id, _)| *id == parent).unwrap().1;
    let bounds = parent_node.bounds().unwrap();
    assert_eq!(
        (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
        (0.0, 0.0, 200.0, 10.0)
    );
}

#[test]
fn generated_discard_separators_keep_ordinary_trailing_advance_and_original_zwsp() {
    for (operation, full_width, letter, separator, separator_advance, total, presentation) in [
        (WordSpaceTransform::Space, false, 35, 3, 5.0, 15.0, "A "),
        (
            WordSpaceTransform::IdeographicSpace,
            false,
            35,
            1,
            20.0,
            30.0,
            "A\u{3000}",
        ),
        (
            WordSpaceTransform::Space,
            true,
            36,
            1,
            20.0,
            40.0,
            "\u{ff21}\u{3000}",
        ),
    ] {
        let source = Source::identified("A\u{200b}", crate::Id::from_u64(7402), 8);
        let p = capture(
            source.clone(),
            Style {
                text_transform: crate::TextTransform::Set(
                    crate::TextTransformSet::none().with_full_width(full_width),
                ),
                ..separator_style(operation, WhiteSpace::Discard)
            },
        );
        assert_eq!(
            p.analyzed_source()
                .processed_source()
                .processed_text()
                .code_units(),
            [0x41, 0x200b]
        );
        assert_eq!(p.analyzed_source().inner.projection.text, presentation);
        let mut r = request(&p);
        r.context.available_inline = Some(10.0);
        let line = p
            .select(r)
            .unwrap()
            .finalize(crate::LineAllocations {
                spacing: SpacingAllocations::None,
                scopes: crate::ScopeAllocations::None,
            })
            .unwrap();
        let letter_advance = if full_width { 20.0 } else { 10.0 };
        assert_eq!(
            glyphs(&line),
            [(letter, letter_advance), (separator, separator_advance)]
        );
        assert_eq!(line.feedback().advance, total);
        assert_eq!(
            (line.feedback().hanging_start, line.feedback().hanging_end),
            (0.0, 0.0)
        );
        for measure in [
            line.measurements().fit(),
            line.measurements().alignment(),
            line.measurements().min_content_contribution(),
            line.measurements().max_content_contribution(),
        ] {
            assert_eq!(
                (
                    measure.advance(),
                    measure.occupied_start(),
                    measure.occupied_end()
                ),
                (total, 0.0, total)
            );
        }
        assert!(
            line.edge_units()
                .all(|unit| unit.disposition() == crate::LineEdgeDisposition::Ordinary)
        );
        let original_rows: Vec<_> = line
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.face().bytes(), crate::shaping_test_fonts::width_ahem());
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                run.glyphs().map(|glyph| glyph.range()).collect::<Vec<_>>()
            })
            .collect();
        assert_eq!(original_rows, [Range::new(0, 1), Range::new(1, 2)]);
        let origin = p.origin_views().next().unwrap().handle();
        let placed = place(p, line, 0.0, 8.0).unwrap();
        assert_eq!(placed.source(), &source);
        let units = placed
            .source_range_fragments(&origin, Range::new(1, 2))
            .unwrap();
        assert_eq!(units.units().len(), 1);
        assert_eq!(placed.source().text().code_units(), [0x41, 0x200b]);
        assert_eq!(placed.source().revision(), 8);
    }
}
