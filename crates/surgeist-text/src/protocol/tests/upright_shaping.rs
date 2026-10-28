//! Native upright features and complete horizontal-cursive arrangement cells.
//! Writing Modes 4 §5.1.1 supplies vert/isol semantics; authored OT data below
//! makes substitution, attachment and loss of complete-unit context observable.
use super::*;
use crate::{
    AnalysisTypography, Direction, LineBoxDirection, LineHeight, Point, Range, Rect,
    baseline_test_fonts as base, test_fonts as sfnt, vertical_test_fonts as vertical,
};
use skrifa::{
    GlyphId, MetadataProvider,
    instance::{LocationRef, Size},
    raw::TableProvider,
};

const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");

fn words(values: &[u16]) -> Vec<u8> {
    values.iter().flat_map(|v| v.to_be_bytes()).collect()
}
fn offset(value: usize) -> u16 {
    u16::try_from(value).unwrap()
}

// Features refer to explicit lookup indices. Context lookups can recurse into
// a lookup without enabling that nested substitution as a global feature.
fn ot_layout(
    script: [u8; 4],
    mut features: Vec<([u8; 4], Vec<u16>)>,
    lookups: Vec<(u16, u16, Vec<u8>)>,
) -> Vec<u8> {
    features.sort_by_key(|f| f.0);
    let mut scripts = words(&[1]);
    scripts.extend(script);
    scripts.extend(words(&[8, 4, 0, 0, 0xffff, offset(features.len())]));
    scripts.extend(words(&(0..offset(features.len())).collect::<Vec<_>>()));
    let mut feature_list = words(&[offset(features.len())]);
    feature_list.resize(2 + 6 * features.len(), 0);
    for (i, (tag, indices)) in features.iter().enumerate() {
        let at = 2 + 6 * i;
        feature_list[at..at + 4].copy_from_slice(tag);
        let start = offset(feature_list.len());
        feature_list[at + 4..at + 6].copy_from_slice(&start.to_be_bytes());
        feature_list.extend(words(&[0, offset(indices.len())]));
        feature_list.extend(words(indices));
    }
    let mut lookup_list = words(&[offset(lookups.len())]);
    lookup_list.resize(2 + 2 * lookups.len(), 0);
    for (i, (kind, flags, subtable)) in lookups.into_iter().enumerate() {
        let start = offset(lookup_list.len());
        lookup_list[2 + 2 * i..4 + 2 * i].copy_from_slice(&start.to_be_bytes());
        lookup_list.extend(words(&[kind, flags, 1, 8]));
        lookup_list.extend(subtable);
    }
    let mut table = words(&[
        1,
        0,
        10,
        offset(10 + scripts.len()),
        offset(10 + scripts.len() + feature_list.len()),
    ]);
    table.extend(scripts);
    table.extend(feature_list);
    table.extend(lookup_list);
    table
}

fn single(from: u16, to: u16) -> Vec<u8> {
    words(&[2, 8, 1, to, 1, 1, from])
}

fn joiner_context(joiner: u16, nested_lookup: u16) -> Vec<u8> {
    // ContextSubst format3: input [Beh35, joiner], substitute only Beh.
    // IgnoreMarks on the owning lookup lets a real FATHA remain transparent.
    words(&[3, 2, 1, 14, 20, 0, nested_lookup, 1, 1, 35, 1, 1, joiner])
}

fn arabic_cmap() -> Vec<u8> {
    let mappings = [
        (0x20_u32, 3_u32),
        (0x58, 58),
        (0x628, 35),
        (0x64e, 100),
        (0x200c, 252),
        (0x200d, 253),
    ];
    let mut table = words(&[0, 1, 3, 10]);
    table.extend(12_u32.to_be_bytes());
    table.extend(words(&[12, 0]));
    table.extend(
        u32::try_from(16 + 12 * mappings.len())
            .unwrap()
            .to_be_bytes(),
    );
    table.extend(0_u32.to_be_bytes());
    table.extend(u32::try_from(mappings.len()).unwrap().to_be_bytes());
    for (cp, glyph) in mappings {
        table.extend(cp.to_be_bytes());
        table.extend(cp.to_be_bytes());
        table.extend(glyph.to_be_bytes());
    }
    table
}

