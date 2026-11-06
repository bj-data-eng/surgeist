use super::*;

fn layout_children<S: LayoutScalar, const N: usize>(
    axes: FlexAxes,
    flow: FlowAxes,
    main: f64,
    children: [NodeInputOf<S>; N],
) -> [NodeOutputOf<S>; N] {
    let size = axes.size_from_main_cross(S::from_f64(main), S::from_f64(100.0));
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, 1..=u32::try_from(N).unwrap())
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: flow.writing_mode(),
                direction: flow.direction(),
                flex_direction: if axes.main_logical_axis() == LogicalAxis::Inline {
                    FlexDirection::Row
                } else {
                    FlexDirection::Column
                },
                align_items: Some(AlignItems::Baseline),
                size: size.map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        );
    for (index, child) in children.into_iter().enumerate() {
        tree = tree.style(u32::try_from(index + 1).unwrap(), child);
    }
    let batch = compute_layout(
        &tree,
        0,
        crate::test_support::requests::NativeRootFrameOf::new(
            FlowAxes::new(
                tree.node_input(0).writing_mode,
                tree.node_input(0).direction,
            ),
            size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
            OrthogonalFallbackOf::new(
                size.map(|value| Some(NonNegativeFiniteOf::new(value).unwrap())),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        )
        .request(size.map(|value| {
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(value).unwrap()),
                SizingOperation::Normal,
            )
        })),
    )
    .expect("valid flex layout succeeds");
    core::array::from_fn(|index| {
        let node = u32::try_from(index + 1).unwrap();
        batch
            .unrounded_entries()
            .iter()
            .find(|entry| entry.node() == node)
            .expect("the child has staged output")
            .output()
    })
}

fn assert_inner_basis_shrink<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let axes = FlexAxes::new(flow, direction, FlexWrap::NoWrap);
        for (box_sizing, basis, padding, border) in [
            (BoxSizing::ContentBox, 100.0, 50.0, 0.0),
            (BoxSizing::BorderBox, 150.0, 50.0, 0.0),
            (BoxSizing::ContentBox, 100.0, 30.0, 20.0),
        ] {
            let mut first = NodeInputOf {
                box_sizing,
                flex_basis: FlexBasisOf::px(S::from_f64(basis)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            axes.set_main_start_edge(&mut first.padding, LengthOf::px(S::from_f64(padding)));
            axes.set_main_end_edge(&mut first.border, LengthOf::px(S::from_f64(border)));
            let second = NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            let outputs = layout_children(axes, flow, 200.0, [first, second]);
            assert_eq!(
                outputs.map(|output| axes.main_size(output.size)),
                [S::from_f64(125.0), S::from_f64(75.0)],
                "{direction:?} {box_sizing:?}: padding and border do not carry shrink weight"
            );
        }

        let mut first = NodeInputOf {
            box_sizing: BoxSizing::ContentBox,
            flex_basis: FlexBasisOf::px(S::ZERO),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        axes.set_main_start_edge(&mut first.padding, LengthOf::px(S::from_f64(50.0)));
        let second = NodeInputOf {
            flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        let outputs = layout_children(axes, flow, 100.0, [first, second]);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(50.0), S::from_f64(50.0)],
            "a zero inner basis preserves the padding floor without shrinking"
        );
    }
}

#[test]
fn flex_contract_inner_basis_shrink_f32() {
    assert_inner_basis_shrink::<f32>();
}

#[test]
fn flex_contract_inner_basis_shrink_f64() {
    assert_inner_basis_shrink::<f64>();
}

