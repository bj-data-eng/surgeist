use super::fixtures::{
    FlowRootLeafTree, PublicFlowTree, RootSessionTree, RootTestScrollGeometryFacts,
    assert_fri06_c08_float_line_final_height, assert_fri06_c08_mixed_inline_atomic_x,
    assert_fri06_c08_r1_mixed_unit_traversal, assert_positive_physical_range,
    assert_public_scroll_geometry_error_without_batch, computed_overflow,
    fri05_c03_block_root_state, fri05_c04_assert_initial_local_auto_state,
    fri05_c04_local_auto_state, fri05_c04_nested_flex_auto_tree, fri06_atomic_participation,
    fri06_c02_final_node, fri06_c03_atomic_participation, fri06_c03_atomic_style,
    fri06_c03_mixed_batch_with_root, fri06_c04_bfc_batch, fri06_c04_front_door_batch,
    fri06_c04_line_batch, fri06_c04_line_box, fri06_c04_logical_origin,
    fri06_c12_t08_forced_break_fallback_batch, fri06_mr02_geometry_error_largest_finite,
    prepared_atomic_style, prepared_front_door_batch, prepared_mixed_batch, prepared_source_batch,
    prepared_tree_batch, public_flow_output, public_layout_tree, root_test_scroll_geometry,
    root_writing_mode_directions, scalar, single_final_output,
};
use super::*;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use surgeist_text::{Id, ParagraphReplacement, ParagraphReplacementDefinition, Source, Text};

fn assert_fri08_c07_t02_scroll_source_root_paths<S: LayoutScalar>() {
    let scalar = S::from_f64;
    let size = Size::new(scalar(90.0), scalar(60.0));
    let flow_axes = FlowAxes::new(WritingMode::VerticalLr, Direction::Rtl);
    let scroll_margin =
        ScrollMarginOf::try_new(scalar(1.0), scalar(-2.0), scalar(3.0), scalar(-4.0)).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::Center, ScrollSnapAlignValue::End);
    let style = NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
        overflow_clip_margin: OverflowClipMarginOf::try_new(
            OverflowClipBox::BorderBox,
            scalar(2.0),
        )
        .unwrap(),
        scrollbar_width: ScrollbarWidthOf::try_new(scalar(5.0)).unwrap(),
        size: size.map(PreferredSizeOf::px),
        border: Edges::all(LengthOf::px(scalar(2.0))),
        padding: Edges::all(LengthOf::px(scalar(3.0))),
        scroll_padding: ScrollPaddingOf::new(
            ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(1.0)).unwrap()),
            ScrollPaddingValueOf::AUTO,
            ScrollPaddingValueOf::value(LengthPercentageOf::px(scalar(4.0)).unwrap()),
            ScrollPaddingValueOf::AUTO,
        ),
        scroll_margin,
        scroll_snap_type: ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        },
        scroll_snap_align: snap_align,
        scroll_snap_stop: ScrollSnapStop::Always,
        ..NodeInputOf::default()
    };

    let mut reconstructed_tree = OracleTreeOf::<S>::new()
        .children(7, [])
        .style(7, style.clone())
        .measure(
            7,
            ComputeOutputOf::from_sizes(size, Size::new(scalar(120.0), scalar(95.0))),
        );
    compute_root(
        &mut reconstructed_tree.borrow_compute(),
        7,
        size.map(AvailableOf::definite),
    )
    .unwrap();
    let reconstructed = reconstructed_tree
        .output(7)
        .and_then(|output| output.scroll_geometry)
        .unwrap();
    assert_eq!(reconstructed.flow_axes(), flow_axes);
    assert_eq!(reconstructed.border_box().size(), size);
    assert_eq!(
        reconstructed.target().border_box(),
        reconstructed.border_box()
    );
    assert_eq!(reconstructed.target().scroll_margin(), scroll_margin);
    assert_eq!(reconstructed.target().flow_axes(), flow_axes);
    assert_eq!(reconstructed.target().snap_align(), snap_align);
    assert_eq!(reconstructed.target().snap_stop(), ScrollSnapStop::Always);
    assert!(reconstructed.overflow_clip().x().is_some());
    assert!(reconstructed.overflow_clip().y().is_some());
    assert_eq!(reconstructed.resolved_scroll_padding().top, scalar(1.0));
    assert_eq!(reconstructed.resolved_scroll_padding().bottom, scalar(4.0));
    assert_eq!(
        reconstructed.scroll_snap_type(),
        ScrollSnapType::Enabled {
            axis: ScrollSnapAxis::Block,
            strictness: ScrollSnapStrictness::Mandatory,
        }
    );

    let existing = root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes,
        overflow: style.overflow,
        item_is_replaced: style.item_is_replaced,
        border_box_size: size,
        padding: Edges::all(scalar(3.0)),
        border: Edges::all(scalar(2.0)),
        scrollbar_width: scalar(5.0),
        scrollable_overflow: ScrollRectOf::try_new(
            Point::new(scalar(-7.0), scalar(-3.0)),
            Size::new(scalar(130.0), scalar(100.0)),
        )
        .unwrap(),
    });
    let mut child_output =
        ComputeOutputOf::from_sizes(size, Size::new(scalar(120.0), scalar(95.0)));
    child_output.scroll_geometry = Some(existing);
    let mut existing_tree = OracleTreeOf::<S>::new()
        .children(7, [])
        .style(7, style)
        .measure(7, child_output);
    compute_root(
        &mut existing_tree.borrow_compute(),
        7,
        size.map(AvailableOf::definite),
    )
    .unwrap();
    assert_eq!(
        existing_tree
            .output(7)
            .and_then(|output| output.scroll_geometry),
        Some(existing),
        "matching existing geometry remains byte-for-byte canonical"
    );
}

#[test]
fn fri08_c07_t02_scroll_source_root_preserves_existing_reconstruction_and_metadata() {
    assert_fri08_c07_t02_scroll_source_root_paths::<f32>();
    assert_fri08_c07_t02_scroll_source_root_paths::<f64>();
}

fn fri06_c03_mixed_batch<S: LayoutScalar>(
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    available_inline: AvailableOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    fri06_c03_mixed_batch_with_root(
        children,
        available_inline,
        NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        },
    )
}

fn fri06_c03_logical_block_start<S: LayoutScalar>(
    flow_axes: FlowAxes,
    output: NodeOutputOf<S>,
    container_size: Size<S>,
) -> S {
    flow_axes
        .logical_point(output.location, output.size, container_size)
        .block
}

fn fri06_c03_fragment<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> &InlineFragmentOutputOf<S> {
    batch
        .final_inline_fragments()
        .iter()
        .find(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(node))
        .expect("the requested original occurrence publishes a fragment")
        .fragment()
}

fn prepared_nested_atomic_baseline_batch<S: LayoutScalar>(
    parent_flow: FlowAxes,
    child_flow: FlowAxes,
    overflow: Overflow,
    item_is_replaced: bool,
) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S, core::convert::Infallible> {
    let root = NodeInputOf {
        display: Display::Block,
        writing_mode: parent_flow.writing_mode(),
        direction: parent_flow.direction(),
        ..NodeInputOf::default()
    };
    let atomic_size =
        parent_flow.physical_size(LogicalSizeOf::new(S::from_f64(10.0), S::from_f64(20.0)));
    let atomic = NodeInputOf {
        display: Display::InlineBlock,
        writing_mode: child_flow.writing_mode(),
        direction: child_flow.direction(),
        size: atomic_size.map(PreferredSizeOf::px),
        margin: parent_flow.physical_edges(
            LogicalEdgesOf::new(S::ZERO, S::ZERO, S::from_f64(1.0), S::from_f64(2.0))
                .map(LengthAutoOf::px),
        ),
        overflow: computed_overflow(overflow, overflow),
        item_is_replaced,
        ..NodeInputOf::default()
    };
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, Source::new("X"), ahem_style(10.0, 10.0)),
            ParagraphPart::Atomic {
                node: 2,
                size: surgeist_text::Size::new(10.0, 20.0),
            },
        ],
        ahem_style(10.0, 10.0),
        if parent_flow.direction() == Direction::Rtl {
            surgeist_text::ParagraphBaseDirection::RightToLeft
        } else {
            surgeist_text::ParagraphBaseDirection::LeftToRight
        },
    );
    let inner = PreparedParagraph::new(
        vec![
            ParagraphPart::text(3, Source::new("X"), ahem_style(5.0, 10.0)),
            ParagraphPart::Control {
                node: 4,
                kind: surgeist_text::ParagraphControlKind::ForcedBreak,
            },
            ParagraphPart::text(5, Source::new("X"), ahem_style(5.0, 10.0)),
        ],
        ahem_style(5.0, 10.0),
        if child_flow.direction() == Direction::Rtl {
            surgeist_text::ParagraphBaseDirection::RightToLeft
        } else {
            surgeist_text::ParagraphBaseDirection::LeftToRight
        },
    );
    let line_break = LineBreakInputOf::new()
        .with_writing_mode(child_flow.writing_mode())
        .with_direction(child_flow.direction());
    let tree = public_layout_tree(
        HashMap::from([
            (0, LayoutInputOf::box_input(root)),
            (1, paragraph.text_input(1)),
            (2, LayoutInputOf::box_input(atomic)),
            (3, inner.text_input(3)),
            (4, LayoutInputOf::line_break(line_break)),
            (5, inner.text_input(5)),
        ]),
        HashMap::from([
            (0, vec![1, 2]),
            (1, vec![]),
            (2, vec![3, 4, 5]),
            (3, vec![]),
            (4, vec![]),
            (5, vec![]),
        ]),
    );
    let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(0), inner.binding(2)])?;
    let available = parent_flow.physical_size(LogicalSizeOf::new(
        AvailableOf::definite(S::from_f64(100.0)),
        AvailableOf::MAX_CONTENT,
    ));
    compute_layout_with_text(
        &capture,
        0,
        LayoutRootRequestOf::viewport(available).unwrap(),
    )
}

#[derive(Clone)]
struct Fri06C03CachedAtomicTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    atomic_output: ComputeOutputOf<S>,
}

impl<S: LayoutScalar> Traverse for Fri06C03CachedAtomicTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;

    fn children(&self, node: Self::Node) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }

    fn child_count(&self, node: Self::Node) -> usize {
        self.tree.child_count(node)
    }

    fn child(&self, node: Self::Node, index: usize) -> Self::Node {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for Fri06C03CachedAtomicTree<S> {
    type MeasureError = ();

    fn node_input(&self, node: Self::Node) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }

    fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }

    fn cache_get(
        &self,
        node: Self::Node,
        _input: &ComputeInputOf<S>,
        _context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        if node == 1 {
            Some(ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                Size::new(S::from_f64(10.0), S::from_f64(10.0)),
                BaselinesOf::from_block_coordinates(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    Some(S::from_f64(8.0)),
                    None,
                ),
            ))
        } else {
            (node == 2).then_some(self.atomic_output)
        }
    }
}

