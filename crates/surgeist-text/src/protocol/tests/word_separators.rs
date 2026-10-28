//! Existing authored word spacing over actual independent provider units.
//! Text4 §8.1 requires separator coverage, equal halves and intrinsic-zero suppression.
use super::*;
use crate::{
    Affinity, Cursor, FontBaseline, FontBaselineScript, GlyphBasis, Id, LineHeight, Point, Range,
    Rect, Selection, test_fonts as sfnt, vertical_test_fonts as vertical,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size as FontSize},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/fonts/ahem/Ahem.ttf"
));
const FRAMES: [LineOrientation; 3] = [
    LineOrientation::Horizontal,
    LineOrientation::Sideways,
    LineOrientation::Vertical,
];

fn mapped(separator: char, zero_horizontal: bool) -> Vec<u8> {
    // Real format12 cmap over licensed Ahem squares. The optional82+82 lookup
    // cannot match A35/separator58/A35; no shaping result is supplied by a fake.
    let bytes = crate::shaping_test_fonts::range_ligature_ahem(
        &[82, 82],
        &[(0x41, 35), (u32::from(separator), 58)],
    );
    if zero_horizontal {
        let mut hmtx = sfnt::table(&bytes, *b"hmtx").to_vec();
        hmtx[58 * 4..58 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
        sfnt::replace_tables(&bytes, vec![(*b"hmtx", hmtx)])
    } else {
        bytes
    }
}

fn vertical_zero_space() -> Vec<u8> {
    let mapping = mapped(' ', false);
    // This existing native table fixture has X58 advance0/TSB−1000 and
    // A35 advance1200/TSB100. A zero hmtx would not prove a zero TTB advance.
    sfnt::replace_tables(
        &vertical::zero_signed(),
        vec![(*b"cmap", sfnt::table(&mapping, *b"cmap").to_vec())],
    )
}

fn system(bytes: &[u8], separator: char, zero: bool, native_zero: bool) -> System {
    let parsed = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(parsed.head().unwrap().units_per_em(), 1000);
    assert_eq!(parsed.maxp().unwrap().num_glyphs(), 281);
    assert_eq!(parsed.charmap().map('A'), Some(GlyphId::new(35)));
    assert_eq!(parsed.charmap().map(separator), Some(GlyphId::new(58)));
    assert!(parsed.table_data(skrifa::Tag::new(b"fvar")).is_none());
    for tag in [*b"glyf", *b"loca", *b"hhea", *b"OS/2"] {
        assert_eq!(sfnt::table(bytes, tag), sfnt::table(AHEM, tag));
    }
    let hhea = parsed.hhea().unwrap();
    assert_eq!(
        (hhea.ascender().to_i16(), hhea.descender().to_i16()),
        (800, -200)
    );
    let metrics = parsed.glyph_metrics(FontSize::unscaled(), LocationRef::default());
    for glyph in [35, 58] {
        assert_eq!(
            metrics.advance_width(GlyphId::new(glyph)),
            Some(if glyph == 58 && zero && !native_zero {
                0.0
            } else {
                1000.0
            })
        );
        let bounds = metrics.bounds(GlyphId::new(glyph)).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            [0.0, -200.0, 1000.0, 800.0]
        );
    }
    if native_zero {
        let vhea = parsed.vhea().unwrap();
        assert_eq!(vhea.version().to_major_minor(), (1, 0));
        assert_eq!(vhea.number_of_long_ver_metrics(), 281);
        let vmtx = parsed.vmtx().unwrap();
        for (glyph, expected) in [(35, (1200, 100)), (58, (0, -1000))] {
            let row = &vmtx.v_metrics()[glyph];
            assert_eq!((row.advance.get(), row.side_bearing.get()), expected);
        }
    } else {
        assert!(parsed.table_data(skrifa::Tag::new(b"vhea")).is_none());
        assert!(parsed.table_data(skrifa::Tag::new(b"vmtx")).is_none());
    }
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let face = system.registered_font_faces().next().unwrap().to_owned();
    assert_eq!(face.bytes(), bytes);
    assert_eq!(face.index(), 0);
    let instance = face.unsynthesized_instance(10.0, &[]).unwrap();
    assert_eq!(instance.axis_count(), 0);
    let raw = instance.metrics(system.font_catalogue_identity()).unwrap();
    assert_eq!((raw.ascent(), raw.descent()), (8.0, -2.0));
    if native_zero {
        for (glyph, advance, y) in [(35, 12.0, 9.0), (58, 0.0, -2.0)] {
            let raw = instance
                .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
                .unwrap();
            assert_eq!(
                (raw.advance(), raw.origin_x(), raw.origin_y()),
                (advance, 5.0, y)
            );
        }
    } else if !zero {
        for glyph in [35, 58] {
            let raw = instance
                .vertical_glyph_metrics(glyph, system.font_catalogue_identity())
                .unwrap();
            assert_eq!(
                (raw.advance(), raw.origin_x(), raw.origin_y()),
                (10.0, 5.0, 8.0)
            );
        }
    }
    let cross = instance
        .upright_metrics(
            system.font_catalogue_identity(),
            FontBaselineScript::from_iso15924(*b"Latn").unwrap(),
        )
        .unwrap();
    assert_eq!((cross.used_over(), cross.used_under()), (5.0, -5.0));
    system
}

