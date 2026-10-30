//! Native horizontal context and scalar tab resources inside a Mixed frame.
use super::*;
use icu_properties::props::BidiClass;

fn rtl_native(
    bytes: &[u8],
    text: &str,
    script: surgeist_shaper::Script,
    authored: &[&str],
) -> Vec<(u32, u32, i32)> {
    let face = surgeist_shaper::FontRef::new(bytes).unwrap();
    let data = surgeist_shaper::ShaperData::new(&face);
    let shaper = data.shaper(&face).build();
    let mut buffer = surgeist_shaper::UnicodeBuffer::new();
    for (ordinal, scalar) in text.chars().enumerate() {
        buffer.add(scalar, u32::try_from(ordinal).unwrap());
    }
    buffer.set_direction(surgeist_shaper::Direction::RightToLeft);
    buffer.set_script(script);
    buffer.observe_components();
    let mut features = vec![surgeist_shaper::Feature::new(
        surgeist_shaper::Tag::new(b"vrtr"),
        1,
        ..,
    )];
    features.extend(
        authored
            .iter()
            .map(|feature| feature.parse::<surgeist_shaper::Feature>().unwrap()),
    );
    let output = shaper.shape(buffer, &features);
    assert_eq!(
        output.shaping_status(),
        surgeist_shaper::ShapingStatus::Complete
    );
    if script == surgeist_shaper::script::ARABIC && authored.is_empty() {
        let receipt = output.component_receipt().unwrap().unwrap();
        let relations = receipt.ligatures().collect::<Vec<_>>();
        assert_eq!(relations.len(), 1);
        assert_eq!(relations[0].components(), [0, 1]);
        assert!(
            relations[0]
                .provenance()
                .iter()
                .any(|row| row.tag() == surgeist_shaper::Tag::new(b"ccmp")
                    && row.role() == surgeist_shaper::FeatureRole::Required)
        );
    }
    output
        .glyph_infos()
        .iter()
        .zip(output.glyph_positions())
        .map(|(info, pos)| (info.glyph_id, info.cluster, pos.x_advance))
        .collect()
}

#[test]
fn mixed_ordinary_rtl_keeps_han_and_digits_even_and_horizontal_hebrew_reversed_once() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    let classes = CodePointMapData::<BidiClass>::new();
    assert_eq!(classes.get('א'), BidiClass::RightToLeft);
    assert_eq!(classes.get('一'), BidiClass::LeftToRight);
    assert_eq!(classes.get('1'), BidiClass::EuropeanNumber);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('1'),
        VerticalOrientation::Rotated
    );
    assert_eq!(
        rtl_native(&bytes, "אב", surgeist_shaper::script::HEBREW, &[]),
        [(274, 1, 1000), (274, 0, 1000)]
    );
    assert_eq!(
        native(&bytes, "12", false, surgeist_shaper::script::LATIN, &[]),
        [(35, 0, 10.0, 0.0, 0.0), (35, 1, 10.0, 0.0, 0.0)]
    );
    // W7 takes the numbers' preceding Han L, and I1/I2 yield1,1,2,2,2.
    // L2 consequently gives Han,1,2,Bet,Alef, not a global string reversal.
    let (prepared, line) = composed_line(
        &mut system_with(&bytes),
        "אב一12",
        &[],
        AnalysisTypography::Mixed,
        LineOrientation::Vertical,
        ParagraphBaseDirection::RightToLeft,
    );
    assert_eq!(
        prepared
            .unit_views()
            .map(|unit| unit.level().get())
            .collect::<Vec<_>>(),
        [1, 1, 2, 2, 2]
    );
    assert_eq!(
        packets(&line, &bytes)
            .iter()
            .map(|p| (p.0, p.1, p.2))
            .collect::<Vec<_>>(),
        [
            (Range::new(2, 3), GlyphBasis::Unrotated, 82),
            (Range::new(3, 4), GlyphBasis::Clockwise, 35),
            (Range::new(4, 5), GlyphBasis::Clockwise, 35),
            (Range::new(1, 2), GlyphBasis::Clockwise, 274),
            (Range::new(0, 1), GlyphBasis::Clockwise, 274),
        ]
    );
    assert_eq!(line.feedback().advance, 52.0);
    let source = prepared.analyzed_source().source().clone();
    copy_source(&place_upright(prepared, line), &source);
}

