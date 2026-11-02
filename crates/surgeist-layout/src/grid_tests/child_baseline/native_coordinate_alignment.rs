//! Compatible baseline participants share one physical point after composing
//! their genuine ancestor origins. No measured output or font supplies ports.
use super::*;

#[derive(Clone, Copy, Debug)]
enum SelectedPort {
    First,
    Last,
}

fn value<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn physical_size<S: LayoutScalar>(mode: WritingMode, inline: f64, block: f64) -> Size<S> {
    match mode {
        WritingMode::HorizontalTb => Size::new(value(inline), value(block)),
        WritingMode::VerticalRl | WritingMode::VerticalLr => Size::new(value(block), value(inline)),
        _ => unreachable!("this fixture declares three parallel writing-mode profiles"),
    }
}

fn block_edges<T: Copy>(mode: WritingMode, start: T, end: T, inline: T) -> Edges<T> {
    match mode {
        WritingMode::HorizontalTb => Edges {
            top: start,
            bottom: end,
            right: inline,
            left: inline,
        },
        WritingMode::VerticalRl => Edges {
            top: inline,
            bottom: inline,
            right: start,
            left: end,
        },
        WritingMode::VerticalLr => Edges {
            top: inline,
            bottom: inline,
            right: end,
            left: start,
        },
        _ => unreachable!("this fixture declares three parallel writing-mode profiles"),
    }
}

fn inherited_tracks<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(vec![]))]
}

fn fixed_box<S: LayoutScalar>(
    mode: WritingMode,
    row: isize,
    column: isize,
    inline: f64,
    block: f64,
    port: SelectedPort,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Flex,
        writing_mode: mode,
        direction: Direction::Ltr,
        box_sizing: BoxSizing::BorderBox,
        size: physical_size(mode, inline, block).map(PreferredSizeOf::px),
        grid_row: GridPlacement::try_lines(row, row + 1).unwrap(),
        grid_column: GridPlacement::try_lines(column, column + 1).unwrap(),
        align_self: Some(match port {
            SelectedPort::First => AlignItems::Baseline,
            SelectedPort::Last => AlignItems::LastBaseline,
        }),
        margin: block_edges(
            mode,
            LengthAutoOf::px(value(3.0)),
            LengthAutoOf::px(value(5.0)),
            LengthAutoOf::ZERO,
        ),
        border: Edges::all(LengthOf::px(value(5.0))),
        ..NodeInputOf::default()
    }
}

fn native_tree<S: LayoutScalar>(mode: WritingMode) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 20, 21, 22, 23, 24, 25])
        .children(1, [2, 14, 15])
        .children(2, [10, 11, 12, 13])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: mode,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: physical_size(mode, 400.0, 600.0).map(PreferredSizeOf::px),
                grid_template_columns: vec![TrackComponentOf::px(value(100.0)); 4],
                grid_template_rows: vec![TrackComponentOf::AUTO; 3],
                ..NodeInputOf::default()
            },
        );
    for (node, rows, margin_start, margin_end, padding_start, padding_end, gap) in [
        (1, 4, 5.0, 10.0, 10.0, 20.0, 10.0),
        (2, 3, 3.0, 7.0, 5.0, 10.0, 20.0),
    ] {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Grid,
                writing_mode: mode,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                grid_row: GridPlacement::try_lines(1, rows).unwrap(),
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_template_columns: inherited_tracks(),
                grid_template_rows: inherited_tracks(),
                gap: Size::splat(LengthOf::px(value(gap))),
                margin: block_edges(
                    mode,
                    LengthAutoOf::px(value(margin_start)),
                    LengthAutoOf::px(value(margin_end)),
                    LengthAutoOf::ZERO,
                ),
                padding: block_edges(
                    mode,
                    LengthOf::px(value(padding_start)),
                    LengthOf::px(value(padding_end)),
                    LengthOf::ZERO,
                ),
                border: Edges::all(LengthOf::px(value(5.0))),
                ..NodeInputOf::default()
            },
        );
    }
    // Every slot is explicitly placed in the genuine native tree. The inner
    // grid covers owner rows1/2; the outer grid also supplies owner row3.
    for (node, row, column, inline, block, port) in [
        (10, 1, 1, 40.0, 50.0, SelectedPort::First),
        (11, 1, 2, 40.0, 50.0, SelectedPort::Last),
        (12, 2, 1, 40.0, 50.0, SelectedPort::First),
        (13, 2, 2, 40.0, 50.0, SelectedPort::Last),
        (14, 3, 1, 40.0, 50.0, SelectedPort::First),
        (15, 3, 2, 40.0, 50.0, SelectedPort::Last),
        (20, 1, 3, 20.0, 50.0, SelectedPort::First),
        (21, 1, 4, 20.0, 100.0, SelectedPort::Last),
        (22, 2, 3, 20.0, 50.0, SelectedPort::First),
        (23, 2, 4, 20.0, 100.0, SelectedPort::Last),
        (24, 3, 3, 20.0, 50.0, SelectedPort::First),
        (25, 3, 4, 20.0, 100.0, SelectedPort::Last),
    ] {
        tree = tree
            .children(node, [])
            .style(node, fixed_box(mode, row, column, inline, block, port));
    }
    tree
}

