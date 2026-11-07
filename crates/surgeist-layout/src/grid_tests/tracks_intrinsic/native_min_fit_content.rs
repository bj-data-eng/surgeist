//! Real native min-content/fit-content Grid under Normal absolute shrink-to-fit.
//! No browser rectangle, Text/font identity or completed Grid output is supplied.
use super::*;
use crate::test_support::readback::{self, Phase};
use crate::{LayoutTree, Traverse};
use std::cell::RefCell;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn nn<S: LayoutScalar>(value: f64) -> crate::NonNegativeFiniteOf<S> {
    crate::NonNegativeFiniteOf::new(n(value)).unwrap()
}
fn fallback<S: LayoutScalar>() -> OrthogonalFallbackOf<S> {
    OrthogonalFallbackOf::new(
        Size::new(Some(nn(800.0)), Some(nn(600.0))),
        crate::NearestScrollportOf::Absent,
        Size::splat(crate::OrthogonalFallbackLimitOf::Unbounded),
    )
}
#[derive(Clone, Copy, Debug)]
enum FitCap {
    Ten,
    Thirty,
    Eighty,
}
impl FitCap {
    fn value(self) -> f64 {
        match self {
            Self::Ten => 10.0,
            Self::Thirty => 30.0,
            Self::Eighty => 80.0,
        }
    }
}
struct NativeContent<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    calls: RefCell<Vec<LeafMeasureInputOf<S>>>,
}
impl<S: LayoutScalar> Traverse for NativeContent<S> {
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
impl<S: LayoutScalar> LayoutTree for NativeContent<S> {
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
        self.calls.borrow_mut().push(input);
        let known = input.known_content_size();
        // Native size policy: two unbreakable40-wide pieces, natural width80.
        // Respect actual known content; otherwise dispatch the actual per-axis
        // operation and Normal finite band. This returns only a leaf Size.
        let width = known.width.unwrap_or_else(|| {
            let axis = input.content_sizing_request().axes().width;
            match axis.operation() {
                SizingOperation::MinContent => n(40.0),
                SizingOperation::MaxContent => n(80.0),
                SizingOperation::Normal => axis
                    .space()
                    .definite_size()
                    .map_or(n(80.0), |space| space.get().max(n(40.0)).min(n(80.0))),
            }
        });
        let natural_height = if width < n(80.0) { n(20.0) } else { n(10.0) };
        Some(Ok(Size::new(width, known.height.unwrap_or(natural_height))))
    }
}

