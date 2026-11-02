//! Relative visual placement must not advance normal block flow.
//! Real public computation checks normal flow and visual geometry independently.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn px<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .unrounded_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}
fn assert_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    position: (f64, f64),
    size: (f64, f64),
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let actual = entries
            .iter()
            .find(|entry| entry.node() == node)
            .unwrap()
            .output();
        assert_eq!(
            actual.location,
            Point::new(px::<S>(position.0), px::<S>(position.1)),
            "node {node}"
        );
        assert_eq!(
            actual.size,
            Size::new(px::<S>(size.0), px::<S>(size.1)),
            "node {node}"
        );
    }
}
fn compute<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(
        tree,
        0,
        LayoutRootRequestOf::viewport(Size::new(
            AvailableOf::definite(px::<S>(50.0)),
            AvailableOf::MAX_CONTENT,
        ))
        .unwrap(),
    )
    .unwrap()
}
fn root_style<S: LayoutScalar>(direction: Direction) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        direction,
        size: Size::new(PreferredSizeOf::px(px::<S>(50.0)), PreferredSizeOf::AUTO),
        ..NodeInputOf::default()
    }
}
fn ten_box<S: LayoutScalar>(
    display: Display,
    direction: Direction,
    box_sizing: BoxSizing,
    decorated: bool,
) -> NodeInputOf<S> {
    // A real 10px border box in both sizing modes. With decoration each axis
    // has border1 + padding1 on both sides; content-box height is therefore6.
    let height = if decorated && box_sizing == BoxSizing::ContentBox {
        6.0
    } else {
        10.0
    };
    NodeInputOf {
        display,
        direction,
        box_sizing,
        position: Position::Relative,
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px::<S>(height))),
        border: Edges::all(LengthOf::px(px::<S>(if decorated { 1.0 } else { 0.0 }))),
        padding: Edges::all(LengthOf::px(px::<S>(if decorated { 1.0 } else { 0.0 }))),
        ..NodeInputOf::default()
    }
}
fn three_boxes<S: LayoutScalar>(
    display: Display,
    direction: Direction,
    box_sizing: BoxSizing,
    decorated: bool,
) -> PublicLayoutTreeOf<S> {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2, 3])
        .style(0, root_style(direction));
    for node in [1, 2, 3] {
        tree = tree
            .children(node, [])
            .style(node, ten_box(display, direction, box_sizing, decorated));
    }
    tree
}
fn replace_inset<S: LayoutScalar>(
    tree: &mut PublicLayoutTreeOf<S>,
    node: u32,
    inset: Edges<LengthAutoOf<S>>,
) {
    let mut style = crate::LayoutTree::node_input(tree, node).clone();
    style.inset = inset;
    tree.insert_input(node, LayoutInputOf::box_input(style));
}
fn top<S: LayoutScalar>(value: f64) -> Edges<LengthAutoOf<S>> {
    Edges {
        top: LengthAutoOf::px(px::<S>(value)),
        ..Edges::all(LengthAutoOf::AUTO)
    }
}
fn assert_unshifted_trio<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    assert_box(batch, 0, (0.0, 0.0), (50.0, 30.0));
    for (node, y) in [(1, 0.0), (2, 10.0), (3, 20.0)] {
        assert_box(batch, node, (0.0, y), (50.0, 10.0));
    }
}

