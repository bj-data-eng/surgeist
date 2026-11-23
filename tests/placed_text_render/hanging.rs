//! A real Layout tail replay remains the one source of painted Text geometry.
use super::*;

#[test]
fn terminal_hanging_replay_reaches_paint_and_source_geometry_without_rewrapping() {
    const AHEM: &[u8] =
        include_bytes!("../../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");
    let mut system = text::System::default();
    system
        .register_fonts(AHEM.to_vec(), surgeist_test::fixtures::FONT_LIMITS)
        .unwrap();
    let source = text::source::Source::new("A AA)");
    let style = text::style::Style {
        font: text::style::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::style::LineHeight::Absolute(10.0),
        hanging_punctuation: text::style::HangingPunctuation::new(false, None, true),
        ..text::style::Style::default()
    };
    let analyzed = text::analyzed::AnalyzedSource::from_paragraph(
        text::paragraph::ParagraphInput {
            source: source.clone(),
            base_style: style.clone(),
            segment_breaks: text::processing::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::paragraph::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::paragraph::ParagraphStrut::DefaultFont,
            terminal_edge: text::paragraph::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            origins: vec![text::paragraph::ParagraphOrigin {
                source: source.clone(),
                local: text::source::Range::new(0, 5),
                aggregate: text::source::Range::new(0, 5),
                kind: text::paragraph::ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::analyzed::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::analyzed::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: style.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::prepared::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::node::LayoutInput::box_input(layout::node::NodeInput {
                display: layout::box_model::Display::Block,
                ..layout::node::NodeInput::DEFAULT
            }),
            layout::node::LayoutInput::InlineText(layout::text_layout::TextParticipation::new(
                origin.clone(),
            )),
        ],
        non_box: layout::node::NodeInput::non_box(),
    };
    let capture = layout::text_layout::LayoutFlowInput::try_new(
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
        surgeist_layout::inline_wrapping::WrappingInput {
            native_inline_boxes: Vec::new(),
            gaps: Vec::new(),
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        NativeViewport::new(
            layout::geometry::Size::new(Some(40.0), None),
            layout::geometry::FlowAxes::new(
                layout::geometry::WritingMode::HorizontalTb,
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
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source(), &source);
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(placed.lines()[0].range(), text::source::Range::new(0, 5));
    assert_eq!(placed.lines()[0].bounds().size.width, 40.0);
    let finalized = batch.final_inline_fragments()[0]
        .fragment()
        .finalized_fragment()
        .finalized_line();
    assert_eq!(finalized.prepared_text(), prepared);
    assert_eq!(finalized.feedback().advance, 50.0);
    assert_eq!(finalized.measurements().fit().advance(), 40.0);
    assert!(finalized.measurements().formatted_line_usage().last());
    let glyphs = placed
        .run_views()
        .flat_map(|run| {
            assert_eq!(run.face().bytes(), AHEM);
            let x = run.origin().x;
            run.glyphs()
                .filter(|glyph| glyph.id() != 3)
                .map(|glyph| (glyph.id(), x + glyph.pen(), glyph.advance()))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        glyphs,
        [
            (35, 0.0, 10.0),
            (35, 20.0, 10.0),
            (35, 30.0, 10.0),
            (11, 40.0, 10.0)
        ]
    );
    assert_eq!(
        placed
            .cursor(text::layout::Cursor::new(5, text::layout::Affinity::After))
            .unwrap()
            .rect()
            .origin
            .x,
        50.0
    );
    let closing = placed
        .source_range_fragments(&origin, text::source::Range::new(4, 5))
        .unwrap();
    let pieces = closing
        .units()
        .flat_map(|unit| unit.pieces().map(|piece| piece.rect()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(pieces, [text::geometry::Rect::new(40.0, 0.0, 10.0, 10.0)]);
    drop(system);
    let paint = render::text_layout::prepare_text_layout(
        placed,
        render::geometry::Transform::identity(),
        render::text_layout::TextLayoutLimits::new(
            render::text::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
        &mut surgeist_render::accounting::RenderAccountingContext::default(),
    )
    .unwrap();
    assert!(std::ptr::eq(paint.layout(), placed));
    assert_eq!(paint.scene().len(), 1);
    assert_eq!(paint.layout().lines(), placed.lines());
}
