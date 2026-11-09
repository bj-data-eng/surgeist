use super::fixtures::{
    BlockTree, PublicBlockTree, ShapeProviderBehavior, all_writing_mode_directions,
    computed_overflow, fri06_atomic_participation, public_final_output, scalar_value,
};
use super::*;
use crate::test_support::readback::{self, Phase};

fn assert_replaced_fixed_block_clears_full_width_float<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    for replaced in [false, true] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: replaced,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(50.0)),
                        PreferredSizeOf::px(scalar(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                ),
                OrthogonalFallbackOf::new(
                    Size::new(
                        Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                        Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    ),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
            )),
        )
        .expect("fixed block siblings lay out");
        let float = public_final_output(&batch, 1);
        let block = public_final_output(&batch, 2);

        assert_eq!(float.location, Point::ZERO);
        assert_eq!(float.size, Size::new(scalar(100.0), scalar(20.0)));
        assert_eq!(block.size, Size::new(scalar(50.0), scalar(10.0)));
        assert_eq!(block.location.x, S::ZERO);
        // CSS2's bfc-next-to-float rule forbids replacement/float overlap.
        // The float fills the containing width, so the fixed replacement
        // cannot fit beside it. An ordinary block may extend under it.
        assert_eq!(
            block.location.y,
            scalar(if replaced { 20.0 } else { 0.0 }),
            "unexpected float isolation for replaced={replaced}",
        );
    }
}

#[test]
fn replaced_fixed_block_clears_full_width_float_in_f32() {
    assert_replaced_fixed_block_clears_full_width_float::<f32>();
}

#[test]
fn replaced_fixed_block_clears_full_width_float_in_f64() {
    assert_replaced_fixed_block_clears_full_width_float::<f64>();
}

fn assert_replaced_auto_block_retains_natural_measurement_around_floats<S: LayoutScalar>() {
    let scalar = scalar_value::<S>;
    for (float_width, fills_inline_band) in [(100.0, true), (40.0, false)] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(100.0)),
                        PreferredSizeOf::px(scalar(100.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(scalar(float_width)),
                        PreferredSizeOf::px(scalar(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    item_is_replaced: true,
                    ..NodeInputOf::default()
                },
            )
            .with_measurement(2, Size::new(scalar(50.0), scalar(10.0)));
        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                ),
                OrthogonalFallbackOf::new(
                    Size::new(
                        Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                        Some(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    ),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(scalar(100.0)).unwrap()),
                    SizingOperation::Normal,
                ),
            )),
        )
        .expect("naturally measured replacement lays out");
        let float = public_final_output(&batch, 1);
        let replacement = public_final_output(&batch, 2);

        assert_eq!(
            replacement.size,
            Size::new(scalar(50.0), scalar(10.0)),
            "float placement must retain the actual host measurement for float width {float_width}",
        );
        assert!(replacement.location.x >= S::ZERO);
        assert!(replacement.location.x + replacement.size.width <= scalar(100.0));
        // CSS2 permits beside or below placement when a band is wide
        // enough. Assert isolation, not an arbitrary choice between them.
        assert!(
            replacement.location.x >= float.location.x + float.size.width
                || replacement.location.y >= float.location.y + float.size.height,
            "the replacement must not overlap the preceding float",
        );
        if fills_inline_band {
            assert_eq!(replacement.location.x, S::ZERO);
            assert_eq!(replacement.location.y, scalar(20.0));
        }
    }
}

#[test]
fn replaced_auto_block_retains_natural_measurement_around_floats_in_f32() {
    assert_replaced_auto_block_retains_natural_measurement_around_floats::<f32>();
}

#[test]
fn replaced_auto_block_retains_natural_measurement_around_floats_in_f64() {
    assert_replaced_auto_block_retains_natural_measurement_around_floats::<f64>();
}

fn fri06_c04_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_size: crate::geometry::LogicalSizeOf<S>,
    side: Float,
    clear: Clear,
    logical_margin: crate::geometry::LogicalEdgesOf<S>,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        float: side,
        clear,
        size: flow_axes
            .physical_size(logical_size)
            .map(PreferredSizeOf::px),
        margin: flow_axes
            .physical_edges(logical_margin)
            .map(LengthAutoOf::px),
        ..NodeInputOf::default()
    }
}

fn fri06_c04_float_batch<S: LayoutScalar>(
    flow_axes: FlowAxes,
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> CompletedLayoutBatchOf<u32, S> {
    let logical_root_size =
        crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = PublicBlockTree::default()
        .with_children(0, child_ids)
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.with_children(node, []).with_style(node, style);
    }

    compute_layout(
        &tree,
        0,
        crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(
                tree.node_input(0).writing_mode,
                tree.node_input(0).direction,
            ),
            root_size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
            OrthogonalFallbackOf::new(
                root_size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(root_size.map(|value| {
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
                SizingOperation::Normal,
            )
        })),
    )
    .expect("rectangular float layout succeeds")
}

fn fri06_c04_block_layout_without_shape_provider<S: LayoutScalar>(
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> crate::test_support::layout_tree::OracleTreeOf<S> {
    let root_size = Size::new(scalar_value(100.0), scalar_value(160.0));
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
        .children(0, child_ids)
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.children(node, []).style(node, style);
    }

    crate::compute_block(
        &mut tree.borrow_compute(),
        0,
        ComputeInputOf::root_layout(
            Size::NONE,
            root_size.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                root_size.map(|value| {
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
                        SizingOperation::Normal,
                    )
                }),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .expect("C04 block layout succeeds without a shape provider");
    tree
}

fn fri06_c04_expected_float_location<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_origin: crate::geometry::LogicalPointOf<S>,
    logical_size: crate::geometry::LogicalSizeOf<S>,
) -> Point<S> {
    flow_axes.physical_point(
        logical_origin,
        logical_size,
        flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
            scalar_value(100.0),
            scalar_value(160.0),
        )),
    )
}

fn fri06_c05_provider_tree<S: LayoutScalar>(
    exclusion: FloatExclusion,
    provider: ShapeProviderBehavior<S>,
    spacer_block_extent: Option<S>,
) -> PublicBlockTree<S> {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let root_size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
    let mut children = vec![1];
    let mut tree = PublicBlockTree::default()
        .with_shape_provider(provider)
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .with_style(
            1,
            NodeInputOf {
                display: Display::Block,
                float: Float::Left,
                float_exclusion: exclusion,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(30.0)),
                    PreferredSizeOf::px(S::from_f64(30.0)),
                ),
                ..NodeInputOf::default()
            },
        );
    if let Some(block_extent) = spacer_block_extent {
        children.push(2);
        tree = tree.with_children(2, []).with_style(
            2,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(100.0)),
                    PreferredSizeOf::px(block_extent),
                ),
                ..NodeInputOf::default()
            },
        );
    }
    children.push(3);
    tree.with_children(0, children)
        .with_children(3, [])
        .with_style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                atomic_inline_participation: Some(fri06_atomic_participation()),
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(40.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                ..NodeInputOf::default()
            },
        )
}

fn fri06_c05_provider_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(
            Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
            Some(NonNegativeFiniteOf::new(S::from_f64(80.0)).unwrap()),
        ),
        OrthogonalFallbackOf::new(
            Size::new(
                Some(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
                Some(NonNegativeFiniteOf::new(S::from_f64(80.0)).unwrap()),
            ),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap()),
            SizingOperation::Normal,
        ),
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(80.0)).unwrap()),
            SizingOperation::Normal,
        ),
    ))
}

fn fri06_c05_expected_query<S: LayoutScalar>() -> FloatExclusionQueryOf<S> {
    FloatExclusionQueryOf::try_new(
        ScrollRectOf::try_new(Point::ZERO, Size::new(S::from_f64(30.0), S::from_f64(30.0)))
            .unwrap(),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        S::ZERO,
        S::from_f64(10.0),
    )
    .unwrap()
}

#[test]
fn fri06_c05_provider_role_valid_shape_reaches_exact_overlapping_band_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: S::from_f64(30.0),
            },
            None,
        );
        let batch = compute_layout(&tree, 0, fri06_c05_provider_request())
            .expect("a valid shape interval reaches the typed provider front door");

        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
        assert_eq!(
            public_final_output(&batch, 3).location,
            Point::new(S::from_f64(30.0), S::ZERO),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_role_empty_shape_is_valid_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree =
            fri06_c05_provider_tree::<S>(FloatExclusion::Shape, ShapeProviderBehavior::Empty, None);
        let batch = compute_layout(&tree, 0, fri06_c05_provider_request())
            .expect("an empty shape interval is a valid provider result");

        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
        assert_eq!(public_final_output(&batch, 3).location, Point::ZERO);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_role_margin_box_and_nonoverlapping_shape_make_zero_calls_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let margin_box = fri06_c05_provider_tree::<S>(
            FloatExclusion::MarginBox,
            ShapeProviderBehavior::Failure,
            None,
        );
        compute_layout(&margin_box, 0, fri06_c05_provider_request())
            .expect("margin-box exclusion does not require the provider");
        assert!(margin_box.shape_queries().is_empty());

        let nonoverlapping = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Failure,
            Some(S::from_f64(40.0)),
        );
        compute_layout(&nonoverlapping, 0, fri06_c05_provider_request())
            .expect("a non-overlapping shape does not require the provider");
        assert!(nonoverlapping.shape_queries().is_empty());
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_missing_provider_has_exact_context_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Missing,
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::MissingContext(LayoutMissingContext::FloatExclusionProvider),
        );
        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_failure_preserves_measurement_and_context_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Failure,
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert!(matches!(error.kind(), LayoutErrorKindOf::Measurement(())));
        assert_eq!(tree.shape_queries(), vec![(1, fri06_c05_expected_query())]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_provider_error_mismatched_query_preserves_expected_and_actual_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let expected = fri06_c05_expected_query();
        let actual = FloatExclusionQueryOf::try_new(
            expected.margin_box(),
            expected.flow_axes(),
            S::from_f64(12.0),
            S::from_f64(22.0),
        )
        .unwrap();
        let tree = fri06_c05_provider_tree::<S>(
            FloatExclusion::Shape,
            ShapeProviderBehavior::Mismatch {
                query: actual,
                minimum: S::ZERO,
                maximum: S::from_f64(30.0),
            },
            None,
        );
        let error = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap_err();

        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        crate::test_support::assert_error_kind(
            error.kind(),
            &LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::FloatExclusionProviderOutput {
                error: FloatExclusionIntervalErrorOf::QueryMismatch { expected, actual },
            }),
        );
        assert_eq!(tree.shape_queries(), vec![(1, expected)]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

fn fri06_c05_shape_tree<S: LayoutScalar>(
    flow_axes: FlowAxes,
    provider: ShapeProviderBehavior<S>,
    children: impl IntoIterator<Item = (u32, NodeInputOf<S>)>,
) -> PublicBlockTree<S> {
    let logical_root_size =
        crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(160.0));
    let root_size = flow_axes.physical_size(logical_root_size);
    let children = children.into_iter().collect::<Vec<_>>();
    let child_ids = children.iter().map(|(node, _)| *node).collect::<Vec<_>>();
    let mut tree = PublicBlockTree::default()
        .with_shape_provider(provider)
        .with_children(0, child_ids)
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: flow_axes.writing_mode(),
                direction: flow_axes.direction(),
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (node, style) in children {
        tree = tree.with_children(node, []).with_style(node, style);
    }
    tree
}

fn fri06_c05_shape_request<S: LayoutScalar>(flow_axes: FlowAxes) -> LayoutRootRequestOf<S> {
    let root_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    crate::test_support::requests::NativeRootFrameOf::new(
        flow_axes,
        root_size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
        OrthogonalFallbackOf::new(
            root_size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(root_size.map(|value| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
            SizingOperation::Normal,
        )
    }))
}

fn fri06_c05_shape_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    side: Float,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    let mut style = fri06_c04_float_style(
        flow_axes,
        crate::geometry::LogicalSizeOf::new(scalar_value(inline), scalar_value(block)),
        side,
        Clear::None,
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    );
    style.float_exclusion = FloatExclusion::Shape;
    style
}

fn fri06_c05_margin_float_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    side: Float,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    fri06_c04_float_style(
        flow_axes,
        crate::geometry::LogicalSizeOf::new(scalar_value(inline), scalar_value(block)),
        side,
        Clear::None,
        crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
    )
}

fn fri06_c05_atomic_style<S: LayoutScalar>(
    flow_axes: FlowAxes,
    inline: f64,
    block: f64,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::InlineBlock,
        writing_mode: flow_axes.writing_mode(),
        direction: flow_axes.direction(),
        atomic_inline_participation: Some(fri06_atomic_participation()),
        size: flow_axes
            .physical_size(crate::geometry::LogicalSizeOf::new(
                scalar_value(inline),
                scalar_value(block),
            ))
            .map(PreferredSizeOf::px),
        ..NodeInputOf::default()
    }
}