#[test]
fn fri06_c03_atomic_baseline_visible_inner_and_non_visible_margin_edge_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let parallel = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let opposing_parent = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
        let opposing_child = FlowAxes::new(WritingMode::VerticalLr, Direction::Ltr);

        // Ahem5/LH10 contributes6.5px before the baseline, while the real
        // default forced-control allocation contributes12px before/4px after.
        // Its inner baseline12 plus the atomic's1px top margin selects13.
        let batch = prepared_nested_atomic_baseline_batch::<S>(
            parallel,
            parallel,
            Overflow::Visible,
            false,
        )
        .unwrap();
        let root = fri06_c02_final_node(&batch, 0);
        let inner = batch
            .placed_text()
            .iter()
            .find(|output| output.container() == 2)
            .unwrap()
            .layout();
        assert_eq!(
            inner.lines()[0].baseline(),
            13.0,
            "the inner common12px baseline is placed at atomic top1 plus12"
        );
        let inner_finalized = fri06_c03_fragment(&batch, 3)
            .finalized_fragment()
            .finalized_line();
        assert_eq!(inner_finalized.feedback().baseline, 12.0);
        let inner_selected = inner_finalized.selected_line();
        let font = inner_selected
            .request()
            .participants
            .iter()
            .find_map(|participant| {
                if let surgeist_text::PreparedParticipant::Text(unit) = participant {
                    Some(unit.view())
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(
            (font.ascent(), font.descent(), font.line_height()),
            (4.0, 1.0, 10.0)
        );
        assert_eq!(
            font.ascent() + (font.line_height() - font.ascent() - font.descent()) / 2.0,
            6.5
        );
        let control = inner_selected.request().controls[0].strut.unwrap();
        assert_eq!((control.baseline, control.line_extent), (12.0, 16.0));
        assert_eq!(
            fri06_c03_logical_block_start(parallel, fri06_c02_final_node(&batch, 2), root.size),
            S::from_f64(1.0)
        );
        assert_eq!(
            fri06_c03_fragment(&batch, 1).baseline().y,
            S::from_f64(13.0)
        );
        assert_eq!(root.size.height, S::from_f64(23.0));
        let outer = fri06_c03_fragment(&batch, 1)
            .finalized_fragment()
            .finalized_line();
        assert_eq!(
            (outer.feedback().ascent, outer.feedback().descent),
            (13.0, 10.0)
        );
        let error = prepared_nested_atomic_baseline_batch::<S>(
            opposing_parent,
            opposing_child,
            Overflow::Visible,
            false,
        )
        .unwrap_err();
        let LayoutErrorKindOf::Text(source) = error.kind() else {
            panic!("{error:?}")
        };
        assert!(matches!(
            source.detail(),
            Some(surgeist_text::ErrorDetail::InvalidTextProtocol {
                reason: surgeist_text::ProtocolError::UnsupportedContext {
                    field: surgeist_text::ProtocolContextField::Orientation
                }
            })
        ));

        for overflow in [
            Overflow::Clip,
            Overflow::Hidden,
            Overflow::Scroll,
            Overflow::Auto,
        ] {
            let batch =
                prepared_nested_atomic_baseline_batch::<S>(parallel, parallel, overflow, false)
                    .unwrap();
            let root = fri06_c02_final_node(&batch, 0);
            let atomic = fri06_c02_final_node(&batch, 2);
            let parent_fragment = fri06_c03_fragment(&batch, 1);
            assert_eq!(
                fri06_c03_logical_block_start(parallel, atomic, root.size),
                S::from_f64(1.0),
                "{overflow:?} falls back to the block-end margin edge"
            );
            assert_eq!(parent_fragment.baseline().y, S::from_f64(23.0));
            assert_eq!(root.size.height, S::from_f64(25.0));
        }

        let replaced_hidden =
            prepared_nested_atomic_baseline_batch::<S>(parallel, parallel, Overflow::Hidden, true)
                .unwrap();
        let root = fri06_c02_final_node(&replaced_hidden, 0);
        assert_eq!(
            fri06_c03_fragment(&replaced_hidden, 1).baseline().y,
            S::from_f64(23.0)
        );
        assert_eq!(root.size.height, S::from_f64(25.0));

        let atomic_style = NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(2.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            atomic_inline_participation: Some(fri06_c03_atomic_participation(
                0,
                InlineBreakOpportunity::Prohibited,
            )),
            ..NodeInputOf::default()
        };
        let root_style = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let tree = Fri06C03CachedAtomicTree {
            tree: public_layout_tree(
                HashMap::from([
                    (0, LayoutInputOf::box_input(root_style.clone())),
                    (
                        1,
                        LayoutInputOf::box_input(fri06_c03_atomic_style(
                            10.0,
                            10.0,
                            0.0,
                            0.0,
                            0,
                            InlineBreakOpportunity::Prohibited,
                        )),
                    ),
                    (2, LayoutInputOf::box_input(atomic_style.clone())),
                ]),
                HashMap::from([(0, vec![1, 2]), (1, Vec::new()), (2, Vec::new())]),
            ),
            atomic_output: ComputeOutputOf::from_sizes_and_baselines(
                Size::new(S::from_f64(10.0), S::from_f64(20.0)),
                Size::new(S::from_f64(10.0), S::from_f64(20.0)),
                BaselinesOf::from_block_coordinates(parallel, None, Some(S::from_f64(6.0))),
            ),
        };
        let last_only = compute_layout(
            &tree,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let root = fri06_c02_final_node(&last_only, 0);
        assert_eq!(
            fri06_c02_final_node(&last_only, 2).location.y,
            S::from_f64(2.0)
        );
        assert_eq!(fri06_c02_final_node(&last_only, 1).location.y, S::ZERO);
        assert_eq!(root.size.height, S::from_f64(24.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_atomic_baseline_absent_inner_uses_positive_and_negative_margin_edges_once() {
    fn assert_lane<S: LayoutScalar>() {
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("X", Id::from_u64(710), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
                ParagraphPart::Atomic {
                    node: 3,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let text = paragraph.text_input(1);
        let positive = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(3.0)),
                bottom: LengthAutoOf::px(S::from_f64(5.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..prepared_atomic_style(10.0, 10.0, 0.0, 0.0)
        };
        let negative = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(-2.0)),
                bottom: LengthAutoOf::px(S::from_f64(-3.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..prepared_atomic_style(10.0, 10.0, 0.0, 0.0)
        };
        let batch = prepared_mixed_batch(
            &paragraph,
            vec![
                (1, text, NodeInputOf::non_box()),
                (2, LayoutInputOf::box_input(positive.clone()), positive),
                (3, LayoutInputOf::box_input(negative.clone()), negative),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
            NodeInputOf::default(),
        );
        let root = fri06_c02_final_node(&batch, 0);
        assert_eq!(
            fri06_c03_fragment(&batch, 1).baseline().y,
            S::from_f64(18.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location.y, S::from_f64(3.0));
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.y,
            S::from_f64(11.0)
        );
        assert_eq!(root.size.height, S::from_f64(20.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_top_bottom_mixed_atomics_and_metric_controls_expand_one_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("X", Id::from_u64(720), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(10.0, 20.0),
                },
                ParagraphPart::Atomic {
                    node: 3,
                    size: surgeist_text::Size::new(10.0, 25.0),
                },
                ParagraphPart::Atomic {
                    node: 4,
                    size: surgeist_text::Size::new(10.0, 30.0),
                },
                ParagraphPart::Atomic {
                    node: 5,
                    size: surgeist_text::Size::new(10.0, 35.0),
                },
                ParagraphPart::Control {
                    node: 6,
                    kind: surgeist_text::ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphPart::Control {
                    node: 7,
                    kind: surgeist_text::ParagraphControlKind::InlineBoundaryEnd,
                },
                ParagraphPart::Control {
                    node: 8,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(
                    9,
                    Source::identified("XX", Id::from_u64(721), 0),
                    ahem_style(5.0, 10.0),
                ),
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let text = paragraph.text_input(1);
        let top_with_margins = NodeInputOf {
            vertical_align: VerticalAlign::Top,
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(2.0)),
                bottom: LengthAutoOf::px(S::from_f64(3.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..prepared_atomic_style(10.0, 20.0, 0.0, 0.0)
        };
        let top = NodeInputOf {
            vertical_align: VerticalAlign::Top,
            ..prepared_atomic_style(10.0, 25.0, 0.0, 0.0)
        };
        let bottom_with_margins = NodeInputOf {
            vertical_align: VerticalAlign::Bottom,
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(4.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..prepared_atomic_style(10.0, 30.0, 0.0, 0.0)
        };
        let bottom = NodeInputOf {
            vertical_align: VerticalAlign::Bottom,
            ..prepared_atomic_style(10.0, 35.0, 0.0, 0.0)
        };
        let top_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(22.0), S::from_f64(6.0))
                .unwrap();
        let bottom_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(32.0), S::from_f64(20.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(10.0), S::from_f64(8.0))
                .unwrap();
        let batch = prepared_mixed_batch(
            &paragraph,
            vec![
                (1, text, NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(top_with_margins.clone()),
                    top_with_margins,
                ),
                (3, LayoutInputOf::box_input(top.clone()), top),
                (
                    4,
                    LayoutInputOf::box_input(bottom_with_margins.clone()),
                    bottom_with_margins,
                ),
                (5, LayoutInputOf::box_input(bottom.clone()), bottom),
                (
                    6,
                    LayoutInputOf::inline_boundary(
                        InlineBoundaryInputOf::new(InlineBoundaryKind::Start, top_metrics)
                            .with_vertical_align(VerticalAlign::Top),
                    ),
                    NodeInputOf::non_box(),
                ),
                (
                    7,
                    LayoutInputOf::inline_boundary(
                        InlineBoundaryInputOf::new(InlineBoundaryKind::End, bottom_metrics)
                            .with_vertical_align(VerticalAlign::Bottom),
                    ),
                    NodeInputOf::non_box(),
                ),
                (
                    8,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(break_metrics)),
                    NodeInputOf::non_box(),
                ),
                (9, paragraph.text_input(9), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
            NodeInputOf::default(),
        );

        let root = fri06_c02_final_node(&batch, 0);
        // Real5px Ahem contributes ascent6.5/descent3.5 at line-height10.
        // The8px break strut selects baseline43; actual descent closes at46.5.
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size.height,
            S::from_f64(46.5)
        );
        assert_eq!(root.size.height, S::from_f64(47.0));
        assert_eq!(
            fri06_c03_fragment(&batch, 1).baseline().y,
            S::from_f64(18.0)
        );
        assert_eq!(
            fri06_c03_fragment(&batch, 9).baseline().y,
            S::from_f64(43.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location.y, S::from_f64(2.0));
        let first_line = fri06_c03_fragment(&batch, 1)
            .finalized_fragment()
            .finalized_line();
        let top_slot = first_line
            .atomic_slots()
            .find(|slot| slot.occurrence.view().original_box().id() == Id::from_u64(2))
            .unwrap();
        let surgeist_text::AtomicLineAllocation::InFlow {
            flow, box_geometry, ..
        } = top_slot.allocation
        else {
            panic!("the top-aligned fixture supplies a genuine in-flow atomic")
        };
        assert_eq!(
            (flow.before_baseline, box_geometry.border.origin.y),
            (25.0, -23.0)
        );
        let line_top = batch.placed_text()[0].layout().lines()[0].bounds().origin.y;
        assert_eq!(line_top, 0.0);
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.y,
            S::from_f64(line_top + 2.0),
            "Top aligns the margin box, so the2px top margin places the border below it"
        );
        assert_eq!(fri06_c02_final_node(&batch, 3).location.y, S::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 4).location.y, S::from_f64(1.0));
        assert_eq!(fri06_c02_final_node(&batch, 5).location.y, S::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).location.y, S::from_f64(6.0));
        assert_eq!(
            fri06_c02_final_node(&batch, 7).location.y,
            S::from_f64(23.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 8).location.y,
            S::from_f64(18.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_mixed_break_boundary_and_hidden_output_publish_from_one_source_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let boundary_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let break_metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(12.0), S::from_f64(9.0))
                .unwrap();
        let atomic_style = prepared_atomic_style(10.0, 10.0, 0.0, 0.0);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("X", Id::from_u64(401), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
                ParagraphPart::Control {
                    node: 3,
                    kind: surgeist_text::ParagraphControlKind::InlineBoundaryStart,
                },
                ParagraphPart::text(
                    4,
                    Source::identified("X", Id::from_u64(402), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Control {
                    node: 5,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(
                    7,
                    Source::identified("X", Id::from_u64(403), 0),
                    ahem_style(5.0, 10.0),
                ),
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let batch = prepared_mixed_batch(
            &paragraph,
            vec![
                (1, paragraph.text_input(1), NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(atomic_style.clone()),
                    atomic_style,
                ),
                (
                    3,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        boundary_metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                (4, paragraph.text_input(4), NodeInputOf::non_box()),
                (
                    5,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(break_metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    6,
                    LayoutInputOf::line_break(LineBreakInputOf::new().hidden()),
                    NodeInputOf::non_box(),
                ),
                (7, paragraph.text_input(7), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(40.0)),
            NodeInputOf::default(),
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(32.0)
        );
        assert_eq!(fri06_c02_final_node(&batch, 3).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 5).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 6).location, Point::ZERO);
        assert_eq!(fri06_c02_final_node(&batch, 3).source_index.get(), 2);
        assert_eq!(fri06_c02_final_node(&batch, 5).source_index.get(), 4);
        assert_eq!(fri06_c02_final_node(&batch, 6).source_index.get(), 5);
        let source_fragments =
            super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
        assert_eq!(source_fragments.len(), 3);
        assert_eq!(
            source_fragments
                .iter()
                .map(|entry| entry.5)
                .collect::<Vec<_>>(),
            [0, 0, 1]
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.x,
            S::from_f64(10.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.x,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 5).location.x,
            S::from_f64(30.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_forced_break_splits_min_content_indivisible_groups_both_scalars() {
    fn atomic<S: LayoutScalar>(
        node: u32,
        extent: f64,
        following_break: InlineBreakOpportunity,
    ) -> (u32, LayoutInputOf<S>, NodeInputOf<S>) {
        let style = fri06_c03_atomic_style(extent, 10.0, 0.0, 0.0, 0, following_break);
        (node, LayoutInputOf::box_input(style.clone()), style)
    }

    fn children<S: LayoutScalar>() -> Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)> {
        vec![
            atomic(1, 12.0, InlineBreakOpportunity::Prohibited),
            atomic(2, 18.0, InlineBreakOpportunity::Prohibited),
            (
                3,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new().with_metrics(
                        InlineMetricsOf::from_line_height_and_baseline(
                            S::from_f64(10.0),
                            S::from_f64(8.0),
                        )
                        .unwrap(),
                    ),
                ),
                NodeInputOf::non_box(),
            ),
            atomic(4, 17.0, InlineBreakOpportunity::Prohibited),
            atomic(5, 23.0, InlineBreakOpportunity::Allowed),
            atomic(6, 25.0, InlineBreakOpportunity::Prohibited),
        ]
    }

    fn lane<S: LayoutScalar>() -> (S, S, S, S, S) {
        let min = fri06_c03_mixed_batch(children(), AvailableOf::MIN_CONTENT);
        let max = fri06_c03_mixed_batch(children(), AvailableOf::MAX_CONTENT);

        (
            fri06_c02_final_node(&min, 0).size.width,
            fri06_c02_final_node(&max, 0).size.width,
            fri06_c02_final_node(&min, 3).location.x,
            fri06_c02_final_node(&min, 4).location.x,
            fri06_c02_final_node(&min, 6).location.x,
        )
    }

    let f32_lane = lane::<f32>();
    let f64_lane = lane::<f64>();
    assert_eq!(
        [
            f64::from(f32_lane.0),
            f64::from(f32_lane.1),
            f64::from(f32_lane.2),
            f64::from(f32_lane.3),
            f64::from(f32_lane.4),
            f64_lane.0,
            f64_lane.1,
            f64_lane.2,
            f64_lane.3,
            f64_lane.4,
        ],
        [40.0, 65.0, 30.0, 0.0, 0.0, 40.0, 65.0, 30.0, 0.0, 0.0]
    );
}

#[test]
fn fri06_c03_strut_adjacent_final_breaks_preserve_empty_following_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        let batch = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(60.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 1).location.y,
            S::from_f64(15.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.y,
            S::from_f64(35.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn control_leading_trailing_only_child_and_boundary_edges_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(20.0), S::from_f64(15.0))
                .unwrap();
        // This case owns numeric control envelopes; real inline boxes supply
        // adjacent geometry, while separate prepared cases own Text semantics.
        let text = || {
            let atomic =
                fri06_c03_atomic_style(10.0, 10.0, 0.0, 0.0, 0, InlineBreakOpportunity::Prohibited);
            (2, LayoutInputOf::box_input(atomic.clone()), atomic)
        };

        let leading = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                text(),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&leading, 0).size.height,
            S::from_f64(40.0)
        );
        assert_eq!(
            fri06_c02_final_node(&leading, 1).location.y,
            S::from_f64(15.0)
        );

        let trailing = fri06_c03_mixed_batch(
            vec![
                text(),
                (
                    3,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&trailing, 0).size.height,
            S::from_f64(40.0)
        );
        assert_eq!(
            fri06_c02_final_node(&trailing, 3).location,
            Point::new(S::from_f64(10.0), S::from_f64(15.0))
        );

        let only_break = fri06_c03_mixed_batch(
            vec![(
                1,
                LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                NodeInputOf::non_box(),
            )],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&only_break, 0).size.height,
            S::from_f64(40.0)
        );

        let only_boundary = fri06_c03_mixed_batch(
            vec![(
                1,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                    InlineBoundaryKind::Start,
                    metrics,
                )),
                NodeInputOf::non_box(),
            )],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&only_boundary, 0).size.height,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&only_boundary, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );

        let adjacent_boundaries = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                (
                    2,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&adjacent_boundaries, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );
        assert_eq!(
            fri06_c02_final_node(&adjacent_boundaries, 2).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );

        let boundaries = fri06_c03_mixed_batch(
            vec![
                (
                    1,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::Start,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
                text(),
                (
                    3,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
            AvailableOf::MAX_CONTENT,
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 0).size.height,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 1).location,
            Point::new(S::ZERO, S::from_f64(15.0))
        );
        assert_eq!(
            fri06_c02_final_node(&boundaries, 3).location,
            Point::new(S::from_f64(10.0), S::from_f64(15.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c03_control_visible_break_uses_each_unequal_lines_alignment_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let metrics =
            InlineMetricsOf::from_line_height_and_baseline(S::from_f64(12.0), S::from_f64(9.0))
                .unwrap();
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("XX"), ahem_style(10.0, 10.0)),
                ParagraphPart::Control {
                    node: 2,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(3, Source::new("X"), ahem_style(6.0, 10.0)),
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let batch = prepared_mixed_batch(
            &paragraph,
            vec![
                (1, paragraph.text_input(1), NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
                (3, paragraph.text_input(3), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(40.0)),
            NodeInputOf {
                text_align: TextAlign::LegacyCenter,
                ..NodeInputOf::default()
            },
        );

        assert_eq!(
            fri06_c02_final_node(&batch, 1).location.x,
            S::from_f64(10.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 2).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 3).location.x,
            S::from_f64(17.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn atomic_control_line_bands_rewrap_transition_and_progress_both_scalars() {
    fn atomic_participation(following_break: InlineBreakOpportunity) -> AtomicInlineParticipation {
        fri06_c03_atomic_participation(0, following_break)
    }

    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let float = |node, side, inline, block| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)),
                side,
                None,
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let atomic = |node, inline, block, following_break| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)),
                Float::None,
                Some(atomic_participation(following_break)),
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(8.0), S::from_f64(2.0)).unwrap();

        let mixed = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Left, 30.0, 30.0),
                (
                    2,
                    LayoutInputOf::box_input(fri06_c03_atomic_style(
                        20.0,
                        10.0,
                        0.0,
                        0.0,
                        0,
                        InlineBreakOpportunity::Prohibited,
                    )),
                    NodeInputOf::non_box(),
                ),
                atomic(3, 10.0, 10.0, InlineBreakOpportunity::Prohibited),
                (
                    4,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&mixed, 2).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&mixed, 3).location.x,
            S::from_f64(50.0)
        );
        assert_eq!(
            fri06_c02_final_node(&mixed, 4).location.x,
            S::from_f64(60.0)
        );
        assert_eq!(fri06_c02_final_node(&mixed, 4).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&mixed, 0).location, Point::ZERO);
        assert_eq!(
            fri06_c02_final_node(&mixed, 0).size.width,
            S::from_f64(100.0)
        );

        let right = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Right, 30.0, 20.0),
                (
                    2,
                    LayoutInputOf::box_input(fri06_c03_atomic_style(
                        20.0,
                        10.0,
                        0.0,
                        0.0,
                        0,
                        InlineBreakOpportunity::Prohibited,
                    )),
                    NodeInputOf::non_box(),
                ),
                atomic(3, 20.0, 10.0, InlineBreakOpportunity::Prohibited),
                (
                    4,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        assert_eq!(fri06_c02_final_node(&right, 2).location.x, S::ZERO);
        assert_eq!(
            fri06_c02_final_node(&right, 3).location.x,
            S::from_f64(20.0)
        );
        assert_eq!(
            fri06_c02_final_node(&right, 4).location.x,
            S::from_f64(40.0)
        );

        let opposing = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Left, 30.0, 20.0),
                float(2, Float::Right, 20.0, 20.0),
                (
                    3,
                    LayoutInputOf::box_input(fri06_c03_atomic_style(
                        20.0,
                        10.0,
                        0.0,
                        0.0,
                        0,
                        InlineBreakOpportunity::Prohibited,
                    )),
                    NodeInputOf::non_box(),
                ),
                atomic(4, 30.0, 10.0, InlineBreakOpportunity::Prohibited),
                (
                    5,
                    LayoutInputOf::inline_boundary(InlineBoundaryInputOf::new(
                        InlineBoundaryKind::End,
                        metrics,
                    )),
                    NodeInputOf::non_box(),
                ),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&opposing, 3).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&opposing, 4).location.x,
            S::from_f64(50.0)
        );
        assert_eq!(
            fri06_c02_final_node(&opposing, 5).location.x,
            S::from_f64(80.0)
        );

        let rewrapped = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Left, 60.0, 10.0),
                atomic(2, 30.0, 10.0, InlineBreakOpportunity::Allowed),
                atomic(3, 30.0, 10.0, InlineBreakOpportunity::Allowed),
                atomic(4, 30.0, 10.0, InlineBreakOpportunity::Prohibited),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&rewrapped, 2).location,
            Point::new(S::from_f64(60.0), S::ZERO)
        );
        assert_eq!(
            fri06_c02_final_node(&rewrapped, 3).location,
            Point::new(S::ZERO, S::from_f64(10.0))
        );
        assert_eq!(
            fri06_c02_final_node(&rewrapped, 4).location,
            Point::new(S::from_f64(30.0), S::from_f64(10.0))
        );
        let forced = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Left, 30.0, 15.0),
                atomic(2, 20.0, 20.0, InlineBreakOpportunity::Prohibited),
                (
                    3,
                    LayoutInputOf::line_break(
                        LineBreakInputOf::new().with_metrics(
                            InlineMetricsOf::from_line_height_and_baseline(
                                S::from_f64(20.0),
                                S::from_f64(15.0),
                            )
                            .unwrap(),
                        ),
                    ),
                    NodeInputOf::non_box(),
                ),
                atomic(4, 10.0, 10.0, InlineBreakOpportunity::Prohibited),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&forced, 2).location.x,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&forced, 4).location,
            Point::new(S::ZERO, S::from_f64(30.0))
        );

        let no_space = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![
                float(1, Float::Left, 50.0, 20.0),
                float(2, Float::Right, 50.0, 20.0),
                atomic(3, 120.0, 10.0, InlineBreakOpportunity::Prohibited),
            ],
        );
        assert_eq!(
            fri06_c02_final_node(&no_space, 3).location,
            Point::new(S::ZERO, S::from_f64(20.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_ordinary_block_keeps_outer_edge_and_inherits_parent_float_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for child_direction in [Direction::Ltr, Direction::Rtl] {
            let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
            let root_style = NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                ..NodeInputOf::default()
            };
            let float_style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(40.0), S::from_f64(20.0)),
                Float::Left,
                None,
            );
            let ordinary_style = NodeInputOf {
                display: Display::Block,
                direction: child_direction,
                text_align: TextAlign::LegacyLeft,
                margin: Edges {
                    right: LengthAutoOf::px(S::from_f64(5.0)),
                    left: LengthAutoOf::px(S::from_f64(5.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                border: Edges::all(LengthOf::px(S::from_f64(2.0))),
                padding: Edges::all(LengthOf::px(S::from_f64(3.0))),
                ..NodeInputOf::default()
            };
            // Actual Unicode zero-width opportunities separate three3em words.
            let paragraph = PreparedParagraph::new(
                vec![ParagraphPart::text(
                    3,
                    Source::new("XXX\u{200b}XXX\u{200b}XXX"),
                    ahem_style(10.0, 10.0),
                )],
                ahem_style(10.0, 10.0),
                if child_direction == Direction::Rtl {
                    surgeist_text::ParagraphBaseDirection::RightToLeft
                } else {
                    surgeist_text::ParagraphBaseDirection::LeftToRight
                },
            );
            let tree = public_layout_tree(
                HashMap::from([
                    (0, LayoutInputOf::box_input(root_style.clone())),
                    (1, LayoutInputOf::box_input(float_style.clone())),
                    (2, LayoutInputOf::box_input(ordinary_style.clone())),
                    (3, paragraph.text_input(3)),
                ]),
                HashMap::from([
                    (0, vec![1, 2]),
                    (1, Vec::new()),
                    (2, vec![3]),
                    (3, Vec::new()),
                ]),
            );

            let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(2)]).unwrap();
            let batch = compute_layout_with_text(
                &capture,
                0,
                LayoutRootRequestOf::viewport(Size::new(
                    AvailableOf::definite(S::from_f64(100.0)),
                    AvailableOf::definite(S::from_f64(80.0)),
                ))
                .unwrap(),
            )
            .unwrap();

            let ordinary = fri06_c02_final_node(&batch, 2);
            assert_eq!(ordinary.source_index, SourceIndex::new(1));
            assert_eq!(ordinary.location, Point::new(S::from_f64(5.0), S::ZERO));
            assert_eq!(ordinary.size.width, S::from_f64(90.0));
            assert_eq!(
                fri06_c02_final_node(&batch, 3).source_index,
                SourceIndex::ZERO
            );

            let fragments = batch
                .final_inline_fragments()
                .iter()
                .filter(|entry| entry.owner() == &crate::InlineFragmentOwner::Node(3))
                .map(|entry| entry.fragment())
                .collect::<Vec<_>>();
            let ranges = fragments
                .iter()
                .map(|fragment| {
                    let view = fragment.finalized_fragment().view();
                    assert_eq!(view.origin(), Some(paragraph.origin(3)));
                    view.local_range().unwrap()
                })
                .collect::<Vec<_>>();
            assert_eq!(ranges.first().unwrap().start, 0);
            assert_eq!(ranges.last().unwrap().end, 11);
            assert!(ranges.windows(2).all(|pair| pair[0].end == pair[1].start));
            for (line, x, baseline_y) in [(0, 35.0, 13.0), (1, 35.0, 23.0), (2, 5.0, 33.0)] {
                let rows = fragments
                    .iter()
                    .filter(|fragment| fragment.line_index() == line)
                    .collect::<Vec<_>>();
                assert!(!rows.is_empty());
                let rects = rows
                    .iter()
                    .filter_map(|fragment| fragment.rect())
                    .collect::<Vec<_>>();
                let left = rects
                    .iter()
                    .map(|rect| rect.origin().x)
                    .fold(S::INFINITY, S::min);
                let right = rects
                    .iter()
                    .map(|rect| rect.origin().x + rect.size().width)
                    .fold(-S::INFINITY, S::max);
                assert_eq!(left, S::from_f64(x));
                assert_eq!(right - left, S::from_f64(30.0));
                // The common physical local-x0 anchor is the line's left edge,
                // independently of its paragraph's inline progression.
                let baseline_x = x;
                assert_eq!(
                    rows.iter()
                        .map(|fragment| fragment.baseline())
                        .collect::<Vec<_>>(),
                    vec![Point::new(S::from_f64(baseline_x), S::from_f64(baseline_y)); rows.len()]
                );
            }
            assert!(fragments.iter().all(|fragment| fragment.line_index() < 3));
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_nested_local_float_keeps_combined_ledger_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let root_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(100.0)),
                PreferredSizeOf::px(S::from_f64(80.0)),
            ),
            ..NodeInputOf::default()
        };
        let preceding_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
            ..NodeInputOf::default()
        };
        let parent_float_style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(30.0), S::from_f64(30.0)),
            Float::Left,
            None,
        );
        let ordinary_style = NodeInputOf {
            display: Display::Block,
            ..NodeInputOf::default()
        };
        let local_float_style = fri06_c04_line_box(
            flow_axes,
            LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(20.0)),
            Float::Right,
            None,
        );
        let paragraph = PreparedParagraph::plain(5, Source::new("XXXX"), ahem_style(10.0, 10.0));
        let tree = public_layout_tree(
            HashMap::from([
                (0, LayoutInputOf::box_input(root_style.clone())),
                (1, LayoutInputOf::box_input(preceding_style.clone())),
                (2, LayoutInputOf::box_input(parent_float_style.clone())),
                (3, LayoutInputOf::box_input(ordinary_style.clone())),
                (4, LayoutInputOf::box_input(local_float_style.clone())),
                (5, paragraph.text_input(5)),
            ]),
            HashMap::from([
                (0, vec![1, 2, 3]),
                (1, Vec::new()),
                (2, Vec::new()),
                (3, vec![4, 5]),
                (4, Vec::new()),
                (5, Vec::new()),
            ]),
        );

        let capture = TextLayoutInput::try_new(&tree, 0, vec![paragraph.binding(3)]).unwrap();
        let batch = compute_layout_with_text(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(100.0)),
                AvailableOf::definite(S::from_f64(80.0)),
            ))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            fri06_c02_final_node(&batch, 2).source_index,
            SourceIndex::new(1)
        );
        let ordinary = fri06_c02_final_node(&batch, 3);
        assert_eq!(ordinary.source_index, SourceIndex::new(2));
        assert_eq!(ordinary.location, Point::ZERO);
        assert_eq!(ordinary.size.width, S::from_f64(100.0));

        let local_float = fri06_c02_final_node(&batch, 4);
        assert_eq!(local_float.source_index, SourceIndex::ZERO);
        assert_eq!(local_float.location, Point::new(S::from_f64(80.0), S::ZERO));
        assert_eq!(
            fri06_c02_final_node(&batch, 5).source_index,
            SourceIndex::new(1)
        );
        let fragment = fri06_c03_fragment(&batch, 5);
        assert_eq!(fragment.line_index(), 0);
        assert_eq!(
            fragment.rect().unwrap().origin(),
            Point::new(S::from_f64(30.0), S::ZERO)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_role_exact_positive_and_negative_matrix_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for flow_axes in [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ] {
            let sized = |display, overflow, item_is_replaced, position, float| {
                let mut style = NodeInputOf {
                    display,
                    writing_mode: flow_axes.writing_mode(),
                    direction: flow_axes.direction(),
                    overflow,
                    item_is_replaced,
                    position,
                    float,
                    size: flow_axes.physical_size(LogicalSizeOf::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    )),
                    ..NodeInputOf::default()
                };
                if display.is_inline_level() {
                    style.atomic_inline_participation = Some(fri06_c03_atomic_participation(
                        0,
                        InlineBreakOpportunity::Prohibited,
                    ));
                }
                style
            };
            let float_style = sized(
                Display::Block,
                ComputedOverflow::VISIBLE,
                false,
                Position::Relative,
                Float::Left,
            );
            let float_style = NodeInputOf {
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::px(S::from_f64(30.0)),
                )),
                ..float_style
            };
            let hidden = computed_overflow(Overflow::Hidden, Overflow::Hidden);
            let scroll = computed_overflow(Overflow::Scroll, Overflow::Scroll);
            let auto = computed_overflow(Overflow::Auto, Overflow::Auto);
            let clip = computed_overflow(Overflow::Clip, Overflow::Clip);

            let positive = [
                ("flex", Display::Flex, ComputedOverflow::VISIBLE, false),
                (
                    "replaced-flex",
                    Display::Flex,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                ("grid", Display::Grid, ComputedOverflow::VISIBLE, false),
                (
                    "replaced-grid",
                    Display::Grid,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                (
                    "grid-lanes",
                    Display::GridLanes,
                    ComputedOverflow::VISIBLE,
                    false,
                ),
                (
                    "replaced-grid-lanes",
                    Display::GridLanes,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                ("block-hidden", Display::Block, hidden, false),
                ("block-scroll", Display::Block, scroll, false),
                ("block-auto", Display::Block, auto, false),
                (
                    "replaced-block-visible",
                    Display::Block,
                    ComputedOverflow::VISIBLE,
                    true,
                ),
                ("replaced-block-clip", Display::Block, clip, true),
                ("replaced-block-hidden", Display::Block, hidden, true),
                ("replaced-block-scroll", Display::Block, scroll, true),
                ("replaced-block-auto", Display::Block, auto, true),
            ];
            for (label, display, overflow, item_is_replaced) in positive {
                let subject = sized(
                    display,
                    overflow,
                    item_is_replaced,
                    Position::Relative,
                    Float::None,
                );
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                let origin = fri06_c04_logical_origin(flow_axes, output);
                assert_eq!(
                    origin,
                    LogicalPointOf::new(S::from_f64(40.0), S::ZERO),
                    "positive BFC role {label} did not avoid the active float in {flow_axes:?}",
                );
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    S::from_f64(20.0),
                    "positive BFC role {label} changed its definite size in {flow_axes:?}",
                );
            }

            let ordinary_negative = [
                ("block-visible", ComputedOverflow::VISIBLE, false),
                ("block-clip", clip, false),
            ];
            for (label, overflow, item_is_replaced) in ordinary_negative {
                let subject = sized(
                    Display::Block,
                    overflow,
                    item_is_replaced,
                    Position::Relative,
                    Float::None,
                );
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, fri06_c02_final_node(&batch, 2)),
                    LogicalPointOf::new(S::ZERO, S::ZERO),
                    "negative BFC role {label} moved its ordinary outer edge in {flow_axes:?}",
                );
            }

            for display in [
                Display::InlineBlock,
                Display::InlineGrid,
                Display::InlineGridLanes,
            ] {
                let subject = sized(display, hidden, false, Position::Relative, Float::None);
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, output),
                    LogicalPointOf::new(S::from_f64(40.0), S::ZERO),
                    "{display:?} must participate in the float-adjusted inline line",
                );
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    S::from_f64(20.0)
                );
            }

            let absolute = sized(
                Display::Flex,
                hidden,
                false,
                Position::Absolute,
                Float::None,
            );
            let floating = sized(
                Display::Flex,
                hidden,
                false,
                Position::Relative,
                Float::Left,
            );
            let none = sized(
                Display::None,
                hidden,
                false,
                Position::Relative,
                Float::None,
            );
            for (label, subject, expected_origin, expected_inline_size) in [
                ("absolute", absolute, S::ZERO, S::from_f64(20.0)),
                ("floating", floating, S::from_f64(40.0), S::from_f64(20.0)),
                ("display-none", none, S::ZERO, S::ZERO),
            ] {
                let batch = fri06_c04_bfc_batch(
                    flow_axes,
                    vec![1, 2],
                    vec![
                        (1, float_style.clone(), Vec::new()),
                        (2, subject, Vec::new()),
                    ],
                );
                let output = fri06_c02_final_node(&batch, 2);
                if label == "display-none" {
                    assert_eq!(output.location, Point::ZERO);
                } else {
                    assert_eq!(
                        fri06_c04_logical_origin(flow_axes, output).inline,
                        expected_origin,
                        "{label} entered block-child BFC avoidance in {flow_axes:?}",
                    );
                }
                assert_eq!(
                    flow_axes.logical_size(output.size).inline,
                    expected_inline_size,
                    "{label} changed size in {flow_axes:?}",
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_size_auto_definite_zero_overwide_margin_boxes_and_clear_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for flow_axes in [
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl),
        ] {
            let box_style = |inline: PreferredSizeOf<S>, block: f64| NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    inline,
                    PreferredSizeOf::px(S::from_f64(block)),
                )),
                ..NodeInputOf::default()
            };
            let floated = |side, inline, block| NodeInputOf {
                float: side,
                overflow: ComputedOverflow::VISIBLE,
                ..box_style(PreferredSizeOf::px(S::from_f64(inline)), block)
            };
            let margin = |inline_start: f64, inline_end: f64| {
                flow_axes.physical_edges(
                    crate::geometry::LogicalEdgesOf::new(
                        S::from_f64(inline_start),
                        S::from_f64(inline_end),
                        S::ZERO,
                        S::ZERO,
                    )
                    .map(LengthAutoOf::px),
                )
            };
            let run = |subject: NodeInputOf<S>, floats: Vec<NodeInputOf<S>>| {
                let mut children = Vec::new();
                let mut nodes = Vec::new();
                for (index, float) in floats.into_iter().enumerate() {
                    let node = u32::try_from(index + 1).unwrap();
                    children.push(node);
                    nodes.push((node, float, Vec::new()));
                }
                let subject_node = u32::try_from(children.len() + 1).unwrap();
                children.push(subject_node);
                nodes.push((subject_node, subject, Vec::new()));
                let batch = fri06_c04_bfc_batch(flow_axes, children, nodes);
                fri06_c02_final_node(&batch, subject_node)
            };

            for (display, overflow) in [
                (
                    Display::Block,
                    computed_overflow(Overflow::Hidden, Overflow::Hidden),
                ),
                (Display::Flex, ComputedOverflow::VISIBLE),
                (Display::Grid, ComputedOverflow::VISIBLE),
                (Display::GridLanes, ComputedOverflow::VISIBLE),
            ] {
                let auto = run(
                    NodeInputOf {
                        display,
                        overflow,
                        margin: margin(10.0, 10.0),
                        ..box_style(PreferredSizeOf::AUTO, 10.0)
                    },
                    vec![floated(Float::Left, 40.0, 20.0)],
                );
                assert_eq!(
                    fri06_c04_logical_origin(flow_axes, auto),
                    LogicalPointOf::new(S::from_f64(50.0), S::ZERO),
                    "{display:?} auto inline placement did not use its selected band",
                );
                assert_eq!(
                    flow_axes.logical_size(auto.size).inline,
                    S::from_f64(40.0),
                    "{display:?} auto inline size was not saturated to its selected band",
                );
            }

            let definite = run(
                NodeInputOf {
                    margin: margin(5.0, 5.0),
                    ..box_style(PreferredSizeOf::px(S::from_f64(50.0)), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, definite),
                LogicalPointOf::new(S::from_f64(45.0), S::ZERO),
            );

            let spanning = run(
                box_style(PreferredSizeOf::px(S::from_f64(70.0)), 20.0),
                vec![
                    floated(Float::Left, 20.0, 10.0),
                    floated(Float::Right, 90.0, 20.0),
                ],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, spanning),
                LogicalPointOf::new(S::ZERO, S::from_f64(30.0)),
                "complete-span collision must observe the later-starting float",
            );

            let zero = run(
                NodeInputOf {
                    margin: margin(35.0, 35.0),
                    ..box_style(PreferredSizeOf::px(S::ZERO), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, zero),
                LogicalPointOf::new(S::from_f64(35.0), S::from_f64(20.0)),
            );
            assert_eq!(flow_axes.logical_size(zero.size).inline, S::ZERO);

            let overwide = run(
                NodeInputOf {
                    margin: margin(5.0, 5.0),
                    ..box_style(PreferredSizeOf::px(S::from_f64(120.0)), 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, overwide),
                LogicalPointOf::new(S::from_f64(5.0), S::from_f64(20.0)),
            );

            let cleared = run(
                NodeInputOf {
                    clear: Clear::Left,
                    margin: margin(10.0, 10.0),
                    ..box_style(PreferredSizeOf::AUTO, 10.0)
                },
                vec![
                    floated(Float::Left, 40.0, 20.0),
                    floated(Float::Right, 30.0, 40.0),
                ],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, cleared),
                LogicalPointOf::new(S::from_f64(10.0), S::from_f64(20.0)),
            );
            assert_eq!(
                flow_axes.logical_size(cleared.size).inline,
                S::from_f64(50.0),
            );

            let ordinary = run(
                NodeInputOf {
                    overflow: ComputedOverflow::VISIBLE,
                    margin: margin(10.0, 10.0),
                    ..box_style(PreferredSizeOf::AUTO, 10.0)
                },
                vec![floated(Float::Left, 40.0, 20.0)],
            );
            assert_eq!(
                fri06_c04_logical_origin(flow_axes, ordinary),
                LogicalPointOf::new(S::from_f64(10.0), S::ZERO),
            );
            assert_eq!(
                flow_axes.logical_size(ordinary.size).inline,
                S::from_f64(80.0),
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_size_auto_block_encloses_inset_float_mixed_and_nested_both_scalars() {
    fn float_style<S: LayoutScalar>(
        flow_axes: FlowAxes,
        inline: f64,
        block: f64,
        clear: Clear,
    ) -> NodeInputOf<S> {
        NodeInputOf {
            display: Display::Block,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            float: Float::Left,
            clear,
            size: flow_axes
                .physical_size(LogicalSizeOf::new(S::from_f64(inline), S::from_f64(block)))
                .map(PreferredSizeOf::px),
            margin: flow_axes
                .physical_edges(LogicalEdgesOf::new(
                    S::from_f64(1.0),
                    S::from_f64(2.0),
                    S::from_f64(2.0),
                    S::from_f64(3.0),
                ))
                .map(LengthAutoOf::px),
            ..NodeInputOf::default()
        }
    }

    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in root_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let padding = LogicalEdgesOf::new(
                S::from_f64(1.0),
                S::from_f64(1.0),
                S::from_f64(3.0),
                S::from_f64(5.0),
            );
            let border = LogicalEdgesOf::new(
                S::from_f64(1.0),
                S::from_f64(1.0),
                S::from_f64(2.0),
                S::from_f64(4.0),
            );
            let root_style = NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: flow_axes.physical_size(LogicalSizeOf::new(
                    PreferredSizeOf::px(S::from_f64(80.0)),
                    PreferredSizeOf::AUTO,
                )),
                padding: flow_axes.physical_edges(padding).map(LengthOf::px),
                border: flow_axes.physical_edges(border).map(LengthOf::px),
                ..NodeInputOf::default()
            };
            let float = float_style(flow_axes, 20.0, 10.0, Clear::None);
            let float_only = fri06_c04_front_door_batch(
                root_style.clone(),
                LogicalSizeOf::new(
                    AvailableOf::definite(S::from_f64(80.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                vec![1],
                vec![(
                    1,
                    LayoutInputOf::box_input(float.clone()),
                    float.clone(),
                    Vec::new(),
                )],
            );
            let root = fri06_c02_final_node(&float_only, 0);
            assert_eq!(
                flow_axes.logical_size(root.size).block,
                S::from_f64(29.0),
                "float-only auto block size must count the five-unit start inset once for {flow_axes:?}",
            );
            let floated = fri06_c02_final_node(&float_only, 1);
            let floated_origin = flow_axes.logical_point(floated.location, floated.size, root.size);
            assert_eq!(floated_origin.block, S::from_f64(7.0));

            let atomic = NodeInputOf {
                display: Display::InlineBlock,
                writing_mode,
                direction,
                size: flow_axes
                    .physical_size(LogicalSizeOf::new(S::from_f64(20.0), S::from_f64(10.0)))
                    .map(PreferredSizeOf::px),
                atomic_inline_participation: Some(fri06_atomic_participation()),
                ..NodeInputOf::default()
            };
            let mixed = fri06_c04_front_door_batch(
                root_style,
                LogicalSizeOf::new(
                    AvailableOf::definite(S::from_f64(80.0)),
                    AvailableOf::MAX_CONTENT,
                ),
                vec![1, 2],
                vec![
                    (
                        1,
                        LayoutInputOf::box_input(float.clone()),
                        float,
                        Vec::new(),
                    ),
                    (
                        2,
                        LayoutInputOf::box_input(atomic.clone()),
                        atomic,
                        Vec::new(),
                    ),
                ],
            );
            assert_eq!(
                flow_axes
                    .logical_size(fri06_c02_final_node(&mixed, 0).size)
                    .block,
                S::from_f64(29.0),
                "mixed normal flow must not hide the taller owned float for {flow_axes:?}",
            );
        }

        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let root_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(80.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let nested_style = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(60.0)),
                PreferredSizeOf::AUTO,
            ),
            padding: Edges {
                top: LengthOf::px(S::from_f64(4.0)),
                bottom: LengthOf::px(S::from_f64(6.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            overflow: ComputedOverflow::try_new(Overflow::Hidden, Overflow::Hidden).unwrap(),
            ..NodeInputOf::default()
        };
        let nested_float = NodeInputOf {
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(1.0)),
                bottom: LengthAutoOf::px(S::from_f64(2.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..float_style(flow_axes, 20.0, 8.0, Clear::Both)
        };
        let paragraph = PreparedParagraph::plain(3, Source::new("X"), ahem_style(10.0, 10.0));
        let nested = prepared_front_door_batch(
            root_style,
            LogicalSizeOf::new(
                AvailableOf::definite(S::from_f64(80.0)),
                AvailableOf::MAX_CONTENT,
            ),
            vec![1, 3],
            vec![
                (
                    1,
                    LayoutInputOf::box_input(nested_style.clone()),
                    nested_style,
                    vec![2],
                ),
                (
                    2,
                    LayoutInputOf::box_input(nested_float.clone()),
                    nested_float,
                    Vec::new(),
                ),
                (
                    3,
                    paragraph.text_input(3),
                    NodeInputOf::non_box(),
                    Vec::new(),
                ),
            ],
            vec![paragraph.binding(0)],
        );
        let root = fri06_c02_final_node(&nested, 0);
        let nested_output = fri06_c02_final_node(&nested, 1);
        assert_eq!(nested_output.size.height, S::from_f64(21.0));
        assert_eq!(root.size.height, S::from_f64(31.0));
        assert_eq!(
            fri06_c02_final_node(&nested, 3).location,
            Point::new(S::ZERO, S::from_f64(21.0))
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_scroll_signed_fractional_geometry_and_source_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root_style = NodeInputOf {
            display: Display::Block,
            direction: Direction::Rtl,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(20.0)),
                PreferredSizeOf::px(S::from_f64(8.0)),
            ),
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            ..NodeInputOf::default()
        };
        let float = NodeInputOf {
            display: Display::Block,
            direction: Direction::Rtl,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(30.5)),
                PreferredSizeOf::px(S::from_f64(8.25)),
            ),
            margin: Edges {
                top: LengthAutoOf::px(S::from_f64(0.25)),
                bottom: LengthAutoOf::px(S::from_f64(0.25)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            ..NodeInputOf::default()
        };
        let paragraph = PreparedParagraph::new(
            vec![ParagraphPart::text(
                2,
                Source::new("X"),
                ahem_style(5.25, 10.0),
            )],
            ahem_style(5.25, 10.0),
            surgeist_text::ParagraphBaseDirection::RightToLeft,
        );
        let batch = prepared_front_door_batch(
            root_style,
            LogicalSizeOf::new(
                AvailableOf::definite(S::from_f64(20.0)),
                AvailableOf::definite(S::from_f64(8.0)),
            ),
            vec![1, 2],
            vec![
                (
                    1,
                    LayoutInputOf::box_input(float.clone()),
                    float,
                    Vec::new(),
                ),
                (
                    2,
                    paragraph.text_input(2),
                    NodeInputOf::non_box(),
                    Vec::new(),
                ),
            ],
            vec![paragraph.binding(0)],
        );
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(
                entries
                    .iter()
                    .map(LayoutOutputEntryOf::node)
                    .collect::<Vec<_>>(),
                vec![0, 1, 2],
            );
            assert_eq!(
                public_flow_output(entries, 1).source_index,
                SourceIndex::new(0)
            );
            assert_eq!(
                public_flow_output(entries, 2).source_index,
                SourceIndex::new(1)
            );
        }
        let unrounded_float = public_flow_output(batch.unrounded_entries(), 1);
        let rounded_float = public_flow_output(batch.final_entries(), 1);
        assert_eq!(
            unrounded_float.location,
            Point::new(S::from_f64(-10.5), S::from_f64(0.25))
        );
        assert_eq!(
            rounded_float.location,
            Point::new(S::from_f64(-10.0), S::ZERO)
        );
        let unrounded_root = public_flow_output(batch.unrounded_entries(), 0);
        let unrounded_geometry = unrounded_root.scroll_geometry.unwrap();
        assert_eq!(
            unrounded_geometry.scrollable_overflow().origin().x,
            S::from_f64(-10.5)
        );
        assert_eq!(
            unrounded_geometry.scrollable_overflow().size().width,
            S::from_f64(30.5)
        );
        assert_eq!(
            unrounded_geometry.physical_range().x().minimum(),
            S::from_f64(-10.5)
        );
        assert_eq!(unrounded_geometry.physical_range().x().maximum(), S::ZERO);
        let fragment = batch.unrounded_inline_fragments()[0].fragment();
        assert_eq!(fragment.line_index(), 0);
        let clearance = unrounded_float.location.y
            + unrounded_float.size.height
            + unrounded_float.margin.bottom;
        assert_eq!(clearance, S::from_f64(8.75));
        // One common canonical baseline16 minus actual Ahem body-before6.575.
        let body_before = 0.8 * 5.25 + (10.0 - 5.25) / 2.0;
        assert_eq!(fragment.baseline().y, S::from_f64(16.0));
        assert_eq!(
            fragment.rect().unwrap().origin().y,
            S::from_f64(16.0 - body_before)
        );
        assert!(fragment.rect().unwrap().origin().y >= clearance);
        let rounded_fragment = batch.final_inline_fragments()[0].fragment();
        assert_eq!(rounded_fragment.line_index(), fragment.line_index());
        assert_eq!(
            rounded_fragment.rect().unwrap().origin().y,
            S::from_f64(16.0 - body_before)
        );
        assert_eq!(rounded_fragment.baseline().y, S::from_f64(16.0));
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_text_atomic_source_gaps_and_repeat_keep_exact_occurrences_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let first = PreparedParagraph::at_boundary(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("X", Id::from_u64(101), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 3,
                    size: surgeist_text::Size::new(20.0, 6.0),
                },
                ParagraphPart::text(
                    5,
                    Source::identified("X", Id::from_u64(102), 0),
                    ahem_style(10.0, 10.0),
                ),
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
        );
        let after = PreparedParagraph::plain(
            7,
            Source::identified("X", Id::from_u64(103), 0),
            ahem_style(5.0, 10.0),
        );
        let mut binding = first.binding(0);
        binding.nodes = vec![1, 2, 3, 4, 5]; // hidden/absolute subjects keep original slots
        let hidden = NodeInputOf {
            display: Display::None,
            ..NodeInputOf::default()
        };
        let atomic = prepared_atomic_style(20.0, 6.0, 2.0, 3.0);
        let absolute = NodeInputOf {
            position: Position::Absolute,
            ..fri06_c03_atomic_style(7.0, 5.0, 0.0, 0.0, 0, InlineBreakOpportunity::Prohibited)
        };
        let floated = NodeInputOf {
            float: Float::Left,
            ..fri06_c03_atomic_style(8.0, 4.0, 0.0, 0.0, 0, InlineBreakOpportunity::Prohibited)
        };
        let inputs = HashMap::from([
            (
                0,
                LayoutInputOf::box_input(NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                }),
            ),
            (1, first.text_input(1)),
            (2, LayoutInputOf::box_input(hidden)),
            (3, LayoutInputOf::box_input(atomic)),
            (4, LayoutInputOf::box_input(absolute)),
            (5, first.text_input(5)),
            (6, LayoutInputOf::box_input(floated)),
            (7, after.text_input(7)),
        ]);
        let children = HashMap::from([
            (0, vec![1, 2, 3, 4, 5, 6, 7]),
            (1, vec![]),
            (2, vec![]),
            (3, vec![]),
            (4, vec![]),
            (5, vec![]),
            (6, vec![]),
            (7, vec![]),
        ]);
        let request = LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(S::from_f64(100.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap();
        let run = || {
            prepared_tree_batch(
                inputs.clone(),
                children.clone(),
                vec![binding.clone(), after.binding(0)],
                request,
            )
        };
        let one = run();
        let two = run();
        assert_eq!(one.final_entries(), two.final_entries());
        let fragments = super::fixtures::origin_fragment_associations(one.final_inline_fragments());
        assert_eq!(
            fragments,
            super::fixtures::origin_fragment_associations(two.final_inline_fragments())
        );
        assert_eq!(
            fragments.iter().map(|row| row.0).collect::<Vec<_>>(),
            [1, 5, 7]
        );
        for (index, (node, origin, x, y)) in [
            (1, first.origin(1), 0.0, 0.0),
            (5, first.origin(5), 35.0, 0.0),
            (7, after.origin(7), 8.0, 10.5),
        ]
        .into_iter()
        .enumerate()
        {
            assert_eq!(fragments[index].0, node);
            assert_eq!(fragments[index].1, origin);
            assert_eq!(fragments[index].2, surgeist_text::Range::new(0, 1));
            if node == 7 {
                assert_eq!(fragments[index].4.y, S::from_f64(17.0));
            }
            assert_eq!(
                fragments[index].3.unwrap().origin(),
                Point::new(S::from_f64(x), S::from_f64(y))
            );
        }
        assert_eq!(fri06_c02_final_node(&one, 2).size, Size::ZERO);
        assert_eq!(fri06_c02_final_node(&one, 3).location.x, S::from_f64(12.0));
        for (node, slot) in [(2, 1), (3, 2), (5, 4), (6, 5), (7, 6)] {
            assert_eq!(
                fri06_c02_final_node(&one, node).source_index,
                SourceIndex::new(slot)
            );
        }
        assert_eq!(
            one.final_entries()
                .iter()
                .filter(|entry| entry.node() == 3)
                .count(),
            1
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_atomic_break_opportunities_overwide_and_intrinsic_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let style = ahem_style(10.0, 10.0);
        let allowed = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X\u{200b}"), style.clone()),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(8.0, 10.0),
                },
                ParagraphPart::Atomic {
                    node: 3,
                    size: surgeist_text::Size::new(8.0, 10.0),
                },
                ParagraphPart::text(4, Source::new("XXX"), ahem_style(5.0, 10.0)),
            ],
            style.clone(),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let batch = prepared_mixed_batch(
            &allowed,
            vec![
                (1, allowed.text_input(1), NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(prepared_atomic_style(8.0, 10.0, 1.0, 1.0)),
                    NodeInputOf::non_box(),
                ),
                (
                    3,
                    LayoutInputOf::box_input(prepared_atomic_style(8.0, 10.0, 1.0, 1.0)),
                    NodeInputOf::non_box(),
                ),
                (4, allowed.text_input(4), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(30.0)),
            NodeInputOf::default(),
        );
        let fragments =
            super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
        let last = fragments
            .iter()
            .find(|row| row.0 == 4 && row.3.is_some())
            .unwrap();
        assert_eq!(last.5, 1);
        assert_eq!(
            last.3.unwrap().origin(),
            Point::new(S::ZERO, S::from_f64(13.5))
        );
        assert_eq!(last.4.y, S::from_f64(20.0));
        assert_eq!(
            batch.placed_text()[0].layout().lines()[1].bounds().origin.y,
            12.0
        );
        assert_eq!(fri06_c02_final_node(&batch, 3).location.y, S::ZERO);

        let mandatory = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X"), style.clone()),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
                ParagraphPart::Control {
                    node: 4,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(3, Source::new("X"), style.clone()),
            ],
            style.clone(),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let forced = prepared_mixed_batch(
            &mandatory,
            vec![
                (1, mandatory.text_input(1), NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(prepared_atomic_style(10.0, 10.0, 0.0, 0.0)),
                    NodeInputOf::non_box(),
                ),
                (
                    4,
                    LayoutInputOf::line_break(LineBreakInputOf::new()),
                    NodeInputOf::non_box(),
                ),
                (3, mandatory.text_input(3), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(100.0)),
            NodeInputOf::default(),
        );
        assert_eq!(fri06_c03_fragment(&forced, 1).line_index(), 0);
        assert_eq!(fri06_c03_fragment(&forced, 3).line_index(), 1);

        let overwide = PreparedParagraph::new(
            vec![
                ParagraphPart::Atomic {
                    node: 1,
                    size: surgeist_text::Size::new(40.0, 10.0),
                },
                ParagraphPart::Control {
                    node: 3,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(2, Source::new("X"), ahem_style(5.0, 10.0)),
            ],
            style.clone(),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let progress = prepared_mixed_batch(
            &overwide,
            vec![
                (
                    1,
                    LayoutInputOf::box_input(prepared_atomic_style(40.0, 10.0, 0.0, 0.0)),
                    NodeInputOf::non_box(),
                ),
                (
                    3,
                    LayoutInputOf::line_break(LineBreakInputOf::new()),
                    NodeInputOf::non_box(),
                ),
                (2, overwide.text_input(2), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(20.0)),
            NodeInputOf::default(),
        );
        assert_eq!(
            fri06_c02_final_node(&progress, 1).size.width,
            S::from_f64(40.0)
        );
        assert_eq!(fri06_c03_fragment(&progress, 2).line_index(), 1);
        let intrinsic = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X\u{200b}"), style.clone()),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(20.0, 10.0),
                },
                ParagraphPart::Control {
                    node: 4,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
                ParagraphPart::text(3, Source::new("XXX"), ahem_style(5.0, 10.0)),
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let run = |available: AvailableOf<S>| {
            prepared_mixed_batch(
                &intrinsic,
                vec![
                    (1, intrinsic.text_input(1), NodeInputOf::non_box()),
                    (
                        2,
                        LayoutInputOf::box_input(prepared_atomic_style(20.0, 10.0, 5.0, 5.0)),
                        NodeInputOf::non_box(),
                    ),
                    (
                        4,
                        LayoutInputOf::line_break(LineBreakInputOf::new()),
                        NodeInputOf::non_box(),
                    ),
                    (3, intrinsic.text_input(3), NodeInputOf::non_box()),
                ],
                available,
                NodeInputOf::default(),
            )
        };
        assert_eq!(
            fri06_c02_final_node(&run(AvailableOf::MIN_CONTENT), 0)
                .size
                .width,
            S::from_f64(30.0)
        );
        assert_eq!(
            fri06_c02_final_node(&run(AvailableOf::MAX_CONTENT), 0)
                .size
                .width,
            S::from_f64(40.0)
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_mixed_bidi_uses_original_directional_source_and_resets_each_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let mut collapse = ahem_style(5.0, 10.0);
        collapse.white_space = surgeist_text::WhiteSpace::Collapse;
        // Source RLO admits the actual odd text level; nested LRE supplies the
        // even box context. These are captured characters, not supplied levels.
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(
                    1,
                    Source::identified("\u{202e}X\u{202a}", Id::from_u64(301), 0),
                    ahem_style(10.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 2,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
                ParagraphPart::text(
                    3,
                    Source::identified("\u{202c} ", Id::from_u64(302), 0),
                    collapse,
                ),
                ParagraphPart::text(
                    4,
                    Source::identified("X\u{202a}", Id::from_u64(303), 0),
                    ahem_style(5.0, 10.0),
                ),
                ParagraphPart::Atomic {
                    node: 5,
                    size: surgeist_text::Size::new(10.0, 10.0),
                },
                ParagraphPart::text(
                    6,
                    Source::identified("\u{202c}X\u{202c}", Id::from_u64(304), 0),
                    ahem_style(5.0, 10.0),
                ),
            ],
            ahem_style(10.0, 10.0),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let batch = prepared_mixed_batch(
            &paragraph,
            vec![
                (1, paragraph.text_input(1), NodeInputOf::non_box()),
                (
                    2,
                    LayoutInputOf::box_input(prepared_atomic_style(10.0, 10.0, 0.0, 0.0)),
                    NodeInputOf::non_box(),
                ),
                (3, paragraph.text_input(3), NodeInputOf::non_box()),
                (4, paragraph.text_input(4), NodeInputOf::non_box()),
                (
                    5,
                    LayoutInputOf::box_input(prepared_atomic_style(10.0, 10.0, 0.0, 0.0)),
                    NodeInputOf::non_box(),
                ),
                (6, paragraph.text_input(6), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(25.0)),
            NodeInputOf::default(),
        );
        let rows = super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
        for (node, local, line, x, y, width) in [
            (1, surgeist_text::Range::new(1, 2), 0, 10.0, 2.0, 10.0),
            (4, surgeist_text::Range::new(0, 1), 1, 15.0, 17.5, 5.0),
            (6, surgeist_text::Range::new(1, 2), 1, 0.0, 17.5, 5.0),
        ] {
            let row = rows
                .iter()
                .find(|row| row.0 == node && row.2 == local)
                .unwrap();
            assert_eq!(row.1, paragraph.origin(node));
            assert_eq!(row.5, line);
            assert_eq!(
                row.3.unwrap().origin(),
                Point::new(S::from_f64(x), S::from_f64(y))
            );
            assert_eq!(row.3.unwrap().size().width, S::from_f64(width));
            let fragment = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| {
                    entry.owner() == &crate::InlineFragmentOwner::Node(node)
                        && entry.fragment().finalized_fragment().view().local_range() == Some(local)
                })
                .unwrap()
                .fragment();
            let metrics = fragment.finalized_fragment().finalized_line().feedback();
            assert_eq!(
                (metrics.baseline, metrics.descent, metrics.line_height),
                (10.0, 3.5, 13.5)
            );
            assert_eq!(
                fragment.baseline().y,
                S::from_f64(if line == 0 { 10.0 } else { 24.0 })
            );
        }
        // RLO/LRE/PDF and suppressed whitespace retain their actual source
        // intersections and metric anchors rather than masquerading as X rows.
        for (node, length, visible) in [
            (1, 3, Some(surgeist_text::Range::new(1, 2))),
            (3, 2, None),
            (4, 2, Some(surgeist_text::Range::new(0, 1))),
            (6, 3, Some(surgeist_text::Range::new(1, 2))),
        ] {
            let source_rows = rows.iter().filter(|row| row.0 == node).collect::<Vec<_>>();
            assert_eq!(source_rows.first().unwrap().2.start, 0);
            assert_eq!(source_rows.last().unwrap().2.end, length);
            assert!(
                source_rows
                    .windows(2)
                    .all(|pair| pair[0].2.end == pair[1].2.start)
            );
            for row in source_rows {
                assert_eq!(row.1, paragraph.origin(node));
                if Some(row.2) != visible {
                    assert_eq!(row.3.unwrap().size().width, S::ZERO);
                }
            }
        }
        assert_eq!(
            paragraph
                .prepared
                .unit_views()
                .find(|unit| unit.source_range() == surgeist_text::Range::new(1, 2))
                .unwrap()
                .level(),
            BidiLevel::try_new(1).unwrap()
        );
        for at in [3, 7] {
            assert_eq!(
                paragraph
                    .prepared
                    .atomic_views()
                    .find(|atomic| atomic.source_position().get() == at)
                    .unwrap()
                    .level(),
                BidiLevel::try_new(2).unwrap()
            );
        }
        let lines = batch.placed_text()[0].layout().lines();
        assert_eq!(
            lines.iter().map(|line| line.baseline()).collect::<Vec<_>>(),
            [10.0, 24.0]
        );
        assert_eq!(
            lines
                .iter()
                .map(|line| line.bounds().origin.y)
                .collect::<Vec<_>>(),
            [0.0, 14.0]
        );
        assert_eq!(fri06_c02_final_node(&batch, 2).location, Point::ZERO);
        assert_eq!(
            fri06_c02_final_node(&batch, 5).location,
            Point::new(S::from_f64(5.0), S::from_f64(14.0))
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_r1_bidi_range_four_variants_preserve_complete_unit_geometry() {
    fn assert_lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
                assert_fri06_c08_r1_mixed_unit_traversal::<S>(
                    FlowAxes::new(WritingMode::HorizontalTb, direction),
                    box_sizing,
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn numeric_rtl_visual_placement_and_internal_supplied_edge_semantics_all_flows_both_scalars() {
    use surgeist_text::{SuppliedInlineParticipant as Participant, SuppliedLine};
    for edge in [
        InlineWhitespaceEdge::DiscardAtBoth,
        InlineWhitespaceEdge::Preserve,
    ] {
        let supplied = [
            Participant::Atomic {
                level: BidiLevel::try_new(1).unwrap(),
            },
            Participant::Text {
                level: BidiLevel::try_new(1).unwrap(),
                edge,
            },
            Participant::Atomic {
                level: BidiLevel::try_new(1).unwrap(),
            },
        ];
        assert_eq!(
            SuppliedLine::new(&supplied).edges().is_discarded(1),
            Some(false),
            "a genuine interior source slot is not a line edge"
        );
        let mut order = [usize::MAX; 3];
        assert_eq!(
            SuppliedLine::new(&supplied)
                .reorder_into(&mut order)
                .unwrap()
                .iter()
                .map(|(index, _)| index)
                .collect::<Vec<_>>(),
            [2, 1, 0]
        );
    }
    fn assert_lane<S: LayoutScalar>() {
        for writing_mode in [
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            WritingMode::VerticalLr,
            WritingMode::SidewaysRl,
            WritingMode::SidewaysLr,
        ] {
            let axes = FlowAxes::new(writing_mode, Direction::Rtl);
            let report = super::fixtures::numeric_atomic_fit_with_flow(
                vec![
                    (10.0, 1, InlineBreakOpportunity::Prohibited),
                    (15.0, 1, InlineBreakOpportunity::Prohibited),
                    (20.0, 1, InlineBreakOpportunity::Prohibited),
                ],
                AvailableOf::definite(S::from_f64(100.0)),
                writing_mode,
                Direction::Rtl,
                TextAlign::Start,
            );
            assert_eq!(
                [
                    report.atomics[0].inline_start,
                    report.atomics[2].inline_start
                ],
                [S::from_f64(35.0), S::ZERO]
            );
            let containing =
                axes.physical_size(LogicalSizeOf::new(S::from_f64(100.0), S::from_f64(160.0)));
            for row in &report.atomics {
                let projected = axes.physical_rect(
                    crate::geometry::LogicalRectOf::new(
                        LogicalPointOf::new(row.inline_start, row.block_start),
                        axes.logical_size(row.item.size),
                    ),
                    containing,
                );
                assert_eq!(
                    axes.logical_point(projected.0, projected.1, containing)
                        .inline,
                    row.inline_start
                );
            }
        }
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_mixed_inline_rtl_border_box_projects_reordered_slice_once() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Rtl, BoxSizing::BorderBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Rtl, BoxSizing::BorderBox, 99.0);
}

#[test]
fn fri06_c08_mixed_inline_rtl_content_box_projects_reordered_slice_once() {
    assert_fri06_c08_mixed_inline_atomic_x::<f32>(Direction::Rtl, BoxSizing::ContentBox, 99.0);
    assert_fri06_c08_mixed_inline_atomic_x::<f64>(Direction::Rtl, BoxSizing::ContentBox, 99.0);
}

fn fri06_c08_float_line_control_batch<S: LayoutScalar>(
    in_flow: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    right_float_block_extent: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    fri06_c08_float_line_control_capture(in_flow, right_float_block_extent, None)
}

fn fri06_c08_native_atomic<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        // The actual PreparedAtomic owns bidi and opportunity facts.
        atomic_inline_participation: None,
        ..NodeInputOf::default()
    }
}

fn fri06_c08_float_line_control_capture<S: LayoutScalar>(
    mut in_flow: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    right_float_block_extent: f64,
    paragraph: Option<&PreparedParagraph>,
) -> CompletedLayoutBatchOf<u32, S> {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let left_float = fri06_c04_line_box(
        flow_axes,
        LogicalSizeOf::new(S::from_f64(42.0), S::from_f64(42.0)),
        Float::Left,
        None,
    );
    let right_float = fri06_c04_line_box(
        flow_axes,
        LogicalSizeOf::new(S::from_f64(50.0), S::from_f64(right_float_block_extent)),
        Float::Right,
        None,
    );
    let mut children = vec![
        (1, LayoutInputOf::box_input(left_float.clone()), left_float),
        (
            2,
            LayoutInputOf::box_input(right_float.clone()),
            right_float,
        ),
    ];
    children.append(&mut in_flow);
    let root = NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(180.0)),
            PreferredSizeOf::AUTO,
        ),
        ..NodeInputOf::default()
    };
    if let Some(paragraph) = paragraph {
        super::fixtures::prepared_mixed_batch(
            paragraph,
            children,
            AvailableOf::definite(S::from_f64(180.0)),
            root,
        )
    } else {
        fri06_c03_mixed_batch_with_root(children, AvailableOf::definite(S::from_f64(180.0)), root)
    }
}

fn assert_fri06_c08_float_line_control_height<S: LayoutScalar>(
    in_flow: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    paragraph: Option<&PreparedParagraph>,
) {
    let batch = if let Some(paragraph) = paragraph {
        fri06_c08_float_line_control_capture(in_flow, 62.0, Some(paragraph))
    } else {
        fri06_c08_float_line_control_batch(in_flow, 62.0)
    };
    assert_eq!(
        public_flow_output(batch.final_entries(), 0).size,
        Size::new(S::from_f64(180.0), S::from_f64(62.0))
    );
}

fn fri06_c08_float_line_continuation_batch<S: LayoutScalar>(
    line_height: f64,
    right_float_block_extent: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    // Ahem's 16px cell plus authored 4px letter spacing gives two real 20px
    // clusters. Ahem ascent/descent 12.8/3.2 and 20px line height give 14.8/5.2.
    let style = surgeist_text::Style {
        letter_spacing: 4.0,
        ..ahem_style(16.0, 20.0)
    };
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(3, Source::new("XX"), style.clone()),
            ParagraphPart::Control {
                node: 8,
                kind: surgeist_text::ParagraphControlKind::InlineBoundaryStart,
            },
            ParagraphPart::Atomic {
                node: 4,
                size: surgeist_text::Size::new(28.0, 16.0),
            },
            ParagraphPart::Atomic {
                node: 5,
                size: surgeist_text::Size::new(32.0, 16.0),
            },
            ParagraphPart::Atomic {
                node: 6,
                size: surgeist_text::Size::new(36.0, 16.0),
            },
            ParagraphPart::Atomic {
                node: 7,
                size: surgeist_text::Size::new(40.0, 16.0),
            },
        ],
        style,
        surgeist_text::ParagraphBaseDirection::LeftToRight,
    );
    let strut = InlineBoundaryInputOf::new(
        InlineBoundaryKind::Start,
        InlineMetricsOf::from_line_height_and_baseline(S::from_f64(line_height), S::from_f64(12.0))
            .unwrap(),
    );
    let mut children = vec![
        (3, paragraph.text_input(3), NodeInputOf::non_box()),
        (
            8,
            LayoutInputOf::inline_boundary(strut),
            NodeInputOf::non_box(),
        ),
    ];
    for (node, width) in [(4, 28.0), (5, 32.0), (6, 36.0), (7, 40.0)] {
        let atomic = fri06_c08_native_atomic(width, 16.0);
        children.push((node, LayoutInputOf::box_input(atomic.clone()), atomic));
    }
    fri06_c08_float_line_control_capture(children, right_float_block_extent, Some(&paragraph))
}

#[test]
fn fri06_c08_float_line_actual_bidi_continuation_uses_visual_placement() {
    fn assert_lane<S: LayoutScalar>() {
        // Actual Hebrew source resolves the RTL line. The real Ahem cell plus
        // authored letter spacing is 40px and the atomic border is 28px.
        let style = surgeist_text::Style {
            letter_spacing: 24.0,
            ..ahem_style(16.0, 20.0)
        };
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(3, Source::new("א"), style.clone()),
                ParagraphPart::Atomic {
                    node: 4,
                    size: surgeist_text::Size::new(28.0, 16.0),
                },
            ],
            style,
            surgeist_text::ParagraphBaseDirection::RightToLeft,
        );
        let atomic = fri06_c08_native_atomic(28.0, 16.0);
        let left = NodeInputOf {
            display: Display::Block,
            float: Float::Right,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(42.0)),
                PreferredSizeOf::px(S::from_f64(42.0)),
            ),
            ..NodeInputOf::default()
        };
        let right = NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(50.0)),
                PreferredSizeOf::px(S::from_f64(62.0)),
            ),
            ..NodeInputOf::default()
        };
        let batch = super::fixtures::prepared_mixed_batch(
            &paragraph,
            vec![
                (1, LayoutInputOf::box_input(left.clone()), left),
                (2, LayoutInputOf::box_input(right.clone()), right),
                (3, paragraph.text_input(3), NodeInputOf::non_box()),
                (4, LayoutInputOf::box_input(atomic.clone()), atomic),
            ],
            AvailableOf::definite(S::from_f64(180.0)),
            NodeInputOf {
                display: Display::Block,
                direction: Direction::Rtl,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(180.0)),
                    PreferredSizeOf::AUTO,
                ),
                ..NodeInputOf::default()
            },
        );
        let rows = super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].1, paragraph.origin(3));
        let text = rows[0].3.unwrap();
        let atomic = public_flow_output(batch.final_entries(), 4);
        assert_eq!(text.size().width, S::from_f64(40.0));
        assert_eq!(atomic.size.width, S::from_f64(28.0));
        assert_eq!(text.origin().x, atomic.location.x + atomic.size.width);
        assert_eq!(text.origin().x + text.size().width, S::from_f64(130.0));
        assert_eq!(
            public_flow_output(batch.final_entries(), 0).size.height,
            S::from_f64(62.0)
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_fractional_terminal_extent_uses_resolved_geometry() {
    fn assert_lane<S: LayoutScalar>() {
        // Four authored forced lines allocate4*20.25=81px beyond both floats.
        // Ahem16's before12.8 plus half-leading2.125 gives14.925. Rounding
        // each physical baseline preserves separate fractional metric bounds;
        // those rigid translations do not advance the next logical line.
        let paragraph =
            PreparedParagraph::plain(3, Source::new("X\nX\nX\nX"), ahem_style(16.0, 20.25));
        let batch = fri06_c08_float_line_control_capture::<S>(
            vec![(3, paragraph.text_input(3), NodeInputOf::non_box())],
            62.0,
            Some(&paragraph),
        );
        let layout = batch.placed_text()[0].layout();
        let lines = layout.lines();
        assert_eq!(lines.len(), 4);
        for (line, baseline) in lines.iter().zip([15.0, 35.0, 55.0, 76.0]) {
            assert!((line.bounds().size.height - 20.25).abs() < 0.000_001);
            assert_eq!(line.baseline(), baseline);
            assert!((line.bounds().origin.y - (baseline - 14.925)).abs() < 0.000_1);
        }
        let terminal = lines.last().unwrap().bounds();
        let resolved_bottom = terminal.origin.y + terminal.size.height;
        assert!((resolved_bottom - 81.325).abs() < 0.000_1);
        let actual = public_flow_output(batch.unrounded_entries(), 0).size.height;
        assert_eq!(actual, S::from_f64(4.0 * 20.25));
        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(81.0)
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_pure_text_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let paragraph = PreparedParagraph::plain(3, Source::new("XX"), ahem_style(20.0, 20.1));
        assert_fri06_c08_float_line_control_height::<S>(
            vec![(3, paragraph.text_input(3), NodeInputOf::non_box())],
            Some(&paragraph),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_all_atomic_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let atomic =
            fri06_c03_atomic_style(40.0, 20.2, 0.0, 0.0, 0, InlineBreakOpportunity::Prohibited);
        assert_fri06_c08_float_line_control_height::<S>(
            vec![(3, LayoutInputOf::box_input(atomic.clone()), atomic)],
            None,
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_empty_inline_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let style = ahem_style(1.0, 10.0);
        let paragraph = PreparedParagraph::new(
            vec![
                ParagraphPart::text(3, Source::new("X"), style.clone()),
                ParagraphPart::Control {
                    node: 4,
                    kind: surgeist_text::ParagraphControlKind::ForcedBreak,
                },
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let metrics =
            InlineMetricsOf::from_ascent_descent(S::from_f64(7.8), S::from_f64(2.3)).unwrap();
        assert_fri06_c08_float_line_control_height::<S>(
            vec![
                (3, paragraph.text_input(3), NodeInputOf::non_box()),
                (
                    4,
                    LayoutInputOf::line_break(LineBreakInputOf::new().with_metrics(metrics)),
                    NodeInputOf::non_box(),
                ),
            ],
            Some(&paragraph),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c08_float_line_nonterminal_mixed_inline_keeps_integral_float_height_at_62() {
    fn assert_lane<S: LayoutScalar>() {
        let style = surgeist_text::Style {
            letter_spacing: 4.0,
            ..ahem_style(16.0, 20.0)
        };
        let paragraph = PreparedParagraph::at_boundary(
            vec![
                ParagraphPart::text(3, Source::new("XX"), style.clone()),
                ParagraphPart::Atomic {
                    node: 4,
                    size: surgeist_text::Size::new(28.0, 16.0),
                },
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            surgeist_text::ParagraphTerminalEdge::NonBlockBoundary,
        );
        let atomic = fri06_c08_native_atomic(28.0, 16.0);
        let terminal = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(10.0)),
                PreferredSizeOf::px(S::from_f64(1.0)),
            ),
            ..NodeInputOf::default()
        };
        assert_fri06_c08_float_line_control_height::<S>(
            vec![
                (3, paragraph.text_input(3), NodeInputOf::non_box()),
                (4, LayoutInputOf::box_input(atomic.clone()), atomic),
                (5, LayoutInputOf::box_input(terminal.clone()), terminal),
            ],
            Some(&paragraph),
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_horizontal_forced_break_fallback_expands_each_line_envelope() {
    fn assert_lane<S: LayoutScalar>() {
        let batch = fri06_c12_t08_forced_break_fallback_batch::<S>(15.0);

        for (parent_node, first_atomic, second_atomic, block_start) in
            [(1, 2, 4, 0.0), (5, 6, 8, 42.0), (9, 10, 12, 84.0)]
        {
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), parent_node)
                    .size
                    .height,
                S::from_f64(42.0),
                "each completed parent publishes two exact 21px line envelopes",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), parent_node)
                    .location
                    .y,
                S::from_f64(block_start),
                "parent block progression consumes the exact completed envelope",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), first_atomic)
                    .location
                    .y,
                S::ZERO,
                "the first atomic starts at the actual completed line envelope",
            );
            assert_eq!(
                public_flow_output(batch.unrounded_entries(), second_atomic)
                    .location
                    .y,
                S::from_f64(21.0),
                "the second fallback envelope starts after exactly one completed line",
            );
        }
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 0).size.height,
            S::from_f64(126.0),
        );
        assert_eq!(
            fri06_c02_final_node(&batch, 0).size.height,
            S::from_f64(126.0),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_ltr_post_exclusion_continuation_restarts_at_line_start() {
    fn assert_lane<S: LayoutScalar>() {
        // This real 28px allocated control has 16px below its 12px baseline.
        // Combined with the 16px baseline atomic it makes the first line at
        // least 32px. The next 16px atomic line ends beyond the 42px left float.
        let batch = fri06_c08_float_line_continuation_batch::<S>(28.0, 62.0);
        let terminal = public_flow_output(batch.unrounded_entries(), 7);
        assert_eq!(terminal.location.x, S::ZERO);
        assert!(terminal.location.y >= S::from_f64(42.0));
        assert_eq!(
            public_flow_output(batch.unrounded_entries(), 6).location.x,
            S::from_f64(74.0),
            "the preceding line still intersects the actual 42px left float"
        );
        assert_eq!(
            terminal.size,
            Size::new(S::from_f64(40.0), S::from_f64(16.0))
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_terminal_line_phase_survives_both_inline_progressions() {
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Ltr, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f32>(Direction::Rtl, BoxSizing::BorderBox);
    assert_fri06_c08_float_line_final_height::<f64>(Direction::Rtl, BoxSizing::BorderBox);
}

fn fri06_c08_recovery_characterization_batch<S: LayoutScalar>(
    children: Vec<(u32, LayoutInputOf<S>, NodeInputOf<S>)>,
    root_input: NodeInputOf<S>,
) -> CompletedLayoutBatchOf<u32, S> {
    let child_nodes = children
        .iter()
        .map(|(node, _, _)| *node)
        .collect::<Vec<_>>();
    let mut inputs = HashMap::from([(0, LayoutInputOf::box_input(root_input.clone()))]);
    let mut tree_children = HashMap::from([(0, child_nodes)]);
    for (node, layout_input, _node_input) in children {
        inputs.insert(node, layout_input);
        tree_children.insert(node, Vec::new());
    }
    let tree = public_layout_tree(inputs, tree_children);
    compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}

#[test]
fn fri06_c08_recovery_characterization_exact_public_inputs_cover_both_scalar_lanes() {
    fn assert_percentage_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            // Three Ahem cells with authored 3.265625px spacing: 57.796875.
            // Four cells with 5.67578125px spacing: 86.703125. Strong Hebrew
            // source establishes actual level one instead of supplied glyph levels.
            let first_style = surgeist_text::Style {
                letter_spacing: 3.265625,
                ..ahem_style(16.0, 20.0)
            };
            let last_style = surgeist_text::Style {
                letter_spacing: 5.67578125,
                ..ahem_style(16.0, 20.0)
            };
            let paragraph = PreparedParagraph::new(
                vec![
                    ParagraphPart::text(1, Source::new("אבג"), first_style.clone()),
                    ParagraphPart::Atomic {
                        node: 2,
                        size: surgeist_text::Size::new(20.0, 40.0),
                    },
                    ParagraphPart::text(3, Source::new("אבגד"), last_style),
                ],
                first_style,
                surgeist_text::ParagraphBaseDirection::RightToLeft,
            );
            let atomic = NodeInputOf {
                display: Display::InlineBlock,
                direction: Direction::Rtl,
                box_sizing,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(20.0)),
                    PreferredSizeOf::percent(S::from_f64(0.5)),
                ),
                atomic_inline_participation: None,
                ..NodeInputOf::default()
            };
            let root = NodeInputOf {
                display: Display::Block,
                direction: Direction::Rtl,
                box_sizing,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(180.0)),
                    PreferredSizeOf::px(S::from_f64(80.0)),
                ),
                ..NodeInputOf::default()
            };
            let batch = super::fixtures::prepared_mixed_batch(
                &paragraph,
                vec![
                    (1, paragraph.text_input(1), NodeInputOf::non_box()),
                    (2, LayoutInputOf::box_input(atomic.clone()), atomic),
                    (3, paragraph.text_input(3), NodeInputOf::non_box()),
                ],
                AvailableOf::MAX_CONTENT,
                root,
            );
            assert_eq!(
                fri06_c02_final_node(&batch, 0).size,
                Size::new(S::from_f64(180.0), S::from_f64(80.0))
            );
            let atomic = fri06_c02_final_node(&batch, 2);
            assert_eq!(atomic.source_index, SourceIndex::new(1));
            // L2 puts the last Hebrew origin first, then the atomic, then the
            // first origin. Start alignment uses 180 - (57.796875+20+86.703125).
            assert_eq!(
                (atomic.location, atomic.size, atomic.content_size),
                (
                    Point::new(S::from_f64(102.0), S::ZERO),
                    Size::new(S::from_f64(20.0), S::from_f64(40.0)),
                    Size::new(S::from_f64(20.0), S::from_f64(40.0))
                )
            );
            for (node, x, width) in [(1, 122.203125, 57.796875), (3, 15.5, 86.703125)] {
                let output = public_flow_output(batch.final_entries(), node);
                assert_eq!(output.location.x, S::from_f64(x));
                assert_eq!(output.size.width, S::from_f64(width));
                let rows =
                    super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
                let rows: Vec<_> = rows.iter().filter(|row| row.0 == node).collect();
                assert!(!rows.is_empty());
                assert!(
                    rows.iter()
                        .all(|row| row.1 == paragraph.origin(node) && row.5 == 0)
                );
                assert!(rows.iter().all(|row| row.4.y == S::from_f64(40.0)));
                assert!(rows.iter().all(|row| row.3.is_some()));
            }
            assert_eq!(
                batch.placed_text()[0].layout().source().text().code_units(),
                Source::new("אבגאבגד").text().code_units()
            );
        }
    }

    fn assert_vertical_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let axes = FlowAxes::new(WritingMode::VerticalRl, direction);
                let bidi_level = u8::from(direction == Direction::Rtl);
                let box_input = |display, width, height, float, clear, participation| NodeInputOf {
                    display,
                    writing_mode: WritingMode::VerticalRl,
                    direction,
                    box_sizing,
                    float,
                    clear,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::px(S::from_f64(height)),
                    ),
                    atomic_inline_participation: participation,
                    ..NodeInputOf::default()
                };
                let participation = || {
                    Some(fri06_c03_atomic_participation(
                        bidi_level,
                        InlineBreakOpportunity::Prohibited,
                    ))
                };
                let floating =
                    box_input(Display::Block, 28.0, 36.0, Float::Left, Clear::None, None);
                let first = box_input(
                    Display::InlineBlock,
                    18.0,
                    42.0,
                    Float::None,
                    Clear::None,
                    participation(),
                );
                let second = box_input(
                    Display::InlineBlock,
                    18.0,
                    30.0,
                    Float::None,
                    Clear::None,
                    participation(),
                );
                let cleared = box_input(Display::Block, 18.0, 18.0, Float::None, Clear::Left, None);
                let line_break = LineBreakInputOf::new()
                    .with_writing_mode(WritingMode::VerticalRl)
                    .with_direction(direction)
                    .with_metrics(
                        InlineMetricsOf::from_line_height_and_baseline(
                            S::from_f64(24.0),
                            S::from_f64(16.8),
                        )
                        .unwrap(),
                    );
                let root = NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    direction,
                    box_sizing,
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_width: ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap(),
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(96.0)),
                        PreferredSizeOf::px(S::from_f64(140.0)),
                    ),
                    ..NodeInputOf::default()
                };
                let batch = fri06_c08_recovery_characterization_batch(
                    vec![
                        (1, LayoutInputOf::box_input(floating.clone()), floating),
                        (2, LayoutInputOf::box_input(first.clone()), first),
                        (
                            3,
                            LayoutInputOf::line_break(line_break),
                            NodeInputOf::non_box(),
                        ),
                        (4, LayoutInputOf::box_input(second.clone()), second),
                        (5, LayoutInputOf::box_input(cleared.clone()), cleared),
                    ],
                    root,
                );
                let (float_y, first_y, second_y, clear_y) = match direction {
                    Direction::Ltr => (0.0, 36.0, 36.0, 0.0),
                    Direction::Rtl => (104.0, 62.0, 74.0, 122.0),
                };
                assert_eq!(
                    fri06_c02_final_node(&batch, 0).size,
                    Size::new(S::from_f64(96.0), S::from_f64(140.0))
                );
                let mut mismatches = Vec::new();
                for (node, x, y, width, height) in [
                    (1, 68.0, float_y, 28.0, 36.0),
                    (2, 75.0, first_y, 18.0, 42.0),
                    (4, 51.0, second_y, 18.0, 30.0),
                    (5, 30.0, clear_y, 18.0, 18.0),
                ] {
                    let output = fri06_c02_final_node(&batch, node);
                    let expected = (
                        Point::new(S::from_f64(x), S::from_f64(y)),
                        Size::new(S::from_f64(width), S::from_f64(height)),
                    );
                    if (output.location, output.size) != expected {
                        mismatches.push(format!(
                            "source {node}: expected {expected:?}, got {:?}",
                            (output.location, output.size)
                        ));
                    }
                }
                let control = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 3)
                    .expect("forced break publishes unrounded geometry")
                    .output();
                assert_eq!(control.size, Size::ZERO);
                let control_block = axes
                    .logical_point(
                        control.location,
                        control.size,
                        Size::new(S::from_f64(96.0), S::from_f64(140.0)),
                    )
                    .block;
                if (control_block - S::from_f64(16.8)).abs() > S::from_f64(0.000_1) {
                    mismatches.push(format!(
                        "forced-break metric baseline: expected 16.8, got {control_block:?}"
                    ));
                }
                let clear_block = batch
                    .unrounded_entries()
                    .iter()
                    .find(|entry| entry.node() == 5)
                    .map(|entry| {
                        let output = entry.output();
                        axes.logical_point(
                            output.location,
                            output.size,
                            Size::new(S::from_f64(96.0), S::from_f64(140.0)),
                        )
                        .block
                    })
                    .expect("cleared block publishes unrounded geometry");
                if clear_block != S::from_f64(48.0) {
                    mismatches.push(format!(
                        "two resolved 24px line bands: expected 48, got {clear_block:?}"
                    ));
                }
                assert!(
                    mismatches.is_empty(),
                    "{axes:?} {box_sizing:?} vertical line-band mismatches:\n{}",
                    mismatches.join("\n")
                );
            }
        }
    }

    fn assert_float_lane<S: LayoutScalar>() {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
                let floating = |physical_left: bool, width, height| {
                    let side = match (direction, physical_left) {
                        (Direction::Ltr, true) | (Direction::Rtl, false) => Float::Left,
                        _ => Float::Right,
                    };
                    let mut input = fri06_c04_line_box(
                        axes,
                        LogicalSizeOf::new(S::from_f64(width), S::from_f64(height)),
                        side,
                        None,
                    );
                    input.box_sizing = box_sizing;
                    input
                };
                let atomic = |width| {
                    let mut input = fri06_c04_line_box(
                        axes,
                        LogicalSizeOf::new(S::from_f64(width), S::from_f64(16.0)),
                        Float::None,
                        None,
                    );
                    input.box_sizing = box_sizing;
                    input
                };
                let left = floating(true, 42.0, 42.0);
                let right = floating(false, 50.0, 62.0);
                let atomics = [atomic(28.0), atomic(32.0), atomic(36.0), atomic(40.0)];
                // One genuine 16px Ahem cell plus exact authored spacing.
                let style = surgeist_text::Style {
                    letter_spacing: 22.53125,
                    ..ahem_style(16.0, 20.0)
                };
                let paragraph = PreparedParagraph::new(
                    vec![
                        ParagraphPart::text(
                            3,
                            Source::new(if direction == Direction::Rtl {
                                "א"
                            } else {
                                "X"
                            }),
                            style.clone(),
                        ),
                        ParagraphPart::Atomic {
                            node: 4,
                            size: surgeist_text::Size::new(28.0, 16.0),
                        },
                        ParagraphPart::Atomic {
                            node: 5,
                            size: surgeist_text::Size::new(32.0, 16.0),
                        },
                        ParagraphPart::Atomic {
                            node: 6,
                            size: surgeist_text::Size::new(36.0, 16.0),
                        },
                        ParagraphPart::Atomic {
                            node: 7,
                            size: surgeist_text::Size::new(40.0, 16.0),
                        },
                    ],
                    style,
                    if direction == Direction::Rtl {
                        surgeist_text::ParagraphBaseDirection::RightToLeft
                    } else {
                        surgeist_text::ParagraphBaseDirection::LeftToRight
                    },
                );
                let mut children = vec![
                    (1, LayoutInputOf::box_input(left.clone()), left),
                    (2, LayoutInputOf::box_input(right.clone()), right),
                    (3, paragraph.text_input(3), NodeInputOf::non_box()),
                ];
                for (index, input) in atomics.into_iter().enumerate() {
                    children.push((
                        4 + index as u32,
                        LayoutInputOf::box_input(input.clone()),
                        input,
                    ));
                }
                let batch = super::fixtures::prepared_mixed_batch(
                    &paragraph,
                    children,
                    AvailableOf::MAX_CONTENT,
                    NodeInputOf {
                        display: Display::Block,
                        direction,
                        box_sizing,
                        size: Size::new(
                            PreferredSizeOf::px(S::from_f64(180.0)),
                            PreferredSizeOf::AUTO,
                        ),
                        ..NodeInputOf::default()
                    },
                );
                // Ahem16/LH20 contributes after5.2; each baseline-aligned
                // atomic contributes before16, giving three21.2px lines.
                // With no clear minimum, nearest common baselines16/37/58
                // place their actual metric tops at0/21/42 and bottom63.2.
                // Logical starts remain0/21.2/42.4, so three21.2px allocations
                // end at63.6, beyond both float bottoms42/62, and round to64.
                let lines = batch.placed_text()[0].layout().lines();
                assert_eq!(lines.len(), 3);
                for (line, (top, baseline)) in
                    lines.iter().zip([(0.0, 16.0), (21.0, 37.0), (42.0, 58.0)])
                {
                    assert!((line.bounds().origin.y - top).abs() < 0.0001);
                    assert_eq!(line.baseline(), baseline);
                    assert!((line.bounds().size.height - 21.2).abs() < 0.0001);
                }
                let terminal_metrics = lines.last().unwrap().bounds();
                assert!(
                    (terminal_metrics.origin.y + terminal_metrics.size.height - 63.2).abs()
                        < 0.0001
                );
                // Match normal-flow's repeated f64 addition before the one
                // selected-scalar conversion, rather than rounding to63.6.
                let logical_extent = [21.2_f64; 3].into_iter().sum::<f64>();
                assert_eq!(
                    public_flow_output(batch.unrounded_entries(), 0).size,
                    Size::new(S::from_f64(180.0), S::from_f64(logical_extent))
                );
                assert_eq!(
                    fri06_c02_final_node(&batch, 0).size,
                    Size::new(S::from_f64(180.0), S::from_f64(64.0))
                );
                for (node, x, width, height) in [(1, 0.0, 42.0, 42.0), (2, 130.0, 50.0, 62.0)] {
                    let output = fri06_c02_final_node(&batch, node);
                    assert_eq!(
                        (output.location, output.size),
                        (
                            Point::new(S::from_f64(x), S::ZERO),
                            Size::new(S::from_f64(width), S::from_f64(height))
                        )
                    );
                }
                let rows =
                    super::fixtures::origin_fragment_associations(batch.final_inline_fragments());
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0].1, paragraph.origin(3));
                assert_eq!(rows[0].5, 0);
                let text = rows[0].3.unwrap();
                assert_eq!(text.size().width, S::from_f64(38.53125));
                let first = fri06_c02_final_node(&batch, 4);
                assert_eq!(
                    first.location.x,
                    S::from_f64(if direction == Direction::Rtl {
                        63.0
                    } else {
                        81.0
                    })
                );
                // Every allocated atomic preserves its real source position and size;
                // text's actual height/float feedback, not nominal descriptors,
                // determines which candidate and line receives it.
                for (index, width) in [28.0, 32.0, 36.0, 40.0].into_iter().enumerate() {
                    let node = 4 + index as u32;
                    let output = fri06_c02_final_node(&batch, node);
                    assert_eq!(output.source_index, SourceIndex::new(3 + index));
                    assert_eq!(
                        output.size,
                        Size::new(S::from_f64(width), S::from_f64(16.0))
                    );
                }
                assert_eq!(rows[0].4.y, S::from_f64(16.0));
            }
        }
    }

    assert_percentage_lane::<f32>();
    assert_percentage_lane::<f64>();
    assert_vertical_lane::<f32>();
    assert_vertical_lane::<f64>();
    assert_float_lane::<f32>();
    assert_float_lane::<f64>();
}

