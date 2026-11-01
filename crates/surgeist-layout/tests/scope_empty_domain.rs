#![forbid(unsafe_code)]

//! Genuine empty captured paragraphs associate domains without adding text layout.

use std::{convert::Infallible, sync::Arc};
use surgeist_layout as layout;
use surgeist_text as text;

const JAPANESE: &str = "今日は天気です。";

struct Tree {
    inputs: Vec<layout::LayoutInput>,
    non_box: layout::NodeInput,
}

impl layout::Traverse for Tree {
    type Node = usize;
    type Scalar = f32;
    type Children<'a> = std::ops::Range<usize>;
    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 {
            1..self.inputs.len()
        } else {
            0..0
        }
    }
    fn child_count(&self, node: usize) -> usize {
        if node == 0 { self.inputs.len() - 1 } else { 0 }
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

fn prepare() -> Vec<text::PreparedText> {
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        locale: Some("ja".into()),
        word_space_transform: text::WordSpaceTransform::None,
        ..Default::default()
    };
    let mut source = text::Source::new(format!("{JAPANESE}{JAPANESE}"));
    let n = JAPANESE.encode_utf16().count();
    let end = 2 * n;
    source.span(text::Range::new(0, end), style.clone());
    let origin = |start, end| text::ParagraphOrigin {
        text_node: None,
        source: source.clone(),
        local: text::Range::new(start, end),
        aggregate: text::Range::new(start, end),
        kind: text::ParagraphOriginKind::Authored,
    };
    let block = || text::SourceProcessingScope::Block {
        writing_axis: None,
        horizontal_inner_direction: None,
        font_reference: None,
        topology: text::SourceProcessingTopology::Complete,
        trim: Default::default(),
        base_style: style.clone(),
        used_direction: text::ParagraphBaseDirection::LeftToRight,
    };
    use text::SourceProcessingEvent as E;
    let control = |control| E::Control {
        control,
        affinity: text::Affinity::After,
    };
    let occurrence = |origin| E::Origin {
        origin,
        affinity: text::Affinity::After,
    };
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![origin(0, n), origin(n, end)],
            connections: vec![],
            controls: vec![
                text::ParagraphControl {
                    at: 0,
                    kind: text::ParagraphControlKind::InlineBoundaryStart,
                },
                text::ParagraphControl {
                    at: end,
                    kind: text::ParagraphControlKind::InlineBoundaryEnd,
                },
            ],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![
                block(),
                text::SourceProcessingScope::Inline {
                    horizontal_inner_direction: None,
                    alignment_boundary: None,
                    font_reference: None,
                    trim: Default::default(),
                    style: Some(style.clone()),
                    boundaries: Some(text::SourceProcessingScopeBoundaries { start: 0, end: 1 }),
                },
                block(),
            ],
            paragraphs: (0..3)
                .map(|_| text::SourceProcessingParagraph {
                    base_direction: text::ParagraphBaseDirection::LeftToRight,
                    paragraph_strut: text::ParagraphStrut::DefaultFont,
                })
                .collect(),
            // The inner plain Block has its required genuine paragraph, with no
            // original, control or atomic occurrences inside that paragraph.
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::EnterScope(1),
                control(0),
                occurrence(0),
                E::ExitParagraph(0),
                E::EnterScope(2),
                E::EnterParagraph(1),
                E::ExitParagraph(1),
                E::ExitScope(2),
                E::EnterParagraph(2),
                occurrence(1),
                control(1),
                E::ExitScope(1),
                E::ExitParagraph(2),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.paragraphs().count(), 3);
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    processed
        .paragraphs()
        .map(|paragraph| {
            let analyzed = text::AnalyzedSource::from_processed_paragraph(
                paragraph,
                text::AnalysisTypography::HorizontalComposition,
            )
            .unwrap();
            let policies = analyzed
                .wrapping_relations()
                .filter(|row| row.ancestor_wrap().is_none())
                .map(|row| text::WrappingPolicy {
                    relation: row.handle(),
                    ancestor_wrap: text::Wrap::Word,
                })
                .collect();
            let analyzed = analyzed.with_wrapping_policies(policies).unwrap();
            system
                .prepare(&analyzed, text::PreparationOptions::default())
                .unwrap()
        })
        .collect()
}

