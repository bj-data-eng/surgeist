use super::fixtures::subgrid_track_of;
use super::subgrid_border_basis::{assert_layout, expect, lines, standalone_request};
use super::*;
use crate::test_support::readback::{self, Phase};

// CSS Grid Level 2, section 9 (pinned edition CRD-css-grid-2-20250326): a
// subgrid "acts as if it was completely empty for track sizing purposes" in
// its subgridded dimension, and it "is always stretched in its subgridded
// dimension(s)": its align-self and justify-self are ignored, "as are any
// specified width/height constraints". So in a subgridded axis a subgrid has
// the size of the area its tracks span, whatever width, height, min-width,
// min-height, max-width or max-height it carries; in an axis it does not
// subgrid, its sizes apply as for any grid item. WebKit (Safari 26.5,
// Playwright build 2327) gives the same boxes for every case below.
//
// Every layout runs at a 100px definite available width and an indefinite
// block size. Node 0 is the Grid, node 1 the subgrid under test (the only item
// of node 0, in its first row and first column) and node 2 a leaf item of
// node 1, 30px wide and 10px tall unless a case says otherwise.

#[derive(Clone, Copy)]
enum Subgridded {
    Rows,
    Columns,
    Both,
}

fn leaf_style<S: LayoutScalar>(width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..NodeInputOf::default()
    }
}

// Node 0 is a Grid whose rows or columns `root` may fix. A column-subgrid case
// packs the columns to the start so that a content-sized column is not
// stretched to the 100px width.
fn tree<S: LayoutScalar>(
    subgridded: Subgridded,
    root: impl FnOnce(&mut NodeInputOf<S>),
    subgrid: impl FnOnce(&mut NodeInputOf<S>),
    leaf: Option<NodeInputOf<S>>,
) -> PublicLayoutTreeOf<S> {
    let mut root_style = NodeInputOf::<S> {
        display: Display::Grid,
        ..NodeInputOf::default()
    };
    if matches!(subgridded, Subgridded::Columns) {
        root_style.justify_content = Some(AlignContent::Start);
    }
    root(&mut root_style);
    let mut style = NodeInputOf::<S> {
        display: Display::Grid,
        grid_column: lines(1, 2),
        grid_row: lines(1, 2),
        ..NodeInputOf::default()
    };
    if matches!(subgridded, Subgridded::Rows | Subgridded::Both) {
        style.grid_template_rows = subgrid_track_of();
    }
    if matches!(subgridded, Subgridded::Columns | Subgridded::Both) {
        style.grid_template_columns = subgrid_track_of();
    }
    subgrid(&mut style);
    let mut tree = PublicLayoutTreeOf::new()
        .style(0, root_style)
        .style(1, style)
        .children(0, [1]);
    if let Some(leaf) = leaf {
        tree = tree.style(2, leaf).children(1, [2]);
    }
    tree
}

fn px_track<S: LayoutScalar>(value: f64) -> Vec<TrackComponentOf<S>> {
    vec![TrackComponentOf::px(S::from_f64(value))]
}

fn rows_tree<S: LayoutScalar>(
    root: impl FnOnce(&mut NodeInputOf<S>),
    subgrid: impl FnOnce(&mut NodeInputOf<S>),
) -> PublicLayoutTreeOf<S> {
    tree(
        Subgridded::Rows,
        root,
        subgrid,
        Some(leaf_style(30.0, 10.0)),
    )
}

fn columns_tree<S: LayoutScalar>(
    root: impl FnOnce(&mut NodeInputOf<S>),
    subgrid: impl FnOnce(&mut NodeInputOf<S>),
) -> PublicLayoutTreeOf<S> {
    tree(
        Subgridded::Columns,
        root,
        subgrid,
        Some(leaf_style(30.0, 10.0)),
    )
}

fn nothing<S: LayoutScalar>(_: &mut NodeInputOf<S>) {}

// Each case runs in both scalar lanes: `mod name { f32, f64 }`.
macro_rules! scalar_tests {
    ($name:ident, $body:ident) => {
        mod $name {
            use super::*;

            #[test]
            fn f32() {
                $body::<f32>();
            }

            #[test]
            fn f64() {
                $body::<f64>();
            }
        }
    };
}