fn source<S: LayoutScalar>(
    cap: FitCap,
    overflow: Overflow,
    direction: Direction,
) -> NativeContent<S> {
    let mut tree = PublicLayoutTreeOf::new().children(0, [1, 2, 3]).style(
        0,
        NodeInputOf {
            display: Display::Grid,
            position: Position::Absolute,
            direction,
            grid_template_columns: vec![
                TrackComponentOf::MIN_CONTENT,
                TrackComponentOf::Track(TrackSizingOf::fit_content(SizingCalculationOf::value(
                    LengthPercentageOf::px(n(cap.value())).unwrap(),
                ))),
            ],
            grid_template_rows: vec![TrackComponentOf::px(n(40.0))],
            ..NodeInputOf::default()
        },
    );
    for node in 1..=3 {
        tree = tree.children(node, []).style(
            node,
            NodeInputOf {
                display: Display::Flex,
                direction,
                grid_column: if node == 3 {
                    GridPlacement::try_lines(1, 3).unwrap()
                } else {
                    GridPlacement::try_lines(node as isize, node as isize + 1).unwrap()
                },
                grid_row: GridPlacement::try_lines(1, 2).unwrap(),
                overflow: computed_overflow(
                    if node == 3 {
                        overflow
                    } else {
                        Overflow::Visible
                    },
                    if node == 3 {
                        overflow
                    } else {
                        Overflow::Visible
                    },
                ),
                ..NodeInputOf::default()
            },
        );
    }
    NativeContent {
        tree,
        calls: RefCell::new(Vec::new()),
    }
}
fn formal_request<S: LayoutScalar>(operation: SizingOperation) -> LayoutRootRequestOf<S> {
    // Full CB percentage/fallback facts remain separate from the
    // explicit intrinsic operation on otherwise unbounded physical inline space.
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::new(
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation),
                AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
            ),
            fallback(),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::new(Some(nn(800.0)), Some(nn(600.0))),
    )
}
fn local_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let context = crate::RootLocalPositionedContextOf::horizontal_auto_insets(
        Size::new(nn(800.0), nn(600.0)),
        Direction::Ltr,
        S::ZERO,
        fallback(),
    )
    .unwrap();
    let request = LayoutRootRequestOf::positioned_root_local(context).unwrap();
    assert_eq!(
        request.sizing_request().axes(),
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(nn(800.0)),
                SizingOperation::Normal
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        )
    );
    request
}
fn output<S: LayoutScalar>(entries: &[LayoutOutputEntryOf<u32, S>], node: u32) -> NodeOutputOf<S> {
    let mut rows = entries.iter().filter(|entry| entry.node() == node);
    let value = rows.next().expect("actual native box has output").output();
    assert!(rows.next().is_none());
    value
}
fn qualify<S: LayoutScalar>(
    host: &NativeContent<S>,
    cap: FitCap,
    overflow: Overflow,
    direction: Direction,
) {
    assert_eq!(Traverse::children(host, 0).collect::<Vec<_>>(), [1, 2, 3]);
    assert_eq!(host.node_input(0).position, Position::Absolute);
    assert_eq!(
        host.node_input(0).size,
        Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::AUTO)
    );
    assert_eq!(
        host.node_input(0).grid_template_columns,
        vec![
            TrackComponentOf::MIN_CONTENT,
            TrackComponentOf::Track(TrackSizingOf::fit_content(SizingCalculationOf::value(
                LengthPercentageOf::px(n(cap.value())).unwrap()
            )))
        ]
    );
    assert_eq!(
        host.node_input(0).grid_template_rows,
        vec![TrackComponentOf::px(n(40.0))]
    );
    assert_eq!(
        host.node_input(3).grid_column,
        GridPlacement::try_lines(1, 3).unwrap()
    );
    assert_eq!(
        host.node_input(3).overflow,
        computed_overflow(overflow, overflow)
    );
    for node in 0..=3 {
        let input = host.node_input(node);
        assert_eq!(input.direction, direction);
        assert_eq!(input.writing_mode, WritingMode::HorizontalTb);
        assert_eq!(input.margin, Edges::all(LengthAutoOf::px(S::ZERO)));
        assert_eq!(input.border, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(input.padding, Edges::all(LengthOf::px(S::ZERO)));
        assert_eq!(host.has_leaf_measurement(node), node == 3);
        if node != 0 {
            assert_eq!(host.child_count(node), 0);
        }
    }
    // Real fixed row40 is the opposite allocation for the Grid leaf query.
    for (operation, width) in [
        (SizingOperation::MinContent, 40.0),
        (SizingOperation::MaxContent, 80.0),
    ] {
        host.calls.borrow_mut().clear();
        let query = ComputeInputOf::for_child(
            RunMode::ComputeSize,
            SizingMode::ContentSize,
            RequestedAxis::Horizontal,
            Size::new(None, Some(n(40.0))),
            Size::new(None, Some(n(40.0))),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, direction),
                ParentFormattingContext::Grid,
            ),
            formal_request::<S>(operation).sizing_request(),
        );
        let leaf =
            crate::measurement::compute_tree_leaf(host, 3, query, host.node_input(3)).unwrap();
        assert_eq!(leaf.size, Size::new(n(width), n(40.0)));
        assert_eq!(leaf.content_size, leaf.size);
        assert_eq!(leaf.baselines(), BaselinesOf::NONE);
        assert_eq!(leaf.named_baselines, NamedBaselinesOf::unknown());
        assert!(
            host.calls
                .borrow()
                .iter()
                .any(|call| call.known_content_size().width.is_none()
                    && call.content_sizing_request().axes().width.operation() == operation)
        );
        for node in [1, 2] {
            let zero = compute_layout(host, node, formal_request(operation)).unwrap();
            assert_eq!(
                readback::single_box_output(&zero, node, Phase::Unrounded)
                    .size
                    .width,
                S::ZERO
            );
        }
        // Public formal Grid operation, independently of the later Normal root
        // request. Auto root properties add no suppressed preferred-size fact.
        let candidate = compute_layout(host, 0, formal_request(operation)).unwrap();
        for entries in [
            readback::continuous_custody(&candidate, Phase::Unrounded).as_slice(),
            readback::continuous_custody(&candidate, Phase::Final).as_slice(),
        ] {
            assert_eq!(entries.len(), 4);
            let root = output(entries, 0);
            assert_eq!(root.location, Point::ZERO);
            assert_eq!(root.size, Size::new(n(width), n(40.0)));
        }
    }
    host.calls.borrow_mut().clear();
}