#[test]
fn fri06_c07_sideways_lr_rtl_all_atomic_odd_bidi_keeps_visual_placement_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::SidewaysLr, Direction::Rtl);
        let atomic = |node, inline_extent| {
            let style = fri06_c04_line_box(
                flow_axes,
                LogicalSizeOf::new(S::from_f64(inline_extent), S::from_f64(20.0)),
                Float::None,
                Some(fri06_c03_atomic_participation(
                    1,
                    InlineBreakOpportunity::Prohibited,
                )),
            );
            (node, LayoutInputOf::box_input(style.clone()), style)
        };
        let batch = fri06_c04_line_batch(
            flow_axes,
            TextAlign::Start,
            vec![atomic(1, 10.0), atomic(2, 20.0)],
        );
        let first_source = fri06_c02_final_node(&batch, 1);
        let second_source = fri06_c02_final_node(&batch, 2);

        assert_eq!(
            first_source.size,
            Size::new(S::from_f64(20.0), S::from_f64(10.0))
        );
        assert_eq!(
            second_source.size,
            Size::new(S::from_f64(20.0), S::from_f64(20.0))
        );
        assert_eq!(first_source.location.y, S::from_f64(20.0));
        assert_eq!(second_source.location.y, S::ZERO);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn supplied_line_bidi_and_actual_atomic_geometry_preserve_nested_levels_and_slot_gaps_both_scalars()
{
    use surgeist_text::{SuppliedInlineParticipant as Participant, SuppliedLine};
    let participants = [0, 1, 2, 2, 1, 0].map(|level| Participant::Text {
        level: BidiLevel::try_new(level).unwrap(),
        edge: InlineWhitespaceEdge::Preserve,
    });
    let mut scratch = [usize::MAX; 6];
    let visual = SuppliedLine::new(&participants)
        .reorder_into(&mut scratch)
        .unwrap();
    assert_eq!(
        visual.iter().map(|(index, _)| index).collect::<Vec<_>>(),
        [0, 4, 2, 3, 1, 5]
    );
    let discarded = [
        InlineWhitespaceEdge::DiscardAtLineStart,
        InlineWhitespaceEdge::Preserve,
        InlineWhitespaceEdge::DiscardAtLineEnd,
    ]
    .map(|edge| Participant::Text {
        level: BidiLevel::try_new(0).unwrap(),
        edge,
    });
    let edges = SuppliedLine::new(&discarded).edges();
    assert_eq!(
        (0..3)
            .filter(|index| edges.is_discarded(*index) == Some(false))
            .collect::<Vec<_>>(),
        [1]
    );
    // The actual box fitter retains the same genuine L2 producer/geometry join;
    // this is not an obsolete text-shaped input to a canonical tree.
    fn assert_lane<S: LayoutScalar>() {
        let report = super::fixtures::numeric_atomic_fit_with_flow(
            [0, 1, 2, 2, 1, 0]
                .map(|level| (10.0, level, InlineBreakOpportunity::Prohibited))
                .to_vec(),
            AvailableOf::definite(S::from_f64(100.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
        );
        assert_eq!(
            report
                .atomics
                .iter()
                .map(|a| a.visual_index)
                .collect::<Vec<_>>(),
            [Some(0), Some(4), Some(2), Some(3), Some(1), Some(5)]
        );
        assert_eq!(
            report
                .atomics
                .iter()
                .map(|a| a.inline_start)
                .collect::<Vec<_>>(),
            [0.0, 40.0, 20.0, 30.0, 10.0, 50.0].map(S::from_f64)
        );
        let wrapped = super::fixtures::numeric_atomic_fit_with_flow(
            (0..4)
                .map(|i| {
                    (
                        10.0,
                        1,
                        if i == 1 {
                            InlineBreakOpportunity::Mandatory
                        } else {
                            InlineBreakOpportunity::Prohibited
                        },
                    )
                })
                .collect(),
            AvailableOf::definite(S::from_f64(100.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
        );
        assert_eq!(
            wrapped
                .atomics
                .iter()
                .map(|a| (a.line_index, a.visual_index, a.inline_start, a.block_start))
                .collect::<Vec<_>>(),
            [
                (0, 1, 10.0, 0.0),
                (0, 0, 0.0, 0.0),
                (1, 1, 10.0, 10.0),
                (1, 0, 0.0, 10.0)
            ]
            .map(|(l, v, x, y)| (l, Some(v), S::from_f64(x), S::from_f64(y)))
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn prepared_text_latest_replacement_mandatory_final_and_overwide_progress_both_scalars() {
    fn replacement() -> ParagraphReplacement {
        ParagraphReplacement::resolved_native_logical_before(
            ParagraphReplacementDefinition::Source {
                origin: 0,
                source: surgeist_text::Range::new(3, 4),
            },
            4,
            vec![Text::from("-")],
        )
        .unwrap()
    }
    fn assert_lane<S: LayoutScalar>() {
        // Seven authored Xs at5px plus a source-defined soft hyphen. The selected
        // computed hyphen is another5px; it is not a caller-authored break width.
        let source = Source::identified("XXX\u{ad}XXXX", Id::from_u64(72), 0);
        let selected = prepared_source_batch(
            source.clone(),
            ahem_style(5.0, 10.0),
            AvailableOf::definite(S::from_f64(20.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
            vec![replacement()],
        )
        .unwrap();
        let layout = selected.placed_text()[0].layout();
        assert_eq!(layout.metrics().line_count(), 2);
        assert_eq!(
            layout
                .glyph_runs()
                .iter()
                .map(|run| run.glyphs().len())
                .sum::<usize>(),
            8
        );
        assert_eq!(
            layout
                .lines()
                .iter()
                .map(|line| line.bounds().size.width)
                .collect::<Vec<_>>(),
            [20.0, 20.0]
        );
        assert_eq!(
            layout.source().text().code_units(),
            source.text().code_units()
        );
        let repeated = prepared_source_batch(
            source.clone(),
            ahem_style(5.0, 10.0),
            AvailableOf::definite(S::from_f64(20.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
            vec![replacement()],
        )
        .unwrap();
        assert_eq!(selected.final_entries(), repeated.final_entries());
        assert_eq!(layout.lines(), repeated.placed_text()[0].layout().lines());
        // Equal source is not the identity of a separately prepared occurrence.
        assert_ne!(
            selected.final_inline_fragments()[0]
                .fragment()
                .finalized_fragment()
                .view()
                .origin(),
            repeated.final_inline_fragments()[0]
                .fragment()
                .finalized_fragment()
                .view()
                .origin()
        );
        let unselected = prepared_source_batch(
            source,
            ahem_style(5.0, 10.0),
            AvailableOf::definite(S::from_f64(100.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
            vec![replacement()],
        )
        .unwrap();
        let layout = unselected.placed_text()[0].layout();
        assert_eq!(layout.metrics().line_count(), 1);
        let finalized = unselected.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let mut glyphs = Vec::new();
        for run in finalized.run_views() {
            let instance = run.effective_instance().unwrap();
            for glyph in run.glyphs() {
                let bounds = instance
                    .glyph_outline(glyph.id(), run.face().catalogue_identity())
                    .unwrap()
                    .bounds();
                glyphs.push((
                    glyph.range(),
                    glyph.id(),
                    glyph.advance(),
                    glyph.pen(),
                    bounds,
                ));
            }
        }
        // The retained default-ignorable SHY record is not an inserted hyphen.
        // Seven actual Ahem X outlines carry the 35px advance; SHY has no outline ink.
        let shy_range = surgeist_text::Range::new(3, 4);
        let visible = glyphs
            .iter()
            .filter(|glyph| glyph.0 != shy_range)
            .collect::<Vec<_>>();
        assert_eq!(
            visible.iter().map(|glyph| glyph.0).collect::<Vec<_>>(),
            [(0, 1), (1, 2), (2, 3), (4, 5), (5, 6), (6, 7), (7, 8)]
                .map(|(start, end)| surgeist_text::Range::new(start, end))
        );
        for glyph in visible {
            assert_eq!(glyph.1, 58); // Ahem's actual X glyph.
            assert_eq!(glyph.2, 5.0);
            assert_eq!(glyph.4, Some(surgeist_text::Rect::new(0.0, -4.0, 5.0, 5.0)));
        }
        let shy = glyphs
            .iter()
            .filter(|glyph| glyph.0 == shy_range)
            .collect::<Vec<_>>();
        assert_eq!(shy.len(), 1);
        assert_eq!(
            (shy[0].1, shy[0].2, shy[0].3, shy[0].4),
            (3, 0.0, 15.0, None)
        );
        assert_eq!(
            glyphs.iter().map(|glyph| f64::from(glyph.2)).sum::<f64>(),
            35.0
        );
        assert_eq!(
            layout.source().text().code_units(),
            "XXX\u{ad}XXXX".encode_utf16().collect::<Vec<_>>()
        );
        assert!(finalized.selected_line().request().alternative.is_none());
        assert!(
            unselected
                .final_inline_fragments()
                .iter()
                .all(|row| matches!(
                    row.fragment().finalized_fragment().view().owner(),
                    surgeist_text::FinalizedFragmentOwner::Origin(_)
                ))
        );
        let feedback = finalized.feedback();
        assert_eq!(
            (
                feedback.advance,
                feedback.occupied_start,
                feedback.occupied_end
            ),
            (35.0, 0.0, 35.0)
        );
        assert_eq!(
            finalized.selected_line().request().context.available_inline,
            Some(100.0)
        );
        assert_eq!(layout.lines()[0].bounds().size.width, 100.0);
        let mandatory = prepared_source_batch(
            Source::new("X\nX\n"),
            ahem_style(10.0, 10.0),
            AvailableOf::definite(S::from_f64(100.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
            vec![],
        )
        .unwrap();
        assert_eq!(
            mandatory.placed_text()[0].layout().metrics().line_count(),
            3
        );
        assert_eq!(
            fri06_c02_final_node(&mandatory, 0).size.height,
            S::from_f64(30.0)
        );
        let overwide = prepared_source_batch(
            Source::new("XXXXXX"),
            ahem_style(10.0, 10.0),
            AvailableOf::definite(S::from_f64(40.0)),
            WritingMode::HorizontalTb,
            Direction::Ltr,
            TextAlign::Start,
            vec![],
        )
        .unwrap();
        assert_eq!(overwide.placed_text()[0].layout().metrics().line_count(), 1);
        assert_eq!(
            overwide.placed_text()[0].layout().lines()[0]
                .bounds()
                .size
                .width,
            40.0
        );
        let overwide_line = overwide.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(overwide_line.feedback().advance, 60.0);
        assert_eq!(
            overwide_line
                .selected_line()
                .request()
                .context
                .available_inline,
            Some(40.0)
        );
        assert_eq!(
            overwide.placed_text()[0]
                .layout()
                .glyph_runs()
                .iter()
                .flat_map(|run| run.glyphs().iter())
                .map(|glyph| f64::from(glyph.advance()))
                .sum::<f64>(),
            60.0
        );
        assert_eq!(
            fri06_c02_final_node(&overwide, 0).content_size.width,
            S::from_f64(60.0)
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn supplied_line_edge_decisions_retain_every_logical_slot_without_fabricated_geometry() {
    use surgeist_text::{SuppliedInlineParticipant as Participant, SuppliedLine};
    let participants = [
        InlineWhitespaceEdge::DiscardAtLineStart,
        InlineWhitespaceEdge::Preserve,
        InlineWhitespaceEdge::DiscardAtLineEnd,
        InlineWhitespaceEdge::DiscardAtBoth,
        InlineWhitespaceEdge::Preserve,
    ]
    .map(|edge| Participant::Text {
        level: BidiLevel::try_new(0).unwrap(),
        edge,
    });
    let first = SuppliedLine::new(&participants[..3]);
    let second = SuppliedLine::new(&participants[3..]);
    assert_eq!(
        (0..3)
            .filter(|i| first.edges().is_discarded(*i) == Some(false))
            .collect::<Vec<_>>(),
        [1]
    );
    assert_eq!(
        (0..2)
            .filter(|i| second.edges().is_discarded(*i) == Some(false))
            .collect::<Vec<_>>(),
        [1]
    );
    let mut first_order = [usize::MAX; 3];
    assert_eq!(
        first
            .reorder_into(&mut first_order)
            .unwrap()
            .iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let only = [Participant::Text {
        level: BidiLevel::try_new(0).unwrap(),
        edge: InlineWhitespaceEdge::DiscardAtBoth,
    }];
    assert_eq!(SuppliedLine::new(&only).edges().is_discarded(0), Some(true));
    let mut order = [usize::MAX; 1];
    assert_eq!(
        SuppliedLine::new(&only)
            .reorder_into(&mut order)
            .unwrap()
            .iter()
            .map(|(i, _)| i)
            .collect::<Vec<_>>(),
        [0]
    );
    // Source anchors and absent-vs-zero allocation are tested by actual prepared
    // root captures, not inferred from this deliberately geometry-free front.
}

#[test]
fn prepared_text_intrinsic_uses_actual_selected_replacement_min_and_unselected_max_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let source = Source::new("XXX\u{ad}XXXX");
        let batch = |available| {
            prepared_source_batch(
                source.clone(),
                ahem_style(5.0, 10.0),
                available,
                WritingMode::HorizontalTb,
                Direction::Ltr,
                TextAlign::Start,
                vec![
                    ParagraphReplacement::resolved_native_logical_before(
                        ParagraphReplacementDefinition::Source {
                            origin: 0,
                            source: surgeist_text::Range::new(3, 4),
                        },
                        4,
                        vec![Text::from("-")],
                    )
                    .unwrap(),
                ],
            )
            .unwrap()
        };
        let min = batch(AvailableOf::<S>::MIN_CONTENT);
        let max = batch(AvailableOf::<S>::MAX_CONTENT);
        assert_eq!(fri06_c02_final_node(&min, 0).size.width, S::from_f64(20.0));
        assert_eq!(fri06_c02_final_node(&max, 0).size.width, S::from_f64(35.0));
        assert_eq!(min.placed_text()[0].layout().metrics().line_count(), 2);
        assert_eq!(max.placed_text()[0].layout().metrics().line_count(), 1);
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_block_text_adjacent_nodes_share_soft_mandatory_and_bidi_lines_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let style = ahem_style(5.0, 10.0);
        let soft = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X "), style.clone()),
                ParagraphPart::text(2, Source::new("XX"), style.clone()),
            ],
            style.clone(),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let batch = |paragraph: &PreparedParagraph, direction, available: AvailableOf<S>| {
            super::fixtures::prepared_mixed_batch(
                paragraph,
                vec![
                    (1, paragraph.text_input(1), NodeInputOf::non_box()),
                    (2, paragraph.text_input(2), NodeInputOf::non_box()),
                ],
                available,
                NodeInputOf {
                    display: Display::Block,
                    direction,
                    ..NodeInputOf::default()
                },
            )
        };
        let selected = batch(
            &soft,
            Direction::Ltr,
            AvailableOf::definite(S::from_f64(15.0)),
        );
        let rows = super::fixtures::origin_fragment_associations(selected.final_inline_fragments());
        assert!(
            rows.iter()
                .filter(|row| row.0 == 1)
                .all(|row| row.1 == soft.origin(1) && row.5 == 0)
        );
        assert!(
            rows.iter()
                .filter(|row| row.0 == 2)
                .all(|row| row.1 == soft.origin(2) && row.5 == 1)
        );
        assert_eq!(selected.placed_text()[0].layout().metrics().line_count(), 2);

        let rtl_style = ahem_style(10.0, 10.0);
        let bidi = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("א"), rtl_style.clone()),
                ParagraphPart::text(2, Source::new("ב"), rtl_style.clone()),
            ],
            rtl_style,
            surgeist_text::ParagraphBaseDirection::RightToLeft,
        );
        let selected = batch(
            &bidi,
            Direction::Rtl,
            AvailableOf::definite(S::from_f64(40.0)),
        );
        let rows = super::fixtures::origin_fragment_associations(selected.final_inline_fragments());
        assert_eq!(rows.len(), 2);
        let first = rows.iter().find(|row| row.0 == 1).unwrap();
        let second = rows.iter().find(|row| row.0 == 2).unwrap();
        assert_eq!(
            (first.1.clone(), second.1.clone()),
            (bidi.origin(1), bidi.origin(2))
        );
        assert_eq!((first.5, second.5), (0, 0));
        assert_eq!(
            first.3.unwrap().origin().x,
            second.3.unwrap().origin().x + S::from_f64(10.0)
        );

        let mandatory = PreparedParagraph::new(
            vec![
                ParagraphPart::text(1, Source::new("X\n"), style.clone()),
                ParagraphPart::text(2, Source::new("X"), style.clone()),
            ],
            style.clone(),
            surgeist_text::ParagraphBaseDirection::LeftToRight,
        );
        let selected = batch(
            &mandatory,
            Direction::Ltr,
            AvailableOf::definite(S::from_f64(40.0)),
        );
        let rows = super::fixtures::origin_fragment_associations(selected.final_inline_fragments());
        assert!(
            rows.iter()
                .filter(|row| row.0 == 1 && row.2.start == 0)
                .all(|row| row.5 == 0)
        );
        assert!(rows.iter().filter(|row| row.0 == 2).all(|row| row.5 == 1));
        assert_eq!(selected.placed_text()[0].layout().metrics().line_count(), 2);
        assert_eq!(
            selected.placed_text()[0]
                .layout()
                .source()
                .text()
                .code_units(),
            Source::new("X\nX").text().code_units()
        );

        let intrinsic = PreparedParagraph::with_replacements(
            vec![
                ParagraphPart::text(1, Source::new("XXX\u{ad}"), style.clone()),
                ParagraphPart::text(2, Source::new("XXXX\n"), style.clone()),
                ParagraphPart::text(3, Source::new("XXX"), style.clone()),
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            vec![
                ParagraphReplacement::resolved_native_logical_before(
                    ParagraphReplacementDefinition::Source {
                        origin: 0,
                        source: surgeist_text::Range::new(3, 4),
                    },
                    4,
                    vec![Text::from("-")],
                )
                .unwrap(),
            ],
        );
        let intrinsic_batch = |available: AvailableOf<S>| {
            super::fixtures::prepared_mixed_batch(
                &intrinsic,
                vec![
                    (1, intrinsic.text_input(1), NodeInputOf::non_box()),
                    (2, intrinsic.text_input(2), NodeInputOf::non_box()),
                    (3, intrinsic.text_input(3), NodeInputOf::non_box()),
                ],
                available,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
        };
        let min = intrinsic_batch(AvailableOf::MIN_CONTENT);
        let max = intrinsic_batch(AvailableOf::MAX_CONTENT);
        // Three cells plus the selected one-cell hyphen, versus seven cells
        // before the authored forced break. The second hard line is three cells.
        assert_eq!(fri06_c02_final_node(&min, 0).size.width, S::from_f64(20.0));
        assert_eq!(fri06_c02_final_node(&max, 0).size.width, S::from_f64(35.0));
        assert_eq!(max.placed_text()[0].layout().metrics().line_count(), 2);
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c02_scroll_uses_actual_source_and_replacement_fragment_rects_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let style = ahem_style(5.0, 5.0);
        let paragraph = PreparedParagraph::with_replacements(
            vec![
                ParagraphPart::text(1, Source::new("XXX\u{ad}"), style.clone()),
                ParagraphPart::text(2, Source::new("XX"), style.clone()),
            ],
            style,
            surgeist_text::ParagraphBaseDirection::LeftToRight,
            vec![
                ParagraphReplacement::resolved_native_logical_before(
                    ParagraphReplacementDefinition::Source {
                        origin: 0,
                        source: surgeist_text::Range::new(3, 4),
                    },
                    4,
                    vec![Text::from("-")],
                )
                .unwrap(),
            ],
        );
        let root = NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(22.0)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            overflow: ComputedOverflow::try_new(Overflow::Auto, Overflow::Auto).unwrap(),
            ..NodeInputOf::default()
        };
        let selected = super::fixtures::prepared_mixed_batch(
            &paragraph,
            vec![
                (1, paragraph.text_input(1), NodeInputOf::non_box()),
                (2, paragraph.text_input(2), NodeInputOf::non_box()),
            ],
            AvailableOf::definite(S::from_f64(22.0)),
            root,
        );
        let root = fri06_c02_final_node(&selected, 0);
        let range = root.scroll_geometry.unwrap().physical_range();
        assert_eq!(
            (
                range.x().minimum(),
                range.x().maximum(),
                range.y().minimum(),
                range.y().maximum()
            ),
            (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
        );
        assert_eq!(selected.placed_text()[0].layout().metrics().line_count(), 2);
        let replacements: Vec<_> = selected
            .final_inline_fragments()
            .iter()
            .filter(|entry| {
                matches!(
                    entry.fragment().finalized_fragment().view().owner(),
                    surgeist_text::FinalizedFragmentOwner::Alternative(_)
                )
            })
            .collect();
        assert_eq!(replacements.len(), 1);
        let replacement = replacements[0].fragment();
        assert_eq!(replacement.rect().unwrap().size().width, S::from_f64(5.0));
        assert_eq!(replacement.rect().unwrap().origin().x, S::from_f64(15.0));
        assert_eq!(
            replacement.finalized_fragment().view().origin(),
            Some(paragraph.origin(1))
        );
        assert!(
            selected
                .final_inline_fragments()
                .iter()
                .filter_map(|entry| entry.fragment().rect())
                .all(|rect| rect.origin().x + rect.size().width <= S::from_f64(22.0))
        );
        assert_eq!(
            selected.placed_text()[0]
                .layout()
                .source()
                .text()
                .code_units(),
            Source::new("XXX\u{ad}XX").text().code_units()
        );
    }
    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn assert_flex_root_percentage_parent_is_separate_from_host_fill<S: LayoutScalar>() {
    let host = Size::new(
        AvailableOf::definite(scalar(70.0)),
        AvailableOf::definite(scalar(110.0)),
    );

    for writing_mode in [WritingMode::VerticalRl, WritingMode::SidewaysLr] {
        let style = NodeInputOf::<S> {
            writing_mode,
            size: Size::new(PreferredSizeOf::px(scalar(20.0)), PreferredSizeOf::AUTO),
            max_size: Size::new(MaxSizeOf::NONE, MaxSizeOf::percent(scalar(0.8))),
            padding: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            border: Edges::new(
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
                LengthOf::percent(scalar(0.04)),
                LengthOf::ZERO,
            ),
            ..NodeInputOf::default()
        };

        for (viewport_height, expected_height, expected_edge) in
            [(210.0, 110.0, 8.0), (100.0, 80.0, 4.0)]
        {
            let viewport = Size::new(
                AvailableOf::definite(scalar(130.0)),
                AvailableOf::definite(scalar(viewport_height)),
            );
            let tree = FlowRootLeafTree::new(style.clone());
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::flex_item_under_viewport(
                    host,
                    FlexItemRootContextOf::under_viewport(
                        viewport,
                        FlowAxes::new(writing_mode, Direction::Ltr),
                    )
                    .expect("valid flex root viewport context"),
                )
                .expect("valid flex root request"),
            )
            .expect("flex root layout succeeds");
            let output = single_final_output(&batch);

            assert_eq!(output.location, Point::ZERO);
            assert_eq!(
                output.size,
                Size::new(scalar(20.0), scalar(expected_height))
            );
            assert_eq!(
                output.padding,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
            assert_eq!(
                output.border,
                Edges::new(
                    scalar(expected_edge),
                    S::ZERO,
                    scalar(expected_edge),
                    S::ZERO,
                )
            );
        }
    }
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f32() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f32>();
}

#[test]
fn flex_root_percentage_parent_separates_host_fill_for_f64() {
    assert_flex_root_percentage_parent_is_separate_from_host_fill::<f64>();
}

fn overflowing_scroll_edges() -> Edges<Length> {
    Edges {
        left: Length::px(f32::MAX),
        ..Edges::all(Length::ZERO)
    }
}

fn assert_leaf_content_overflow_is_invalid_numeric<S: LayoutScalar>() {
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let size = Size::new(largest, S::ONE);
    let style = NodeInputOf {
        padding: Edges {
            left: LengthOf::px(largest),
            ..Edges::all(LengthOf::ZERO)
        },
        ..NodeInputOf::default()
    };
    let input = ComputeInputOf::leaf_layout(
        size.map(Some),
        size.map(Some),
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        size.map(AvailableOf::definite),
    )
    .unwrap();
    let error = compute_leaf(input, &style, |_measurement| {
        Ok::<_, ()>(Size::new(largest, S::ZERO))
    })
    .expect_err("overflowing standalone measured-content geometry must fail");

    assert_eq!(error.site(), LayoutErrorSiteOf::Standalone);
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == S::INFINITY
    ));
}

#[test]
fn leaf_content_overflow_is_invalid_numeric_in_both_scalars() {
    assert_leaf_content_overflow_is_invalid_numeric::<f32>();
    assert_leaf_content_overflow_is_invalid_numeric::<f64>();
}

#[test]
fn scroll_geometry_error_maps_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default().style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            padding: overflowing_scroll_edges(),
            border: overflowing_scroll_edges(),
            ..NodeInput::default()
        },
    );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(f32::MAX), Available::definite(1.0)),
        LayoutErrorSite::Node(0),
        LayoutOperation::RootLayout,
        LayoutInternalInvariant::InvalidRootScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_non_root_block_overflow_through_the_public_front_door() {
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(1.0)),
                ..NodeInput::default()
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
                padding: overflowing_scroll_edges(),
                border: overflowing_scroll_edges(),
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                ..NodeInput::default()
            },
        );

    assert_public_scroll_geometry_error_without_batch(
        &tree,
        Size::new(Available::definite(100.0), Available::definite(1.0)),
        LayoutErrorSite::Node(1),
        LayoutOperation::ChildLayout,
        LayoutInternalInvariant::InvalidBlockScrollGeometry,
    );
}

#[test]
fn scroll_geometry_error_maps_block_inline_float_and_absolute_overflow_to_the_subject() {
    let available = Size::new(Available::definite(100.0), Available::definite(1.0));
    let variants = [
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            display: Display::InlineBlock,
            atomic_inline_participation: Some(fri06_atomic_participation()),
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            float: Float::Left,
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
        NodeInput {
            position: Position::Absolute,
            display: Display::Block,
            size: Size::new(PreferredSize::px(f32::MAX), PreferredSize::px(1.0)),
            margin: Edges {
                left: LengthAuto::px(f32::MAX),
                ..Edges::all(LengthAuto::ZERO)
            },
            ..NodeInput::default()
        },
    ];

    for child_style in variants {
        let tree: RootSessionTree = RootSessionTree::default()
            .children(0, [1])
            .children(1, [])
            .style(
                0,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(1.0)),
                    ..NodeInput::default()
                },
            )
            .style(1, child_style)
            .measure(1, Ok(Size::new(f32::MAX, 1.0)));

        assert_public_scroll_geometry_error_without_batch(
            &tree,
            available,
            LayoutErrorSite::ContainerSubject {
                container: 0,
                subject: 1,
            },
            LayoutOperation::ChildLayout,
            LayoutInternalInvariant::InvalidBlockScrollGeometry,
        );
    }
}

#[test]
fn root_request_rejects_invalid_definite_availability() {
    let cases = [
        (
            Size::new(Available::definite(-1.0), Available::MAX_CONTENT),
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::Negative { value: -1.0 },
        ),
        (
            Size::new(Available::definite(f32::NAN), Available::MAX_CONTENT),
            PhysicalAxis::Horizontal,
            NonNegativeFiniteScalarErrorOf::NonFinite { value: f32::NAN },
        ),
        (
            Size::new(Available::MAX_CONTENT, Available::definite(f32::INFINITY)),
            PhysicalAxis::Vertical,
            NonNegativeFiniteScalarErrorOf::NonFinite {
                value: f32::INFINITY,
            },
        ),
    ];

    for (available, axis, scalar_error) in cases {
        let error = LayoutRootRequest::viewport(available).unwrap_err();

        assert_eq!(error.axis(), axis);
        match (error.scalar(), scalar_error) {
            (
                NonNegativeFiniteScalarErrorOf::Negative { value },
                NonNegativeFiniteScalarErrorOf::Negative { value: expected },
            ) => assert_eq!(value, expected),
            (
                NonNegativeFiniteScalarErrorOf::NonFinite { value },
                NonNegativeFiniteScalarErrorOf::NonFinite { value: expected },
            ) => {
                if expected.is_nan() {
                    assert!(value.is_nan());
                } else {
                    assert_eq!(value, expected);
                }
            }
            (actual, expected) => panic!("expected {expected:?}, got {actual:?}"),
        }
    }

    let valid_viewport = Size::new(Available::definite(100.0), Available::definite(80.0));
    let flex_context = FlexItemRootContext::under_viewport(
        valid_viewport,
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    )
    .unwrap();
    let error = LayoutRootRequest::flex_item_under_viewport(
        Size::new(Available::definite(-2.0), Available::MAX_CONTENT),
        flex_context,
    )
    .unwrap_err();
    assert_eq!(error.axis(), PhysicalAxis::Horizontal);
    assert_eq!(
        error.scalar(),
        NonNegativeFiniteScalarErrorOf::Negative { value: -2.0 }
    );
}

#[test]
fn compute_layout_stops_after_first_recursive_child_error() {
    let tree = RootSessionTree::default()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, NodeInput::default())
        .style(1, NodeInput::default())
        .style(2, NodeInput::default())
        .measure(1, Err("first child failed"))
        .measure(2, Ok(Size::new(20.0, 10.0)));
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(1));
    assert_eq!(error.operation(), LayoutOperation::LeafMeasurement);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::Measurement("first child failed"),
    );
    assert_eq!(tree.measured_nodes(), vec![1]);
}

#[test]
fn compute_layout_reports_consumed_invalid_numeric_resolution() {
    let invalid_padding =
        LengthPercentageOf::from_coefficients(-f32::MAX, -1.0).expect("finite coefficients");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::px(10.0), PreferredSize::px(10.0)),
            padding: Edges::new(
                Length::value(invalid_padding),
                Length::ZERO,
                Length::ZERO,
                Length::ZERO,
            ),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(80.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    crate::test_support::assert_error_kind(
        error.kind(),
        &LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric {
            value: f32::NEG_INFINITY,
        }),
    );
}

#[test]
fn compute_layout_rejects_overflowing_affine_grid_auto_fit_track() {
    let overflowing =
        LengthPercentageOf::from_coefficients(f32::MAX, 1.0).expect("finite coefficients");
    let track = TrackSizing::from(overflowing);
    let repeat = TrackRepetition::auto_fit(vec![track]).expect("nonempty repeated track list");
    let tree: RootSessionTree = RootSessionTree::default().children(0, []).style(
        0,
        NodeInput {
            display: Display::Grid,
            grid_template_columns: vec![TrackComponent::Repeat(repeat)],
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(f32::MAX),
        Available::definite(20.0),
    ))
    .unwrap();

    let error = compute_layout(&tree, 0, request).unwrap_err();

    assert_eq!(error.site(), LayoutErrorSite::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKind::InvalidInput(LayoutInvalidInput::InvalidNumeric { .. })
    ));
}

#[test]
fn track_sizing_nested_calculation_produces_track_geometry() {
    let nested = SizingCalculation::min(vec![
        SizingCalculation::value(LengthPercentageOf::px(10.0).expect("finite track")),
        SizingCalculation::value(LengthPercentageOf::px(20.0).expect("finite track")),
    ])
    .expect("nonempty minimum");
    let tree: RootSessionTree = RootSessionTree::default()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            NodeInput {
                display: Display::Grid,
                grid_template_columns: vec![TrackSizing::calculation(nested).into()],
                ..NodeInput::default()
            },
        )
        .style(1, NodeInput::default());
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(20.0),
    ))
    .unwrap();

    let batch = compute_layout(&tree, 0, request).expect("nested track calculation resolves");
    let child = public_flow_output(batch.unrounded_entries(), 1);

    assert_eq!(child.location, Point::ZERO);
    assert_eq!(child.size.width, 10.0);
}

