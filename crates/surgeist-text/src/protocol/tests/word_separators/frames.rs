//! Authored separator halves follow the actual visual inline frame, while an
//! explicitly used Alphabetic reference retains its independent cross frame.
use super::*;
use crate::{
    FontBaselineAxis, FontBaselineOutcome, FontBaselineProvenance, baseline_test_fonts as base,
};

fn qualify(bytes: &[u8], mappings: &[(char, u32)]) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert!(font.table_data(skrifa::Tag::new(b"fvar")).is_none());
    assert!(font.table_data(skrifa::Tag::new(b"vhea")).is_none());
    assert!(font.table_data(skrifa::Tag::new(b"vmtx")).is_none());
    for &(scalar, glyph) in mappings {
        assert_eq!(font.charmap().map(scalar), Some(GlyphId::new(glyph)));
    }
    for tag in [*b"glyf", *b"loca", *b"hhea", *b"OS/2"] {
        assert_eq!(sfnt::table(bytes, tag), sfnt::table(AHEM, tag));
    }
    let hhea = font.hhea().unwrap();
    assert_eq!(
        (
            hhea.ascender().to_i16(),
            hhea.descender().to_i16(),
            hhea.line_gap().to_i16()
        ),
        (800, -200, 0)
    );
    assert_eq!(sfnt::u16_at(sfnt::table(bytes, *b"OS/2"), 62) & 0x80, 0);
    let metrics = font.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for glyph in [35, 36, 58] {
        assert_eq!(metrics.advance_width(GlyphId::new(glyph)), Some(1000.0));
        let bounds = metrics.bounds(GlyphId::new(glyph)).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            [0.0, -200.0, 1000.0, 800.0]
        );
    }
    // The real enabled ccmp ligature requires82+82. Its coverage and second
    // component do not match any actual35/36/58 glyph in these witnesses.
    // No optional-feature switch or fabricated independent advance is used.
    let gsub = sfnt::table(bytes, *b"GSUB");
    let features = usize::from(sfnt::u16_at(gsub, 6));
    assert_eq!(sfnt::u16_at(gsub, features), 1);
    assert_eq!(&gsub[features + 2..features + 6], b"ccmp");
    let feature = features + usize::from(sfnt::u16_at(gsub, features + 6));
    assert_eq!(
        (
            sfnt::u16_at(gsub, feature + 2),
            sfnt::u16_at(gsub, feature + 4)
        ),
        (1, 0)
    );
    let lookups = usize::from(sfnt::u16_at(gsub, 8));
    assert_eq!(sfnt::u16_at(gsub, lookups), 1);
    let lookup = lookups + usize::from(sfnt::u16_at(gsub, lookups + 2));
    assert_eq!(
        (sfnt::u16_at(gsub, lookup), sfnt::u16_at(gsub, lookup + 4)),
        (4, 1)
    );
    let ligatures = lookup + usize::from(sfnt::u16_at(gsub, lookup + 6));
    assert_eq!(
        (
            sfnt::u16_at(gsub, ligatures),
            sfnt::u16_at(gsub, ligatures + 4)
        ),
        (1, 1)
    );
    let coverage = ligatures + usize::from(sfnt::u16_at(gsub, ligatures + 2));
    assert_eq!(
        (
            sfnt::u16_at(gsub, coverage),
            sfnt::u16_at(gsub, coverage + 2),
            sfnt::u16_at(gsub, coverage + 4)
        ),
        (1, 1, 82)
    );
    let set = ligatures + usize::from(sfnt::u16_at(gsub, ligatures + 6));
    assert_eq!(sfnt::u16_at(gsub, set), 1);
    let ligature = set + usize::from(sfnt::u16_at(gsub, set + 2));
    assert_eq!(
        (
            sfnt::u16_at(gsub, ligature),
            sfnt::u16_at(gsub, ligature + 2),
            sfnt::u16_at(gsub, ligature + 4)
        ),
        (58, 2, 82)
    );
}

fn prepare_profile(
    system: &mut System,
    source: &Source,
    orientation: LineOrientation,
    spacing: f64,
    direction: ParagraphBaseDirection,
    alphabetic: bool,
) -> PreparedText {
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: Style {
                    font: Font::new().family("Ahem"),
                    size: 10.0,
                    line_height: LineHeight::Absolute(10.0),
                    word_spacing: spacing,
                    white_space: WhiteSpace::Preserve,
                    ..Style::default()
                },
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: direction,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(0, 3),
                    aggregate: Range::new(0, 3),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    let options = PreparationOptions::default().with_orientation(orientation);
    let options = if alphabetic {
        options.with_baseline_reference(FontBaseline::Alphabetic)
    } else {
        options
    };
    let prepared = system.prepare(&analysis, options).unwrap();
    assert_eq!(prepared.analyzed_source().identity(), analysis.identity());
    assert_eq!(prepared.analyzed_source().source(), source);
    assert_eq!(
        prepared.baseline_reference(),
        alphabetic.then_some(FontBaseline::Alphabetic)
    );
    prepared
}

