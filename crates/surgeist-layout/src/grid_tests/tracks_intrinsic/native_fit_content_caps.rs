//! Source-derived fit-content caps through real positioned-root-local layout.
use super::*;
use crate::{LayoutTree, Traverse};
use std::cell::RefCell;

struct FitContentTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    calls: RefCell<Vec<(LeafMeasureInputOf<S>, Size<S>)>>,
}

impl<S: LayoutScalar> Traverse for FitContentTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = <PublicLayoutTreeOf<S> as Traverse>::Children<'a>
    where
        Self: 'a;

    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.tree, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.tree.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.tree.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for FitContentTree<S> {
    type MeasureError = core::convert::Infallible;

    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == 3
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != 3 {
            return None;
        }
        let n = S::from_f64;
        let known = input.known_content_size();
        let axis = input.content_sizing_request().axes().width;
        // Two ordinary native unbreakable40-wide pieces separated by a break
        // opportunity. Only a SIZE is supplied: no Grid output, font, Source,
        // baseline, track vector or prepared paragraph is fabricated.
        let width = known.width.unwrap_or_else(|| match axis.operation() {
            SizingOperation::MinContent => n(40.0),
            SizingOperation::MaxContent => n(80.0),
            SizingOperation::Normal => axis
                .space()
                .definite_size()
                .map_or(n(80.0), |band| band.get().max(n(40.0)).min(n(80.0))),
        });
        let measured = Size::new(width, known.height.unwrap_or(n(40.0)));
        self.calls.borrow_mut().push((input, measured));
        Some(Ok(measured))
    }
}

fn source<S: LayoutScalar>(
    cap: f64,
    overflow: Overflow,
    direction: Direction,
) -> FitContentTree<S> {
    let n = S::from_f64;
    let columns = vec![
        TrackComponentOf::Track(TrackSizingOf::MAX_CONTENT),
        TrackComponentOf::Track(TrackSizingOf::fit_content(SizingCalculationOf::value(
            LengthPercentageOf::px(n(cap)).unwrap(),
        ))),
    ];
    let mut tree = PublicLayoutTreeOf::new().children(0, [1, 2, 3]).style(
        0,
        NodeInputOf {
            display: Display::Grid,
            position: Position::Absolute,
            box_sizing: BoxSizing::BorderBox,
            writing_mode: WritingMode::HorizontalTb,
            direction,
            grid_template_columns: columns,
            grid_template_rows: vec![TrackComponentOf::px(n(40.0))],
            gap: Size::splat(LengthOf::px(S::ZERO)),
            justify_content: Some(AlignContent::Start),
            align_content: Some(AlignContent::Start),
            justify_items: Some(AlignItems::Stretch),
            align_items: Some(AlignItems::Stretch),
            ..NodeInputOf::default()
        },
    );
    for (node, first, last) in [(1, 1, 2), (2, 2, 3), (3, 1, 3)] {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Flex,
                position: Position::Relative,
                box_sizing: BoxSizing::BorderBox,
                writing_mode: WritingMode::HorizontalTb,
                direction,
                overflow: if node == 3 {
                    computed_overflow(overflow, overflow)
                } else {
                    computed_overflow(Overflow::Visible, Overflow::Visible)
                },
                grid_column: GridPlacement::try_lines(first, last).unwrap(),
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                ..NodeInputOf::default()
            },
        );
    }
    FitContentTree {
        tree,
        calls: RefCell::new(Vec::new()),
    }
}

fn host_frame<S: LayoutScalar>() -> Size<crate::NonNegativeFiniteOf<S>> {
    Size::new(
        crate::NonNegativeFiniteOf::new(S::from_f64(800.0)).unwrap(),
        crate::NonNegativeFiniteOf::new(S::from_f64(600.0)).unwrap(),
    )
}

fn fallback<S: LayoutScalar>() -> OrthogonalFallbackOf<S> {
    OrthogonalFallbackOf::new(
        host_frame::<S>().map(Some),
        crate::NearestScrollportOf::Absent,
        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
    )
}