fn fixture() -> (
    Tree,
    Vec<layout::TextParagraphBinding<usize>>,
    layout::TextParagraphBinding<usize>,
) {
    let all = prepare();
    let tree = Tree {
        inputs: vec![
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineBoundary(
                layout::InlineBoundaryInput::contribution(
                    layout::InlineBoundaryKind::Start,
                    layout::InlineMetrics::try_new(
                        surgeist_layout::InlineBaselineContext::Physical,
                        8.0,
                        10.0,
                    )
                    .unwrap(),
                    layout::InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(
                all[0].origin_views().next().unwrap().handle(),
            )),
            layout::LayoutInput::box_input(layout::NodeInput {
                display: layout::Display::Block,
                ..layout::NodeInput::DEFAULT
            }),
            layout::LayoutInput::InlineText(layout::TextParticipation::new(
                all[2].origin_views().next().unwrap().handle(),
            )),
            layout::LayoutInput::InlineBoundary(
                layout::InlineBoundaryInput::contribution(
                    layout::InlineBoundaryKind::End,
                    layout::InlineMetrics::try_new(
                        surgeist_layout::InlineBaselineContext::Physical,
                        8.0,
                        10.0,
                    )
                    .unwrap(),
                    layout::InlineBoundaryEdgeOf::ZERO,
                )
                .unwrap(),
            ),
        ],
        non_box: layout::NodeInput::non_box(),
    };
    let bindings: Vec<_> = [(0, vec![1, 2], 2, 1), (2, vec![4, 5], 4, 5)]
        .into_iter()
        .map(|(index, nodes, origin_node, control_node)| {
            let prepared = all[index].clone();
            let origin = prepared.origin_views().next().unwrap().handle();
            layout::TextParagraphBinding {
                origins: vec![layout::TextOriginBinding {
                    origin,
                    node: origin_node,
                }],
                controls: prepared
                    .control_views()
                    .map(|view| layout::TextControlBinding {
                        occurrence: view.handle(),
                        node: control_node,
                    })
                    .collect(),
                prepared,
                container: 0,
                nodes,
                atomics: vec![],
                line_strut: None,
            }
        })
        .collect();
    let empty = layout::TextParagraphBinding {
        prepared: all[1].clone(),
        container: 3,
        nodes: vec![],
        origins: vec![],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    (tree, bindings, empty)
}

fn request() -> layout::LayoutRootRequest {
    layout::LayoutRootRequest::viewport(layout::Size::new(
        layout::Available::definite(500.0),
        layout::Available::MAX_CONTENT,
    ))
    .unwrap()
}

#[test]
fn empty_captured_child_paragraph_preserves_ordinary_box_and_parent_text_layout() {
    let (tree, parents, empty) = fixture();
    assert_eq!(empty.prepared.origin_views().count(), 0);
    assert_eq!(empty.prepared.control_views().count(), 0);
    assert_eq!(empty.prepared.atomic_views().count(), 0);
    assert_eq!(empty.prepared.participants().count(), 0);
    assert_eq!(empty.prepared.scopes().count(), 0);
    assert_eq!(empty.prepared.virtual_separators().count(), 0);
    assert!(empty.line_strut.is_none());

    // The real empty block already computes normally without a text run.
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        parents.clone(),
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let baseline = layout::compute_layout_with_flow(&capture, 0, request()).unwrap();
    assert_eq!(baseline.placed_text().len(), 2);
    let box3 = baseline
        .final_entries()
        .iter()
        .find(|entry| entry.node() == 3)
        .unwrap()
        .output();
    assert_eq!(box3.location, layout::Point::new(0.0, 10.0));
    assert_eq!(box3.size, layout::Size::new(500.0, 0.0));
    assert_eq!(
        baseline
            .placed_text()
            .iter()
            .map(|entry| entry.layout().lines().len())
            .sum::<usize>(),
        2
    );

    // This genuine empty captured paragraph supplies domain association only:
    // it must add no strut, line, placed Text result or generated geometry.
    let mut complete = parents;
    complete.insert(1, empty);
    let capture = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        complete,
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap_or_else(|error| {
        panic!(
            "a genuine empty captured child paragraph must associate without a Text run: {error:?}"
        )
    });
    let associated = layout::compute_layout_with_flow(&capture, 0, request()).unwrap();
    assert_eq!(associated.final_entries(), baseline.final_entries());
    assert_eq!(associated.placed_text().len(), 2);
    for (actual, expected) in associated.placed_text().iter().zip(baseline.placed_text()) {
        assert_eq!(actual.container(), expected.container());
        assert_eq!(actual.nodes(), expected.nodes());
        assert_eq!(
            actual.layout().source().text(),
            expected.layout().source().text()
        );
        assert_eq!(actual.layout().lines(), expected.layout().lines());
    }
    let fragments = |batch: &layout::CompletedLayoutBatch<usize>| {
        batch
            .final_inline_fragments()
            .iter()
            .map(|entry| {
                (
                    entry.owner().clone(),
                    entry.fragment().rect(),
                    entry.fragment().baseline(),
                    entry.fragment().line_index(),
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(fragments(&associated), fragments(&baseline));
}

#[test]
fn source_bearing_paragraph_with_empty_nodes_is_not_association_only() {
    let (tree, mut parents, _) = fixture();
    assert_eq!(parents[0].prepared.origin_views().count(), 1);
    assert_eq!(parents[0].origins.len(), 1);
    assert!(parents[0].prepared.participants().next().is_some());
    // Clear supplied facts while preserving the genuine prepared inventories.
    // Caller omissions cannot disguise a source-bearing preparation as empty.
    parents[0].nodes.clear();
    parents[0].origins.clear();
    parents[0].controls.clear();
    parents[0].atomics.clear();
    let Err(error) = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        parents,
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("a source-bearing paragraph with no run nodes must refuse")
    };
    assert!(
        matches!(
            error.kind(),
            layout::LayoutErrorKind::InvalidInput(layout::LayoutInvalidInput::TextBinding(
                layout::TextBindingError::EmptyRun
            ))
        ),
        "{error:?}"
    );
}

#[test]
fn discarded_source_space_keeps_its_origin_and_is_not_association_only() {
    let style = text::Style {
        font: text::Font::new().family("Ahem"),
        size: 10.0,
        line_height: text::LineHeight::Absolute(10.0),
        white_space: text::WhiteSpace::Discard,
        ..Default::default()
    };
    let mut source = text::Source::new(" ");
    let whole = text::Range::new(0, 1);
    source.span(whole, style.clone());
    use text::SourceProcessingEvent as E;
    let processed = Arc::new(
        text::process_contextual_source(text::ContextualSourceInput {
            joint_flows: Vec::new(),
            scope_alignment_limits: None,
            computed_line_boundaries: vec![],
            source: source.clone(),
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            envelope: text::SourceProcessingEnvelope::Complete,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source,
                local: whole,
                aggregate: whole,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            boundary_policies: vec![],
            replacements: vec![],
            scopes: vec![text::SourceProcessingScope::Block {
                writing_axis: None,
                horizontal_inner_direction: None,
                font_reference: None,
                topology: text::SourceProcessingTopology::Complete,
                trim: Default::default(),
                base_style: style,
                used_direction: text::ParagraphBaseDirection::LeftToRight,
            }],
            paragraphs: vec![text::SourceProcessingParagraph {
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                paragraph_strut: text::ParagraphStrut::DefaultFont,
            }],
            events: vec![
                E::EnterScope(0),
                E::EnterParagraph(0),
                E::Origin {
                    origin: 0,
                    affinity: text::Affinity::After,
                },
                E::ExitParagraph(0),
                E::ExitScope(0),
            ],
        })
        .unwrap(),
    );
    assert_eq!(processed.processed_text().len(), 0);
    let analyzed = text::AnalyzedSource::from_processed_paragraph(
        processed.paragraphs().next().unwrap(),
        text::AnalysisTypography::HorizontalComposition,
    )
    .unwrap();
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system
        .register_fonts(
            include_bytes!("../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf").to_vec(),
        )
        .unwrap();
    let prepared = system
        .prepare(&analyzed, text::PreparationOptions::default())
        .unwrap();
    // Output emptiness does not erase the genuine original occurrence.
    assert_eq!(prepared.origin_views().count(), 1);
    let binding = layout::TextParagraphBinding {
        prepared,
        container: 0,
        nodes: vec![],
        origins: vec![],
        atomics: vec![],
        controls: vec![],
        line_strut: None,
    };
    let tree = Tree {
        inputs: vec![layout::LayoutInput::box_input(layout::NodeInput {
            display: layout::Display::Block,
            ..layout::NodeInput::DEFAULT
        })],
        non_box: layout::NodeInput::non_box(),
    };
    let Err(error) = layout::LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        surgeist_layout::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
        Vec::new(),
    ) else {
        panic!("discarding a genuine source space cannot make its paragraph association-only")
    };
    assert!(
        matches!(
            error.kind(),
            layout::LayoutErrorKind::InvalidInput(layout::LayoutInvalidInput::TextBinding(
                layout::TextBindingError::EmptyRun
            ))
        ),
        "{error:?}"
    );
}
