//! Functional qualification of explicit root-border-local positioned requests.
//! All expected widths come from the native/real Ahem MIN30/MAX70 content,
//! CB percentages, and continuous local corner rounding.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::readback::{self, Phase};
use crate::test_support::text::{PreparedParagraph, ahem_style};
use surgeist_text as text;

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn nn<S: LayoutScalar>(value: f64) -> NonNegativeFiniteOf<S> {
    NonNegativeFiniteOf::new(n(value)).unwrap()
}
fn fallback<S: LayoutScalar>() -> OrthogonalFallbackOf<S> {
    OrthogonalFallbackOf::new(
        Size::new(Some(nn(600.0)), Some(nn(40.0))),
        NearestScrollportOf::Absent,
        Size::splat(OrthogonalFallbackLimitOf::Unbounded),
    )
}
fn context<S: LayoutScalar>(offset: f64, direction: Direction) -> RootLocalPositionedContextOf<S> {
    RootLocalPositionedContextOf::horizontal_auto_insets(
        Size::new(nn(100.0), nn(80.0)),
        direction,
        n(offset),
        fallback(),
    )
    .unwrap()
}
fn request<S: LayoutScalar>(offset: f64, direction: Direction) -> LayoutRootRequestOf<S> {
    LayoutRootRequestOf::positioned_root_local(context(offset, direction)).unwrap()
}
fn explicit_static_block_offset_normalizes_independent_availability<S: LayoutScalar>() {
    let original = context::<S>(13.0, Direction::Ltr);
    assert_eq!(original.hypothetical_block_offset(), None);
    let without_block_offset = LayoutRootRequestOf::positioned_root_local(original).unwrap();
    assert_eq!(
        without_block_offset.sizing_request().axes().height.space(),
        AvailableSpaceOf::Indefinite
    );

    for (offset, remaining) in [(27.0, 53.0), (-20.0, 100.0), (91.0, 0.0)] {
        let exact = original.with_hypothetical_block_offset(n(offset)).unwrap();
        assert_eq!(exact.hypothetical_block_offset(), Some(n(offset)));
        assert_eq!(exact.containing_size(), Size::new(nn(100.0), nn(80.0)));
        let sizing = LayoutRootRequestOf::positioned_root_local(exact)
            .unwrap()
            .sizing_request();
        assert_eq!(
            sizing.axes(),
            Size::new(
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(nn(87.0)),
                    SizingOperation::Normal
                ),
                AxisSizingRequestOf::new(
                    AvailableSpaceOf::Definite(nn(remaining)),
                    SizingOperation::Normal
                ),
            )
        );
        assert_eq!(sizing.orthogonal_fallback(), fallback());
    }
    assert!(
        original
            .with_hypothetical_block_offset(n(f64::NAN))
            .is_err()
    );
}

#[test]
fn static_block_offset_qualifies_both_scalar_lanes() {
    explicit_static_block_offset_normalizes_independent_availability::<f32>();
    explicit_static_block_offset_normalizes_independent_availability::<f64>();
}

fn root_style<S: LayoutScalar>(direction: Direction) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        position: Position::Absolute,
        direction,
        ..NodeInputOf::default()
    }
}
fn output<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    finalized: bool,
) -> NodeOutputOf<S> {
    let entries = if finalized {
        readback::continuous_custody(batch, Phase::Final)
    } else {
        readback::continuous_custody(batch, Phase::Unrounded)
    };
    entries
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

// Two explicit opaque Native formatting policies, not invented Grid outputs.
// Words has unbreakable width30, natural width70, and a 10px line box.
// OrthogonalStrip has natural physical10x50 and consumes a required fallback
// only for its own real orthogonal Normal+Indefinite inline operation.
#[derive(Clone, Copy)]
enum NativeContent {
    Words,
    OrthogonalStrip,
    VerticalWords,
}
struct NativeProbe<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    measured_node: u32,
    content: NativeContent,
    calls: RefCell<Vec<LeafMeasureInputOf<S>>>,
}
impl<S: LayoutScalar> NativeProbe<S> {
    fn words(style: NodeInputOf<S>) -> Self {
        Self {
            tree: PublicLayoutTreeOf::new().children(0, []).style(0, style),
            measured_node: 0,
            content: NativeContent::Words,
            calls: RefCell::new(Vec::new()),
        }
    }
}
impl<S: LayoutScalar> Traverse for NativeProbe<S> {
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
impl<S: LayoutScalar> LayoutTree for NativeProbe<S> {
    type MeasureError = OrthogonalFallbackErrorOf<S>;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn has_leaf_measurement(&self, node: u32) -> bool {
        node == self.measured_node
    }
    fn measure_leaf(
        &self,
        node: u32,
        input: LeafMeasureInputOf<S>,
    ) -> Option<Result<Size<S>, Self::MeasureError>> {
        if node != self.measured_node {
            return None;
        }
        self.calls.borrow_mut().push(input);
        let packet = input.content_sizing_request();
        let measured = match self.content {
            NativeContent::Words => {
                let width = match packet.axes().width.operation() {
                    SizingOperation::MinContent => n(30.0),
                    SizingOperation::MaxContent => n(70.0),
                    SizingOperation::Normal => {
                        input.known_content_size().width.unwrap_or_else(|| {
                            packet
                                .axes()
                                .width
                                .space()
                                .definite_size()
                                .map_or(n(70.0), |space| space.get().max(n(30.0)).min(n(70.0)))
                        })
                    }
                };
                Size::new(width, if width < n(70.0) { n(20.0) } else { n(10.0) })
            }
            NativeContent::VerticalWords => {
                assert_eq!(input.flow_axes().inline_axis(), PhysicalAxis::Vertical);
                assert_eq!(
                    input.containing_layout_context().flow_axes().inline_axis(),
                    PhysicalAxis::Horizontal
                );
                let height = match packet.axes().height.operation() {
                    SizingOperation::MinContent => n(40.0),
                    SizingOperation::MaxContent => n(80.0),
                    SizingOperation::Normal => {
                        input.known_content_size().height.unwrap_or_else(|| {
                            packet
                                .axes()
                                .height
                                .space()
                                .definite_size()
                                .map_or(n(80.0), |space| space.get().max(n(40.0)).min(n(80.0)))
                        })
                    }
                };
                Size::new(n(20.0), height)
            }
            NativeContent::OrthogonalStrip => {
                assert_eq!(input.flow_axes().inline_axis(), PhysicalAxis::Vertical);
                assert_eq!(
                    input.containing_layout_context().flow_axes().inline_axis(),
                    PhysicalAxis::Horizontal
                );
                let inline = packet.axes().height;
                let mut height = n::<S>(50.0);
                if inline.operation() == SizingOperation::Normal {
                    if let Some(space) = inline.space().definite_size() {
                        height = height.min(space.get());
                    } else if input.known_content_size().height.is_none() {
                        match packet.orthogonal_fallback().height {
                            Ok(space) => height = height.min(space.get()),
                            Err(error) => return Some(Err(error)),
                        }
                    }
                }
                Size::new(n(10.0), height)
            }
        };
        Some(Ok(input
            .known_content_size()
            .zip_map(measured, |known, value| known.unwrap_or(value))))
    }
}

fn construction<S: LayoutScalar>() {
    for (offset, remaining) in [
        (0.0, 100.0),
        (50.0, 50.0),
        (80.0, 20.0),
        (100.0, 0.0),
        (120.0, 0.0),
        (-20.0, 120.0),
        (49.75, 50.25),
    ] {
        let request = request::<S>(offset, Direction::Rtl);
        let actual = request.sizing_request();
        assert_eq!(
            actual.axes().width,
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(nn(remaining)),
                SizingOperation::Normal
            )
        );
        assert_eq!(
            actual.axes().height,
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal)
        );
        assert_eq!(actual.orthogonal_fallback(), fallback());
        assert_eq!(
            request.context(),
            LayoutRootContextOf::RootLocalPositioned(context(offset, Direction::Rtl))
        );
    }
    let zero = context::<S>(-0.0, Direction::Ltr);
    assert!(
        !zero
            .hypothetical_inline_offset()
            .to_f64()
            .is_sign_negative()
    );
    for invalid in [S::NAN, S::INFINITY, -S::INFINITY] {
        assert!(matches!(
            RootLocalPositionedContextOf::horizontal_auto_insets(
                Size::new(nn(100.0), nn(80.0)),
                Direction::Ltr,
                invalid,
                fallback(),
            ),
            Err(FiniteScalarErrorOf::NonFinite { .. })
        ));
    }
    let largest = if core::mem::size_of::<S>() == core::mem::size_of::<f32>() {
        S::from_f32(f32::MAX)
    } else {
        S::from_f64(f64::MAX)
    };
    let overflow = RootLocalPositionedContextOf::horizontal_auto_insets(
        Size::new(NonNegativeFiniteOf::new(largest).unwrap(), nn(80.0)),
        Direction::Ltr,
        -largest,
        fallback(),
    )
    .unwrap();
    assert!(matches!(
        LayoutRootRequestOf::positioned_root_local(overflow),
        Err(NonNegativeFiniteScalarErrorOf::NonFinite { .. })
    ));
}

