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

// Additional genuine native coordinate profiles.

fn opposite_mode(mode: WritingMode) -> WritingMode {
    match mode {
        WritingMode::VerticalRl => WritingMode::VerticalLr,
        WritingMode::VerticalLr => WritingMode::VerticalRl,
        _ => unreachable!("reversal cases use the same genuine physical block axis"),
    }
}

fn opposite_port(port: SelectedPort) -> SelectedPort {
    match port {
        SelectedPort::First => SelectedPort::Last,
        SelectedPort::Last => SelectedPort::First,
    }
}

fn alignment(port: SelectedPort) -> AlignItems {
    match port {
        SelectedPort::First => AlignItems::Baseline,
        SelectedPort::Last => AlignItems::LastBaseline,
    }
}

fn auto_block_size<S: LayoutScalar>(mode: WritingMode) -> Size<PreferredSizeOf<S>> {
    match mode {
        WritingMode::HorizontalTb => {
            Size::new(PreferredSizeOf::px(value(400.0)), PreferredSizeOf::AUTO)
        }
        WritingMode::VerticalRl | WritingMode::VerticalLr => {
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(value(400.0)))
        }
        _ => {
            unreachable!("these fixtures have the same physical block-axis profiles as the parent")
        }
    }
}

fn configured_tree<S: LayoutScalar>(
    root_mode: WritingMode,
    outer_mode: WritingMode,
    inner_mode: WritingMode,
    inner_gap: f64,
    root_first_extent: f64,
) -> PublicLayoutTreeOf<S> {
    let mut tree = native_tree::<S>(root_mode);
    let mut root_input = LayoutTree::node_input(&tree, 0).clone();
    // A genuine intrinsic root avoids free-space thirds. This is authored input,
    // never a post-compute geometry edit or a different measurement provider.
    root_input.size = auto_block_size(root_mode);
    root_input.align_content = Some(AlignContent::Start);
    tree.insert_input(0, LayoutInputOf::box_input(root_input));
    for (node, mode, margin_start, margin_end, padding_start, padding_end, gap) in [
        (1, outer_mode, 5.0, 10.0, 10.0, 20.0, 10.0),
        (2, inner_mode, 3.0, 7.0, 5.0, 10.0, inner_gap),
    ] {
        let mut input = LayoutTree::node_input(&tree, node).clone();
        input.writing_mode = mode;
        input.margin = block_edges(
            mode,
            LengthAutoOf::px(value(margin_start)),
            LengthAutoOf::px(value(margin_end)),
            LengthAutoOf::ZERO,
        );
        input.padding = block_edges(
            mode,
            LengthOf::px(value(padding_start)),
            LengthOf::px(value(padding_end)),
            LengthOf::ZERO,
        );
        input.gap = Size::splat(LengthOf::px(value(gap)));
        tree.insert_input(node, LayoutInputOf::box_input(input));
    }
    for (node, row, column, port) in [
        (10, 1, 1, SelectedPort::First),
        (11, 1, 2, SelectedPort::Last),
        (12, 2, 1, SelectedPort::First),
        (13, 2, 2, SelectedPort::Last),
        (14, 3, 1, SelectedPort::First),
        (15, 3, 2, SelectedPort::Last),
    ] {
        let mode = if node <= 13 { inner_mode } else { outer_mode };
        tree.insert_input(
            node,
            LayoutInputOf::box_input(fixed_box(mode, row, column, 40.0, 50.0, port)),
        );
    }
    for (node, row, column, port) in [
        (20, 1, 3, SelectedPort::First),
        (21, 1, 4, SelectedPort::Last),
        (22, 2, 3, SelectedPort::First),
        (23, 2, 4, SelectedPort::Last),
        (24, 3, 3, SelectedPort::First),
        (25, 3, 4, SelectedPort::Last),
    ] {
        let extent = match port {
            SelectedPort::First => root_first_extent,
            SelectedPort::Last => 100.0,
        };
        tree.insert_input(
            node,
            LayoutInputOf::box_input(fixed_box(root_mode, row, column, 20.0, extent, port)),
        );
    }
    tree
}

fn completed<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(
        tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .expect("real native intrinsic-root subgrids complete")
}

