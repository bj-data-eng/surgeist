//! Used size of an absolutely positioned box against the same containing
//! block in Block, Flex and Grid parents, through `compute_layout`.
//!
//! The containing block is the 400px by 300px padding box of the parent. The
//! child has no insets and no content. Expected sizes follow CSS Sizing 3 §3.1
//! and §5.2, CSS Sizing 4 §4.1, §4.2 and §4.4, and CSS 2.1 §10.3.7, §10.4 and
//! §10.6.4.
use surgeist_layout::{
    AspectRatioOf, AvailableSpaceOf, AxisSizingRequestOf, BoxSizing, Direction, Display, Edges,
    FlowAxes, LayoutInputOf, LayoutInputViewOf, LayoutRootRequestOf, LayoutScalar, LayoutTree,
    LengthOf, LengthPercentageOf, MaxSizeOf, MinSizeOf, NearestScrollportOf, NodeInputOf,
    OrthogonalFallbackLimitOf, OrthogonalFallbackOf, Position, PreferredSizeOf, Size,
    SizingCalculationOf, SizingOperation, SizingRequestOf, TrackComponentOf, TrackSizingOf,
    Traverse, WritingMode, compute_layout,
};

const PARENTS: [Display; 3] = [Display::Block, Display::Flex, Display::Grid];

struct Tree<S: LayoutScalar> {
    inputs: Vec<LayoutInputOf<S>>,
}

impl<S: LayoutScalar> Traverse for Tree<S> {
    type Node = usize;
    type Scalar = S;
    type Children<'a>
        = std::ops::Range<usize>
    where
        Self: 'a;

    fn children(&self, node: usize) -> Self::Children<'_> {
        if node == 0 { 1..2 } else { 0..0 }
    }
    fn child_count(&self, node: usize) -> usize {
        usize::from(node == 0)
    }
    fn child(&self, node: usize, index: usize) -> usize {
        assert_eq!((node, index), (0, 0));
        1
    }
}

impl<S: LayoutScalar> LayoutTree for Tree<S> {
    type MeasureError = std::convert::Infallible;

    fn node_input(&self, node: usize) -> &NodeInputOf<S> {
        self.inputs[node].as_box().unwrap()
    }
    fn layout_input(&self, node: usize) -> LayoutInputViewOf<'_, S> {
        self.inputs[node].as_view()
    }
}

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn px<S: LayoutScalar>(value: f64) -> LengthPercentageOf<S> {
    LengthPercentageOf::px(n(value)).unwrap()
}

fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Indefinite,
                SizingOperation::MaxContent,
            )),
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

fn parent<S: LayoutScalar>(display: Display) -> NodeInputOf<S> {
    let track = |extent: f64| {
        TrackComponentOf::from(TrackSizingOf::calculation(SizingCalculationOf::value(px(
            extent,
        ))))
    };
    NodeInputOf {
        display,
        size: Size::new(
            PreferredSizeOf::value(px(400.0)),
            PreferredSizeOf::value(px(300.0)),
        ),
        grid_template_columns: vec![track(400.0)],
        grid_template_rows: vec![track(300.0)],
        ..NodeInputOf::default()
    }
}

fn positioned<S: LayoutScalar>(box_sizing: BoxSizing) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        position: Position::Absolute,
        box_sizing,
        ..NodeInputOf::default()
    }
}

fn used_size<S: LayoutScalar>(display: Display, child: NodeInputOf<S>) -> (S, S) {
    let tree = Tree {
        inputs: vec![
            LayoutInputOf::box_input(parent(display)),
            LayoutInputOf::box_input(child),
        ],
    };
    let output = compute_layout(&tree, 0, request())
        .unwrap()
        .node_layout(1)
        .unwrap()
        .single_box()
        .expect("the positioned child produces one box")
        .unrounded_output();
    (output.size.width, output.size.height)
}

fn assert_used_size<S: LayoutScalar>(case: &str, child: NodeInputOf<S>, width: f64, height: f64) {
    let expected = (n::<S>(width), n::<S>(height));
    let failures: Vec<String> = PARENTS
        .into_iter()
        .filter_map(|display| {
            let actual = used_size(display, child.clone());
            (actual != expected).then(|| format!("{display:?} parent gave {actual:?}"))
        })
        .collect();
    assert!(
        failures.is_empty(),
        "{case}: expected {expected:?}; {}",
        failures.join("; ")
    );
}

macro_rules! scalar_lane_tests {
    ($($name:ident => $case:ident,)*) => {
        $(
            mod $name {
                #[test]
                fn f32_lane() {
                    super::$case::<f32>();
                }

                #[test]
                fn f64_lane() {
                    super::$case::<f64>();
                }
            }
        )*
    };
}

fn ratio<S: LayoutScalar>(value: f64) -> Option<AspectRatioOf<S>> {
    Some(AspectRatioOf::new(n(value)).unwrap())
}

fn horizontal_padding<S: LayoutScalar>(value: f64) -> Edges<LengthOf<S>> {
    Edges {
        left: LengthOf::value(px(value)),
        right: LengthOf::value(px(value)),
        ..Edges::default()
    }
}

