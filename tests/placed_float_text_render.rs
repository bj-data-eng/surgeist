#![cfg(feature = "render-text")]
#![forbid(unsafe_code)]

//! One retained paragraph across an actual float supplies paint and interaction.

use std::convert::Infallible;

use surgeist::{layout, render, text};

struct Tree {
    inputs: [layout::LayoutInput; 4],
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1, 3, 2] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        if node == 0 { 3 } else { 0 }
    }

    fn child(&self, node: usize, index: usize) -> usize {
        self.children(node).nth(index).unwrap()
    }
}

impl layout::LayoutTree for Tree {
    type MeasureError = Infallible;

    fn node_input(&self, node: usize) -> &layout::NodeInput {
        self.inputs[node]
            .as_view()
            .as_box()
            .unwrap_or(&self.non_box)
    }

    fn layout_input(&self, node: usize) -> layout::LayoutInputView<'_> {
        self.inputs[node].as_view()
    }
}

#[test]
fn interleaved_float_keeps_one_paragraph_for_paint_cursor_and_accessibility() {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        ..text::Style::default()
    };
    let analyzed = text::AnalyzedSource::from_paragraph(
        text::ParagraphInput {
            terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
            tab_reference: None,
            source: text::Source::new("AX"),
            base_style: style.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            origins: ["A", "X"]
                .into_iter()
                .enumerate()
                .map(|(index, value)| text::ParagraphOrigin {
                    text_node: None,
                    source: text::Source::new(value),
                    local: text::Range::new(0, 1),
                    aggregate: text::Range::new(index, index + 1),
                    kind: text::ParagraphOriginKind::Authored,
                })
                .collect(),
            connections: vec![text::ParagraphConnection {
                at: 1,
                kind: text::ParagraphConnectionKind::Continuous,
            }],
            controls: vec![],
            boundary_policies: vec![text::ParagraphBoundaryPolicy {
                definition: text::ParagraphBoundaryDefinition::Connection(0),
                preceding_wrap: style.wrap,
                following_wrap: style.wrap,
                ancestor_wrap: style.wrap,
            }],
            replacements: vec![],
            anchor_order: vec![],
        },
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    // Both text origins are direct children of Tree's one block. The float is
    // outside their influencing pair; retain the existing connection policy.
    let policies = analyzed
        .wrapping_relations()
        .filter(|relation| relation.ancestor_wrap().is_none())
        .map(|relation| text::WrappingPolicy {
            relation: relation.handle(),
            ancestor_wrap: style.wrap,
        })
        .collect();
    let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    assert_eq!(prepared.atomic_views().count(), 0);
    assert_eq!(prepared.control_views().count(), 0);
    let origins: Vec<_> = prepared.origin_views().map(|view| view.handle()).collect();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origins[0].clone())),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origins[1].clone())),
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                float: layout::Float::Left,
                size: layout::Size::new(
                    layout::PreferredSize::value(layout::LengthPercentageOf::px(15.0).unwrap()),
                    layout::PreferredSize::value(layout::LengthPercentageOf::px(10.0).unwrap()),
                ),
                ..layout::NodeInput::DEFAULT
            }),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            inline_box_fragments: layout::InlineBoxFragmentationDemand::NotRequested,
            prepared,
            container: 0,
            nodes: vec![1, 3, 2],
            origins: origins
                .into_iter()
                .enumerate()
                .map(|(index, origin)| layout::TextOriginBinding {
                    origin,
                    node: index + 1,
                })
                .collect(),
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let batch = layout::compute_layout_with_flow(
        &capture,
        0,
        layout::LayoutRootRequest::viewport(layout::Size::new(
            layout::Available::Definite(35.0),
            layout::Available::MaxContent,
        ))
        .unwrap(),
    )
    .unwrap();
    let float = batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(float.location, layout::Point::ZERO);
    assert_eq!(float.size, layout::Size::new(15.0, 10.0));
    assert_eq!(batch.placed_text().len(), 1);
    let placed = batch.placed_text()[0].layout();
    assert_eq!(placed.source().text().code_units(), &[65, 88]);
    assert_eq!(placed.lines().len(), 1);
    assert_eq!(placed.lines()[0].range(), text::Range::new(0, 2));
    // Independent Ahem10 cells and the real left float exactly fill 35px.
    // The float is not text; the one line occupies the remaining 20px band.
    assert_eq!(placed.lines()[0].baseline(), 8.0);
    assert_eq!(
        placed.lines()[0].bounds(),
        text::Rect::new(15.0, 0.0, 20.0, 10.0)
    );
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.advance())
            .sum::<f64>(),
        20.0
    );
    for (offset, x) in [(0, 15.0), (1, 25.0)] {
        assert_eq!(
            placed
                .cursor(text::Cursor::new(offset, text::Affinity::After))
                .unwrap()
                .rect()
                .origin,
            text::Point::new(x, 0.0)
        );
    }
    let stats = system.stats();
    drop(system);
    let paint = render::prepare_text_layout(
        placed,
        render::Transform::identity(),
        render::TextLayoutLimits::new(
            render::GlyphRunLimits::new(32, 1024, 8 << 20, 8 << 20),
            8,
            32,
            8,
        ),
    )
    .unwrap();
    assert!(std::ptr::eq(paint.layout(), placed));
    assert_eq!(paint.layout().lines(), placed.lines());
    assert_eq!(stats.layout_misses(), 0);
    #[cfg(feature = "text-accessibility")]
    {
        let mut accessibility = text::Accessibility::new(10);
        let parent = 1000_u64.into();
        let update = paint
            .layout()
            .accessibility_update(
                &mut accessibility,
                parent,
                parent,
                text::Point::new(100.0, 200.0),
            )
            .unwrap();
        let position = paint
            .layout()
            .access_position(&accessibility, text::Cursor::new(1, text::Affinity::After))
            .unwrap()
            .unwrap();
        let node = &update
            .nodes
            .iter()
            .find(|(id, _)| *id == position.node)
            .unwrap()
            .1;
        let bounds = node.bounds().unwrap();
        assert_eq!(
            (bounds.x0, bounds.y0, bounds.x1, bounds.y1),
            (125.0, 200.0, 135.0, 210.0)
        );
        assert_eq!(node.value(), Some("X"));
        assert_eq!(node.font_size(), Some(10.0));
    }
}
