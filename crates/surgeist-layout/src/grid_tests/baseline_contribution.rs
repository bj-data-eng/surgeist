// Intended private child of grid_tests.rs. Real normalized inputs mirror the two
// retained grid_align_items_baseline_child_multiline_* fixture families.
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;

#[derive(Clone, Copy, Debug)]
enum OuterHeight {
    Fixed25,
    Auto40,
}

#[derive(Clone, Copy, Debug)]
struct Case {
    direction: Direction,
    box_sizing: BoxSizing,
    baseline_demand: bool,
    first_row_override: bool,
    overflow: Overflow,
    height: OuterHeight,
}

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn fixture<S: LayoutScalar>(case: Case) -> PublicLayoutTreeOf<S> {
    let grid = |width: f64, height: PreferredSizeOf<S>| NodeInputOf {
        display: Display::Grid,
        direction: case.direction,
        box_sizing: case.box_sizing,
        size: Size::new(PreferredSizeOf::px(n::<S>(width)), height),
        ..NodeInputOf::default()
    };
    let mut outer = grid(100.0, PreferredSizeOf::px(n::<S>(100.0)));
    if case.baseline_demand {
        outer.align_items = Some(AlignItems::Baseline);
    }
    let height = match case.height {
        OuterHeight::Fixed25 => PreferredSizeOf::px(n::<S>(25.0)),
        OuterHeight::Auto40 => PreferredSizeOf::AUTO,
    };
    let mut inner = grid(50.0, height);
    inner.grid_template_columns = vec![TrackComponentOf::AUTO, TrackComponentOf::AUTO];
    inner.overflow = ComputedOverflow::try_new(case.overflow, case.overflow).unwrap();
    let mut first_short = grid(25.0, PreferredSizeOf::px(n::<S>(10.0)));
    if case.first_row_override {
        first_short.align_self = Some(AlignItems::Baseline);
    }
    let mut last_short = grid(25.0, PreferredSizeOf::px(n::<S>(10.0)));
    last_short.align_self = Some(AlignItems::Baseline);
    // No measure provider or invented baseline: every item is an actual Grid.
    // Empty descendants synthesize their alignment baseline from their border.
    PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [3, 4, 5, 6])
        .children(3, [])
        .children(4, [])
        .children(5, [])
        .children(6, [])
        .style(0, outer)
        .style(1, grid(50.0, PreferredSizeOf::px(n::<S>(60.0))))
        .style(2, inner)
        .style(3, grid(25.0, PreferredSizeOf::px(n::<S>(20.0))))
        .style(4, first_short)
        .style(5, grid(25.0, PreferredSizeOf::px(n::<S>(20.0))))
        .style(6, last_short)
}

fn node<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    id: u32,
    rounded: bool,
) -> NodeOutputOf<S> {
    let entries = if rounded {
        batch.final_entries()
    } else {
        batch.unrounded_entries()
    };
    entries
        .iter()
        .find(|entry| entry.node() == id)
        .unwrap()
        .output()
}

fn assert_box<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    case: Case,
    rounded: bool,
    id: u32,
    rect: (f64, f64, f64, f64),
    overflow_height: f64,
) {
    let output = node(batch, id, rounded);
    let (x, y, width, height) = rect;
    assert_eq!(
        output.location,
        Point::new(n::<S>(x), n::<S>(y)),
        "{case:?} rounded={rounded} node={id}: border location"
    );
    assert_eq!(
        output.size,
        Size::new(n::<S>(width), n::<S>(height)),
        "{case:?} rounded={rounded} node={id}: border size"
    );
    assert_eq!(
        (output.border, output.padding, output.margin),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO),
        "{case:?} node={id}: genuine zero edges"
    );
    let geometry = output.scroll_geometry.unwrap();
    assert_eq!(geometry.border_box().origin(), Point::ZERO);
    assert_eq!(geometry.border_box().size(), output.size);
    assert_eq!(geometry.padding_box(), geometry.border_box());
    assert_eq!(geometry.content_box(), geometry.border_box());
    let overflow = geometry.scrollable_overflow();
    assert_eq!(
        overflow.origin(),
        Point::ZERO,
        "{case:?} rounded={rounded} node={id}"
    );
    assert_eq!(
        overflow.size(),
        Size::new(n::<S>(width), n::<S>(overflow_height)),
        "{case:?} rounded={rounded} node={id}: actual overflow remains distinct from border"
    );
}

