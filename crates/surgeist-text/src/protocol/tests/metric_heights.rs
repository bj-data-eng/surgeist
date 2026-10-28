//! Actual component metrics preserve shared provider shaping across height spans.
use super::*;
use crate::{
    LineHeight, ParagraphConnection, ParagraphConnectionKind, ParagraphOrigin, ParagraphOriginKind,
    ParagraphTerminalEdge, Range,
};
const AHEM: &[u8] = include_bytes!("../../../tests/fixtures/fonts/ahem/Ahem.ttf");
fn height(value: LineHeight) -> Style {
    Style {
        size: 10.0,
        line_height: value,
        ..style()
    }
}
fn prepare(bytes: &[u8], text: &str, heights: [LineHeight; 2]) -> PreparedText {
    let mut source = Source::new(text);
    for (index, h) in heights.into_iter().enumerate() {
        source.span(Range::new(index, index + 1), height(h));
    }
    prepare_source(bytes, source, height(heights[0]), &[0, 1, 2])
}
fn prepare_source(bytes: &[u8], source: Source, base_style: Style, cuts: &[usize]) -> PreparedText {
    let mut system = System::new(SystemOptions {
        font_catalogue: FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(bytes.to_vec()).unwrap();
    let input = ParagraphInput {
        terminal_edge: ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        source: source.clone(),
        base_style,
        paragraph_strut: ParagraphStrut::DefaultFont,
        segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: ParagraphBaseDirection::LeftToRight,
        origins: cuts
            .windows(2)
            .map(|pair| ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: Range::new(pair[0], pair[1]),
                aggregate: Range::new(pair[0], pair[1]),
                kind: ParagraphOriginKind::Authored,
            })
            .collect(),
        connections: cuts[1..cuts.len() - 1]
            .iter()
            .map(|&at| ParagraphConnection {
                at,
                kind: ParagraphConnectionKind::Continuous,
            })
            .collect(),
        controls: vec![],
        boundary_policies: (0..cuts.len() - 2)
            .map(|index| crate::ParagraphBoundaryPolicy {
                definition: crate::ParagraphBoundaryDefinition::Connection(index),
                preceding_wrap: crate::Wrap::Word,
                following_wrap: crate::Wrap::Word,
                ancestor_wrap: crate::Wrap::Word,
            })
            .collect(),
        replacements: vec![],
        anchor_order: vec![],
    };
    let analyzed = crate::test_wrapping::bind_missing(
        &AnalyzedSource::from_paragraph(input, crate::AnalysisTypography::HorizontalComposition)
            .unwrap(),
        crate::Wrap::Word,
    );
    system
        .prepare(&analyzed, PreparationOptions::default())
        .unwrap()
}
fn finalize(p: &PreparedText) -> FinalizedLine {
    p.select(request(p))
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap()
}
fn glyphs(line: &FinalizedLine) -> Vec<(u32, f32, f32, f32, Range)> {
    line.run_views()
        .flat_map(|run| {
            assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
            run.glyphs()
                .map(|g| (g.id(), g.advance(), g.x(), g.y(), g.range()))
                .collect::<Vec<_>>()
        })
        .collect()
}
#[test]
fn height_only_spans_keep_joint_ligature_and_mark_shapes_with_owned_unit_and_fragment_metrics() {
    let ligature = crate::shaping_test_fonts::range_ligature_ahem(&[35, 35], &[(0x41, 35)]);
    let mark = crate::shaping_test_fonts::mark_ahem();
    for (font, text, expected) in [
        (
            ligature.as_slice(),
            "AA",
            vec![(58, 10.0, 0.0, 0.0, Range::new(0, 2))],
        ),
        (
            mark.as_slice(),
            "A\u{301}",
            vec![
                (35, 10.0, 0.0, 0.0, Range::new(0, 2)),
                // Base anchor4 minus mark anchor1 gives logical3; after base advance10, raw offset is-7.
                (100, 0.0, -7.0, -4.0, Range::new(0, 2)),
            ],
        ),
    ] {
        let control = finalize(&prepare(font, text, [LineHeight::Absolute(10.0); 2]));
        assert_eq!(glyphs(&control), expected);
        if text == "A\u{301}" {
            let mark = control
                .run_views()
                .find_map(|run| {
                    run.glyphs()
                        .find(|glyph| glyph.id() == 100)
                        .map(|mark| (mark.pen(), mark.pen() + f64::from(mark.x()), mark.y()))
                })
                .unwrap();
            assert_eq!(mark, (10.0, 3.0, -4.0));
        }
        for (heights, resolved) in [
            (
                [LineHeight::Absolute(10.0), LineHeight::Absolute(20.0)],
                [10.0, 20.0],
            ),
            (
                [LineHeight::Absolute(20.0), LineHeight::Absolute(10.0)],
                [20.0, 10.0],
            ),
            (
                [
                    LineHeight::Absolute(10.0),
                    LineHeight::FontSizeRelative(2.0),
                ],
                [10.0, 20.0],
            ),
            (
                [LineHeight::Absolute(10.0), LineHeight::MetricsRelative(2.0)],
                [10.0, 20.0],
            ),
            (
                [LineHeight::Absolute(2.0), LineHeight::Absolute(4.0)],
                [2.0, 4.0],
            ),
        ] {
            let resolved: [f64; 2] = resolved;
            let p = prepare(font, text, heights);
            assert_eq!(
                p.unit_views()
                    .map(|unit| unit.line_height())
                    .collect::<Vec<_>>(),
                resolved
            );
            if text == "A\u{301}" {
                assert_eq!(p.inner.analyzed.analysis.graphemes.len(), 1);
                assert_eq!(p.inner.analyzed.analysis.graphemes[0], 0..2);
            } else {
                assert_eq!(p.inner.analyzed.analysis.graphemes, [0..1, 1..2]);
            }
            let line = finalize(&p);
            assert_eq!(glyphs(&line), expected);
            // Same actual provider invocation and memberships, never per-style shaping.
            assert_eq!(line.inner.data.font_runs.len(), 1);
            assert_eq!(
                line.inner
                    .data
                    .clusters
                    .iter()
                    .map(|c| c.provider_atoms.clone())
                    .collect::<Vec<_>>(),
                control
                    .inner
                    .data
                    .clusters
                    .iter()
                    .map(|c| c.provider_atoms.clone())
                    .collect::<Vec<_>>()
            );
            assert!(line.run_views().all(|run| run.face().bytes() == font));
            let maximum = resolved[0].max(resolved[1]);
            let baseline = 8.0 + (maximum - 10.0) * 0.5;
            assert_eq!(
                (
                    line.feedback().baseline,
                    line.feedback().line_height,
                    line.feedback().advance
                ),
                (baseline, maximum, 10.0)
            );
            let fragments = line
                .fragment_views()
                .filter(|fragment| fragment.origin().is_some())
                .collect::<Vec<_>>();
            assert_eq!(fragments.len(), 2);
            for fragment in fragments {
                let range = fragment.aggregate_range().unwrap();
                assert_eq!(range.len(), 1);
                let bound = fragment.bounds().unwrap();
                let own_height = resolved[range.start];
                assert_eq!(
                    (bound.origin.y, bound.size.height),
                    (baseline - (8.0 + (own_height - 10.0) * 0.5), own_height)
                );
            }
            assert_eq!(
                p.analyzed_source().source().text().code_units(),
                text.encode_utf16().collect::<Vec<_>>()
            );
        }
    }
}
#[test]
fn raw_profiles_keep_their_rounding_policy_with_span_owned_height_maximum() {
    for quantize in [false, true] {
        for tall_first in [false, true] {
            let mut system = System::new(SystemOptions {
                font_catalogue: FontCatalogueMode::RegisteredOnly,
            })
            .unwrap();
            system.register_fonts(AHEM.to_vec()).unwrap();
            let mut source = Source::new("XX");
            source.span(
                if tall_first {
                    Range::new(0, 1)
                } else {
                    Range::new(1, 2)
                },
                height(LineHeight::Absolute(21.0)),
            );
            let layout = system
                .layout(
                    source.clone(),
                    height(LineHeight::Absolute(10.0)),
                    crate::Options {
                        quantize,
                        ..crate::Options::default()
                    },
                )
                .unwrap();
            assert_eq!(layout.source(), &source);
            let line = &layout.lines()[0];
            assert_eq!(line.bounds().size.height, 21.0);
            assert_eq!(line.baseline(), if quantize { 13.0 } else { 13.5 });
            assert_eq!(
                layout
                    .glyph_runs()
                    .iter()
                    .flat_map(|run| run.glyphs().iter().map(|g| (g.id(), g.advance())))
                    .collect::<Vec<_>>(),
                [(58, 10.0), (58, 10.0)]
            );
        }
    }
}

#[test]
fn short_continuation_does_not_inherit_an_unselected_tall_component() {
    let mut source = Source::new("X X");
    source.span(Range::new(0, 1), height(LineHeight::Absolute(20.0)));
    let p = prepare_source(AHEM, source, height(LineHeight::Absolute(10.0)), &[0, 2, 3]);
    let whole = finalize(&p);
    assert_eq!(
        (whole.feedback().baseline, whole.feedback().line_height),
        (13.0, 20.0)
    );
    let start = p
        .opportunity_views()
        .find(|offer| {
            offer.kind() == PreparedBreakKind::Allowed
                && offer.boundary().view().source_before().get() == 2
        })
        .unwrap()
        .boundary();
    let mut value = request(&p);
    value.start = start.clone();
    value.participants = p
        .participants()
        .skip(start.view().participant_offset())
        .collect();
    value.context.start = LineStart::Continuation;
    let suffix = p
        .select(value)
        .unwrap()
        .finalize(SpacingAllocations::None)
        .unwrap();
    assert_eq!(
        (
            suffix.feedback().advance,
            suffix.feedback().baseline,
            suffix.feedback().line_height
        ),
        (10.0, 8.0, 10.0)
    );
    assert_eq!(glyphs(&suffix), [(58, 10.0, 0.0, 0.0, Range::new(2, 3))]);
    assert_eq!(
        suffix
            .fragment_views()
            .filter(|f| f.origin().is_some())
            .map(|f| (f.aggregate_range(), f.bounds()))
            .collect::<Vec<_>>(),
        [(
            Some(Range::new(2, 3)),
            Some(crate::Rect::new(0.0, 0.0, 10.0, 10.0))
        )]
    );
}
