//! The real native Layout result supplies the direct clockwise paint adapter.
use super::*;

#[test]
fn native_sideways_rl_batch_retains_its_actual_paint_and_interaction_basis() {
    const AHEM: &[u8] =
        include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
    let mut system = text::System::default();
    system
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let source = text::source::Source::identified("AB", text::source::Id::from_u64(1520001), 3);
    let style = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        white_space: text::style::WhiteSpace::Preserve,
        word_break: text::style::WordBreak::BreakAll,
        ..text::style::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let capture = std::sync::Arc::new(
        text::capture::process_contextual_source(text::capture::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            source: source.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::capture::SourceProcessingEnvelope::Complete,
            origins: vec![text::paragraph::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::source::Range::new(0, 2),
                aggregate: text::source::Range::new(0, 2),
                kind: text::paragraph::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![text::capture::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::capture::SourceProcessingTopology::Unknown,
                used_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                trim: text::capture::WhiteSpaceTrim::default(),
                base_style: style,
            }],
            paragraphs: vec![text::capture::SourceProcessingParagraph {
                base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                text::capture::SourceProcessingEvent::EnterScope(0),
                text::capture::SourceProcessingEvent::EnterParagraph(0),
                text::capture::SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: text::layout::Affinity::After,
                },
                text::capture::SourceProcessingEvent::ExitParagraph(0),
                text::capture::SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::analyzed::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::analyzed::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wrapping).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::prepared::PreparationOptions::default()
                .with_orientation(text::selection::LineOrientation::SidewaysClockwise),
        )
        .unwrap();
    assert_eq!(
        prepared
            .captured_owner()
            .unwrap()
            .members()
            .collect::<Vec<_>>(),
        [capture.captured_paragraph(0).unwrap()]
    );
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                writing_mode: layout::geometry::WritingMode::SidewaysRl,
                direction: layout::geometry::Direction::Ltr,
                text_align: layout::text_flow::TextAlign::Start,
                inline_baseline_context: layout::baseline::InlineBaselineContext::Physical,
                size: layout::geometry::Size::new(
                    layout::sizing::PreferredSizeOf::value(
                        layout::length::LengthPercentageOf::px(40.0).unwrap(),
                    ),
                    layout::sizing::PreferredSizeOf::value(
                        layout::length::LengthPercentageOf::px(15.0).unwrap(),
                    ),
                ),
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let flow = layout::text_layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::text_layout::TextParagraphBinding {
            inline_box_fragments: layout::text_layout::InlineBoxFragmentationDemand::NotRequested,
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![layout::text_layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        layout::inline_wrapping::WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        NativeViewport::new(
            layout::geometry::Size::new(Some(40.0), Some(15.0)),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::SidewaysRl,
                layout::geometry::Direction::Ltr,
            ),
        )
        .request(layout::geometry::Size::new(
            layout::constraint::SizingOperation::Normal,
            layout::constraint::SizingOperation::Normal,
        )),
    )
    .unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let entry = &batch.placed_text()[0];
    assert_eq!(entry.container(), 0);
    assert_eq!(entry.nodes(), [1]);
    let placed = entry.layout();
    assert_eq!(
        placed.orientation(),
        text::selection::LineOrientation::SidewaysClockwise
    );
    assert_eq!(placed.source(), &source);
    assert_eq!(
        placed.source().id(),
        Some(text::source::Id::from_u64(1520001))
    );
    assert_eq!(placed.source().revision(), 3);
    let lines = placed.lines();
    assert_eq!(
        lines.iter().map(|line| line.range()).collect::<Vec<_>>(),
        [
            text::source::Range::new(0, 1),
            text::source::Range::new(1, 2)
        ]
    );
    let runs = placed
        .run_views()
        .map(|run| {
            assert_eq!(
                run.orientation(),
                text::selection::LineOrientation::SidewaysClockwise
            );
            assert_eq!(run.face().bytes(), AHEM);
            assert_eq!(run.face().index(), 0);
            assert_eq!(
                run.face().catalogue_identity(),
                prepared.font_catalogue_identity()
            );
            (
                run.origin(),
                run.range(),
                run.glyphs()
                    .map(|glyph| (glyph.id(), glyph.pen(), glyph.advance()))
                    .collect::<Vec<_>>(),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        runs,
        [
            (
                text::geometry::Point::new(32.0, 0.0),
                text::source::Range::new(0, 1),
                vec![(35, 0.0, 10.0)]
            ),
            (
                text::geometry::Point::new(22.0, 0.0),
                text::source::Range::new(1, 2),
                vec![(36, 0.0, 10.0)]
            ),
        ]
    );
    let first = placed
        .source_range_fragments(&origin, text::source::Range::new(0, 1))
        .unwrap();
    assert_eq!(
        first
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [text::geometry::Rect::new(30.0, 0.0, 10.0, 10.0)]
    );
    let stats = system.stats();
    let limits = |runs| {
        render::text_layout::TextLayoutLimits::new(
            render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            runs,
            32,
            8,
        )
    };
    let refusal = render::text_layout::prepare_text_layout(
        placed,
        render::geometry::Transform::identity(),
        limits(1),
        &mut surgeist_render::accounting::RenderAccountingContext::default(),
    )
    .unwrap_err();
    assert!(matches!(
        refusal.text_layout_diagnostic(),
        Some(render::text_layout::TextLayoutError::Limit {
            resource: render::text_layout::TextLayoutResource::Runs,
            required: 2,
            limit: 1
        })
    ));
    assert_eq!(placed.lines(), lines);
    assert_eq!(placed.source(), &source);
    drop(system);
    // The owning Render tests qualify actual CW outlines and noncommuting
    // transforms. Here the producer is the authentic native Layout batch.
    for placement in [
        render::geometry::Transform::identity(),
        render::geometry::Transform::translation(7.0, 9.0).unwrap(),
    ] {
        let paint = render::text_layout::prepare_text_layout(
            placed,
            placement,
            limits(8),
            &mut surgeist_render::accounting::RenderAccountingContext::default(),
        )
        .unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert_eq!(paint.scene().len(), 2);
        assert_eq!(paint.layout().lines(), lines);
        assert_eq!(paint.layout().source(), &source);
        assert_eq!(
            paint
                .layout()
                .cursor(text::layout::Cursor::new(0, text::layout::Affinity::After))
                .unwrap()
                .rect(),
            text::geometry::Rect::new(30.0, 0.0, 10.0, 1.0)
        );
    }
    assert_eq!(stats.layout_misses(), 0);
}