fn fri06_c05_physical_inline_interval<S: LayoutScalar>(
    flow_axes: FlowAxes,
    logical_minimum: f64,
    logical_maximum: f64,
) -> (S, S) {
    let containing_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    let logical_size = crate::geometry::LogicalSizeOf::new(
        scalar_value(logical_maximum - logical_minimum),
        S::ZERO,
    );
    let physical_origin = flow_axes.physical_point(
        crate::geometry::LogicalPointOf::new(scalar_value(logical_minimum), S::ZERO),
        logical_size,
        containing_size,
    );
    let physical_size = flow_axes.physical_size(logical_size);
    match flow_axes.inline_axis() {
        PhysicalAxis::Horizontal => (physical_origin.x, physical_origin.x + physical_size.width),
        PhysicalAxis::Vertical => (physical_origin.y, physical_origin.y + physical_size.height),
    }
}

fn fri06_c05_expected_shape_query<S: LayoutScalar>(
    flow_axes: FlowAxes,
    margin_box_origin: crate::geometry::LogicalPointOf<S>,
    margin_box_size: crate::geometry::LogicalSizeOf<S>,
    block_start: S,
    block_end: S,
) -> FloatExclusionQueryOf<S> {
    let containing_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        scalar_value(100.0),
        scalar_value(160.0),
    ));
    let physical_margin_box = ScrollRectOf::try_new(
        flow_axes.physical_point(margin_box_origin, margin_box_size, containing_size),
        flow_axes.physical_size(margin_box_size),
    )
    .unwrap();
    let physical_band_size = flow_axes.physical_size(crate::geometry::LogicalSizeOf::new(
        S::ZERO,
        block_end - block_start,
    ));
    let physical_band_origin = flow_axes.physical_point(
        crate::geometry::LogicalPointOf::new(S::ZERO, block_start),
        crate::geometry::LogicalSizeOf::new(S::ZERO, block_end - block_start),
        containing_size,
    );
    let (band_minimum, band_maximum) = match flow_axes.block_axis() {
        PhysicalAxis::Horizontal => (
            physical_band_origin.x,
            physical_band_origin.x + physical_band_size.width,
        ),
        PhysicalAxis::Vertical => (
            physical_band_origin.y,
            physical_band_origin.y + physical_band_size.height,
        ),
    };
    FloatExclusionQueryOf::try_new(physical_margin_box, flow_axes, band_minimum, band_maximum)
        .unwrap()
}

