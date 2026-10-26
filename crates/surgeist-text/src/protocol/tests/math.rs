use super::*;
use crate::{CompleteTextNodeContribution, Range, Text, TextTransform};

fn style() -> Style {
    Style {
        font: Font::new().family("Ahem"),
        size: 10.0,
        line_height: crate::LineHeight::Absolute(10.0),
        text_transform: TextTransform::MathAuto,
        white_space: WhiteSpace::Preserve,
        overflow_wrap: crate::OverflowWrap::Anywhere,
        ..Style::default()
    }
}
fn analyzed(text: &str, complete: &str) -> AnalyzedSource {
    let source = Source::new(text);
    let len = source.text().len();
    AnalyzedSource::from_paragraph(ParagraphInput {
        source: source.clone(),
        base_style: style(),
        paragraph_strut: ParagraphStrut::DefaultFont,
        terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: vec![paragraph::ParagraphOrigin {
            source,
            local: Range::new(0, len),
            aggregate: Range::new(0, len),
            kind: paragraph::ParagraphOriginKind::Authored,
            text_node: Some(
                CompleteTextNodeContribution::new(Text::from(complete), Range::new(0, len))
                    .unwrap(),
            ),
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap()
}
fn system() -> System {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(crate::shaping_test_fonts::math_ahem())
        .unwrap();
    system
}
fn prepared(system: &mut System, a: &AnalyzedSource) -> PreparedText {
    system
        .prepare(
            &crate::test_wrapping::bind_missing(a, crate::Wrap::Word),
            PreparationOptions::default(),
        )
        .unwrap()
}
fn line(p: &PreparedText) -> FinalizedLine {
    p.select(request(p))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32)> {
    line.run_views()
        .flat_map(|run| {
            run.glyphs()
                .map(|g| (g.id(), g.advance()))
                .collect::<Vec<_>>()
        })
        .collect()
}

#[test]
fn native_math_uses_actual_target_font_metrics_and_exact_original_selection() {
    for (original, target, glyph, advance) in [
        ("A", "𝐴", 36, 20.0),
        ("c", "𝑐", 82, 15.0),
        ("h", "ℎ", 100, 18.0),
        ("ı", "𝚤", 277, 17.0),
        ("α", "𝛼", 274, 16.0),
        ("ϴ", "𝛳", 37, 14.0),
        ("∂", "𝜕", 38, 13.0),
    ] {
        let a = analyzed(original, original);
        assert_eq!(a.inner.projection.text, target);
        let p = prepared(&mut system(), &a);
        let unit = p.unit_views().next().unwrap();
        assert_eq!(unit.source_range(), Range::new(0, 1));
        assert_eq!(
            (
                unit.processed_range().start().get(),
                unit.processed_range().end().get()
            ),
            (0, 1)
        );
        assert_eq!(
            p.analyzed_source().inner.projection.presentation_end(),
            target.encode_utf16().count()
        );
        let origin = p.origin_views().next().unwrap().handle();
        let line = line(&p);
        assert_eq!(glyphs(&line), [(glyph, advance as f32)]);
        assert_eq!(line.feedback().advance, advance);
        for run in line.run_views() {
            assert_eq!(run.face().bytes(), crate::shaping_test_fonts::math_ahem());
        }
        let placed = place(p, line, 0.0, 8.0).unwrap();
        assert_eq!(placed.source().text(), &Text::from(original));
        let queried = placed
            .source_range_fragments(&origin, Range::new(0, 1))
            .unwrap();
        assert_eq!(queried.units().len(), 1);
        let positions: Vec<_> = placed.positions().collect();
        assert_eq!(
            positions.len(),
            2,
            "supplementary scalar has no interior caret"
        );
        let selection = placed
            .position_selection(positions.first().unwrap(), positions.last().unwrap())
            .unwrap();
        assert_eq!(selection.source_text(), &Text::from(original));
        assert_eq!(selection.plaintext(), &Text::from(original));
        assert_eq!(selection.geometry().rects()[0].rect().size.width, advance);
        assert_eq!(
            placed
                .position_geometry(positions.last().unwrap())
                .unwrap()
                .rect()
                .origin
                .x,
            advance
        );
    }
}

#[test]
fn capture_context_identity_survives_repeated_preparation_and_does_not_reuse_equal_source_results()
{
    let mut warm = system();
    for complete in ["c", "cos", "c"] {
        let a = analyzed("c", complete);
        let p = prepared(&mut warm, &a);
        let first = line(&p);
        let second = line(&prepared(&mut warm, &a));
        let fresh = line(&prepared(&mut system(), &a));
        let expected = if complete == "c" {
            vec![(82, 15.0)]
        } else {
            vec![(69, 10.0)]
        };
        assert_eq!(glyphs(&first), expected);
        assert_eq!(glyphs(&second), expected);
        assert_eq!(glyphs(&fresh), expected);
        assert_eq!(a.source().text(), &Text::from("c"));
    }
}

#[test]
#[cfg(feature = "text-accessibility")]
fn native_math_accessibility_retains_original_character_and_transformed_geometry() {
    let p = prepared(&mut system(), &analyzed("c", "c"));
    let finalized = line(&p);
    assert_eq!(glyphs(&finalized), [(82, 15.0)]);
    let placed = place(p, finalized, 0.0, 8.0).unwrap();
    let mut state = crate::Accessibility::new(2163);
    let parent = 1001_u64.into();
    let update = placed
        .accessibility_update(&mut state, parent, parent, crate::Point::default())
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
    assert_eq!(node.value(), Some("c"));
    assert_eq!(node.character_lengths(), [1]);
    let bounds = node.bounds().unwrap();
    assert_eq!((bounds.x0, bounds.x1), (0.0, 15.0));
}

#[test]
fn raw_math_refusal_cannot_reuse_a_successful_neutral_standalone_cache_entry() {
    let mut warm = system();
    let neutral = Style {
        text_transform: TextTransform::none(),
        ..style()
    };
    for _ in 0..2 {
        let actual = warm
            .layout(Source::new("c"), neutral.clone(), crate::Options::default())
            .unwrap();
        assert_eq!(
            actual
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs())
                .map(|glyph| f64::from(glyph.advance()))
                .sum::<f64>(),
            10.0
        );
        for text in ["c", ""] {
            let error = warm
                .layout(Source::new(text), style(), crate::Options::default())
                .unwrap_err();
            assert!(matches!(
                error.detail(),
                Some(crate::ErrorDetail::UnsupportedTextStyle {
                    feature: crate::TextStyleFeature::MathAuto,
                    reason: crate::UnsupportedTextStyleReason::RequiresCompleteTextNodeContext
                })
            ));
        }
    }
}