fn native_widths<S: LayoutScalar>() {
    for containing_direction in [Direction::Ltr, Direction::Rtl] {
        for own_direction in [Direction::Ltr, Direction::Rtl] {
            let host = NativeProbe::words(root_style::<S>(own_direction));
            for (offset, width, final_width, height) in [
                (0.0, 70.0, 70.0, 10.0),
                (50.0, 50.0, 50.0, 20.0),
                (80.0, 30.0, 30.0, 20.0),
                (120.0, 30.0, 30.0, 20.0),
                (-20.0, 70.0, 70.0, 10.0),
                (49.75, 50.25, 50.0, 20.0),
            ] {
                let batch =
                    compute_layout(&host, 0, request(offset, containing_direction)).unwrap();
                for finalized in [false, true] {
                    let root = output(&batch, 0, finalized);
                    assert_eq!(root.location, Point::ZERO);
                    assert_eq!(
                        root.size,
                        Size::new(n(if finalized { final_width } else { width }), n(height)),
                        "offset{offset}, containing{containing_direction:?}, own{own_direction:?}; actual Normal packets: {:?}",
                        host.calls
                            .borrow()
                            .iter()
                            .filter(
                                |input| input.content_sizing_request().axes().width.operation()
                                    == SizingOperation::Normal
                            )
                            .map(|input| (
                                input.known_content_size(),
                                input.content_sizing_request().axes()
                            ))
                            .collect::<Vec<_>>()
                    );
                }
            }
            let calls = host.calls.borrow();
            for operation in [
                SizingOperation::MinContent,
                SizingOperation::MaxContent,
                SizingOperation::Normal,
            ] {
                assert!(calls.iter().any(|input| {
                    input.content_sizing_request().axes().width.operation() == operation
                }));
            }
            assert!(
                calls
                    .iter()
                    .all(|input| input.content_sizing_request().axes().height.space()
                        == AvailableSpaceOf::Indefinite)
            );
            drop(calls);
            // The existing capability retains containing-frame initial-start
            // semantics. The new local capability does not rewrite it.
            let old = LayoutRootRequestOf::positioned_auto_insets(
                PositionedRootContextOf::horizontal_auto_insets(
                    Size::new(nn(100.0), nn(80.0)),
                    containing_direction,
                    fallback(),
                ),
            );
            let old_batch = compute_layout(&host, 0, old).unwrap();
            assert_eq!(
                output(&old_batch, 0, false).location,
                Point::new(
                    n(if containing_direction == Direction::Rtl {
                        30.0
                    } else {
                        0.0
                    }),
                    S::ZERO
                )
            );
        }
    }
}

fn percentage_basis<S: LayoutScalar>() {
    let mut style = root_style::<S>(Direction::Ltr);
    style.size = Size::new(
        PreferredSizeOf::percent(n(0.5)),
        PreferredSizeOf::percent(n(0.25)),
    );
    let host = NativeProbe::words(style);
    let ctx = RootLocalPositionedContextOf::horizontal_auto_insets(
        Size::new(nn(200.0), nn(80.0)),
        Direction::Rtl,
        n(140.0),
        fallback(),
    )
    .unwrap();
    let request = LayoutRootRequestOf::positioned_root_local(ctx).unwrap();
    assert_eq!(
        request.sizing_request().axes().width.space(),
        AvailableSpaceOf::Definite(nn(60.0))
    );
    assert_eq!(
        request.sizing_request().axes().height.space(),
        AvailableSpaceOf::Indefinite
    );
    let batch = compute_layout(&host, 0, request).unwrap();
    for finalized in [false, true] {
        let root = output(&batch, 0, finalized);
        assert_eq!(root.location, Point::ZERO);
        assert_eq!(
            root.size,
            Size::new(n(100.0), n(20.0)),
            "percentages use fullCB200x80, not remaining60 or omitted block space"
        );
    }
}

fn fixed_orthogonal_root<S: LayoutScalar>() {
    let containing_size = Size::new(nn(192.0), nn(80.0));
    for (direction, containing_direction) in [
        (Direction::Ltr, Direction::Rtl),
        (Direction::Rtl, Direction::Ltr),
    ] {
        for box_sizing in [BoxSizing::BorderBox, BoxSizing::ContentBox] {
            let mut root = root_style::<S>(direction);
            root.display = Display::Flex;
            root.writing_mode = WritingMode::SidewaysRl;
            root.box_sizing = box_sizing;
            root.size = Size::new(
                PreferredSizeOf::percent(n(0.5)),
                PreferredSizeOf::percent(n(0.25)),
            );
            root.max_size.width = MaxSizeOf::percent(n(0.375));
            root.min_size.height =
                MinSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.5)).unwrap());
            let child = NodeInputOf {
                display: Display::Block,
                flex_shrink: FlexShrinkOf::try_new(S::ZERO).unwrap(),
                size: Size::new(
                    PreferredSizeOf::percent(n(0.5)),
                    PreferredSizeOf::percent(n(0.5)),
                ),
                ..NodeInputOf::default()
            };
            let tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, root)
                .style(1, child);
            let context = RootLocalPositionedContextOf::horizontal_auto_insets(
                containing_size,
                containing_direction,
                n(160.0),
                fallback(),
            )
            .unwrap();
            let request = LayoutRootRequestOf::positioned_root_local(context).unwrap();
            let batch = compute_layout(&tree, 0, request).unwrap();
            for finalized in [false, true] {
                let root = output(&batch, 0, finalized);
                assert_eq!(root.location, Point::ZERO);
                // Full 192×80 physical percentage bases yield preferred 96×20.
                // The authored max-width caps it at 72 and min-height floors it at 40.
                assert_eq!(root.size, Size::new(n(72.0), n(40.0)));
                assert_eq!(
                    root.scroll_geometry.unwrap().flow_axes(),
                    FlowAxes::new(WritingMode::SidewaysRl, direction)
                );
                // The child uses the accepted 72×40 content box as its basis.
                assert_eq!(
                    output(&batch, 1, finalized).size,
                    Size::new(n(36.0), n(20.0))
                );
            }
        }
    }
}

