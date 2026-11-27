use super::fixtures::{
    empty_subgrid_track, fri08_c02_auto_fit_output, fri08_c02_auto_fit_repeat,
    vertical_baseline_measure,
};
use super::*;

#[test]
fn fri08_c06_collapsed_gutter_automatic_and_absolute_spans_use_the_same_line_offsets() {
    let tree = PublicLayoutTreeOf::<f32>::new()
        .children(1, [2, 3, 4])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                size: Size::new(PreferredSize::px(140.0), PreferredSize::px(20.0)),
                grid_template_columns: vec![fri08_c02_auto_fit_repeat()],
                grid_template_rows: vec![TrackComponent::px(20.0)],
                gap: Size::new(Length::px(10.0), Length::ZERO),
                justify_content: Some(AlignContent::Center),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            2,
            NodeInput {
                grid_column: GridPlacement::try_line(1).expect("first repeated track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            3,
            NodeInput {
                grid_column: GridPlacement::try_line(3).expect("third repeated track"),
                ..NodeInput::DEFAULT
            },
        )
        .style(
            4,
            NodeInput {
                position: Position::Absolute,
                grid_column: GridPlacement::try_lines(1, 4)
                    .expect("absolute span through the interior collapsed run"),
                grid_row: GridPlacement::try_lines(1, 2).expect("single row"),
                inset: Edges::all(LengthAuto::ZERO),
                ..NodeInput::DEFAULT
            },
        );
    let absolute = fri08_c02_auto_fit_output(&tree, Size::new(140.0, 20.0), 4);
    assert_eq!((absolute.location.x, absolute.size.width), (25.0, 90.0));
}

#[test]
fn fri06_c12_t08_vertical_auto_rows_preserve_full_fixture_targets() {
    let mut tree = OracleTree::new()
        .children(1, [2, 3, 5, 6, 7])
        .children(2, [])
        .children(3, [4])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .children(7, [])
        .style(
            1,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSize::px(600.0), PreferredSize::px(400.0)),
                grid_template_rows: vec![
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                    TrackComponent::AUTO,
                ],
                grid_template_columns: vec![
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                    TrackComponent::px(100.0),
                ],
                ..NodeInput::default()
            },
        )
        .style(
            2,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(1).expect("valid first column"),
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            3,
            NodeInput {
                display: Display::Grid,
                writing_mode: WritingMode::VerticalRl,
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(2).expect("valid second column"),
                grid_template_rows: vec![empty_subgrid_track()],
                grid_template_columns: vec![TrackComponent::px(100.0)],
                ..NodeInput::default()
            },
        )
        .style(
            4,
            NodeInput {
                writing_mode: WritingMode::VerticalRl,
                align_self: Some(AlignItems::Baseline),
                ..NodeInput::default()
            },
        )
        .style(
            5,
            NodeInput {
                grid_row: GridPlacement::try_line(1).expect("valid first row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .style(
            6,
            NodeInput {
                grid_row: GridPlacement::try_line(2).expect("valid second row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .style(
            7,
            NodeInput {
                grid_row: GridPlacement::try_line(3).expect("valid third row"),
                grid_column: GridPlacement::try_line(3).expect("valid third column"),
                ..NodeInput::default()
            },
        )
        .measure(2, vertical_baseline_measure(10.0, 20.0, Some(9.0), None))
        .measure(4, vertical_baseline_measure(10.0, 20.0, Some(1.0), None));

    {
        let axes = Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::value::NonNegativeFiniteOf::new(600.0).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(crate::value::NonNegativeFiniteOf::new(400.0).unwrap()),
                SizingOperation::Normal,
            ),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        let style = tree.node_input(1);
        let flow = FlowAxes::new(style.writing_mode, style.direction);
        let request = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::value::NearestScrollportOf::Absent,
                    Size::splat(crate::value::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            flow,
            definite,
        );
        compute_root(&mut tree.borrow_compute(), 1, request)
    }
    .expect("vertical auto-row fixture computes");
    round_layout(&mut tree.borrow_compute(), 1).expect("vertical auto-row fixture rounds");

    assert_eq!(
        [5_u32, 6, 7].map(|node| {
            tree.final_layout(node)
                .expect("auto-row probe is laid out")
                .size
                .width
        }),
        [212.0, 194.0, 194.0],
    );
}