// Source-declared one-based rows mapped through each actual inherited span.
// Both outer and inner spans begin at1; their exclusive ends are4 and3.
fn parent_row(row: isize, exclusive_end: isize, reversed: bool) -> isize {
    if reversed { exclusive_end - row } else { row }
}

fn qualify_and_assert<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    batch: &CompletedLayoutBatchOf<u32, S>,
    root_mode: WritingMode,
    outer_mode: WritingMode,
    inner_mode: WritingMode,
    root_first_extent: f64,
) {
    let root_axes = FlowAxes::new(root_mode, Direction::Ltr);
    let outer_axes = FlowAxes::new(outer_mode, Direction::Ltr);
    let inner_axes = FlowAxes::new(inner_mode, Direction::Ltr);
    assert_eq!(root_axes.block_axis(), outer_axes.block_axis());
    assert_eq!(root_axes.block_axis(), inner_axes.block_axis());
    let physical_axis = root_axes.block_axis();
    let outer_reversed = root_axes.physical_axis_progression(physical_axis)
        != outer_axes.physical_axis_progression(physical_axis);
    let inner_reversed = outer_axes.physical_axis_progression(physical_axis)
        != inner_axes.physical_axis_progression(physical_axis);
    assert_eq!(
        LayoutTree::node_input(tree, 1).grid_row,
        GridPlacement::try_lines(1, 4).unwrap()
    );
    assert_eq!(
        LayoutTree::node_input(tree, 2).grid_row,
        GridPlacement::try_lines(1, 3).unwrap()
    );
    let mut coordinates = Vec::new();
    for (node, local_row, column, selected) in [
        (10, 1, 1, SelectedPort::First),
        (11, 1, 2, SelectedPort::Last),
        (12, 2, 1, SelectedPort::First),
        (13, 2, 2, SelectedPort::Last),
        (14, 3, 1, SelectedPort::First),
        (15, 3, 2, SelectedPort::Last),
    ] {
        let inside_inner = node <= 13;
        let axes = if inside_inner { inner_axes } else { outer_axes };
        let mode = if inside_inner { inner_mode } else { outer_mode };
        let outer_row = if inside_inner {
            parent_row(local_row, 3, inner_reversed)
        } else {
            local_row
        };
        let owner_row = parent_row(outer_row, 4, outer_reversed);
        assert!((1..=3).contains(&owner_row));
        let relative_reversal = axes.physical_axis_progression(physical_axis)
            != root_axes.physical_axis_progression(physical_axis);
        let owner_selected = if relative_reversal {
            opposite_port(selected)
        } else {
            selected
        };
        // Opposite baseline preferences belong to compatible groups only when
        // their real block progressions are opposite. The actual child still
        // selects its own First or Last set; no port is replaced with its peer.
        let role_offset = match owner_selected {
            SelectedPort::First => 0,
            SelectedPort::Last => 1,
        };
        let owner = u32::try_from(20 + (owner_row - 1) * 2 + role_offset).unwrap();
        let input = LayoutTree::node_input(tree, node);
        assert_eq!(input.writing_mode, mode);
        assert_eq!(input.align_self, Some(alignment(selected)));
        assert_eq!(
            input.grid_row,
            GridPlacement::try_lines(local_row, local_row + 1).unwrap()
        );
        assert_eq!(
            input.grid_column,
            GridPlacement::try_lines(column, column + 1).unwrap()
        );
        let owner_input = LayoutTree::node_input(tree, owner);
        assert_eq!(owner_input.writing_mode, root_mode);
        assert_eq!(owner_input.align_self, Some(alignment(owner_selected)));
        assert_eq!(
            owner_input.grid_row,
            GridPlacement::try_lines(owner_row, owner_row + 1).unwrap()
        );
        assert_eq!(
            normal_box(batch, node).size,
            physical_size(mode, 40.0, 50.0)
        );
        let owner_extent = match owner_selected {
            SelectedPort::First => root_first_extent,
            SelectedPort::Last => 100.0,
        };
        assert_eq!(
            normal_box(batch, owner).size,
            physical_size(root_mode, 20.0, owner_extent)
        );
        for (subject, subject_mode) in [(node, mode), (owner, root_mode)] {
            assert_eq!(Traverse::child_count(tree, subject), 0);
            assert!(!LayoutTree::has_leaf_measurement(tree, subject));
            let actual = normal_box(batch, subject);
            assert_eq!(
                actual.margin,
                block_edges(subject_mode, value(3.0), value(5.0), S::ZERO)
            );
            assert_eq!(actual.border, Edges::all(value(5.0)));
        }
        let port = selected_port(batch, node, axes, selected);
        let owner_port = selected_port(batch, owner, root_axes, owner_selected);
        let origin = absolute_origin(tree, batch, node);
        let owner_origin = absolute_origin(tree, batch, owner);
        let (coordinate, owner_coordinate) = match physical_axis {
            PhysicalAxis::Horizontal => (origin.x + port, owner_origin.x + owner_port),
            PhysicalAxis::Vertical => (origin.y + port, owner_origin.y + owner_port),
        };
        coordinates.push((
            node,
            owner,
            selected,
            owner_selected,
            coordinate,
            owner_coordinate,
        ));
    }
    // All native body/port and declared row/role premises execute before the
    // desired physical alignment assertions. No observed output chooses pairs.
    for (node, owner, selected, owner_selected, coordinate, owner_coordinate) in coordinates {
        assert_eq!(
            coordinate, owner_coordinate,
            "root={root_mode:?} outer={outer_mode:?} inner={inner_mode:?}: node{node}/{selected:?} shares physical port with owner{owner}/{owner_selected:?}"
        );
    }
}