fn normal_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> NodeOutputOf<S> {
    let entries = batch
        .unrounded_entries()
        .iter()
        .filter(|entry| entry.node() == node)
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "one actual unrounded box for node{node}");
    entries[0].output()
}

fn selected_port<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    axes: FlowAxes,
    port: SelectedPort,
) -> S {
    let size = normal_box(batch, node).size;
    let mut outputs = batch
        .cache_store_entries()
        .iter()
        .filter(|entry| {
            entry.node() == node
                && entry.input().run_mode() == RunMode::PerformLayout
                && entry.output().size == size
        })
        .map(LayoutCacheStoreEntryOf::output);
    let first = outputs
        .next()
        .expect("the real fixed native box has a performed matching-body cache receipt");
    // Retain the actual supplied presence as well as its value: an absent Last
    // does not acquire the First port. Defined synthesis is applied only to the
    // actual output through its public selected-set helper.
    let actual_sets = (
        first.baselines().first_block(axes),
        first.baselines().last_block(axes),
    );
    for output in outputs {
        assert_eq!(
            (
                output.baselines().first_block(axes),
                output.baselines().last_block(axes)
            ),
            actual_sets,
            "matching performed native bodies must have consistent ports, node{node}"
        );
    }
    match port {
        SelectedPort::First => first.baselines().first_or_synthesize_block(axes, size),
        SelectedPort::Last => first.baselines().last_or_synthesize_block(axes, size),
    }
}

fn absolute_origin<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
) -> Point<S> {
    let parents = [
        (1, 0),
        (2, 1),
        (10, 2),
        (11, 2),
        (12, 2),
        (13, 2),
        (14, 1),
        (15, 1),
        (20, 0),
        (21, 0),
        (22, 0),
        (23, 0),
        (24, 0),
        (25, 0),
    ];
    let mut current = node;
    let mut origin = normal_box(batch, current).location;
    while current != 0 {
        let parent = parents
            .iter()
            .find_map(|(child, parent)| (*child == current).then_some(*parent))
            .expect("every actual fixture child has its declared parent");
        assert!(Traverse::children(tree, parent).any(|child| child == current));
        let location = normal_box(batch, parent).location;
        origin.x = origin.x + location.x;
        origin.y = origin.y + location.y;
        current = parent;
    }
    origin
}

