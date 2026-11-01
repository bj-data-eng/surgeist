//! Numeric box edges use actual immediate-parent sides in all five flow modes.
//! Literal physical positions and components are independent oracles over real
//! native pairs and boxes, without Text preparation or font geometry.
use super::*;

#[derive(Clone, Copy, Debug)]
enum Side {
    Top,
    Right,
    Bottom,
    Left,
}
impl Side {
    fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
        }
    }
    fn edges<S: LayoutScalar>(self, value: f64) -> Edges<S> {
        let n = S::from_f64(value);
        match self {
            Self::Top => Edges::new(n, S::ZERO, S::ZERO, S::ZERO),
            Self::Right => Edges::new(S::ZERO, n, S::ZERO, S::ZERO),
            Self::Bottom => Edges::new(S::ZERO, S::ZERO, n, S::ZERO),
            Self::Left => Edges::new(S::ZERO, S::ZERO, S::ZERO, n),
        }
    }
}
#[derive(Clone, Copy, Debug)]
struct FrameCase {
    mode: WritingMode,
    direction: Direction,
    start: Side,
    // Zero-baseline controls lie at actual block-start; body Top fills10px band.
    cross: f64,
    // Physical minimum inline coordinates, in authentic SOURCE node order.
    direct: [f64; 3], // [start1,body3,end5], inline extent15.
    nested: [f64; 5], // [Pstart1,Cstart2,body3,Cend4,Pend5], extent20.
}
const FRAMES: [FrameCase; 10] = [
    FrameCase {
        mode: WritingMode::HorizontalTb,
        direction: Direction::Ltr,
        start: Side::Left,
        cross: 0.0,
        direct: [0.0, 2.0, 12.0],
        nested: [0.0, 14.0, 4.0, 1.0, 16.0],
    },
    FrameCase {
        mode: WritingMode::HorizontalTb,
        direction: Direction::Rtl,
        start: Side::Right,
        cross: 0.0,
        direct: [13.0, 3.0, 0.0],
        nested: [19.0, 4.0, 6.0, 16.0, 0.0],
    },
    FrameCase {
        mode: WritingMode::VerticalRl,
        direction: Direction::Ltr,
        start: Side::Top,
        cross: 10.0,
        direct: [0.0, 2.0, 12.0],
        nested: [0.0, 14.0, 4.0, 1.0, 16.0],
    },
    FrameCase {
        mode: WritingMode::VerticalRl,
        direction: Direction::Rtl,
        start: Side::Bottom,
        cross: 10.0,
        direct: [13.0, 3.0, 0.0],
        nested: [19.0, 4.0, 6.0, 16.0, 0.0],
    },
    FrameCase {
        mode: WritingMode::VerticalLr,
        direction: Direction::Ltr,
        start: Side::Top,
        cross: 0.0,
        direct: [0.0, 2.0, 12.0],
        nested: [0.0, 14.0, 4.0, 1.0, 16.0],
    },
    FrameCase {
        mode: WritingMode::VerticalLr,
        direction: Direction::Rtl,
        start: Side::Bottom,
        cross: 0.0,
        direct: [13.0, 3.0, 0.0],
        nested: [19.0, 4.0, 6.0, 16.0, 0.0],
    },
    FrameCase {
        mode: WritingMode::SidewaysRl,
        direction: Direction::Ltr,
        start: Side::Top,
        cross: 10.0,
        direct: [0.0, 2.0, 12.0],
        nested: [0.0, 14.0, 4.0, 1.0, 16.0],
    },
    FrameCase {
        mode: WritingMode::SidewaysRl,
        direction: Direction::Rtl,
        start: Side::Bottom,
        cross: 10.0,
        direct: [13.0, 3.0, 0.0],
        nested: [19.0, 4.0, 6.0, 16.0, 0.0],
    },
    // SidewaysLr has physical line-left at the BOTTOM. The table directly
    // reflects each whole interval into extent15 or20, including strip width.
    FrameCase {
        mode: WritingMode::SidewaysLr,
        direction: Direction::Ltr,
        start: Side::Bottom,
        cross: 0.0,
        direct: [13.0, 3.0, 0.0],
        nested: [19.0, 4.0, 6.0, 16.0, 0.0],
    },
    FrameCase {
        mode: WritingMode::SidewaysLr,
        direction: Direction::Rtl,
        start: Side::Top,
        cross: 0.0,
        direct: [0.0, 2.0, 12.0],
        nested: [0.0, 14.0, 4.0, 1.0, 16.0],
    },
];
fn other(direction: Direction) -> Direction {
    match direction {
        Direction::Ltr => Direction::Rtl,
        Direction::Rtl => Direction::Ltr,
    }
}