fn negative_gap<S: LayoutScalar>(mode: WritingMode) {
    // Root gap0 -> outer10 -> inner0 gives actual signed differences +10/-10,
    // hence genuine per-side +5/-5. The inner gap is authored zero, not omitted.
    let tree = configured_tree::<S>(mode, mode, mode, 0.0, 50.0);
    assert_eq!(
        LayoutTree::node_input(&tree, 1).gap,
        Size::splat(LengthOf::px(value(10.0)))
    );
    assert_eq!(
        LayoutTree::node_input(&tree, 2).gap,
        Size::splat(LengthOf::ZERO)
    );
    let batch = completed(&tree);
    qualify_and_assert(&tree, &batch, mode, mode, mode, 50.0);
}

fn changed_winner<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let axes = FlowAxes::new(mode, Direction::Ltr);
    let narrow = configured_tree::<S>(mode, mode, mode, 20.0, 50.0);
    let wide = configured_tree::<S>(mode, mode, mode, 20.0, 150.0);
    let narrow_batch = completed(&narrow);
    let wide_batch = completed(&wide);
    assert_eq!(
        normal_box(&narrow_batch, 20).size,
        physical_size(mode, 20.0, 50.0)
    );
    assert_eq!(
        normal_box(&wide_batch, 20).size,
        physical_size(mode, 20.0, 150.0)
    );
    assert_eq!(
        normal_box(&narrow_batch, 10).size,
        physical_size(mode, 40.0, 50.0)
    );
    assert_eq!(
        normal_box(&wide_batch, 10).size,
        physical_size(mode, 40.0, 50.0)
    );
    let narrow_root_port = selected_port(&narrow_batch, 20, axes, SelectedPort::First);
    let wide_root_port = selected_port(&wide_batch, 20, axes, SelectedPort::First);
    let narrow_leaf_port = selected_port(&narrow_batch, 10, axes, SelectedPort::First);
    let wide_leaf_port = selected_port(&wide_batch, 10, axes, SelectedPort::First);
    assert_eq!(narrow_leaf_port, wide_leaf_port);
    // At the touched owner-start edge, real outer m5+b5+p10=20 and
    // inner m3+b5+p5=13 add33. Neither boundary's internal gap touches
    // that start edge. Own right margin3 and p are real native inputs.
    let leaf_distance = value::<S>(33.0 + 3.0 + 50.0) - narrow_leaf_port;
    let narrow_root_distance = value::<S>(3.0 + 50.0) - narrow_root_port;
    let wide_root_distance = value::<S>(3.0 + 150.0) - wide_root_port;
    assert!(
        narrow_root_distance < leaf_distance,
        "the real descendant supplies the narrow First maximum"
    );
    assert!(
        wide_root_distance > leaf_distance,
        "the larger actual root body now supplies the First maximum"
    );
    // The two internal touched end edges contribute half(10-0)+half(20-10)=10.
    // Thus the other source wins the opposite distance, which must not replace
    // the genuine owner target when its current-view coordinates are converted.
    let wide_root_opposite = value::<S>(5.0) + wide_root_port;
    let leaf_opposite = value::<S>(10.0 + 5.0) + wide_leaf_port;
    assert!(
        leaf_opposite > wide_root_opposite,
        "opposite-distance maximization would select a different actual source"
    );
    qualify_and_assert(&narrow, &narrow_batch, mode, mode, mode, 50.0);
    qualify_and_assert(&wide, &wide_batch, mode, mode, mode, 150.0);
}