#[test]
fn fri06_c05_float_margin_boxes_ignore_empty_partial_full_clipped_and_zero_contours_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let cases = [
            ShapeProviderBehavior::Empty,
            ShapeProviderBehavior::Interval {
                minimum: scalar_value(0.0),
                maximum: scalar_value(40.0),
            },
            ShapeProviderBehavior::Interval {
                minimum: scalar_value(0.0),
                maximum: scalar_value(80.0),
            },
            ShapeProviderBehavior::Interval {
                minimum: scalar_value(-20.0),
                maximum: scalar_value(40.0),
            },
            ShapeProviderBehavior::Interval {
                minimum: scalar_value(90.0),
                maximum: scalar_value(110.0),
            },
            ShapeProviderBehavior::Interval {
                minimum: scalar_value(40.0),
                maximum: scalar_value(40.0),
            },
        ];
        for provider in cases {
            let tree = fri06_c05_shape_tree(
                flow_axes,
                provider,
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Left, 30.0, 10.0),
                    ),
                ],
            );
            let batch = compute_layout(&tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&batch, 2).location,
                Point::new(S::ZERO, scalar_value(20.0)),
                "a contour cannot change float margin-box collision"
            );
            assert!(tree.shape_queries().is_empty());
        }

        let stacked = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(100.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 20.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 20.0, 30.0),
                ),
                (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
            ],
        );
        let stacked_batch =
            compute_layout(&stacked, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&stacked_batch, 3).location,
            Point::new(scalar_value(40.0), S::ZERO),
            "same-side shape intervals must choose the farthest inward edge"
        );

        let opposing = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(100.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 40.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_shape_float_style(flow_axes, Float::Right, 60.0, 30.0),
                ),
                (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
            ],
        );
        let opposing_batch =
            compute_layout(&opposing, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&opposing_batch, 3).location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "opposing shapes must close the first band and advance to the finite transition"
        );

        let mut cleared = fri06_c05_margin_float_style(flow_axes, Float::Left, 10.0, 10.0);
        cleared.clear = Clear::Left;
        let cleared_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Empty,
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (2, cleared),
                (
                    3,
                    fri06_c05_margin_float_style(flow_axes, Float::Right, 120.0, 10.0),
                ),
            ],
        );
        let cleared_batch =
            compute_layout(&cleared_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&cleared_batch, 2).location,
            Point::new(S::ZERO, scalar_value(20.0)),
        );
        assert_eq!(
            public_final_output(&cleared_batch, 3).location,
            Point::new(scalar_value(-20.0), scalar_value(30.0)),
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_shape_flow_separates_float_margin_boxes_and_inline_contours_all_flows_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);

            let (minimum, maximum) = fri06_c05_physical_inline_interval(flow_axes, 0.0, 25.0);
            let start_tree = fri06_c05_shape_tree(
                flow_axes,
                ShapeProviderBehavior::Interval { minimum, maximum },
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Left, 40.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Left, 10.0, 10.0),
                    ),
                    (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
                ],
            );
            let start_batch =
                compute_layout(&start_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&start_batch, 2).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(40.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line-start shape side was lost for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&start_batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(50.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line band did not reuse the mapped line-start identity for {writing_mode:?} {direction:?}",
            );
            let expected_start_query = fri06_c05_expected_shape_query(
                flow_axes,
                crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
                S::ZERO,
                scalar_value(10.0),
            );
            assert_eq!(
                start_tree.shape_queries(),
                vec![(1, expected_start_query)],
                "only the line candidate queries the mapped contour for {writing_mode:?} {direction:?}",
            );

            let (minimum, maximum) = fri06_c05_physical_inline_interval(flow_axes, 75.0, 100.0);
            let end_tree = fri06_c05_shape_tree(
                flow_axes,
                ShapeProviderBehavior::Interval { minimum, maximum },
                [
                    (
                        1,
                        fri06_c05_shape_float_style(flow_axes, Float::Right, 40.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(flow_axes, Float::Right, 10.0, 10.0),
                    ),
                    (3, fri06_c05_atomic_style(flow_axes, 10.0, 10.0)),
                ],
            );
            let end_batch =
                compute_layout(&end_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
            assert_eq!(
                public_final_output(&end_batch, 2).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(50.0), S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
                ),
                "line-end shape side was lost for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&end_batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0))
                )
            );
            let expected_end_query = fri06_c05_expected_shape_query(
                flow_axes,
                crate::geometry::LogicalPointOf::new(scalar_value(60.0), S::ZERO),
                crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
                S::ZERO,
                scalar_value(10.0),
            );
            assert_eq!(
                end_tree.shape_queries(),
                vec![(1, expected_end_query)],
                "the line-end inline candidate retains the physical margin box and ordered band for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c05_shape_query_float_and_line_consumers_preserve_bfc_margin_box_isolation_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let expected_margin_box = ScrollRectOf::try_new(
            Point::ZERO,
            Size::new(scalar_value(80.0), scalar_value(20.0)),
        )
        .unwrap();
        let expected = FloatExclusionQueryOf::try_new(
            expected_margin_box,
            flow_axes,
            S::ZERO,
            scalar_value(10.0),
        )
        .unwrap();

        let float_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_margin_float_style(flow_axes, Float::Left, 30.0, 10.0),
                ),
            ],
        );
        let float_batch =
            compute_layout(&float_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&float_batch, 2).location,
            Point::new(S::ZERO, scalar_value(20.0))
        );
        assert!(float_tree.shape_queries().is_empty());

        let bfc_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                        size: Size::new(
                            PreferredSizeOf::px(scalar_value(30.0)),
                            PreferredSizeOf::px(scalar_value(10.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                ),
            ],
        );
        let bfc_batch = compute_layout(&bfc_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&bfc_batch, 2).location,
            Point::new(S::ZERO, scalar_value(20.0)),
        );
        assert!(bfc_tree.shape_queries().is_empty());

        let line_tree = fri06_c05_shape_tree(
            flow_axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: scalar_value(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(flow_axes, Float::Left, 80.0, 20.0),
                ),
                (2, fri06_c05_atomic_style(flow_axes, 60.0, 10.0)),
            ],
        );
        let line_batch = compute_layout(&line_tree, 0, fri06_c05_shape_request(flow_axes)).unwrap();
        assert_eq!(
            public_final_output(&line_batch, 2).location,
            Point::new(scalar_value(40.0), S::ZERO)
        );
        assert_eq!(line_tree.shape_queries(), vec![(1, expected)]);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn bfc_border_avoids_shape_float_margin_boxes_without_querying_contours() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            let (minimum, maximum) = fri06_c05_physical_inline_interval(axes, 0.0, 40.0);
            for provider in [
                ShapeProviderBehavior::Interval { minimum, maximum },
                ShapeProviderBehavior::Missing,
                ShapeProviderBehavior::Failure,
                ShapeProviderBehavior::Empty,
            ] {
                // A 15-wide border fits in the 20-wide remainder; a 30-wide
                // border must clear the float. Fixed BFC placement retains its
                // authored margins inside the remaining margin-box band.
                for (float_inline, float_margin, inline, margin, clear, x, y) in [
                    (80.0, 0.0, 15.0, 0.0, Clear::None, 80.0, 0.0),
                    (80.0, 0.0, 30.0, 0.0, Clear::None, 0.0, 20.0),
                    (70.0, 10.0, 15.0, 5.0, Clear::None, 85.0, 0.0),
                    (80.0, 0.0, 15.0, 0.0, Clear::Both, 0.0, 20.0),
                ] {
                    let mut float =
                        fri06_c05_shape_float_style(axes, Float::Left, float_inline, 20.0);
                    float.margin = axes
                        .physical_edges(LogicalEdgesOf::new(
                            S::ZERO,
                            px(float_margin),
                            S::ZERO,
                            S::ZERO,
                        ))
                        .map(LengthAutoOf::px);
                    let size = LogicalSizeOf::new(px(inline), px(10.0));
                    let tree = fri06_c05_shape_tree(
                        axes,
                        provider.clone(),
                        [
                            (1, float),
                            (
                                2,
                                NodeInputOf {
                                    display: Display::Block,
                                    writing_mode,
                                    direction,
                                    block_flow_context: crate::BlockFlowContext::Independent,
                                    size: axes.physical_size(size).map(PreferredSizeOf::px),
                                    margin: axes
                                        .physical_edges(LogicalEdgesOf::new(
                                            px(margin),
                                            S::ZERO,
                                            S::ZERO,
                                            S::ZERO,
                                        ))
                                        .map(LengthAutoOf::px),
                                    clear,
                                    ..NodeInputOf::default()
                                },
                            ),
                        ],
                    );
                    let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes))
                        .expect("BFC collision needs margin boxes, not a contour provider");
                    let bfc = public_final_output(&batch, 2);
                    assert_eq!(bfc.size, axes.physical_size(size));
                    assert_eq!(
                        bfc.location,
                        fri06_c04_expected_float_location(
                            axes,
                            LogicalPointOf::new(px(x), px(y)),
                            size,
                        ),
                        "{writing_mode:?} {direction:?}: float {float_inline}+{float_margin}, BFC {inline}+{margin}, clear {clear:?}"
                    );
                    assert!(tree.shape_queries().is_empty());
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn fri06_c12_t08_float_dominated_terminal_extent_stops_at_the_float_edge() {
    fn assert_lane<S: LayoutScalar>() {
        let root = NodeInputOf {
            display: Display::Block,
            overflow: computed_overflow(Overflow::Auto, Overflow::Auto),
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(180.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let mut floating = fri06_c05_shape_float_style(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Float::Left,
            44.0,
            60.0,
        );
        floating.float_exclusion = FloatExclusion::Shape;
        // Numeric inline boxes preserve this float-edge experiment:
        // the leading box has before/after-baseline extents 14.8/5.2; the
        // following 16px boxes therefore make the first line 21.2px high.
        let leading = NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(48.164_062_5)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                InlineBreakOpportunity::Prohibited,
            )),
            ..NodeInputOf::default()
        };
        let atomic = |inline_extent, following_break| NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(inline_extent)),
                PreferredSizeOf::px(S::from_f64(16.0)),
            ),
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                following_break,
            )),
            ..NodeInputOf::default()
        };
        let atomics = [
            atomic(34.0, InlineBreakOpportunity::Prohibited),
            // Both real groups must fit the136px shape band: the first is
            // 48.1640625+34+38, the second42+46. An indivisible162.1640625
            // group would instead wait below the60px float.
            atomic(38.0, InlineBreakOpportunity::Allowed),
            atomic(42.0, InlineBreakOpportunity::Prohibited),
            atomic(46.0, InlineBreakOpportunity::Prohibited),
        ];
        let mut tree = PublicBlockTree::default()
            .with_shape_provider(ShapeProviderBehavior::Bands(vec![
                (S::ZERO, S::from_f64(21.2), S::ZERO, S::from_f64(44.0)),
                (
                    S::from_f64(21.2),
                    S::from_f64(37.2),
                    S::ZERO,
                    S::from_f64(44.0),
                ),
            ]))
            .with_children(0, [1, 2, 3, 4, 5, 6])
            .with_children(1, [])
            .with_children(2, [7])
            .with_children(7, [])
            .with_style(7, NodeInputOf::non_box())
            .with_layout_input(
                7,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        InlineBoundaryKind::Start,
                        InlineMetricsOf::from_line_height_and_baseline(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(20.0),
                            S::from_f64(14.8),
                        )
                        .unwrap(),
                        crate::InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            )
            .with_style(0, root)
            .with_style(1, floating)
            .with_style(2, leading);
        for (offset, style) in atomics.into_iter().enumerate() {
            let node = u32::try_from(offset + 3).unwrap();
            tree = tree.with_children(node, []).with_style(node, style);
        }

        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(
                    Some(NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap()),
                    None,
                ),
                OrthogonalFallbackOf::new(
                    Size::new(
                        Some(NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap()),
                        None,
                    ),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
        .expect("shape-excluded inline layout succeeds");

        assert_eq!(
            readback::single_box_output(&batch, 0, Phase::Unrounded)
                .size
                .height,
            S::from_f64(60.0),
        );
        assert_eq!(
            public_final_output(&batch, 0).size.height,
            S::from_f64(60.0)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c12_t08_float_shape_slots_follow_visual_order_before_physical_projection() {
    fn assert_lane<S: LayoutScalar>(direction: Direction) {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
        let root = NodeInputOf {
            display: Display::Block,
            writing_mode: WritingMode::HorizontalTb,
            direction,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(180.0)),
                PreferredSizeOf::AUTO,
            ),
            ..NodeInputOf::default()
        };
        let floating = fri06_c05_shape_float_style(flow_axes, Float::Right, 44.0, 60.0);
        let (shape_minimum, shape_maximum) = match direction {
            Direction::Ltr => (S::from_f64(136.0), S::from_f64(180.0)),
            Direction::Rtl => (S::ZERO, S::from_f64(44.0)),
        };
        let bidi_level = BidiLevel::try_new(u8::from(direction == Direction::Rtl)).unwrap();
        let inline_box = |width| NodeInputOf {
            display: Display::InlineBlock,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::px(S::from_f64(20.0)),
            ),
            atomic_inline_participation: Some(AtomicInlineParticipation::new(
                bidi_level,
                InlineBreakOpportunity::Prohibited,
            )),
            ..NodeInputOf::default()
        };
        let tree = PublicBlockTree::default()
            .with_shape_provider(ShapeProviderBehavior::Interval {
                minimum: shape_minimum,
                maximum: shape_maximum,
            })
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [4])
            .with_children(3, [5])
            .with_children(4, [])
            .with_children(5, [])
            .with_style(4, NodeInputOf::non_box())
            .with_style(5, NodeInputOf::non_box())
            .with_layout_input(
                4,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        InlineBoundaryKind::Start,
                        InlineMetricsOf::from_line_height_and_baseline(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(20.0),
                            S::from_f64(14.0),
                        )
                        .unwrap(),
                        crate::InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            )
            .with_layout_input(
                5,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        InlineBoundaryKind::Start,
                        InlineMetricsOf::from_line_height_and_baseline(
                            crate::InlineBaselineContext::Physical,
                            S::from_f64(20.0),
                            S::from_f64(14.0),
                        )
                        .unwrap(),
                        crate::InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            )
            .with_style(0, root)
            .with_style(1, floating)
            .with_style(2, inline_box(66.0))
            .with_style(3, inline_box(114.0));

        let batch = compute_layout(
            &tree,
            0,
            crate::test_support::requests::NativeRootFrameOf::new(
                FlowAxes::new(
                    tree.node_input(0).writing_mode,
                    tree.node_input(0).direction,
                ),
                Size::new(
                    Some(NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap()),
                    None,
                ),
                OrthogonalFallbackOf::new(
                    Size::new(
                        Some(NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap()),
                        None,
                    ),
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            )
            .request(Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(
                        NonNegativeFiniteOf::new(S::from_f64(180.0)).unwrap(),
                    ),
                    SizingOperation::Normal,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
        .expect("shape-backed float slot layout succeeds");
        let flow_inline_starts = [2, 3]
            .into_iter()
            .map(|node| {
                let output = readback::single_box_output(&batch, node, Phase::Unrounded);
                match direction {
                    Direction::Ltr => output.location.x,
                    Direction::Rtl => output.location.x + output.size.width,
                }
            })
            .collect::<Vec<_>>();
        let expected = match direction {
            Direction::Ltr => vec![S::ZERO, S::from_f64(66.0)],
            Direction::Rtl => vec![S::from_f64(180.0), S::from_f64(114.0)],
        };
        assert_eq!(
            flow_inline_starts, expected,
            "L2 places RTL B114/A66 from line-left; their physical right edges are A180/B114",
        );
    }

    assert_lane::<f32>(Direction::Ltr);
    assert_lane::<f64>(Direction::Ltr);
    assert_lane::<f32>(Direction::Rtl);
    assert_lane::<f64>(Direction::Rtl);
}

#[test]
fn fri06_c04_float_place_mapped_sides_and_clear_values_all_flows_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(30.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(25.0),
                                scalar_value(30.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(20.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::Left,
                            zero,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(15.0),
                                scalar_value(10.0),
                            ),
                            Float::Right,
                            Clear::Right,
                            zero,
                        ),
                    ),
                    (
                        5,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(10.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::Both,
                            zero,
                        ),
                    ),
                ],
            );

            for (node, inline, block, inline_size, block_size) in [
                (1, 0.0, 0.0, 30.0, 20.0),
                (2, 75.0, 0.0, 25.0, 30.0),
                (3, 0.0, 20.0, 20.0, 10.0),
                (4, 85.0, 30.0, 15.0, 10.0),
                (5, 0.0, 40.0, 10.0, 10.0),
            ] {
                assert_eq!(
                    public_final_output(&batch, node).location,
                    fri06_c04_expected_float_location(
                        flow_axes,
                        crate::geometry::LogicalPointOf::new(
                            scalar_value(inline),
                            scalar_value(block),
                        ),
                        crate::geometry::LogicalSizeOf::new(
                            scalar_value(inline_size),
                            scalar_value(block_size),
                        ),
                    ),
                    "mapped float placement diverged for {writing_mode:?} {direction:?} node {node}",
                );
            }
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_place_margin_box_remains_physical_for_later_float_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let margin_box_float = fri06_c04_float_style(
            flow_axes,
            crate::geometry::LogicalSizeOf::new(scalar_value(80.0), scalar_value(20.0)),
            Float::Left,
            Clear::None,
            zero,
        );
        let tree = fri06_c04_block_layout_without_shape_provider([
            (1, margin_box_float),
            (
                2,
                fri06_c04_float_style(
                    flow_axes,
                    crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(10.0)),
                    Float::Left,
                    Clear::None,
                    zero,
                ),
            ),
        ]);

        assert_eq!(
            tree.layout(2).expect("later float was laid out").location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "later float must avoid the Shape float's physical margin box",
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_bfc_margin_box_remains_physical_without_provider_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        let margin_box_float = fri06_c04_float_style(
            flow_axes,
            crate::geometry::LogicalSizeOf::new(scalar_value(80.0), scalar_value(20.0)),
            Float::Left,
            Clear::None,
            zero,
        );
        let tree = fri06_c04_block_layout_without_shape_provider([
            (1, margin_box_float),
            (
                2,
                NodeInputOf {
                    display: Display::Block,
                    overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                    size: Size::new(
                        PreferredSizeOf::px(scalar_value(30.0)),
                        PreferredSizeOf::px(scalar_value(10.0)),
                    ),
                    ..NodeInputOf::default()
                },
            ),
        ]);

        assert_eq!(
            tree.layout(2)
                .expect("qualifying BFC was laid out")
                .location,
            Point::new(S::ZERO, scalar_value(20.0)),
            "qualifying BFC must avoid the Shape float's physical margin box",
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_line_band_direct_compute_inherits_float_into_internal_line_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let root_size = Size::new(S::from_f64(100.0), S::from_f64(80.0));
        let fixed_size = |width, height| {
            Size::new(
                PreferredSizeOf::px(S::from_f64(width)),
                PreferredSizeOf::px(S::from_f64(height)),
            )
        };
        let mut tree = crate::test_support::layout_tree::OracleTreeOf::<S>::new()
            .children(0, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [4, 5])
            .children(4, [])
            .children(5, [])
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    size: fixed_size(0.0, 0.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: fixed_size(30.0, 30.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                3,
                NodeInputOf {
                    display: Display::Block,
                    ..NodeInputOf::default()
                },
            )
            .style(
                4,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Right,
                    size: fixed_size(20.0, 20.0),
                    ..NodeInputOf::default()
                },
            )
            .style(
                5,
                NodeInputOf {
                    display: Display::InlineBlock,
                    size: fixed_size(40.0, 10.0),
                    atomic_inline_participation: Some(fri06_atomic_participation()),
                    ..NodeInputOf::default()
                },
            );

        crate::compute_block(
            &mut tree.borrow_compute(),
            0,
            ComputeInputOf::root_layout(
                Size::NONE,
                root_size.map(Some),
                ContainingLayoutContext::new(
                    FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                    ParentFormattingContext::NoParent,
                ),
                SizingRequestOf::new(
                    root_size.map(|value| {
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
                            SizingOperation::Normal,
                        )
                    }),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        NearestScrollportOf::Unknown,
                        Size::splat(OrthogonalFallbackLimitOf::Unknown),
                    ),
                ),
            ),
        )
        .expect("direct block compute succeeds");

        let parent_float = tree.layout(2).expect("parent float is laid out");
        assert_eq!(parent_float.source_index, SourceIndex::new(1));
        let ordinary = tree.layout(3).expect("ordinary child is laid out");
        assert_eq!(ordinary.source_index, SourceIndex::new(2));
        assert_eq!(ordinary.location, Point::ZERO);
        assert_eq!(ordinary.size.width, S::from_f64(100.0));

        let local_float = tree.layout(4).expect("child-local float is laid out");
        assert_eq!(local_float.source_index, SourceIndex::ZERO);
        assert_eq!(local_float.location, Point::new(S::from_f64(80.0), S::ZERO));
        let internal_line = tree.layout(5).expect("internal atomic line is laid out");
        assert_eq!(internal_line.source_index, SourceIndex::new(1));
        assert_eq!(
            internal_line.location,
            Point::new(S::from_f64(30.0), S::ZERO)
        );
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_full_span_asymmetric_opposing_and_source_order_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let candidate_margin = crate::geometry::LogicalEdgesOf::new(
                scalar_value(3.0),
                scalar_value(7.0),
                scalar_value(5.0),
                scalar_value(5.0),
            );
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(40.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(30.0),
                                scalar_value(60.0),
                            ),
                            Float::Right,
                            Clear::Left,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(40.0),
                                scalar_value(30.0),
                            ),
                            Float::Left,
                            Clear::None,
                            candidate_margin,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(20.0),
                                scalar_value(10.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                ],
            );

            assert_eq!(
                public_final_output(&batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(3.0), scalar_value(25.0),),
                    crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(30.0),),
                ),
                "full-span collision was missed for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&batch, 4).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(50.0), scalar_value(20.0),),
                    crate::geometry::LogicalSizeOf::new(scalar_value(20.0), scalar_value(10.0),),
                ),
                "same-side source order diverged for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_evaluates_each_float_span_pair_once_per_candidate_pass() {
    fn assert_lane<S: LayoutScalar>() {
        let flow_axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Rtl);
        let logical_container =
            crate::geometry::LogicalSizeOf::new(scalar_value(100.0), scalar_value(120.0));
        let mut exclusions = FloatExclusions::new(
            flow_axes,
            flow_axes.physical_size(logical_container),
            scalar_value(100.0),
            crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(S::ZERO, S::ZERO),
            crate::geometry::LogicalSizeOf::new(scalar_value(40.0), scalar_value(20.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineEnd,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(70.0), scalar_value(20.0)),
            crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(60.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(10.0), scalar_value(90.0)),
            crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(10.0)),
        );
        exclusions.record_test_float(
            FloatLedgerSide::LineStart,
            FloatExclusion::MarginBox,
            crate::geometry::LogicalPointOf::new(scalar_value(20.0), S::ZERO),
            crate::geometry::LogicalSizeOf::new(scalar_value(30.0), scalar_value(40.0)),
        );

        let band = exclusions
            .query_rectangular_line_band(S::ZERO, scalar_value(40.0))
            .unwrap();
        assert_eq!(band.inline_start, scalar_value(50.0));
        assert_eq!(band.inline_end, scalar_value(70.0));
        assert_eq!(band.next_transition, Some(scalar_value(20.0)));
        assert_eq!(band.evaluated, 4);
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn fri06_c04_float_ledger_shape_is_not_approximated_by_rectangular_line_band() {
    let flow_axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut exclusions = FloatExclusions::new(
        flow_axes,
        Size::new(100.0, 100.0),
        100.0,
        crate::geometry::LogicalEdgesOf::new(0.0, 0.0, 0.0, 0.0),
    );
    exclusions.record_test_float(
        FloatLedgerSide::LineStart,
        FloatExclusion::Shape,
        crate::geometry::LogicalPointOf::new(0.0, 0.0),
        crate::geometry::LogicalSizeOf::new(80.0, 50.0),
    );

    let band = exclusions.query_rectangular_line_band(0.0, 20.0).unwrap();
    assert_eq!(band.inline_start, 0.0);
    assert_eq!(band.inline_end, 100.0);
    assert_eq!(band.next_transition, None);
    assert_eq!(band.evaluated, 1);
}

