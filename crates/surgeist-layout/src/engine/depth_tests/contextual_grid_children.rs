use super::*;
use crate::{
    AlignContent, AlignItems, GridAxisKind, GridLanesFlow, GridPlacement, MinSizeOf,
    PreferredSizeOf, SubgridTrack, TrackComponentOf,
};

fn subgrid<S: LayoutScalar>() -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::Subgrid(SubgridTrack::new(Vec::new()))]
}

fn fixed_tracks<S: LayoutScalar>(sizes: [f64; 2]) -> Vec<TrackComponentOf<S>> {
    sizes
        .map(|size| TrackComponentOf::px(S::from_f64(size)))
        .into()
}

fn px_size<S: LayoutScalar>(width: f64, height: f64) -> Size<PreferredSizeOf<S>> {
    Size::new(width, height).map(|value| PreferredSizeOf::px(S::from_f64(value)))
}

/// Where each chained subgrid sits in its parent's explicit tracks.
#[derive(Clone, Copy)]
enum Span {
    /// Both tracks of both axes; the subgrid covers the parent's whole area.
    Whole,
    /// The second column track and the first row track only.
    SecondColumnFirstRow,
}

impl Span {
    fn placement(self) -> (GridPlacement, GridPlacement) {
        match self {
            Self::Whole => (
                GridPlacement::try_lines(1, 3).unwrap(),
                GridPlacement::try_lines(1, 3).unwrap(),
            ),
            Self::SecondColumnFirstRow => (
                GridPlacement::try_lines(2, 3).unwrap(),
                GridPlacement::try_lines(1, 2).unwrap(),
            ),
        }
    }
}

// Grid chain. Node 0 is a 70x70 Grid with fixed 30px and 40px columns and
// fixed 20px and 50px rows. Nodes 1..=depth are Grid subgrids in both axes,
// each the only item of its parent. The last subgrid holds two stretched
// leaves, `depth + 1` in the first inherited track of each axis and
// `depth + 2` in the last. Every track size is explicit, so no automatic
// track or intrinsic contribution is involved.
fn subgrid_grid_chain<S: LayoutScalar>(depth: u32, span: Span) -> PublicLayoutTreeOf<S> {
    let (column, row) = span.placement();
    let (leaf_column, leaf_row) = match span {
        Span::Whole => (
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(2).unwrap(),
            ],
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(2).unwrap(),
            ],
        ),
        // One inherited track per axis, so both leaves share it.
        Span::SecondColumnFirstRow => (
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(1).unwrap(),
            ],
            [
                GridPlacement::try_line(1).unwrap(),
                GridPlacement::try_line(1).unwrap(),
            ],
        ),
    };
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 70.0),
            grid_template_columns: fixed_tracks([30.0, 40.0]),
            grid_template_rows: fixed_tracks([20.0, 50.0]),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        // Only the outermost subgrid is placed by the explicit span. Its
        // descendants fill the inherited tracks of the level above, which is
        // the whole of them for `Whole` and the single track otherwise.
        let (grid_column, grid_row) = if node == 1 {
            (column, row)
        } else {
            match span {
                Span::Whole => (column, row),
                Span::SecondColumnFirstRow => (
                    GridPlacement::try_lines(1, 2).unwrap(),
                    GridPlacement::try_lines(1, 2).unwrap(),
                ),
            }
        };
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::Grid,
                grid_column,
                grid_row,
                grid_template_columns: subgrid(),
                grid_template_rows: subgrid(),
                ..NodeInputOf::default()
            },
        );
        tree = tree.children(node - 1, [node]);
    }
    for (index, leaf) in [depth + 1, depth + 2].into_iter().enumerate() {
        tree = tree.style(
            leaf,
            NodeInputOf {
                grid_column: leaf_column[index],
                grid_row: leaf_row[index],
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2])
}

fn assert_box<S: LayoutScalar>(
    batch: &crate::CompletedLayoutBatchOf<u32, S>,
    node: u32,
    location: (f64, f64),
    size: (f64, f64),
) {
    let (_, layout) = batch
        .node_layouts()
        .find(|(candidate, _)| *candidate == node)
        .unwrap_or_else(|| panic!("node {node} is published"));
    let output = layout.single_box().expect("visible box").final_output();
    assert_eq!(
        (output.location.x, output.location.y),
        (S::from_f64(location.0), S::from_f64(location.1)),
        "node {node} location"
    );
    assert_eq!(
        (output.size.width, output.size.height),
        (S::from_f64(size.0), S::from_f64(size.1)),
        "node {node} size"
    );
}

