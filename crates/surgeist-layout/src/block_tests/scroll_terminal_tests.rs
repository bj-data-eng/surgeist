//! End padding follows the actual in-flow (or floated) endpoint, independently
//! of visible descendant/absolute overflow. CSS Overflow 3 §3.3 includes those
//! rectangles separately; they do not acquire another copy of terminal padding.
use super::fixtures::{computed_overflow, fri06_atomic_participation};
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;

fn scalar<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}

fn run<S: LayoutScalar>(tree: &PublicLayoutTreeOf<S>) -> CompletedLayoutBatchOf<u32, S> {
    compute_layout(
        tree,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
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
    at: (f64, f64),
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
            Point::new(scalar::<S>(at.0), scalar::<S>(at.1)),
            "node {node}"
        );
        assert_eq!(
            actual.size,
            Size::new(scalar::<S>(size.0), scalar::<S>(size.1)),
            "node {node}"
        );
    }
}

fn assert_overflow<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    origin: (f64, f64),
    size: (f64, f64),
) {
    for entries in [batch.unrounded_entries(), batch.final_entries()] {
        let root = entries
            .iter()
            .find(|entry| entry.node() == 0)
            .unwrap()
            .output();
        let overflow = root.scroll_geometry.unwrap().scrollable_overflow();
        assert_eq!(
            overflow.origin(),
            Point::new(scalar::<S>(origin.0), scalar::<S>(origin.1))
        );
        assert_eq!(
            overflow.size(),
            Size::new(scalar::<S>(size.0), scalar::<S>(size.1))
        );
    }
}

fn sized<S: LayoutScalar>(
    mode: WritingMode,
    direction: Direction,
    width: f64,
    height: f64,
) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        writing_mode: mode,
        direction,
        size: Size::new(
            PreferredSizeOf::px(scalar::<S>(width)),
            PreferredSizeOf::px(scalar::<S>(height)),
        ),
        ..NodeInputOf::default()
    }
}

fn container<S: LayoutScalar>(
    mode: WritingMode,
    direction: Direction,
    padding: Edges<LengthOf<S>>,
) -> NodeInputOf<S> {
    NodeInputOf {
        // The authored CONTENT is 10×10. A single 3px padding side therefore
        // gives a 13px border/padding box on that physical axis, in either flow.
        box_sizing: BoxSizing::ContentBox,
        padding,
        overflow: computed_overflow(Overflow::Hidden, Overflow::Hidden),
        ..sized(mode, direction, 10.0, 10.0)
    }
}

#[derive(Clone, Copy, Debug)]
enum End {
    Top,
    Right,
    Bottom,
    Left,
}