#[test]
fn fri06_c04_float_progress_zero_band_exact_transition_and_overwide_side_both_scalars() {
    fn assert_lane<S: LayoutScalar>() {
        let zero = crate::geometry::LogicalEdgesOf::new(S::ZERO, S::ZERO, S::ZERO, S::ZERO);
        for (writing_mode, direction) in all_writing_mode_directions() {
            let flow_axes = FlowAxes::new(writing_mode, direction);
            let batch = fri06_c04_float_batch(
                flow_axes,
                [
                    (
                        1,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(50.0),
                                scalar_value(20.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        2,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(50.0),
                                scalar_value(30.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        3,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(10.0),
                                scalar_value(15.0),
                            ),
                            Float::Left,
                            Clear::None,
                            zero,
                        ),
                    ),
                    (
                        4,
                        fri06_c04_float_style(
                            flow_axes,
                            crate::geometry::LogicalSizeOf::new(
                                scalar_value(120.0),
                                scalar_value(10.0),
                            ),
                            Float::Right,
                            Clear::None,
                            zero,
                        ),
                    ),
                ],
            );

            assert_eq!(
                public_final_output(&batch, 3).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(S::ZERO, scalar_value(20.0)),
                    crate::geometry::LogicalSizeOf::new(scalar_value(10.0), scalar_value(15.0),),
                ),
                "zero band did not advance to the exact transition for {writing_mode:?} {direction:?}",
            );
            assert_eq!(
                public_final_output(&batch, 4).location,
                fri06_c04_expected_float_location(
                    flow_axes,
                    crate::geometry::LogicalPointOf::new(scalar_value(-20.0), scalar_value(35.0),),
                    crate::geometry::LogicalSizeOf::new(scalar_value(120.0), scalar_value(10.0),),
                ),
                "overwide float did not terminate on line-end for {writing_mode:?} {direction:?}",
            );
        }
    }

    assert_lane::<f32>();
    assert_lane::<f64>();
}

#[test]
fn block_float_contributes_to_intrinsic_width_and_places_from_right_edge() {
    let mut tree = BlockTree::default();
    tree.insert_children(1, vec![2, 3, 4]);
    tree.insert_children(2, vec![]);
    tree.insert_children(3, vec![]);
    tree.insert_children(4, vec![]);
    tree.insert_style(
        1,
        NodeInput {
            display: Display::Block,
            size: Size::new(PreferredSize::AUTO, PreferredSize::px(80.0)),
            border: Edges::all(Length::px(2.0)),
            ..NodeInput::default()
        },
    );
    for node in [2, 3, 4] {
        tree.insert_style(
            node,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::default()
            },
        );
        tree.insert_measure(
            node,
            ComputeOutput::from_sizes(Size::new(50.0, 20.0), Size::new(50.0, 20.0)),
        );
    }

    let output = crate::compute_block(
        &mut tree.borrow_compute(),
        1,
        ComputeInput::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(300.0), Some(200.0)),
            crate::ContainingLayoutContext::new(
                crate::geometry::FlowAxes::new(
                    crate::WritingMode::HorizontalTb,
                    crate::Direction::Ltr,
                ),
                crate::ParentFormattingContext::NoParent,
            ),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Unknown,
                    Size::splat(OrthogonalFallbackLimitOf::Unknown),
                ),
            ),
        ),
    )
    .unwrap();

    assert_eq!(output.size, Size::new(154.0, 80.0));
    assert_eq!(
        tree.layout(2).expect("child layout is staged").location,
        Point::new(102.0, 2.0)
    );
    assert_eq!(
        tree.layout(3).expect("child layout is staged").location,
        Point::new(52.0, 2.0)
    );
    assert_eq!(
        tree.layout(4).expect("child layout is staged").location,
        Point::new(2.0, 2.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(tree.final_layout(1).unwrap().location, Point::new(0.0, 0.0));
    assert_eq!(
        tree.final_layout(2).unwrap().location,
        Point::new(100.0, 0.0)
    );
    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_zero_width_child_fits_between_opposing_floats_above_full_width_float() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(200.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(0.0, 200.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(100.0, 0.0)
    );
}

#[test]
fn block_bfc_overflow_clip_zero_width_child_ignores_float_exclusion_without_clear() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Clip, Overflow::Clip),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(tree.final_layout(3).unwrap().location, Point::new(0.0, 0.0));
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_right_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(150.0, 0.0)
    );
}

#[test]
fn block_bfc_hidden_child_keeps_legacy_center_alignment_without_float_exclusion() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyCenter,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(75.0, 0.0)
    );
}

#[test]
fn block_bfc_float_content_size_height_excludes_container_top_inset() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                border: Edges {
                    top: Length::px(5.0),
                    ..Edges::all(Length::ZERO)
                },
                padding: Edges {
                    top: Length::px(10.0),
                    ..Edges::all(Length::ZERO)
                },
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(30.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(1).unwrap().location,
        Point::new(0.0, 15.0)
    );
    assert_eq!(tree.final_layout(0).unwrap().content_size.height, 40.0);
}

#[test]
fn block_bfc_clear_only_visible_child_keeps_normal_x_while_clearing_y() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                text_align: TextAlign::LegacyRight,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(50.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(150.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Visible, Overflow::Visible),
                size: Size::new(PreferredSize::px(50.0), PreferredSize::px(20.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(20.0), PreferredSize::px(10.0)),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(3).unwrap().location,
        Point::new(150.0, 50.0)
    );
    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(180.0, 70.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_left_sits_below_left_float_row() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Left,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 100.0)
    );
}

#[test]
fn block_bfc_zero_width_child_with_clear_right_sits_below_all_right_floats() {
    let mut tree = crate::test_support::layout_tree::OracleTree::new()
        .children(0, [1, 2, 3, 4])
        .style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(200.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Block,
                float: Float::Right,
                size: Size::new(PreferredSize::percent(1.0), PreferredSize::px(100.0)),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                display: Display::Block,
                clear: crate::Clear::Right,
                overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                size: Size::new(PreferredSize::px(0.0), PreferredSize::AUTO),
                ..NodeInput::DEFAULT
            },
        );

    {
        let root_frame = crate::test_support::requests::NativeRootFrameOf::new(
            {
                let source = tree.borrow_compute();
                let root = source.node_input(0);
                FlowAxes::new(root.writing_mode, root.direction)
            },
            Size::new(None, None),
            OrthogonalFallbackOf::new(
                Size::new(None, None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        );
        compute_root(
            &mut tree.borrow_compute(),
            0,
            root_frame.request(Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            )),
        )
    }
    .unwrap();
    round_layout(&mut tree.borrow_compute(), 0).unwrap();

    assert_eq!(
        tree.final_layout(4).unwrap().location,
        Point::new(0.0, 200.0)
    );
}

#[test]
fn block_ratio_with_a_height_cap_preserves_float_exclusion() {
    let tree = PublicBlockTree::<f32>::default()
        .with_children(0, [1, 2])
        .with_children(1, [])
        .with_children(2, [])
        .with_style(
            0,
            NodeInput {
                display: Display::Block,
                size: Size::new(PreferredSize::px(100.0), PreferredSize::px(80.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            1,
            NodeInput {
                display: Display::Block,
                float: Float::Left,
                box_sizing: BoxSizing::BorderBox,
                aspect_ratio: AspectRatio::new(2.0),
                max_size: Size::new(MaxSize::NONE, MaxSize::px(20.0)),
                ..NodeInput::default()
            },
        )
        .with_style(
            2,
            NodeInput {
                display: Display::Block,
                clear: Clear::Both,
                size: Size::new(PreferredSize::px(30.0), PreferredSize::px(10.0)),
                ..NodeInput::default()
            },
        );
    let batch = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap();
    let floated = public_final_output(&batch, 1);
    let cleared = public_final_output(&batch, 2);
    assert_eq!(
        (floated.size, cleared.location.y),
        (Size::new(40.0, 20.0), 20.0)
    );
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BfcFloatFixture {
    Square,
    InlineContainment,
    InlineContainmentShape,
    EmptyLowerFloat,
    NegativeBlockEndMargin,
    SettledScrollbar,
}

fn assert_bfc_does_not_retry_a_rejected_float_band<S: LayoutScalar>(fixture: BfcFloatFixture) {
    let px = S::from_f64;
    let shape = fixture == BfcFloatFixture::InlineContainmentShape;
    let fixed = |width, height| {
        Size::new(
            PreferredSizeOf::px(px(width)),
            PreferredSizeOf::px(px(height)),
        )
    };
    let float = |side, width| NodeInputOf {
        display: Display::Block,
        float: side,
        float_exclusion: if shape {
            FloatExclusion::Shape
        } else {
            FloatExclusion::MarginBox
        },
        size: fixed(width, 80.0),
        ..NodeInputOf::default()
    };
    let empty_lower = fixture == BfcFloatFixture::EmptyLowerFloat;
    let scrollbar = fixture == BfcFloatFixture::SettledScrollbar;
    let block_end_margin = if fixture == BfcFloatFixture::NegativeBlockEndMargin {
        LengthAutoOf::px(px(-30.0))
    } else {
        LengthAutoOf::ZERO
    };
    for lower_float in [false, true] {
        let mut tree = PublicBlockTree::<S>::default()
            .with_children(
                0,
                if lower_float {
                    vec![1, 2, 3, 4]
                } else {
                    vec![1, 2, 4]
                },
            )
            .with_children(1, [])
            .with_children(2, [])
            .with_children(4, if scrollbar { vec![5, 8] } else { vec![5] })
            .with_children(5, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    block_flow_context: crate::BlockFlowContext::Independent,
                    size: Size::new(PreferredSizeOf::px(px(200.0)), PreferredSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .with_style(1, float(Float::Left, 50.0))
            .with_style(2, float(Float::Right, 50.0))
            .with_style(
                4,
                NodeInputOf {
                    display: Display::Block,
                    block_flow_context: crate::BlockFlowContext::Independent,
                    position: Position::Relative,
                    size_containment: if matches!(
                        fixture,
                        BfcFloatFixture::InlineContainment
                            | BfcFloatFixture::InlineContainmentShape
                    ) {
                        crate::SizeContainment::InlineAxis
                    } else {
                        crate::SizeContainment::None
                    },
                    margin: Edges {
                        bottom: block_end_margin,
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    overflow: if scrollbar {
                        computed_overflow(Overflow::Hidden, Overflow::Auto)
                    } else {
                        computed_overflow(Overflow::Visible, Overflow::Visible)
                    },
                    scrollbar_width: ScrollbarWidthOf::try_new(px(15.0)).unwrap(),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                5,
                NodeInputOf {
                    display: Display::Block,
                    aspect_ratio: crate::AspectRatioOf::new(S::ONE),
                    ..NodeInputOf::default()
                },
            );
        if shape {
            // Clipping this interval to each query's margin box reproduces
            // exactly the rectangular exclusions of the MarginBox control.
            tree = tree.with_shape_provider(ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: px(200.0),
            });
        }
        if lower_float {
            let lower = if empty_lower {
                NodeInputOf {
                    clear: Clear::Both,
                    size: fixed(160.0, 0.0),
                    ..float(Float::Left, 160.0)
                }
            } else {
                float(Float::Left, 160.0)
            };
            tree = tree.with_children(3, []).with_style(3, lower);
        }
        if scrollbar {
            let item = NodeInputOf {
                display: Display::Block,
                size: fixed(48.0, 45.0),
                ..NodeInputOf::default()
            };
            tree = tree
                .with_children(5, [6, 7])
                .with_children(6, [])
                .with_children(7, [])
                .with_children(8, [])
                .with_style(
                    5,
                    NodeInputOf {
                        display: Display::Flex,
                        flex_wrap: FlexWrap::Wrap,
                        ..NodeInputOf::default()
                    },
                )
                .with_style(6, item.clone())
                .with_style(7, item)
                .with_style(
                    8,
                    NodeInputOf {
                        display: Display::Block,
                        position: Position::Absolute,
                        size: fixed(1.0, 200.0),
                        inset: Edges::all(LengthAutoOf::ZERO),
                        ..NodeInputOf::default()
                    },
                );
        }
        let width = NonNegativeFiniteOf::new(px(200.0)).unwrap();
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(width),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let batch = compute_layout(&tree, 0, request).unwrap();
        assert_eq!(public_final_output(&batch, 1).location, Point::ZERO);
        assert_eq!(
            public_final_output(&batch, 2).location,
            Point::new(px(150.0), S::ZERO)
        );
        if lower_float {
            let lower = public_final_output(&batch, 3);
            assert_eq!(lower.location, Point::new(S::ZERO, px(80.0)));
            assert_eq!(lower.size.height, px(if empty_lower { 0.0 } else { 80.0 }));
        }
        let (left, top, side) = if lower_float && !empty_lower {
            (160.0, 80.0, 40.0)
        } else {
            (50.0, 0.0, 100.0)
        };
        let article = public_final_output(&batch, 4);
        let expected_height = if scrollbar { 90.0 } else { side };
        assert_eq!(
            (article.location, article.size),
            (
                Point::new(px(left), px(top)),
                Size::new(px(side), px(expected_height))
            ),
            "{fixture:?}, lower float present: {lower_float}",
        );
        let child = public_final_output(&batch, 5);
        if scrollbar {
            assert_eq!(child.size, Size::new(px(side - 15.0), px(expected_height)));
        } else {
            assert_eq!(child.size, article.size);
        }
    }
}

#[test]
fn bfc_does_not_retry_a_rejected_float_band_f32() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::Square);
}

#[test]
fn bfc_does_not_retry_a_rejected_float_band_f64() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::Square);
}

#[test]
fn inline_size_containment_does_not_retry_a_rejected_float_band() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::InlineContainment);
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::InlineContainment);
}