// CSS Grid Level 2, section 9: a subgrid takes the tracks of its parent
// that its grid area spans, and its own items are placed on those tracks.
// A subgrid with the default stretch alignment fills its grid area, so its
// border box is that area; a stretched item fills the area of its tracks.
fn assert_subgrid_grid_chain<S: LayoutScalar>(depth: u32, span: Span) {
    let tree = subgrid_grid_chain::<S>(depth, span);
    let batch = compute_layout(&tree, 0, standalone()).expect("subgrid chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 70.0));
    match span {
        Span::Whole => {
            for node in 1..=depth {
                assert_box(&batch, node, (0.0, 0.0), (70.0, 70.0));
            }
            // The leaves take the first and the second inherited column and row.
            assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
            assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 50.0));
        }
        Span::SecondColumnFirstRow => {
            // The outermost subgrid covers column 2 and row 1: x 30, 40x20.
            assert_box(&batch, 1, (30.0, 0.0), (40.0, 20.0));
            for node in 2..=depth {
                assert_box(&batch, node, (0.0, 0.0), (40.0, 20.0));
            }
            // Both leaves share the single inherited track in each axis.
            assert_box(&batch, depth + 1, (0.0, 0.0), (40.0, 20.0));
            assert_box(&batch, depth + 2, (0.0, 0.0), (40.0, 20.0));
        }
    }
}

#[test]
fn shallow_subgrid_grid_chain_inherits_parent_tracks() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_subgrid_grid_chain_inherits_parent_tracks",
        || {
            for span in [Span::Whole, Span::SecondColumnFirstRow] {
                assert_subgrid_grid_chain::<f32>(3, span);
                assert_subgrid_grid_chain::<f64>(3, span);
            }
        },
    );
}

#[test]
fn deep_subgrid_grid_chain_inherits_parent_tracks_on_bounded_stack() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::deep_subgrid_grid_chain_inherits_parent_tracks_on_bounded_stack",
        || {
            for span in [Span::Whole, Span::SecondColumnFirstRow] {
                assert_subgrid_grid_chain::<f32>(10, span);
                assert_subgrid_grid_chain::<f64>(10, span);
            }
        },
    );
}

// Grid Lanes chain. Node 0 is a 50x14 Grid Lanes container with fixed 20px
// and 30px lanes in the column axis. Nodes 1..=depth are Grid Lanes
// subgrids in the lane axis, each the only item of its parent, spanning both
// lanes. The last holds three leaves, 20px wide and 5px, 7px and 9px high.
fn subgrid_lanes_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let flow = GridLanesFlow::new(GridAxisKind::Column, false, false);
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::GridLanes,
            grid_lanes_flow: flow,
            size: px_size(50.0, 14.0),
            grid_template_columns: fixed_tracks([20.0, 30.0]),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree.style(
            node,
            NodeInputOf {
                display: Display::GridLanes,
                grid_lanes_flow: flow,
                grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                size: Size::new(
                    PreferredSizeOf::AUTO,
                    PreferredSizeOf::px(S::from_f64(14.0)),
                ),
                grid_template_columns: subgrid(),
                justify_content: Some(AlignContent::Start),
                align_content: Some(AlignContent::Start),
                ..NodeInputOf::default()
            },
        );
        tree = tree.children(node - 1, [node]);
    }
    for (index, height) in [5.0, 7.0, 9.0].into_iter().enumerate() {
        let leaf = depth + 1 + index as u32;
        tree = tree.style(
            leaf,
            NodeInputOf {
                size: px_size(20.0, height),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2, depth + 3])
}