fn assert_freeze_before_initial_space<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let axes = FlexAxes::new(flow, direction, FlexWrap::NoWrap);
        let growing = [
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(100.0)),
                max_size: axes.with_main_size(
                    Size::new(MaxSizeOf::NONE, MaxSizeOf::NONE),
                    MaxSizeOf::px(S::from_f64(50.0)),
                ),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(50.0)),
                flex_grow: FlexGrowOf::try_new(S::from_f64(0.5)).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        ];
        let outputs = layout_children(axes, flow, 200.0, growing);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(50.0), S::from_f64(100.0)],
            "fractional growth uses the frozen target in initial free space"
        );
        let shrinking = [
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(50.0)),
                min_size: axes.with_main_size(
                    Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                    MinSizeOf::px(S::from_f64(100.0)),
                ),
                ..NodeInputOf::default()
            },
            NodeInputOf {
                flex_basis: FlexBasisOf::px(S::from_f64(150.0)),
                flex_shrink: FlexShrinkOf::try_new(S::from_f64(0.5)).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            },
        ];
        let outputs = layout_children(axes, flow, 150.0, shrinking);
        assert_eq!(
            outputs.map(|output| axes.main_size(output.size)),
            [S::from_f64(100.0), S::from_f64(100.0)],
            "fractional shrink uses the frozen target in initial free space"
        );
    }
}

#[test]
fn flex_contract_initial_free_space_f32() {
    assert_freeze_before_initial_space::<f32>();
}

#[test]
fn flex_contract_initial_free_space_f64() {
    assert_freeze_before_initial_space::<f64>();
}

fn assert_auto_margin_baseline_exclusion<S: LayoutScalar>() {
    for writing_mode in [WritingMode::HorizontalTb, WritingMode::VerticalLr] {
        let flow = FlowAxes::new(writing_mode, Direction::Ltr);
        let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
        for auto_start in [true, false] {
            for (first_cross, second_cross) in [(100.0, 20.0), (20.0, 100.0)] {
                let mut children = [first_cross, second_cross].map(|cross| NodeInputOf {
                    writing_mode,
                    size: axes
                        .size_from_main_cross(S::from_f64(50.0), S::from_f64(cross))
                        .map(PreferredSizeOf::px),
                    ..NodeInputOf::default()
                });
                if auto_start {
                    axes.set_cross_start_edge(&mut children[0].margin, LengthAutoOf::AUTO);
                } else {
                    axes.set_cross_end_edge(&mut children[0].margin, LengthAutoOf::AUTO);
                }
                let outputs = layout_children(axes, flow, 200.0, children);
                assert_eq!(
                    axes.cross_point(outputs[1].location),
                    S::ZERO,
                    "{writing_mode:?}: the auto-margin item does not contribute to the baseline group"
                );
                let first_offset = if auto_start { 100.0 - first_cross } else { 0.0 };
                assert_eq!(
                    axes.cross_point(outputs[0].location),
                    S::from_f64(first_offset),
                    "{writing_mode:?}: baseline alignment does not override the resolved auto margin"
                );
            }
        }
    }
}

#[test]
fn flex_contract_auto_margin_baseline_f32() {
    assert_auto_margin_baseline_exclusion::<f32>();
}

#[test]
fn flex_contract_auto_margin_baseline_f64() {
    assert_auto_margin_baseline_exclusion::<f64>();
}

fn assert_auto_margin_does_not_expand_baseline_line<S: LayoutScalar>() {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let mut tree = OracleTreeOf::new().children(0, [1, 2]).style(
        0,
        NodeInputOf {
            display: Display::Flex,
            size: Size::new(
                PreferredSizeOf::px(S::from_f64(200.0)),
                PreferredSizeOf::AUTO,
            ),
            align_items: Some(AlignItems::Baseline),
            ..NodeInputOf::default()
        },
    );
    for (node, height, baseline) in [(1, 100.0, 100.0), (2, 20.0, 5.0)] {
        let size = Size::new(S::from_f64(50.0), S::from_f64(height));
        tree = tree
            .children(node, [])
            .style(
                node,
                NodeInputOf {
                    size: size.map(PreferredSizeOf::px),
                    margin: Edges {
                        top: if node == 1 {
                            LengthAutoOf::AUTO
                        } else {
                            LengthAutoOf::ZERO
                        },
                        ..Edges::all(LengthAutoOf::ZERO)
                    },
                    ..NodeInputOf::default()
                },
            )
            .measure(
                node,
                ComputeOutputOf::from_sizes_and_baselines(
                    size,
                    size,
                    BaselinesOf::first(Point::new(None, Some(S::from_f64(baseline)))),
                ),
            );
    }
    let output = compute_flex(
        &mut tree.borrow_compute(),
        0,
        ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(200.0)), None),
            ContainingLayoutContext::new(flow, ParentFormattingContext::NoParent),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(200.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    ),
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
    .expect("baseline line layout succeeds");
    assert_eq!(
        output.size.height,
        S::from_f64(100.0),
        "an excluded baseline must not expand the line to 115px"
    );
    assert_eq!(tree.layout(2).unwrap().location.y, S::ZERO);
}