fn prepare(
    system: &mut System,
    source: &Source,
    orientation: LineOrientation,
    spacing: f64,
) -> PreparedText {
    let analyzed = crate::test_wrapping::bind_missing(
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
                base_direction: ParagraphBaseDirection::LeftToRight,
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
            if orientation == LineOrientation::Vertical {
                crate::AnalysisTypography::Upright {
                    computed_direction: ParagraphBaseDirection::RightToLeft,
                }
            } else {
                crate::AnalysisTypography::HorizontalComposition
            },
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::default().with_orientation(orientation),
        )
        .unwrap();
    assert_eq!(prepared.analyzed_source().source(), source);
    assert_eq!(
        prepared.baseline_reference(),
        if orientation == LineOrientation::Vertical {
            Some(FontBaseline::Central)
        } else {
            None
        }
    );
    prepared
}

fn finalize(prepared: &PreparedText) -> FinalizedLine {
    let mut selection = request(prepared);
    selection.context.orientation = prepared.orientation();
    selection.context.available_inline = None;
    selection.context.spacing = SpacingPolicy::None;
    prepared
        .select(selection)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}

type RawGlyph = (Range, u32, f32, f32, f32);

fn raw(line: &FinalizedLine) -> Vec<RawGlyph> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.range(),
                        glyph.id(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn paints(line: &FinalizedLine) -> Vec<f64> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|glyph| run.origin().x + glyph.pen())
                .collect::<Vec<_>>()
        })
        .collect()
}