#[test]
fn hidden_layout_writes_zero_line_break_output_without_box_compute() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct HiddenTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        layouts: &'input mut HashMap<u32, NodeOutput>,
        caches: &'input mut HashMap<u32, Cache>,
        inputs: &'input HashMap<u32, LayoutInput>,
        hidden_children: &'input mut Vec<u32>,
    }

    impl HiddenTree {
        fn borrow_compute(&mut self) -> HiddenTreeCompute<'_> {
            HiddenTreeCompute {
                children: &self.children,
                layouts: &mut self.layouts,
                caches: &mut self.caches,
                inputs: &self.inputs,
                hidden_children: &mut self.hidden_children,
            }
        }
    }

    impl<'input> Traverse for HiddenTreeCompute<'input> {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for HiddenTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("line break node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            self.inputs[&node].as_view()
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::ContainingLayoutContext::new(
                        crate::geometry::FlowAxes::new(
                            crate::WritingMode::HorizontalTb,
                            crate::Direction::Ltr,
                        ),
                        crate::ParentFormattingContext::NoParent
                    ))
                );
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl<'input> CacheAccess for HiddenTreeCompute<'input> {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(3, LayoutInput::line_break(LineBreakInput::new()));
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(
        compute_hidden(
            &mut tree.borrow_compute(),
            1,
            SourceIndex::ZERO,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(
        tree.layouts[&1],
        NodeOutput::with_source_index(crate::SourceIndex::new(0))
    );
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn hidden_compute_sets_inline_boundary_children_to_hidden_output() {
    #[derive(Default)]
    struct HiddenTree {
        children: HashMap<u32, Vec<u32>>,
        layouts: HashMap<u32, NodeOutput>,
        caches: HashMap<u32, Cache>,
        inputs: HashMap<u32, LayoutInput>,
        hidden_children: Vec<u32>,
    }

    impl Traverse for HiddenTree {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    struct HiddenTreeCompute<'input> {
        children: &'input HashMap<u32, Vec<u32>>,
        layouts: &'input mut HashMap<u32, NodeOutput>,
        caches: &'input mut HashMap<u32, Cache>,
        inputs: &'input HashMap<u32, LayoutInput>,
        hidden_children: &'input mut Vec<u32>,
    }

    impl HiddenTree {
        fn borrow_compute(&mut self) -> HiddenTreeCompute<'_> {
            HiddenTreeCompute {
                children: &self.children,
                layouts: &mut self.layouts,
                caches: &mut self.caches,
                inputs: &self.inputs,
                hidden_children: &mut self.hidden_children,
            }
        }
    }

    impl<'input> Traverse for HiddenTreeCompute<'input> {
        type Node = u32;
        type Scalar = Scalar;
        type Children<'a>
            = std::iter::Copied<std::slice::Iter<'a, u32>>
        where
            Self: 'a;

        fn children(&self, node: Self::Node) -> Self::Children<'_> {
            self.children[&node].iter().copied()
        }

        fn child_count(&self, node: Self::Node) -> usize {
            self.children[&node].len()
        }

        fn child(&self, node: Self::Node, index: usize) -> Self::Node {
            self.children[&node][index]
        }
    }

    impl<'input> Compute<'input> for HiddenTreeCompute<'input> {
        fn observe_grid_overflow(
            &mut self,
            _node: Self::Node,
            _probe: crate::grid::GridOverflowProbe<'input, '_, Self::Node, Self::Scalar>,
        ) -> crate::LayoutResultOf<
            Self::Node,
            crate::grid::GridOverflowObservation<Self::Scalar>,
            Self::Scalar,
            core::convert::Infallible,
        > {
            panic!(
                "this focused computation fixture does not admit intrinsic Grid Auto overflow observation"
            );
        }

        fn node_input(&self, node: Self::Node) -> &'input NodeInput {
            self.inputs[&node]
                .as_box()
                .unwrap_or_else(|| panic!("inline boundary node {node} has no box NodeInput"))
        }

        fn layout_input(&self, node: Self::Node) -> crate::LayoutInputViewOf<'input, Self::Scalar> {
            self.inputs[&node].as_view()
        }

        fn set_unrounded(&mut self, node: Self::Node, layout: NodeOutput) {
            self.layouts.insert(node, layout);
        }

        fn compute_child(
            &mut self,
            node: Self::Node,
            input: ComputeInput,
        ) -> crate::LayoutResultOf<Self::Node, crate::ComputeOutputOf<Self::Scalar>, Self::Scalar>
        {
            Ok({
                assert_eq!(
                    input,
                    ComputeInput::hidden(crate::ContainingLayoutContext::new(
                        crate::geometry::FlowAxes::new(
                            crate::WritingMode::HorizontalTb,
                            crate::Direction::Ltr,
                        ),
                        crate::ParentFormattingContext::NoParent
                    ))
                );
                let _ = self.node_input(node);
                self.hidden_children.push(node);
                ComputeOutput::HIDDEN
            })
        }
    }

    impl<'input> CacheAccess for HiddenTreeCompute<'input> {
        type Node = u32;
        type Scalar = Scalar;

        fn cache_context(&self) -> crate::CacheKeyContext {
            crate::CacheKeyContext::new()
        }

        fn cache_get(
            &self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
        ) -> Option<ComputeOutput> {
            self.caches[&node].get_with_context(input, context)
        }

        fn cache_store(
            &mut self,
            node: Self::Node,
            input: &ComputeInput,
            context: crate::CacheKeyContext,
            output: ComputeOutput,
        ) {
            self.caches
                .get_mut(&node)
                .unwrap()
                .store_with_context(input, context, output);
        }

        fn cache_clear(&mut self, node: Self::Node) {
            self.caches.get_mut(&node).unwrap().clear();
        }
    }

    let metrics = InlineMetrics::from_line_height_and_baseline(16.0, 12.0).unwrap();
    let mut tree = HiddenTree::default();
    tree.children.insert(1, vec![2, 3]);
    tree.children.insert(2, vec![]);
    tree.children.insert(3, vec![]);
    tree.inputs
        .insert(1, LayoutInput::box_input(NodeInput::default()));
    tree.inputs
        .insert(2, LayoutInput::box_input(NodeInput::default()));
    tree.inputs.insert(
        3,
        LayoutInput::inline_boundary(InlineBoundaryInput::new(InlineBoundaryKind::Start, metrics)),
    );
    tree.caches.insert(1, Cache::new());
    tree.caches.insert(2, Cache::new());
    tree.caches.insert(3, Cache::new());

    assert_eq!(
        compute_hidden(
            &mut tree.borrow_compute(),
            1,
            SourceIndex::ZERO,
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            crate::scroll::SettledAutoScrollbarState::INITIAL,
        )
        .unwrap(),
        ComputeOutput::HIDDEN
    );
    assert_eq!(tree.hidden_children, vec![2]);
    assert_eq!(
        tree.layouts[&1],
        NodeOutput::with_source_index(crate::SourceIndex::new(0))
    );
    assert_eq!(
        tree.layouts[&3],
        NodeOutput::with_source_index(crate::SourceIndex::new(1))
    );
    assert!(tree.caches[&1].is_empty());
    assert!(tree.caches[&3].is_empty());
}

