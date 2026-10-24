use crate::{AnalyzedSource, Source, Style, Text, TextTransform};

mod casing;
mod width;

fn style() -> Style {
    Style {
        text_transform: TextTransform::none().with_full_size_kana(true),
        ..Style::default()
    }
}

#[test]
fn all_pinned_kana_mappings_and_unmapped_controls_execute_before_analysis() {
    // Independent literal vectors from normative CSS Appendix G, not the lookup.
    for (original, expected) in [
        (
            "ぁぃぅぇぉゕゖ𛄲っゃゅょゎ𛅐𛅑𛅒",
            "あいうえおかけこつやゆよわゐゑを",
        ),
        (
            "ァィゥェォヵㇰヶ𛅕ㇱㇲッㇳㇴㇵㇶㇷㇸㇹㇺャュョㇻㇼㇽㇾㇿヮ𛅤𛅥𛅦𛅧",
            "アイウエオカクケコシスツトヌハヒフヘホムヤユヨラリルレロワヰヱヲン",
        ),
        ("ｧｨｩｪｫｯｬｭｮ", "ｱｲｳｴｵﾂﾔﾕﾖ"),
        (
            "あアｱゔヴゝヽー\u{3099}\u{309a}A1\u{1b000}",
            "あアｱゔヴゝヽー\u{3099}\u{309a}A1\u{1b000}",
        ),
    ] {
        let source = Source::new(original);
        let result = AnalyzedSource::new(source.clone(), style()).unwrap();
        assert_eq!(result.source(), &source);
        assert_eq!(result.processed_source().processed_text(), source.text());
        assert_eq!(result.inner.projection.text, expected);
        assert_eq!(
            result.inner.projection.presentation_end(),
            expected.encode_utf16().count()
        );
        assert_eq!(
            result.inner.projection.atoms.len(),
            original.chars().count()
        );
        for (atom, ch) in result.inner.projection.atoms.iter().zip(expected.chars()) {
            assert_eq!(atom.code_point, u32::from(ch));
            assert_eq!(atom.glyph, ch);
            assert_eq!(atom.processed, atom.source);
        }
    }
}

#[test]
fn contraction_preserves_phase_i_and_original_while_presentation_is_shorter() {
    let source = Source::new("A\u{1b132}C");
    let result = AnalyzedSource::new(source.clone(), style()).unwrap();
    let projection = &result.inner.projection;
    assert_eq!(source.text().code_units(), [0x41, 0xd82c, 0xdd32, 0x43]);
    assert_eq!(result.processed_source().processed_text().len(), 4);
    assert_eq!(projection.presentation_end(), 3);
    assert_eq!(projection.atoms[1].source, crate::Range::new(1, 3));
    assert_eq!(projection.atoms[1].processed, crate::Range::new(1, 3));
    assert_eq!(projection.atoms[1].presentation, crate::Range::new(1, 2));
    assert_eq!(projection.graphemes, [0, 1, 2, 3]);
    assert_eq!(result.inner.analysis.graphemes, [0..1, 1..2, 2..3]);
    assert_eq!(result.inner.host_projection.text, "A\u{1b132}C");
}

#[test]
fn actual_input_styles_and_isolated_surrogates_keep_their_own_associations() {
    let mut source = Source::new(Text::from_code_units(vec![0x3041, 0xd800, 0xfffd, 0x3043]));
    source.span(crate::Range::new(3, 4), Style::default());
    let analyzed = AnalyzedSource::new(source.clone(), style()).unwrap();
    assert_eq!(analyzed.source(), &source);
    assert_eq!(analyzed.inner.projection.text, "あ\u{fffd}\u{fffd}ぃ");
    assert_eq!(analyzed.inner.projection.segmentation, "あ\u{a6}\u{fffd}ぃ");
    assert_eq!(
        analyzed
            .inner
            .projection
            .atoms
            .iter()
            .map(|a| a.code_point)
            .collect::<Vec<_>>(),
        [0x3042, 0xd800, 0xfffd, 0x3043]
    );
}

#[test]
fn native_whitespace_plaintext_and_style_winners_precede_semantic_transform() {
    for (whitespace, expected, plaintext) in [
        (crate::WhiteSpace::Collapse, "あ い", "ぁ ぃ"),
        (crate::WhiteSpace::Preserve, "あ  い", "ぁ  ぃ"),
    ] {
        let source = Source::new("ぁ  ぃ");
        let analyzed = AnalyzedSource::from_paragraph(crate::ParagraphInput {
            source: source.clone(),
            base_style: Style {
                white_space: whitespace,
                ..style()
            },
            paragraph_strut: crate::ParagraphStrut::DefaultFont,
            terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: crate::ParagraphBaseDirection::LeftToRight,
            origins: vec![crate::ParagraphOrigin {
                source: source.clone(),
                local: crate::Range::new(0, 4),
                aggregate: crate::Range::new(0, 4),
                kind: crate::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        })
        .unwrap();
        assert_eq!(analyzed.inner.projection.text, expected);
        let actual: Vec<_> = analyzed
            .processed_source()
            .plaintext_items(crate::Affinity::After)
            .flat_map(|item| match item {
                crate::PlaintextItem::Text(text) => text.code_units().to_vec(),
                crate::PlaintextItem::InlineBox(_) => vec![],
            })
            .collect();
        assert_eq!(actual, plaintext.encode_utf16().collect::<Vec<_>>());
        assert_eq!(analyzed.source(), &source);
    }
    assert_eq!(
        crate::TextStyleFeature::FullSizeKana.support(crate::TextStylePhase::Standalone),
        crate::TextStyleSupport::Supported
    );
    assert_eq!(
        crate::TextStyleFeature::FullSizeKana.support(crate::TextStylePhase::ProcessedParagraph),
        crate::TextStyleSupport::Supported
    );
}