fn with_native_metrics(font: &[u8]) -> Vec<u8> {
    let count = sfnt::u16_at(sfnt::table(font, *b"maxp"), 4);
    let font = vertical::with_native(font, 1200, 100, count);
    let script = |tag| base::Script {
        tag,
        default_index: 0,
        coordinates: vec![base::Coord::Design(0), base::Coord::Design(1200)],
    };
    sfnt::replace_tables(
        &font,
        vec![(
            *b"BASE",
            base::base(
                0,
                None,
                Some(base::Axis {
                    tags: vec![*b"ideo", *b"idtp"],
                    scripts: vec![script(*b"arab"), script(*b"latn")],
                }),
                None,
            ),
        )],
    )
}

fn vert_font() -> Vec<u8> {
    with_native_metrics(&crate::shaping_test_fonts::sideways_ahem())
}

fn arabic_font() -> Vec<u8> {
    // Real existing Ahem outlines: nominal Beh35 (square), isol274 (horizontal
    // bar), init82 (bottom strip), medi277 (vertical bar), fina58 (square).
    // Distinct contextual isolated forms36/37 witness complete joiner input.
    let gsub = ot_layout(
        *b"arab",
        vec![
            (*b"ccmp", vec![0]),
            (*b"isol", vec![1, 2, 3]),
            (*b"init", vec![6]),
            (*b"medi", vec![7]),
            (*b"fina", vec![8]),
        ],
        vec![
            // A real cross-cell ligature must never see adjacent upright Beh.
            (4, 0, words(&[1, 18, 1, 8, 1, 4, 58, 2, 35, 1, 1, 35])),
            (5, 8, joiner_context(253, 4)),
            (5, 8, joiner_context(252, 5)),
            (1, 0, single(35, 274)),
            (1, 0, single(35, 36)),
            (1, 0, single(35, 37)),
            (1, 0, single(35, 82)),
            (1, 0, single(35, 277)),
            (1, 0, single(35, 58)),
        ],
    );
    let bases = [35, 36, 37, 274];
    // MarkBasePos format1, one class, one actual mark and four actual bases.
    let mut mark = words(&[1, 58, 64, 1, 12, 24, 1, 0, 6, 1, 100, 200]);
    mark.extend(words(&[4, 10, 16, 22, 28]));
    for _ in bases {
        mark.extend(words(&[1, 400, 600]));
    }
    mark.extend(words(&[1, 1, 100, 1, 4]));
    mark.extend(words(&bases));
    let gpos = ot_layout(*b"arab", vec![(*b"mark", vec![0])], vec![(4, 0, mark)]);
    let gdef = words(&[
        1, 0, 12, 0, 0, 0, 2, 6, 35, 37, 1, 58, 58, 1, 82, 82, 1, 100, 100, 3, 274, 274, 1, 277,
        277, 1,
    ]);
    let mut hmtx = sfnt::table(AHEM, *b"hmtx").to_vec();
    hmtx[100 * 4..100 * 4 + 2].copy_from_slice(&0_u16.to_be_bytes());
    with_native_metrics(&sfnt::replace_tables(
        AHEM,
        vec![
            (*b"cmap", arabic_cmap()),
            (*b"hmtx", hmtx),
            (*b"GDEF", gdef),
            (*b"GSUB", gsub),
            (*b"GPOS", gpos),
        ],
    ))
}