fn fixed_root_with_static_top<S: LayoutScalar>() {
    let containing = context::<S>(70.0, Direction::Ltr);
    let request = LayoutRootRequestOf::positioned_root_local(containing).unwrap();
    for mode in [
        WritingMode::HorizontalTb,
        WritingMode::VerticalRl,
        WritingMode::VerticalLr,
        WritingMode::SidewaysRl,
        WritingMode::SidewaysLr,
    ] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let mut root = root_style::<S>(direction);
            root.writing_mode = mode;
            root.size = Size::new(
                PreferredSizeOf::percent(n(0.75)),
                PreferredSizeOf::percent(n(0.5)),
            );
            root.max_size.width = MaxSizeOf::percent(n(0.5));
            let child = NodeInputOf {
                display: Display::Block,
                writing_mode: mode,
                direction,
                size: Size::new(PreferredSizeOf::px(n(12.0)), PreferredSizeOf::px(n(10.0))),
                ..NodeInputOf::default()
            };
            let tree = PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, root)
                .style(1, child);
            let baseline = compute_layout(&tree, 0, request).unwrap();
            for top in [0.0, 70.0] {
                let positioned = LayoutRootRequestOf::positioned_root_local(
                    containing.with_hypothetical_block_offset(n(top)).unwrap(),
                )
                .unwrap();
                let batch = compute_layout(&tree, 0, positioned).unwrap_or_else(|error| {
                    panic!("{mode:?}/{direction:?} at top {top}: {error:?}")
                });
                for finalized in [false, true] {
                    let root = output(&batch, 0, finalized);
                    assert_eq!(root.location, Point::ZERO);
                    // The actual containing block is 100×80 despite the offsets.
                    assert_eq!(root.size, Size::new(n(50.0), n(40.0)));
                    let child = output(&batch, 1, finalized);
                    assert_eq!(child.size, Size::new(n(12.0), n(10.0)));
                    assert_eq!(child.location, output(&baseline, 1, finalized).location);
                }
            }
        }
    }
}

fn orthogonal_root_clamps_definite_calculation<S: LayoutScalar>() {
    let mut style = root_style::<S>(Direction::Rtl);
    style.display = Display::Flex;
    style.writing_mode = WritingMode::SidewaysRl;
    style.size = Size::new(
        PreferredSizeOf::value(LengthPercentageOf::from_coefficients(n(-64.0), n(0.25)).unwrap()),
        PreferredSizeOf::percent(n(0.25)),
    );
    style.min_size.width = MinSizeOf::px(n(12.0));
    let tree = PublicLayoutTreeOf::new().children(0, []).style(0, style);
    let context = RootLocalPositionedContextOf::horizontal_auto_insets(
        Size::new(nn(192.0), nn(80.0)),
        Direction::Ltr,
        n(160.0),
        fallback(),
    )
    .unwrap();
    let request = LayoutRootRequestOf::positioned_root_local(context).unwrap();
    let batch = compute_layout(&tree, 0, request).unwrap();
    for finalized in [false, true] {
        // The definite width calculation resolves to -16 before the normal
        // zero/minimum floor. The unconstrained height is 25% of 80.
        assert_eq!(
            output(&batch, 0, finalized).size,
            Size::new(n(12.0), n(20.0))
        );
    }
}

fn auto_orthogonal_root_uses_the_static_block_offset<S: LayoutScalar>() {
    let containing = RootLocalPositionedContextOf::<S>::horizontal_auto_insets(
        Size::new(nn(120.0), nn(100.0)),
        Direction::Ltr,
        n(52.0),
        fallback(),
    )
    .unwrap();
    for mode in [WritingMode::VerticalLr, WritingMode::VerticalRl] {
        for direction in [Direction::Ltr, Direction::Rtl] {
            for (offset, min_height, max_height, expected) in [
                (70.0, MinSizeOf::AUTO, MaxSizeOf::NONE, 40.0),
                (50.0, MinSizeOf::AUTO, MaxSizeOf::NONE, 50.0),
                (10.0, MinSizeOf::AUTO, MaxSizeOf::NONE, 80.0),
                (
                    50.0,
                    MinSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.75)).unwrap()),
                    MaxSizeOf::NONE,
                    75.0,
                ),
                (50.0, MinSizeOf::AUTO, MaxSizeOf::percent(n(0.25)), 25.0),
            ] {
                let mut style = root_style(direction);
                style.writing_mode = mode;
                style.size.width = PreferredSizeOf::percent(n(0.75));
                style.max_size.width = MaxSizeOf::percent(n(0.60));
                style.min_size.height = min_height;
                style.max_size.height = max_height;
                let mut host = NativeProbe::words(style);
                host.content = NativeContent::VerticalWords;
                let request = LayoutRootRequestOf::positioned_root_local(
                    containing
                        .with_hypothetical_block_offset(n(offset))
                        .unwrap(),
                )
                .unwrap();
                let batch = compute_layout(&host, 0, request).unwrap();
                for final_output in [false, true] {
                    let root = output(&batch, 0, final_output);
                    assert_eq!(root.location, Point::ZERO);
                    assert_eq!(root.size, Size::new(n(72.0), n(expected)));
                }
                // Known root dimensions allow final layout to publish
                // without another measurement of this leaf.
                let calls = host.calls.borrow();
                for operation in [SizingOperation::MinContent, SizingOperation::MaxContent] {
                    assert!(
                        calls.iter().any(|input| {
                            let packet = input.content_sizing_request();
                            packet.axes().width.operation() == SizingOperation::Normal
                                && packet.axes().height.operation() == operation
                                && packet.axes().height.space()
                                    == AvailableSpaceOf::Definite(nn(100.0 - offset))
                                && packet.orthogonal_fallback()
                                    == Size::new(Ok(nn(600.0)), Ok(nn(40.0)))
                                && input.known_content_size().width == Some(n(72.0))
                        }),
                        "missing {operation:?} packet at {mode:?}/{direction:?}, offset {offset}"
                    );
                }
            }
        }
    }
}