fn positioned_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    // The native host declares the same genuine horizontal containing frame as
    // the source-qualified absolute-local profile: HTML/bodyLTR, hypothetical
    // Grid at inline-start0. Root/child direction remains independently varied.
    LayoutRootRequestOf::positioned_root_local(
        crate::RootLocalPositionedContextOf::horizontal_auto_insets(
            host_frame(),
            Direction::Ltr,
            S::ZERO,
            fallback(),
        )
        .unwrap(),
    )
    .unwrap()
}

fn intrinsic_leaf_request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    // Separate public leaf controls qualify40/80. They do not replace the
    // positioned-root request or claim a source parent for an isolated leaf.
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(host_frame::<S>().width),
                    operation,
                ),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback(),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::NONE,
    )
}

fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut found = entries.iter().filter(|entry| entry.node() == node);
    let result = found
        .next()
        .expect("actual source node is published")
        .output();
    assert!(found.next().is_none(), "one output for source node{node}");
    result
}

fn qualify_source<S: LayoutScalar>(tree: &FitContentTree<S>, cap: f64, direction: Direction) {
    let n = S::from_f64;
    assert_eq!(Traverse::children(tree, 0).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(tree.node_input(0).position, Position::Absolute);
    assert_eq!(tree.node_input(0).display, Display::Grid);
    assert_eq!(
        tree.node_input(0).grid_template_columns,
        vec![
            TrackComponentOf::Track(TrackSizingOf::MAX_CONTENT),
            TrackComponentOf::Track(TrackSizingOf::fit_content(SizingCalculationOf::value(
                LengthPercentageOf::px(n(cap)).unwrap(),
            ))),
        ]
    );
    assert_eq!(
        tree.node_input(0).grid_template_rows,
        vec![TrackComponentOf::px(n(40.0))]
    );
    for node in 0..=3 {
        let style = tree.node_input(node);
        assert_eq!(style.writing_mode, WritingMode::HorizontalTb);
        assert_eq!(style.direction, direction);
        assert_eq!(
            style.size,
            Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
        );
        assert_eq!(style.inset, Edges::all(LengthAutoOf::AUTO));
        assert_eq!(style.margin, Edges::all(LengthAutoOf::px(S::ZERO)));
        assert_eq!(style.border, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(style.padding, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(tree.has_leaf_measurement(node), node == 3);
        if node != 0 {
            assert_eq!(style.display, Display::Flex);
            assert_eq!(style.position, Position::Relative);
            assert_eq!(tree.child_count(node), 0);
        }
    }
    for (operation, width) in [
        (SizingOperation::MinContent, 40.0),
        (SizingOperation::MaxContent, 80.0),
    ] {
        let measured = compute_layout(tree, 3, intrinsic_leaf_request(operation))
            .expect("the genuine opaque leaf intrinsic control completes");
        for entries in [measured.unrounded_entries(), measured.final_entries()] {
            assert_eq!(output(entries, 3).size, Size::new(n(width), n(40.0)));
            assert_eq!(
                output(entries, 3).content_size,
                Size::new(n(width), n(40.0))
            );
        }
        assert!(tree.calls.borrow().iter().any(|(call, size)| {
            call.known_content_size().width.is_none()
                && call.content_sizing_request().axes().width.operation() == operation
                && *size == Size::new(n(width), n(40.0))
        }));
    }
    for node in [1, 2] {
        for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
            let empty = compute_layout(tree, node, intrinsic_leaf_request(operation))
                .expect("real unmeasured empty Flex single completes");
            assert_eq!(output(empty.unrounded_entries(), node).size, Size::ZERO);
            assert_eq!(output(empty.final_entries(), node).size, Size::ZERO);
        }
    }
    tree.calls.borrow_mut().clear();
}

fn qualify_root_receipts<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    let n = S::from_f64;
    // These are the actual executed root candidates retained by the public
    // completed batch, not another request or a supplied Grid response.
    for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
        let candidate = batch
            .cache_store_entries()
            .iter()
            .find(|entry| {
                let input = entry.input();
                entry.node() == 0
                    && input.run_mode() == RunMode::ComputeSize
                    && input.sizing_mode() == SizingMode::ContentSize
                    && input.requested_axis() == RequestedAxis::Horizontal
                    && input.sizing_request().axes().width.operation() == operation
            })
            .expect("the actual positioned MIN/MAX candidate has a public receipt");
        let input = candidate.input();
        assert_eq!(input.known(), Size::NONE);
        assert_eq!(
            input.parent(),
            host_frame::<S>().map(|extent| Some(extent.get()))
        );
        assert_eq!(
            input.containing_layout_context(),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::BlockFlow,
            )
        );
        assert_eq!(
            input.sizing_request().axes().width.space(),
            AvailableSpaceOf::Definite(host_frame::<S>().width)
        );
        assert_eq!(
            input.sizing_request().axes().height.operation(),
            SizingOperation::Normal
        );
        assert_eq!(
            input.sizing_request().axes().height.space(),
            AvailableSpaceOf::Indefinite
        );
        assert_eq!(
            input.sizing_request(),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(host_frame::<S>().width),
                        operation
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                fallback(),
            )
        );
        assert_eq!(candidate.output().size, Size::new(n(80.0), n(40.0)));
    }
}