#[test]
fn inline_size_containment_does_not_retry_an_equivalent_shape_float_band_f32() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::InlineContainmentShape);
}

#[test]
fn inline_size_containment_does_not_retry_an_equivalent_shape_float_band_f64() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::InlineContainmentShape);
}

#[test]
fn bfc_does_not_retry_a_rejected_float_band_with_negative_block_margin() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::NegativeBlockEndMargin);
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::NegativeBlockEndMargin);
}

#[test]
fn bfc_does_not_retry_a_rejected_float_band_with_settled_scrollbar() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::SettledScrollbar);
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::SettledScrollbar);
}

#[test]
fn an_empty_lower_float_band_does_not_displace_the_border_box() {
    assert_bfc_does_not_retry_a_rejected_float_band::<f32>(BfcFloatFixture::EmptyLowerFloat);
    assert_bfc_does_not_retry_a_rejected_float_band::<f64>(BfcFloatFixture::EmptyLowerFloat);
}

#[test]
fn rectangular_float_band_accounts_for_an_inherited_touching_edge() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for floated in [false, true] {
            let mut tree = PublicBlockTree::<S>::default()
                .with_children(0, if floated { vec![1, 2] } else { vec![2] })
                .with_children(2, [3])
                .with_children(3, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(PreferredSizeOf::px(px(80.0)), PreferredSizeOf::AUTO),
                        margin: Edges {
                            left: LengthAutoOf::px(px(20.0)),
                            ..Edges::all(LengthAutoOf::ZERO)
                        },
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    3,
                    NodeInputOf {
                        display: Display::Block,
                        block_flow_context: crate::BlockFlowContext::Independent,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                        margin: Edges {
                            left: LengthAutoOf::px(px(-10.0)),
                            ..Edges::all(LengthAutoOf::ZERO)
                        },
                        ..NodeInputOf::default()
                    },
                );
            if floated {
                tree = tree.with_children(1, []).with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        float: Float::Left,
                        size: Size::new(
                            PreferredSizeOf::px(px(20.0)),
                            PreferredSizeOf::px(px(100.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                );
            }
            let batch = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap();
            let parent = public_final_output(&batch, 2);
            let child = public_final_output(&batch, 3);
            assert_eq!(parent.location, Point::new(px(20.0), S::ZERO));
            assert_eq!(child.size.height, px(10.0));
            assert!(child.size.width > S::ZERO);
            if floated {
                let float = public_final_output(&batch, 1);
                assert_eq!(float.location, Point::ZERO);
                assert!(
                    child.location.x >= S::ZERO || child.location.y >= px(100.0),
                    "border box {:?} overlaps the inherited float",
                    child.location,
                );
            } else {
                assert_eq!(child.location, Point::new(px(-10.0), S::ZERO));
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn rectangular_float_band_resolves_inline_margins_in_their_own_space() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let fixed = |inline, block| {
            Size::new(
                PreferredSizeOf::px(px(inline)),
                PreferredSizeOf::px(px(block)),
            )
        };
        let auto = LengthAutoOf::AUTO;
        let margin = |value| LengthAutoOf::px(px(value));
        let mut differences = Vec::new();
        for (start, end, maximum, x, width, used_start, used_end) in [
            (auto, auto, MaxSizeOf::NONE, 50.0, 150.0, 50.0, 0.0),
            (
                margin(20.0),
                margin(0.0),
                MaxSizeOf::NONE,
                50.0,
                150.0,
                20.0,
                0.0,
            ),
            (
                margin(-20.0),
                margin(0.0),
                MaxSizeOf::NONE,
                50.0,
                150.0,
                -20.0,
                0.0,
            ),
            (
                margin(60.0),
                margin(0.0),
                MaxSizeOf::NONE,
                60.0,
                140.0,
                60.0,
                0.0,
            ),
            (
                margin(0.0),
                margin(20.0),
                MaxSizeOf::NONE,
                50.0,
                130.0,
                0.0,
                20.0,
            ),
            (
                auto,
                auto,
                MaxSizeOf::px(px(100.0)),
                75.0,
                100.0,
                75.0,
                25.0,
            ),
        ] {
            let tree = PublicBlockTree::<S>::default()
                .with_children(0, [1, 2])
                .with_children(1, [])
                .with_children(2, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: fixed(200.0, 100.0),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        float: Float::Left,
                        size: fixed(50.0, 80.0),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        block_flow_context: crate::BlockFlowContext::Independent,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(10.0))),
                        max_size: Size::new(maximum.clone(), MaxSizeOf::NONE),
                        margin: Edges {
                            left: start,
                            right: end,
                            ..Edges::all(LengthAutoOf::ZERO)
                        },
                        ..NodeInputOf::default()
                    },
                );
            let batch = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap();
            let child = public_final_output(&batch, 2);
            let actual = (
                child.location,
                child.size,
                child.margin.left,
                child.margin.right,
            );
            let expected = (
                Point::new(px(x), S::ZERO),
                Size::new(px(width), px(10.0)),
                px(used_start),
                px(used_end),
            );
            if actual != expected {
                differences.push(format!(
                    "{start:?} / {end:?}, {maximum:?}: {actual:?}, expected {expected:?}"
                ));
            }
        }
        assert!(differences.is_empty(), "{}", differences.join("\n"));
    }

    lane::<f32>();
    lane::<f64>();
}

#[test]
fn zero_inline_edge_float_band_keeps_explicit_clearance() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for clear in [Clear::None, Clear::Both] {
            let tree = PublicBlockTree::<S>::default()
                .with_children(0, [1, 2])
                .with_children(1, [])
                .with_children(2, [])
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        size: Size::new(PreferredSizeOf::px(px(100.0)), PreferredSizeOf::AUTO),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        float: Float::Right,
                        size: Size::new(
                            PreferredSizeOf::px(S::ZERO),
                            PreferredSizeOf::px(px(50.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                )
                .with_style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
                        clear,
                        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px(200.0))),
                        margin: Edges {
                            right: LengthAutoOf::px(px(-100.0)),
                            ..Edges::all(LengthAutoOf::ZERO)
                        },
                        ..NodeInputOf::default()
                    },
                );
            let batch = compute_layout(&tree, 0, fri06_c05_provider_request()).unwrap();
            let float = public_final_output(&batch, 1);
            assert_eq!(float.location, Point::new(px(100.0), S::ZERO));
            assert_eq!(float.size, Size::new(S::ZERO, px(50.0)));
            let child = public_final_output(&batch, 2);
            let top = if clear == Clear::Both {
                px(50.0)
            } else {
                S::ZERO
            };
            assert_eq!(
                (child.location, child.size, child.margin.right),
                (
                    Point::new(S::ZERO, top),
                    Size::new(px(200.0), px(200.0)),
                    px(-100.0)
                ),
                "{clear:?}",
            );
        }
    }

    lane::<f32>();
    lane::<f64>();
}

fn assert_native_float_relative_offset_preserves_clearance<S: LayoutScalar>() {
    let px = S::from_f64;
    let viewport = Size::new(px(100.0), px(20.0))
        .map(|extent| Some(NonNegativeFiniteOf::new(extent).unwrap()));
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        viewport,
        OrthogonalFallbackOf::new(
            viewport,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(viewport.map(|extent| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(extent.unwrap()),
            SizingOperation::Normal,
        )
    }));
    let fixed_block = |width, height| NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(px(width)),
            PreferredSizeOf::px(px(height)),
        ),
        ..NodeInputOf::default()
    };
    for (left, top, overflow_bottom) in [(0.0, 0.0, 20.0), (3.0, 14.0, 24.0)] {
        let mut float = fixed_block(20.0, 10.0);
        float.float = Float::Left;
        float.inset.left = LengthAutoOf::px(px(left));
        float.inset.top = LengthAutoOf::px(px(top));
        let mut cleared = fixed_block(20.0, 10.0);
        cleared.clear = Clear::Both;
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(0, fixed_block(100.0, 20.0))
            .with_style(1, float)
            .with_style(2, cleared);
        let batch = compute_layout(&tree, 0, request).unwrap();
        let root = public_final_output(&batch, 0);
        let float = public_final_output(&batch, 1);
        let cleared = public_final_output(&batch, 2);
        assert_eq!(root.size, Size::new(px(100.0), px(20.0)));
        assert_eq!(float.size, Size::new(px(20.0), px(10.0)));
        assert_eq!(float.location, Point::new(px(left), px(top)));
        assert_eq!(cleared.location, Point::new(S::ZERO, px(10.0)));
        let float_entry = batch
            .node_layout(1)
            .map(|layout| match layout {
                crate::NodeLayoutRefOf::SingleBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::NonBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::Fragments(_) => {
                    panic!("continuous fixture unexpectedly fragmented")
                }
            })
            .unwrap();
        let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
            float_entry.geometry_state()
        else {
            panic!("the float has ordinary unrounded geometry");
        };
        assert_eq!(measured_normal.location, Point::ZERO);
        assert_eq!(root.content_size.height, px(overflow_bottom));
    }
}

#[test]
fn native_float_relative_offset_preserves_clearance() {
    assert_native_float_relative_offset_preserves_clearance::<f32>();
    assert_native_float_relative_offset_preserves_clearance::<f64>();
}

fn native_relative_float_range_batch<S: LayoutScalar>(
    width: S,
    direction: Direction,
    overflow: ComputedOverflow,
    float: NodeInputOf<S>,
) -> LayoutResultOf<u32, CompletedLayoutBatchOf<u32, S>, S, ()> {
    let root_size = Size::new(width, S::from_f64(20.0));
    let viewport = root_size.map(|extent| Some(NonNegativeFiniteOf::new(extent).unwrap()));
    let tree = PublicBlockTree::default()
        .with_children(0, [1])
        .with_children(1, [])
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction,
                overflow,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .with_style(1, float);
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, direction),
        viewport,
        OrthogonalFallbackOf::new(
            viewport,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(viewport.map(|extent| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(extent.unwrap()),
            SizingOperation::Normal,
        )
    }));
    compute_layout(&tree, 0, request)
}

fn assert_native_relative_float_keeps_finite_margin_end<S: LayoutScalar>(quarter: S) {
    let px = S::from_f64;
    let width = (px(1.5) + px(3.0) * S::EPSILON) * quarter;
    let mut float = NodeInputOf {
        display: Display::Block,
        float: Float::Left,
        size: Size::new(PreferredSizeOf::px(width), PreferredSizeOf::px(px(10.0))),
        ..NodeInputOf::default()
    };
    float.margin.left = LengthAutoOf::px(quarter / px(8.0));
    float.margin.right = LengthAutoOf::px((px(2.0) - px(5.0) * S::EPSILON) * quarter);
    float.inset.left = LengthAutoOf::px(px(3.0) * quarter / px(8.0));
    let batch = native_relative_float_range_batch(
        px(3.75) * quarter,
        Direction::Ltr,
        ComputedOverflow::VISIBLE,
        float,
    )
    .expect("the retained margin rectangle fits");
    let entries = readback::continuous_custody(&batch, Phase::Unrounded);
    let entry = |node| entries.iter().find(|entry| entry.node() == node).unwrap();
    assert_eq!(entry(1).output().location.x, quarter / px(2.0));
    assert_eq!(entry(1).output().size.width, width);
    assert_eq!(
        entry(0).output().content_size.width,
        fri06_mr02_geometry_error_largest_finite::<S>()
    );
}

#[test]
fn native_relative_float_keeps_finite_margin_end() {
    assert_native_relative_float_keeps_finite_margin_end(2.0_f32.powi(126));
    assert_native_relative_float_keeps_finite_margin_end(2.0_f64.powi(1022));
}