fn fixed_orthogonal_root_exclusions<S: LayoutScalar>() {
    let mut fixed = root_style::<S>(Direction::Rtl);
    fixed.display = Display::Flex;
    fixed.writing_mode = WritingMode::SidewaysRl;
    fixed.size = Size::new(PreferredSizeOf::px(n(72.0)), PreferredSizeOf::px(n(40.0)));
    let local = request(0.0, Direction::Ltr);
    let assert_refused = |style, request, reason| {
        let tree = PublicLayoutTreeOf::new().children(0, []).style(0, style);
        let error = compute_layout(&tree, 0, request).unwrap_err();
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(
            matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(actual)) if *actual == reason),
            "{error:?}"
        );
    };
    for preferred in [
        PreferredSizeOf::AUTO,
        PreferredSizeOf::MIN_CONTENT,
        PreferredSizeOf::MAX_CONTENT,
    ] {
        let mut auto_width = fixed.clone();
        auto_width.size.width = preferred.clone();
        assert_refused(auto_width, local, RootContextMismatch::WritingMode);
        let mut auto_height = fixed.clone();
        auto_height.size.height = preferred;
        assert_refused(auto_height, local, RootContextMismatch::WritingMode);
    }
    let mut padded = fixed.clone();
    padded.padding.left = LengthOf::px(n(1.0));
    assert_refused(padded, local, RootContextMismatch::BoxEdges);
    let mut aligned = fixed.clone();
    aligned.align_self = Some(AlignItems::Start);
    assert_refused(aligned, local, RootContextMismatch::Alignment);
    let mut ratio = fixed.clone();
    ratio.aspect_ratio = AspectRatioOf::new(S::ONE);
    assert_refused(ratio, local, RootContextMismatch::AspectRatio);
    let initial = LayoutRootRequestOf::positioned_auto_insets(
        PositionedRootContextOf::horizontal_auto_insets(
            Size::new(nn(192.0), nn(80.0)),
            Direction::Ltr,
            fallback(),
        ),
    );
    assert_refused(fixed, initial, RootContextMismatch::WritingMode);
}

fn independent_fallback<S: LayoutScalar>() {
    let mut root = root_style::<S>(Direction::Ltr);
    root.size.width = PreferredSizeOf::px(n(30.0));
    let host = NativeProbe {
        tree: PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [])
            .style(0, root)
            .style(
                1,
                NodeInputOf {
                    display: Display::Block,
                    writing_mode: WritingMode::VerticalRl,
                    ..NodeInputOf::default()
                },
            ),
        measured_node: 1,
        content: NativeContent::OrthogonalStrip,
        calls: RefCell::new(Vec::new()),
    };
    let batch = compute_layout(&host, 0, request(50.0, Direction::Ltr)).unwrap();
    assert_eq!(output(&batch, 0, false).size, Size::new(n(30.0), n(40.0)));
    assert_eq!(output(&batch, 1, false).size.height, n(40.0));
    let calls = host.calls.borrow();
    assert!(
        calls.iter().any(|input| {
            let packet = input.content_sizing_request();
            packet.axes().height.operation() == SizingOperation::Normal
                && packet.axes().height.space() == AvailableSpaceOf::Indefinite
                && input.known_content_size().height.is_none()
                && packet.orthogonal_fallback().height == Ok(nn(40.0))
        }),
        "actual orthogonal opaque provider consumes independent40 fallback, notCBheight80 or remaining inline50"
    );
}

fn local_rounding<S: LayoutScalar>() {
    let mut root = root_style::<S>(Direction::Ltr);
    root.size = Size::new(PreferredSizeOf::px(n(10.4)), PreferredSizeOf::px(n(10.4)));
    let child = NodeInputOf {
        display: Display::Block,
        size: Size::new(PreferredSizeOf::px(n(2.4)), PreferredSizeOf::px(n(2.4))),
        inset: Edges {
            left: LengthAutoOf::px(n(0.4)),
            top: LengthAutoOf::px(n(0.4)),
            ..Edges::all(LengthAutoOf::AUTO)
        },
        ..NodeInputOf::default()
    };
    let tree = PublicLayoutTreeOf::new()
        .children(0, [1])
        .children(1, [])
        .style(0, root)
        .style(1, child);
    for offset in [0.25, 0.75] {
        let batch = compute_layout(&tree, 0, request(offset, Direction::Rtl)).unwrap();
        let unrounded = output(&batch, 0, false);
        assert_eq!(unrounded.location, Point::ZERO);
        assert_eq!(unrounded.size, Size::new(n(10.4), n(10.4)));
        let root = output(&batch, 0, true);
        assert_eq!(root.location, Point::ZERO);
        assert_eq!(root.size, Size::new(n(10.0), n(10.0)));
        let child = output(&batch, 1, true);
        assert_eq!(child.location, Point::ZERO);
        assert_eq!(
            child.size,
            Size::new(n(3.0), n(3.0)),
            "round local corners .4..2.8; do not snap a CB translation then subtract it"
        );
    }
}

fn profile_refusals<S: LayoutScalar>() {
    let mut cases = Vec::new();
    let mut add = |reason, mutate: fn(&mut NodeInputOf<S>)| {
        let mut style = root_style::<S>(Direction::Ltr);
        mutate(&mut style);
        cases.push((reason, style));
    };
    add(RootContextMismatch::Position, |s| {
        s.position = Position::Relative
    });
    add(RootContextMismatch::WritingMode, |s| {
        s.writing_mode = WritingMode::VerticalRl
    });
    add(RootContextMismatch::Display, |s| {
        s.display = Display::InlineBlock
    });
    add(RootContextMismatch::Float, |s| s.float = Float::Left);
    add(RootContextMismatch::Clearance, |s| s.clear = Clear::Both);
    add(RootContextMismatch::BoxEdges, |s| {
        s.padding.left = LengthOf::px(n(1.0))
    });
    add(RootContextMismatch::BoxEdges, |s| {
        s.margin.left = LengthAutoOf::px(n(1.0))
    });
    add(RootContextMismatch::Insets, |s| {
        s.inset.left = LengthAutoOf::ZERO
    });
    add(RootContextMismatch::Replaced, |s| s.item_is_replaced = true);
    add(RootContextMismatch::AspectRatio, |s| {
        s.aspect_ratio = AspectRatioOf::new(S::ONE)
    });
    add(RootContextMismatch::Alignment, |s| {
        s.align_self = Some(AlignItems::Start)
    });
    add(RootContextMismatch::Alignment, |s| {
        s.justify_self = Some(AlignItems::Start)
    });
    for (reason, style) in cases {
        let host = NativeProbe::words(style);
        let error = compute_layout(&host, 0, request(50.0, Direction::Ltr)).unwrap_err();
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(
            matches!(error.kind(), LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(actual)) if *actual == reason),
            "{error:?}"
        );
        assert!(
            host.calls.borrow().is_empty(),
            "profile validation precedes actual native measurement"
        );
    }
}