#[test]
fn flex_contract_auto_margin_line_size_f32() {
    assert_auto_margin_does_not_expand_baseline_line::<f32>();
}

#[test]
fn flex_contract_auto_margin_line_size_f64() {
    assert_auto_margin_does_not_expand_baseline_line::<f64>();
}

fn assert_flex_extreme_finite_factors_complete<S: LayoutScalar>() {
    let px = S::from_f64;
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
    let grow = |basis: [f64; 2], factors: [S; 2]| {
        let children = core::array::from_fn(|index| NodeInputOf {
            flex_basis: FlexBasisOf::px(px(basis[index])),
            flex_grow: FlexGrowOf::try_new(factors[index]).expect("finite grow factor"),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        });
        layout_children(axes, flow, 100.0, children).map(|output| output.size.width)
    };
    let shrink = |available, factors: [S; 2]| {
        let children = core::array::from_fn(|index| NodeInputOf {
            flex_basis: FlexBasisOf::px(px(100.0)),
            flex_shrink: FlexShrinkOf::try_new(factors[index]).expect("finite shrink factor"),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        });
        layout_children(axes, flow, available, children).map(|output| output.size.width)
    };

    assert_eq!(grow([10.0, 10.0], [S::ONE, S::ONE]), [px(50.0), px(50.0)]);
    assert_eq!(grow([10.0, 0.0], [largest, S::ZERO]), [px(100.0), S::ZERO]);
    assert_eq!(grow([10.0, 10.0], [largest, largest]), [px(50.0), px(50.0)]);
    assert_eq!(
        grow([5.0, 5.0], [largest, largest / px(2.0)]),
        [px(65.0), px(35.0)]
    );
    assert_eq!(shrink(100.0, [largest, largest]), [px(50.0), px(50.0)]);
    assert_eq!(
        shrink(110.0, [largest, largest / px(2.0)]),
        [px(40.0), px(70.0)]
    );
    assert_eq!(shrink(100.0, [px(0.25), px(0.25)]), [px(75.0), px(75.0)]);
}

#[test]
fn flex_extreme_finite_factors_complete_f32() {
    assert_flex_extreme_finite_factors_complete::<f32>();
}

#[test]
fn flex_extreme_finite_factors_complete_f64() {
    assert_flex_extreme_finite_factors_complete::<f64>();
}

fn assert_flex_unrepresentable_shrink_proposal_clamps_before_narrowing<S: LayoutScalar>() {
    let px = S::from_f64;
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
    let children = [largest, S::ONE, S::ONE, S::ONE].map(|factor| NodeInputOf {
        flex_basis: FlexBasisOf::px(largest / px(2.0)),
        flex_shrink: FlexShrinkOf::try_new(factor).expect("finite shrink factor"),
        overflow: super::fixtures::computed_overflow(Overflow::Hidden, Overflow::Hidden),
        ..NodeInputOf::default()
    });
    let available = largest / px(4.0);
    let widths =
        layout_children(axes, flow, available.to_f64(), children).map(|output| output.size.width);

    assert_eq!(widths[0], S::ZERO);
    let expected = available / px(3.0);
    for width in &widths[1..] {
        assert!((*width / expected - S::ONE).abs() <= px(3.0) * S::EPSILON);
    }
}

#[test]
fn flex_unrepresentable_shrink_proposal_clamps_before_narrowing() {
    assert_flex_unrepresentable_shrink_proposal_clamps_before_narrowing::<f32>();
    assert_flex_unrepresentable_shrink_proposal_clamps_before_narrowing::<f64>();
}

