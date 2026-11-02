use super::*;
use crate::LayoutTree;

fn preserve_row_margins<S: LayoutScalar>() {
    let n = S::from_f64;
    for writing_mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
    ] {
        let flow = FlowAxes::new(writing_mode, Direction::Ltr);
        let (margin, expected, border, position) = match writing_mode {
            WritingMode::HorizontalTb => (
                Edges {
                    top: LengthAutoOf::px(n(3.0)),
                    bottom: LengthAutoOf::px(n(4.0)),
                    left: LengthAutoOf::ZERO,
                    right: LengthAutoOf::ZERO,
                },
                Size::new(n(30.0), n(27.0)),
                Size::new(n(30.0), n(20.0)),
                Point::new(S::ZERO, n(3.0)),
            ),
            WritingMode::VerticalRl => (
                Edges {
                    left: LengthAutoOf::px(n(4.0)),
                    right: LengthAutoOf::px(n(3.0)),
                    top: LengthAutoOf::ZERO,
                    bottom: LengthAutoOf::ZERO,
                },
                Size::new(n(27.0), n(30.0)),
                Size::new(n(20.0), n(30.0)),
                Point::new(n(4.0), S::ZERO),
            ),
            WritingMode::VerticalLr => (
                Edges {
                    left: LengthAutoOf::px(n(3.0)),
                    right: LengthAutoOf::px(n(4.0)),
                    top: LengthAutoOf::ZERO,
                    bottom: LengthAutoOf::ZERO,
                },
                Size::new(n(27.0), n(30.0)),
                Size::new(n(20.0), n(30.0)),
                Point::new(n(3.0), S::ZERO),
            ),
            _ => unreachable!("three independently specified physical cases"),
        };
        for overflow in [Overflow::Visible, Overflow::Clip] {
            let tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Grid,
                        writing_mode,
                        size: flow.physical_size(LogicalSizeOf::new(
                            PreferredSizeOf::px(n(30.0)),
                            PreferredSizeOf::MIN_CONTENT,
                        )),
                        grid_template_columns: vec![TrackComponentOf::px(n(30.0))],
                        grid_template_rows: vec![TrackComponentOf::MAX_CONTENT],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Grid,
                        writing_mode,
                        grid_template_columns: vec![TrackComponentOf::px(n(30.0))],
                        grid_template_rows: vec![TrackComponentOf::Subgrid(SubgridTrack::new(
                            Vec::new(),
                        ))],
                        justify_content: Some(AlignContent::Start),
                        align_content: Some(AlignContent::Start),
                        align_items: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        writing_mode,
                        margin: margin.clone(),
                        size: flow.physical_size(LogicalSizeOf::new(
                            PreferredSizeOf::px(n(30.0)),
                            PreferredSizeOf::AUTO,
                        )),
                        max_size: flow.physical_size(LogicalSizeOf::new(
                            MaxSizeOf::px(n(30.0)),
                            MaxSizeOf::px(n(20.0)),
                        )),
                        overflow: computed_overflow(overflow, overflow),
                        align_self: Some(AlignItems::Start),
                        ..NodeInputOf::default()
                    },
                )
                .measure(2, flow.physical_size(LogicalSizeOf::new(n(90.0), n(80.0))));
            for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                let output = crate::measurement::compute_tree_leaf(
                    &tree,
                    2,
                    ComputeInputOf::for_child(
                        RunMode::ComputeSize,
                        SizingMode::InherentSize,
                        RequestedAxis::Both,
                        Size::NONE,
                        Size::NONE,
                        ContainingLayoutContext::new(flow, ParentFormattingContext::Grid),
                        flow.physical_size(LogicalSizeOf::new(
                            AvailableOf::Definite(n(30.0)),
                            available,
                        )),
                    ),
                    tree.node_input(2),
                )
                .unwrap();
                assert_eq!(
                    output.size, border,
                    "real leaf normal box {writing_mode:?}/{overflow:?}"
                );
            }
            let batch = compute_layout(
                &tree,
                0,
                LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
            )
            .unwrap();
            assert_eq!(
                fri08_c01_placement_output(&batch, 0).size,
                expected,
                "the inherited row includes the real seven-unit physical margin {writing_mode:?}/{overflow:?}"
            );
            assert_eq!(fri08_c01_placement_output(&batch, 1).size, expected);
            let leaf = fri08_c01_placement_output(&batch, 2);
            assert_eq!(leaf.size, border);
            assert_eq!(leaf.location, position);
        }
    }
}

#[test]
fn flattened_rows_preserve_physical_margins_f32() {
    preserve_row_margins::<f32>();
}
#[test]
fn flattened_rows_preserve_physical_margins_f64() {
    preserve_row_margins::<f64>();
}