fn real_text<S: LayoutScalar>() {
    let paragraph = PreparedParagraph::plain(
        1,
        text::Source::new("AAA BBB"),
        text::Style {
            white_space: text::WhiteSpace::Collapse,
            wrap: text::Wrap::Word,
            ..ahem_style(10.0, 10.0)
        },
    );
    let tree = paragraph.flat_text_tree(0, root_style::<S>(Direction::Ltr));
    let flow = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![paragraph.binding(0)],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    // The source/font-backed content supplies MIN30/MAX70 before the
    // local request exercises shrink-to-fit. Intrinsic geometry is not reused
    // as final Normal placement.
    for (operation, expected) in [
        (SizingOperation::MinContent, 30.0),
        (SizingOperation::MaxContent, 70.0),
    ] {
        let intrinsic = LayoutRootRequestOf::standalone(
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, operation),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                fallback(),
            ),
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            Size::NONE,
        );
        let batch = compute_layout_with_flow(&flow, 0, intrinsic).unwrap();
        assert_eq!(output(&batch, 0, false).size.width, n(expected));
    }
    for (offset, expected_width, expected_lines) in
        [(0.0, 70.0, 1), (50.0, 50.0, 2), (80.0, 30.0, 2)]
    {
        let batch = compute_layout_with_flow(&flow, 0, request(offset, Direction::Rtl)).unwrap();
        for finalized in [false, true] {
            let root = output(&batch, 0, finalized);
            assert_eq!(root.location, Point::ZERO);
            assert_eq!(
                root.size,
                Size::new(n(expected_width), n(expected_lines as f64 * 10.0))
            );
        }
        let placed = batch
            .placed_text()
            .iter()
            .find(|placed| placed.container() == 0)
            .unwrap();
        assert_eq!(placed.nodes(), &[1]);
        assert_eq!(
            placed.layout().source().text(),
            &text::Text::from("AAA BBB")
        );
        assert_eq!(placed.layout().lines().len(), expected_lines);
        for (index, line) in placed.layout().lines().iter().enumerate() {
            assert_eq!(line.baseline(), 8.0 + index as f64 * 10.0);
        }
        let mut units = Vec::new();
        let mut seen_lines = Vec::new();
        for entry in batch
            .final_inline_fragments()
            .iter()
            .filter(|entry| entry.owner() == &InlineFragmentOwner::Node(1))
        {
            let issued = entry.fragment().finalized_fragment();
            assert_eq!(issued.view().origin(), Some(paragraph.origin(1)));
            let line = issued.finalized_line();
            assert_eq!(line.prepared_text(), paragraph.prepared);
            if seen_lines.contains(&entry.fragment().line_index()) {
                continue;
            }
            seen_lines.push(entry.fragment().line_index());
            assert_eq!(
                line.feedback().advance,
                if expected_lines == 1 { 70.0 } else { 30.0 }
            );
            for mapping in line.mapping_views() {
                let range = mapping.source_range();
                units.extend(range.start..range.end);
            }
            for run in line.run_views() {
                assert_eq!(run.effective_instance().unwrap().font_size(), 10.0);
                assert!(
                    run.glyphs()
                        .filter(|glyph| glyph.advance() != 0.0)
                        .all(|glyph| glyph.advance() == 10.0)
                );
            }
        }
        assert_eq!(seen_lines.len(), expected_lines);
        units.sort_unstable();
        units.dedup();
        assert_eq!(units, (0..7).collect::<Vec<_>>());
    }
}