fn scroll<S: LayoutScalar>(item: NodeOutputOf<S>, direction: Direction, overflow: Overflow) {
    let rect = ScrollRectOf::try_new(Point::ZERO, item.size).unwrap();
    let geometry = item
        .scroll_geometry
        .expect("accepted native owner retains own scroll facts");
    assert_eq!(
        geometry.flow_axes(),
        FlowAxes::new(WritingMode::HorizontalTb, direction)
    );
    assert_eq!(geometry.border_box(), rect);
    assert_eq!(geometry.padding_box(), rect);
    assert_eq!(geometry.content_box(), rect);
    assert_eq!(geometry.scrollport(), rect);
    assert_eq!(geometry.scrollable_overflow(), rect);
    assert_eq!(geometry.used_overflow_x(), overflow);
    assert_eq!(geometry.used_overflow_y(), overflow);
    let clip = geometry.overflow_clip();
    if overflow == Overflow::Hidden {
        let x = clip.x().unwrap();
        let y = clip.y().unwrap();
        assert_eq!((x.minimum(), x.maximum()), (S::ZERO, item.size.width));
        assert_eq!((y.minimum(), y.maximum()), (S::ZERO, item.size.height));
    } else {
        assert_eq!(clip.x(), None);
        assert_eq!(clip.y(), None);
    }
    let range = geometry.physical_range();
    assert_eq!(
        (range.x().minimum(), range.x().maximum()),
        (S::ZERO, S::ZERO)
    );
    assert_eq!(
        (range.y().minimum(), range.y().maximum()),
        (S::ZERO, S::ZERO)
    );
    assert_eq!(geometry.scrollbar_size(), Size::ZERO);
    let gutters = geometry.gutters();
    assert_eq!(
        (
            gutters.top(),
            gutters.right(),
            gutters.bottom(),
            gutters.left()
        ),
        (None, None, None, None)
    );
    assert_eq!(geometry.resolved_scroll_padding(), Edges::ZERO);
    assert_eq!(geometry.optimal_viewing_region(), rect);
    let target = geometry.target();
    assert_eq!(target.border_box(), rect);
    assert_eq!(
        target.flow_axes(),
        FlowAxes::new(WritingMode::HorizontalTb, direction)
    );
}
fn qualify_executed_candidates<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>) {
    // These are receipts from THIS positioned-root computation. They do not
    // depend on an independently issued formal control or a performed nonleaf
    // cache entry. No incidental query count or ordering is required.
    for (operation, expected) in [
        (SizingOperation::MinContent, 40.0),
        (SizingOperation::MaxContent, 80.0),
    ] {
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
            .expect("actual positioned root MIN/MAX candidate has a public receipt");
        let input = candidate.input();
        assert_eq!(input.known(), Size::NONE);
        assert_eq!(input.parent(), Size::new(Some(n(800.0)), Some(n(600.0))));
        assert_eq!(
            input.containing_layout_context(),
            ContainingLayoutContext::new(
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                ParentFormattingContext::BlockFlow,
            )
        );
        assert_eq!(
            input.sizing_request(),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Definite(nn(800.0)), operation),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                fallback(),
            )
        );
        assert_eq!(candidate.output().size, Size::new(n(expected), n(40.0)));
    }
}