fn vertical_padding<S: LayoutScalar>(value: f64) -> Edges<LengthOf<S>> {
    Edges {
        top: LengthOf::value(px(value)),
        bottom: LengthOf::value(px(value)),
        ..Edges::default()
    }
}

// Sizing 3 §5.2: the minimum size constraint is the strongest constraint, so
// a maximum below the minimum does not lower the used width.
fn minimum_width_beats_smaller_maximum<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        size: Size::new(
            PreferredSizeOf::value(px(150.0)),
            PreferredSizeOf::value(px(10.0)),
        ),
        min_size: Size::new(MinSizeOf::value(px(200.0)), MinSizeOf::AUTO),
        max_size: Size::new(MaxSizeOf::value(px(100.0)), MaxSizeOf::NONE),
        ..positioned(BoxSizing::BorderBox)
    };
    assert_used_size("min-width 200 over max-width 100", child, 200.0, 10.0);
}

// Sizing 3 §3.1: the inner size is floored at zero, so a border-box maximum
// smaller than padding plus border leaves the border box at that sum.
fn border_box_maximum_floors_at_padding<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        size: Size::new(
            PreferredSizeOf::value(px(50.0)),
            PreferredSizeOf::value(px(10.0)),
        ),
        max_size: Size::new(MaxSizeOf::value(px(10.0)), MaxSizeOf::NONE),
        padding: horizontal_padding(20.0),
        ..positioned(BoxSizing::BorderBox)
    };
    assert_used_size("border-box max-width 10 with padding 40", child, 40.0, 10.0);
}

// Sizing 4 §4.4: a definite minimum height transfers to the width axis. The
// empty box has a shrink-to-fit width of zero, so the transferred minimum 25
// decides the width.
fn minimum_height_transfers_to_width<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        aspect_ratio: ratio(0.5),
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::value(px(50.0))),
        ..positioned(BoxSizing::BorderBox)
    };
    assert_used_size("ratio 0.5 with min-height 50", child, 25.0, 50.0);
}

// Sizing 4 §4.1: size calculations involving the ratio work with the
// box-sizing box. A content-box width 100 with horizontal padding 20 gives the
// content box 100 by 100 and the border box 140 by 100.
fn ratio_applies_to_box_sizing_box_from_width<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        size: Size::new(PreferredSizeOf::value(px(100.0)), PreferredSizeOf::AUTO),
        padding: horizontal_padding(20.0),
        aspect_ratio: ratio(1.0),
        ..positioned(BoxSizing::ContentBox)
    };
    assert_used_size(
        "content-box width 100, horizontal padding 20",
        child,
        140.0,
        100.0,
    );
}

// Sizing 4 §4.1, height axis: a content-box height 100 with vertical padding
// 20 gives the content box 100 by 100 and the border box 100 by 140.
fn ratio_applies_to_box_sizing_box_from_height<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::value(px(100.0))),
        padding: vertical_padding(20.0),
        aspect_ratio: ratio(1.0),
        ..positioned(BoxSizing::ContentBox)
    };
    assert_used_size(
        "content-box height 100, vertical padding 20",
        child,
        100.0,
        140.0,
    );
}

// Sizing 4 §4.4: a transferred minimum is capped by a definite preferred size
// in the destination axis, so min-height 100 with width 50 gives 50 by 100.
fn transferred_minimum_is_capped_by_definite_width<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        size: Size::new(PreferredSizeOf::value(px(50.0)), PreferredSizeOf::AUTO),
        min_size: Size::new(MinSizeOf::AUTO, MinSizeOf::value(px(100.0))),
        aspect_ratio: ratio(1.0),
        ..positioned(BoxSizing::BorderBox)
    };
    assert_used_size("width 50, min-height 100, ratio 1", child, 50.0, 100.0);
}

// Sizing 4 §4.2 computes the ratio-dependent axis like a replaced element
// with a ratio and no natural size, and CSS 2.1 §10.3.7 gives the other axis
// its shrink-to-fit width, which is zero for an empty box. The ratio transfers
// that width to a height of zero, and a maximum only caps a size.
fn maximums_only_cap_the_ratio_size<S: LayoutScalar>() {
    let child = NodeInputOf::<S> {
        aspect_ratio: ratio(1.0),
        max_size: Size::new(MaxSizeOf::value(px(100.0)), MaxSizeOf::value(px(80.0))),
        ..positioned(BoxSizing::BorderBox)
    };
    assert_used_size("ratio 1 with max 100 by 80", child, 0.0, 0.0);
}

scalar_lane_tests! {
    min_width_beats_smaller_max_width_in_every_parent => minimum_width_beats_smaller_maximum,
    border_box_max_width_keeps_padding_in_every_parent => border_box_maximum_floors_at_padding,
    min_height_transfers_to_width_through_ratio_in_every_parent => minimum_height_transfers_to_width,
    ratio_applies_to_box_sizing_box_from_width_in_every_parent => ratio_applies_to_box_sizing_box_from_width,
    ratio_applies_to_box_sizing_box_from_height_in_every_parent => ratio_applies_to_box_sizing_box_from_height,
    transferred_minimum_is_capped_by_definite_width_in_every_parent => transferred_minimum_is_capped_by_definite_width,
    max_sizes_only_cap_the_ratio_size_in_every_parent => maximums_only_cap_the_ratio_size,
}