// CSS Grid Level 3, Grid Lanes: each item is placed in the lane with the
// least used extent, the first such lane on ties. The 5px item takes lane 1
// and the 7px item lane 2, both being empty at the time. The 9px item then
// goes below the 5px item in lane 1, whose extent 5 is less than 7. A 20px
// item starts at its lane's start edge; lane 2 starts at 20px.
fn assert_subgrid_lanes_chain<S: LayoutScalar>(depth: u32) {
    let tree = subgrid_lanes_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("subgrid lanes chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 4);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 14.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (50.0, 14.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (20.0, 5.0));
    assert_box(&batch, depth + 2, (20.0, 0.0), (20.0, 7.0));
    assert_box(&batch, depth + 3, (0.0, 5.0), (20.0, 9.0));
}

#[test]
fn shallow_subgrid_lanes_chain_inherits_parent_lanes() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_subgrid_lanes_chain_inherits_parent_lanes",
        || {
            assert_subgrid_lanes_chain::<f32>(3);
            assert_subgrid_lanes_chain::<f64>(3);
        },
    );
}

// One-axis Grid chain. Node 0 is a 70x40 Grid with fixed 30px and 40px
// columns and one fixed 40px row. Nodes 1..=depth are Grid items that inherit
// only the column axis (`grid-template-columns: subgrid`); their rows are
// their own fixed 20px and 30px tracks. Each has `min-height: min-content`
// and `align-self: start`. Every level spans both
// inherited columns and its parent's first row. The last level holds two
// stretched leaves, `depth + 1` in the first column and row and `depth + 2`
// in the second. The non-inherited row axis makes each level a standalone
// Grid for its minimum.
fn standalone_minimum_chain<S: LayoutScalar>(depth: u32) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new().style(
        0,
        NodeInputOf {
            display: Display::Grid,
            size: px_size(70.0, 40.0),
            grid_template_columns: fixed_tracks([30.0, 40.0]),
            grid_template_rows: vec![TrackComponentOf::px(S::from_f64(40.0))],
            ..NodeInputOf::default()
        },
    );
    for node in 1..=depth {
        tree = tree
            .style(
                node,
                NodeInputOf {
                    display: Display::Grid,
                    grid_column: GridPlacement::try_lines(1, 3).unwrap(),
                    grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                    align_self: Some(AlignItems::Start),
                    min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::MIN_CONTENT),
                    grid_template_columns: subgrid(),
                    grid_template_rows: fixed_tracks([20.0, 30.0]),
                    ..NodeInputOf::default()
                },
            )
            .children(node - 1, [node]);
    }
    for (leaf, line) in [(depth + 1, 1), (depth + 2, 2)] {
        tree = tree.style(
            leaf,
            NodeInputOf {
                grid_column: GridPlacement::try_line(line).unwrap(),
                grid_row: GridPlacement::try_line(line).unwrap(),
                ..NodeInputOf::default()
            },
        );
    }
    tree.children(depth, [depth + 1, depth + 2])
}

// CSS Grid Level 2, section 9: a subgrid takes the tracks of its parent that
// it spans in each inherited axis, here both columns, and sizes the other axis
// with its own tracks. An auto-height Grid container with `align-self: start`
// is as high as its tracks, 20px plus 30px, and the min-content block size of
// a Grid is that same 50px (CSS Sizing 3, section 5.1), so every level is
// 50px high. Its width is the 70px inherited span, stretched.
fn assert_standalone_minimum_chain<S: LayoutScalar>(depth: u32) {
    let tree = standalone_minimum_chain::<S>(depth);
    let batch = compute_layout(&tree, 0, standalone()).expect("one-axis subgrid chain layout");
    assert_eq!(batch.node_layouts().len(), depth as usize + 3);
    assert_box(&batch, 0, (0.0, 0.0), (70.0, 40.0));
    for node in 1..=depth {
        assert_box(&batch, node, (0.0, 0.0), (70.0, 50.0));
    }
    assert_box(&batch, depth + 1, (0.0, 0.0), (30.0, 20.0));
    assert_box(&batch, depth + 2, (30.0, 20.0), (40.0, 30.0));
}

#[test]
fn shallow_one_axis_subgrid_chain_clamps_to_standalone_minimum() {
    on_bounded_stack(
        "engine::depth_tests::contextual_grid_children::shallow_one_axis_subgrid_chain_clamps_to_standalone_minimum",
        || {
            assert_standalone_minimum_chain::<f32>(3);
            assert_standalone_minimum_chain::<f64>(3);
        },
    );
}