#[test]
fn mixed_compatible_arabic_r_units_keep_actual_horizontal_context_and_explicit_ccmp_override() {
    let mut mappings = mixed_fonts::mappings();
    mappings.extend([(0x628, 35), (0x64e, 100), (0x200c, 252), (0x200d, 253)]);
    let bytes = mixed_fonts::with_cmap(&mixed_fonts::with_metrics(&arabic_font(), true), &mappings);
    assert_eq!(
        CodePointMapData::<VerticalOrientation>::new().get('ب'),
        VerticalOrientation::Rotated
    );
    let raw = skrifa::FontRef::new(&bytes).unwrap();
    for id in [35, 58, 82] {
        assert_eq!(raw.hmtx().unwrap().h_metrics()[id].advance.get(), 1000);
        assert_eq!(raw.vmtx().unwrap().v_metrics()[id].advance.get(), 1200);
    }
    assert_eq!(
        rtl_native(&bytes, "بب", surgeist_shaper::script::ARABIC, &[]),
        [(58, 0, 1000)]
    );
    assert_eq!(
        rtl_native(
            &bytes,
            "بب",
            surgeist_shaper::script::ARABIC,
            &[r#""ccmp" 0"#]
        ),
        [(58, 1, 1000), (82, 0, 1000)]
    );
    for (features, expected, total) in [
        (
            &[][..],
            vec![
                (Range::new(2, 3), GlyphBasis::Unrotated, 58),
                (Range::new(0, 2), GlyphBasis::Clockwise, 58),
            ],
            22.0,
        ),
        (
            &[r#""ccmp" 0"#][..],
            vec![
                (Range::new(2, 3), GlyphBasis::Unrotated, 58),
                (Range::new(1, 2), GlyphBasis::Clockwise, 58),
                (Range::new(0, 1), GlyphBasis::Clockwise, 82),
            ],
            32.0,
        ),
    ] {
        let (prepared, line) = composed_line(
            &mut system_with(&bytes),
            "بب一",
            features,
            AnalysisTypography::Mixed,
            LineOrientation::Vertical,
            ParagraphBaseDirection::RightToLeft,
        );
        assert_eq!(
            packets(&line, &bytes)
                .iter()
                .map(|p| (p.0, p.1, p.2))
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(line.feedback().advance, total);
        let source = prepared.analyzed_source().source().clone();
        copy_source(&place_upright(prepared, line), &source);
    }
}

#[test]
fn mixed_css_tab_uses_the_actual_horizontal_scalar_space_resource() {
    let bytes = mixed_fonts::fixture(true);
    premises(&bytes, true);
    let face = skrifa::FontRef::new(&bytes).unwrap();
    assert_eq!(face.charmap().map(' '), Some(GlyphId::new(3)));
    assert_eq!(face.hmtx().unwrap().h_metrics()[3].advance.get(), 1000);
    assert_eq!(face.vmtx().unwrap().v_metrics()[3].advance.get(), 1200);
    assert_eq!(
        native(&bytes, " ", false, surgeist_shaper::script::LATIN, &[]),
        [(3, 0, 10.0, 0.0, 0.0)]
    );
    let mut own = Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: LineHeight::Absolute(12.0),
        white_space: WhiteSpace::Preserve,
        ..Style::default()
    };
    own.tab_size = crate::TabSize::spaces(2.0).unwrap();
    let source = Source::new("X\t一");
    let extent = Range::new(0, source.text().len());
    let value = ParagraphInput {
        source: source.clone(),
        base_style: own.clone(),
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: Some(
            crate::ParagraphTabReference::new(own, ParagraphBaseDirection::LeftToRight).unwrap(),
        ),
        origins: vec![crate::ParagraphOrigin {
            text_node: None,
            source: source.clone(),
            local: extent,
            aggregate: extent,
            kind: crate::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(value, AnalysisTypography::Mixed).unwrap(),
        crate::Wrap::Word,
    );
    let mut system = system_with(&bytes);
    let prepared = system
        .prepare(
            &analyzed,
            PreparationOptions::try_new(1.0, crate::TabPreparation::CssStops)
                .unwrap()
                .with_orientation(LineOrientation::Vertical),
        )
        .unwrap();
    let mut value = request(&prepared);
    value.context.orientation = LineOrientation::Vertical;
    value.context.available_inline = None;
    value.context.spacing = SpacingPolicy::None;
    value.context.tabs = TabContext::CssStops { origin: Some(0.0) };
    let selected = prepared.select(value).unwrap();
    let line = selected
        .finalize(crate::LineAllocations {
            spacing: SpacingAllocations::None,
            scopes: crate::ScopeAllocations::None,
        })
        .unwrap();
    // R X10 reaches the next2*SPACE10 stop20, followed by upright Han12.
    assert_eq!(line.feedback().advance, 32.0);
    assert_eq!(
        packets(&line, &bytes),
        [
            (
                Range::new(0, 1),
                GlyphBasis::Clockwise,
                274,
                0.0,
                0.0,
                0.0,
                10.0
            ),
            (
                Range::new(2, 3),
                GlyphBasis::Unrotated,
                82,
                20.0,
                -6.0,
                1.0,
                12.0
            ),
        ]
    );
    let restored = line.with_spacing(SpacingAllocations::None).unwrap();
    assert_eq!(restored.feedback().advance, 32.0);
    assert_eq!(packets(&restored, &bytes), packets(&line, &bytes));
    copy_source(&place_upright(prepared, line), &source);
}