fn one_reversal<S: LayoutScalar>() {
    for root_mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let outer_mode = opposite_mode(root_mode);
        let tree = configured_tree::<S>(root_mode, outer_mode, outer_mode, 20.0, 50.0);
        let batch = completed(&tree);
        qualify_and_assert(&tree, &batch, root_mode, outer_mode, outer_mode, 50.0);
    }
}

fn two_reversals<S: LayoutScalar>() {
    for root_mode in [WritingMode::VerticalRl, WritingMode::VerticalLr] {
        let outer_mode = opposite_mode(root_mode);
        let tree = configured_tree::<S>(root_mode, outer_mode, root_mode, 20.0, 50.0);
        let batch = completed(&tree);
        qualify_and_assert(&tree, &batch, root_mode, outer_mode, root_mode, 50.0);
    }
}

macro_rules! negative_case {
    ($name:ident, $scalar:ty, $mode:expr) => {
        #[test]
        fn $name() {
            negative_gap::<$scalar>($mode);
        }
    };
}
negative_case!(
    negative_nested_gap_horizontal_f32,
    f32,
    WritingMode::HorizontalTb
);
negative_case!(
    negative_nested_gap_horizontal_f64,
    f64,
    WritingMode::HorizontalTb
);
negative_case!(
    negative_nested_gap_vertical_rl_f32,
    f32,
    WritingMode::VerticalRl
);
negative_case!(
    negative_nested_gap_vertical_rl_f64,
    f64,
    WritingMode::VerticalRl
);
negative_case!(
    negative_nested_gap_vertical_lr_f32,
    f32,
    WritingMode::VerticalLr
);
negative_case!(
    negative_nested_gap_vertical_lr_f64,
    f64,
    WritingMode::VerticalLr
);
#[test]
fn changed_first_group_winner_f32() {
    changed_winner::<f32>();
}
#[test]
fn changed_first_group_winner_f64() {
    changed_winner::<f64>();
}
#[test]
fn one_relative_reversal_f32() {
    one_reversal::<f32>();
}
#[test]
fn one_relative_reversal_f64() {
    one_reversal::<f64>();
}
#[test]
fn two_relative_reversals_compose_f32() {
    two_reversals::<f32>();
}
#[test]
fn two_relative_reversals_compose_f64() {
    two_reversals::<f64>();
}

