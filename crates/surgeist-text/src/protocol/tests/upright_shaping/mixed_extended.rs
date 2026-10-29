//! Additional real-source Mixed qualification; linked after the core freeze.
use super::*;
#[cfg(feature = "text-accessibility")]
use crate::{Affinity, Cursor};
use crate::{
    Decoration, DecorationKind, DecorationOffset, DecorationThickness, ErrorDetail,
    FontMetricTable, FontMetricTableError, TextStyleOwner, UprightDecorationRole,
    UprightScriptFailure,
};
use icu_properties::{
    props::{BidiClass, BidiMirroringGlyph, BidiPairedBracketType, GeneralCategory, Script},
    script::ScriptWithExtensions,
};

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    }
}
fn prepare_source(
    system: &mut System,
    source: Source,
    style: Style,
) -> crate::Result<PreparedText> {
    prepare_source_with_strut(system, source, style, ParagraphStrut::DefaultFont)
}
fn prepare_source_with_strut(
    system: &mut System,
    source: Source,
    style: Style,
    paragraph_strut: ParagraphStrut,
) -> crate::Result<PreparedText> {
    let extent = Range::new(0, source.text().len());
    let value = ParagraphInput {
        source: source.clone(),
        base_style: style,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source,
            local: extent,
            aggregate: extent,
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: if extent.is_empty() {
            vec![crate::ParagraphAnchor {
                affinity: crate::Affinity::Before,
                item: crate::ParagraphAnchorItem::EmptyOrigin(0),
            }]
        } else {
            vec![]
        },
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, AnalysisTypography::Mixed)?,
        crate::Wrap::Word,
    );
    system.prepare(
        &analyzed,
        PreparationOptions::default().with_orientation(LineOrientation::Vertical),
    )
}
fn finish(prepared: &PreparedText) -> FinalizedLine {
    let mut value = request(prepared);
    value.context.orientation = LineOrientation::Vertical;
    value.context.base_direction = ParagraphBaseDirection::LeftToRight;
    value.context.line_box_direction = LineBoxDirection::LeftToRight;
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::None;
    prepared
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn close(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 0.000001,
        "{actual} != {expected}"
    );
}

fn native_rtl(bytes: &[u8], text: &str) -> Vec<(u32, u32)> {
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    for (ordinal, scalar) in text.chars().enumerate() {
        input.add(scalar, u32::try_from(ordinal).unwrap());
    }
    input.set_direction(surgeist_shaper::Direction::RightToLeft);
    input.set_script(surgeist_shaper::script::HEBREW);
    let output = shaper.shape(input, &[]);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    output
        .glyph_infos()
        .iter()
        .map(|info| (info.glyph_id, info.cluster))
        .collect()
}

