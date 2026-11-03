//! Functional qualification of explicit root-border-local positioned requests.
//! All expected widths come from the native/real Ahem MIN30/MAX70 content,
//! genuine CB percentages, and continuous local corner rounding.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
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
        batch.final_entries()
    } else {
        batch.unrounded_entries()
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
    // The source/font-backed content supplies genuine MIN30/MAX70 before the
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