fn saturated_columns<S: LayoutScalar>() {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 20])
        .children(1, [14, 15, 16, 17])
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                size: Size::new(
                    PreferredSizeOf::px(value(110.0)),
                    PreferredSizeOf::px(value(80.0)),
                ),
                grid_template_columns: vec![
                    TrackComponentOf::px(value(10.0)),
                    TrackComponentOf::px(value(100.0)),
                ],
                grid_template_rows: vec![TrackComponentOf::px(value(40.0)); 2],
                ..NodeInputOf::default()
            },
        )
        .style(
            1,
            NodeInputOf {
                display: Display::Grid,
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                grid_template_columns: inherited_tracks(),
                grid_template_rows: vec![TrackComponentOf::px(value(20.0)); 2],
                gap: Size::new(LengthOf::px(value(40.0)), LengthOf::ZERO),
                ..NodeInputOf::default()
            },
        );
    for (node, row, column) in [(14, 1, 1), (15, 1, 2), (20, 2, 2)] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Flex,
                writing_mode: WritingMode::VerticalRl,
                direction: Direction::Ltr,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(
                    PreferredSizeOf::px(value(10.0)),
                    PreferredSizeOf::px(value(20.0)),
                ),
                grid_row: GridPlacement::try_lines(row, row + 1).unwrap(),
                grid_column: GridPlacement::try_lines(column, column + 1).unwrap(),
                justify_self: Some(AlignItems::Baseline),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
    }
    for (node, column) in [(16, 1), (17, 2)] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Flex,
                box_sizing: BoxSizing::BorderBox,
                size: Size::new(PreferredSizeOf::px(S::ONE), PreferredSizeOf::px(S::ONE)),
                grid_row: GridPlacement::try_lines(2, 3).unwrap(),
                grid_column: GridPlacement::try_lines(column, column + 1).unwrap(),
                justify_self: Some(AlignItems::Start),
                align_self: Some(AlignItems::Start),
                ..NodeInputOf::default()
            },
        );
    }
    let batch = completed(&tree);
    // Current genuine native Columns layout uses shifted boundaries when a
    // positive gap difference saturates a track. For [10,100], parent gap0,
    // child gap40: shifted lines are0, 10+40/2=30, 110, so actual tracks30/80
    // with zero gutter. These plain Start-aligned boxes authenticate the
    // placed frame; the inheritance report's [0,80]+gap40 is not that frame.
    assert_eq!(
        normal_box(&batch, 1).size,
        Size::new(value(110.0), value(40.0))
    );
    assert_eq!(normal_box(&batch, 16).location.x, S::ZERO);
    assert_eq!(normal_box(&batch, 17).location.x, value(30.0));
    let axes = FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr);
    for node in [14, 15, 20] {
        assert_eq!(
            normal_box(&batch, node).size,
            Size::new(value(10.0), value(20.0))
        );
        assert_eq!(Traverse::child_count(&tree, node), 0);
        assert!(!LayoutTree::has_leaf_measurement(&tree, node));
    }
    let descendant = selected_port(&batch, 15, axes, SelectedPort::First);
    let owner = selected_port(&batch, 20, axes, SelectedPort::First);
    let _other_real_column_port = selected_port(&batch, 14, axes, SelectedPort::First);
    let descendant_origin = absolute_origin(&tree, &batch, 15);
    let owner_origin = absolute_origin(&tree, &batch, 20);
    assert_eq!(
        descendant_origin.x + descendant,
        owner_origin.x + owner,
        "the selected physical baseline uses the actual saturated column layout frame"
    );
}

#[test]
fn saturated_column_layout_frame_f32() {
    saturated_columns::<f32>();
}
#[test]
fn saturated_column_layout_frame_f64() {
    saturated_columns::<f64>();
}