fn endpoint<S: LayoutScalar>(
    frame: FrameCase,
    kind: InlineBoundaryKind,
    border: f64,
    padding: f64,
    parent: FlowAxes,
) -> InlineBoundaryInputOf<S> {
    InlineBoundaryInputOf::box_edge(
        kind,
        InlineMetricsOf::try_new(InlineBaselineContext::Physical, S::ZERO, S::ZERO).unwrap(),
        InlineBoundaryEdgeOf::try_new(S::ZERO, S::from_f64(border), S::from_f64(padding)).unwrap(),
        parent,
    )
    .with_writing_mode(frame.mode)
    .with_direction(frame.direction)
}
fn fixture<S: LayoutScalar>(frame: FrameCase, nested: bool) -> PublicLayoutTreeOf<S> {
    let formatter = FlowAxes::new(frame.mode, frame.direction);
    let p_own = FlowAxes::new(frame.mode, other(frame.direction));
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(
            0,
            if nested {
                vec![1, 2, 3, 4, 5]
            } else {
                vec![1, 3, 5]
            },
        )
        .style(
            0,
            NodeInputOf {
                display: Display::Block,
                writing_mode: frame.mode,
                direction: frame.direction,
                text_align: TextAlign::Start,
                ..NodeInputOf::default()
            },
        )
        .children(3, [])
        .style(
            3,
            NodeInputOf {
                display: Display::InlineBlock,
                writing_mode: frame.mode,
                direction: frame.direction,
                size: Size::new(
                    PreferredSizeOf::px(S::from_f64(10.0)),
                    PreferredSizeOf::px(S::from_f64(10.0)),
                ),
                vertical_align: VerticalAlignOf::Top,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(if frame.direction == Direction::Ltr {
                        0
                    } else {
                        1
                    })
                    .unwrap(),
                    InlineBreakOpportunity::Prohibited,
                )),
                ..NodeInputOf::default()
            },
        );
    for (node, kind, border, padding, parent) in if nested {
        vec![
            (1, InlineBoundaryKind::Start, 1.0, 0.0, formatter),
            (2, InlineBoundaryKind::Start, 1.0, 1.0, p_own),
            (4, InlineBoundaryKind::End, 2.0, 1.0, p_own),
            (5, InlineBoundaryKind::End, 1.0, 3.0, formatter),
        ]
    } else {
        vec![
            (1, InlineBoundaryKind::Start, 1.0, 1.0, formatter),
            (5, InlineBoundaryKind::End, 1.0, 2.0, formatter),
        ]
    } {
        let input = endpoint(frame, kind, border, padding, parent);
        // Preconditions distinguish actual formatter facts from the separately
        // requested parent basis. Geometry below is the decisive observation.
        assert_eq!(
            (input.writing_mode(), input.direction()),
            (frame.mode, frame.direction)
        );
        assert_eq!(input.parent_edge_axes(), Some(parent));
        tree = tree
            .children(node, [])
            .input(node, LayoutInputOf::inline_boundary(input));
    }
    tree
}
fn pair(start: u32, end: u32) -> GeneratedBoxRef<u32> {
    GeneratedBoxRef::Inline(InlineBoxRef::NativePair { start, end })
}
fn order(parent: GeneratedBoxRef<u32>, child: GeneratedBoxRef<u32>) -> InlineChildOrderInput<u32> {
    InlineChildOrderInput {
        parent,
        children: vec![GeneratedChildBoxInput {
            box_: child,
            participation: GeneratedBoxParticipation::InFlow,
        }],
        prefix: ChildOrderCompleteness::Complete,
        suffix: ChildOrderCompleteness::Complete,
        formatting_context: 0,
    }
}
fn wrapping(nested: bool) -> WrappingInput<u32> {
    let mut scopes = vec![WrappingScopeInput::NativePair {
        start: 1,
        end: 5,
        wrap_inside: WrapInside::Auto,
        edges: WrapEdges::AUTO,
        metric_contribution: None,
    }];
    let mut child_order = vec![order(GeneratedBoxRef::Node(0), pair(1, 5))];
    if nested {
        scopes.push(WrappingScopeInput::NativePair {
            start: 2,
            end: 4,
            wrap_inside: WrapInside::Auto,
            edges: WrapEdges::AUTO,
            metric_contribution: None,
        });
        child_order.push(order(pair(1, 5), pair(2, 4)));
        child_order.push(order(pair(2, 4), GeneratedBoxRef::Node(3)));
    } else {
        child_order.push(order(pair(1, 5), GeneratedBoxRef::Node(3)));
    }
    WrappingInput {
        native_inline_boxes: Vec::new(),
        gaps: Vec::new(),
        scopes,
        child_order,
    }
}
fn run<S: LayoutScalar>(
    tree: &PublicLayoutTreeOf<S>,
    nested: bool,
) -> CompletedLayoutBatchOf<u32, S> {
    let input =
        LayoutFlowInput::try_new(tree, 0, Vec::new(), wrapping(nested), Vec::new()).unwrap();
    compute_layout_with_flow(
        &input,
        0,
        LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
    )
    .unwrap()
}
fn check_strip<S: LayoutScalar>(
    actual: NodeOutputOf<S>,
    frame: FrameCase,
    inline: f64,
    advance: f64,
    side: Side,
    border: f64,
    padding: f64,
) {
    let horizontal = frame.mode == WritingMode::HorizontalTb;
    assert_eq!(
        actual.location,
        if horizontal {
            Point::new(S::from_f64(inline), S::ZERO)
        } else {
            Point::new(S::from_f64(frame.cross), S::from_f64(inline))
        },
        "frame={frame:?}"
    );
    assert_eq!(
        actual.size,
        if horizontal {
            Size::new(S::from_f64(advance), S::ZERO)
        } else {
            Size::new(S::ZERO, S::from_f64(advance))
        },
        "frame={frame:?}"
    );
    assert_eq!(actual.border, side.edges(border), "frame={frame:?}");
    assert_eq!(actual.padding, side.edges(padding), "frame={frame:?}");
}
fn lane<S: LayoutScalar>(nested: bool) {
    for frame in FRAMES {
        let tree = fixture::<S>(frame, nested);
        let batch = run(&tree, nested);
        assert!(batch.placed_text().is_empty());
        assert!(batch.final_inline_fragments().is_empty());
        let inline_extent = if nested { 20.0 } else { 15.0 };
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            let output = |node| {
                entries
                    .iter()
                    .find(|entry| entry.node() == node)
                    .unwrap()
                    .output()
            };
            let root = output(0);
            assert_eq!(
                root.size,
                if frame.mode == WritingMode::HorizontalTb {
                    Size::new(S::from_f64(inline_extent), S::from_f64(10.0))
                } else {
                    Size::new(S::from_f64(10.0), S::from_f64(inline_extent))
                },
                "frame={frame:?}"
            );
            let body_inline = if nested {
                frame.nested[2]
            } else {
                frame.direct[1]
            };
            let body = output(3);
            assert_eq!(
                body.location,
                if frame.mode == WritingMode::HorizontalTb {
                    Point::new(S::from_f64(body_inline), S::ZERO)
                } else {
                    Point::new(S::ZERO, S::from_f64(body_inline))
                },
                "frame={frame:?}"
            );
            assert_eq!(body.size, Size::new(S::from_f64(10.0), S::from_f64(10.0)));
            if nested {
                for (node, inline, advance, side, border, padding) in [
                    (1, frame.nested[0], 1.0, frame.start, 1.0, 0.0),
                    (2, frame.nested[1], 2.0, frame.start.opposite(), 1.0, 1.0),
                    (4, frame.nested[3], 3.0, frame.start, 2.0, 1.0),
                    (5, frame.nested[4], 4.0, frame.start.opposite(), 1.0, 3.0),
                ] {
                    check_strip(output(node), frame, inline, advance, side, border, padding);
                }
                for (rank, node) in [1, 2, 3, 4, 5].into_iter().enumerate() {
                    assert_eq!(output(node).source_index.get(), rank);
                }
            } else {
                check_strip(
                    output(1),
                    frame,
                    frame.direct[0],
                    2.0,
                    frame.start,
                    1.0,
                    1.0,
                );
                check_strip(
                    output(5),
                    frame,
                    frame.direct[2],
                    3.0,
                    frame.start.opposite(),
                    1.0,
                    2.0,
                );
                for (rank, node) in [1, 3, 5].into_iter().enumerate() {
                    assert_eq!(output(node).source_index.get(), rank);
                }
            }
        }
    }
}
fn formatter_mismatch<S: LayoutScalar>() {
    for frame in FRAMES {
        let mut tree = fixture::<S>(frame, true);
        // Keep genuine P parent basis; change ONLY the old formatter field.
        let changed = endpoint(
            frame,
            InlineBoundaryKind::Start,
            1.0,
            1.0,
            FlowAxes::new(frame.mode, other(frame.direction)),
        )
        .with_direction(other(frame.direction));
        tree.insert_input(2, LayoutInputOf::inline_boundary(changed));
        let input =
            LayoutFlowInput::try_new(&tree, 0, Vec::new(), wrapping(true), Vec::new()).unwrap();
        let error = compute_layout_with_flow(
            &input,
            0,
            LayoutRootRequestOf::viewport(Size::splat(AvailableOf::MAX_CONTENT)).unwrap(),
        )
        .expect_err("opposed actual parent basis does not replace formatter validation");
        assert_eq!(
            error.site(),
            LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 2
            }
        );
        assert_eq!(error.operation(), LayoutOperation::ChildLayout);
        assert!(
            matches!(error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::InlineFlowMismatch { expected,actual })
            if *expected == FlowAxes::new(frame.mode,frame.direction)
                && *actual == FlowAxes::new(frame.mode,other(frame.direction))),
            "{error:?}"
        );
    }
}
#[test]
fn declared_numeric_box_edges_preserve_direct_block_parent_all_frames_f32() {
    lane::<f32>(false);
}
#[test]
fn declared_numeric_box_edges_preserve_direct_block_parent_all_frames_f64() {
    lane::<f64>(false);
}
#[test]
fn declared_numeric_child_edges_use_immediate_opposing_parent_all_frames_f32() {
    lane::<f32>(true);
}
#[test]
fn declared_numeric_child_edges_use_immediate_opposing_parent_all_frames_f64() {
    lane::<f64>(true);
}
#[test]
fn numeric_parent_basis_does_not_replace_formatter_validation_f32() {
    formatter_mismatch::<f32>();
}
#[test]
fn numeric_parent_basis_does_not_replace_formatter_validation_f64() {
    formatter_mismatch::<f64>();
}

#[path = "parent_geometry_numeric_admission.rs"]
mod admission;

#[path = "parent_geometry_numeric/signed.rs"]
mod signed;

#[path = "parent_geometry_numeric/signed_native_fold.rs"]
mod signed_native_fold;