fn inline_size_and_visual_overflow<S: LayoutScalar>(fixed_width: bool) {
    for display in [Display::Block, Display::Flex, Display::Grid] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for offset in [4.0, -4.0] {
                let mut tree = PublicLayoutTreeOf::new()
                    .children(0, [1])
                    .style(
                        0,
                        NodeInputOf {
                            display: Display::Block,
                            direction,
                            size: Size::new(
                                if fixed_width {
                                    PreferredSizeOf::px(px::<S>(10.0))
                                } else {
                                    PreferredSizeOf::AUTO
                                },
                                PreferredSizeOf::AUTO,
                            ),
                            ..NodeInputOf::default()
                        },
                    )
                    .children(1, [])
                    .style(
                        1,
                        NodeInputOf {
                            display,
                            direction,
                            position: Position::Relative,
                            size: Size::new(
                                PreferredSizeOf::px(px::<S>(10.0)),
                                PreferredSizeOf::px(px::<S>(10.0)),
                            ),
                            ..NodeInputOf::default()
                        },
                    );
                let run = |tree: &PublicLayoutTreeOf<S>| {
                    compute_layout(
                        tree,
                        0,
                        LayoutRootRequestOf::viewport(Size::new(
                            AvailableOf::MAX_CONTENT,
                            AvailableOf::MAX_CONTENT,
                        ))
                        .unwrap(),
                    )
                    .unwrap()
                };
                let control = run(&tree);
                assert_box(&control, 0, (0.0, 0.0), (10.0, 10.0));
                assert_box(&control, 1, (0.0, 0.0), (10.0, 10.0));
                replace_inset(
                    &mut tree,
                    1,
                    Edges {
                        left: LengthAutoOf::px(px::<S>(offset)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                );
                let shifted = run(&tree);
                assert_box(&shifted, 0, (0.0, 0.0), (10.0, 10.0));
                assert_box(&shifted, 1, (offset, 0.0), (10.0, 10.0));
                let overflow = output(&shifted, 0)
                    .scroll_geometry
                    .unwrap()
                    .scrollable_overflow();
                assert_eq!(
                    overflow.origin(),
                    Point::new(px::<S>(offset.min(0.0)), S::ZERO),
                    "{display:?}/{direction:?}/offset={offset}/fixed={fixed_width}",
                );
                assert_eq!(
                    overflow.size(),
                    Size::new(px::<S>(14.0), px::<S>(10.0)),
                    "{display:?}/{direction:?}/offset={offset}/fixed={fixed_width}"
                );
            }
        }
    }
}
#[test]
fn relative_flow_auto_inline_size_f32() {
    inline_size_and_visual_overflow::<f32>(false);
}
#[test]
fn relative_flow_auto_inline_size_f64() {
    inline_size_and_visual_overflow::<f64>(false);
}
#[test]
fn relative_flow_fixed_inline_size_overflow_f32() {
    inline_size_and_visual_overflow::<f32>(true);
}
#[test]
fn relative_flow_fixed_inline_size_overflow_f64() {
    inline_size_and_visual_overflow::<f64>(true);
}

// A fixed-size intermediate box does not transfer its descendant's visual
// overflow into its own normal contribution to an auto-sized ancestor.
fn nested_visual_overflow<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for offset in [4.0, -4.0] {
            let fixed_box = NodeInputOf {
                display: Display::Block,
                direction,
                size: Size::new(
                    PreferredSizeOf::px(px::<S>(10.0)),
                    PreferredSizeOf::px(px::<S>(10.0)),
                ),
                ..NodeInputOf::default()
            };
            let mut tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [2])
                .children(2, [])
                .style(
                    0,
                    NodeInputOf {
                        display: Display::Block,
                        direction,
                        ..NodeInputOf::default()
                    },
                )
                .style(1, fixed_box.clone())
                .style(
                    2,
                    NodeInputOf {
                        position: Position::Relative,
                        ..fixed_box
                    },
                );
            let run = |tree: &PublicLayoutTreeOf<S>| {
                compute_layout(
                    tree,
                    0,
                    LayoutRootRequestOf::viewport(Size::new(
                        AvailableOf::MAX_CONTENT,
                        AvailableOf::MAX_CONTENT,
                    ))
                    .unwrap(),
                )
                .unwrap()
            };
            let control = run(&tree);
            for node in [0, 1, 2] {
                assert_box(&control, node, (0.0, 0.0), (10.0, 10.0));
            }
            replace_inset(
                &mut tree,
                2,
                Edges {
                    left: LengthAutoOf::px(px::<S>(offset)),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
            );
            let batch = run(&tree);
            assert_box(&batch, 0, (0.0, 0.0), (10.0, 10.0));
            assert_box(&batch, 1, (0.0, 0.0), (10.0, 10.0));
            assert_box(&batch, 2, (offset, 0.0), (10.0, 10.0));
            let overflow = output(&batch, 0)
                .scroll_geometry
                .unwrap()
                .scrollable_overflow();
            assert_eq!(
                overflow.origin(),
                Point::new(px::<S>(offset.min(0.0)), S::ZERO)
            );
            assert_eq!(overflow.size(), Size::new(px::<S>(14.0), px::<S>(10.0)));
        }
    }
}
#[test]
fn relative_flow_nested_visual_overflow_f32() {
    nested_visual_overflow::<f32>();
}
#[test]
fn relative_flow_nested_visual_overflow_f64() {
    nested_visual_overflow::<f64>();
}