fn qualify(bytes: &[u8], mappings: &[(u32, u32)]) {
    let font = skrifa::FontRef::new(bytes).unwrap();
    assert_eq!(font.head().unwrap().units_per_em(), 1000);
    assert_eq!(font.maxp().unwrap().num_glyphs(), 281);
    assert!(font.table_data(skrifa::Tag::new(b"fvar")).is_none());
    assert!(font.glyf().is_ok());
    for &(cp, glyph) in mappings {
        assert_eq!(font.charmap().map(cp), Some(GlyphId::new(glyph)));
    }
    let metrics = font.glyph_metrics(Size::unscaled(), LocationRef::default());
    for (id, expected) in [
        (35, [0.0, -200.0, 1000.0, 800.0]),
        (36, [0.0, -200.0, 1000.0, 800.0]),
        (37, [0.0, -200.0, 1000.0, 800.0]),
        (58, [0.0, -200.0, 1000.0, 800.0]),
        (82, [0.0, -200.0, 1000.0, 0.0]),
        (274, [0.0, 400.0, 1000.0, 600.0]),
        (277, [200.0, -200.0, 400.0, 800.0]),
    ] {
        let bounds = metrics.bounds(GlyphId::new(id)).unwrap();
        assert_eq!(
            [bounds.x_min, bounds.y_min, bounds.x_max, bounds.y_max],
            expected
        );
        let native = &font.vmtx().unwrap().v_metrics()[usize::try_from(id).unwrap()];
        assert_eq!(
            (native.advance.get(), native.side_bearing.get()),
            (1200, 100)
        );
    }
    let parsed = font.gsub().unwrap();
    assert!(parsed.script_list().is_ok());
    assert!(parsed.feature_list().is_ok());
    assert!(parsed.lookup_list().is_ok());
    let base = font.base().unwrap();
    let scripts = base
        .vert_axis()
        .unwrap()
        .unwrap()
        .base_script_list()
        .unwrap();
    assert_eq!(scripts.base_script_records().len(), 2);
    for (record, tag) in scripts
        .base_script_records()
        .iter()
        .zip([*b"arab", *b"latn"])
    {
        assert_eq!(record.base_script_tag().to_be_bytes(), tag);
        let values = record
            .base_script(scripts.offset_data())
            .unwrap()
            .base_values()
            .unwrap()
            .unwrap();
        assert_eq!(
            values
                .base_coords()
                .iter()
                .map(|v| v.unwrap().coordinate())
                .collect::<Vec<_>>(),
            [0, 1200]
        );
    }
}

fn system_with(bytes: &[u8]) -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let faces = system.registered_font_faces().collect::<Vec<_>>();
    assert_eq!(faces.len(), 1);
    assert_eq!(faces[0].bytes(), bytes);
    system
}

fn composed_line(
    system: &mut System,
    text: &str,
    features: &[&str],
    typography: AnalysisTypography,
    orientation: LineOrientation,
    direction: ParagraphBaseDirection,
) -> (PreparedText, FinalizedLine) {
    let mut font = Font::new().family("Ahem");
    for feature in features {
        font = font.feature(*feature);
    }
    let style = Style {
        font,
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        ..Style::default()
    };
    let source = Source::new(text);
    let len = source.text().len();
    let analysis = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(
            ParagraphInput {
                source: source.clone(),
                base_style: style,
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: direction,
                paragraph_strut: ParagraphStrut::DefaultFont,
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                origins: vec![crate::ParagraphOrigin {
                    text_node: None,
                    source: source.clone(),
                    local: Range::new(0, len),
                    aggregate: Range::new(0, len),
                    kind: crate::ParagraphOriginKind::Authored,
                }],
                connections: vec![],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![],
            },
            typography,
        )
        .unwrap(),
        crate::Wrap::Word,
    );
    assert_eq!(analysis.typography(), typography);
    assert_eq!(analysis.source(), &source);
    if matches!(typography, AnalysisTypography::Upright { .. }) {
        assert_eq!(analysis.direction(), Direction::LeftToRight);
    }
    let prepared = system
        .prepare(
            &analysis,
            PreparationOptions::default().with_orientation(orientation),
        )
        .expect("qualified native upright shaping must prepare");
    let mut input = request(&prepared);
    input.context.orientation = orientation;
    input.context.base_direction = direction;
    input.context.line_box_direction = match direction {
        ParagraphBaseDirection::LeftToRight => LineBoxDirection::LeftToRight,
        ParagraphBaseDirection::RightToLeft => LineBoxDirection::RightToLeft,
    };
    input.context.alignment = crate::Alignment::Start;
    input.context.available_inline = None;
    input.context.spacing = SpacingPolicy::None;
    let line = prepared
        .select(input)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    (prepared, line)
}

