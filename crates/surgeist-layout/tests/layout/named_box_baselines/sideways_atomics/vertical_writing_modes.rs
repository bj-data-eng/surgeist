//! The same clockwise preparation keeps its Text basis in both vertical WMs.
use super::*;
mod supplemental;

fn qualify(parent: &Captured) {
    let p = &parent.prepared;
    assert_eq!(
        p.orientation(),
        t::selection::LineOrientation::SidewaysClockwise
    );
    assert_eq!(
        p.analyzed_source().typography(),
        t::analyzed::AnalysisTypography::HorizontalComposition
    );
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == t::prepared::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = p
        .select(t::selection::LineSelectionRequest {
            inline_boxes: None,
            segment: p.source_segments().next().unwrap(),
            start: p.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: t::selection::PreparedTermination::Ordinary(end),
            participants: p.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::selection::LineContext {
                alignment_insets: t::allocation::LineAlignmentInsets::ZERO,
                formatted_line: t::selection::FormattedLineContext::default(),
                base_direction: t::paragraph::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::selection::LineBoxDirection::LeftToRight,
                orientation: t::selection::LineOrientation::SidewaysClockwise,
                start: t::selection::LineStart::Paragraph,
                end: t::selection::LineEnd::Paragraph,
                available_inline: Some(40.0),
                indent: 0.0,
                alignment: t::options::Alignment::Start,
                tabs: t::tabs::TabContext::NoTabs,
                spacing: t::spacing::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(surgeist_text::scope_alignment::LineAllocations {
            spacing: t::spacing::SpacingAllocations::None,
            scopes: surgeist_text::scope_alignment::ScopeAllocations::None,
        })
        .unwrap();
    assert_eq!(
        (
            line.feedback().advance,
            line.feedback().baseline,
            line.feedback().line_height
        ),
        (20.0, 8.0, 10.0)
    );
    let glyphs = line
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Clockwise);
            assert_eq!(run.face().bytes(), AHEM);
            let metrics = run
                .effective_instance()
                .unwrap()
                .metrics(p.font_catalogue_identity())
                .unwrap();
            assert_eq!(
                (metrics.ascent(), metrics.descent(), metrics.leading()),
                (8.0, -2.0, 0.0)
            );
            run.glyphs()
                .map(|glyph| {
                    (
                        glyph.id(),
                        glyph.pen(),
                        glyph.x(),
                        glyph.y(),
                        glyph.advance(),
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [(35, 0.0, 0.0, 0.0, 10.0), (36, 10.0, 0.0, 0.0, 10.0)]
    );
    let text = t::layout::Layout::from_finalized_lines(t::placement::PlacementRequest {
        prepared: p.clone(),
        coverage: t::placement::PlacementCoverage::WholeParagraph,
        entries: vec![t::placement::PlacementEntry::Formatted(
            t::placement::LinePlacement {
                line,
                anchor: t::placement::LineAnchor {
                    inline_origin: 20.0,
                    baseline: 100.0,
                },
                orientation: t::selection::LineOrientation::SidewaysClockwise,
                atomics: vec![],
            },
        )],
    })
    .unwrap();
    assert_eq!(text.source(), &parent.source);
    assert_eq!(text.lines()[0].baseline(), 100.0);
    assert_eq!(
        text.run_views().next().unwrap().origin(),
        t::geometry::Point::new(100.0, 20.0)
    );
}

fn vertical_paragraph<S: l::scalar::LayoutScalar>() {
    let parent = capture(
        AHEM,
        "AB",
        None,
        t::paragraph::ParagraphBaseDirection::LeftToRight,
        t::selection::LineOrientation::SidewaysClockwise,
        10.0,
    );
    // Existing public Text preparation, selection, finalization and placement
    // succeed before the public Layout operation consumes this exact owner.
    qualify(&parent);
    for (mode, baselines, lefts) in [
        (
            l::geometry::WritingMode::VerticalRl,
            [32.0, 22.0],
            [30.0, 20.0],
        ),
        (
            l::geometry::WritingMode::VerticalLr,
            [2.0, 12.0],
            [0.0, 10.0],
        ),
    ] {
        let mut input = root(
            mode,
            l::geometry::Direction::Ltr,
            l::baseline::InlineBaselineContext::Physical,
        );
        input.size = l::geometry::Size::new(px(40.0), px(15.0));
        let mut host = Host::<S>::new(input);
        let origin = parent.prepared.origin_views().next().unwrap().handle();
        assert_eq!(
            host.push(
                Some(0),
                l::node::LayoutInputOf::inline_text(l::text_layout::TextParticipation::new(
                    origin.clone()
                ))
            ),
            1
        );
        let binding = l::text_layout::TextParagraphBinding {
            inline_box_fragments:
                surgeist_layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: parent.prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![l::text_layout::TextOriginBinding { origin, node: 1 }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        };
        let batch = run(&host, vec![binding], &[], available(40.0, 15.0))
            .expect("clockwise Text must retain its orientation in VerticalRl and VerticalLr");
        for entries in [
            crate::layout::observation::phase(&batch, false),
            crate::layout::observation::phase(&batch, true),
        ] {
            assert_eq!(
                node(entries, 0).size,
                l::geometry::Size::new(n(40.0), n(15.0))
            );
            unknown(node(entries, 0).named_baselines);
        }
        let text = placed(&batch, 0);
        assert_eq!(
            text.orientation(),
            t::selection::LineOrientation::SidewaysClockwise
        );
        assert_eq!(text.source(), &parent.source);
        assert_eq!(
            text.analyzed_source().identity(),
            parent.prepared.analyzed_source().identity()
        );
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.baseline())
                .collect::<Vec<_>>(),
            baselines
        );
        assert_eq!(
            text.lines()
                .iter()
                .map(|line| line.bounds())
                .collect::<Vec<_>>(),
            [
                t::geometry::Rect::new(lefts[0], 0.0, 10.0, 15.0),
                t::geometry::Rect::new(lefts[1], 0.0, 10.0, 15.0)
            ]
        );
        let glyphs = text
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), t::geometry::GlyphBasis::Clockwise);
                run.glyphs()
                    .map(|glyph| (glyph.id(), run.origin().x, run.origin().y + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [(35, baselines[0], 0.0), (36, baselines[1], 0.0)]);
        let mut lines = std::collections::BTreeMap::new();
        for row in batch.unrounded_inline_fragments() {
            if matches!(row.owner(), l::text_layout::InlineFragmentOwner::Node(1)) {
                let fragment = row.fragment();
                let range = if fragment.line_index() == 0 {
                    t::source::Range::new(0, 1)
                } else {
                    t::source::Range::new(1, 2)
                };
                let receipt = fragment.finalized_fragment();
                assert_eq!(receipt.view().aggregate_range(), Some(range));
                assert_eq!(receipt.view().local_range(), Some(range));
                lines
                    .entry(fragment.line_index())
                    .or_insert_with(|| fragment.finalized_fragment().finalized_line().clone());
            }
        }
        assert_eq!(lines.len(), 2);
        for line in lines.values() {
            assert_eq!(line.prepared_text(), parent.prepared);
            assert_eq!(
                line.selected_line().request().context.orientation,
                t::selection::LineOrientation::SidewaysClockwise
            );
            assert_eq!(
                (
                    line.feedback().advance,
                    line.feedback().baseline,
                    line.feedback().line_height
                ),
                (10.0, 8.0, 10.0)
            );
        }
    }
}

#[test]
fn clockwise_preparation_keeps_physical_columns_in_vertical_writing_modes_f32() {
    vertical_paragraph::<f32>();
}
#[test]
fn clockwise_preparation_keeps_physical_columns_in_vertical_writing_modes_f64() {
    vertical_paragraph::<f64>();
}