fn directed_endpoint<S: LayoutScalar>(mode: WritingMode, direction: Direction, end: End) {
    // Literal physical-side expectations, not FlowAxes projections. The real
    // content border starts at the padding-box content origin. Moving 4 toward
    // the selected end leaves 3 more pixels beyond its actual terminal border.
    let (padding, normal, inset, shifted, root_size, overflow_origin, overflow_size) = match end {
        End::Right => (
            Edges {
                right: LengthOf::px(scalar::<S>(3.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            (0.0, 0.0),
            Edges {
                left: LengthAutoOf::px(scalar::<S>(4.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            (4.0, 0.0),
            (13.0, 10.0),
            (0.0, 0.0),
            (17.0, 10.0),
        ),
        End::Left => (
            Edges {
                left: LengthOf::px(scalar::<S>(3.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            (3.0, 0.0),
            Edges {
                left: LengthAutoOf::px(scalar::<S>(-4.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            (-1.0, 0.0),
            (13.0, 10.0),
            (-4.0, 0.0),
            (17.0, 10.0),
        ),
        End::Bottom => (
            Edges {
                bottom: LengthOf::px(scalar::<S>(3.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            (0.0, 0.0),
            Edges {
                top: LengthAutoOf::px(scalar::<S>(4.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            (0.0, 4.0),
            (10.0, 13.0),
            (0.0, 0.0),
            (10.0, 17.0),
        ),
        End::Top => (
            Edges {
                top: LengthOf::px(scalar::<S>(3.0)),
                ..Edges::all(LengthOf::ZERO)
            },
            (0.0, 3.0),
            Edges {
                top: LengthAutoOf::px(scalar::<S>(-4.0)),
                ..Edges::all(LengthAutoOf::AUTO)
            },
            (0.0, -1.0),
            (10.0, 13.0),
            (0.0, -4.0),
            (10.0, 17.0),
        ),
    };
    let child = NodeInputOf {
        position: Position::Relative,
        ..sized(mode, direction, 10.0, 10.0)
    };
    let mut tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, container(mode, direction, padding))
        .style(1, child.clone());
    let control = run(&tree);
    assert_box(&control, 0, (0.0, 0.0), root_size);
    assert_box(&control, 1, normal, (10.0, 10.0));
    assert_overflow(&control, (0.0, 0.0), root_size);
    tree.insert_input(1, LayoutInputOf::box_input(NodeInputOf { inset, ..child }));
    let actual = run(&tree);
    assert_box(&actual, 0, (0.0, 0.0), root_size);
    assert_box(&actual, 1, shifted, (10.0, 10.0));
    assert_overflow(&actual, overflow_origin, overflow_size);
    // A later computation cannot alter the previously published control.
    assert_box(&control, 1, normal, (10.0, 10.0));
    assert_overflow(&control, (0.0, 0.0), root_size);
}

fn all_directed_endpoints<S: LayoutScalar>() {
    // All numeric writing modes, both inline directions, and all four physical
    // end sides. Sideways-lr has a bottom-to-top LTR inline progression.
    for (mode, direction, end) in [
        (WritingMode::HorizontalTb, Direction::Ltr, End::Right),
        (WritingMode::HorizontalTb, Direction::Rtl, End::Left),
        (WritingMode::VerticalLr, Direction::Ltr, End::Bottom),
        (WritingMode::VerticalLr, Direction::Rtl, End::Top),
        (WritingMode::VerticalRl, Direction::Ltr, End::Bottom),
        (WritingMode::VerticalRl, Direction::Rtl, End::Top),
        (WritingMode::SidewaysRl, Direction::Ltr, End::Bottom),
        (WritingMode::SidewaysRl, Direction::Rtl, End::Top),
        (WritingMode::SidewaysLr, Direction::Ltr, End::Top),
        (WritingMode::SidewaysLr, Direction::Rtl, End::Bottom),
        (WritingMode::HorizontalTb, Direction::Ltr, End::Bottom),
        (WritingMode::HorizontalTb, Direction::Rtl, End::Bottom),
        (WritingMode::VerticalLr, Direction::Ltr, End::Right),
        (WritingMode::VerticalLr, Direction::Rtl, End::Right),
        (WritingMode::VerticalRl, Direction::Ltr, End::Left),
        (WritingMode::VerticalRl, Direction::Rtl, End::Left),
    ] {
        directed_endpoint::<S>(mode, direction, end);
    }
}

fn signed_end_margins<S: LayoutScalar>() {
    for margin in [-2.0, 2.0] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let padding = Edges {
                bottom: LengthOf::px(scalar::<S>(3.0)),
                ..Edges::all(LengthOf::ZERO)
            };
            let child = NodeInputOf {
                position: Position::Relative,
                inset: Edges {
                    top: LengthAutoOf::px(scalar::<S>(4.0)),
                    ..Edges::all(LengthAutoOf::AUTO)
                },
                margin: Edges {
                    bottom: LengthAutoOf::px(scalar::<S>(margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                ..sized(WritingMode::HorizontalTb, direction, 10.0, 10.0)
            };
            let tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, container(WritingMode::HorizontalTb, direction, padding))
                .style(1, child);
            let actual = run(&tree);
            assert_box(&actual, 0, (0.0, 0.0), (10.0, 13.0));
            assert_box(&actual, 1, (0.0, 4.0), (10.0, 10.0));
            assert_eq!(output(&actual, 1).margin.bottom, scalar::<S>(margin));
            // Border end14 is independent of the signed normal-flow margin.
            // The adopted optional envelope uses only positive outward margins:
            // -2 =>14+0+3=17; +2 =>14+2+3=19. No border is retracted.
            assert_overflow(
                &actual,
                (0.0, 0.0),
                (10.0, if margin < 0.0 { 17.0 } else { 19.0 }),
            );
        }
    }
}

fn floated_endpoints<S: LayoutScalar>() {
    for direction in [Direction::Ltr, Direction::Rtl] {
        for margin in [-2.0, 0.0, 2.0] {
            let padding = match direction {
                Direction::Ltr => Edges {
                    right: LengthOf::px(scalar::<S>(3.0)),
                    bottom: LengthOf::px(scalar::<S>(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
                Direction::Rtl => Edges {
                    left: LengthOf::px(scalar::<S>(3.0)),
                    bottom: LengthOf::px(scalar::<S>(3.0)),
                    ..Edges::all(LengthOf::ZERO)
                },
            };
            let margins = match direction {
                Direction::Ltr => Edges {
                    right: LengthAutoOf::px(scalar::<S>(margin)),
                    bottom: LengthAutoOf::px(scalar::<S>(margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
                Direction::Rtl => Edges {
                    left: LengthAutoOf::px(scalar::<S>(margin)),
                    bottom: LengthAutoOf::px(scalar::<S>(margin)),
                    ..Edges::all(LengthAutoOf::ZERO)
                },
            };
            let child = NodeInputOf {
                float: if direction == Direction::Ltr {
                    Float::Left
                } else {
                    Float::Right
                },
                margin: margins,
                ..sized(WritingMode::HorizontalTb, direction, 14.0, 14.0)
            };
            let tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, container(WritingMode::HorizontalTb, direction, padding))
                .style(1, child);
            let actual = run(&tree);
            assert_box(&actual, 0, (0.0, 0.0), (13.0, 13.0));
            assert_box(
                &actual,
                1,
                (
                    if direction == Direction::Ltr {
                        0.0
                    } else {
                        -1.0
                    },
                    0.0,
                ),
                (14.0, 14.0),
            );
            // The first oversized float is placed at its actual requested edge.
            // Its outward border plus any POSITIVE margin and parent padding:
            // LTR [0,14+max(m,0)+3]; RTL [−1−max(m,0)−3,13].
            let positive = if margin > 0.0 { 2.0 } else { 0.0 };
            assert_overflow(
                &actual,
                (
                    if direction == Direction::Ltr {
                        0.0
                    } else {
                        -4.0 - positive
                    },
                    0.0,
                ),
                (17.0 + positive, 17.0 + positive),
            );
        }
    }
}

fn absolute_does_not_own_padding<S: LayoutScalar>() {
    for include_in_flow in [false, true] {
        let padding = Edges {
            right: LengthOf::px(scalar::<S>(3.0)),
            bottom: LengthOf::px(scalar::<S>(3.0)),
            ..Edges::all(LengthOf::ZERO)
        };
        let mut tree = PublicLayoutTreeOf::new()
            .children(0, if include_in_flow { vec![1, 2] } else { vec![2] })
            .style(
                0,
                container(WritingMode::HorizontalTb, Direction::Ltr, padding),
            )
            .children(2, [])
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    inset: Edges {
                        left: LengthAutoOf::px(scalar::<S>(13.0)),
                        top: LengthAutoOf::px(scalar::<S>(13.0)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..sized(WritingMode::HorizontalTb, Direction::Ltr, 5.0, 5.0)
                },
            );
        if include_in_flow {
            tree = tree.children(1, []).style(
                1,
                NodeInputOf {
                    position: Position::Relative,
                    inset: Edges {
                        left: LengthAutoOf::px(scalar::<S>(4.0)),
                        top: LengthAutoOf::px(scalar::<S>(4.0)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..sized(WritingMode::HorizontalTb, Direction::Ltr, 10.0, 10.0)
                },
            );
        }
        let actual = run(&tree);
        assert_box(&actual, 0, (0.0, 0.0), (13.0, 13.0));
        assert_box(&actual, 2, (13.0, 13.0), (5.0, 5.0));
        if include_in_flow {
            assert_box(&actual, 1, (4.0, 4.0), (10.0, 10.0));
        }
        // Own padding box ends13; the real in-flow endpoint (when present)
        // requires17. Absolute overflow ends18, so UNION ends18, never21.
        assert_overflow(&actual, (0.0, 0.0), (18.0, 18.0));
    }
}

fn descendant_does_not_own_parent_padding<S: LayoutScalar>() {
    for clipped in [false, true] {
        let padding = Edges {
            right: LengthOf::px(scalar::<S>(3.0)),
            bottom: LengthOf::px(scalar::<S>(3.0)),
            ..Edges::all(LengthOf::ZERO)
        };
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [2])
            .children(2, [])
            .style(
                0,
                container(WritingMode::HorizontalTb, Direction::Ltr, padding),
            )
            .style(
                1,
                NodeInputOf {
                    position: Position::Relative,
                    overflow: if clipped {
                        computed_overflow(Overflow::Hidden, Overflow::Hidden)
                    } else {
                        ComputedOverflow::VISIBLE
                    },
                    inset: Edges {
                        left: LengthAutoOf::px(scalar::<S>(4.0)),
                        top: LengthAutoOf::px(scalar::<S>(4.0)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..sized(WritingMode::HorizontalTb, Direction::Ltr, 10.0, 10.0)
                },
            )
            .style(
                2,
                NodeInputOf {
                    position: Position::Relative,
                    inset: Edges {
                        left: LengthAutoOf::px(scalar::<S>(8.0)),
                        top: LengthAutoOf::px(scalar::<S>(8.0)),
                        ..Edges::all(LengthAutoOf::AUTO)
                    },
                    ..sized(WritingMode::HorizontalTb, Direction::Ltr, 10.0, 10.0)
                },
            );
        let actual = run(&tree);
        assert_box(&actual, 0, (0.0, 0.0), (13.0, 13.0));
        assert_box(&actual, 1, (4.0, 4.0), (10.0, 10.0));
        assert_box(&actual, 2, (8.0, 8.0), (10.0, 10.0));
        let own = output(&actual, 1)
            .scroll_geometry
            .unwrap()
            .scrollable_overflow();
        assert_eq!(own.origin(), Point::ZERO);
        assert_eq!(own.size(), Size::splat(scalar::<S>(18.0)));
        // The direct border requires14+3=17. Its descendant ends at4+8+10=22,
        // not25. Clipping removes that descendant contribution at the child
        // padding box; the child's own scroll range still reaches18.
        assert_overflow(
            &actual,
            (0.0, 0.0),
            if clipped { (17.0, 17.0) } else { (22.0, 22.0) },
        );
    }
}

fn atomic_line_endpoint<S: LayoutScalar>() {
    let padding = Edges {
        right: LengthOf::px(scalar::<S>(3.0)),
        ..Edges::all(LengthOf::ZERO)
    };
    let child = NodeInputOf {
        display: Display::InlineBlock,
        atomic_inline_participation: Some(fri06_atomic_participation()),
        position: Position::Relative,
        inset: Edges {
            left: LengthAutoOf::px(scalar::<S>(4.0)),
            ..Edges::all(LengthAutoOf::AUTO)
        },
        ..sized(WritingMode::HorizontalTb, Direction::Ltr, 10.0, 10.0)
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(
            0,
            container(WritingMode::HorizontalTb, Direction::Ltr, padding),
        )
        .style(1, child);
    let actual = run(&tree);
    // This is a numeric supplied level0 atomic, with no fabricated Text source
    // or font. The empty atomic is the line's only 10px metric contribution.
    assert_box(&actual, 0, (0.0, 0.0), (13.0, 10.0));
    assert_box(&actual, 1, (4.0, 0.0), (10.0, 10.0));
    assert_overflow(&actual, (0.0, 0.0), (17.0, 10.0));
}

macro_rules! scalar_cases {
    ($f32:ident, $f64:ident, $body:ident) => {
        #[test]
        fn $f32() {
            $body::<f32>();
        }
        #[test]
        fn $f64() {
            $body::<f64>();
        }
    };
}
scalar_cases!(
    terminal_padding_directed_physical_ends_f32,
    terminal_padding_directed_physical_ends_f64,
    all_directed_endpoints
);
scalar_cases!(
    terminal_padding_signed_margins_preserve_border_f32,
    terminal_padding_signed_margins_preserve_border_f64,
    signed_end_margins
);
scalar_cases!(
    terminal_padding_floated_content_f32,
    terminal_padding_floated_content_f64,
    floated_endpoints
);
scalar_cases!(
    absolute_overflow_does_not_gain_terminal_padding_f32,
    absolute_overflow_does_not_gain_terminal_padding_f64,
    absolute_does_not_own_padding
);
scalar_cases!(
    descendant_overflow_does_not_gain_parent_terminal_padding_f32,
    descendant_overflow_does_not_gain_parent_terminal_padding_f64,
    descendant_does_not_own_parent_padding
);
scalar_cases!(
    numeric_atomic_line_uses_actual_terminal_border_f32,
    numeric_atomic_line_uses_actual_terminal_border_f64,
    atomic_line_endpoint
);