fn assert_native_relative_float_ignores_end_inset<S: LayoutScalar>(quarter: S) {
    let px = S::from_f64;
    let width = px(3.0) * quarter;
    let start = LengthAutoOf::px(quarter / px(8.0));
    let end = LengthAutoOf::percent(px(2.0));
    for (direction, expected_x) in [
        (Direction::Ltr, quarter / px(8.0)),
        (Direction::Rtl, px(2.625) * quarter),
    ] {
        let mut float = NodeInputOf {
            display: Display::Block,
            float: Float::Left,
            size: Size::new(
                PreferredSizeOf::px(quarter / px(4.0)),
                PreferredSizeOf::px(px(10.0)),
            ),
            ..NodeInputOf::default()
        };
        let (left, right) = match direction {
            Direction::Ltr => (start, end),
            Direction::Rtl => (end, start),
        };
        float.inset.left = left;
        float.inset.right = right;
        let mut end_only = float.clone();
        match direction {
            Direction::Ltr => end_only.inset.left = LengthAutoOf::AUTO,
            Direction::Rtl => end_only.inset.right = LengthAutoOf::AUTO,
        }
        let error = native_relative_float_range_batch(
            width,
            direction,
            ComputedOverflow::VISIBLE,
            end_only,
        )
        .unwrap_err();
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(1));
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
                if *value == S::INFINITY
        ));
        let batch =
            native_relative_float_range_batch(width, direction, ComputedOverflow::VISIBLE, float)
                .expect("the ignored end inset cannot prevent placement");
        let entry = batch
            .node_layout(1)
            .map(|layout| match layout {
                crate::NodeLayoutRefOf::SingleBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::NonBox(value) => value.unrounded_entry(),
                crate::NodeLayoutRefOf::Fragments(_) => {
                    panic!("continuous fixture unexpectedly fragmented")
                }
            })
            .unwrap();
        assert_eq!(entry.output().location.x, expected_x);
    }
}

#[test]
fn native_relative_float_ignores_end_inset() {
    assert_native_relative_float_ignores_end_inset(2.0_f32.powi(126));
    assert_native_relative_float_ignores_end_inset(2.0_f64.powi(1022));
}

fn assert_native_relative_float_keeps_visible_border_end<S: LayoutScalar>() {
    let px = S::from_f64;
    let large_margin = px(2.0_f64.powi(60));
    let mut float = NodeInputOf {
        display: Display::Block,
        float: Float::Left,
        size: Size::new(PreferredSizeOf::px(px(10.0)), PreferredSizeOf::px(px(30.0))),
        ..NodeInputOf::default()
    };
    float.margin.top = LengthAutoOf::px(large_margin);
    float.inset.top = LengthAutoOf::px(-large_margin);
    let batch = native_relative_float_range_batch(
        px(100.0),
        Direction::Ltr,
        computed_overflow(Overflow::Hidden, Overflow::Scroll),
        float,
    )
    .unwrap();
    let entries = readback::continuous_custody(&batch, Phase::Unrounded);
    let entry = |node| entries.iter().find(|entry| entry.node() == node).unwrap();
    let child = entry(1);
    let crate::output::GeometryStateOf::OrdinaryUnrounded { measured_normal } =
        child.geometry_state()
    else {
        panic!("the float has ordinary unrounded geometry");
    };
    assert_eq!(measured_normal.location.y, large_margin);
    assert_eq!(child.output().location.y, S::ZERO);
    assert_eq!(
        entry(0)
            .output()
            .scroll_geometry
            .unwrap()
            .physical_range()
            .y()
            .maximum(),
        px(10.0)
    );
}

#[test]
fn native_relative_float_keeps_visible_border_end() {
    assert_native_relative_float_keeps_visible_border_end::<f32>();
    assert_native_relative_float_keeps_visible_border_end::<f64>();
}

fn assert_later_float_avoids_shape_margin_box<S: LayoutScalar>(width: f64, expected: Point<S>) {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let root_size = Size::new(px(width), px(160.0));
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
        OrthogonalFallbackOf::new(
            root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(root_size.map(|v| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(v).unwrap()),
            SizingOperation::Normal,
        )
    }));
    for exclusion in [FloatExclusion::MarginBox, FloatExclusion::Shape] {
        let mut first = fri06_c05_shape_float_style(axes, Float::Left, 80.0, 20.0);
        first.float_exclusion = exclusion;
        let tree = fri06_c05_shape_tree(
            axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: px(40.0),
            },
            [
                (1, first),
                (
                    2,
                    fri06_c05_margin_float_style(axes, Float::Left, 30.0, 10.0),
                ),
            ],
        )
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: root_size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
        let batch = compute_layout(&tree, 0, request).unwrap();
        let first = public_final_output(&batch, 1);
        let later = public_final_output(&batch, 2);
        assert_eq!(public_final_output(&batch, 0).size, root_size);
        assert_eq!(
            (first.location, first.size),
            (Point::ZERO, Size::new(px(80.0), px(20.0)))
        );
        assert_eq!(later.size, Size::new(px(30.0), px(10.0)));
        assert_eq!(
            later.location, expected,
            "{exclusion:?}, root width {width}"
        );
        assert!(
            later.location.x >= first.location.x + first.size.width
                || later.location.y >= first.location.y + first.size.height,
            "later float overlaps the earlier float margin box: {exclusion:?}, root width {width}"
        );
    }
}

#[test]
fn later_float_clears_shape_margin_box_in_narrow_root_f32() {
    assert_later_float_avoids_shape_margin_box::<f32>(100.0, Point::new(0.0, 20.0));
}

#[test]
fn later_float_clears_shape_margin_box_in_narrow_root_f64() {
    assert_later_float_avoids_shape_margin_box::<f64>(100.0, Point::new(0.0, 20.0));
}

#[test]
fn later_float_stays_beside_shape_margin_box_in_wide_root_f32() {
    assert_later_float_avoids_shape_margin_box::<f32>(140.0, Point::new(80.0, 0.0));
}

#[test]
fn later_float_stays_beside_shape_margin_box_in_wide_root_f64() {
    assert_later_float_avoids_shape_margin_box::<f64>(140.0, Point::new(80.0, 0.0));
}

#[test]
fn shape_float_margin_boxes_keep_opposite_sides_clear_and_changing_bands() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalPointOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            for (first_side, second_side, clear, x, y) in [
                (Float::Left, Float::Right, Clear::None, 70.0, 20.0),
                (Float::Right, Float::Left, Clear::None, 0.0, 20.0),
                (Float::Left, Float::Right, Clear::Left, 70.0, 20.0),
                (Float::Right, Float::Left, Clear::Right, 0.0, 20.0),
            ] {
                let mut second = fri06_c05_shape_float_style(axes, second_side, 30.0, 10.0);
                second.clear = clear;
                let tree = fri06_c05_shape_tree(
                    axes,
                    ShapeProviderBehavior::Failure,
                    [
                        (1, fri06_c05_shape_float_style(axes, first_side, 80.0, 20.0)),
                        (2, second),
                    ],
                );
                let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
                let second = public_final_output(&batch, 2);
                assert_eq!(
                    second.size,
                    axes.physical_size(LogicalSizeOf::new(px(30.0), px(10.0)))
                );
                assert_eq!(
                    second.location,
                    fri06_c04_expected_float_location(
                        axes,
                        LogicalPointOf::new(px(x), px(y)),
                        LogicalSizeOf::new(px(30.0), px(10.0))
                    ),
                    "{axes:?} {first_side:?} -> {second_side:?}, clear {clear:?}"
                );
                assert!(tree.shape_queries().is_empty());
            }
            // Float2 starts below float1. Float3 intersects each successive
            // cohort and fits only after both margin-box bottoms have passed.
            let tree = fri06_c05_shape_tree(
                axes,
                ShapeProviderBehavior::Missing,
                [
                    (
                        1,
                        fri06_c05_shape_float_style(axes, Float::Left, 70.0, 10.0),
                    ),
                    (
                        2,
                        fri06_c05_shape_float_style(axes, Float::Right, 50.0, 30.0),
                    ),
                    (
                        3,
                        fri06_c05_shape_float_style(axes, Float::Left, 60.0, 20.0),
                    ),
                ],
            );
            let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
            for (node, x, y, w, h) in [
                (1, 0.0, 0.0, 70.0, 10.0),
                (2, 50.0, 10.0, 50.0, 30.0),
                (3, 0.0, 40.0, 60.0, 20.0),
            ] {
                let output = public_final_output(&batch, node);
                let size = LogicalSizeOf::new(px(w), px(h));
                assert_eq!(output.size, axes.physical_size(size));
                assert_eq!(
                    output.location,
                    fri06_c04_expected_float_location(
                        axes,
                        LogicalPointOf::new(px(x), px(y)),
                        size
                    ),
                    "{axes:?}, node {node}"
                );
            }
            assert!(tree.shape_queries().is_empty());
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn deferred_float_margin_box_keeps_actual_inline_contour_demand() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let tree = fri06_c05_shape_tree(
            axes,
            ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: px(40.0),
            },
            [
                (
                    1,
                    fri06_c05_shape_float_style(axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_margin_float_style(axes, Float::Left, 30.0, 10.0),
                ),
                (3, fri06_c05_atomic_style(axes, 60.0, 10.0)),
            ],
        );
        let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
        assert_eq!(
            public_final_output(&batch, 2).location,
            Point::new(S::ZERO, px(20.0))
        );
        assert_eq!(
            public_final_output(&batch, 3).location,
            Point::new(px(40.0), S::ZERO)
        );
        assert_eq!(
            public_final_output(&batch, 3).size,
            Size::new(px(60.0), px(10.0))
        );
        assert_eq!(
            tree.shape_queries(),
            vec![(
                1,
                FloatExclusionQueryOf::try_new(
                    ScrollRectOf::try_new(Point::ZERO, Size::new(px(80.0), px(20.0))).unwrap(),
                    axes,
                    S::ZERO,
                    px(10.0)
                )
                .unwrap()
            )]
        );
    }
    lane::<f32>();
    lane::<f64>();
}

fn assert_native_float_top_keeps_source_order<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    // The wider control has no deferred float; all three tops stay at zero.
    for (width, second, third) in [
        (
            140.0,
            Point::new(px(80.0), S::ZERO),
            Point::new(px(110.0), S::ZERO),
        ),
        (
            100.0,
            Point::new(S::ZERO, px(20.0)),
            Point::new(px(30.0), px(20.0)),
        ),
    ] {
        let root_size = Size::new(px(width), px(160.0));
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2, 3])
            .with_children(1, [])
            .with_children(2, [])
            .with_children(3, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                fri06_c05_margin_float_style(axes, Float::Left, 80.0, 20.0),
            )
            .with_style(
                2,
                fri06_c05_margin_float_style(axes, Float::Left, 30.0, 10.0),
            )
            .with_style(
                3,
                fri06_c05_margin_float_style(axes, Float::Left, 10.0, 10.0),
            );
        let request = crate::test_support::requests::NativeRootFrameOf::new(
            axes,
            root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
            OrthogonalFallbackOf::new(
                root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(root_size.map(|v| {
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(v).unwrap()),
                SizingOperation::Normal,
            )
        }));
        let batch = compute_layout(&tree, 0, request).unwrap();
        assert_eq!(public_final_output(&batch, 0).size, root_size);
        for (node, w, h) in [(1, 80.0, 20.0), (2, 30.0, 10.0), (3, 10.0, 10.0)] {
            let output = public_final_output(&batch, node);
            assert_eq!(output.size, Size::new(px(w), px(h)));
            assert_eq!(output.margin, Edges::ZERO);
        }
        assert_eq!(public_final_output(&batch, 1).location, Point::ZERO);
        assert_eq!(public_final_output(&batch, 2).location, second);
        assert_eq!(
            public_final_output(&batch, 3).location,
            third,
            "later float must respect earlier outer tops: root width {width}"
        );
        assert!(public_final_output(&batch, 3).location.y >= second.y);
    }
}

#[test]
fn native_float_top_keeps_source_order_f32() {
    assert_native_float_top_keeps_source_order::<f32>();
}

#[test]
fn native_float_top_keeps_source_order_f64() {
    assert_native_float_top_keeps_source_order::<f64>();
}