#[test]
fn mixed_odd_level_upright_brackets_keep_mirroring_and_visual_source_order_once() {
    let base = mixed_fonts::fixture(true);
    premises(&base, true);
    let mut mappings = mixed_fonts::mappings();
    for (cp, glyph) in &mut mappings {
        if [0x2329, 0x3008].contains(cp) {
            *glyph = 35;
        }
        if [0x232a, 0x3009].contains(cp) {
            *glyph = 36;
        }
    }
    let bytes = mixed_fonts::with_cmap(&base, &mappings);
    let mirror = CodePointMapData::<BidiMirroringGlyph>::new();
    for (scalar, paired, kind) in [
        ('\u{2329}', '\u{232a}', BidiPairedBracketType::Open),
        ('\u{232a}', '\u{2329}', BidiPairedBracketType::Close),
    ] {
        let property = mirror.get(scalar);
        assert_eq!(
            (
                property.mirrored,
                property.mirroring_glyph,
                property.paired_bracket_type
            ),
            (true, Some(paired), kind)
        );
        assert_eq!(
            CodePointMapData::<VerticalOrientation>::new().get(scalar),
            VerticalOrientation::TransformedRotated
        );
        assert_eq!(
            CodePointMapData::<BidiClass>::new().get(scalar),
            BidiClass::OtherNeutral
        );
    }
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('、'),
        VerticalOrientation::TransformedUpright
    );
    // The provider's TTB operation preserves logical clusters and does not do
    // RTL mirroring merely because its script is Hebrew. Genuine horizontal
    // RTL controls establish the reciprocal mirrored input glyphs independently.
    assert_eq!(native_rtl(&bytes, "\u{2329}"), [(36, 0)]);
    assert_eq!(native_rtl(&bytes, "\u{232a}"), [(35, 0)]);
    assert_eq!(
        native(
            &bytes,
            "\u{2329}、\u{232a}",
            true,
            surgeist_shaper::script::HEBREW,
            &[]
        ),
        [
            (35, 0, 12.0, -6.0, 9.0),
            (82, 1, 12.0, -6.0, 1.0),
            (36, 2, 12.0, -6.0, 9.0),
        ]
    );
    assert_eq!(
        native(
            &bytes,
            "\u{232a}、\u{2329}",
            true,
            surgeist_shaper::script::HEBREW,
            &[]
        ),
        [
            (36, 0, 12.0, -6.0, 9.0),
            (82, 1, 12.0, -6.0, 1.0),
            (35, 2, 12.0, -6.0, 9.0),
        ]
    );
    // N0 resolves the paired neutral bracket contents against the surrounding
    // RTL Hebrew. All five units are level1; L2 reverses source4,3,2,1,0.
    let text = "א\u{2329}、\u{232a}ב";
    let (prepared, line) = composed_line(
        &mut system_with(&bytes),
        text,
        &[],
        AnalysisTypography::Mixed,
        LineOrientation::Vertical,
        ParagraphBaseDirection::RightToLeft,
    );
    assert_eq!(
        prepared.analyzed_source().direction(),
        Direction::RightToLeft
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [1; 5]
    );
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.0, p.1, p.2))
            .collect::<Vec<_>>(),
        [
            (Range::new(4, 5), GlyphBasis::Clockwise, 274),
            (Range::new(3, 4), GlyphBasis::Unrotated, 35),
            (Range::new(2, 3), GlyphBasis::Unrotated, 82),
            (Range::new(1, 2), GlyphBasis::Unrotated, 36),
            (Range::new(0, 1), GlyphBasis::Clockwise, 274),
        ]
    );
    assert_eq!(line.feedback().advance, 56.0);
    let source = prepared.analyzed_source().source().clone();
    copy_source(&place_upright(prepared, line), &source);
    let (whole, control) = composed_line(
        &mut system_with(&bytes),
        text,
        &[],
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::RightToLeft,
        },
        LineOrientation::Vertical,
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(whole.analyzed_source().direction(), Direction::LeftToRight);
    assert_eq!(
        whole
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [0; 5]
    );
    assert_eq!(
        packets(&control, &bytes)
            .iter()
            .map(|p| p.0)
            .collect::<Vec<_>>(),
        (0..5).map(|i| Range::new(i, i + 1)).collect::<Vec<_>>()
    );
}

fn mark_font() -> Vec<u8> {
    let mut mappings = mixed_fonts::mappings();
    for (cp, glyph) in &mut mappings {
        if [0x58, 0x4e00].contains(cp) {
            *glyph = 35;
        }
    }
    let font = mixed_fonts::with_metrics(&crate::shaping_test_fonts::mark_ahem(), true);
    mixed_fonts::with_cmap(&font, &mappings)
}
fn mark_premises(bytes: &[u8]) {
    let gpos = sfnt::table(bytes, *b"GPOS");
    let list = usize::from(sfnt::u16_at(gpos, 8));
    let lookup = list + usize::from(sfnt::u16_at(gpos, list + 2));
    assert_eq!(sfnt::u16_at(gpos, lookup), 4);
    let sub = lookup + usize::from(sfnt::u16_at(gpos, lookup + 6));
    let marks = sub + usize::from(sfnt::u16_at(gpos, sub + 8));
    let bases = sub + usize::from(sfnt::u16_at(gpos, sub + 10));
    let mark = marks + usize::from(sfnt::u16_at(gpos, marks + 4));
    let base = bases + usize::from(sfnt::u16_at(gpos, bases + 2));
    assert_eq!(
        (
            sfnt::u16_at(gpos, mark),
            sfnt::u16_at(gpos, mark + 2),
            sfnt::u16_at(gpos, mark + 4)
        ),
        (1, 100, 200)
    );
    assert_eq!(
        (
            sfnt::u16_at(gpos, base),
            sfnt::u16_at(gpos, base + 2),
            sfnt::u16_at(gpos, base + 4)
        ),
        (1, 400, 600)
    );
    assert_eq!(sfnt::table(bytes, *b"glyf"), sfnt::table(AHEM, *b"glyf"));
    assert_eq!(sfnt::table(bytes, *b"loca"), sfnt::table(AHEM, *b"loca"));
}
fn native_points(rows: &[(u32, u32, f64, f64, f64)], upright: bool) -> Vec<(u32, Point)> {
    let mut pen = 0.0;
    rows.iter()
        .map(|&(id, _, advance, x, y)| {
            let point = if upright {
                Point::new(x, pen + y)
            } else {
                Point::new(pen + x, y)
            };
            pen += advance;
            (id, point)
        })
        .collect()
}