macro_rules! lanes {
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
lanes!(
    root_local_auto_orthogonal_fit_f32,
    root_local_auto_orthogonal_fit_f64,
    auto_orthogonal_root_uses_the_static_block_offset
);
lanes!(
    root_local_construction_f32,
    root_local_construction_f64,
    construction
);
lanes!(
    root_local_native_widths_f32,
    root_local_native_widths_f64,
    native_widths
);
lanes!(
    root_local_percentage_basis_f32,
    root_local_percentage_basis_f64,
    percentage_basis
);
lanes!(
    root_local_fixed_orthogonal_f32,
    root_local_fixed_orthogonal_f64,
    fixed_orthogonal_root
);
lanes!(
    root_local_fixed_static_top_f32,
    root_local_fixed_static_top_f64,
    fixed_root_with_static_top
);
lanes!(
    root_local_fixed_orthogonal_calculation_f32,
    root_local_fixed_orthogonal_calculation_f64,
    orthogonal_root_clamps_definite_calculation
);
lanes!(
    root_local_fixed_orthogonal_exclusions_f32,
    root_local_fixed_orthogonal_exclusions_f64,
    fixed_orthogonal_root_exclusions
);
lanes!(
    root_local_independent_fallback_f32,
    root_local_independent_fallback_f64,
    independent_fallback
);
lanes!(
    root_local_rounding_f32,
    root_local_rounding_f64,
    local_rounding
);
lanes!(
    root_local_profile_refusals_f32,
    root_local_profile_refusals_f64,
    profile_refusals
);
lanes!(
    root_local_real_text_f32,
    root_local_real_text_f64,
    real_text
);

// A fixed own inline preferred size is distinct from edge-sensitive Auto
// shrink-to-fit. These real Grid roots use authored sizes, never measured Grid
// outputs. The existing zero-edge request is the baseline for the same inputs.
fn fixed_padding_border<S: LayoutScalar>(box_sizing: BoxSizing) {
    for case in 0..3 {
        let mut root = root_style::<S>(Direction::Ltr);
        root.display = Display::Grid;
        root.box_sizing = box_sizing;
        root.size = if case == 1 {
            // The real full CB is200x400; remaining inline availability is20.
            Size::new(
                PreferredSizeOf::percent(n(0.5)),
                PreferredSizeOf::percent(n(0.25)),
            )
        } else {
            Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::px(n(100.0)))
        };
        if case == 2 {
            root.max_size.width = MaxSizeOf::px(n(80.0));
            root.min_size.height = MinSizeOf::px(n(120.0));
        }
        let make_tree = |root: NodeInputOf<S>| {
            PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, root)
                .style(
                    1,
                    NodeInputOf {
                        display: Display::Block,
                        box_sizing: BoxSizing::BorderBox,
                        size: Size::new(PreferredSizeOf::px(n(10.0)), PreferredSizeOf::px(n(10.0))),
                        ..NodeInputOf::default()
                    },
                )
        };
        let context = RootLocalPositionedContextOf::horizontal_auto_insets(
            Size::new(nn(200.0), nn(400.0)),
            Direction::Rtl,
            n(180.0),
            fallback(),
        )
        .unwrap();
        let request = LayoutRootRequestOf::positioned_root_local(context).unwrap();
        assert_eq!(
            request.sizing_request().axes().width.space(),
            AvailableSpaceOf::Definite(nn(20.0))
        );
        let used = if case == 2 {
            Size::new(n(80.0), n(120.0))
        } else {
            Size::new(n(100.0), n(100.0))
        };
        let baseline = compute_layout(&make_tree(root.clone()), 0, request).unwrap();
        for finalized in [false, true] {
            let actual = output(&baseline, 0, finalized);
            assert_eq!(actual.location, Point::ZERO);
            assert_eq!(
                actual.size, used,
                "case{case}: authentic zero-edge fixed/percentage/constraint baseline"
            );
            assert_eq!(
                actual.scroll_geometry.unwrap().content_box().origin(),
                Point::ZERO
            );
            assert_eq!(actual.scroll_geometry.unwrap().content_box().size(), used);
            assert_eq!(output(&baseline, 1, finalized).location, Point::ZERO);
            assert_eq!(
                output(&baseline, 1, finalized).size,
                Size::new(n(10.0), n(10.0))
            );
        }
        root.padding = Edges::all(LengthOf::px(n(10.0)));
        root.border = Edges::all(LengthOf::px(n(10.0)));
        let tree = make_tree(root);
        let batch = compute_layout(&tree, 0, request)
            .expect("a definite own inline preferred size admits actual padding and border");
        // PB totals40 per physical axis. Content-box authoring adds that to
        // used100 (or constrained80/120); border-box authoring includes it.
        let (border, content) = match box_sizing {
            BoxSizing::ContentBox => (Size::new(used.width + n(40.0), used.height + n(40.0)), used),
            BoxSizing::BorderBox => (used, Size::new(used.width - n(40.0), used.height - n(40.0))),
        };
        for finalized in [false, true] {
            let actual = output(&batch, 0, finalized);
            assert_eq!(actual.location, Point::ZERO);
            assert_eq!(
                actual.size, border,
                "case{case}, {box_sizing:?}: real fixed border dimensions"
            );
            assert_eq!(actual.padding, Edges::all(n(10.0)));
            assert_eq!(actual.border, Edges::all(n(10.0)));
            assert_eq!(actual.margin, Edges::all(S::ZERO));
            assert_eq!(actual.content_box_size(), content);
            let geometry = actual.scroll_geometry.unwrap();
            assert_eq!(
                geometry.content_box().origin(),
                Point::new(n(20.0), n(20.0))
            );
            assert_eq!(geometry.content_box().size(), content);
            let child = output(&batch, 1, finalized);
            assert_eq!(
                child.location,
                Point::new(n(20.0), n(20.0)),
                "actual Grid content start"
            );
            assert_eq!(child.size, Size::new(n(10.0), n(10.0)));
        }
    }
}
fn fixed_padding_border_content_box<S: LayoutScalar>() {
    fixed_padding_border::<S>(BoxSizing::ContentBox);
}
fn fixed_padding_border_border_box<S: LayoutScalar>() {
    fixed_padding_border::<S>(BoxSizing::BorderBox);
}
fn padding_border_profile_guards<S: LayoutScalar>() {
    let mut fixed = root_style::<S>(Direction::Ltr);
    fixed.display = Display::Grid;
    fixed.size = Size::new(PreferredSizeOf::px(n(100.0)), PreferredSizeOf::px(n(100.0)));
    fixed.padding = Edges::all(LengthOf::px(n(10.0)));
    fixed.border = Edges::all(LengthOf::px(n(10.0)));
    let assert_box_edges = |style: NodeInputOf<S>, request| {
        let tree = PublicLayoutTreeOf::new().children(0, []).style(0, style);
        let error = compute_layout(&tree, 0, request).unwrap_err();
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(
            matches!(
                error.kind(),
                LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(
                    RootContextMismatch::BoxEdges
                ))
            ),
            "{error:?}"
        );
    };
    for width in [
        PreferredSizeOf::AUTO,
        PreferredSizeOf::MIN_CONTENT,
        PreferredSizeOf::MAX_CONTENT,
    ] {
        let mut unsupported = fixed.clone();
        unsupported.size.width = width;
        assert_box_edges(unsupported, request(50.0, Direction::Ltr));
    }
    for margin in [-1.0, 1.0] {
        let mut unsupported = fixed.clone();
        unsupported.margin.left = LengthAutoOf::px(n(margin));
        assert_box_edges(unsupported, request(50.0, Direction::Ltr));
    }
    // The initial-start positioned capability remains zero-edge, even for a
    // fixed root. Only the separate root-local profile is extended.
    assert_box_edges(
        fixed,
        LayoutRootRequestOf::positioned_auto_insets(
            PositionedRootContextOf::horizontal_auto_insets(
                Size::new(nn(200.0), nn(400.0)),
                Direction::Ltr,
                fallback(),
            ),
        ),
    );
}
lanes!(
    root_local_fixed_padding_border_content_box_f32,
    root_local_fixed_padding_border_content_box_f64,
    fixed_padding_border_content_box
);
lanes!(
    root_local_fixed_padding_border_border_box_f32,
    root_local_fixed_padding_border_border_box_f64,
    fixed_padding_border_border_box
);
lanes!(
    root_local_padding_border_profile_guards_f32,
    root_local_padding_border_profile_guards_f64,
    padding_border_profile_guards
);

fn inset_container_request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let extent = nn(100.0);
    LayoutRootRequestOf::standalone(
        SizingRequestOf::new(
            Size::splat(AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(extent),
                SizingOperation::Normal,
            )),
            fallback(),
        ),
        FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        Size::splat(Some(extent)),
    )
}

fn assert_positioned_flex_leaf_uses_absolute_percentage_basis<S: LayoutScalar>() {
    let margin = LengthAutoOf::percent(n(0.1));
    let tree = NativeProbe {
        tree: PublicLayoutTreeOf::<S>::new()
            .children(1, [2])
            .style(
                1,
                NodeInputOf {
                    display: Display::Flex,
                    position: Position::Relative,
                    box_sizing: BoxSizing::BorderBox,
                    size: Size::splat(n(100.0)).map(PreferredSizeOf::px),
                    padding: Edges::all(LengthOf::px(n(10.0))),
                    ..NodeInputOf::default()
                },
            )
            .style(
                2,
                NodeInputOf {
                    position: Position::Absolute,
                    size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(n(20.0))),
                    inset: Edges::all(LengthAutoOf::px(n(20.0))),
                    margin: Edges {
                        left: margin,
                        right: margin,
                        top: LengthAutoOf::ZERO,
                        bottom: LengthAutoOf::ZERO,
                    },
                    justify_self: Some(AlignItems::Center),
                    ..NodeInputOf::default()
                },
            ),
        measured_node: 2,
        content: NativeContent::Words,
        calls: RefCell::new(Vec::new()),
    };
    let batch = compute_layout(&tree, 1, inset_container_request()).unwrap();
    let child = output(&batch, 2, false);
    let calls = tree.calls.borrow();
    let request = calls.last().expect("the positioned child was measured");
    // The absolute padding box is 100px. Its 60px inset interval leaves
    // 40px for the measured child after two 10px percentage margins.
    assert_eq!(
        (
            request
                .content_sizing_request()
                .axes()
                .width
                .space()
                .definite_size(),
            child.size.width,
            child.location.x,
            child.margin.left,
            child.margin.right,
        ),
        (Some(nn(40.0)), n(40.0), n(30.0), n(10.0), n(10.0))
    );
}

#[test]
fn positioned_flex_leaf_uses_absolute_percentage_basis() {
    assert_positioned_flex_leaf_uses_absolute_percentage_basis::<f32>();
    assert_positioned_flex_leaf_uses_absolute_percentage_basis::<f64>();
}