#[test]
fn native_float_source_floor_maps_sides_and_clear_in_all_flows() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalPointOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            for exclusion in [FloatExclusion::MarginBox, FloatExclusion::Shape] {
                for (first_side, second_side, third_side, clear, third_x, third_y) in [
                    (
                        Float::Left,
                        Float::Left,
                        Float::Left,
                        Clear::None,
                        30.0,
                        20.0,
                    ),
                    (
                        Float::Right,
                        Float::Right,
                        Float::Right,
                        Clear::None,
                        60.0,
                        20.0,
                    ),
                    (
                        Float::Left,
                        Float::Right,
                        Float::Left,
                        Clear::None,
                        0.0,
                        20.0,
                    ),
                    (
                        Float::Right,
                        Float::Left,
                        Float::Right,
                        Clear::None,
                        90.0,
                        20.0,
                    ),
                    (
                        Float::Left,
                        Float::Left,
                        Float::Left,
                        Clear::Left,
                        0.0,
                        30.0,
                    ),
                    (
                        Float::Right,
                        Float::Left,
                        Float::Right,
                        Clear::Right,
                        90.0,
                        20.0,
                    ),
                ] {
                    let floated = |side, width, height, clear| NodeInputOf {
                        clear,
                        float_exclusion: exclusion,
                        ..fri06_c05_margin_float_style(axes, side, width, height)
                    };
                    let tree = fri06_c05_shape_tree(
                        axes,
                        ShapeProviderBehavior::Missing,
                        [
                            (1, floated(first_side, 80.0, 20.0, Clear::None)),
                            (2, floated(second_side, 30.0, 10.0, Clear::None)),
                            (3, floated(third_side, 10.0, 10.0, clear)),
                        ],
                    );
                    let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
                    let first_x = if first_side == Float::Left { 0.0 } else { 20.0 };
                    let second_x = if second_side == Float::Left {
                        0.0
                    } else {
                        70.0
                    };
                    for (node, x, y, w, h) in [
                        (1, first_x, 0.0, 80.0, 20.0),
                        (2, second_x, 20.0, 30.0, 10.0),
                        (3, third_x, third_y, 10.0, 10.0),
                    ] {
                        let output = public_final_output(&batch, node);
                        let size = LogicalSizeOf::new(px(w), px(h));
                        assert_eq!(output.size, axes.physical_size(size));
                        assert_eq!(
                            output.location,
                            fri06_c04_expected_float_location(
                                axes,
                                LogicalPointOf::new(px(x), px(y)),
                                size
                            ),
                            "{axes:?} {exclusion:?}: {first_side:?}/{second_side:?}/{third_side:?}, clear {clear:?}, node {node}"
                        );
                    }
                    assert!(tree.shape_queries().is_empty());
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn native_float_source_floor_retains_expired_zero_area_and_later_source_positions() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalPointOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            for width in [0.0, 30.0] {
                // The cleared float's margin top is20 and its bottom is also20.
                // It excludes no area from the next20..30 band, but its source
                // top still prevents a return to the first float's earlier row.
                let tree = fri06_c05_shape_tree(
                    axes,
                    ShapeProviderBehavior::Missing,
                    [
                        (
                            1,
                            fri06_c05_margin_float_style(axes, Float::Left, 80.0, 20.0),
                        ),
                        (
                            2,
                            NodeInputOf {
                                clear: Clear::Both,
                                ..fri06_c05_margin_float_style(axes, Float::Left, width, 0.0)
                            },
                        ),
                        (
                            3,
                            fri06_c05_margin_float_style(axes, Float::Left, 10.0, 10.0),
                        ),
                    ],
                );
                let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
                for (node, w, h) in [(2, width, 0.0), (3, 10.0, 10.0)] {
                    let size = LogicalSizeOf::new(px(w), px(h));
                    let output = public_final_output(&batch, node);
                    assert_eq!(output.size, axes.physical_size(size));
                    assert_eq!(
                        output.location,
                        fri06_c04_expected_float_location(
                            axes,
                            LogicalPointOf::new(S::ZERO, px(20.0)),
                            size
                        ),
                        "{axes:?}, width {width}, node {node}"
                    );
                }
            }
            let block = NodeInputOf {
                display: Display::Block,
                writing_mode,
                direction,
                size: axes
                    .physical_size(LogicalSizeOf::new(px(100.0), px(40.0)))
                    .map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            };
            let tree = fri06_c05_shape_tree(
                axes,
                ShapeProviderBehavior::Missing,
                [
                    (
                        1,
                        fri06_c05_margin_float_style(axes, Float::Left, 80.0, 20.0),
                    ),
                    (
                        2,
                        fri06_c05_margin_float_style(axes, Float::Left, 30.0, 10.0),
                    ),
                    (4, block),
                    (
                        3,
                        fri06_c05_margin_float_style(axes, Float::Left, 10.0, 10.0),
                    ),
                ],
            );
            let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
            // Both earlier floats have ended. Actual source position40 wins
            // over their retained top20; floats did not advance the block cursor.
            let size = LogicalSizeOf::new(px(10.0), px(10.0));
            assert_eq!(
                public_final_output(&batch, 3).location,
                fri06_c04_expected_float_location(
                    axes,
                    LogicalPointOf::new(S::ZERO, px(40.0)),
                    size
                )
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn native_float_source_floor_uses_translated_shared_inherited_ledger() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let tree = fri06_c05_shape_tree(
            axes,
            ShapeProviderBehavior::Missing,
            [
                (
                    1,
                    fri06_c05_margin_float_style(axes, Float::Left, 80.0, 20.0),
                ),
                (
                    2,
                    fri06_c05_margin_float_style(axes, Float::Left, 30.0, 10.0),
                ),
                (
                    4,
                    NodeInputOf {
                        display: Display::Block,
                        block_flow_context: crate::BlockFlowContext::Normal,
                        ..NodeInputOf::default()
                    },
                ),
            ],
        )
        .with_children(4, [3])
        .with_children(3, [])
        .with_style(
            3,
            fri06_c05_margin_float_style(axes, Float::Left, 10.0, 10.0),
        )
        .with_style(
            0,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(
                    PreferredSizeOf::px(px(100.0)),
                    PreferredSizeOf::px(px(160.0)),
                ),
                box_sizing: BoxSizing::BorderBox,
                padding: Edges {
                    top: LengthOf::px(px(10.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );
        let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
        assert_eq!(
            public_final_output(&batch, 1).location,
            Point::new(S::ZERO, px(10.0))
        );
        assert_eq!(
            public_final_output(&batch, 2).location,
            Point::new(S::ZERO, px(30.0))
        );
        assert_eq!(
            public_final_output(&batch, 4).location,
            Point::new(S::ZERO, px(10.0))
        );
        assert_eq!(
            public_final_output(&batch, 3).location,
            Point::new(px(30.0), px(20.0))
        );
        assert_eq!(
            public_final_output(&batch, 3).size,
            Size::new(px(10.0), px(10.0))
        );
    }
    lane::<f32>();
    lane::<f64>();
}

fn assert_table_border_avoids_full_width_float<S: LayoutScalar>() {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let root_size = Size::new(px(100.0), px(100.0));
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
        OrthogonalFallbackOf::new(
            root_size.map(|v| Some(NonNegativeFiniteOf::new(v).unwrap())),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(root_size.map(|v| {
        AxisSizingRequestOf::new(
            AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(v).unwrap()),
            SizingOperation::Normal,
        )
    }));
    for (role, table, replaced, context, top) in [
        (
            "ordinary",
            false,
            false,
            crate::BlockFlowContext::Normal,
            0.0,
        ),
        (
            "replaced",
            false,
            true,
            crate::BlockFlowContext::Normal,
            20.0,
        ),
        (
            "independent",
            false,
            false,
            crate::BlockFlowContext::Independent,
            20.0,
        ),
        ("table", true, false, crate::BlockFlowContext::Normal, 20.0),
    ] {
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [])
            .with_style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: root_size.map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                },
            )
            .with_style(
                1,
                fri06_c05_margin_float_style(axes, Float::Left, 100.0, 20.0),
            )
            .with_style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    block_flow_context: context,
                    item_is_table: table,
                    item_is_replaced: replaced,
                    size: Size::new(PreferredSizeOf::px(px(50.0)), PreferredSizeOf::px(px(10.0))),
                    ..NodeInputOf::default()
                },
            );
        let input = tree.node_input(2);
        assert_eq!(input.item_is_table, table);
        assert_eq!(input.item_is_replaced, replaced);
        assert_eq!(input.block_flow_context, context);
        assert_eq!(input.float, Float::None);
        assert_eq!(input.clear, Clear::None);
        assert_eq!(input.overflow, ComputedOverflow::VISIBLE);
        let batch = compute_layout(&tree, 0, request).unwrap();
        assert_eq!(public_final_output(&batch, 0).size, root_size);
        let floated = public_final_output(&batch, 1);
        assert_eq!(floated.location, Point::ZERO);
        assert_eq!(floated.size, Size::new(px(100.0), px(20.0)));
        assert_eq!(floated.margin, Edges::ZERO);
        let subject = public_final_output(&batch, 2);
        assert_eq!(subject.size, Size::new(px(50.0), px(10.0)), "{role}");
        assert_eq!(subject.margin, Edges::ZERO, "{role}");
        assert_eq!(
            subject.location,
            Point::new(S::ZERO, px(top)),
            "{role} table={table}, replaced={replaced}, context={context:?}"
        );
        if role != "ordinary" {
            assert!(
                subject.location.y >= floated.location.y + floated.size.height,
                "{role} border box must avoid the full-width float margin box"
            );
        }
    }
}

#[test]
fn table_border_avoids_full_width_float_with_normal_context_f32() {
    assert_table_border_avoids_full_width_float::<f32>();
}

#[test]
fn table_border_avoids_full_width_float_with_normal_context_f64() {
    assert_table_border_avoids_full_width_float::<f64>();
}