#[test]
fn mixed_marks_retain_complete_egc_axes_attachment_and_independent_style_height() {
    let bytes = mark_font();
    mark_premises(&bytes);
    let vo = CodePointMapData::<VerticalOrientation>::new();
    assert_eq!(vo.get('\u{3099}'), VerticalOrientation::Upright);
    assert_eq!(vo.get('\u{301}'), VerticalOrientation::Rotated);
    let h = native_points(
        &native(
            &bytes,
            "X\u{3099}",
            false,
            surgeist_shaper::script::LATIN,
            &[],
        ),
        false,
    );
    assert_eq!(
        h,
        [(35, Point::new(0.0, 0.0)), (100, Point::new(3.0, -4.0))]
    );
    let v = native_points(
        &native(&bytes, "一\u{301}", true, surgeist_shaper::script::HAN, &[]),
        true,
    );
    assert_eq!((v[1].1.x - v[0].1.x, v[1].1.y - v[0].1.y), (3.0, -4.0));
    let mut source = Source::new("X\u{3099}一\u{301}");
    let mut tall = style();
    tall.line_height = LineHeight::Absolute(16.0);
    source.span(Range::new(1, 2), tall.clone());
    source.span(Range::new(3, 4), tall);
    let prepared = prepare_source(&mut system_with(&bytes), source.clone(), style()).unwrap();
    assert_eq!(
        prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
        [Range::new(0, 2), Range::new(2, 4)]
    );
    let line = finish(&prepared);
    assert_eq!(line.feedback().advance, 22.0);
    // Mark16 contributes R7/9 and U8/8 around the same Central, hence17.
    assert_eq!(
        (
            line.feedback().ascent,
            line.feedback().descent,
            line.feedback().line_height
        ),
        (8.0, 9.0, 17.0)
    );
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.1, p.2, p.6))
            .collect::<Vec<_>>(),
        [
            (GlyphBasis::Clockwise, 35, 10.0),
            (GlyphBasis::Clockwise, 100, 0.0),
            (GlyphBasis::Unrotated, 35, 12.0),
            (GlyphBasis::Unrotated, 100, 0.0),
        ]
    );
    let layout = place_upright(prepared, line);
    let mut points = Vec::new();
    for run in layout.run_views() {
        let origin = run.origin();
        for glyph in run.glyphs() {
            points.push(match run.glyph_basis() {
                GlyphBasis::Clockwise => Point::new(
                    origin.x - f64::from(glyph.y()),
                    origin.y + glyph.pen() + f64::from(glyph.x()),
                ),
                GlyphBasis::Counterclockwise => Point::new(
                    origin.x + f64::from(glyph.y()),
                    origin.y + glyph.pen() - f64::from(glyph.x()),
                ),
                GlyphBasis::Unrotated => Point::new(
                    origin.x + f64::from(glyph.x()),
                    origin.y + glyph.pen() + f64::from(glyph.y()),
                ),
            });
        }
    }
    assert_eq!(
        (points[1].x - points[0].x, points[1].y - points[0].y),
        (4.0, 3.0)
    );
    assert_eq!(
        (points[3].x - points[2].x, points[3].y - points[2].y),
        (3.0, -4.0)
    );
    copy_source(&layout, &source);
}