fn named_offsets<S: LayoutScalar>(display: Display, direction: Direction, box_sizing: BoxSizing) {
    for decorated in [false, true] {
        let mut tree = three_boxes::<S>(display, direction, box_sizing, decorated);
        assert_unshifted_trio(&compute(&tree));
        replace_inset(
            &mut tree,
            1,
            Edges {
                top: LengthAutoOf::px(px::<S>(4.0)),
                left: LengthAutoOf::px(px::<S>(2.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
        );
        replace_inset(
            &mut tree,
            2,
            Edges {
                bottom: LengthAutoOf::px(px::<S>(8.0)),
                right: LengthAutoOf::px(px::<S>(6.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
        );
        replace_inset(
            &mut tree,
            3,
            Edges {
                top: LengthAutoOf::px(px::<S>(4.0)),
                left: LengthAutoOf::px(px::<S>(2.0)),
                bottom: LengthAutoOf::px(px::<S>(8.0)),
                right: LengthAutoOf::px(px::<S>(6.0)),
            },
        );
        let shifted = compute(&tree);
        assert_box(&shifted, 0, (0.0, 0.0), (50.0, 30.0));
        assert_box(&shifted, 1, (2.0, 4.0), (50.0, 10.0));
        assert_box(&shifted, 2, (-6.0, 2.0), (50.0, 10.0));
        assert_box(
            &shifted,
            3,
            (
                if direction == Direction::Ltr {
                    2.0
                } else {
                    -6.0
                },
                24.0,
            ),
            (50.0, 10.0),
        );
    }
}

macro_rules! named_case {
    ($name:ident, $scalar:ty, $display:expr, $direction:expr, $box_sizing:expr) => {
        #[test]
        fn $name() {
            named_offsets::<$scalar>($display, $direction, $box_sizing);
        }
    };
}
named_case!(
    relative_flow_flex_ltr_border_f32,
    f32,
    Display::Flex,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_flex_rtl_border_f32,
    f32,
    Display::Flex,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_flex_ltr_content_f32,
    f32,
    Display::Flex,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_flex_rtl_content_f32,
    f32,
    Display::Flex,
    Direction::Rtl,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_grid_ltr_border_f32,
    f32,
    Display::Grid,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_grid_rtl_border_f32,
    f32,
    Display::Grid,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_grid_ltr_content_f32,
    f32,
    Display::Grid,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_grid_rtl_content_f32,
    f32,
    Display::Grid,
    Direction::Rtl,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_block_ltr_border_f32,
    f32,
    Display::Block,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_block_rtl_border_f32,
    f32,
    Display::Block,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_block_ltr_content_f32,
    f32,
    Display::Block,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_block_rtl_content_f32,
    f32,
    Display::Block,
    Direction::Rtl,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_flex_ltr_border_f64,
    f64,
    Display::Flex,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_flex_rtl_border_f64,
    f64,
    Display::Flex,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_flex_ltr_content_f64,
    f64,
    Display::Flex,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_flex_rtl_content_f64,
    f64,
    Display::Flex,
    Direction::Rtl,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_grid_ltr_border_f64,
    f64,
    Display::Grid,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_grid_rtl_border_f64,
    f64,
    Display::Grid,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_grid_ltr_content_f64,
    f64,
    Display::Grid,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_grid_rtl_content_f64,
    f64,
    Display::Grid,
    Direction::Rtl,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_block_ltr_border_f64,
    f64,
    Display::Block,
    Direction::Ltr,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_block_rtl_border_f64,
    f64,
    Display::Block,
    Direction::Rtl,
    BoxSizing::BorderBox
);
named_case!(
    relative_flow_block_ltr_content_f64,
    f64,
    Display::Block,
    Direction::Ltr,
    BoxSizing::ContentBox
);
named_case!(
    relative_flow_block_rtl_content_f64,
    f64,
    Display::Block,
    Direction::Rtl,
    BoxSizing::ContentBox
);

fn noncanceling_shift<S: LayoutScalar>() {
    let mut tree = three_boxes::<S>(Display::Flex, Direction::Ltr, BoxSizing::BorderBox, false);
    assert_unshifted_trio(&compute(&tree));
    replace_inset(&mut tree, 1, top(4.0));
    let batch = compute(&tree);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 30.0));
    for (node, y) in [(1, 4.0), (2, 10.0), (3, 20.0)] {
        assert_box(&batch, node, (0.0, y), (50.0, 10.0));
    }
}
#[test]
fn relative_flow_noncanceling_shift_f32() {
    noncanceling_shift::<f32>();
}
#[test]
fn relative_flow_noncanceling_shift_f64() {
    noncanceling_shift::<f64>();
}

fn overflow_keeps_normal_height<S: LayoutScalar>(negative: bool) {
    let mut tree = three_boxes::<S>(Display::Flex, Direction::Ltr, BoxSizing::BorderBox, false);
    assert_unshifted_trio(&compute(&tree));
    if negative {
        replace_inset(&mut tree, 1, top(-4.0));
    } else {
        replace_inset(&mut tree, 3, top(4.0));
    }
    let batch = compute(&tree);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 30.0));
    let ys = if negative {
        [-4.0, 10.0, 20.0]
    } else {
        [0.0, 10.0, 24.0]
    };
    for (node, y) in [1, 2, 3].into_iter().zip(ys) {
        assert_box(&batch, node, (0.0, y), (50.0, 10.0));
    }
    // Union the actual visual borders with the normal parent0..30. Positive
    // overflow reaches34; negative overflow spans-4..30, also of extent34.
    let root = output(&batch, 0);
    let geometry = root.scroll_geometry.unwrap();
    let overflow = geometry.scrollable_overflow();
    assert_eq!(
        overflow.origin(),
        Point::new(S::ZERO, px::<S>(if negative { -4.0 } else { 0.0 }))
    );
    assert_eq!(overflow.size(), Size::new(px::<S>(50.0), px::<S>(34.0)));
}
#[test]
fn relative_flow_positive_overflow_keeps_normal_height_f32() {
    overflow_keeps_normal_height::<f32>(false);
}
#[test]
fn relative_flow_positive_overflow_keeps_normal_height_f64() {
    overflow_keeps_normal_height::<f64>(false);
}
#[test]
fn relative_flow_negative_overflow_keeps_normal_height_f32() {
    overflow_keeps_normal_height::<f32>(true);
}
#[test]
fn relative_flow_negative_overflow_keeps_normal_height_f64() {
    overflow_keeps_normal_height::<f64>(true);
}

fn collapsed_margin<S: LayoutScalar>(display: Display) {
    let mut tree = three_boxes::<S>(Display::Block, Direction::Ltr, BoxSizing::BorderBox, false);
    let mut first = crate::LayoutTree::node_input(&tree, 1).clone();
    first.margin.bottom = LengthAutoOf::px(px::<S>(6.0));
    tree.insert_input(1, LayoutInputOf::box_input(first));
    let mut middle = ten_box::<S>(display, Direction::Ltr, BoxSizing::BorderBox, false);
    middle.margin.top = LengthAutoOf::px(px::<S>(8.0));
    tree.insert_input(2, LayoutInputOf::box_input(middle));
    // The adjoining sibling margins collapse to max(6,8)=8.
    let control = compute(&tree);
    assert_box(&control, 0, (0.0, 0.0), (50.0, 38.0));
    for (node, y) in [(1, 0.0), (2, 18.0), (3, 28.0)] {
        assert_box(&control, node, (0.0, y), (50.0, 10.0));
    }
    replace_inset(&mut tree, 2, top(4.0));
    let batch = compute(&tree);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 38.0));
    for (node, y) in [(1, 0.0), (2, 22.0), (3, 28.0)] {
        assert_box(&batch, node, (0.0, y), (50.0, 10.0));
    }
}
macro_rules! collapse_case {
    ($name:ident, $scalar:ty, $display:expr) => {
        #[test]
        fn $name() {
            collapsed_margin::<$scalar>($display);
        }
    };
}
collapse_case!(relative_flow_collapsed_margin_flex_f32, f32, Display::Flex);
collapse_case!(relative_flow_collapsed_margin_flex_f64, f64, Display::Flex);
collapse_case!(relative_flow_collapsed_margin_grid_f32, f32, Display::Grid);
collapse_case!(relative_flow_collapsed_margin_grid_f64, f64, Display::Grid);
collapse_case!(
    relative_flow_collapsed_margin_block_f32,
    f32,
    Display::Block
);
collapse_case!(
    relative_flow_collapsed_margin_block_f64,
    f64,
    Display::Block
);