// Rows. Node 2 is the one item in the subgrid's row; row 1 is the auto track
// of node 0 and takes the leaf's 10px, since the subgrid contributes nothing
// of its own (it "acts as if it was completely empty"). Node 1 stretches to
// row 1 whatever its height, min-height, max-height or percentage height say,
// so nodes 0, 1 and 2 are 10px tall. The columns are not subgridded: node 1
// is stretched to the 100px width and node 2 is 30 wide at the origin.
fn assert_row_subgrid_ignores<S: LayoutScalar>(
    label: &str,
    subgrid: impl FnOnce(&mut NodeInputOf<S>),
) {
    assert_layout(
        &rows_tree(nothing, subgrid),
        label,
        &[
            expect(0.0, 0.0, 100.0, 10.0),
            expect(0.0, 0.0, 100.0, 10.0),
            expect(0.0, 0.0, 30.0, 10.0),
        ],
    );
}

macro_rules! row_ignores {
    ($name:ident, $label:expr, |$style:ident, $s:ident| $edit:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn f32() {
                type $s = f32;
                assert_row_subgrid_ignores::<f32>($label, |$style| $edit);
            }

            #[test]
            fn f64() {
                type $s = f64;
                assert_row_subgrid_ignores::<f64>($label, |$style| $edit);
            }
        }
    };
}

row_ignores!(
    row_subgrid_ignores_a_specified_height_smaller_than_its_content,
    "height 4",
    |style, S| style.size.height = PreferredSizeOf::px(S::from_f64(4.0))
);
row_ignores!(
    row_subgrid_ignores_a_specified_height_larger_than_its_content,
    "height 50",
    |style, S| style.size.height = PreferredSizeOf::px(S::from_f64(50.0))
);
row_ignores!(
    row_subgrid_ignores_a_percentage_height,
    "height 50%",
    |style, S| style.size.height = PreferredSizeOf::percent(S::from_f64(0.5))
);
row_ignores!(
    row_subgrid_ignores_a_specified_min_height,
    "min-height 50",
    |style, S| style.min_size.height = MinSizeOf::px(S::from_f64(50.0))
);
row_ignores!(
    row_subgrid_ignores_a_specified_max_height,
    "max-height 3",
    |style, S| style.max_size.height = MaxSizeOf::px(S::from_f64(3.0))
);

// Both a height and a min-height together, as the gutter case of the
// negative-extent tests combines them.
row_ignores!(
    row_subgrid_ignores_a_zero_height_with_a_min_height,
    "height 0 and min-height 20",
    |style, S| {
        style.size.height = PreferredSizeOf::px(S::ZERO);
        style.min_size.height = MinSizeOf::px(S::from_f64(20.0));
    }
);

// The align-self of a subgrid is ignored in its subgridded axis (section 9
// above): it stretches to the 40px fixed row instead of centring at 10px, and
// in an auto row it stays the row's 10px.
fn assert_row_subgrid_align_self_is_ignored<S: LayoutScalar>() {
    let centre = |style: &mut NodeInputOf<S>| style.align_self = Some(AlignItems::Center);
    assert_layout(
        &rows_tree(|root| root.grid_template_rows = px_track(40.0), centre),
        "align-self center, 40px row",
        &[
            expect(0.0, 0.0, 100.0, 40.0),
            expect(0.0, 0.0, 100.0, 40.0),
            expect(0.0, 0.0, 30.0, 10.0),
        ],
    );
    assert_layout(
        &rows_tree(nothing, centre),
        "align-self center, auto row",
        &[
            expect(0.0, 0.0, 100.0, 10.0),
            expect(0.0, 0.0, 100.0, 10.0),
            expect(0.0, 0.0, 30.0, 10.0),
        ],
    );
}
scalar_tests!(
    row_subgrid_ignores_its_align_self,
    assert_row_subgrid_align_self_is_ignored
);

