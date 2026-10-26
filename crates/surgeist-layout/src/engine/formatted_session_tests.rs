//! Real known-tree Last consumers, independent of content/size heuristics.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use crate::*;
use surgeist_text as text;

fn paragraph(node: u32, value: &str, terminal: text::ParagraphTerminalEdge) -> PreparedParagraph {
    let style = text::Style {
        hanging_punctuation: text::HangingPunctuation::new(false, None, true),
        ..ahem_style(10.0, 10.0)
    };
    PreparedParagraph::at_boundary(
        vec![ParagraphPart::text(
            node,
            text::Source::identified(value, text::Id::from_u64(u64::from(node)), 71),
            style.clone(),
        )],
        style,
        text::ParagraphBaseDirection::LeftToRight,
        terminal,
    )
}

fn root<S: LayoutScalar>(width: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    }
}

fn compute<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    paragraphs: Vec<TextParagraphBinding<u32>>,
    width: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let capture = LayoutFlowInput::try_new(
        tree,
        0,
        paragraphs,
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    compute_layout_with_flow(
        &capture,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(width)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}

fn lines<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    p: &PreparedParagraph,
) -> Vec<text::FinalizedLine> {
    let mut result = Vec::new();
    for entry in batch.final_inline_fragments() {
        let line = entry.fragment().finalized_fragment().finalized_line();
        if line.prepared_text() == p.prepared && !result.contains(&line) {
            result.push(line);
        }
    }
    result
}