fn actual_float_clearance<S: LayoutScalar>(display: Display, clear: Clear) {
    for offset in [0.0, 4.0, -4.0] {
        let mut shifted = ten_box::<S>(display, Direction::Ltr, BoxSizing::BorderBox, false);
        shifted.size.width = PreferredSizeOf::px(px::<S>(50.0));
        shifted.clear = clear;
        shifted.inset = top(offset);
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1, 2, 3])
            .style(0, root_style(Direction::Ltr))
            .children(1, [])
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(px::<S>(50.0)),
                        PreferredSizeOf::px(px::<S>(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            )
            .children(2, [])
            .style(2, shifted)
            .children(3, [])
            .style(
                3,
                ten_box(Display::Block, Direction::Ltr, BoxSizing::BorderBox, false),
            );
        let batch = compute(&tree);
        // A real full-width float forces normal start20. Relative movement may
        // overlap its visual border but cannot undo the actual clearance20.
        assert_box(&batch, 0, (0.0, 0.0), (50.0, 40.0));
        assert_box(&batch, 1, (0.0, 0.0), (50.0, 20.0));
        assert_box(&batch, 2, (0.0, 20.0 + offset), (50.0, 10.0));
        assert_box(&batch, 3, (0.0, 30.0), (50.0, 10.0));
    }
}
macro_rules! float_case {
    ($name:ident, $scalar:ty, $display:expr, $clear:expr) => {
        #[test]
        fn $name() {
            actual_float_clearance::<$scalar>($display, $clear);
        }
    };
}
float_case!(
    relative_flow_float_placement_flex_f32,
    f32,
    Display::Flex,
    Clear::None
);
float_case!(
    relative_flow_float_placement_flex_f64,
    f64,
    Display::Flex,
    Clear::None
);
float_case!(
    relative_flow_float_placement_grid_f32,
    f32,
    Display::Grid,
    Clear::None
);
float_case!(
    relative_flow_float_placement_grid_f64,
    f64,
    Display::Grid,
    Clear::None
);
float_case!(
    relative_flow_actual_clearance_block_f32,
    f32,
    Display::Block,
    Clear::Left
);
float_case!(
    relative_flow_actual_clearance_block_f64,
    f64,
    Display::Block,
    Clear::Left
);

