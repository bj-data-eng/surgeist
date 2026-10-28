//! The real native Layout result supplies the direct clockwise paint adapter.
use super::*;

#[test]
fn native_sideways_rl_batch_retains_its_actual_paint_and_interaction_basis() {
    const AHEM: &[u8] =
        include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    let source = text::Source::identified("AB", text::Id::from_u64(1520001), 3);
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Preserve,
        word_break: text::WordBreak::BreakAll,
        ..text::Style::default()
    };
    let ancestor_wrap = style.wrap;
    let capture = std::sync::Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: source.clone(),
                local: text::Range::new(0, 2),
                aggregate: text::Range::new(0, 2),
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            computed_line_boundaries: vec![],
            scopes: vec![text::SourceProcessingScope::Block {
                topology: text::SourceProcessingTopology::Unknown,
                used_direction: text::ParagraphBaseDirection::LeftToRight,
                trim: text::WhiteSpaceTrim::default(),
                base_style: style,
            }],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                text::SourceProcessingEvent::EnterScope(0),
                text::SourceProcessingEvent::EnterParagraph(0),
                text::SourceProcessingEvent::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                text::SourceProcessingEvent::ExitParagraph(0),
                text::SourceProcessingEvent::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        capture.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let wrapping = analyzed
        .wrapping_relations()
        .filter(|row| row.ancestor_wrap().is_none())
        .map(|row| text::WrappingPolicy {
            relation: row.handle(),
            ancestor_wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(wrapping).unwrap();
    let prepared = system
        .prepare(
            &analyzed,
            text::PreparationOptions::default().with_orientation(text::LineOrientation::Sideways),
        )
        .unwrap();
    assert_eq!(prepared.captured_paragraph(), capture.captured_paragraph(0));
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                writing_mode: layout::WritingMode::SidewaysRl,
                direction: layout::Direction::Ltr,
                text_align: layout::TextAlign::Start,
                inline_baseline_context: layout::InlineBaselineContext::Physical,
                size: layout::Size::new(
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(40.0).unwrap()),
                    layout::PreferredSizeOf::value(layout::LengthPercentageOf::px(15.0).unwrap()),
                ),
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let flow = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared: prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding {
                origin: origin.clone(),
                node: 1,
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        layout::WrappingInput::default(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &flow,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(40.0),
            layout::Available::Definite(15.0),
        ))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let entry = &batch.placed_text()[0];
    assert_eq!(entry.container(), 0);
    assert_eq!(entry.nodes(), [1]);
    let placed = entry.layout();
    assert_eq!(placed.orientation(), text::LineOrientation::Sideways);
    assert_eq!(placed.source(), &source);
    assert_eq!(placed.source().id(), Some(text::Id::from_u64(1520001)));
    assert_eq!(placed.source().revision(), 3);
    let lines = placed.lines();
    assert_eq!(
        lines.iter().map(|line| line.range()).collect::<Vec<_>>(),
        [text::Range::new(0, 1), text::Range::new(1, 2)]
    );
    let runs = placed
        .run_views()
        .map(|run| {
            assert_eq!(run.orientation(), text::LineOrientation::Sideways);
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
                text::Point::new(32.0, 0.0),
                text::Range::new(0, 1),
                vec![(35, 0.0, 10.0)]
            ),
            (
                text::Point::new(22.0, 0.0),
                text::Range::new(1, 2),
                vec![(36, 0.0, 10.0)]
            ),
        ]
    );
    let first = placed
        .source_range_fragments(&origin, text::Range::new(0, 1))
        .unwrap();
    assert_eq!(
        first
            .units()
            .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        [text::Rect::new(30.0, 0.0, 10.0, 10.0)]
    );
    let stats = system.stats();
    let limits = |runs| {
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            runs,
            32,
            8,
        )
    };
    let refusal =
        render::prepare_text_layout(placed, render::Transform::identity(), limits(1)).unwrap_err();
    assert!(matches!(
        refusal.text_layout_diagnostic(),
        Some(render::TextLayoutError::Limit {
            resource: render::TextLayoutResource::Runs,
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
        render::Transform::identity(),
        render::Transform::translation(7.0, 9.0).unwrap(),
    ] {
        let paint = render::prepare_text_layout(placed, placement, limits(8)).unwrap();
        assert!(std::ptr::eq(paint.layout(), placed));
        assert_eq!(paint.scene().len(), 2);
        assert_eq!(paint.layout().lines(), lines);
        assert_eq!(paint.layout().source(), &source);
        assert_eq!(
            paint
                .layout()
                .cursor(text::Cursor::new(0, text::Affinity::After))
                .unwrap()
                .rect(),
            text::Rect::new(30.0, 0.0, 10.0, 1.0)
        );
    }
    assert_eq!(stats.layout_misses(), 0);
}