fn qualify_box<S: LayoutScalar>(
    entries: &[LayoutOutputEntryOf<u32, S>],
    node: u32,
    location: Point<S>,
    size: Size<S>,
    source_index: usize,
    overflow: Overflow,
) {
    let item = output(entries, node);
    assert_eq!(item.source_index, SourceIndex::new(source_index));
    assert_eq!(item.location, location, "node{node}");
    assert_eq!(item.size, size, "node{node}");
    assert_eq!(item.content_size, size);
    assert_eq!(
        (item.margin, item.border, item.padding),
        (Edges::ZERO, Edges::ZERO, Edges::ZERO)
    );
    assert_eq!(item.content_box_size(), size);
    let scroll = item
        .scroll_geometry
        .expect("every completed box owns scroll geometry");
    for rect in [
        scroll.border_box(),
        scroll.padding_box(),
        scroll.content_box(),
        scroll.scrollport(),
        scroll.scrollable_overflow(),
    ] {
        assert_eq!(rect.origin(), Point::ZERO);
        assert_eq!(rect.size(), size);
    }
    assert_eq!(scroll.used_overflow_x(), overflow);
    assert_eq!(scroll.used_overflow_y(), overflow);
    assert_eq!(scroll.scrollbar_size(), Size::ZERO);
    let range = scroll.physical_range();
    assert_eq!(
        (
            range.x().minimum(),
            range.x().maximum(),
            range.y().minimum(),
            range.y().maximum()
        ),
        (S::ZERO, S::ZERO, S::ZERO, S::ZERO)
    );
}