fn rtl_line(prepared: &PreparedText) -> FinalizedLine {
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.base_direction = ParagraphBaseDirection::RightToLeft;
    selection.context.line_box_direction = crate::LineBoxDirection::RightToLeft;
    selection.context.available_inline = None;
    selection.context.spacing = SpacingPolicy::None;
    prepared
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

fn placed(prepared: &PreparedText, line: FinalizedLine) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: if prepared.orientation() == LineOrientation::Horizontal {
                    70.0
                } else {
                    100.0
                },
            },
            orientation: prepared.orientation(),
            atomics: vec![],
        })],
    })
    .unwrap()
}

fn physical_glyphs(layout: &crate::Layout) -> Vec<(Range, Point)> {
    layout
        .glyph_runs()
        .iter()
        .flat_map(|run| {
            let origin = run.origin();
            run.glyphs()
                .iter()
                .map(|glyph| {
                    let point = match run.orientation() {
                        LineOrientation::Horizontal => Point::new(
                            origin.x + glyph.pen() + f64::from(glyph.x()),
                            origin.y + f64::from(glyph.y()),
                        ),
                        LineOrientation::SidewaysClockwise => Point::new(
                            origin.x - f64::from(glyph.y()),
                            origin.y + glyph.pen() + f64::from(glyph.x()),
                        ),
                        LineOrientation::Vertical => Point::new(
                            origin.x + f64::from(glyph.x()),
                            origin.y + glyph.pen() + f64::from(glyph.y()),
                        ),
                    };
                    (glyph.range(), point)
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_custody(
    layout: &crate::Layout,
    prepared: &PreparedText,
    source: &Source,
    inline_carets: [f64; 4],
    low: f64,
) {
    assert_eq!(layout.source(), source);
    let origins = prepared.origin_views().collect::<Vec<_>>();
    assert_eq!(origins.len(), 1);
    assert_eq!(origins[0].source(), source);
    assert_eq!(origins[0].local_range(), Range::new(0, 3));
    let horizontal = prepared.orientation() == LineOrientation::Horizontal;
    for (index, inline) in inline_carets.into_iter().enumerate() {
        let cursor = Cursor::new(
            index,
            if index == 3 {
                Affinity::Before
            } else {
                Affinity::After
            },
        );
        assert_eq!(
            layout.cursor(cursor).unwrap().rect(),
            if horizontal {
                Rect::new(20.0 + inline, low, 1.0, 10.0)
            } else {
                Rect::new(low, 20.0 + inline, 10.0, 1.0)
            }
        );
    }
    let first = layout
        .positions()
        .find(|position| position.source_cursor() == Cursor::new(0, Affinity::After))
        .unwrap();
    let last = layout
        .positions()
        .find(|position| position.source_cursor() == Cursor::new(3, Affinity::Before))
        .unwrap();
    let selection = layout.position_selection(&first, &last).unwrap();
    assert_eq!(
        selection.source_text().code_units(),
        source.text().code_units()
    );
    assert_eq!(
        selection.plaintext().code_units(),
        source.text().code_units()
    );
    let pieces = layout
        .source_range_fragments(&origins[0].handle(), Range::new(1, 2))
        .unwrap();
    assert_eq!(
        pieces
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [if horizontal {
            Rect::new(30.0, low, 14.0, 10.0)
        } else {
            Rect::new(low, 30.0, 10.0, 14.0)
        }]
    );
}

#[test]
fn odd_rtl_separator_paints_between_equal_authored_halves_in_horizontal_and_clockwise_frames() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0xa0, 58), (0x5d0, 35), (0x5d1, 36)],
    );
    qualify(
        &bytes,
        &[('A', 35), ('\u{a0}', 58), ('\u{5d0}', 35), ('\u{5d1}', 36)],
    );
    let mut system = super::system(&bytes, '\u{a0}', false, false);
    let source = Source::identified("\u{5d0}\u{a0}\u{5d1}", Id::from_u64(1564020), 3);
    for orientation in [
        LineOrientation::Horizontal,
        LineOrientation::SidewaysClockwise,
    ] {
        let control = prepare_profile(
            &mut system,
            &source,
            orientation,
            0.0,
            ParagraphBaseDirection::RightToLeft,
            false,
        );
        let natural = rtl_line(&control);
        let prepared = prepare_profile(
            &mut system,
            &source,
            orientation,
            4.0,
            ParagraphBaseDirection::RightToLeft,
            false,
        );
        assert_eq!(
            prepared
                .unit_views()
                .map(|unit| (unit.source_range(), unit.level().get()))
                .collect::<Vec<_>>(),
            [
                (Range::new(0, 1), 1),
                (Range::new(1, 2), 1),
                (Range::new(2, 3), 1)
            ]
        );
        let line = rtl_line(&prepared);
        // The true R/CS/R body is level1. L2 puts BET, separator, ALEF in
        // increasing visual inline order. Independent hmtx10 cells occupy
        // [0,10], [10,24], [24,34]; visible separator paint starts at12.
        assert_eq!(
            raw(&natural),
            [
                (Range::new(2, 3), 36, 0.0, 0.0, 10.0),
                (Range::new(1, 2), 58, 0.0, 0.0, 10.0),
                (Range::new(0, 1), 35, 0.0, 0.0, 10.0)
            ]
        );
        assert_eq!(raw(&line), raw(&natural));
        assert_eq!(natural.feedback().advance, 30.0);
        assert_eq!(line.feedback().advance, 34.0);
        assert_eq!(paints(&natural), [0.0, 10.0, 20.0]);
        assert_eq!(paints(&line), [0.0, 12.0, 24.0]);
        let layout = placed(&prepared, line);
        assert_eq!(
            physical_glyphs(&layout),
            if orientation == LineOrientation::Horizontal {
                vec![
                    (Range::new(2, 3), Point::new(20.0, 70.0)),
                    (Range::new(1, 2), Point::new(32.0, 70.0)),
                    (Range::new(0, 1), Point::new(44.0, 70.0)),
                ]
            } else {
                vec![
                    (Range::new(2, 3), Point::new(100.0, 20.0)),
                    (Range::new(1, 2), Point::new(100.0, 32.0)),
                    (Range::new(0, 1), Point::new(100.0, 44.0)),
                ]
            }
        );
        assert_custody(
            &layout,
            &prepared,
            &source,
            [34.0, 24.0, 10.0, 0.0],
            if orientation == LineOrientation::Horizontal {
                62.0
            } else {
                98.0
            },
        );
    }
}

#[test]
fn upright_rtl_source_uses_ltr_cells_and_keeps_the_unrotated_separator_axis() {
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (0xa0, 58), (0x5d0, 35), (0x5d1, 36)],
    );
    qualify(
        &bytes,
        &[('A', 35), ('\u{a0}', 58), ('\u{5d0}', 35), ('\u{5d1}', 36)],
    );
    let mut system = super::system(&bytes, '\u{a0}', false, false);
    let source = Source::identified("\u{5d0}\u{a0}\u{5d1}", Id::from_u64(1564021), 4);
    let control = super::prepare(&mut system, &source, LineOrientation::Vertical, 0.0);
    let natural = finalize(&control);
    let prepared = super::prepare(&mut system, &source, LineOrientation::Vertical, 4.0);
    assert_eq!(prepared.baseline_reference(), Some(FontBaseline::Central));
    assert_eq!(
        prepared.analyzed_source().direction(),
        crate::Direction::LeftToRight
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [0, 0, 0]
    );
    let line = finalize(&prepared);
    // Genuine absent vertical tables synthesize each10px TTB advance and
    // native origin(-5,8). Upright does not inherit the RTL body's odd level.
    assert_eq!(
        raw(&natural),
        [
            (Range::new(0, 1), 35, -5.0, 8.0, 10.0),
            (Range::new(1, 2), 58, -5.0, 8.0, 10.0),
            (Range::new(2, 3), 36, -5.0, 8.0, 10.0)
        ]
    );
    assert_eq!(raw(&line), raw(&natural));
    assert_eq!(line.feedback().advance, 34.0);
    assert_eq!(paints(&line), [0.0, 12.0, 24.0]);
    assert_placement(
        &prepared,
        line.clone(),
        &source,
        [0.0, 12.0, 24.0],
        34.0,
        false,
    );
    let layout = placed(&prepared, line);
    assert_eq!(
        physical_glyphs(&layout),
        [
            (Range::new(0, 1), Point::new(95.0, 28.0)),
            (Range::new(1, 2), Point::new(95.0, 40.0)),
            (Range::new(2, 3), Point::new(95.0, 52.0))
        ]
    );
    assert_custody(&layout, &prepared, &source, [0.0, 10.0, 24.0, 34.0], 95.0);
}