#[test]
fn f64_tree_can_run_root_layout_smoke_test() {
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<f64>::new().style(
        0,
        NodeInputOf::<f64> {
            display: Display::Block,
            size: Size::new(PreferredSizeOf::px(100.0), PreferredSizeOf::px(50.0)),
            ..NodeInputOf::<f64>::default()
        },
    );

    compute_root(
        &mut tree.borrow_compute(),
        0,
        Size::new(AvailableOf::definite(100.0), AvailableOf::definite(50.0)),
    )
    .unwrap();

    assert_eq!(
        tree.output(0)
            .expect("root layout must stage output for the root node")
            .size,
        Size::new(100.0, 50.0)
    );
}

#[test]
fn root_layout_emits_scroll_geometry_for_scroll_overflow() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::ZERO, Size::new(90.0, 30.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(40.0, 40.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, -5.0).unwrap()),
        PhysicalScrollOffset::try_new(40.0, 0.0).unwrap()
    );
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
}

#[test]
fn root_layout_emits_visible_scroll_geometry_without_range() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), None);
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::ZERO, Size::new(130.0, 70.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_layout_emits_clip_geometry_without_range() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.overflow_clip(), Some(geometry.scrollport()));
    assert_positive_physical_range(geometry.physical_range(), Size::ZERO);
}