// A fixed 30px row: the subgrid is 30px tall whatever it specifies.
fn assert_row_subgrid_in_a_fixed_row<S: LayoutScalar>() {
    for (label, edit) in [
        (
            "height 4",
            (|style: &mut NodeInputOf<S>| style.size.height = PreferredSizeOf::px(S::from_f64(4.0)))
                as fn(&mut NodeInputOf<S>),
        ),
        ("min-height 50", |style: &mut NodeInputOf<S>| {
            style.min_size.height = MinSizeOf::px(S::from_f64(50.0));
        }),
        ("max-height 3", |style: &mut NodeInputOf<S>| {
            style.max_size.height = MaxSizeOf::px(S::from_f64(3.0));
        }),
    ] {
        assert_layout(
            &rows_tree(|root| root.grid_template_rows = px_track(30.0), edit),
            label,
            &[
                expect(0.0, 0.0, 100.0, 30.0),
                expect(0.0, 0.0, 100.0, 30.0),
                expect(0.0, 0.0, 30.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    row_subgrid_in_a_fixed_row_ignores_its_height_bounds,
    assert_row_subgrid_in_a_fixed_row
);

// An empty row subgrid (no items) is 0px tall whatever height it specifies.
fn assert_empty_row_subgrid_ignores_its_height<S: LayoutScalar>() {
    assert_layout(
        &tree(
            Subgridded::Rows,
            nothing,
            |style| style.size.height = PreferredSizeOf::px(S::from_f64(4.0)),
            None,
        ),
        "empty, height 4",
        &[expect(0.0, 0.0, 100.0, 0.0), expect(0.0, 0.0, 100.0, 0.0)],
    );
}
scalar_tests!(
    empty_row_subgrid_ignores_its_height,
    assert_empty_row_subgrid_ignores_its_height
);

// Aspect ratio. The pinned text does not name it; WebKit keeps the stretched
// subgridded block extent (10px) rather than deriving a 100px height from the
// 100px stretched width, so that extent is asserted and the width, which the
// ratio may transfer, is not.
fn assert_row_subgrid_keeps_its_stretched_height_under_an_aspect_ratio<S: LayoutScalar>() {
    let tree = rows_tree(nothing, |style: &mut NodeInputOf<S>| {
        style.aspect_ratio = AspectRatioOf::new(S::ONE);
    });
    let batch = compute_layout(&tree, 0, standalone_request())
        .unwrap_or_else(|error| panic!("layout failed: {error:?}"));
    for node in 0..2 {
        let output = readback::single_box_output(&batch, node, Phase::Unrounded);
        assert_eq!(output.size.height.to_f64(), 10.0, "node {node} height");
    }
}
scalar_tests!(
    row_subgrid_keeps_its_stretched_height_under_an_aspect_ratio,
    assert_row_subgrid_keeps_its_stretched_height_under_an_aspect_ratio
);

// Controls for the rows: the width axis is not subgridded, so the subgrid's
// width, min-width and max-width apply as for any item. A definite width puts
// the item at the start of its column; a minimum below the stretched width
// does not change it.
fn assert_row_subgrid_keeps_its_width_bounds<S: LayoutScalar>() {
    let cases: [(&str, fn(&mut NodeInputOf<S>), f64); 3] = [
        (
            "width 40",
            |style| style.size.width = PreferredSizeOf::px(S::from_f64(40.0)),
            40.0,
        ),
        (
            "max-width 20",
            |style| style.max_size.width = MaxSizeOf::px(S::from_f64(20.0)),
            20.0,
        ),
        (
            "min-width 80",
            |style| style.min_size.width = MinSizeOf::px(S::from_f64(80.0)),
            100.0,
        ),
    ];
    for (label, edit, width) in cases {
        assert_layout(
            &rows_tree(nothing, edit),
            label,
            &[
                expect(0.0, 0.0, 100.0, 10.0),
                expect(0.0, 0.0, width, 10.0),
                expect(0.0, 0.0, 30.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    row_subgrid_keeps_its_width_bounds,
    assert_row_subgrid_keeps_its_width_bounds
);

// Columns. The one item is the leaf, 30px wide; node 0 packs its columns to the
// start, so column 1 is the leaf's 30px and node 1 is 30px wide whatever its
// width, min-width or max-width say. Rows are not subgridded: node 1 and node 0
// are the leaf's 10px tall.
fn assert_column_subgrid_ignores<S: LayoutScalar>() {
    let cases: [(&str, fn(&mut NodeInputOf<S>)); 4] = [
        ("width 80", |style| {
            style.size.width = PreferredSizeOf::px(S::from_f64(80.0))
        }),
        ("width 5", |style| {
            style.size.width = PreferredSizeOf::px(S::from_f64(5.0))
        }),
        ("min-width 80", |style| {
            style.min_size.width = MinSizeOf::px(S::from_f64(80.0))
        }),
        ("max-width 10", |style| {
            style.max_size.width = MaxSizeOf::px(S::from_f64(10.0))
        }),
    ];
    for (label, edit) in cases {
        assert_layout(
            &columns_tree(nothing, edit),
            label,
            &[
                expect(0.0, 0.0, 100.0, 10.0),
                expect(0.0, 0.0, 30.0, 10.0),
                expect(0.0, 0.0, 30.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    column_subgrid_ignores_its_width_bounds,
    assert_column_subgrid_ignores
);

// A fixed 60px column: the subgrid is 60px wide whatever it specifies, and its
// justify-self (centre) does not move it.
fn assert_column_subgrid_in_a_fixed_column<S: LayoutScalar>() {
    let cases: [(&str, fn(&mut NodeInputOf<S>)); 2] = [
        ("width 5", |style| {
            style.size.width = PreferredSizeOf::px(S::from_f64(5.0))
        }),
        ("justify-self center", |style| {
            style.justify_self = Some(AlignItems::Center)
        }),
    ];
    for (label, edit) in cases {
        assert_layout(
            &columns_tree(|root| root.grid_template_columns = px_track(60.0), edit),
            label,
            &[
                expect(0.0, 0.0, 100.0, 10.0),
                expect(0.0, 0.0, 60.0, 10.0),
                expect(0.0, 0.0, 30.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    column_subgrid_in_a_fixed_column_ignores_its_width_and_justify_self,
    assert_column_subgrid_in_a_fixed_column
);

// Controls for the columns: the height axis is not subgridded, so height and
// min-height apply. Node 0 grows to the subgrid's row.
fn assert_column_subgrid_keeps_its_height_bounds<S: LayoutScalar>() {
    let cases: [(&str, fn(&mut NodeInputOf<S>), f64); 2] = [
        (
            "height 4",
            |style| style.size.height = PreferredSizeOf::px(S::from_f64(4.0)),
            4.0,
        ),
        (
            "min-height 50",
            |style| style.min_size.height = MinSizeOf::px(S::from_f64(50.0)),
            50.0,
        ),
    ];
    for (label, edit, height) in cases {
        assert_layout(
            &columns_tree(nothing, edit),
            label,
            &[
                expect(0.0, 0.0, 100.0, height),
                expect(0.0, 0.0, 30.0, height),
                expect(0.0, 0.0, 30.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    column_subgrid_keeps_its_height_bounds,
    assert_column_subgrid_keeps_its_height_bounds
);

// A subgrid in both axes ignores both sets of specified sizes. Node 0's single
// column is 70px, and its row takes the leaf's 10px.
fn assert_both_axes_subgrid_ignores<S: LayoutScalar>() {
    let cases: [(&str, fn(&mut NodeInputOf<S>)); 3] = [
        ("width 40, height 4", |style| {
            style.size = Size::new(
                PreferredSizeOf::px(S::from_f64(40.0)),
                PreferredSizeOf::px(S::from_f64(4.0)),
            );
        }),
        ("min 80 x 50", |style| {
            style.min_size = Size::new(
                MinSizeOf::px(S::from_f64(80.0)),
                MinSizeOf::px(S::from_f64(50.0)),
            );
        }),
        ("max 10 x 3", |style| {
            style.max_size = Size::new(
                MaxSizeOf::px(S::from_f64(10.0)),
                MaxSizeOf::px(S::from_f64(3.0)),
            );
        }),
    ];
    for (label, edit) in cases {
        assert_layout(
            &tree(
                Subgridded::Both,
                |root| root.grid_template_columns = px_track(70.0),
                edit,
                Some(leaf_style(20.0, 10.0)),
            ),
            label,
            &[
                expect(0.0, 0.0, 100.0, 10.0),
                expect(0.0, 0.0, 70.0, 10.0),
                expect(0.0, 0.0, 20.0, 10.0),
            ],
        );
    }
}
scalar_tests!(
    subgrid_in_both_axes_ignores_its_specified_sizes,
    assert_both_axes_subgrid_ignores
);

// A scroll container reserves a scrollbar gutter inside its border box
// (CSS Overflow 3); the reservation is clamped to the box. In a subgridded axis
// a subgrid's border box is the area its tracks span and its specified size is
// ignored (section 9 above), so a width or height smaller than the gutter must
// not clamp the reservation: the layout of a scroll-container subgrid is the
// layout it has without the specified size. That invariance is the oracle; the
// absolute heights the gutter adds are not asserted, because the pinned text
// counts the block-end gutter in the edge contribution (+15px per axis here)
// while WebKit (Safari 26.5, Playwright build 2327) does not.
//
// The subgrid has `overflow: scroll` and a 15px scrollbar. Each case below
// lays the same tree out with and without the specified size and requires the
// same used geometry for every node. The 4px height and 5px width are below
// the 15px gutter; larger sizes, min-sizes and max-sizes are the passing
// companions that the clamp does not reach.
fn scrolled<S: LayoutScalar>(style: &mut NodeInputOf<S>) {
    style.overflow = ComputedOverflow::try_new(Overflow::Scroll, Overflow::Scroll).unwrap();
    style.scrollbar_width = ScrollbarWidthOf::try_new(S::from_f64(15.0)).unwrap();
}

fn used_geometry<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    nodes: u32,
) -> Vec<(f64, f64, f64, f64)> {
    let batch = compute_layout(tree, 0, standalone_request())
        .unwrap_or_else(|error| panic!("layout failed: {error:?}"));
    (0..nodes)
        .map(|node| {
            let output = readback::single_box_output(&batch, node, Phase::Unrounded);
            (
                output.location.x.to_f64(),
                output.location.y.to_f64(),
                output.size.width.to_f64(),
                output.size.height.to_f64(),
            )
        })
        .collect()
}

#[derive(Clone, Copy)]
enum Shape {
    // Node 1, the scroll container, is the only item of node 0.
    Single,
    // Node 1 is a plain subgrid over two rows or columns of node 0, node 2 the
    // scroll container, node 3 an empty subgrid spanning node 2's other axis.
    Chain,
}

fn gutter_tree<S: LayoutScalar>(
    subgridded: Subgridded,
    shape: Shape,
    with_leaf: bool,
    specified: impl FnOnce(&mut NodeInputOf<S>),
) -> (PublicLayoutTreeOf<S>, u32) {
    let rows = matches!(subgridded, Subgridded::Rows);
    let subgrid_style = |column: GridPlacement, row: GridPlacement| {
        let mut style = NodeInputOf::<S> {
            display: Display::Grid,
            grid_column: column,
            grid_row: row,
            ..NodeInputOf::default()
        };
        if rows {
            style.grid_template_rows = subgrid_track_of();
        } else {
            style.grid_template_columns = subgrid_track_of();
        }
        style
    };
    let mut root = NodeInputOf::<S> {
        display: Display::Grid,
        ..NodeInputOf::default()
    };
    if !rows {
        root.justify_content = Some(AlignContent::Start);
    }
    let leaf = leaf_style(30.0, 10.0);
    match shape {
        Shape::Single => {
            let mut scroller = subgrid_style(lines(1, 2), lines(1, 2));
            scrolled(&mut scroller);
            specified(&mut scroller);
            let mut tree = PublicLayoutTreeOf::new()
                .style(0, root)
                .style(1, scroller)
                .children(0, [1]);
            if with_leaf {
                tree = tree.style(2, leaf).children(1, [2]);
            }
            (tree, if with_leaf { 3 } else { 2 })
        }
        Shape::Chain => {
            let (outer, inner, innermost) = if rows {
                (
                    (lines(1, 2), lines(1, 3)),
                    (lines(1, 2), lines(1, 2)),
                    (lines(1, 3), lines(1, 2)),
                )
            } else {
                (
                    (lines(1, 3), lines(1, 2)),
                    (lines(1, 2), lines(1, 2)),
                    (lines(1, 2), lines(1, 3)),
                )
            };
            let mut scroller = subgrid_style(inner.0, inner.1);
            scrolled(&mut scroller);
            specified(&mut scroller);
            let mut tree = PublicLayoutTreeOf::new()
                .style(0, root)
                .style(1, subgrid_style(outer.0, outer.1))
                .style(2, scroller)
                .style(3, subgrid_style(innermost.0, innermost.1))
                .children(0, [1])
                .children(1, [2])
                .children(2, [3]);
            if with_leaf {
                tree = tree.style(4, leaf).children(3, [4]);
            }
            (tree, if with_leaf { 5 } else { 4 })
        }
    }
}

fn assert_specified_size_does_not_change_the_layout<S: LayoutScalar>(
    subgridded: Subgridded,
    specified: fn(&mut NodeInputOf<S>),
) {
    for shape in [Shape::Single, Shape::Chain] {
        for with_leaf in [false, true] {
            let (plain, nodes) = gutter_tree::<S>(subgridded, shape, with_leaf, nothing);
            let (sized, _) = gutter_tree::<S>(subgridded, shape, with_leaf, specified);
            assert_eq!(
                used_geometry(&sized, nodes),
                used_geometry(&plain, nodes),
                "{} {}",
                match shape {
                    Shape::Single => "single",
                    Shape::Chain => "chain",
                },
                if with_leaf { "with a leaf" } else { "empty" },
            );
        }
    }
}

fn assert_height_below_the_gutter_is_ignored<S: LayoutScalar>() {
    assert_specified_size_does_not_change_the_layout::<S>(Subgridded::Rows, |style| {
        style.size.height = PreferredSizeOf::px(S::from_f64(4.0));
    });
}
scalar_tests!(
    scrolled_row_subgrid_ignores_a_height_below_its_gutter,
    assert_height_below_the_gutter_is_ignored
);

fn assert_width_below_the_gutter_is_ignored<S: LayoutScalar>() {
    assert_specified_size_does_not_change_the_layout::<S>(Subgridded::Columns, |style| {
        style.size.width = PreferredSizeOf::px(S::from_f64(5.0));
    });
}
scalar_tests!(
    scrolled_column_subgrid_ignores_a_width_below_its_gutter,
    assert_width_below_the_gutter_is_ignored
);

// Companions: sizes that do not fall below the gutter, and bounds, never
// changed the layout of a scroll-container subgrid.
fn assert_other_sizes_are_ignored<S: LayoutScalar>() {
    let rows: [fn(&mut NodeInputOf<S>); 4] = [
        |style| style.size.height = PreferredSizeOf::px(S::from_f64(50.0)),
        |style| style.size.height = PreferredSizeOf::percent(S::from_f64(0.5)),
        |style| style.min_size.height = MinSizeOf::px(S::from_f64(50.0)),
        |style| style.max_size.height = MaxSizeOf::px(S::from_f64(3.0)),
    ];
    for specified in rows {
        assert_specified_size_does_not_change_the_layout::<S>(Subgridded::Rows, specified);
    }
    let columns: [fn(&mut NodeInputOf<S>); 4] = [
        |style| style.size.width = PreferredSizeOf::px(S::from_f64(80.0)),
        |style| style.size.width = PreferredSizeOf::percent(S::from_f64(0.5)),
        |style| style.min_size.width = MinSizeOf::px(S::from_f64(80.0)),
        |style| style.max_size.width = MaxSizeOf::px(S::from_f64(3.0)),
    ];
    for specified in columns {
        assert_specified_size_does_not_change_the_layout::<S>(Subgridded::Columns, specified);
    }
}
scalar_tests!(
    scrolled_subgrid_ignores_sizes_that_do_not_reach_its_gutter,
    assert_other_sizes_are_ignored
);

// Control, passing before and after the fix: the combination that keeps a
// scroll box saturating its gutter against a border box raised by its own
// border once a subgrid's specified sizes are ignored. Node 2 of the row chain
// has 3px borders and a 15px scrollbar and no specified size. Its stretched
// block extent can be below its 6px border, so the border box is raised to
// 6px and the gutter has no room left in it. Only the facts on which the
// pinned text and WebKit agree are asserted (the spec counts the block-end
// gutter in the edge contribution and WebKit does not, so the heights of
// nodes 0 to 2 are not): nodes 0, 1 and 2 are 100 wide at the origin, and
// node 3, spanning node 2's content box, is 100 - 6 - 15 = 79 wide and 0 tall
// at (3, 3).
fn assert_bordered_scrolled_row_chain<S: LayoutScalar>() {
    let (tree, nodes) = gutter_tree::<S>(Subgridded::Rows, Shape::Chain, false, |style| {
        style.border = Edges::all(LengthOf::px(S::from_f64(3.0)));
    });
    let geometry = used_geometry(&tree, nodes);
    for (node, (x, y, width, _)) in geometry.iter().take(3).enumerate() {
        assert_eq!((*x, *y, *width), (0.0, 0.0, 100.0), "node {node}");
    }
    assert_eq!(geometry[3], (3.0, 3.0, 79.0, 0.0), "node 3");
}
scalar_tests!(
    bordered_scrolled_row_subgrid_chain_lays_out_inside_its_border,
    assert_bordered_scrolled_row_chain
);