fn assert_native_shared_physical_baselines<S: LayoutScalar>(mode: WritingMode) {
    let tree = native_tree::<S>(mode);
    let axes = FlowAxes::new(mode, Direction::Ltr);
    let batch = compute_layout(
        &tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .expect("genuine native nested subgrids complete");
    assert_eq!(
        normal_box(&batch, 0).size,
        physical_size(mode, 400.0, 600.0)
    );
    for (node, margin_start, margin_end, padding_start, padding_end) in
        [(1, 5.0, 10.0, 10.0, 20.0), (2, 3.0, 7.0, 5.0, 10.0)]
    {
        let actual = normal_box(&batch, node);
        assert_eq!(
            actual.margin,
            block_edges(mode, value(margin_start), value(margin_end), S::ZERO)
        );
        assert_eq!(
            actual.padding,
            block_edges(mode, value(padding_start), value(padding_end), S::ZERO)
        );
        assert_eq!(actual.border, Edges::all(value(5.0)));
    }
    let mut coordinates = Vec::new();
    // Qualify every actual body/port before any desired shared-coordinate
    // assertion, so a missing receipt is a setup failure, not behavioral RED.
    for (root, descendant, port) in [
        (20, 10, SelectedPort::First),
        (21, 11, SelectedPort::Last),
        (22, 12, SelectedPort::First),
        (23, 13, SelectedPort::Last),
        (24, 14, SelectedPort::First),
        (25, 15, SelectedPort::Last),
    ] {
        let root_block = match port {
            SelectedPort::First => 50.0,
            SelectedPort::Last => 100.0,
        };
        assert_eq!(
            normal_box(&batch, root).size,
            physical_size(mode, 20.0, root_block)
        );
        assert_eq!(
            normal_box(&batch, descendant).size,
            physical_size(mode, 40.0, 50.0)
        );
        for node in [root, descendant] {
            assert_eq!(Traverse::child_count(&tree, node), 0);
            assert!(!LayoutTree::has_leaf_measurement(&tree, node));
            let actual = normal_box(&batch, node);
            assert_eq!(
                actual.margin,
                block_edges(mode, value(3.0), value(5.0), S::ZERO)
            );
            assert_eq!(actual.border, Edges::all(value(5.0)));
        }
        let root_port = selected_port(&batch, root, axes, port);
        let descendant_port = selected_port(&batch, descendant, axes, port);
        let root_origin = absolute_origin(&tree, &batch, root);
        let descendant_origin = absolute_origin(&tree, &batch, descendant);
        let (root_coordinate, descendant_coordinate) = match axes.block_axis() {
            PhysicalAxis::Horizontal => (
                root_origin.x + root_port,
                descendant_origin.x + descendant_port,
            ),
            PhysicalAxis::Vertical => (
                root_origin.y + root_port,
                descendant_origin.y + descendant_port,
            ),
        };
        coordinates.push((
            root,
            descendant,
            port,
            root_coordinate,
            descendant_coordinate,
        ));
    }
    // The independent oracle is the physical equality of compatible selected
    // ports. No browser rectangle, sizing-phase shim, or reported group target
    // supplies the expected coordinate.
    for (root, descendant, port, root_coordinate, descendant_coordinate) in coordinates {
        assert_eq!(
            root_coordinate, descendant_coordinate,
            "{mode:?} {port:?}: root node{root} and descendant node{descendant} share one physical baseline"
        );
    }
}

macro_rules! native_coordinate_case {
    ($name:ident, $scalar:ty, $mode:expr) => {
        #[test]
        fn $name() {
            assert_native_shared_physical_baselines::<$scalar>($mode);
        }
    };
}

native_coordinate_case!(
    native_shared_physical_baselines_horizontal_f32,
    f32,
    WritingMode::HorizontalTb
);
native_coordinate_case!(
    native_shared_physical_baselines_horizontal_f64,
    f64,
    WritingMode::HorizontalTb
);
native_coordinate_case!(
    native_shared_physical_baselines_vertical_rl_f32,
    f32,
    WritingMode::VerticalRl
);
native_coordinate_case!(
    native_shared_physical_baselines_vertical_rl_f64,
    f64,
    WritingMode::VerticalRl
);
native_coordinate_case!(
    native_shared_physical_baselines_vertical_lr_f32,
    f32,
    WritingMode::VerticalLr
);
native_coordinate_case!(
    native_shared_physical_baselines_vertical_lr_f64,
    f64,
    WritingMode::VerticalLr
);