fn fixed_composite_ports_and_alignment<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let axes = FlowAxes::new(mode, Direction::Ltr);
    let mut tree = configured_tree::<S>(mode, mode, mode, 20.0, 50.0);
    for (actor, first_child, last_child) in [(10, 30, 31), (11, 32, 33)] {
        let mut input = LayoutTree::node_input(&tree, actor).clone();
        input.display = Display::Grid;
        input.border = Edges::all(LengthOf::ZERO);
        input.padding = Edges::all(LengthOf::ZERO);
        input.grid_template_columns = vec![TrackComponentOf::px(value(40.0))];
        input.grid_template_rows = vec![
            TrackComponentOf::px(value(20.0)),
            TrackComponentOf::px(value(30.0)),
        ];
        input.gap = Size::splat(LengthOf::ZERO);
        tree = tree.children(actor, [first_child, last_child]);
        tree.insert_input(actor, LayoutInputOf::box_input(input));
        for (node, row, block) in [(first_child, 1, 20.0), (last_child, 2, 30.0)] {
            tree = tree.children(node, []).style(
                node,
                NodeInputOf {
                    display: Display::Flex,
                    writing_mode: mode,
                    direction: Direction::Ltr,
                    box_sizing: BoxSizing::BorderBox,
                    size: physical_size(mode, 40.0, block).map(PreferredSizeOf::px),
                    grid_column: GridPlacement::try_lines(1, 2).unwrap(),
                    grid_row: GridPlacement::try_lines(row, row + 1).unwrap(),
                    ..NodeInputOf::default()
                },
            );
        }
    }
    let batch = completed(&tree);
    let expected_body = physical_size(mode, 40.0, 50.0);
    let mut actor_ports = Vec::new();
    for (actor, first_child, last_child) in [(10, 30, 31), (11, 32, 33)] {
        assert_eq!(normal_box(&batch, actor).size, expected_body);
        assert_eq!(normal_box(&batch, actor).border, Edges::ZERO);
        assert_eq!(normal_box(&batch, actor).padding, Edges::ZERO);
        assert_eq!(
            normal_box(&batch, actor).margin,
            block_edges(mode, value(3.0), value(5.0), S::ZERO)
        );
        for (node, block) in [(first_child, 20.0), (last_child, 30.0)] {
            assert_eq!(Traverse::child_count(&tree, node), 0);
            assert!(!LayoutTree::has_leaf_measurement(&tree, node));
            assert_eq!(
                normal_box(&batch, node).size,
                physical_size(mode, 40.0, block)
            );
        }
        // In this real VerticalRl actor, row20 is physically30..50 and row30
        // is0..30. Empty fixed Flex children select the established physical
        // line-under border port0. The actor's real ports must therefore be
        // First30 and Last0, with Last0 present rather than synthesized/First.
        assert_eq!(normal_box(&batch, first_child).location.x, value(30.0));
        assert_eq!(normal_box(&batch, last_child).location.x, S::ZERO);
        assert_eq!(
            selected_port(&batch, first_child, axes, SelectedPort::First),
            S::ZERO
        );
        assert_eq!(
            selected_port(&batch, last_child, axes, SelectedPort::Last),
            S::ZERO
        );

        // Independent fixed-body port qualification, NOT a receipt from the
        // original placement request: native performed nonleaf requests need
        // not enter the host cache. Use the existing real computation owner.
        // Every actor track/child/edge is fixed; it has no percentages, text,
        // wrapping, inherited tracks or outer owner targets. Thus fixing the
        // actual border determines its own internal layout/ports independently
        // of the outer available track span. The real immediate parent is2.
        assert!(Traverse::children(&tree, 2).any(|child| child == actor));
        let parent = normal_box(&batch, 2).content_box_size();
        let parent_input = LayoutTree::node_input(&tree, 2);
        let request = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            expected_body.map(Some),
            parent.map(Some),
            ContainingLayoutContext::new(
                FlowAxes::new(parent_input.writing_mode, parent_input.direction),
                ParentFormattingContext::Grid,
            ),
            expected_body.map(AvailableOf::definite),
        );
        let mut native = crate::engine::ComputeSession::new(&tree, Vec::new());
        let actual = crate::engine::contracts::Compute::compute_child(&mut native, actor, request)
            .expect("the genuine fixed native actor performs its own layout");
        assert_eq!(actual.size, expected_body);
        assert_eq!(actual.size, normal_box(&batch, actor).size);
        let first = actual
            .baselines()
            .first_block(axes)
            .expect("the composite native Grid supplies its actual First port");
        let last = actual
            .baselines()
            .last_block(axes)
            .expect("the composite native Grid supplies Last0, not First fallback");
        assert_eq!(first, value(30.0));
        assert_eq!(last, S::ZERO);
        assert_ne!(first, last);
        actor_ports.push((actor, first, last));
    }
    // Qualify all real selected ports and bodies before physical alignment.
    let root_first = selected_port(&batch, 20, axes, SelectedPort::First);
    let root_last = selected_port(&batch, 21, axes, SelectedPort::Last);
    let first_actor = actor_ports.iter().find(|row| row.0 == 10).unwrap().1;
    let last_actor = actor_ports.iter().find(|row| row.0 == 11).unwrap().2;
    let root_first_origin = absolute_origin(&tree, &batch, 20);
    let root_last_origin = absolute_origin(&tree, &batch, 21);
    let first_actor_origin = absolute_origin(&tree, &batch, 10);
    let last_actor_origin = absolute_origin(&tree, &batch, 11);
    assert_eq!(
        first_actor_origin.x + first_actor,
        root_first_origin.x + root_first,
        "actual native First30 participates in the shared physical First baseline"
    );
    assert_eq!(
        last_actor_origin.x + last_actor,
        root_last_origin.x + root_last,
        "actual supplied Last0 participates independently in the shared physical Last baseline"
    );
}

#[test]
fn supplied_native_composite_first_and_last_zero_f32() {
    fixed_composite_ports_and_alignment::<f32>();
}
#[test]
fn supplied_native_composite_first_and_last_zero_f64() {
    fixed_composite_ports_and_alignment::<f64>();
}