#[test]
fn mixed_appendix_e_uses_the_real_first_mark_but_not_a_fabricated_common_me() {
    let bytes = mark_font();
    mark_premises(&bytes);
    let categories = CodePointMapData::<GeneralCategory>::new();
    let scripts = ScriptWithExtensions::new();
    // Unicode17 supplies no Common-SCX Me. Inherited U20DD must retain X's
    // actual R property; it is not evidence for the abstract Common-Me branch.
    assert_eq!(categories.get('\u{20dd}'), GeneralCategory::EnclosingMark);
    assert_eq!(
        CodePointMapData::<Script>::new().get('\u{20dd}'),
        Script::Inherited
    );
    assert!(!scripts.has_script32(0x20dd, Script::Common));
    for (text, basis, advance) in [
        ("X\u{20dd}", GlyphBasis::Clockwise, 10.0),
        (" \u{3099}", GlyphBasis::Unrotated, 12.0),
    ] {
        assert_eq!(
            CodePointMapData::<VerticalOrientation>::new().get(' '),
            VerticalOrientation::Rotated
        );
        assert_eq!(
            CodePointMapData::<VerticalOrientation>::new().get('\u{3099}'),
            VerticalOrientation::Upright
        );
        let prepared =
            prepare_source(&mut system_with(&bytes), Source::new(text), style()).unwrap();
        assert_eq!(
            prepared.analyzed_source().graphemes().collect::<Vec<_>>(),
            [Range::new(0, 2)]
        );
        let control = native(
            &bytes,
            text,
            basis == GlyphBasis::Unrotated,
            surgeist_shaper::script::LATIN,
            &[],
        );
        assert_eq!(control.iter().map(|row| row.2).sum::<f64>(), advance);
        let line = finish(&prepared);
        assert!(packets(&line, &bytes).iter().all(|row| row.1 == basis));
        assert_eq!(line.feedback().advance, advance);
        let source = prepared.analyzed_source().source().clone();
        copy_source(&place_upright(prepared, line), &source);
    }
}

#[test]
fn mixed_ogham_rotated_role_does_not_borrow_the_whole_upright_intrinsic_refusal() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('\u{1681}'),
        VerticalOrientation::Rotated
    );
    assert_eq!(
        native(
            &bytes,
            "\u{1681}",
            false,
            surgeist_shaper::script::OGHAM,
            &[]
        ),
        [(274, 0, 10.0, 0.0, 0.0)]
    );
    let (_, line) = composed_line(
        &mut system_with(&bytes),
        "\u{1681}一",
        &[],
        AnalysisTypography::Mixed,
        LineOrientation::Vertical,
        ParagraphBaseDirection::LeftToRight,
    );
    assert_eq!(line.feedback().advance, 22.0);
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|row| row.1)
            .collect::<Vec<_>>(),
        [GlyphBasis::Clockwise, GlyphBasis::Unrotated]
    );
    let captured = AnalyzedSource::new(
        Source::new("\u{1681}"),
        style(),
        AnalysisTypography::Upright {
            computed_direction: ParagraphBaseDirection::LeftToRight,
        },
    )
    .unwrap();
    let error = system_with(&bytes)
        .prepare(
            &captured,
            PreparationOptions::default().with_orientation(LineOrientation::Vertical),
        )
        .unwrap_err();
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::UprightScript {
            reason: UprightScriptFailure::Intrinsic { resolved: None },
            ..
        })
    ));
}