#[test]
fn root_scroll_geometry_range_accounts_for_padding_border_and_gutter() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(10.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                padding: Edges::all(Length::px(2.0)),
                border: Edges::all(Length::px(3.0)),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(
        geometry.scrollport(),
        ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(84.0, 34.0)).unwrap()
    );
    assert_eq!(
        geometry.scrollable_overflow(),
        ScrollRect::try_new(Point::new(3.0, 3.0), Size::new(132.0, 72.0)).unwrap()
    );
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 38.0));
    assert_eq!(
        geometry
            .physical_range()
            .clamp(PhysicalScrollOffset::try_new(99.0, 99.0).unwrap()),
        PhysicalScrollOffset::try_new(48.0, 38.0).unwrap()
    );
}

#[test]
fn root_scroll_geometry_preserves_child_origin_bearing_scrollable_overflow() {
    let child_overflow =
        ScrollRect::try_new(Point::new(-12.0, -4.0), Size::new(160.0, 74.0)).unwrap();
    let child_geometry = root_test_scroll_geometry(RootTestScrollGeometryFacts {
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        item_is_replaced: false,
        border_box_size: Size::new(100.0, 40.0),
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_width: 0.0,
        scrollable_overflow: child_overflow,
    });
    let mut output = ComputeOutput::from_sizes(Size::new(100.0, 40.0), Size::new(130.0, 70.0));
    output.scroll_geometry = Some(child_geometry);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(40.0)),
                ..NodeInput::default()
            },
        )
        .measure(1, output);

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(100.0), Available::definite(40.0)),
    )
    .unwrap();

    let geometry = tree.layout(1).unwrap().scroll_geometry.unwrap();
    assert_eq!(geometry.scrollable_overflow(), child_overflow);
    assert_positive_physical_range(geometry.physical_range(), Size::new(48.0, 30.0));
}