fn complete<S: LayoutScalar>(cap: f64, overflow: Overflow, widths: [f64; 2]) {
    let n = S::from_f64;
    for direction in [Direction::Ltr, Direction::Rtl] {
        let tree = source::<S>(cap, overflow, direction);
        qualify_source(&tree, cap, direction);
        let request = positioned_request::<S>();
        let crate::LayoutRootContextOf::RootLocalPositioned(context) = request.context() else {
            panic!("the actual root must retain its positioned local context");
        };
        assert_eq!(context.containing_size(), host_frame());
        assert_eq!(
            context.containing_flow_axes(),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr)
        );
        assert_eq!(context.hypothetical_inline_offset(), S::ZERO);
        assert_eq!(context.orthogonal_fallback(), fallback());
        assert_eq!(
            request.sizing_request().axes().width.operation(),
            SizingOperation::Normal
        );
        assert_eq!(
            request.sizing_request().axes().width.space(),
            AvailableSpaceOf::Definite(host_frame::<S>().width)
        );
        assert_eq!(
            request.sizing_request().axes().height.space(),
            AvailableSpaceOf::Indefinite
        );
        let batch =
            compute_layout(&tree, 0, request).expect("actual positioned-root-local Grid completes");
        assert!(
            batch.placed_text().is_empty(),
            "the host supplies no Text/font identity"
        );
        qualify_root_receipts(&batch);
        // Both shrink-to-fit candidates genuinely query the child. The first
        // track's MaxContent minimum forces BOTH root candidates to80; actual
        // Normal layout then receives accepted80, never the outer800 as known.
        for (operation, width) in [
            (SizingOperation::MinContent, 40.0),
            (SizingOperation::MaxContent, 80.0),
        ] {
            assert!(
                tree.calls.borrow().iter().any(|(call, size)| {
                    call.known_content_size().width.is_none()
                        && call.content_sizing_request().axes().width.operation() == operation
                        && *size == Size::new(n(width), n(40.0))
                }),
                "real positioned query {operation:?}, {direction:?}"
            );
        }
        assert!(
            tree.calls.borrow().iter().any(|(call, size)| {
                let axis = call.content_sizing_request().axes().width;
                call.known_content_size().width.is_none()
                    && axis.operation() == SizingOperation::Normal
                    && axis
                        .space()
                        .definite_size()
                        .map(crate::NonNegativeFiniteOf::get)
                        == Some(n(80.0))
                    && call.containing_layout_context().flow_axes()
                        == FlowAxes::new(WritingMode::HorizontalTb, direction)
                    && *size == Size::new(n(80.0), n(40.0))
            }),
            "accepted Normal80 must reach the actual leaf, {direction:?}"
        );
        for entries in [batch.unrounded_entries(), batch.final_entries()] {
            assert_eq!(entries.len(), 4);
            for (node, logical_x, width, source_index) in [
                (0, 0.0, 80.0, 0),
                (1, 0.0, widths[0], 0),
                (2, widths[0], widths[1], 1),
                (3, 0.0, 80.0, 2),
            ] {
                let x = if direction == Direction::Rtl {
                    80.0 - logical_x - width
                } else {
                    logical_x
                };
                qualify_box(
                    entries,
                    node,
                    Point::new(n(x), S::ZERO),
                    Size::new(n(width), n(40.0)),
                    source_index,
                    if node == 3 {
                        overflow
                    } else {
                        Overflow::Visible
                    },
                );
            }
        }
    }
}

macro_rules! cohort {
    ($name:ident, $scalar:ty, $cap:expr, $overflow:expr, $widths:expr) => {
        #[test]
        fn $name() {
            complete::<$scalar>($cap, $overflow, $widths);
        }
    };
}

// Grid1 §§6.6,11.5,11.5.1: actual zero singles initialize finite growth0/0.
// Visible Minimum40 distributes30/10 under fit10; the sole MaxContent-minimum
// track then receives40 more. Hidden Minimum0 yields80/0 instead. With fit80,
// Visible Minimum40 splits20/20 and the MaxContent-minimum step yields60/20.
// These exact integers are independent raw AND publication oracles in both lanes.
cohort!(
    visible_fit10_f32,
    f32,
    10.0,
    Overflow::Visible,
    [70.0, 10.0]
);
cohort!(
    visible_fit10_f64,
    f64,
    10.0,
    Overflow::Visible,
    [70.0, 10.0]
);
cohort!(hidden_fit10_f32, f32, 10.0, Overflow::Hidden, [80.0, 0.0]);
cohort!(hidden_fit10_f64, f64, 10.0, Overflow::Hidden, [80.0, 0.0]);
cohort!(
    visible_fit80_f32,
    f32,
    80.0,
    Overflow::Visible,
    [60.0, 20.0]
);
cohort!(
    visible_fit80_f64,
    f64,
    80.0,
    Overflow::Visible,
    [60.0, 20.0]
);