fn qualify_base(bytes: &[u8], coordinate: i16) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    let table = font.base().unwrap();
    assert_eq!((table.version().major, table.version().minor), (1, 0));
    assert!(table.vert_axis().is_none());
    let axis = table.horiz_axis().unwrap().unwrap();
    assert_eq!(
        axis.base_tag_list()
            .unwrap()
            .unwrap()
            .baseline_tags()
            .iter()
            .map(|tag| tag.get().to_be_bytes())
            .collect::<Vec<_>>(),
        [*b"romn"]
    );
    let scripts = axis.base_script_list().unwrap();
    assert_eq!(scripts.base_script_records().len(), 1);
    let script = &scripts.base_script_records()[0];
    assert_eq!(script.base_script_tag(), skrifa::Tag::new(b"latn"));
    let values = script
        .base_script(scripts.offset_data())
        .unwrap()
        .base_values()
        .unwrap()
        .unwrap();
    assert_eq!(values.default_baseline_index(), 0);
    assert_eq!(
        values
            .base_coords()
            .iter()
            .map(|value| value.unwrap().coordinate())
            .collect::<Vec<_>>(),
        [coordinate]
    );
}

#[test]
fn authored_inline_halves_do_not_become_a_second_used_alphabetic_cross_displacement() {
    let source = Source::identified("A\u{a0}A", Id::from_u64(1564022), 5);
    for coordinate in [-100_i16, 100] {
        let bytes = sfnt::replace_tables(
            &mapped('\u{a0}', false),
            vec![(
                *b"BASE",
                base::base(
                    0,
                    Some(base::single_axis(&[*b"romn"], &[coordinate])),
                    None,
                    None,
                ),
            )],
        );
        qualify(&bytes, &[('A', 35), ('\u{a0}', 58)]);
        qualify_base(&bytes, coordinate);
        let mut system = super::system(&bytes, '\u{a0}', false, false);
        let b = f64::from(coordinate) / 100.0;
        for orientation in [
            LineOrientation::Horizontal,
            LineOrientation::SidewaysClockwise,
        ] {
            let control = prepare_profile(
                &mut system,
                &source,
                orientation,
                0.0,
                ParagraphBaseDirection::LeftToRight,
                true,
            );
            let natural = finalize(&control);
            let prepared = prepare_profile(
                &mut system,
                &source,
                orientation,
                4.0,
                ParagraphBaseDirection::LeftToRight,
                true,
            );
            let line = finalize(&prepared);
            assert_eq!(raw(&line), raw(&natural));
            assert_eq!(
                raw(&line),
                [
                    (Range::new(0, 1), 35, 0.0, 0.0, 10.0),
                    (Range::new(1, 2), 58, 0.0, 0.0, 10.0),
                    (Range::new(2, 3), 35, 0.0, 0.0, 10.0)
                ]
            );
            assert_eq!(
                (
                    line.feedback().baseline,
                    line.feedback().line_height,
                    line.feedback().advance
                ),
                (8.0 - b, 10.0, 34.0)
            );
            assert_eq!(paints(&natural), [0.0, 10.0, 20.0]);
            assert_eq!(paints(&line), [0.0, 12.0, 24.0]);
            assert!(line.run_views().all(|run| run.origin().y == 8.0));
            let layout = placed(&prepared, line);
            for run in layout.run_views() {
                let instance = run.effective_instance().unwrap();
                let raw_metrics = instance
                    .metrics(prepared.font_catalogue_identity())
                    .unwrap();
                assert_eq!((raw_metrics.ascent(), raw_metrics.descent()), (8.0, -2.0));
                let raw = run
                    .baseline(
                        prepared.font_catalogue_identity(),
                        FontBaselineAxis::HorizontalY,
                        FontBaseline::Alphabetic,
                    )
                    .unwrap();
                let FontBaselineOutcome::Available(value) = raw.outcome() else {
                    panic!("qualified romn is present")
                };
                assert_eq!(value.coordinate(), b);
                assert_eq!(value.provenance(), FontBaselineProvenance::BaseCoordinate);
                if orientation == LineOrientation::Horizontal {
                    assert_eq!(run.origin().y, 70.0 + b);
                } else {
                    assert_eq!(run.origin().x, 100.0 - b);
                }
            }
            let expected = [0.0, 12.0, 24.0]
                .into_iter()
                .enumerate()
                .map(|(index, pen)| {
                    (
                        Range::new(index, index + 1),
                        if orientation == LineOrientation::Horizontal {
                            Point::new(20.0 + pen, 70.0 + b)
                        } else {
                            Point::new(100.0 - b, 20.0 + pen)
                        },
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(physical_glyphs(&layout), expected);
            assert_custody(
                &layout,
                &prepared,
                &source,
                [0.0, 10.0, 24.0, 34.0],
                if orientation == LineOrientation::Horizontal {
                    62.0 + b
                } else {
                    98.0 - b
                },
            );
        }
    }
}