fn no_float_clear<S: LayoutScalar>() {
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .style(0, root_style(Direction::Ltr));
    let mut first = ten_box::<S>(Display::Block, Direction::Ltr, BoxSizing::BorderBox, false);
    first.clear = Clear::Both;
    tree = tree.children(1, []).style(1, first).children(2, []).style(
        2,
        ten_box(Display::Block, Direction::Ltr, BoxSizing::BorderBox, false),
    );
    let control = compute(&tree);
    assert_box(&control, 0, (0.0, 0.0), (50.0, 20.0));
    assert_box(&control, 1, (0.0, 0.0), (50.0, 10.0));
    assert_box(&control, 2, (0.0, 10.0), (50.0, 10.0));
    replace_inset(&mut tree, 1, top(4.0));
    let batch = compute(&tree);
    assert_box(&batch, 0, (0.0, 0.0), (50.0, 20.0));
    assert_box(&batch, 1, (0.0, 4.0), (50.0, 10.0));
    assert_box(&batch, 2, (0.0, 10.0), (50.0, 10.0));
}
#[test]
fn relative_flow_no_float_clear_block_f32() {
    no_float_clear::<f32>();
}
#[test]
fn relative_flow_no_float_clear_block_f64() {
    no_float_clear::<f64>();
}

fn reversed_block_axis<S: LayoutScalar>(display: Display) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        let mut tree = PublicLayoutTreeOf::new().children(0, [1, 2, 3]).style(
            0,
            NodeInputOf {
                display: Display::Block,
                direction,
                writing_mode: WritingMode::VerticalRl,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(px::<S>(50.0))),
                ..NodeInputOf::default()
            },
        );
        for node in [1, 2, 3] {
            tree = tree.children(node, []).style(
                node,
                NodeInputOf {
                    display,
                    direction,
                    writing_mode: WritingMode::VerticalRl,
                    position: Position::Relative,
                    size: Size::new(
                        PreferredSizeOf::px(px::<S>(10.0)),
                        PreferredSizeOf::px(px::<S>(50.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        }
        let run = |tree: &PublicLayoutTreeOf<S>| {
            compute_layout(
                tree,
                0,
                LayoutRootRequestOf::viewport(Size::new(
                    AvailableOf::MAX_CONTENT,
                    AvailableOf::definite(px::<S>(50.0)),
                ))
                .unwrap(),
            )
            .unwrap()
        };
        let control = run(&tree);
        assert_box(&control, 0, (0.0, 0.0), (30.0, 50.0));
        for (node, x) in [(1, 20.0), (2, 10.0), (3, 0.0)] {
            assert_box(&control, node, (x, 0.0), (10.0, 50.0));
        }
        // Block progression is right-to-left. A right4 inset moves inward-4;
        // left8 moves+8, and opposing insets retain the block-start right4.
        replace_inset(
            &mut tree,
            1,
            Edges {
                right: LengthAutoOf::px(px::<S>(4.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
        );
        replace_inset(
            &mut tree,
            2,
            Edges {
                left: LengthAutoOf::px(px::<S>(8.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
        );
        replace_inset(
            &mut tree,
            3,
            Edges {
                right: LengthAutoOf::px(px::<S>(4.0)),
                left: LengthAutoOf::px(px::<S>(8.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
        );
        let batch = run(&tree);
        assert_box(&batch, 0, (0.0, 0.0), (30.0, 50.0));
        for (node, x) in [(1, 16.0), (2, 18.0), (3, -4.0)] {
            assert_box(&batch, node, (x, 0.0), (10.0, 50.0));
        }
    }
}
macro_rules! reversed_case {
    ($name:ident, $scalar:ty, $display:expr) => {
        #[test]
        fn $name() {
            reversed_block_axis::<$scalar>($display);
        }
    };
}
reversed_case!(relative_flow_vertical_rl_flex_f32, f32, Display::Flex);
reversed_case!(relative_flow_vertical_rl_flex_f64, f64, Display::Flex);
reversed_case!(relative_flow_vertical_rl_grid_f32, f32, Display::Grid);
reversed_case!(relative_flow_vertical_rl_grid_f64, f64, Display::Grid);
reversed_case!(relative_flow_vertical_rl_block_f32, f32, Display::Block);
reversed_case!(relative_flow_vertical_rl_block_f64, f64, Display::Block);

#[path = "relative_flow/percentage_insets.rs"]
mod percentage_insets;

#[path = "relative_flow/relative_canonical.rs"]
mod relative_canonical;