#[test]
fn endpoint_replay_lets_an_earlier_full_candidate_absorb_the_old_last_line() {
    fn lane<S: LayoutScalar>() {
        let p = paragraph(1, "A AA)", text::ParagraphTerminalEdge::BlockEnd);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(0, root(40.0))
            .input(1, p.text_input(1));
        let batch = compute(&tree, vec![p.binding(0)], 40.0);
        let placed = batch.placed_text()[0].layout();
        assert_eq!(placed.lines().len(), 1);
        assert_eq!(placed.lines()[0].range(), text::Range::new(0, 5));
        assert_eq!(placed.source(), p.prepared.analyzed_source().source());
        let actual = lines(&batch, &p);
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].feedback().advance, 50.0);
        assert_eq!(actual[0].measurements().fit().advance(), 40.0);
        assert_eq!(
            actual[0]
                .selected_line()
                .request()
                .context
                .formatted_line
                .last(),
            Some(true)
        );
        assert!(actual[0].measurements().formatted_line_usage().last());
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn intrinsic_and_finite_traversals_keep_separate_actual_owner_frames() {
    fn lane<S: LayoutScalar>() {
        let p = paragraph(1, "A AA)", text::ParagraphTerminalEdge::BlockEnd);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .input(1, p.text_input(1));
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![p.binding(0)],
            crate::WrappingInput {
                scopes: Vec::new(),
                child_order: Vec::new(),
            },
        )
        .unwrap();
        let cases = [
            (AvailableOf::MIN_CONTENT, 20.0),
            (AvailableOf::MAX_CONTENT, 40.0),
            (AvailableOf::definite(S::from_f64(40.0)), 40.0),
        ];
        for order in [[0, 1, 2], [2, 1, 0]] {
            for index in order {
                let (available, expected) = cases[index];
                let batch = compute_layout_with_flow(
                    &capture,
                    0,
                    LayoutRootRequestOf::viewport(Size::new(available, AvailableOf::MAX_CONTENT))
                        .unwrap(),
                )
                .unwrap();
                let width = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 0)
                    .unwrap()
                    .output()
                    .size
                    .width;
                // Ahem's genuine pieces are A (10) and AA) (30). Last
                // excludes the closing10: minimum20, maximum40. The
                // subsequent finite publication is a different real traversal.
                assert_eq!(width.to_f64(), expected);
                let actual = lines(&batch, &p);
                let terminal = actual.last().unwrap();
                assert_eq!(
                    terminal
                        .selected_line()
                        .request()
                        .context
                        .formatted_line
                        .last(),
                    Some(true)
                );
                assert!(terminal.measurements().formatted_line_usage().last());
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn a_later_real_empty_forced_child_disproves_last_for_earlier_text() {
    fn lane<S: LayoutScalar>() {
        let p = paragraph(1, "A)", text::ParagraphTerminalEdge::Unknown);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [3])
            .children(3, [])
            .style(0, root(30.0))
            .input(1, p.text_input(1))
            .style(2, root(30.0))
            .input(
                3,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new().with_metrics(
                        InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0))
                            .unwrap(),
                    ),
                ),
            );
        let batch = compute(&tree, vec![p.binding(0)], 30.0);
        let actual = lines(&batch, &p);
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].feedback().advance, 20.0);
        assert_eq!(actual[0].measurements().fit().advance(), 20.0);
        assert_eq!(
            actual[0]
                .selected_line()
                .request()
                .context
                .formatted_line
                .last(),
            Some(false)
        );
        assert!(actual[0].measurements().formatted_line_usage().last());
        let child = batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == 2)
            .unwrap()
            .output();
        assert!(child.size.height > S::ZERO);
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn a_sized_empty_block_does_not_invent_a_formatted_successor() {
    fn lane<S: LayoutScalar>() {
        let p = paragraph(1, "A)", text::ParagraphTerminalEdge::Unknown);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [])
            .style(0, root(30.0))
            .input(1, p.text_input(1))
            .style(
                2,
                NodeInputOf {
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..root(30.0)
                },
            );
        let batch = compute(&tree, vec![p.binding(0)], 30.0);
        let actual = lines(&batch, &p);
        assert_eq!(actual[0].measurements().fit().advance(), 10.0);
        assert_eq!(
            actual[0]
                .selected_line()
                .request()
                .context
                .formatted_line
                .last(),
            Some(true)
        );
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn a_floats_private_lines_do_not_become_parent_ifc_lines() {
    fn lane<S: LayoutScalar>() {
        let p = paragraph(1, "A)", text::ParagraphTerminalEdge::Unknown);
        let inner = paragraph(3, "B)", text::ParagraphTerminalEdge::BlockEnd);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, [])
            .children(2, [3])
            .children(3, [])
            .style(0, root(60.0))
            .input(1, p.text_input(1))
            .style(
                2,
                NodeInputOf {
                    float: Float::Left,
                    ..root(30.0)
                },
            )
            .input(3, inner.text_input(3));
        let batch = compute(&tree, vec![p.binding(0), inner.binding(2)], 60.0);
        for paragraph in [&p, &inner] {
            let actual = lines(&batch, paragraph);
            assert_eq!(actual.len(), 1);
            assert_eq!(actual[0].feedback().advance, 20.0);
            assert_eq!(actual[0].measurements().fit().advance(), 10.0);
            assert_eq!(
                actual[0]
                    .selected_line()
                    .request()
                    .context
                    .formatted_line
                    .last(),
                Some(true)
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn float_probe_tail_truth_is_merged_only_when_its_packet_is_accepted() {
    let p = paragraph(3, "B)", text::ParagraphTerminalEdge::BlockEnd);
    let tree = PublicLayoutTreeOf::<f64>::new()
        .children(0, [2])
        .children(2, [3])
        .children(3, [])
        .style(0, root(60.0))
        .style(
            2,
            NodeInputOf {
                float: Float::Left,
                ..root(30.0)
            },
        )
        .input(3, p.text_input(3));
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(2)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let mut session = ComputeSession::with_flow(&capture, vec![]);
    let input = ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::BlockFlow,
        ),
        Size::new(AvailableOf::definite(30.0), AvailableOf::MAX_CONTENT),
    )
    .unwrap();
    let packet = session.probe_float_child(2, input).unwrap();
    assert!(
        packet
            .formatted
            .as_ref()
            .is_some_and(|state| !state.demanded.is_empty())
    );
    assert!(session.text.as_ref().unwrap().formatted.is_none());
    assert!(session.text.as_ref().unwrap().pending.is_empty());
    session.accept_float_child(&packet).unwrap();
    assert!(session.text.as_ref().unwrap().formatted_active());
    assert_eq!(session.text.as_ref().unwrap().pending.len(), 1);
}

struct GeometryCachedTree {
    tree: PublicLayoutTreeOf<f64>,
    caches: std::cell::RefCell<std::collections::HashMap<u32, CacheOf<f64>>>,
    hits: std::cell::RefCell<Vec<(u32, RunMode)>>,
}
impl Traverse for GeometryCachedTree {
    type Node = u32;
    type Scalar = f64;
    type Children<'a> = <PublicLayoutTreeOf<f64> as Traverse>::Children<'a>;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}
impl LayoutTree for GeometryCachedTree {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<f64> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, f64> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<f64>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<f64>> {
        let output = self
            .caches
            .borrow()
            .get(&node)?
            .get_with_context(input, context);
        if output.is_some() {
            self.hits.borrow_mut().push((node, input.run_mode()));
        }
        output
    }
}

#[test]
fn warm_geometry_only_numeric_child_cache_cannot_supply_formatted_tail_truth() {
    let p = paragraph(1, "A)", text::ParagraphTerminalEdge::Unknown);
    let tree = GeometryCachedTree {
        tree: PublicLayoutTreeOf::new()
            .children(0, [2])
            .children(2, [1, 4])
            .children(1, [])
            .children(4, [5])
            .children(5, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    ..root(60.0)
                },
            )
            .style(2, root(30.0))
            .input(1, p.text_input(1))
            .style(
                4,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(30.0), PreferredSizeOf::px(10.0)),
                    ..root(30.0)
                },
            )
            .input(
                5,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new()
                        .with_metrics(InlineMetricsOf::from_ascent_descent(8.0, 2.0).unwrap()),
                ),
            ),
        caches: Default::default(),
        hits: Default::default(),
    };
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![p.binding(2)],
        crate::WrappingInput {
            scopes: Vec::new(),
            child_order: Vec::new(),
        },
    )
    .unwrap();
    let request = LayoutRootRequestOf::viewport(Size::new(
        AvailableOf::definite(60.0),
        AvailableOf::MAX_CONTENT,
    ))
    .unwrap();
    let cold = compute_layout_with_flow(&capture, 0, request).unwrap();
    for entry in cold.cache_store_entries() {
        tree.caches
            .borrow_mut()
            .entry(entry.node())
            .or_default()
            .store_with_context(entry.input(), entry.context(), entry.output());
    }
    assert!(
        tree.caches.borrow().contains_key(&4),
        "real flex sizing cached the numeric child"
    );
    tree.hits.borrow_mut().clear();
    let warm = compute_layout_with_flow(&capture, 0, request).unwrap();
    for batch in [&cold, &warm] {
        let actual = lines(batch, &p);
        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].measurements().fit().advance(), 20.0);
        assert_eq!(
            actual[0]
                .selected_line()
                .request()
                .context
                .formatted_line
                .last(),
            Some(false)
        );
        assert!(actual[0].measurements().formatted_line_usage().last());
    }
    assert!(
        tree.hits
            .borrow()
            .iter()
            .any(|(node, mode)| *node == 4 && *mode == RunMode::ComputeSize)
    );
}
