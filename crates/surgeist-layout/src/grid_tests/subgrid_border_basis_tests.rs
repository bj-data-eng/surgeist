use super::fixtures::subgrid_track_of;
use super::*;
use crate::test_support::readback::{self, Phase};

// A chain of subgrids whose middle level has a border or padding. A box is
// never smaller than its own border and padding, and its content box is never
// smaller than zero (CSS Sizing 3 and CSS Box 4), so an extent derived from an
// outer size and an inset is clamped at zero and no percentage basis is ever
// negative.
//
// Every layout below runs at a 100px definite available width and an
// indefinite block size, horizontal-tb and ltr. The root is a Grid with no
// tracks and no size.

fn lines(start: isize, end: isize) -> GridPlacement {
    GridPlacement::try_lines(start, end).unwrap()
}

fn standalone_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    LayoutRootRequestOf::standalone(
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
    )
}

#[derive(Clone, Copy)]
enum Axis {
    Rows,
    Columns,
}

/// Where the box edge insets of the middle subgrid come from.
#[derive(Clone, Copy)]
enum Inset {
    Border(f64),
    Padding(f64),
}

fn subgrid_item<S: LayoutScalar>(
    axis: Axis,
    column: GridPlacement,
    row: GridPlacement,
    inset: Inset,
) -> NodeInputOf<S> {
    let mut style = NodeInputOf::<S> {
        display: Display::Grid,
        grid_column: column,
        grid_row: row,
        ..NodeInputOf::default()
    };
    match axis {
        Axis::Rows => style.grid_template_rows = subgrid_track_of(),
        Axis::Columns => style.grid_template_columns = subgrid_track_of(),
    }
    match inset {
        Inset::Border(width) => style.border = Edges::all(LengthOf::px(S::from_f64(width))),
        Inset::Padding(width) => style.padding = Edges::all(LengthOf::px(S::from_f64(width))),
    }
    style
}

// Node 0: a Grid, no tracks.
// Node 1: item of 0, subgrid along `axis`, spanning both implicit tracks of
//         that axis and the first track of the other.
// Node 2: item of 1, subgrid along `axis`, first track of both axes, with the
//         edge `inset` on every side.
// Node 3: item of 2, no children, subgrid along `axis`, first track of the
//         subgridded axis and both tracks of the other.
fn chain<S: LayoutScalar>(axis: Axis, inset: Inset) -> PublicLayoutTreeOf<S> {
    let none = Inset::Border(0.0);
    let (first, second, third) = match axis {
        Axis::Rows => (
            (lines(1, 2), lines(1, 3)),
            (lines(1, 2), lines(1, 2)),
            (lines(1, 3), lines(1, 2)),
        ),
        Axis::Columns => (
            (lines(1, 3), lines(1, 2)),
            (lines(1, 2), lines(1, 2)),
            (lines(1, 2), lines(1, 3)),
        ),
    };
    PublicLayoutTreeOf::new()
        .style(
            0,
            NodeInputOf {
                display: Display::Grid,
                ..NodeInputOf::default()
            },
        )
        .style(1, subgrid_item(axis, first.0, first.1, none))
        .style(2, subgrid_item(axis, second.0, second.1, inset))
        .style(3, subgrid_item(axis, third.0, third.1, none))
        .children(0, [1])
        .children(1, [2])
        .children(2, [3])
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Expected {
    location: (f64, f64),
    size: (f64, f64),
}

fn expect(x: f64, y: f64, width: f64, height: f64) -> Expected {
    Expected {
        location: (x, y),
        size: (width, height),
    }
}

fn assert_layout<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    label: &str,
    expected: &[Expected],
) {
    let batch = compute_layout(tree, 0, standalone_request())
        .unwrap_or_else(|error| panic!("{label}: layout failed: {error:?}"));
    for (node, expected) in expected.iter().enumerate() {
        let output = readback::single_box_output(&batch, node as u32, Phase::Final);
        let actual = Expected {
            location: (output.location.x.to_f64(), output.location.y.to_f64()),
            size: (output.size.width.to_f64(), output.size.height.to_f64()),
        };
        assert_eq!(actual, *expected, "{label}: node {node}");
    }
}