fn upright_line(
    system: &mut System,
    text: &str,
    features: &[&str],
) -> (PreparedText, FinalizedLine) {
    composed_line(
        system,
        text,
        features,
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        },
        LineOrientation::Vertical,
        ParagraphBaseDirection::LeftToRight,
    )
}

fn horizontal(system: &mut System, text: &str, features: &[&str]) -> FinalizedLine {
    composed_line(
        system,
        text,
        features,
        AnalysisTypography::HorizontalComposition,
        LineOrientation::Horizontal,
        ParagraphBaseDirection::RightToLeft,
    )
    .1
}

fn source_glyphs(line: &FinalizedLine) -> Vec<(Range, u32)> {
    let mut glyphs = line
        .run_views()
        .flat_map(|r| r.glyphs().map(|g| (g.range(), g.id())).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    // Compare source/component custody independently of RTL paint order within
    // one provider cluster. The authored mark100 follows its base in this list.
    glyphs.sort_by_key(|(range, id)| (range.start, range.end, *id == 100));
    glyphs
}

fn place_upright(prepared: PreparedText, line: FinalizedLine) -> crate::Layout {
    crate::Layout::from_finalized_lines(PlacementRequest {
        prepared,
        coverage: PlacementCoverage::WholeParagraph,
        entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
            line,
            anchor: LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: LineOrientation::Vertical,
            atomics: vec![],
        })],
    })
    .unwrap()
}

