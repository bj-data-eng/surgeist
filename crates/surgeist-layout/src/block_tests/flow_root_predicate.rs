//! Flow roots keep their margins, avoid floats and contain floats whether the
//! host declares them, measures them as leaves, or lays them out in a writing
//! mode that differs from their parent's.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{Phase, single_box_output};
use crate::*;

fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::px(S::from_f64(value))
}

fn block<S: LayoutScalar>(width: Option<f64>, height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            width.map_or(PreferredSizeOf::AUTO, px::<S>),
            height.map_or(PreferredSizeOf::AUTO, px::<S>),
        ),
        min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
        ..Default::default()
    }
}

fn margins<S: LayoutScalar>(top: f64, bottom: f64) -> Edges<LengthAutoOf<S>> {
    Edges {
        top: LengthAutoOf::px(S::from_f64(top)),
        bottom: LengthAutoOf::px(S::from_f64(bottom)),
        ..Edges::all(LengthAutoOf::ZERO)
    }
}

fn root_width<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    width: f64,
) -> CompletedLayoutBatchOf<u32, S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(width)).unwrap();
    let request = crate::test_support::requests::NativeRootFrameOf::new(
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(width), None),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .request(Size::new(
        AxisSizingRequestOf::new(AvailableSpaceOf::Definite(width), SizingOperation::Normal),
        AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
    ));
    compute_layout(tree, 0, request).unwrap()
}

fn location<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> (S, S) {
    let output = single_box_output(batch, node, Phase::Unrounded);
    (output.location.x, output.location.y)
}

#[derive(Clone, Copy, Debug)]
enum EmptyRoot {
    Independent,
    Table,
}

/// An empty block of the given kind sits between two siblings. `leaf` measures
/// it as a 0x0 leaf instead of laying out a childless box.
fn empty_root_positions<S: LayoutScalar>(kind: EmptyRoot, leaf: bool) -> (S, S) {
    let mut middle = NodeInputOf {
        margin: margins(10.0, 30.0),
        ..block::<S>(None, None)
    };
    match kind {
        EmptyRoot::Independent => middle.block_flow_context = BlockFlowContext::Independent,
        EmptyRoot::Table => middle.item_is_table = true,
    }
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(1, [])
        .children(2, [])
        .children(3, [])
        .style(0, block::<S>(Some(400.0), None))
        .style(
            1,
            NodeInputOf {
                margin: margins(0.0, 20.0),
                ..block::<S>(None, Some(10.0))
            },
        )
        .style(2, middle)
        .style(
            3,
            NodeInputOf {
                margin: margins(5.0, 0.0),
                ..block::<S>(None, Some(10.0))
            },
        );
    if leaf {
        tree = tree.measure(2, Size::new(S::ZERO, S::ZERO));
    }
    let batch = root_width(&tree, 400.0);
    (location(&batch, 2).1, location(&batch, 3).1)
}

fn assert_empty_root<S: LayoutScalar>(kind: EmptyRoot, leaf: bool) {
    // CSS 2.1 section 8.3.1: the top and bottom margins of a box adjoin only if
    // it does not establish a new block formatting context, and a table wrapper
    // box establishes one (section 17.4). The first box's 20px bottom margin
    // collapses with the root's 10px top margin: y = 10 + 20 = 30. The root's
    // 30px bottom margin collapses only with the next sibling's 5px top margin,
    // giving 30 + 30 = 60. If the root's own margins collapsed through, the
    // third box would sit at 10 + max(20, 10, 30, 5) = 40.
    assert_eq!(
        empty_root_positions::<S>(kind, leaf),
        (S::from_f64(30.0), S::from_f64(60.0)),
        "{kind:?} leaf={leaf}"
    );
}

/// Orthogonal block 150x30 after a 100x50 left float in a 400px parent.
fn orthogonal_after_float_x<S: LayoutScalar>(context: BlockFlowContext) -> (S, S) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, block::<S>(Some(400.0), None))
        .style(
            1,
            NodeInputOf {
                float: Float::Left,
                ..block::<S>(Some(100.0), Some(50.0))
            },
        )
        .style(
            2,
            NodeInputOf {
                block_flow_context: context,
                writing_mode: WritingMode::VerticalRl,
                ..block::<S>(Some(150.0), Some(30.0))
            },
        );
    let batch = root_width(&tree, 400.0);
    location(&batch, 2)
}

fn assert_orthogonal_avoids_float<S: LayoutScalar>(context: BlockFlowContext) {
    // CSS 2.1 section 9.5: a block formatting context root's border box does not
    // overlap the margin box of a float in the same context. Writing Modes 4
    // section 3.2: a block container whose writing mode differs from its
    // parent's establishes an independent block formatting context. The
    // 150px box fits in the 300px beside the 100px float, so it starts at x = 100.
    assert_eq!(
        orthogonal_after_float_x::<S>(context),
        (S::from_f64(100.0), S::ZERO),
        "{context:?}"
    );
}