fn assert_positioned_block_minimum_sizes_descendants<S: LayoutScalar>() {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(1, [2])
        .children(2, [3])
        .style(
            1,
            NodeInputOf {
                display: Display::Block,
                position: Position::Relative,
                size: Size::splat(n(100.0)).map(PreferredSizeOf::px),
                ..NodeInputOf::default()
            },
        )
        .style(
            2,
            NodeInputOf {
                display: Display::Block,
                position: Position::Absolute,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(n(20.0))),
                min_size: Size::new(MinSizeOf::px(n(80.0)), MinSizeOf::AUTO),
                inset: Edges::all(LengthAutoOf::px(n(20.0))),
                justify_self: Some(AlignItems::Center),
                ..NodeInputOf::default()
            },
        )
        .style(
            3,
            NodeInputOf {
                display: Display::Block,
                size: Size::new(PreferredSizeOf::AUTO, PreferredSizeOf::px(n(5.0))),
                ..NodeInputOf::default()
            },
        );
    let batch = compute_layout(&tree, 1, inset_container_request()).unwrap();
    let child = output(&batch, 2, false);
    let descendant = output(&batch, 3, false);
    assert_eq!(
        (child.size.width, child.location.x, descendant.size.width),
        (n(80.0), n(10.0), n(80.0))
    );
}

#[test]
fn positioned_block_minimum_sizes_descendants() {
    assert_positioned_block_minimum_sizes_descendants::<f32>();
    assert_positioned_block_minimum_sizes_descendants::<f64>();
}