#[test]
fn authored_font_controls_prove_distinct_forms_context_and_real_mark_attachment() {
    let bytes = arabic_font();
    qualify(
        &bytes,
        &[(0x628, 35), (0x64e, 100), (0x200c, 252), (0x200d, 253)],
    );
    let mut system = system_with(&bytes);
    for (text, features, expected) in [
        ("\u{628}", &[][..], vec![(Range::new(0, 1), 274)]),
        (
            "\u{628}",
            &[r#""isol" 0"#][..],
            vec![(Range::new(0, 1), 35)],
        ),
        (
            "\u{628}\u{628}",
            &[r#""ccmp" 0"#][..],
            vec![(Range::new(0, 1), 82), (Range::new(1, 2), 58)],
        ),
        (
            "\u{628}\u{628}\u{628}",
            &[r#""ccmp" 0"#][..],
            vec![
                (Range::new(0, 1), 82),
                (Range::new(1, 2), 277),
                (Range::new(2, 3), 58),
            ],
        ),
        (
            "\u{628}\u{200d}",
            &[r#""isol" 1"#, r#""init" 0"#, r#""medi" 0"#, r#""fina" 0"#][..],
            vec![(Range::new(0, 2), 36)],
        ),
        (
            "\u{628}\u{200c}",
            &[r#""isol" 1"#][..],
            // ZWNJ's zero-output receipt keeps its separate source component;
            // matching it as context does not turn this into a ligature.
            vec![(Range::new(0, 1), 37)],
        ),
        (
            "\u{628}\u{64e}\u{200d}",
            &[r#""isol" 1"#, r#""init" 0"#, r#""medi" 0"#, r#""fina" 0"#][..],
            vec![(Range::new(0, 3), 36), (Range::new(0, 3), 100)],
        ),
    ] {
        assert_eq!(
            source_glyphs(&horizontal(&mut system, text, features)),
            expected,
            "{text:?} {features:?}"
        );
    }
    let mark = horizontal(&mut system, "\u{628}\u{64e}", &[]);
    let mut attachment = mark
        .run_views()
        .flat_map(|r| {
            r.glyphs()
                .map(|g| {
                    (
                        g.id(),
                        g.pen() + f64::from(g.x()),
                        f64::from(g.y()),
                        g.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    attachment.sort_by_key(|(id, _, _, _)| *id == 100);
    assert_eq!(attachment, [(274, 0.0, 0.0, 10.0), (100, 3.0, -4.0, 0.0)]);
    // Explicitly enabling ccmp proves the cross-cell leak witness is real.
    assert_eq!(
        source_glyphs(&horizontal(&mut system, "\u{628}\u{628}", &[r#""ccmp" 1"#])),
        [(Range::new(0, 2), 58)]
    );
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    let mut system = system_with(&bytes);
    for (features, id) in [
        (&[][..], 58),
        (&[r#""vert" 1"#][..], 82),
        (&[r#""vrt2" 1"#][..], 277),
        (&[r#""vrtr" 1"#][..], 274),
    ] {
        assert_eq!(
            source_glyphs(&horizontal(&mut system, "X", features)),
            [(Range::new(0, 1), id)]
        );
    }
}

#[test]
fn upright_vert_uses_post_substitution_metrics_and_authored_feature_composition() {
    let bytes = vert_font();
    qualify(&bytes, &[(0x58, 58)]);
    let mut system = system_with(&bytes);
    for (features, id, y) in [
        (&[][..], 82, 1.0),
        (&[r#""vert" 0"#][..], 58, 9.0),
        (&[r#""vert" 0"#, r#""vert" 1"#][..], 82, 1.0),
        (&[r#""vert" 1"#, r#""vert" 0"#][..], 58, 9.0),
        (&[r#""vrt2" 1"#][..], 277, 9.0),
        (&[r#""vert" 0"#, r#""vrt2" 1"#][..], 277, 9.0),
        (&[r#""vrtr" 1"#][..], 274, 7.0),
        (&[r#""vert" 0"#, r#""vrtr" 1"#, r#""vrtr" 0"#][..], 58, 9.0),
    ] {
        let (prepared, line) = upright_line(&mut system, "X", features);
        assert_eq!(line.feedback().advance, 12.0);
        assert_eq!(source_glyphs(&line), [(Range::new(0, 1), id)]);
        let layout = place_upright(prepared, line);
        assert_eq!(layout.bounds(), Rect::new(94.0, 20.0, 12.0, 12.0));
        let run = layout.run_views().next().unwrap();
        assert_eq!(run.face().bytes(), bytes);
        assert_eq!(run.origin(), Point::new(100.0, 20.0));
        let glyph = run.glyphs().next().unwrap();
        // Native yMax differs by output glyph: 0/800/600; add real TSB100.
        assert_eq!(
            (
                glyph.id(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
                glyph.pen()
            ),
            (id, -6.0, y, 12.0, 0.0)
        );
        assert_eq!(
            Point::new(
                run.origin().x + f64::from(glyph.x()),
                run.origin().y + glyph.pen() + f64::from(glyph.y())
            ),
            Point::new(94.0, 20.0 + f64::from(y))
        );
    }
}

#[test]
fn upright_arabic_uses_distinct_isol_and_never_ligates_adjacent_cells() {
    let bytes = arabic_font();
    qualify(&bytes, &[(0x628, 35)]);
    let mut system = system_with(&bytes);
    for (text, features, expected) in [
        ("\u{628}", &[][..], vec![(Range::new(0, 1), 274)]),
        (
            "\u{628}\u{628}",
            &[][..],
            vec![(Range::new(0, 1), 274), (Range::new(1, 2), 274)],
        ),
        (
            "\u{628}\u{628}\u{628}",
            &[][..],
            vec![
                (Range::new(0, 1), 274),
                (Range::new(1, 2), 274),
                (Range::new(2, 3), 274),
            ],
        ),
        (
            "\u{628}",
            &[r#""isol" 0"#][..],
            vec![(Range::new(0, 1), 35)],
        ),
        (
            "\u{628}",
            &[r#""isol" 0"#, r#""isol" 1"#][..],
            vec![(Range::new(0, 1), 274)],
        ),
        (
            "\u{628}",
            &[r#""isol" 1"#, r#""isol" 0"#][..],
            vec![(Range::new(0, 1), 35)],
        ),
    ] {
        let (prepared, line) = upright_line(&mut system, text, features);
        assert_eq!(source_glyphs(&line), expected, "{text:?} {features:?}");
        assert_eq!(line.feedback().advance, 12.0 * expected.len() as f64);
        let layout = place_upright(prepared, line);
        assert_eq!(
            layout.source().text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(
            layout.bounds(),
            Rect::new(94.0, 20.0, 12.0, 12.0 * expected.len() as f64)
        );
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            assert_eq!(
                run.effective_instance()
                    .unwrap()
                    .normalized_coords()
                    .collect::<Vec<_>>(),
                []
            );
        }
    }
}

#[test]
fn upright_arabic_keeps_mark_and_joiner_context_inside_the_complete_cell() {
    let bytes = arabic_font();
    qualify(
        &bytes,
        &[(0x628, 35), (0x64e, 100), (0x200c, 252), (0x200d, 253)],
    );
    let mut system = system_with(&bytes);
    for (text, expected, positions) in [
        (
            "\u{628}\u{64e}\u{628}",
            vec![
                (Range::new(0, 2), 274),
                (Range::new(0, 2), 100),
                (Range::new(2, 3), 274),
            ],
            vec![
                Point::new(94.0, 27.0),
                Point::new(97.0, 23.0),
                Point::new(94.0, 39.0),
            ],
        ),
        (
            "\u{628}\u{200d}\u{628}",
            vec![(Range::new(0, 2), 36), (Range::new(2, 3), 274)],
            vec![Point::new(94.0, 29.0), Point::new(94.0, 39.0)],
        ),
        (
            "\u{628}\u{200c}\u{628}",
            vec![(Range::new(0, 1), 37), (Range::new(2, 3), 274)],
            vec![Point::new(94.0, 29.0), Point::new(94.0, 39.0)],
        ),
        (
            "\u{628}\u{64e}\u{200d}\u{628}",
            vec![
                (Range::new(0, 3), 36),
                (Range::new(0, 3), 100),
                (Range::new(3, 4), 274),
            ],
            vec![
                Point::new(94.0, 29.0),
                Point::new(97.0, 25.0),
                Point::new(94.0, 39.0),
            ],
        ),
    ] {
        let (prepared, line) = upright_line(&mut system, text, &[]);
        assert_eq!(source_glyphs(&line), expected, "{text:?}");
        assert_eq!(line.feedback().advance, 24.0);
        let layout = place_upright(prepared, line);
        assert_eq!(layout.bounds(), Rect::new(94.0, 20.0, 12.0, 24.0));
        assert_eq!(
            layout.source().text().code_units(),
            text.encode_utf16().collect::<Vec<_>>()
        );
        let mut actual = Vec::new();
        for run in layout.run_views() {
            assert_eq!(run.face().bytes(), bytes);
            for glyph in run.glyphs() {
                actual.push((
                    glyph.id(),
                    Point::new(
                        run.origin().x + f64::from(glyph.x()),
                        run.origin().y + glyph.pen() + f64::from(glyph.y()),
                    ),
                ));
                assert_eq!(glyph.advance(), if glyph.id() == 100 { 0.0 } else { 12.0 });
            }
        }
        assert_eq!(
            actual,
            expected
                .iter()
                .zip(positions)
                .map(|((_, id), point)| (*id, point))
                .collect::<Vec<_>>()
        );
        // Anchor delta (400-100,600-200) = (+3,-4) physically. The mark's
        // origin is three right/four up from its own base, including when a
        // retained transparent FATHA precedes the actual ZWJ input in GSUB.
        if text.starts_with("\u{628}\u{64e}") {
            assert_eq!(
                (actual[1].1.x - actual[0].1.x, actual[1].1.y - actual[0].1.y),
                (3.0, -4.0)
            );
        }
    }
}

#[path = "upright_shaping/failures.rs"]
mod failures;
#[path = "upright_shaping/policy.rs"]
mod policy;

#[path = "upright_shaping/arrangement.rs"]
mod arrangement;

#[path = "upright_shaping/admission.rs"]
mod admission;