fn assert_flex_unrepresentable_used_size_is_refused<S: LayoutScalar>() {
    let px = S::from_f64;
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .style(
            0,
            NodeInputOf {
                display: Display::Flex,
                size: Size::new(
                    PreferredSizeOf::px(px(100.0)),
                    PreferredSizeOf::px(px(20.0)),
                ),
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                flex_basis: FlexBasisOf::px(px(10.0)),
                flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
                margin: Edges {
                    left: LengthAutoOf::px(-largest),
                    right: LengthAutoOf::px(-largest),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..NodeInputOf::default()
            },
        );
    let error = compute_layout(
        &tree,
        0,
        super::fixtures::fri07_c02_collapse_round_request(flow),
    )
    .expect_err("the used main size exceeds the scalar range");
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::ValueResolution);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InvalidNumeric { value })
            if *value == S::INFINITY
    ));
}

#[test]
fn flex_unrepresentable_used_size_is_refused() {
    assert_flex_unrepresentable_used_size_is_refused::<f32>();
    assert_flex_unrepresentable_used_size_is_refused::<f64>();
}

fn assert_flex_oversized_provisional_target_can_settle<S: LayoutScalar>() {
    let px = S::from_f64;
    let largest = fri06_mr02_geometry_error_largest_finite::<S>();
    let available = largest / px(4.0);
    let minimum = largest * px(0.75);
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
    let children = [
        NodeInputOf {
            flex_basis: FlexBasisOf::px(S::ZERO),
            flex_grow: FlexGrowOf::try_new(px(9.0)).unwrap(),
            margin: Edges {
                left: LengthAutoOf::px(-largest / px(2.0)),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        },
        NodeInputOf {
            flex_basis: FlexBasisOf::px(S::ZERO),
            flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
            margin: Edges {
                left: LengthAutoOf::px(-minimum),
                ..Edges::all(LengthAutoOf::ZERO)
            },
            min_size: Size::new(MinSizeOf::px(minimum), MinSizeOf::ZERO),
            ..NodeInputOf::default()
        },
    ];
    let widths =
        layout_children(axes, flow, available.to_f64(), children).map(|output| output.size.width);

    // The second item freezes at its minimum. The first round's larger
    // provisional width for the first item is then recalculated.
    assert_eq!(widths[1], minimum);
    assert!((widths[0] / minimum - S::ONE).abs() <= px(2.0) * S::EPSILON);
}

#[test]
fn flex_oversized_provisional_target_can_settle() {
    assert_flex_oversized_provisional_target_can_settle::<f32>();
    assert_flex_oversized_provisional_target_can_settle::<f64>();
}

fn assert_flex_resolves_small_nonzero_free_space<S: LayoutScalar>() {
    let extent = S::from_f64(1.0 / 32_768.0);
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
    let layout = |available: S, basis: S| {
        let first = NodeInputOf {
            flex_basis: FlexBasisOf::px(basis),
            flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        let second = NodeInputOf {
            flex_basis: FlexBasisOf::px(S::ZERO),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            ..NodeInputOf::default()
        };
        layout_children(axes, flow, available.to_f64(), [first, second])[0]
            .size
            .width
    };

    assert_eq!(
        [layout(extent, S::ZERO), layout(S::ZERO, extent)],
        [extent, S::ZERO]
    );
}

#[test]
fn flex_resolves_small_nonzero_free_space() {
    assert_flex_resolves_small_nonzero_free_space::<f32>();
    assert_flex_resolves_small_nonzero_free_space::<f64>();
}

fn assert_flex_keeps_raw_border_box_base_until_clamping<S: LayoutScalar>() {
    let px = S::from_f64;
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for direction in [FlexDirection::Row, FlexDirection::Column] {
        let axes = FlexAxes::new(flow, direction, FlexWrap::NoWrap);
        let layout = |box_sizing, available| {
            let mut first = NodeInputOf {
                box_sizing,
                flex_basis: FlexBasisOf::px(S::ZERO),
                flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            axes.set_main_start_edge(&mut first.padding, LengthOf::px(px(40.0)));
            let second = NodeInputOf {
                flex_basis: FlexBasisOf::px(px(40.0)),
                flex_grow: FlexGrowOf::try_new(S::ONE).unwrap(),
                min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
                ..NodeInputOf::default()
            };
            layout_children(axes, flow, available, [first, second])
                .map(|output| axes.main_size(output.size))
        };
        assert_eq!(layout(BoxSizing::ContentBox, 100.0), [px(50.0), px(50.0)]);
        assert_eq!(layout(BoxSizing::BorderBox, 70.0), [px(40.0), px(30.0)]);
        // From raw bases 0/40, the provisional grow targets are 30/70.
        // The first item freezes at its 40px padding floor.
        assert_eq!(layout(BoxSizing::BorderBox, 100.0), [px(40.0), px(60.0)]);
    }
}

#[test]
fn flex_keeps_raw_border_box_base_until_clamping() {
    assert_flex_keeps_raw_border_box_base_until_clamping::<f32>();
    assert_flex_keeps_raw_border_box_base_until_clamping::<f64>();
}

fn assert_flex_basis_transfers_ratio_in_the_selected_sizing_box<S: LayoutScalar>() {
    let px = S::from_f64;
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let axes = FlexAxes::new(flow, FlexDirection::Row, FlexWrap::NoWrap);
    let layout = |box_sizing, authored_height| {
        let item = NodeInputOf {
            display: Display::Block,
            box_sizing,
            size: Size::new(
                PreferredSizeOf::AUTO,
                PreferredSizeOf::px(px(authored_height)),
            ),
            min_size: Size::new(MinSizeOf::ZERO, MinSizeOf::ZERO),
            padding: Edges {
                top: LengthOf::px(px(10.0)),
                bottom: LengthOf::px(px(10.0)),
                left: LengthOf::px(px(4.0)),
                right: LengthOf::px(px(4.0)),
            },
            aspect_ratio: AspectRatioOf::new(px(2.0)),
            flex_basis: FlexBasisOf::CONTENT,
            flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
            ..NodeInputOf::default()
        };
        layout_children(axes, flow, 200.0, [item, NodeInputOf::default()])[0].size
    };
    assert_eq!(
        layout(BoxSizing::BorderBox, 60.0),
        Size::new(px(120.0), px(60.0))
    );
    assert_eq!(
        layout(BoxSizing::ContentBox, 40.0),
        Size::new(px(88.0), px(60.0))
    );
}

#[test]
fn flex_basis_transfers_ratio_in_the_selected_sizing_box() {
    assert_flex_basis_transfers_ratio_in_the_selected_sizing_box::<f32>();
    assert_flex_basis_transfers_ratio_in_the_selected_sizing_box::<f64>();
}

fn assert_block_flow_flex_root_retains_auto_inline_fill<S: LayoutScalar>() {
    let px = S::from_f64;
    let parent_width = NonNegativeFiniteOf::new(px(100.0)).unwrap();
    let request = LayoutRootRequestOf::block_flow(
        Size::splat(SizingOperation::Normal),
        BlockFlowRootContextOf::horizontal_auto_block(
            parent_width,
            Direction::Ltr,
            OrthogonalFallbackOf::new(
                Size::new(Some(parent_width), None),
                NearestScrollportOf::Absent,
                Size::splat(OrthogonalFallbackLimitOf::Unbounded),
            ),
        ),
    );
    for (minimum, expected) in [
        (MinSizeOf::AUTO, 100.0),
        (MinSizeOf::ZERO, 100.0),
        (MinSizeOf::px(px(30.0)), 100.0),
        (MinSizeOf::px(px(120.0)), 120.0),
    ] {
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1])
            .style(
                0,
                NodeInputOf {
                    display: Display::Flex,
                    min_size: Size::new(minimum.clone(), MinSizeOf::AUTO),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    size: Size::new(PreferredSizeOf::px(px(20.0)), PreferredSizeOf::px(px(10.0))),
                    ..NodeInputOf::default()
                },
            );
        let batch = compute_layout(&tree, 0, request).unwrap();
        let width = |node| {
            batch
                .unrounded_entries()
                .iter()
                .find(|entry| entry.node() == node)
                .unwrap()
                .output()
                .size
                .width
        };
        assert_eq!(width(0), px(expected), "inline minimum {minimum:?}");
        assert_eq!(width(1), px(20.0));
    }
}

#[test]
fn block_flow_flex_root_retains_auto_inline_fill() {
    assert_block_flow_flex_root_retains_auto_inline_fill::<f32>();
    assert_block_flow_flex_root_retains_auto_inline_fill::<f64>();
}