fn normal<S: LayoutScalar>(cap: FitCap, overflow: Overflow, direction: Direction) {
    let host = source::<S>(cap, overflow, direction);
    qualify(&host, cap, overflow, direction);
    // Grid1 intrinsic MIN/MAX phases give40/80 even for Hidden minimum0.
    // The actual positioned shrink clamp is max(40,min(800,80))=80.
    // Normal then reconstructs different capped tracks. No Auto maximum
    // participates in stretch, so these need not fill the accepted container.
    let (a, b) = match (cap, overflow) {
        (FitCap::Ten, Overflow::Visible) => (30.0, 10.0),
        (FitCap::Ten, Overflow::Hidden) => (40.0, 10.0),
        (FitCap::Thirty, Overflow::Visible) => (20.0, 30.0),
        (FitCap::Thirty, Overflow::Hidden) => (40.0, 30.0),
        (FitCap::Eighty, Overflow::Visible) => (20.0, 60.0),
        (FitCap::Eighty, Overflow::Hidden) => (40.0, 40.0),
        _ => unreachable!("closed authored cap/overflow cohort"),
    };
    let total = a + b;
    let a_x = if direction == Direction::Rtl {
        80.0 - a
    } else {
        0.0
    };
    let b_x = if direction == Direction::Rtl {
        80.0 - total
    } else {
        a
    };
    let span_x = if direction == Direction::Rtl {
        80.0 - total
    } else {
        0.0
    };
    let actual =
        compute_layout(&host, 0, local_request()).expect("actual Normal local root completes");
    assert!(actual.placed_text().is_empty());
    qualify_executed_candidates(&actual);
    // Performed leaf measurement intentionally clears known content while
    // retaining the actual accepted Grid-area band T, not outerCB800/root80.
    assert!(
        host.calls.borrow().iter().any(|call| {
            call.known_content_size() == Size::NONE
                && call.content_sizing_request().axes()
                    == Size::new(
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(nn(total)),
                            SizingOperation::Normal,
                        ),
                        AxisSizingRequestOf::new(
                            AvailableSpaceOf::Definite(nn(40.0)),
                            SizingOperation::Normal,
                        ),
                    )
                && call.flow_axes() == FlowAxes::new(WritingMode::HorizontalTb, direction)
                && call.containing_layout_context().flow_axes()
                    == FlowAxes::new(WritingMode::HorizontalTb, direction)
        }),
        "actual Normal leaf must receive accepted span width{total}, {direction:?}"
    );
    for entries in [
        readback::continuous_custody(&actual, Phase::Unrounded).as_slice(),
        readback::continuous_custody(&actual, Phase::Final).as_slice(),
    ] {
        assert_eq!(entries.len(), 4);
        for (node, source, x, width) in [
            (0, 0, 0.0, 80.0),
            (1, 0, a_x, a),
            (2, 1, b_x, b),
            (3, 2, span_x, total),
        ] {
            let item = output(entries, node);
            assert_eq!(item.source_index, SourceIndex::new(source));
            assert_eq!(item.location, Point::new(n(x), S::ZERO), "node{node}");
            assert_eq!(item.size, Size::new(n(width), n(40.0)), "node{node}");
            assert_eq!(
                (item.margin, item.border, item.padding),
                (Edges::ZERO, Edges::ZERO, Edges::ZERO)
            );
            assert_eq!(item.content_box_size(), item.size);
            // Positive padding seeds include each complete40-high viewport;
            // natural native content wraps into10/20px and stays inside it.
            assert_eq!(item.content_size, item.size, "node{node} own content");
            scroll(
                item,
                direction,
                if node == 3 {
                    overflow
                } else {
                    Overflow::Visible
                },
            );
        }
    }
}
macro_rules! cohort {
    ($name:ident,$scalar:ty,$cap:expr,$overflow:expr,$direction:expr) => {
        #[test]
        fn $name() {
            normal::<$scalar>($cap, $overflow, $direction);
        }
    };
}
cohort!(
    ten_visible_ltr_f32,
    f32,
    FitCap::Ten,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    ten_visible_ltr_f64,
    f64,
    FitCap::Ten,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    ten_visible_rtl_f32,
    f32,
    FitCap::Ten,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    ten_visible_rtl_f64,
    f64,
    FitCap::Ten,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    ten_hidden_ltr_f32,
    f32,
    FitCap::Ten,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    ten_hidden_ltr_f64,
    f64,
    FitCap::Ten,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    ten_hidden_rtl_f32,
    f32,
    FitCap::Ten,
    Overflow::Hidden,
    Direction::Rtl
);
cohort!(
    ten_hidden_rtl_f64,
    f64,
    FitCap::Ten,
    Overflow::Hidden,
    Direction::Rtl
);
cohort!(
    thirty_visible_ltr_f32,
    f32,
    FitCap::Thirty,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    thirty_visible_ltr_f64,
    f64,
    FitCap::Thirty,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    thirty_visible_rtl_f32,
    f32,
    FitCap::Thirty,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    thirty_visible_rtl_f64,
    f64,
    FitCap::Thirty,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    thirty_hidden_ltr_f32,
    f32,
    FitCap::Thirty,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    thirty_hidden_ltr_f64,
    f64,
    FitCap::Thirty,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    thirty_hidden_rtl_f32,
    f32,
    FitCap::Thirty,
    Overflow::Hidden,
    Direction::Rtl
);
cohort!(
    thirty_hidden_rtl_f64,
    f64,
    FitCap::Thirty,
    Overflow::Hidden,
    Direction::Rtl
);
cohort!(
    eighty_visible_ltr_f32,
    f32,
    FitCap::Eighty,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    eighty_visible_ltr_f64,
    f64,
    FitCap::Eighty,
    Overflow::Visible,
    Direction::Ltr
);
cohort!(
    eighty_visible_rtl_f32,
    f32,
    FitCap::Eighty,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    eighty_visible_rtl_f64,
    f64,
    FitCap::Eighty,
    Overflow::Visible,
    Direction::Rtl
);
cohort!(
    eighty_hidden_ltr_f32,
    f32,
    FitCap::Eighty,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    eighty_hidden_ltr_f64,
    f64,
    FitCap::Eighty,
    Overflow::Hidden,
    Direction::Ltr
);
cohort!(
    eighty_hidden_rtl_f32,
    f32,
    FitCap::Eighty,
    Overflow::Hidden,
    Direction::Rtl
);
cohort!(
    eighty_hidden_rtl_f64,
    f64,
    FitCap::Eighty,
    Overflow::Hidden,
    Direction::Rtl
);