fn assert_placement(
    prepared: &PreparedText,
    line: FinalizedLine,
    source: &Source,
    positions: [f64; 3],
    total: f64,
    native_zero: bool,
) {
    let orientation = prepared.orientation();
    let horizontal = orientation == LineOrientation::Horizontal;
    let upright = orientation == LineOrientation::Vertical;
    let cross = if horizontal { 70.0 } else { 100.0 };
    let layout = crate::Layout::from_finalized_lines(PlacementRequest {
        prepared: prepared.clone(),
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: cross,
            },
            orientation,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(layout.source(), source);
    assert_eq!(
        layout.source().text().code_units(),
        source.text().code_units()
    );
    let low = if horizontal {
        62.0
    } else if upright {
        95.0
    } else {
        98.0
    };
    assert_eq!(
        layout.bounds(),
        if horizontal {
            Rect::new(20.0, low, total, 10.0)
        } else {
            Rect::new(low, 20.0, 10.0, total)
        }
    );
    let actual = layout
        .run_views()
        .flat_map(|run| {
            assert_eq!(
                run.glyph_basis(),
                if orientation == LineOrientation::Sideways {
                    GlyphBasis::Clockwise
                } else {
                    GlyphBasis::Unrotated
                }
            );
            let origin = run.origin();
            run.glyphs()
                .map(|glyph| match orientation {
                    LineOrientation::Horizontal => Point::new(
                        origin.x + glyph.pen() + f64::from(glyph.x()),
                        origin.y + f64::from(glyph.y()),
                    ),
                    LineOrientation::Sideways => Point::new(
                        origin.x - f64::from(glyph.y()),
                        origin.y + glyph.pen() + f64::from(glyph.x()),
                    ),
                    LineOrientation::Vertical => Point::new(
                        origin.x + f64::from(glyph.x()),
                        origin.y + glyph.pen() + f64::from(glyph.y()),
                    ),
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    let expected = positions
        .into_iter()
        .enumerate()
        .map(|(index, position)| {
            if horizontal {
                Point::new(20.0 + position, 70.0)
            } else if upright {
                Point::new(
                    95.0,
                    20.0 + position
                        + if native_zero {
                            if index == 1 { -2.0 } else { 9.0 }
                        } else {
                            8.0
                        },
                )
            } else {
                Point::new(100.0, 20.0 + position)
            }
        })
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    let step = if native_zero { 12.0 } else { 10.0 };
    for (index, pen) in [(0, 0.0), (1, step), (2, positions[2]), (3, total)] {
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
                Rect::new(20.0 + pen, low, 1.0, 10.0)
            } else {
                Rect::new(low, 20.0 + pen, 10.0, 1.0)
            }
        );
    }
    let selection = layout
        .selection(Selection::new(
            Cursor::new(0, Affinity::After),
            Cursor::new(3, Affinity::Before),
        ))
        .unwrap();
    assert_eq!(selection.rects().len(), 1);
    assert_eq!(selection.rects()[0].rect(), layout.bounds());
    assert_eq!(
        layout
            .glyph_runs()
            .iter()
            .flat_map(|run| run.glyphs().iter().map(|glyph| glyph.range()))
            .collect::<Vec<_>>(),
        [Range::new(0, 1), Range::new(1, 2), Range::new(2, 3)]
    );
}

fn positive(separator: char) {
    let bytes = mapped(separator, false);
    let mut system = system(&bytes, separator, false, false);
    let source = Source::identified(format!("A{separator}A"), Id::from_u64(1564001), 2);
    for orientation in FRAMES {
        let control = prepare(&mut system, &source, orientation, 0.0);
        let natural = finalize(&control);
        let offsets = if orientation == LineOrientation::Vertical {
            (-5.0, 8.0)
        } else {
            (0.0, 0.0)
        };
        assert_eq!(
            raw(&natural),
            [
                (Range::new(0, 1), 35, offsets.0, offsets.1, 10.0),
                (Range::new(1, 2), 58, offsets.0, offsets.1, 10.0),
                (Range::new(2, 3), 35, offsets.0, offsets.1, 10.0)
            ]
        );
        assert_eq!(natural.feedback().advance, 30.0);
        assert_eq!(paints(&natural), [0.0, 10.0, 20.0]);
        let prepared = prepare(&mut system, &source, orientation, 4.0);
        let line = finalize(&prepared);
        assert_eq!(raw(&line), raw(&natural));
        assert_eq!(line.feedback().advance, 34.0);
        assert_eq!(paints(&line), [0.0, 12.0, 24.0]);
        assert_eq!(
            (line.feedback().baseline, line.feedback().line_height),
            (
                if orientation == LineOrientation::Vertical {
                    5.0
                } else {
                    8.0
                },
                10.0
            )
        );
        assert_placement(&prepared, line, &source, [0.0, 12.0, 24.0], 34.0, false);
    }
}

#[test]
fn visible_ethiopic_wordspace_receives_authored_spacing_in_real_native_frames() {
    positive('\u{1361}');
}

#[test]
fn visible_nbsp_receives_half_the_authored_spacing_on_each_side_of_its_ink() {
    positive('\u{a0}');
}

#[test]
fn actual_zero_intrinsic_space_cannot_acquire_authored_width_in_any_native_frame() {
    let source = Source::identified("A A", Id::from_u64(1564002), 3);
    for orientation in FRAMES {
        let native_zero = orientation == LineOrientation::Vertical;
        let bytes = if native_zero {
            vertical_zero_space()
        } else {
            mapped(' ', true)
        };
        let mut system = system(&bytes, ' ', true, native_zero);
        let control = prepare(&mut system, &source, orientation, 0.0);
        let natural = finalize(&control);
        let (body, raw_body) = if native_zero {
            (12.0, 12.0_f32)
        } else {
            (10.0, 10.0_f32)
        };
        let total = body * 2.0;
        assert_eq!(
            raw(&natural),
            [
                (
                    Range::new(0, 1),
                    35,
                    if native_zero { -5.0 } else { 0.0 },
                    if native_zero { 9.0 } else { 0.0 },
                    raw_body
                ),
                (
                    Range::new(1, 2),
                    58,
                    if native_zero { -5.0 } else { 0.0 },
                    if native_zero { -2.0 } else { 0.0 },
                    0.0
                ),
                (
                    Range::new(2, 3),
                    35,
                    if native_zero { -5.0 } else { 0.0 },
                    if native_zero { 9.0 } else { 0.0 },
                    raw_body
                )
            ]
        );
        assert_eq!(natural.feedback().advance, total);
        assert_eq!(paints(&natural), [0.0, body, body]);
        let prepared = prepare(&mut system, &source, orientation, 4.0);
        let line = finalize(&prepared);
        assert_eq!(raw(&line), raw(&natural));
        assert_eq!(line.feedback().advance, total);
        assert_eq!(paints(&line), [0.0, body, body]);
        assert_placement(
            &prepared,
            line,
            &source,
            [0.0, body, body],
            total,
            native_zero,
        );
    }
}