#[test]
fn table_float_collision_uses_fixed_margin_boxes_in_all_flows() {
    fn lane<S: LayoutScalar>() {
        use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf};
        let px = S::from_f64;
        for (writing_mode, direction) in all_writing_mode_directions() {
            let axes = FlowAxes::new(writing_mode, direction);
            for exclusion in [FloatExclusion::MarginBox, FloatExclusion::Shape] {
                for (side, width, float_margin, table_margin, clear, x, y) in [
                    (Float::Left, 100.0, 0.0, 0.0, Clear::None, 0.0, 20.0),
                    (Float::Right, 100.0, 0.0, 0.0, Clear::None, 0.0, 20.0),
                    (Float::Left, 40.0, 0.0, 0.0, Clear::None, 40.0, 0.0),
                    (Float::Right, 40.0, 0.0, 0.0, Clear::None, 0.0, 0.0),
                    (Float::Left, 40.0, 0.0, 0.0, Clear::Left, 0.0, 20.0),
                    (Float::Right, 40.0, 0.0, 0.0, Clear::Right, 0.0, 20.0),
                    (Float::Left, 40.0, 0.0, 5.0, Clear::None, 45.0, 0.0),
                    (Float::Left, 30.0, 10.0, 0.0, Clear::None, 40.0, 0.0),
                    (Float::Left, 70.0, 10.0, 0.0, Clear::None, 0.0, 20.0),
                ] {
                    let mut floated = fri06_c05_margin_float_style(axes, side, width, 20.0);
                    floated.float_exclusion = exclusion;
                    floated.margin = axes
                        .physical_edges(LogicalEdgesOf::new(
                            S::ZERO,
                            px(float_margin),
                            S::ZERO,
                            S::ZERO,
                        ))
                        .map(LengthAutoOf::px);
                    let size = LogicalSizeOf::new(px(50.0), px(10.0));
                    let tree = fri06_c05_shape_tree(
                        axes,
                        ShapeProviderBehavior::Failure,
                        [
                            (1, floated),
                            (
                                2,
                                NodeInputOf {
                                    display: Display::Block,
                                    writing_mode,
                                    direction,
                                    item_is_table: true,
                                    block_flow_context: crate::BlockFlowContext::Normal,
                                    size: axes.physical_size(size).map(PreferredSizeOf::px),
                                    margin: axes
                                        .physical_edges(LogicalEdgesOf::new(
                                            px(table_margin),
                                            px(table_margin),
                                            S::ZERO,
                                            S::ZERO,
                                        ))
                                        .map(LengthAutoOf::px),
                                    clear,
                                    ..NodeInputOf::default()
                                },
                            ),
                        ],
                    );
                    let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
                    let table = public_final_output(&batch, 2);
                    assert_eq!(table.size, axes.physical_size(size));
                    assert_eq!(
                        table.location,
                        fri06_c04_expected_float_location(
                            axes,
                            LogicalPointOf::new(px(x), px(y)),
                            size
                        ),
                        "{axes:?} {exclusion:?} {side:?} width {width}+{float_margin}, table margin {table_margin}, clear {clear:?}"
                    );
                    assert_eq!(
                        tree.node_input(2).block_flow_context,
                        crate::BlockFlowContext::Normal
                    );
                    assert!(tree.shape_queries().is_empty());
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn table_normal_context_keeps_legacy_alignment_without_float_entries() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for direction in [Direction::Ltr, Direction::Rtl] {
            let axes = FlowAxes::new(WritingMode::HorizontalTb, direction);
            for (align, x) in [
                (TextAlign::LegacyLeft, 0.0),
                (TextAlign::LegacyCenter, 25.0),
                (TextAlign::LegacyRight, 50.0),
            ] {
                let tree = fri06_c05_shape_tree(
                    axes,
                    ShapeProviderBehavior::Failure,
                    [(
                        2,
                        NodeInputOf {
                            display: Display::Block,
                            writing_mode: WritingMode::HorizontalTb,
                            direction,
                            item_is_table: true,
                            size: Size::new(
                                PreferredSizeOf::px(px(50.0)),
                                PreferredSizeOf::px(px(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    )],
                )
                .with_style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        writing_mode: WritingMode::HorizontalTb,
                        direction,
                        text_align: align,
                        size: Size::new(
                            PreferredSizeOf::px(px(100.0)),
                            PreferredSizeOf::px(px(160.0)),
                        ),
                        ..NodeInputOf::default()
                    },
                );
                let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
                let table = public_final_output(&batch, 2);
                assert_eq!(
                    table.location,
                    Point::new(px(x), S::ZERO),
                    "{direction:?} {align:?}"
                );
                assert_eq!(table.size, Size::new(px(50.0), px(10.0)));
                assert_eq!(
                    tree.node_input(2).block_flow_context,
                    crate::BlockFlowContext::Normal
                );
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[derive(Clone, Copy, Debug)]
enum TableWrapperContextCase {
    ExternalClearance,
    FloatHeight,
    TopMargin,
    BottomMargin,
    FloatExport,
}

fn assert_table_wrapper_context<S: LayoutScalar>(case: TableWrapperContextCase) {
    let px = S::from_f64;
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let fixed = |width, height| NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(px(width)),
            PreferredSizeOf::px(px(height)),
        ),
        ..NodeInputOf::default()
    };
    for context in [
        crate::BlockFlowContext::Independent,
        crate::BlockFlowContext::Normal,
    ] {
        let mut table = fixed(50.0, 10.0);
        table.item_is_table = true;
        table.block_flow_context = context;
        let mut child = fixed(50.0, 10.0);
        let mut root_children = vec![2];
        match case {
            TableWrapperContextCase::ExternalClearance => {
                root_children.insert(0, 1);
                child.clear = Clear::Left;
            }
            TableWrapperContextCase::FloatHeight => {
                table.size.height = PreferredSizeOf::AUTO;
                child = fri06_c05_margin_float_style(axes, Float::Left, 30.0, 20.0);
            }
            TableWrapperContextCase::TopMargin => {
                table.size.height = PreferredSizeOf::AUTO;
                table.margin.top = LengthAutoOf::px(px(5.0));
                child.margin.top = LengthAutoOf::px(px(20.0));
            }
            TableWrapperContextCase::BottomMargin => {
                table.size.height = PreferredSizeOf::AUTO;
                table.margin.bottom = LengthAutoOf::px(px(7.0));
                child.margin.bottom = LengthAutoOf::px(px(30.0));
                root_children.push(4);
            }
            TableWrapperContextCase::FloatExport => {
                child = fri06_c05_margin_float_style(axes, Float::Left, 30.0, 40.0);
                root_children.push(4);
            }
        }
        let mut tree = PublicBlockTree::default()
            .with_children(0, root_children)
            .with_children(2, [3])
            .with_children(3, [])
            .with_style(0, fixed(100.0, 160.0))
            .with_style(2, table)
            .with_style(3, child);
        if matches!(case, TableWrapperContextCase::ExternalClearance) {
            tree = tree.with_children(1, []).with_style(
                1,
                fri06_c05_margin_float_style(axes, Float::Left, 100.0, 20.0),
            );
        }
        if matches!(
            case,
            TableWrapperContextCase::BottomMargin | TableWrapperContextCase::FloatExport
        ) {
            let mut following = fixed(100.0, 10.0);
            if matches!(case, TableWrapperContextCase::FloatExport) {
                following.clear = Clear::Left;
            }
            tree = tree.with_children(4, []).with_style(4, following);
        }
        assert!(tree.node_input(2).item_is_table);
        assert_eq!(tree.node_input(2).block_flow_context, context);
        let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
        let table = public_final_output(&batch, 2);
        let child = public_final_output(&batch, 3);
        let label = format!("{case:?}, {context:?}");
        assert_eq!(table.size.width, px(50.0), "{label}");
        match case {
            TableWrapperContextCase::ExternalClearance => {
                let external = public_final_output(&batch, 1);
                assert_eq!(
                    (external.location, external.size),
                    (Point::ZERO, Size::new(px(100.0), px(20.0))),
                    "{label}"
                );
                assert_eq!(table.size, Size::new(px(50.0), px(10.0)), "{label}");
                assert_eq!(table.location, Point::new(S::ZERO, px(20.0)), "{label}");
                assert_eq!(child.size, Size::new(px(50.0), px(10.0)), "{label}");
                assert_eq!(child.margin, Edges::ZERO, "{label}");
                assert_eq!(
                    child.location,
                    Point::ZERO,
                    "parent float must not clear the wrapper's child again: {label}"
                );
                assert_eq!(table.location.y + child.location.y, px(20.0), "{label}");
            }
            TableWrapperContextCase::FloatHeight => {
                assert_eq!(
                    (child.location, child.size),
                    (Point::ZERO, Size::new(px(30.0), px(20.0))),
                    "{label}"
                );
                assert_eq!(table.location, Point::ZERO, "{label}");
                assert_eq!(
                    table.size.height,
                    px(20.0),
                    "wrapper owns its internal float's height: {label}"
                );
            }
            TableWrapperContextCase::TopMargin => {
                assert_eq!(child.size, Size::new(px(50.0), px(10.0)), "{label}");
                assert_eq!(table.margin.top, px(5.0), "{label}");
                assert_eq!(child.margin.top, px(20.0), "{label}");
                assert_eq!(
                    table.location,
                    Point::new(S::ZERO, px(5.0)),
                    "interior top margin must not escape the wrapper: {label}"
                );
                assert_eq!(child.location, Point::new(S::ZERO, px(20.0)), "{label}");
                assert_eq!(table.size.height, px(30.0), "{label}");
            }
            TableWrapperContextCase::BottomMargin => {
                assert_eq!(
                    (child.location, child.size),
                    (Point::ZERO, Size::new(px(50.0), px(10.0))),
                    "{label}"
                );
                assert_eq!(table.location, Point::ZERO, "{label}");
                assert_eq!(
                    table.size.height,
                    px(40.0),
                    "interior bottom margin remains inside wrapper height: {label}"
                );
                assert_eq!(table.margin.bottom, px(7.0), "{label}");
                assert_eq!(
                    public_final_output(&batch, 4).location,
                    Point::new(S::ZERO, px(47.0)),
                    "outer margin remains applicable: {label}"
                );
            }
            TableWrapperContextCase::FloatExport => {
                assert_eq!(
                    (table.location, table.size),
                    (Point::ZERO, Size::new(px(50.0), px(10.0))),
                    "{label}"
                );
                assert_eq!(
                    (child.location, child.size),
                    (Point::ZERO, Size::new(px(30.0), px(40.0))),
                    "{label}"
                );
                assert_eq!(
                    public_final_output(&batch, 4).size,
                    Size::new(px(100.0), px(10.0)),
                    "{label}"
                );
                assert_eq!(
                    public_final_output(&batch, 4).location,
                    Point::new(S::ZERO, px(10.0)),
                    "parent clear must not see the wrapper's internal float: {label}"
                );
            }
        }
    }
}

#[test]
fn table_wrapper_context_excludes_external_clearance_f32() {
    assert_table_wrapper_context::<f32>(TableWrapperContextCase::ExternalClearance);
}
#[test]
fn table_wrapper_context_excludes_external_clearance_f64() {
    assert_table_wrapper_context::<f64>(TableWrapperContextCase::ExternalClearance);
}
#[test]
fn table_wrapper_context_owns_internal_float_auto_height_f32() {
    assert_table_wrapper_context::<f32>(TableWrapperContextCase::FloatHeight);
}
#[test]
fn table_wrapper_context_owns_internal_float_auto_height_f64() {
    assert_table_wrapper_context::<f64>(TableWrapperContextCase::FloatHeight);
}
#[test]
fn table_wrapper_context_blocks_top_child_margin_f32() {
    assert_table_wrapper_context::<f32>(TableWrapperContextCase::TopMargin);
}
#[test]
fn table_wrapper_context_blocks_top_child_margin_f64() {
    assert_table_wrapper_context::<f64>(TableWrapperContextCase::TopMargin);
}
#[test]
fn table_wrapper_context_blocks_bottom_child_margin_f32() {
    assert_table_wrapper_context::<f32>(TableWrapperContextCase::BottomMargin);
}
#[test]
fn table_wrapper_context_blocks_bottom_child_margin_f64() {
    assert_table_wrapper_context::<f64>(TableWrapperContextCase::BottomMargin);
}
#[test]
fn table_wrapper_context_does_not_export_internal_float_f32() {
    assert_table_wrapper_context::<f32>(TableWrapperContextCase::FloatExport);
}
#[test]
fn table_wrapper_context_does_not_export_internal_float_f64() {
    assert_table_wrapper_context::<f64>(TableWrapperContextCase::FloatExport);
}

#[test]
fn table_wrapper_context_excludes_parent_floats_in_all_flows() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        for writing_mode in [
            WritingMode::HorizontalTb,
            WritingMode::VerticalRl,
            WritingMode::VerticalLr,
            WritingMode::SidewaysRl,
            WritingMode::SidewaysLr,
        ] {
            for direction in [Direction::Ltr, Direction::Rtl] {
                let axes = FlowAxes::new(writing_mode, direction);
                let fixed = |inline, block| NodeInputOf {
                    display: Display::Block,
                    writing_mode,
                    direction,
                    size: axes
                        .physical_size(crate::geometry::LogicalSizeOf::new(px(inline), px(block)))
                        .map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                };
                for exclusion in [FloatExclusion::MarginBox, FloatExclusion::Shape] {
                    for atomic in [false, true] {
                        let mut external =
                            fri06_c05_margin_float_style(axes, Float::Left, 100.0, 20.0);
                        external.float_exclusion = exclusion;
                        let child = if atomic {
                            fri06_c05_atomic_style(axes, 50.0, 10.0)
                        } else {
                            NodeInputOf {
                                clear: Clear::Left,
                                ..fixed(50.0, 10.0)
                            }
                        };
                        let tree = PublicBlockTree::default()
                            .with_children(0, [1, 2])
                            .with_children(1, [])
                            .with_children(2, [3])
                            .with_children(3, [])
                            .with_style(0, fixed(100.0, 160.0))
                            .with_style(1, external)
                            .with_style(
                                2,
                                NodeInputOf {
                                    item_is_table: true,
                                    block_flow_context: crate::BlockFlowContext::Normal,
                                    ..fixed(50.0, 10.0)
                                },
                            )
                            .with_style(3, child)
                            .with_shape_provider(ShapeProviderBehavior::Failure);
                        let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes))
                            .expect("wrapper contents cannot demand the parent's shape");
                        let size = crate::geometry::LogicalSizeOf::new(px(50.0), px(10.0));
                        let table = public_final_output(&batch, 2);
                        assert_eq!(table.size, axes.physical_size(size));
                        assert_eq!(
                            table.location,
                            fri06_c04_expected_float_location(
                                axes,
                                crate::geometry::LogicalPointOf::new(S::ZERO, px(20.0)),
                                size
                            ),
                            "{axes:?}, {exclusion:?}, atomic {atomic}"
                        );
                        let child = public_final_output(&batch, 3);
                        assert_eq!(child.size, table.size);
                        assert_eq!(child.location, Point::ZERO);
                        assert!(tree.shape_queries().is_empty());
                        assert_eq!(
                            tree.node_input(2).block_flow_context,
                            crate::BlockFlowContext::Normal
                        );
                    }
                }
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn table_wrapper_context_keeps_local_shape_demand_and_error_identity() {
    fn lane<S: LayoutScalar>() {
        let px = S::from_f64;
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let fixed = |width, height| NodeInputOf {
            display: Display::Block,
            size: Size::new(
                PreferredSizeOf::px(px(width)),
                PreferredSizeOf::px(px(height)),
            ),
            ..NodeInputOf::default()
        };
        let tree = PublicBlockTree::default()
            .with_children(0, [1, 2])
            .with_children(1, [])
            .with_children(2, [3, 4])
            .with_children(3, [])
            .with_children(4, [])
            .with_style(0, fixed(100.0, 160.0))
            .with_style(
                1,
                fri06_c05_shape_float_style(axes, Float::Left, 100.0, 20.0),
            )
            .with_style(
                2,
                NodeInputOf {
                    item_is_table: true,
                    ..fixed(100.0, 20.0)
                },
            )
            .with_style(
                3,
                fri06_c05_shape_float_style(axes, Float::Left, 60.0, 20.0),
            )
            .with_style(4, fri06_c05_atomic_style(axes, 40.0, 10.0))
            .with_shape_provider(ShapeProviderBehavior::Interval {
                minimum: S::ZERO,
                maximum: px(20.0),
            });
        let batch = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap();
        assert_eq!(
            public_final_output(&batch, 2).location,
            Point::new(S::ZERO, px(20.0))
        );
        assert_eq!(public_final_output(&batch, 3).location, Point::ZERO);
        assert_eq!(
            public_final_output(&batch, 4).location,
            Point::new(px(20.0), S::ZERO)
        );
        assert_eq!(
            public_final_output(&batch, 4).size,
            Size::new(px(40.0), px(10.0))
        );
        assert!(!tree.shape_queries().is_empty());
        assert!(tree.shape_queries().iter().all(|(node, _)| *node == 3));
        let tree = tree.with_shape_provider(ShapeProviderBehavior::Failure);
        let error = compute_layout(&tree, 0, fri06_c05_shape_request(axes)).unwrap_err();
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 2,
                subject: 3
            }
        );
        assert_eq!(error.operation(), LayoutOperation::FloatExclusionQuery);
        assert!(matches!(error.kind(), LayoutErrorKindOf::Measurement(())));
        // The failed recomputation cannot mutate the earlier completed batch.
        assert_eq!(
            public_final_output(&batch, 4).location,
            Point::new(px(20.0), S::ZERO)
        );
    }
    lane::<f32>();
    lane::<f64>();
}