/// Orthogonal block with automatic width whose only child is a 40x60 float.
fn orthogonal_floated_content_width<S: LayoutScalar>(context: BlockFlowContext) -> S {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(0, block::<S>(Some(400.0), None))
        .style(
            1,
            NodeInputOf {
                block_flow_context: context,
                writing_mode: WritingMode::VerticalRl,
                ..block::<S>(None, Some(100.0))
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                float: Float::Left,
                ..block::<S>(Some(40.0), Some(60.0))
            },
        );
    let batch = root_width(&tree, 400.0);
    single_box_output(&batch, 1, Phase::Unrounded).size.width
}

fn assert_orthogonal_contains_float<S: LayoutScalar>(context: BlockFlowContext) {
    // CSS 2.1 section 10.6.7: a block formatting context root with automatic
    // block size extends to the margin edge of its floating descendants. The
    // vertical-rl block axis is horizontal, so the 40px-wide float sets a block
    // size of 40 although the box has no in-flow content.
    assert_eq!(
        orthogonal_floated_content_width::<S>(context),
        S::from_f64(40.0),
        "{context:?}"
    );
}

/// Orthogonal block whose only child has a 20px margin on the physical right,
/// which is the block-start side in vertical-rl. Returns the orthogonal block's
/// width and its child's offset in it.
fn orthogonal_child_margin<S: LayoutScalar>(context: BlockFlowContext) -> (S, S) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [2])
        .children(2, [])
        .style(0, block::<S>(Some(400.0), None))
        .style(
            1,
            NodeInputOf {
                block_flow_context: context,
                writing_mode: WritingMode::VerticalRl,
                ..block::<S>(None, Some(100.0))
            },
        )
        .style(
            2,
            NodeInputOf {
                writing_mode: WritingMode::VerticalRl,
                margin: Edges {
                    right: LengthAutoOf::px(S::from_f64(20.0)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..block::<S>(Some(30.0), Some(40.0))
            },
        );
    let batch = root_width(&tree, 400.0);
    (
        single_box_output(&batch, 1, Phase::Unrounded).size.width,
        location(&batch, 2).0,
    )
}

fn assert_orthogonal_blocks_child_margin_collapse<S: LayoutScalar>(context: BlockFlowContext) {
    // CSS 2.1 section 8.3.1: a box that establishes a block formatting context
    // does not collapse its margins with its in-flow children's, and Writing
    // Modes 4 section 3.2 makes the orthogonal block such a box. The child's
    // 20px block-start margin therefore stays inside it: the automatic block
    // size is 20 + 30 = 50 and the child's border box spans x = 0..30. Were the
    // margin to collapse into the parent, the parent would be 30 wide.
    assert_eq!(
        orthogonal_child_margin::<S>(context),
        (S::from_f64(50.0), S::ZERO),
        "{context:?}"
    );
}

macro_rules! lanes {
    ($name:ident, $scalar:ident => $body:expr) => {
        mod $name {
            use super::*;

            #[test]
            fn f32_lane() {
                type $scalar = f32;
                $body
            }

            #[test]
            fn f64_lane() {
                type $scalar = f64;
                $body
            }
        }
    };
}

lanes!(empty_non_leaf_independent_root_keeps_own_margins, S =>
    assert_empty_root::<S>(EmptyRoot::Independent, false));
lanes!(empty_non_leaf_table_keeps_own_margins, S =>
    assert_empty_root::<S>(EmptyRoot::Table, false));
lanes!(empty_leaf_independent_root_keeps_own_margins, S =>
    assert_empty_root::<S>(EmptyRoot::Independent, true));
lanes!(empty_leaf_table_keeps_own_margins, S =>
    assert_empty_root::<S>(EmptyRoot::Table, true));

lanes!(independent_orthogonal_block_avoids_preceding_float, S =>
    assert_orthogonal_avoids_float::<S>(BlockFlowContext::Independent));
lanes!(normal_orthogonal_block_avoids_preceding_float, S =>
    assert_orthogonal_avoids_float::<S>(BlockFlowContext::Normal));

lanes!(independent_orthogonal_block_contains_its_float, S =>
    assert_orthogonal_contains_float::<S>(BlockFlowContext::Independent));
lanes!(normal_orthogonal_block_contains_its_float, S =>
    assert_orthogonal_contains_float::<S>(BlockFlowContext::Normal));

lanes!(independent_orthogonal_block_blocks_child_margin_collapse, S =>
    assert_orthogonal_blocks_child_margin_collapse::<S>(BlockFlowContext::Independent));
lanes!(normal_orthogonal_block_blocks_child_margin_collapse, S =>
    assert_orthogonal_blocks_child_margin_collapse::<S>(BlockFlowContext::Normal));
