#![cfg(feature = "render-text")]
#![forbid(unsafe_code)]

//! Real immutable normalized capture, Layout selection, and Render consumption.
//! This uses explicit native inputs; it does not authenticate a live Style tree.

use std::convert::Infallible;

use surgeist::{layout, render, text};

struct Tree {
    inputs: [layout::LayoutInput; 2],
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::iter::Copied<std::slice::Iter<'a, usize>>;

    fn children(&self, node: usize) -> Self::Children<'_> {
        let children: &[usize] = if node == 0 { &[1] } else { &[] };
        children.iter().copied()
    }

    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
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
fn actual_layout_lines_supply_paint_and_interaction_without_rewrapping() {
    assert_retained_layout_composition(false);
}

#[cfg(feature = "text-accessibility")]
#[test]
fn expanded_spaces_supply_same_paint_cursor_and_geometric_accessibility() {
    assert_retained_layout_composition(true);
}

fn assert_retained_layout_composition(expanded: bool) {
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../crates/surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    let source = text::Source::new(if expanded { "A X X" } else { "AX AX" });
    let analyzed = text::AnalyzedSource::from_paragraph(text::ParagraphInput {
        source: source.clone(),
        base_style: text::Style {
            font: text::Font::new().family("Ahem"),
            size: 20.0,
            line_height: text::LineHeight::Absolute(20.0),
            white_space: text::WhiteSpace::Collapse,
            ..text::Style::default()
        },
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        origins: vec![text::ParagraphOrigin {
            source,
            local: text::Range::new(0, 5),
            aggregate: text::Range::new(0, 5),
            kind: text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        boundary_policies: vec![],
        replacements: vec![],
        anchor_order: vec![],
    })
    .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    let origin = prepared.origin_views().next().unwrap().handle();
    let tree = Tree {
        inputs: [
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                text_align: if expanded {
                    layout::TextAlign::Justify
                } else {
                    layout::TextAlign::Start
                },
                text_justification: if expanded {
                    layout::TextJustification::Spaces
                } else {
                    layout::TextJustification::None
                },
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(origin.clone())),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let capture = layout::TextLayoutInput::try_new(
        &tree,
        0,
        vec![layout::TextParagraphBinding {
            prepared,
            container: 0,
            nodes: vec![1],
            origins: vec![layout::TextOriginBinding { origin, node: 1 }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }],
    )
    .unwrap();
    let request = layout::LayoutRootRequest::viewport(layout::Size::new(
        layout::Available::Definite(if expanded { 65.0 } else { 45.0 }),
        layout::Available::MaxContent,
    ))
    .unwrap();
    let batch = layout::compute_layout_with_text(&capture, 0, request).unwrap();
    assert_eq!(batch.placed_text().len(), 1);
    let committed = &batch.placed_text()[0];
    assert_eq!(committed.container(), 0);
    assert_eq!(committed.nodes(), &[1]);
    let placed = committed.layout();
    let lines = placed.lines();
    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines[0].range(),
        text::Range::new(0, if expanded { 4 } else { 3 })
    );
    assert_eq!(
        lines[1].range(),
        text::Range::new(if expanded { 4 } else { 3 }, 5)
    );
    // Ahem's one-em cells, 0.8em ascent, and explicit 20px line height
    // independently determine the glyph advances and baselines. Each line
    // retains the separate container allocation supplied by Layout. The expanded
    // first line has two 20px letters, one 20px internal space plus 5px extra,
    // and a discarded trailing space. The final line is a single 20px X.
    assert_eq!(lines[0].baseline(), 16.0);
    assert_eq!(lines[1].baseline(), 36.0);
    assert_eq!(
        lines[0].bounds().size.width,
        if expanded { 65.0 } else { 45.0 }
    );
    assert_eq!(
        lines[1].bounds().size.width,
        if expanded { 65.0 } else { 45.0 }
    );
    assert_eq!(
        placed
            .glyph_runs()
            .iter()
            .map(|run| run.advance())
            .collect::<Vec<_>>(),
        if expanded {
            vec![65.0, 20.0]
        } else {
            vec![40.0, 40.0]
        }
    );
    assert_eq!(
        placed
            .cursor(text::Cursor::new(
                if expanded { 2 } else { 4 },
                text::Affinity::After
            ))
            .unwrap()
            .rect()
            .origin,
        if expanded {
            text::Point::new(45.0, 0.0)
        } else {
            text::Point::new(20.0, 20.0)
        }
    );
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
    assert_eq!(paint.scene().len(), 2);
    // Admission consumes the exact retained result; it cannot change its lines,
    // source associations, physical origins, or already finalized glyph rows.
    assert_eq!(paint.layout().lines(), lines);
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
        let parent_node = &update.nodes.iter().find(|(id, _)| *id == parent).unwrap().1;
        let parent_bounds = parent_node.bounds().unwrap();
        assert_eq!(
            (
                parent_bounds.x0,
                parent_bounds.y0,
                parent_bounds.x1,
                parent_bounds.y1
            ),
            (100.0, 200.0, if expanded { 165.0 } else { 145.0 }, 240.0)
        );
        let cursor = text::Cursor::new(if expanded { 2 } else { 4 }, text::Affinity::After);
        let position = paint
            .layout()
            .access_position(&accessibility, cursor)
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
            if expanded {
                (145.0, 200.0, 165.0, 220.0)
            } else {
                (120.0, 220.0, 140.0, 240.0)
            }
        );
        assert_eq!(position.character_index, 0);
        assert_eq!(node.value(), Some("X"));
        assert_eq!(node.font_size(), Some(20.0));
        let other = layout::compute_layout_with_text(&capture, 0, request).unwrap();
        assert_eq!(
            other.placed_text()[0]
                .layout()
                .access_position(&accessibility, cursor)
                .unwrap_err()
                .detail(),
            Some(&text::ErrorDetail::AccessibilityBindingMismatch)
        );
    }
}