fn check<S: LayoutScalar>(case: Case) {
    let tree = fixture::<S>(case);
    // Both source families, direction, box mode and baseline demand are explicit
    // normalized input facts; no mutation of an issued result is involved.
    assert_eq!(tree.child_count(0), 2);
    assert_eq!(tree.child_count(2), 4);
    assert_eq!(
        tree.node_input(0).align_items,
        case.baseline_demand.then_some(AlignItems::Baseline)
    );
    assert_eq!(
        tree.node_input(4).align_self,
        case.first_row_override.then_some(AlignItems::Baseline)
    );
    assert_eq!(tree.node_input(6).align_self, Some(AlignItems::Baseline));
    let batch = compute_layout(&tree, 0, {
        let axes = Size::new(
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::MaxContent),
        );
        let definite = axes.map(|axis| axis.space().definite_size());
        crate::LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                axes,
                OrthogonalFallbackOf::new(
                    definite,
                    crate::NearestScrollportOf::Absent,
                    Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
            crate::geometry::FlowAxes::new(crate::WritingMode::HorizontalTb, crate::Direction::Ltr),
            definite,
        )
    })
    .unwrap();
    assert_eq!(batch.unrounded_entries().len(), 7);
    assert_eq!(batch.final_entries().len(), 7);
    assert!(batch.placed_text().is_empty());

    // Independent arithmetic: fixed preferred outer sizes60 and25 contribute85.
    // Normal auto-row stretching shares the remaining15 equally:67.5/32.5.
    // The true auto-height neighbor contributes40, leaving rows60/40 instead.
    let (first_row, second_row, inner_height, inner_y) = match case.height {
        OuterHeight::Fixed25 => (67.5, 32.5, 25.0, 67.5),
        OuterHeight::Auto40 => (60.0, 40.0, 40.0, 60.0),
    };
    let root_overflow = match (case.height, case.overflow) {
        (OuterHeight::Fixed25, Overflow::Visible) => 107.5,
        _ => 100.0,
    };
    let child_x = if case.direction == Direction::Rtl {
        50.0
    } else {
        0.0
    };
    let (long_x, short_x) = if case.direction == Direction::Rtl {
        (25.0, 0.0)
    } else {
        (0.0, 25.0)
    };
    for rounded in [false, true] {
        let y = if rounded && matches!(case.height, OuterHeight::Fixed25) {
            68.0
        } else {
            inner_y
        };
        let root_overflow = if rounded && root_overflow == 107.5 {
            108.0
        } else {
            root_overflow
        };
        // Observe contribution-driven placement before its derived overflow.
        assert_box(
            &batch,
            case,
            rounded,
            2,
            (child_x, y, 50.0, inner_height),
            40.0,
        );
        assert_box(
            &batch,
            case,
            rounded,
            0,
            (0.0, 0.0, 100.0, 100.0),
            root_overflow,
        );
        assert_box(&batch, case, rounded, 1, (child_x, 0.0, 50.0, 60.0), 60.0);
        assert_box(&batch, case, rounded, 3, (long_x, 0.0, 25.0, 20.0), 20.0);
        assert_box(&batch, case, rounded, 4, (short_x, 0.0, 25.0, 10.0), 10.0);
        assert_box(&batch, case, rounded, 5, (long_x, 20.0, 25.0, 20.0), 20.0);
        assert_box(&batch, case, rounded, 6, (short_x, 20.0, 25.0, 10.0), 10.0);
        assert_eq!(node(&batch, 1, rounded).source_index.get(), 0);
        assert_eq!(node(&batch, 2, rounded).source_index.get(), 1);
        for (index, id) in [3, 4, 5, 6].into_iter().enumerate() {
            assert_eq!(node(&batch, id, rounded).source_index.get(), index);
        }
    }

    // With one participant per row, no gaps and start-aligned baseline groups,
    // the second child's origin is the row boundary; the root supplies the end.
    // These observations come from the public accepted batch, not cache traffic.
    let boundary = node(&batch, 2, false).location.y;
    assert_eq!(
        (boundary, node(&batch, 0, false).size.height - boundary),
        (n::<S>(first_row), n::<S>(second_row)),
        "{case:?}: actual row boundaries",
    );

    // Separate, invocation-owned qualification of operational ComputeOutput.
    // The public batch does not promise to retain final performed cache queries.
    // Run the REAL Grid algorithm with the same authored root dimensions and
    // containing role, without changing or feeding observations into that batch.
    let axes = FlowAxes::new(WritingMode::HorizontalTb, case.direction);
    let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let root = session
        .compute_child(
            0,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::NONE,
                ContainingLayoutContext::new(axes, ParentFormattingContext::NoParent),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .unwrap();
    assert_eq!(root.size, Size::new(n::<S>(100.0), n::<S>(100.0)));
    assert_eq!(
        root.baselines(),
        BaselinesOf {
            first: Point::new(None, Some(n::<S>(60.0))),
            last: Point::new(None, Some(n::<S>(inner_y + 30.0))),
        },
        "{case:?}: separate real root computation exports normal baselines",
    );
    let mut session = crate::engine::ComputeSession::new(&tree, Vec::new());
    let inner = session
        .compute_child(
            2,
            ComputeInputOf::for_child(
                RunMode::PerformLayout,
                SizingMode::InherentSize,
                RequestedAxis::Both,
                Size::NONE,
                Size::new(Some(n::<S>(100.0)), Some(n::<S>(100.0))),
                ContainingLayoutContext::new(axes, ParentFormattingContext::Grid),
                SizingRequestOf::new(
                    Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Indefinite,
                            SizingOperation::MaxContent,
                        ),
                    ),
                    OrthogonalFallbackOf::new(
                        Size::NONE,
                        crate::NearestScrollportOf::Absent,
                        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
                    ),
                ),
            ),
        )
        .unwrap();
    // The100×100 parent is authored, not an expected row size supplied as input.
    // This is the actual baseline-demanding intrinsic observation, separately
    // qualifying its output; it is not a receipt from the accepted public batch.
    let first = if case.first_row_override { 10.0 } else { 20.0 };
    assert_eq!(
        inner.baselines(),
        BaselinesOf {
            first: Point::new(None, Some(n::<S>(first))),
            last: Point::new(None, Some(n::<S>(30.0))),
        },
        "{case:?}: real nested first/last baseline export",
    );
    assert_eq!(inner.size, Size::new(n::<S>(50.0), n::<S>(inner_height)));
    assert_eq!(
        inner.content_size,
        Size::new(n::<S>(50.0), n::<S>(40.0)),
        "{case:?}: real observed content remains distinct from preferred border height",
    );
}