// Rows (I is the inset on each side of node 2). Authority: CSS Grid Level 2,
// section 9 (pinned edition CRD-css-grid-2-20250326). Section 9 says a subgrid
// "acts as if it was completely empty for track sizing purposes" in its
// subgridded dimension: node 3 (empty, itself a row subgrid) contributes no
// content of its own, and node 2 and node 1 contribute only through the items
// they hold. Node 3 is itself a grid item in the one row track of node 2
// (node 2 spans one of node 1's two tracks, and so one of node 0's). The same
// section says an item in a subgrid's first or last track is sized and
// contributes as if it had the subgrid's margin, border and padding at that
// edge as extra margin, and that the per-edge hypothetical items it also
// describes are a shortcut only when real items sit in those tracks. The track
// is both first and last, so node 3 is a real item with extra margin I at the
// start and I at the end, giving a margin box of 0 + 2I and contributing 2I
// to row 1 under Grid Level 1 section 11.5. Row 2 holds nothing and is 0.
// Reading the hypothetical edge items alone, each inflated by one edge, would
// give I instead; node 0's height would then be I, which a browser does not
// produce: WebKit (Safari 26.5, Playwright build 2327) lays this chain out at
// inset 3 as n0, n1, n2 = 100 x 6 at (0, 0) and n3 = 94 x 0 at (3, 3) relative
// to n2, i.e. rows of 2I. The spec text and that witness agree, so the
// expectations below follow them.
//
// Nodes 0, 1 and 2 are therefore 2I tall (node 2 stretches to row 1, which
// section 9 requires of a subgrid in its subgridded dimension), and their
// widths are the single auto column stretched to the 100px available width.
// Node 2 is not a column subgrid, so it lays out an own column grid over its
// content box (100 - 2I wide) in which node 3 spans two auto tracks. Node 3 is
// positioned by its extra margin I at (I, I) and, since 2I - 2I leaves it no
// block extent, is 0 tall.
fn row_chain_expectations(inset: f64) -> [Expected; 4] {
    let inner = 100.0 - 2.0 * inset;
    [
        expect(0.0, 0.0, 100.0, 2.0 * inset),
        expect(0.0, 0.0, 100.0, 2.0 * inset),
        expect(0.0, 0.0, 100.0, 2.0 * inset),
        expect(inset, inset, inner, 0.0),
    ]
}

fn assert_row_chain<S: LayoutScalar>(inset: Inset, amount: f64, label: &str) {
    assert_layout(
        &chain::<S>(Axis::Rows, inset),
        label,
        &row_chain_expectations(amount),
    );
}

#[test]
fn row_subgrid_chain_with_a_border_lays_out_f32() {
    assert_row_chain::<f32>(Inset::Border(3.0), 3.0, "border 3");
    assert_row_chain::<f32>(Inset::Border(0.5), 0.5, "border 0.5");
    assert_row_chain::<f32>(Inset::Border(10.0), 10.0, "border 10");
}

#[test]
fn row_subgrid_chain_with_a_border_lays_out_f64() {
    assert_row_chain::<f64>(Inset::Border(3.0), 3.0, "border 3");
    assert_row_chain::<f64>(Inset::Border(0.5), 0.5, "border 0.5");
    assert_row_chain::<f64>(Inset::Border(10.0), 10.0, "border 10");
}

#[test]
fn row_subgrid_chain_with_padding_lays_out_f32() {
    assert_row_chain::<f32>(Inset::Padding(3.0), 3.0, "padding 3");
}

#[test]
fn row_subgrid_chain_with_padding_lays_out_f64() {
    assert_row_chain::<f64>(Inset::Padding(3.0), 3.0, "padding 3");
}

#[test]
fn row_subgrid_chain_without_an_inset_lays_out_with_zero_height_f32() {
    assert_row_chain::<f32>(Inset::Border(0.0), 0.0, "border 0");
}

#[test]
fn row_subgrid_chain_without_an_inset_lays_out_with_zero_height_f64() {
    assert_row_chain::<f64>(Inset::Border(0.0), 0.0, "border 0");
}

// Columns. The same chain along the inline axis: node 0 has two implicit
// auto columns, node 1 spans both and node 2 occupies the first. As in the
// rows, node 3 is a real item in the one column track of node 2 and has the
// extra margin I at both edges (Grid Level 2 section 9), so it contributes 2I
// to column 1 and nothing to column 2: base sizes 2I and 0. The 100px of free
// space beyond them, 100 - 2I, is split equally between the two auto tracks by
// Grid Level 1 section 11.8 (stretch auto tracks, align-content normal):
// column 1 is 2I + (100 - 2I) / 2 = 50 + I and column 2 is 50 - I. Node 2
// fills column 1; node 3, in the first of node 2's columns, gets that column
// less its extra margin I + I: 50 + I - 2I = 50 - I, at x = I. Rows are
// ordinary: node 2 is 2I tall because node 3 (two implicit rows, no content) is
// 0 tall, and so are nodes 0 and 1. WebKit (Safari 26.5, Playwright build
// 2327) at inset 3 gives n0 and n1 = 100 x 6, n2 = 53 x 6, and n3 = 47 x 0 at
// (3, 3) relative to n2, for border and for padding.
fn column_chain_expectations(inset: f64) -> [Expected; 4] {
    [
        expect(0.0, 0.0, 100.0, 2.0 * inset),
        expect(0.0, 0.0, 100.0, 2.0 * inset),
        expect(0.0, 0.0, 50.0 + inset, 2.0 * inset),
        expect(inset, inset, 50.0 - inset, 0.0),
    ]
}

fn assert_column_chain<S: LayoutScalar>(inset: Inset, amount: f64, label: &str) {
    assert_layout(
        &chain::<S>(Axis::Columns, inset),
        label,
        &column_chain_expectations(amount),
    );
}

#[test]
fn column_subgrid_chain_with_a_border_lays_out_f32() {
    assert_column_chain::<f32>(Inset::Border(3.0), 3.0, "border 3");
}

