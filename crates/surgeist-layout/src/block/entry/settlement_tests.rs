use crate::geometry::LogicalSizeOf;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

fn block<S: LayoutScalar>(axes: FlowAxes, height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode: axes.writing_mode(),
        direction: axes.direction(),
        size: axes.physical_size(LogicalSizeOf::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            height.map_or(PreferredSizeOf::AUTO, |height| {
                PreferredSizeOf::px(S::from_f64(height))
            }),
        )),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    }
}

fn request<S: LayoutScalar>(axes: FlowAxes) -> LayoutRootRequestOf<S> {
    let inline = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
        axes,
        axes.physical_size(LogicalSizeOf::new(Some(inline), None)),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(axes.physical_size(LogicalSizeOf::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(inline), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    )))
}

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .node_layout(node)
        .unwrap()
        .single_box()
        .unwrap()
        .unrounded_output()
}

#[test]
fn shared_clearance_keeps_the_authoritative_fractional_border_f32() {
    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(0, [1, 2, 3])
        .children(3, [4])
        .style(0, block(axes, None))
        .style(
            1,
            NodeInputOf {
                float: Float::Left,
                ..block(axes, Some(100.3))
            },
        )
        .style(2, block(axes, Some(0.1)))
        .style(3, block(axes, None))
        .style(
            4,
            NodeInputOf {
                clear: Clear::Left,
                margin: Edges {
                    top: LengthAutoOf::px(0.7),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..block(axes, Some(10.0))
            },
        );
    let batch = compute_layout(&tree, 0, request(axes)).unwrap();
    let parent = output(&batch, 3);
    let child = output(&batch, 4);
    assert_eq!(output(&batch, 1).size.height, 100.3);
    assert_eq!(output(&batch, 2).size.height, 0.1);
    assert_eq!(parent.location.y, 0.1);
    // The common-BFC bound owns the border. Recovering that border by adding
    // the separately rounded clearance to the top margin changes its f32 value.
    assert_eq!(child.location.y, 100.3_f32 - 0.1_f32);
    assert_eq!(parent.size.height, (100.3_f32 - 0.1_f32) + 10.0);
}

fn local_clearance_passes<S: LayoutScalar>() {
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let axes = FlowAxes::new(mode, direction);
            for clear in [Clear::None, Clear::Left] {
                let margin = axes.physical_edges(crate::geometry::LogicalEdgesOf::new(
                    S::ZERO,
                    S::ZERO,
                    S::from_f64(10.0),
                    S::ZERO,
                ));
                let tree = PublicLayoutTreeOf::<S>::new()
                    .children(0, [1])
                    .children(1, [2, 3])
                    .style(0, block(axes, None))
                    .style(1, block(axes, None))
                    .style(
                        2,
                        NodeInputOf {
                            float: Float::Left,
                            ..block(axes, Some(5.0))
                        },
                    )
                    .style(
                        3,
                        NodeInputOf {
                            clear,
                            margin: margin.map(LengthAutoOf::px),
                            ..block(axes, Some(10.0))
                        },
                    );
                let batch = compute_layout(&tree, 0, request(axes)).unwrap();
                let root = output(&batch, 0);
                let parent = output(&batch, 1);
                let child = output(&batch, 3);
                let introduced = clear == Clear::Left;
                assert_eq!(
                    axes.logical_point(parent.location, parent.size, root.size)
                        .block,
                    S::from_f64(if introduced { 0.0 } else { 10.0 }),
                    "{mode:?}/{direction:?}/{clear:?}"
                );
                // H10/Fh15 is retained while P moves; actual Fs5 gives C0.
                assert_eq!(
                    axes.logical_point(child.location, child.size, parent.size)
                        .block,
                    S::from_f64(if introduced { 10.0 } else { 0.0 }),
                    "{mode:?}/{direction:?}/{clear:?}"
                );
                assert_eq!(
                    axes.logical_size(parent.size).block,
                    S::from_f64(if introduced { 20.0 } else { 10.0 }),
                    "{mode:?}/{direction:?}/{clear:?}"
                );
                assert_eq!(axes.logical_size(root.size).block, S::from_f64(20.0));
            }
        }
    }
}

#[test]
fn local_introduced_clearance_survives_final_geometry_passes_f32() {
    local_clearance_passes::<f32>();
}
#[test]
fn local_introduced_clearance_survives_final_geometry_passes_f64() {
    local_clearance_passes::<f64>();
}