fn controls<S: LayoutScalar>(baseline_demand: bool, overflow: Overflow, height: OuterHeight) {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            for first_row_override in [false, true] {
                check::<S>(Case {
                    direction,
                    box_sizing,
                    baseline_demand,
                    first_row_override,
                    overflow,
                    height,
                });
            }
        }
    }
}

macro_rules! fixed_visible_baseline {
    ($name:ident, $scalar:ty, $direction:expr, $box_sizing:expr, $first_override:expr) => {
        #[test]
        fn $name() {
            check::<$scalar>(Case {
                direction: $direction,
                box_sizing: $box_sizing,
                baseline_demand: true,
                first_row_override: $first_override,
                overflow: Overflow::Visible,
                height: OuterHeight::Fixed25,
            });
        }
    };
}

fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_border_ltr_f32,
    f32,
    Direction::Ltr,
    BoxSizing::BorderBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_border_rtl_f32,
    f32,
    Direction::Rtl,
    BoxSizing::BorderBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_content_ltr_f32,
    f32,
    Direction::Ltr,
    BoxSizing::ContentBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_content_rtl_f32,
    f32,
    Direction::Rtl,
    BoxSizing::ContentBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_border_ltr_f32,
    f32,
    Direction::Ltr,
    BoxSizing::BorderBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_border_rtl_f32,
    f32,
    Direction::Rtl,
    BoxSizing::BorderBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_content_ltr_f32,
    f32,
    Direction::Ltr,
    BoxSizing::ContentBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_content_rtl_f32,
    f32,
    Direction::Rtl,
    BoxSizing::ContentBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_border_ltr_f64,
    f64,
    Direction::Ltr,
    BoxSizing::BorderBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_border_rtl_f64,
    f64,
    Direction::Rtl,
    BoxSizing::BorderBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_content_ltr_f64,
    f64,
    Direction::Ltr,
    BoxSizing::ContentBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_no_override_content_rtl_f64,
    f64,
    Direction::Rtl,
    BoxSizing::ContentBox,
    false
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_border_ltr_f64,
    f64,
    Direction::Ltr,
    BoxSizing::BorderBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_border_rtl_f64,
    f64,
    Direction::Rtl,
    BoxSizing::BorderBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_content_ltr_f64,
    f64,
    Direction::Ltr,
    BoxSizing::ContentBox,
    true
);
fixed_visible_baseline!(
    fixed25_baseline_visible_override_content_rtl_f64,
    f64,
    Direction::Rtl,
    BoxSizing::ContentBox,
    true
);

#[test]
fn fixed25_without_baseline_demand_keeps_rows_f32() {
    controls::<f32>(false, Overflow::Visible, OuterHeight::Fixed25);
}
#[test]
fn fixed25_without_baseline_demand_keeps_rows_f64() {
    controls::<f64>(false, Overflow::Visible, OuterHeight::Fixed25);
}
#[test]
fn clipped_fixed25_keeps_rows_and_own_content_f32() {
    for baseline in [false, true] {
        controls::<f32>(baseline, Overflow::Clip, OuterHeight::Fixed25);
    }
}
#[test]
fn clipped_fixed25_keeps_rows_and_own_content_f64() {
    for baseline in [false, true] {
        controls::<f64>(baseline, Overflow::Clip, OuterHeight::Fixed25);
    }
}
#[test]
fn actual_auto_height40_contributes40_f32() {
    for baseline in [false, true] {
        controls::<f32>(baseline, Overflow::Visible, OuterHeight::Auto40);
    }
}
#[test]
fn actual_auto_height40_contributes40_f64() {
    for baseline in [false, true] {
        controls::<f64>(baseline, Overflow::Visible, OuterHeight::Auto40);
    }
}
