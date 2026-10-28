//! The same genuine clockwise preparation keeps its Text basis in both vertical WMs.
use super::*;

fn qualify(parent: &Captured) {
    let p = &parent.prepared;
    assert_eq!(p.orientation(), t::LineOrientation::Sideways);
    assert_eq!(
        p.analyzed_source().typography(),
        t::AnalysisTypography::HorizontalComposition
    );
    let end = p
        .opportunity_views()
        .find(|row| row.kind() == t::PreparedBreakKind::ParagraphEnd)
        .unwrap()
        .handle();
    let line = p
        .select(t::LineSelectionRequest {
            start: p.boundary_views().next().unwrap().handle(),
            end: end.view().boundary(),
            termination: t::PreparedTermination::Ordinary(end),
            participants: p.participants().collect(),
            alternative: None,
            incoming: None,
            atomics: vec![],
            controls: vec![],
            boundary_context: vec![],
            line_strut: None,
            line_box_struts: vec![],
            context: t::LineContext {
                alignment_insets: t::LineAlignmentInsets::ZERO,
                formatted_line: t::FormattedLineContext::default(),
                base_direction: t::ParagraphBaseDirection::LeftToRight,
                line_box_direction: t::LineBoxDirection::LeftToRight,
                orientation: t::LineOrientation::Sideways,
                start: t::LineStart::Paragraph,
                end: t::LineEnd::Paragraph,
                available_inline: Some(40.0),
                indent: 0.0,
                alignment: t::Alignment::Start,
                tabs: t::TabContext::NoTabs,
                spacing: t::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(t::SpacingAllocations::None)
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
            assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
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
    let text = t::Layout::from_finalized_lines(t::PlacementRequest {
        prepared: p.clone(),
        coverage: t::PlacementCoverage::WholeParagraph,
        entries: vec![t::PlacementEntry::Formatted(t::LinePlacement {
            line,
            anchor: t::LineAnchor {
                inline_origin: 20.0,
                baseline: 100.0,
            },
            orientation: t::LineOrientation::Sideways,
            atomics: vec![],
        })],
    })
    .unwrap();
    assert_eq!(text.source(), &parent.source);
    assert_eq!(text.lines()[0].baseline(), 100.0);
    assert_eq!(
        text.run_views().next().unwrap().origin(),
        t::Point::new(100.0, 20.0)
    );
}

fn vertical_paragraph<S: l::LayoutScalar>() {
    let parent = capture(
        AHEM,
        "AB",
        None,
        t::ParagraphBaseDirection::LeftToRight,
        t::LineOrientation::Sideways,
        10.0,
    );
    // Existing public Text preparation, selection, finalization and placement
    // succeed before the public Layout operation consumes this exact owner.
    qualify(&parent);
    for (mode, baselines, lefts) in [
        (l::WritingMode::VerticalRl, [32.0, 22.0], [30.0, 20.0]),
        (l::WritingMode::VerticalLr, [2.0, 12.0], [0.0, 10.0]),
    ] {
        let mut input = root(mode, l::Direction::Ltr, l::InlineBaselineContext::Physical);
        input.size = l::Size::new(px(40.0), px(15.0));
        let mut host = Host::<S>::new(input);
        let origin = parent.prepared.origin_views().next().unwrap().handle();
        assert_eq!(
            host.push(
                Some(0),
                l::LayoutInputOf::inline_text(l::TextParticipation::new(origin.clone()))
            ),
            1
        );
        let binding = l::TextParagraphBinding {
            prepared: parent.prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![l::TextOriginBinding { origin, node: 1 }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        };
        let batch = run(&host, vec![binding], &[], available(40.0, 15.0)).expect(
            "genuine clockwise Text must retain its orientation in VerticalRl and VerticalLr",
        );
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(node(entries, 0).size, l::Size::new(n(40.0), n(15.0)));
            unknown(node(entries, 0).named_baselines);
        }
        let text = placed(&batch, 0);
        assert_eq!(text.orientation(), t::LineOrientation::Sideways);
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
                t::Rect::new(lefts[0], 0.0, 10.0, 15.0),
                t::Rect::new(lefts[1], 0.0, 10.0, 15.0)
            ]
        );
        let glyphs = text
            .run_views()
            .flat_map(|run| {
                assert_eq!(run.glyph_basis(), t::GlyphBasis::Clockwise);
                run.glyphs()
                    .map(|glyph| (glyph.id(), run.origin().x, run.origin().y + glyph.pen()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(glyphs, [(35, baselines[0], 0.0), (36, baselines[1], 0.0)]);
        let mut lines = std::collections::BTreeMap::new();
        for row in batch.unrounded_inline_fragments() {
            if matches!(row.owner(), l::InlineFragmentOwner::Node(1)) {
                let fragment = row.fragment();
                let range = if fragment.line_index() == 0 {
                    t::Range::new(0, 1)
                } else {
                    t::Range::new(1, 2)
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
                t::LineOrientation::Sideways
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