#[test]
fn column_subgrid_chain_with_a_border_lays_out_f64() {
    assert_column_chain::<f64>(Inset::Border(3.0), 3.0, "border 3");
}

#[test]
fn column_subgrid_chain_with_padding_lays_out_f32() {
    assert_column_chain::<f32>(Inset::Padding(3.0), 3.0, "padding 3");
}

#[test]
fn column_subgrid_chain_with_padding_lays_out_f64() {
    assert_column_chain::<f64>(Inset::Padding(3.0), 3.0, "padding 3");
}

#[test]
fn column_subgrid_chain_without_an_inset_lays_out_with_zero_height_f32() {
    assert_column_chain::<f32>(Inset::Border(0.0), 0.0, "border 0");
}

#[test]
fn column_subgrid_chain_without_an_inset_lays_out_with_zero_height_f64() {
    assert_column_chain::<f64>(Inset::Border(0.0), 0.0, "border 0");
}

// A leaf under the innermost subgrid, 10px tall and half as wide as its
// containing block. Node 4 is the leaf, in node 3's own column grid (a single
// implicit auto column, so its containing block is node 3's content width,
// 100 - 2I). The leaf is now a real item in the one row track of node 3,
// which maps to node 2's one track; the extra margin of node 2 (I at each
// edge, Grid Level 2 section 9) accumulates onto it, so row 1 is 10 + 2I and
// node 3 receives row 1 less that extra margin: 10px, at y = I. The leaf's
// width is 50% of (100 - 2I). WebKit (Safari 26.5, Playwright build 2327) at
// inset 3: n0, n1, n2 = 100 x 16, n3 = 94 x 10 at (3, 3), n4 = 47 x 10 at
// (0, 0) relative to n3.
fn assert_row_chain_with_leaf<S: LayoutScalar>(inset: f64) {
    let leaf = NodeInputOf::<S> {
        size: Size::new(
            PreferredSizeOf::percent(S::from_f64(0.5)),
            PreferredSizeOf::px(S::from_f64(10.0)),
        ),
        ..NodeInputOf::default()
    };
    let tree = chain::<S>(Axis::Rows, Inset::Border(inset))
        .style(4, leaf)
        .children(3, [4]);
    let inner = 100.0 - 2.0 * inset;
    assert_layout(
        &tree,
        &format!("leaf, border {inset}"),
        &[
            expect(0.0, 0.0, 100.0, 10.0 + 2.0 * inset),
            expect(0.0, 0.0, 100.0, 10.0 + 2.0 * inset),
            expect(0.0, 0.0, 100.0, 10.0 + 2.0 * inset),
            expect(inset, inset, inner, 10.0),
            expect(0.0, 0.0, inner / 2.0, 10.0),
        ],
    );
}

#[test]
fn row_subgrid_chain_with_a_percentage_wide_leaf_lays_out_f32() {
    assert_row_chain_with_leaf::<f32>(3.0);
}

#[test]
fn row_subgrid_chain_with_a_percentage_wide_leaf_lays_out_f64() {
    assert_row_chain_with_leaf::<f64>(3.0);
}

#[test]
fn row_subgrid_chain_with_a_percentage_wide_leaf_and_no_border_lays_out_f32() {
    assert_row_chain_with_leaf::<f32>(0.0);
}

#[test]
fn row_subgrid_chain_with_a_percentage_wide_leaf_and_no_border_lays_out_f64() {
    assert_row_chain_with_leaf::<f64>(0.0);
}

// Control: node 3 spans only the first of two tracks of node 2, so the
// container's extent is not its area and no area extent is taken from it.
// Node 2 spans both implicit rows of node 1. Its top inset I joins row 1 and
// its bottom inset I joins row 2 (Grid Level 2 section 9), each row otherwise
// empty: rows are I and I and node 2 is 2I tall. Node 3 in row 1 gets I less
// node 2's top inset: 0, at y = I. Its width spans node 2's two implicit
// columns (100 - 2I).
fn assert_row_chain_over_two_rows<S: LayoutScalar>(inset: f64) {
    let tree = chain::<S>(Axis::Rows, Inset::Border(inset))
        .style(
            2,
            subgrid_item(Axis::Rows, lines(1, 2), lines(1, 3), Inset::Border(inset)),
        )
        .style(
            3,
            subgrid_item(Axis::Rows, lines(1, 3), lines(1, 2), Inset::Border(0.0)),
        );
    let inner = 100.0 - 2.0 * inset;
    assert_layout(
        &tree,
        &format!("two rows, border {inset}"),
        &[
            expect(0.0, 0.0, 100.0, 2.0 * inset),
            expect(0.0, 0.0, 100.0, 2.0 * inset),
            expect(0.0, 0.0, 100.0, 2.0 * inset),
            expect(inset, inset, inner, 0.0),
        ],
    );
}

#[test]
fn row_subgrid_over_two_rows_with_a_border_lays_out_f32() {
    assert_row_chain_over_two_rows::<f32>(3.0);
}

#[test]
fn row_subgrid_over_two_rows_with_a_border_lays_out_f64() {
    assert_row_chain_over_two_rows::<f64>(3.0);
}