#[test]
fn mixed_decoration_demands_follow_the_actual_role_and_common_central() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    let post = sfnt::table(&bytes, *b"post");
    assert_eq!(i16::from_be_bytes(post[8..10].try_into().unwrap()), -133);
    assert_eq!(i16::from_be_bytes(post[10..12].try_into().unwrap()), 20);
    let mut source = Source::new("X一");
    let mut decorated = style();
    decorated.underline = Decoration::solid();
    source.span(Range::new(0, 1), decorated.clone());
    let prepared = prepare_source(&mut system_with(&bytes), source, style()).unwrap();
    let layout = place_upright(prepared.clone(), finish(&prepared));
    let decorations = layout.decorations();
    assert_eq!(decorations.len(), 1);
    assert_eq!(decorations[0].kind(), DecorationKind::Underline);
    let rect = decorations[0].rect();
    // Raw underline -1.33 rebased by c4; thickness .2 extends toward under.
    close(rect.origin.x, 100.0 - 1.33 - 4.0 - 0.2);
    close(rect.origin.y, 20.0);
    close(rect.size.width, 0.2);
    close(rect.size.height, 10.0);
    for (text, range) in [
        ("X一", Range::new(1, 2)),
        ("X一\u{301}", Range::new(2, 3)),
        ("X一", Range::new(0, 2)),
    ] {
        let mut source = Source::new(text);
        source.span(range, decorated.clone());
        let error = prepare_source(&mut system_with(&bytes), source, style()).unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::UprightDecoration {
                owner: TextStyleOwner::Source { range },
                kind: DecorationKind::Underline,
                role: UprightDecorationRole::OffsetAndThickness,
            })
        );
    }
    let mut explicit = style();
    explicit.underline = Decoration::solid()
        .with_offset(DecorationOffset::try_absolute(2.0).unwrap())
        .with_thickness(DecorationThickness::try_absolute(1.0).unwrap());
    let prepared = prepare_source(&mut system_with(&bytes), Source::new("X一"), explicit).unwrap();
    let layout = place_upright(prepared.clone(), finish(&prepared));
    assert_eq!(
        layout
            .decorations()
            .iter()
            .map(|d| d.rect())
            .collect::<Vec<_>>(),
        [
            Rect::new(101.0, 20.0, 1.0, 10.0),
            Rect::new(101.0, 30.0, 1.0, 12.0)
        ]
    );
}

#[test]
fn mixed_only_actual_upright_glyph_demand_consumes_malformed_vertical_metrics() {
    let good = mixed_fonts::fixture(true);
    premises(&good, true);
    let bytes = sfnt::replace_tables(&good, vec![(*b"vmtx", vec![0])]);
    assert_eq!(
        native(&bytes, "X", false, surgeist_shaper::script::LATIN, &[]),
        [(274, 0, 10.0, 0.0, 0.0)]
    );
    let mut system = system_with(&bytes);
    let prior = prepare_source(&mut system, Source::new("X"), style()).unwrap();
    let natural = finish(&prior);
    assert_eq!(natural.feedback().advance, 10.0);
    let placed = place_upright(prior.clone(), natural.clone());
    let old_bounds = placed.bounds();
    let old_glyphs = placed.glyph_runs();
    let error = prepare_source(&mut system, Source::new("X一"), style()).unwrap_err();
    assert_eq!(error.code, crate::ErrorCode::InvalidFontMetrics);
    assert!(matches!(
        error.detail(),
        Some(ErrorDetail::VerticalMetricSupplier { glyph_id: 82, .. })
    ));
    let cause = error
        .source
        .as_deref()
        .unwrap()
        .downcast_ref::<crate::Error>()
        .unwrap();
    assert!(matches!(
        cause.detail(),
        Some(ErrorDetail::FontMetricTable {
            table: FontMetricTable::Vmtx,
            reason: FontMetricTableError::Malformed
        })
    ));
    assert_eq!(placed.bounds(), old_bounds);
    assert_eq!(placed.glyph_runs(), old_glyphs);
    assert_eq!(
        natural
            .with_spacing(SpacingAllocations::None)
            .unwrap()
            .feedback()
            .advance,
        10.0
    );
}

#[cfg(feature = "text-accessibility")]
#[test]
fn mixed_accessibility_keeps_source_and_vertical_cell_extents_separate_from_outline_axes() {
    let bytes = mixed_fonts::fixture(true);
    let (prepared, line) = composed_line(
        &mut system_with(&bytes),
        "X一",
        &[],
        AnalysisTypography::Mixed,
        LineOrientation::Vertical,
        ParagraphBaseDirection::LeftToRight,
    );
    let analyzed = prepared.analyzed_source().clone();
    let layout = place_upright(prepared, line);
    let parent = accesskit::NodeId(1000);
    let logical = analyzed
        .accessibility_update(&mut crate::Accessibility::new(10), parent, parent)
        .unwrap();
    assert_eq!(
        logical
            .nodes
            .iter()
            .filter_map(|(_, node)| node.value())
            .collect::<String>(),
        "X一"
    );
    assert!(
        logical
            .nodes
            .iter()
            .all(|(_, node)| node.bounds().is_none())
    );
    let mut state = crate::Accessibility::new(20);
    let placed = layout
        .accessibility_update(&mut state, parent, parent, Point::default())
        .unwrap();
    let text = placed
        .nodes
        .iter()
        .filter(|(_, node)| node.role() == accesskit::Role::TextRun)
        .map(|(_, node)| node)
        .collect::<Vec<_>>();
    assert_eq!(
        text.iter()
            .filter_map(|node| node.value())
            .collect::<String>(),
        "X一"
    );
    assert_eq!(
        text.iter()
            .map(|node| node.bounds().unwrap().height())
            .collect::<Vec<_>>(),
        [10.0, 12.0]
    );
    assert!(
        text.iter()
            .all(|node| node.text_direction() == Some(accesskit::TextDirection::TopToBottom))
    );
    assert!(
        layout
            .access_position(&state, Cursor::new(0, Affinity::After))
            .unwrap()
            .is_some()
    );
}