#[test]
fn root_layout_stores_child_output_as_root_layout() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                direction: Direction::Rtl,
                overflow: computed_overflow(Overflow::Scroll, Overflow::Scroll),
                scrollbar_width: crate::ScrollbarWidthOf::try_new(13.0).unwrap(),
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1),
        &[ComputeInput::for_child(
            RunMode::PerformRootLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::new(Some(200.0), None),
            Size::new(Some(200.0), Some(100.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Rtl
                ),
                crate::ParentFormattingContext::NoParent
            ),
            Size::new(Available::definite(200.0), Available::definite(100.0))
        )]
    );
    let layout = tree.layout(1).expect("root layout should be stored");
    assert_eq!(layout.location, crate::Point::new(120.0, 0.0));
    assert_eq!(layout.size, Size::new(80.0, 20.0));
    assert_eq!(layout.content_size, Size::new(80.0, 20.0));
    assert_eq!(layout.scrollbar_size(), Size::new(13.0, 13.0));
}

#[test]
fn inline_level_root_keeps_intrinsic_width_under_definite_viewport() {
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::InlineGrid,
                ..NodeInput::default()
            },
        )
        .measure(
            1,
            ComputeOutput::from_sizes(Size::new(80.0, 20.0), Size::new(80.0, 20.0)),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(200.0), Available::definite(100.0)),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known(),
        Size::NONE
    );
    assert_eq!(
        tree.layout(1).expect("root layout should be stored").size,
        Size::new(80.0, 20.0)
    );
}