fn assert_positioned_ratio_uses_logical_axis<S: LayoutScalar>() {
    let px = S::from_f64;
    let inset =
        |edge: Option<f64>| edge.map_or(LengthAutoOf::AUTO, |edge| LengthAutoOf::px(px(edge)));
    let mut differences = Vec::new();
    for display in [
        Display::Block,
        Display::Flex,
        Display::Grid,
        Display::GridLanes,
    ] {
        for (containing_mode, child_mode, inline, block, width, height) in [
            (
                WritingMode::HorizontalTb,
                WritingMode::HorizontalTb,
                [Some(20.0); 2],
                [Some(90.0); 2],
                60.0,
                30.0,
            ),
            (
                WritingMode::VerticalRl,
                WritingMode::VerticalRl,
                [Some(20.0); 2],
                [Some(90.0); 2],
                120.0,
                60.0,
            ),
            (
                WritingMode::VerticalRl,
                WritingMode::VerticalRl,
                [Some(90.0); 2],
                [Some(20.0); 2],
                0.0,
                0.0,
            ),
            (
                WritingMode::VerticalRl,
                WritingMode::VerticalRl,
                [Some(20.0), None],
                [Some(20.0); 2],
                60.0,
                30.0,
            ),
            (
                WritingMode::HorizontalTb,
                WritingMode::VerticalRl,
                [Some(20.0); 2],
                [Some(90.0); 2],
                120.0,
                60.0,
            ),
            (
                WritingMode::VerticalRl,
                WritingMode::HorizontalTb,
                [Some(20.0); 2],
                [Some(90.0); 2],
                60.0,
                30.0,
            ),
        ] {
            let flow = FlowAxes::new(child_mode, Direction::Ltr);
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    NodeInputOf {
                        display,
                        position: Position::Relative,
                        size: Size::splat(px(100.0)).map(PreferredSizeOf::px),
                        writing_mode: containing_mode,
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        position: Position::Absolute,
                        writing_mode: child_mode,
                        aspect_ratio: AspectRatioOf::new(px(2.0)),
                        inset: flow.physical_edges(LogicalEdgesOf::new(
                            inset(inline[0]),
                            inset(inline[1]),
                            inset(block[0]),
                            inset(block[1]),
                        )),
                        ..NodeInputOf::default()
                    },
                );
            let batch = compute_layout(&tree, 1, inset_container_request()).unwrap();
            let actual = output(&batch, 2, false).size;
            let expected = Size::new(px(width), px(height));
            if actual != expected {
                differences.push(format!(
                    "{display:?} {containing_mode:?}/{child_mode:?} inline {inline:?} block {block:?}: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn positioned_ratio_uses_logical_axis_f32() {
    assert_positioned_ratio_uses_logical_axis::<f32>();
}

#[test]
fn positioned_ratio_uses_logical_axis_f64() {
    assert_positioned_ratio_uses_logical_axis::<f64>();
}

fn assert_positioned_ratio_uses_selected_sizing_box<S: LayoutScalar>() {
    let px = S::from_f64;
    let mut differences = Vec::new();
    for display in [
        Display::Block,
        Display::Flex,
        Display::Grid,
        Display::GridLanes,
    ] {
        for (box_sizing, inline_inset, width, height) in [
            (BoxSizing::ContentBox, 20.0, 105.0, 60.0),
            (BoxSizing::BorderBox, 20.0, 120.0, 60.0),
            (BoxSizing::ContentBox, 50.0, 5.0, 10.0),
            (BoxSizing::BorderBox, 50.0, 20.0, 10.0),
        ] {
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(1, [2])
                .children(2, [])
                .style(
                    1,
                    NodeInputOf {
                        display,
                        position: Position::Relative,
                        size: Size::splat(px(100.0)).map(PreferredSizeOf::px),
                        writing_mode: WritingMode::VerticalRl,
                        ..NodeInputOf::default()
                    },
                )
                .style(
                    2,
                    NodeInputOf {
                        display: Display::Block,
                        position: Position::Absolute,
                        writing_mode: WritingMode::VerticalRl,
                        box_sizing,
                        aspect_ratio: AspectRatioOf::new(px(2.0)),
                        padding: Edges {
                            top: LengthOf::px(px(5.0)),
                            right: LengthOf::px(px(3.0)),
                            bottom: LengthOf::px(px(5.0)),
                            left: LengthOf::px(px(2.0)),
                        },
                        inset: Edges {
                            top: LengthAutoOf::px(px(inline_inset)),
                            right: LengthAutoOf::px(px(90.0)),
                            bottom: LengthAutoOf::px(px(inline_inset)),
                            left: LengthAutoOf::px(px(90.0)),
                        },
                        ..NodeInputOf::default()
                    },
                );
            let batch = compute_layout(&tree, 1, inset_container_request()).unwrap();
            let actual = output(&batch, 2, false).size;
            let expected = Size::new(px(width), px(height));
            if actual != expected {
                differences.push(format!(
                    "{display:?} {box_sizing:?} with {inline_inset}px insets: {actual:?}, expected {expected:?}"
                ));
            }
        }
    }
    assert!(differences.is_empty(), "{}", differences.join("\n"));
}

#[test]
fn positioned_ratio_uses_selected_sizing_box_f32() {
    assert_positioned_ratio_uses_selected_sizing_box::<f32>();
}

#[test]
fn positioned_ratio_uses_selected_sizing_box_f64() {
    assert_positioned_ratio_uses_selected_sizing_box::<f64>();
}

fn auto_fixed_grid<S: LayoutScalar>(mode: WritingMode) -> NodeInputOf<S> {
    let mut root = root_style(Direction::Ltr);
    root.display = Display::Grid;
    root.writing_mode = mode;
    root.grid_template_columns = vec![TrackComponentOf::px(n(30.0)), TrackComponentOf::px(n(40.0))];
    root.grid_template_rows = vec![TrackComponentOf::px(n(50.0)), TrackComponentOf::px(n(60.0))];
    root
}

fn auto_grid_tree<S: LayoutScalar>(root: NodeInputOf<S>) -> PublicLayoutTreeOf<S> {
    let mode = root.writing_mode;
    let child = |start| NodeInputOf {
        display: Display::Flex,
        writing_mode: mode,
        grid_column: GridPlacement::try_lines(start, start + 1).unwrap(),
        grid_row: GridPlacement::try_lines(start, start + 1).unwrap(),
        ..NodeInputOf::default()
    };
    PublicLayoutTreeOf::new()
        .children(0, [1, 2])
        .children(1, [])
        .children(2, [])
        .style(0, root)
        .style(1, child(1))
        .style(2, child(2))
}

fn auto_grid_request<S: LayoutScalar>(
    inline_offset: f64,
    block_offset: f64,
) -> LayoutRootRequestOf<S> {
    let size = Size::new(nn(800.0), nn(600.0));
    let frame = RootLocalPositionedContextOf::horizontal_auto_insets(
        size,
        Direction::Ltr,
        n(inline_offset),
        OrthogonalFallbackOf::new(
            size.map(Some),
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
    .unwrap()
    .with_hypothetical_block_offset(n(block_offset))
    .unwrap();
    LayoutRootRequestOf::positioned_root_local(frame).unwrap()
}

fn assert_root_local_auto_grid_geometry<S: LayoutScalar>() {
    for (mode, expected) in [
        (
            WritingMode::HorizontalTb,
            [
                [0.0, 0.0, 70.0, 110.0],
                [0.0, 0.0, 30.0, 50.0],
                [30.0, 50.0, 40.0, 60.0],
            ],
        ),
        (
            WritingMode::VerticalRl,
            [
                [0.0, 0.0, 110.0, 70.0],
                [60.0, 0.0, 50.0, 30.0],
                [0.0, 30.0, 60.0, 40.0],
            ],
        ),
        (
            WritingMode::VerticalLr,
            [
                [0.0, 0.0, 110.0, 70.0],
                [0.0, 0.0, 50.0, 30.0],
                [50.0, 30.0, 60.0, 40.0],
            ],
        ),
        (
            WritingMode::SidewaysRl,
            [
                [0.0, 0.0, 110.0, 70.0],
                [60.0, 0.0, 50.0, 30.0],
                [0.0, 30.0, 60.0, 40.0],
            ],
        ),
        (
            WritingMode::SidewaysLr,
            [
                [0.0, 0.0, 110.0, 70.0],
                [0.0, 40.0, 50.0, 30.0],
                [50.0, 0.0, 60.0, 40.0],
            ],
        ),
    ] {
        let tree = auto_grid_tree(auto_fixed_grid::<S>(mode));
        for (inline, block) in [(0.0, 0.0), (780.0, 585.0)] {
            let batch = compute_layout(&tree, 0, auto_grid_request(inline, block)).unwrap();
            for finalized in [false, true] {
                for (node, [x, y, width, height]) in [0, 1, 2].into_iter().zip(expected) {
                    let actual = output(&batch, node, finalized);
                    assert_eq!(
                        actual.location,
                        Point::new(n(x), n(y)),
                        "{mode:?}, node {node}"
                    );
                    assert_eq!(
                        actual.size,
                        Size::new(n(width), n(height)),
                        "{mode:?}, node {node}"
                    );
                }
            }
        }

        let mut root = auto_fixed_grid::<S>(mode);
        root.min_size.width =
            MinSizeOf::value(LengthPercentageOf::from_percent_fraction(n(0.25)).unwrap());
        root.max_size.height = MaxSizeOf::percent(n(0.0625));
        let tree = auto_grid_tree(root);
        let batch = compute_layout(&tree, 0, auto_grid_request(780.0, 585.0)).unwrap();
        for (finalized, height) in [(false, 37.5), (true, 38.0)] {
            let actual = output(&batch, 0, finalized);
            assert_eq!(actual.location, Point::ZERO, "{mode:?}");
            assert_eq!(actual.size, Size::new(n(200.0), n(height)), "{mode:?}");
        }
    }
}

#[test]
fn root_local_auto_grid_uses_authored_orthogonal_tracks() {
    assert_root_local_auto_grid_geometry::<f64>();
}

#[test]
fn root_local_auto_grid_uses_authored_orthogonal_tracks_f32() {
    assert_root_local_auto_grid_geometry::<f32>();
}

#[test]
fn root_local_auto_grid_refuses_nested_counted_tracks() {
    let mut root = auto_fixed_grid::<f64>(WritingMode::VerticalRl);
    let inner =
        TrackRepetitionOf::count_components(1, std::mem::take(&mut root.grid_template_columns))
            .unwrap();
    let outer =
        TrackRepetitionOf::count_components(1, vec![TrackComponentOf::Repeat(inner)]).unwrap();
    root.grid_template_columns = vec![TrackComponentOf::Repeat(outer)];

    let error = compute_layout(&auto_grid_tree(root), 0, auto_grid_request(0.0, 0.0)).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(
            RootContextMismatch::WritingMode
        ))
    ));
}

#[test]
fn root_local_auto_grid_refuses_leaf_measurement() {
    let root = auto_fixed_grid::<f64>(WritingMode::VerticalRl);
    let tree = PublicLayoutTreeOf::new()
        .children(0, [])
        .style(0, root)
        .measure(0, Size::new(12.0, 10.0));

    let error = compute_layout(&tree, 0, auto_grid_request(0.0, 0.0)).unwrap_err();
    assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
    assert_eq!(error.operation(), LayoutOperation::RootLayout);
    assert!(matches!(
        error.kind(),
        LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(
            RootContextMismatch::WritingMode
        ))
    ));
}

#[test]
fn root_local_auto_grid_refuses_nonfixed_implicit_track() {
    let mut root = auto_fixed_grid::<f64>(WritingMode::HorizontalTb);
    root.grid_template_columns.truncate(1);
    let child = NodeInputOf {
        display: Display::Flex,
        size: Size::new(PreferredSizeOf::px(10.0), PreferredSizeOf::px(10.0)),
        grid_column: GridPlacement::try_lines(2, 3).unwrap(),
        grid_row: GridPlacement::try_lines(1, 2).unwrap(),
        ..NodeInputOf::default()
    };
    for auto_columns in [
        Vec::new(),
        vec![TrackComponentOf::flex(
            TrackFlexFactorOf::try_new(1.0).unwrap(),
        )],
    ] {
        root.grid_auto_columns = auto_columns;
        let tree = PublicLayoutTreeOf::new()
            .children(0, [1])
            .children(1, [])
            .style(0, root.clone())
            .style(1, child.clone());

        let result = compute_layout(&tree, 0, auto_grid_request(0.0, 0.0));
        let error = result.expect_err("the implicit column is not fixed");
        assert_eq!(error.site(), LayoutErrorSiteOf::Node(0));
        assert_eq!(error.operation(), LayoutOperation::RootLayout);
        assert!(matches!(
            error.kind(),
            LayoutErrorKindOf::InvalidInput(LayoutInvalidInputOf::RootContext(
                RootContextMismatch::WritingMode
            ))
        ));
    }
}