fn place_with_style(bytes: &[u8], source: Source, own: Style) -> crate::Result<crate::Layout> {
    let prepared = prepare_source(&mut system_with(bytes), source, own)?;
    place_prepared(prepared, None)
}
fn place_prepared(
    prepared: PreparedText,
    strut: Option<crate::LineStrut>,
) -> crate::Result<crate::Layout> {
    let mut value = request(&prepared);
    value.line_strut = strut;
    value.context.orientation = LineOrientation::Vertical;
    value.context.base_direction = ParagraphBaseDirection::LeftToRight;
    value.context.line_box_direction = LineBoxDirection::LeftToRight;
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::None;
    let line = prepared.select(value)?.finalize(SpacingAllocations::None)?;
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
}

#[test]
fn mixed_metric_only_upright_decoration_refuses_without_rejecting_visible_rotated_text() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('\u{2065}'),
        VerticalOrientation::Upright
    );
    let face = surgeist_shaper::FontRef::new(&bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut input = surgeist_shaper::UnicodeBuffer::new();
    input.add('\u{2065}', 91);
    input.set_direction(surgeist_shaper::Direction::TopToBottom);
    input.set_script(surgeist_shaper::script::LATIN);
    let native = shaper.shape(input, &[]);
    assert_eq!(
        native.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    let receipt = native.default_ignorable_receipt().unwrap();
    assert_eq!(receipt.residual_input_positions(), [0]);
    assert!(receipt.deleted_input_positions().is_empty());

    let mut own = style();
    own.underline = Decoration::solid();
    let rotated = place_with_style(&bytes, Source::new("X"), own.clone()).unwrap();
    assert_eq!(rotated.decorations().len(), 1);
    close(rotated.decorations()[0].rect().size.height, 10.0);
    let mut removed = Source::new("\u{2065}");
    removed.span(Range::new(0, 1), own.clone());
    let undecorated = place_with_style(&bytes, removed, style()).unwrap();
    assert!(undecorated.decorations().is_empty());
    assert_eq!(
        undecorated
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
    for text in ["\u{2065}", ""] {
        let error = place_with_style(&bytes, Source::new(text), own.clone()).unwrap_err();
        assert_eq!(
            error.detail(),
            Some(&ErrorDetail::UprightDecoration {
                owner: TextStyleOwner::Paragraph,
                kind: DecorationKind::Underline,
                role: UprightDecorationRole::OffsetAndThickness,
            })
        );
    }
}

#[test]
fn mixed_removed_source_fallback_does_not_invent_a_decoration_demand() {
    let bytes = mixed_fonts::fixture(true);
    let mut own = style();
    own.underline = Decoration::solid();
    let source = Source::new("\u{2065}");
    let prepared = prepare_source_with_strut(
        &mut system_with(&bytes),
        source.clone(),
        own,
        ParagraphStrut::AllocatedContributions,
    )
    .unwrap();
    let layout = place_prepared(
        prepared,
        Some(crate::LineStrut {
            baseline: 6.0,
            line_extent: 12.0,
        }),
    )
    .unwrap();
    assert_eq!(layout.source(), &source);
    assert!(layout.decorations().is_empty());
    assert_eq!(
        layout
            .run_views()
            .map(|run| run.glyphs().count())
            .sum::<usize>(),
        0
    );
}