#[test]
fn max_width_root_uses_clamped_available_width_under_definite_viewport() {
    let expected_known = Size::new(Some(260.0), None);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
                ..NodeInput::default()
            },
        )
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(260.0, 72.0),
                Size::new(260.0, 72.0),
            ))
            .known(expected_known),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known(),
        expected_known
    );
    assert_eq!(
        tree.layout(1).expect("root layout should be stored").size,
        Size::new(260.0, 72.0)
    );
}

#[test]
fn block_root_with_max_width_uses_clamped_available_outer_width() {
    let expected_known = Size::new(Some(272.0), None);
    let mut tree = OracleTreeOf::new()
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                box_sizing: BoxSizing::ContentBox,
                max_size: Size::new(MaxSize::px(260.0), MaxSize::NONE),
                padding: Edges::new(
                    Length::px(1.0),
                    Length::px(5.0),
                    Length::px(1.0),
                    Length::px(5.0),
                ),
                border: Edges::all(Length::px(1.0)),
                ..NodeInput::default()
            },
        )
        .measure_when(
            1,
            OracleMeasurementOf::new(ComputeOutput::from_sizes(
                Size::new(272.0, 20.0),
                Size::new(272.0, 20.0),
            ))
            .known(expected_known),
        );

    compute_root(
        &mut tree.borrow_compute(),
        1,
        Size::new(Available::definite(800.0), Available::MAX_CONTENT),
    )
    .unwrap();

    assert_eq!(
        tree.inputs(1)
            .first()
            .expect("root should be computed")
            .known()
            .width,
        Some(272.0)
    );
    assert_eq!(
        tree.layout(1)
            .expect("root layout should be stored")
            .size
            .width,
        272.0
    );
}

#[test]
fn fri05_c03_root_geometry_fallback_roots_seed_their_own_padding_box() {
    let style = NodeInput {
        display: Display::Flex,
        size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
        padding: Edges::all(Length::px(10.0)),
        ..NodeInput::default()
    };
    let available = Size::splat(Available::definite(100.0));
    let parent_flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let flex_context = FlexItemRootContext::under_viewport(available, parent_flow_axes).unwrap();
    let requests = [
        (
            "viewport",
            LayoutRootRequest::viewport(available).expect("viewport request is valid"),
        ),
        (
            "flex-item",
            LayoutRootRequest::flex_item_under_viewport(available, flex_context)
                .expect("flex-item root request is valid"),
        ),
    ];

    for (root_kind, request) in requests {
        let tree = PublicFlowTree::default()
            .with_children(0, [])
            .with_style(0, style.clone());
        let batch = compute_layout(&tree, 0, request).expect("empty flex root lays out");
        assert_eq!(batch.unrounded_entries().len(), 1, "{root_kind}");
        assert_eq!(batch.final_entries().len(), 1, "{root_kind}");

        for (phase, output) in [
            ("unrounded", batch.unrounded_entries()[0].output()),
            ("rounded", batch.final_entries()[0].output()),
        ] {
            let geometry = output
                .scroll_geometry
                .expect("a performed fallback root has canonical geometry");
            let expected_padding_box =
                ScrollRect::try_new(Point::ZERO, Size::splat(100.0)).unwrap();
            let expected_content_box =
                ScrollRect::try_new(Point::new(10.0, 10.0), Size::splat(80.0)).unwrap();

            assert_eq!(output.size, Size::splat(100.0), "{root_kind}/{phase}");
            assert_eq!(
                geometry.padding_box(),
                expected_padding_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                geometry.content_box(),
                expected_content_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                geometry.scrollable_overflow(),
                expected_padding_box,
                "{root_kind}/{phase}"
            );
            assert_eq!(
                (
                    geometry.physical_range().x().minimum(),
                    geometry.physical_range().x().maximum(),
                    geometry.physical_range().y().minimum(),
                    geometry.physical_range().y().maximum(),
                ),
                (0.0, 0.0, 0.0, 0.0),
                "{root_kind}/{phase}"
            );
            assert_eq!(
                output.content_box_size(),
                expected_content_box.size(),
                "{root_kind}/{phase}"
            );
            assert_eq!(output.scrollbar_size(), Size::ZERO, "{root_kind}/{phase}");
            assert_eq!(
                geometry.target().border_box(),
                geometry.border_box(),
                "{root_kind}/{phase}"
            );
        }
    }
}

#[test]
fn fri05_c03_root_geometry_viewport_flex_rebuilds_complete_source_and_target() {
    let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
    let scroll_padding = ScrollPadding::new(
        ScrollPaddingValue::value(LengthPercentageOf::px(2.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(4.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(3.0).unwrap()),
        ScrollPaddingValue::value(LengthPercentageOf::px(1.0).unwrap()),
    );
    let scroll_margin = ScrollMargin::try_new(1.0, -2.0, 3.0, -4.0).unwrap();
    let snap_align = ScrollSnapAlign::new(ScrollSnapAlignValue::End, ScrollSnapAlignValue::Center);
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Flex,
            writing_mode: flow_axes.writing_mode(),
            direction: flow_axes.direction(),
            overflow: computed_overflow(Overflow::Hidden, Overflow::Scroll),
            overflow_clip_margin: OverflowClipMargin::try_new(OverflowClipBox::BorderBox, 5.0)
                .unwrap(),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(6.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            scroll_padding,
            scroll_margin,
            scroll_snap_type: ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            },
            scroll_snap_align: snap_align,
            scroll_snap_stop: ScrollSnapStop::Always,
            ..NodeInput::default()
        },
    );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(100.0),
            Available::definite(80.0),
        ))
        .unwrap(),
    )
    .expect("viewport flex root publishes canonical geometry");

    for output in [
        batch.unrounded_entries()[0].output(),
        batch.final_entries()[0].output(),
    ] {
        let geometry = output
            .scroll_geometry
            .expect("a performed viewport flex root has geometry");
        assert_eq!(geometry.flow_axes(), flow_axes);
        assert_eq!(geometry.used_overflow_x(), Overflow::Hidden);
        assert_eq!(geometry.used_overflow_y(), Overflow::Scroll);
        assert_eq!(
            geometry.resolved_scroll_padding(),
            Edges::new(2.0, 4.0, 3.0, 1.0)
        );
        assert_eq!(
            geometry.scroll_snap_type(),
            ScrollSnapType::Enabled {
                axis: ScrollSnapAxis::Both,
                strictness: ScrollSnapStrictness::Mandatory,
            }
        );
        let target = geometry.target();
        assert_eq!(target.border_box(), geometry.border_box());
        assert_eq!(target.scroll_margin(), scroll_margin);
        assert_eq!(target.flow_axes(), flow_axes);
        assert_eq!(target.snap_align(), snap_align);
        assert_eq!(target.snap_stop(), ScrollSnapStop::Always);
        assert_eq!(output.content_box_size(), geometry.content_box().size());
        assert_eq!(output.scrollbar_size(), geometry.scrollbar_size());
    }
}

fn fri05_c03_block_root_output(batch: &CompletedLayoutBatch<u32>, node: u32) -> NodeOutput {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap_or_else(|| panic!("FRI-05 block node {node} has staged output"))
        .output()
}

#[test]
fn fri05_c04_flex_auto_independent_inner_settlement_keys_grandchildren_by_inner_pass() {
    let tree = fri05_c04_nested_flex_auto_tree(true);
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let batch = compute_layout(&tree, 0, request).expect("independent inner auto layout succeeds");
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 0)
            .scroll_geometry
            .unwrap()
            .scrollbar_size(),
        Size::ZERO
    );
    assert_eq!(
        public_flow_output(batch.unrounded_entries(), 1)
            .scroll_geometry
            .unwrap()
            .scrollbar_size(),
        Size::new(15.0, 0.0)
    );

    let grandchild_requests = tree.cache_inputs(2);
    fri05_c04_assert_initial_local_auto_state(&grandchild_requests);
    assert_eq!(
        grandchild_requests
            .iter()
            .map(fri05_c03_block_root_state)
            .collect::<HashSet<_>>(),
        HashSet::from([(false, false), (true, true)])
    );
    for node in [0, 1, 2] {
        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only stable independently settled output is published for node {node}"
        );
    }
}

#[test]
fn fri05_c03_block_reservation_root_preserves_hidden_stable_both_edges() {
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(80.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request).expect("stable block root succeeds");
    let geometry = fri05_c03_block_root_output(&batch, 0)
        .scroll_geometry
        .expect("block root emits geometry");

    assert_eq!(geometry.scrollbar_size(), Size::new(30.0, 0.0));
    assert!(geometry.gutters().left().is_some());
    assert!(geometry.gutters().right().is_some());
    assert_eq!(geometry.gutters().top(), None);
    assert_eq!(geometry.gutters().bottom(), None);
    assert_eq!(geometry.content_box().size(), Size::new(70.0, 80.0));
}

#[test]
fn fri05_c03_block_auto_root_keys_each_pass_and_publishes_only_stable_nodes() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(120.0), PreferredSize::px(100.0)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
    let batch = compute_layout(&tree, 0, request).expect("auto block root succeeds");

    let expected_states = [(false, false), (true, false), (true, true)];
    let cache_inputs = tree.cache_inputs(1);
    fri05_c04_assert_initial_local_auto_state(&cache_inputs);
    assert_eq!(
        cache_inputs
            .iter()
            .map(fri05_c03_block_root_state)
            .collect::<Vec<_>>(),
        expected_states,
        "recorded child cache inputs: {cache_inputs:#?}"
    );
    assert_eq!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .map(|entry| fri05_c03_block_root_state(entry.input()))
            .collect::<Vec<_>>(),
        expected_states
    );
    assert!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .all(|entry| fri05_c04_local_auto_state(entry.input()) == (false, false))
    );

    for node in [0, 1] {
        assert_eq!(
            batch
                .unrounded_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only the stable unrounded output for node {node} is published"
        );
        assert_eq!(
            batch
                .final_entries()
                .iter()
                .filter(|entry| entry.node() == node)
                .count(),
            1,
            "only the stable rounded output for node {node} is published"
        );
    }

    let root = fri05_c03_block_root_output(&batch, 0);
    let geometry = root
        .scroll_geometry
        .expect("stable block root output includes geometry");
    assert_eq!(geometry.content_box().size(), Size::new(85.0, 85.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(15.0, 15.0));
    assert_eq!(root.scrollbar_size(), Size::new(15.0, 15.0));
}

#[test]
fn fri05_c03_block_auto_root_stable_reservations_stage_only_geometry_changes() {
    for (gutter, child_size, expected_states, expected_scrollbar_size) in [
        (
            ScrollbarGutter::Stable,
            Size::new(80.0, 120.0),
            vec![(false, false)],
            Size::new(15.0, 0.0),
        ),
        (
            ScrollbarGutter::StableBothEdges,
            Size::new(60.0, 120.0),
            vec![(false, false)],
            Size::new(30.0, 0.0),
        ),
        (
            ScrollbarGutter::Stable,
            Size::new(90.0, 120.0),
            vec![(false, false), (true, true)],
            Size::new(15.0, 15.0),
        ),
    ] {
        let tree = PublicFlowTree::default()
            .with_children(0, [1])
            .with_children(1, [])
            .with_style(
                0,
                NodeInput {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                    scrollbar_gutter: gutter,
                    scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                    size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                    ..NodeInput::default()
                },
            )
            .with_style(
                1,
                NodeInput {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSize::px(child_size.width),
                        PreferredSize::px(child_size.height),
                    ),
                    ..NodeInput::default()
                },
            );
        let request = LayoutRootRequest::viewport(Size::splat(Available::definite(100.0))).unwrap();
        let batch = compute_layout(&tree, 0, request).expect("stable auto block root succeeds");

        let cache_inputs = tree.cache_inputs(1);
        fri05_c04_assert_initial_local_auto_state(&cache_inputs);
        assert_eq!(
            cache_inputs
                .iter()
                .map(fri05_c03_block_root_state)
                .collect::<Vec<_>>(),
            expected_states,
            "{gutter:?} child size {child_size:?}"
        );
        assert_eq!(
            batch
                .cache_store_entries()
                .iter()
                .filter(|entry| entry.node() == 1)
                .count(),
            expected_states.len(),
            "only geometry-changing child evaluations are staged"
        );
        for node in [0, 1] {
            assert_eq!(
                batch
                    .unrounded_entries()
                    .iter()
                    .filter(|entry| entry.node() == node)
                    .count(),
                1,
                "only one stable unrounded output is published for node {node}"
            );
        }
        assert_eq!(
            fri05_c03_block_root_output(&batch, 0)
                .scroll_geometry
                .expect("stable root geometry is present")
                .scrollbar_size(),
            expected_scrollbar_size
        );
    }
}

#[test]
fn fri05_c03_block_tiny_root_keeps_ordered_zero_scrollport_geometry() {
    let tree = PublicFlowTree::default().with_children(0, []).with_style(
        0,
        NodeInput {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
            scrollbar_gutter: ScrollbarGutter::StableBothEdges,
            scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
            size: Size::new(PreferredSize::px(2.0), PreferredSize::px(20.0)),
            ..NodeInput::default()
        },
    );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(2.0),
        Available::definite(20.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request).expect("tiny block root remains supported");
    let geometry = fri05_c03_block_root_output(&batch, 0)
        .scroll_geometry
        .expect("tiny block root emits geometry");

    assert_eq!(geometry.border_box().size(), Size::new(2.0, 20.0));
    assert_eq!(geometry.content_box().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollport().size(), Size::new(0.0, 20.0));
    assert_eq!(geometry.scrollbar_size(), Size::new(2.0, 0.0));
}

#[test]
fn fri05_c03_block_tiny_root_available_below_raw_edges_avoids_false_auto_settlement() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
                scrollbar_width: ScrollbarWidth::try_new(15.0).unwrap(),
                size: Size::new(PreferredSize::px(100.0), PreferredSize::AUTO),
                border: Edges {
                    top: Length::px(15.0),
                    bottom: Length::px(15.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(0.0)),
                ..NodeInput::default()
            },
        );
    let request = LayoutRootRequest::viewport(Size::new(
        Available::definite(100.0),
        Available::definite(12.0),
    ))
    .unwrap();
    let batch = compute_layout(&tree, 0, request)
        .expect("root availability below raw edges remains supported");

    let cache_inputs = tree.cache_inputs(1);
    fri05_c04_assert_initial_local_auto_state(&cache_inputs);
    assert_eq!(cache_inputs.len(), 1);
    assert_eq!(fri05_c03_block_root_state(&cache_inputs[0]), (false, false));
    assert_eq!(
        cache_inputs[0].available().width,
        Available::definite(100.0)
    );
    assert_eq!(
        batch
            .cache_store_entries()
            .iter()
            .filter(|entry| entry.node() == 1)
            .count(),
        1
    );
    assert_eq!(
        fri05_c03_block_root_output(&batch, 1).location,
        Point::new(0.0, 15.0)
    );

    let root = fri05_c03_block_root_output(&batch, 0);
    assert_eq!(root.size, Size::new(100.0, 30.0));
    let geometry = root
        .scroll_geometry
        .expect("performed root emits canonical geometry");
    assert_eq!(geometry.border_box().size(), root.size);
    assert_eq!(geometry.padding_box().origin(), Point::new(0.0, 15.0));
    assert_eq!(geometry.padding_box().size(), Size::new(100.0, 0.0));
    assert_eq!(geometry.content_box(), geometry.padding_box());
    assert_eq!(geometry.scrollport(), geometry.padding_box());
    assert_eq!(geometry.physical_range().x().maximum(), 0.0);
    assert_eq!(geometry.physical_range().y().maximum(), 0.0);
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
}

#[test]
fn fri05_c04_flex_contribution_root_content_size_unions_anchor_and_overflow_per_axis() {
    let tree = PublicFlowTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Flex,
                size: Size::new(PreferredSize::px(10.0), PreferredSize::px(8.0)),
                align_items: Some(AlignItems::FlexStart),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(3.0)),
                min_size: Size::new(MinSize::ZERO, MinSize::ZERO),
                flex_shrink: FlexShrink::try_new(0.0).unwrap(),
                inset: Edges {
                    left: LengthAuto::px(-6.0),
                    top: LengthAuto::px(-4.0),
                    ..Edges::all(LengthAuto::AUTO)
                },
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequest::viewport(Size::new(
            Available::definite(10.0),
            Available::definite(8.0),
        ))
        .unwrap(),
    )
    .expect("root flex contribution layout succeeds");
    let root = public_flow_output(batch.final_entries(), 0);
    let geometry = root.scroll_geometry.expect("root flex geometry is present");
    let overflow = geometry.scrollable_overflow();
    let anchor = geometry.content_box().origin();
    let minimum = Point::new(
        anchor.x.min(overflow.origin().x),
        anchor.y.min(overflow.origin().y),
    );
    let maximum = Point::new(
        anchor.x.max(overflow.origin().x + overflow.size().width),
        anchor.y.max(overflow.origin().y + overflow.size().height),
    );

    assert!(overflow.origin().x < 0.0);
    assert!(overflow.origin().y < 0.0);
    assert_eq!(
        root.content_size,
        Size::new(maximum.x - minimum.x, maximum.y - minimum.y)
    );
}
