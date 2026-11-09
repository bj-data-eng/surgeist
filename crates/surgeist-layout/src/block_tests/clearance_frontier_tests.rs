//! Actual public Block/Text frontiers for entry-clearance qualification.
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use crate::*;
use std::cell::Cell;
use surgeist_text as text;

fn block<S: LayoutScalar>(height: Option<f64>) -> NodeInputOf<S> {
    NodeInputOf {
        display: Display::Block,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(100.0)),
            height.map_or(PreferredSizeOf::AUTO, |value| {
                PreferredSizeOf::px(S::from_f64(value))
            }),
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

fn float<S: LayoutScalar>(side: Float, width: f64, height: f64) -> NodeInputOf<S> {
    NodeInputOf {
        float: side,
        size: Size::new(
            PreferredSizeOf::px(S::from_f64(width)),
            PreferredSizeOf::px(S::from_f64(height)),
        ),
        ..block(None)
    }
}

fn request<S: LayoutScalar>() -> LayoutRootRequestOf<S> {
    let width = NonNegativeFiniteOf::new(S::from_f64(100.0)).unwrap();
    crate::test_support::requests::NativeRootFrameOf::new(
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
    ))
}

fn output<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .node_layout(node)
        .unwrap()
        .single_box()
        .unwrap()
        .unrounded_output()
}

fn geometry<S: LayoutScalar>(
    batch: &CompletedLayoutBatchOf<u32, S>,
    node: u32,
    y: f64,
    height: f64,
) {
    let actual = output(batch, node);
    assert_eq!(actual.location.y, S::from_f64(y), "node {node} top");
    assert_eq!(
        actual.size.height,
        S::from_f64(height),
        "node {node} height"
    );
}

#[derive(Clone, Copy, Debug)]
enum NativeFrontier {
    Empty,
    ForcedZero,
    Original,
    OriginalZeroAdvance,
    OriginalNegativeAdvance,
    InFlowAtomicZero,
    OutOfFlowAtomicZero,
}

fn native_frontier<S: LayoutScalar>(kind: NativeFrontier) {
    let mut style = ahem_style(10.0, 10.0);
    style.wrap = text::Wrap::None;
    style.letter_spacing = match kind {
        NativeFrontier::OriginalZeroAdvance => -20.0,
        NativeFrontier::OriginalNegativeAdvance => -22.0,
        _ => 0.0,
    };
    let part = match kind {
        NativeFrontier::Empty => ParagraphPart::text(10, text::Source::new(""), style.clone()),
        NativeFrontier::Original => ParagraphPart::text(10, text::Source::new("X"), style.clone()),
        NativeFrontier::OriginalZeroAdvance | NativeFrontier::OriginalNegativeAdvance => {
            // Ahem AB has two10px glyphs and one actual internal tracking gap.
            ParagraphPart::text(10, text::Source::new("AB"), style.clone())
        }
        NativeFrontier::ForcedZero => ParagraphPart::Control {
            node: 10,
            kind: text::ParagraphControlKind::ForcedBreak,
        },
        NativeFrontier::InFlowAtomicZero => ParagraphPart::Atomic {
            node: 10,
            size: text::Size::new(0.0, 0.0),
        },
        NativeFrontier::OutOfFlowAtomicZero => ParagraphPart::OutOfFlow {
            node: 10,
            size: text::Size::new(0.0, 0.0),
        },
    };
    let paragraph = PreparedParagraph::with_strut_at_boundary(
        vec![part],
        style,
        text::ParagraphBaseDirection::LeftToRight,
        if matches!(kind, NativeFrontier::ForcedZero) {
            text::ParagraphStrut::AllocatedContributions
        } else {
            text::ParagraphStrut::DefaultFont
        },
        Vec::new(),
        text::ParagraphTerminalEdge::NonBlockBoundary,
    );
    let mut binding = paragraph.binding(2);
    let mut tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(2, [10, 3])
        .style(0, block(None))
        .style(1, float(Float::Left, 20.0, 5.0))
        .style(2, block(None))
        .style(
            3,
            NodeInputOf {
                clear: Clear::Left,
                margin: margins(10.0, 0.0),
                ..block(Some(10.0))
            },
        );
    tree = match kind {
        NativeFrontier::ForcedZero => {
            binding.line_strut = Some(text::LineStrut {
                baseline: 0.0,
                line_extent: 0.0,
            });
            tree.input(
                10,
                LayoutInputOf::line_break(
                    LineBreakInputOf::new().with_metrics(
                        InlineMetricsOf::from_ascent_descent(
                            InlineBaselineContext::Physical,
                            S::ZERO,
                            S::ZERO,
                        )
                        .unwrap(),
                    ),
                ),
            )
        }
        NativeFrontier::InFlowAtomicZero | NativeFrontier::OutOfFlowAtomicZero => tree.style(
            10,
            NodeInputOf {
                display: Display::InlineBlock,
                position: if matches!(kind, NativeFrontier::OutOfFlowAtomicZero) {
                    Position::Absolute
                } else {
                    Position::Relative
                },
                size: Size::new(PreferredSizeOf::px(S::ZERO), PreferredSizeOf::px(S::ZERO)),
                min_size: Size::new(MinSizeOf::px(S::ZERO), MinSizeOf::px(S::ZERO)),
                ..Default::default()
            },
        ),
        _ => tree.input(10, paragraph.text_input(10)),
    };
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding.clone()],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let batch = compute_layout_with_flow(&capture, 0, request()).unwrap();
    let [placed] = batch.placed_text() else {
        panic!("one actual run: {kind:?}")
    };
    assert_eq!(placed.nodes(), binding.nodes);
    let phantom = matches!(
        kind,
        NativeFrontier::Empty | NativeFrontier::OutOfFlowAtomicZero
    );
    assert_eq!(
        placed.layout().lines().len(),
        usize::from(!phantom),
        "{kind:?}"
    );
    if phantom {
        let completions = placed.layout().nonformatted_lines().collect::<Vec<_>>();
        assert_eq!(completions.len(), 1);
        let finalized = completions[0].line.finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        assert!(!finalized.has_in_flow_content());
    } else {
        let line = &placed.layout().lines()[0];
        let line_height = if matches!(kind, NativeFrontier::ForcedZero) {
            0.0
        } else {
            10.0
        };
        assert_eq!(line.bounds().size.height, line_height);
        if matches!(kind, NativeFrontier::InFlowAtomicZero) {
            // An atomic-only line has no Text-origin fragment. Its actual
            // positioned occurrence and normal box remain independently owned.
            assert!(batch.unrounded_inline_fragments().is_empty());
            let atomics = placed.layout().inline_boxes();
            assert_eq!(atomics.len(), 1);
            assert_eq!(atomics[0].kind(), text::InlineBoxKind::InFlow);
            assert_eq!(atomics[0].line(), 0);
            assert_eq!(atomics[0].rect().size, text::Size::new(0.0, 0.0));
            assert_eq!(
                atomics[0].id(),
                binding.atomics[0].occurrence.view().original_box().id()
            );
            assert_eq!(output(&batch, 10).size, Size::ZERO);
        } else {
            let finalized = batch
                .unrounded_inline_fragments()
                .iter()
                .find(|row| row.fragment().line_index() == 0)
                .unwrap()
                .fragment()
                .finalized_fragment()
                .finalized_line();
            assert_eq!(finalized.prepared_text(), paragraph.prepared);
            if matches!(
                kind,
                NativeFrontier::OriginalZeroAdvance | NativeFrontier::OriginalNegativeAdvance
            ) {
                assert!(finalized.has_in_flow_content());
                assert_eq!(
                    finalized.feedback().advance,
                    if matches!(kind, NativeFrontier::OriginalZeroAdvance) {
                        0.0
                    } else {
                        -2.0
                    }
                );
            }
        }
    }
    geometry(&batch, 1, 0.0, 5.0);
    // The global B top can agree for Empty and ForcedZero. Parent geometry is
    // decisive: authentic zero-height Forced separates; a phantom run adjoins.
    let (parent_y, parent_height, body_y) = if phantom {
        (10.0, 10.0, 0.0)
    } else if matches!(kind, NativeFrontier::ForcedZero) {
        (0.0, 20.0, 10.0)
    } else {
        (0.0, 30.0, 20.0)
    };
    geometry(&batch, 2, parent_y, parent_height);
    geometry(&batch, 3, body_y, 10.0);
    geometry(&batch, 0, 0.0, parent_y + parent_height);
}

fn native_case<S: LayoutScalar>(kind: NativeFrontier) {
    native_frontier::<S>(NativeFrontier::Original);
    native_frontier::<S>(kind);
}
#[test]
fn native_forced_zero_f32() {
    native_case::<f32>(NativeFrontier::ForcedZero);
}
#[test]
fn native_forced_zero_f64() {
    native_case::<f64>(NativeFrontier::ForcedZero);
}
#[test]
fn native_empty_f32() {
    native_case::<f32>(NativeFrontier::Empty);
}
#[test]
fn native_empty_f64() {
    native_case::<f64>(NativeFrontier::Empty);
}
#[test]
fn native_original_zero_advance_f32() {
    native_case::<f32>(NativeFrontier::OriginalZeroAdvance);
}
#[test]
fn native_original_zero_advance_f64() {
    native_case::<f64>(NativeFrontier::OriginalZeroAdvance);
}
#[test]
fn native_original_negative_advance_f32() {
    native_case::<f32>(NativeFrontier::OriginalNegativeAdvance);
}
#[test]
fn native_original_negative_advance_f64() {
    native_case::<f64>(NativeFrontier::OriginalNegativeAdvance);
}
#[test]
fn native_zero_in_flow_atomic_f32() {
    native_case::<f32>(NativeFrontier::InFlowAtomicZero);
}
#[test]
fn native_zero_in_flow_atomic_f64() {
    native_case::<f64>(NativeFrontier::InFlowAtomicZero);
}
#[test]
fn native_zero_out_of_flow_atomic_f32() {
    native_case::<f32>(NativeFrontier::OutOfFlowAtomicZero);
}
#[test]
fn native_zero_out_of_flow_atomic_f64() {
    native_case::<f64>(NativeFrontier::OutOfFlowAtomicZero);
}

fn processed_space_frontier<S: LayoutScalar>() {
    use crate::test_support::inline_endpoint::Fixture;
    for visible in [true, false] {
        let source = if visible { "X" } else { " " };
        let fixture = Fixture::new(source, (0, 1), text::WhiteSpace::Collapse, false, false);
        // Container0 remains the actual complete BlockEnd paragraph. E is an
        // ordinary child of P50, so no terminal receipt is retagged to place B.
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(100.0),
                false,
            )
            .style(0, block(None))
            .children(100, [1, 50])
            .children(50, [0, 51])
            .style(100, block(None))
            .style(1, float(Float::Left, 20.0, 5.0))
            .style(50, block(None))
            .style(
                51,
                NodeInputOf {
                    clear: Clear::Left,
                    margin: margins(10.0, 0.0),
                    ..block(Some(10.0))
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            100,
            vec![fixture.binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(&capture, 100, request()).unwrap();
        let [placed] = batch.placed_text() else {
            panic!("actual processed source")
        };
        assert_eq!(placed.nodes(), fixture.binding.nodes);
        assert_eq!(
            placed.layout().source().text().code_units(),
            source.encode_utf16().collect::<Vec<_>>()
        );
        assert_eq!(placed.layout().lines().len(), usize::from(visible));
        if visible {
            assert_eq!(placed.layout().lines()[0].bounds().size.height, 10.0);
        } else {
            let completions = placed.layout().nonformatted_lines().collect::<Vec<_>>();
            assert_eq!(completions.len(), 1);
            let line = completions[0].line.finalized_line();
            assert_eq!(line.prepared_text(), fixture.binding.prepared);
            assert!(!line.has_in_flow_content());
        }
        geometry(&batch, 1, 0.0, 5.0);
        geometry(&batch, 0, 0.0, if visible { 10.0 } else { 0.0 });
        geometry(
            &batch,
            50,
            if visible { 0.0 } else { 10.0 },
            if visible { 30.0 } else { 10.0 },
        );
        geometry(&batch, 51, if visible { 20.0 } else { 0.0 }, 10.0);
        geometry(&batch, 100, 0.0, if visible { 30.0 } else { 20.0 });
    }
}
#[test]
fn processed_space_native_frontier_f32() {
    processed_space_frontier::<f32>();
}
#[test]
fn processed_space_native_frontier_f64() {
    processed_space_frontier::<f64>();
}

fn text_in_float_bfc<S: LayoutScalar>() {
    let style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![ParagraphPart::text(
            10,
            text::Source::new("X"),
            style.clone(),
        )],
        style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    for contents in [false, true] {
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [1, 2])
            .children(1, if contents { vec![10] } else { Vec::new() })
            .children(2, [3])
            .style(0, block(None))
            .style(1, float(Float::Left, 20.0, 5.0))
            .style(2, block(None))
            .style(
                3,
                NodeInputOf {
                    clear: Clear::Left,
                    margin: margins(10.0, 0.0),
                    ..block(Some(10.0))
                },
            );
        let batch = if contents {
            tree = tree.input(10, paragraph.text_input(10));
            let capture = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![paragraph.binding(1)],
                WrappingInput::default(),
                Vec::new(),
            )
            .unwrap();
            compute_layout_with_flow(&capture, 0, request()).unwrap()
        } else {
            compute_layout(&tree, 0, request()).unwrap()
        };
        if contents {
            let [placed] = batch.placed_text() else {
                panic!("actual float-owned paragraph")
            };
            assert_eq!(placed.nodes(), [10]);
            assert_eq!(placed.layout().lines().len(), 1);
            assert_eq!(placed.layout().lines()[0].bounds().size.height, 10.0);
        }
        // Real Formatted content belongs to the float's independent BFC. Its
        //10px overflow does not become an in-flow separator inside sibling P.
        geometry(&batch, 1, 0.0, 5.0);
        geometry(&batch, 2, 10.0, 10.0);
        geometry(&batch, 3, 0.0, 10.0);
        geometry(&batch, 0, 0.0, 20.0);
    }
}
#[test]
fn formatted_float_bfc_preserves_sibling_entry_f32() {
    text_in_float_bfc::<f32>();
}
#[test]
fn formatted_float_bfc_preserves_sibling_entry_f64() {
    text_in_float_bfc::<f64>();
}

thread_local! { static MEASURED: Cell<usize> = const { Cell::new(0) }; }
fn measure<S: LayoutScalar>(_: LeafMeasureInputOf<S>, height: f64) -> Size<S> {
    MEASURED.with(|count| count.set(count.get() + 1));
    Size::new(S::from_f64(100.0), S::from_f64(height))
}
fn zero<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
    measure(input, 0.0)
}
fn two<S: LayoutScalar>(input: LeafMeasureInputOf<S>) -> Size<S> {
    measure(input, 2.0)
}
fn opaque<S: LayoutScalar>(float_height: f64, leaf_height: f64) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2])
        .children(2, [10, 3])
        .style(0, block(None))
        .style(1, float(Float::Left, 20.0, float_height))
        .style(2, block(None))
        .style(10, block(None))
        .style(
            3,
            NodeInputOf {
                clear: Clear::Left,
                margin: margins(10.0, 0.0),
                ..block(Some(10.0))
            },
        )
        .measure_with(
            10,
            if leaf_height == 0.0 {
                zero::<S>
            } else {
                two::<S>
            },
        );
    MEASURED.with(|count| count.set(0));
    let batch = compute_layout(&tree, 0, request()).unwrap();
    MEASURED.with(|count| assert!(count.get() > 0, "actual host measurement"));
    geometry(&batch, 1, 0.0, float_height);
    geometry(&batch, 10, 0.0, leaf_height);
    let (parent_y, parent_height, body_y) = if leaf_height == 0.0 && float_height == 5.0 {
        (10.0, 10.0, 0.0)
    } else if leaf_height == 2.0 {
        (0.0, 22.0, 12.0)
    } else {
        (0.0, 30.0, 20.0)
    };
    geometry(&batch, 2, parent_y, parent_height);
    geometry(&batch, 3, body_y, 10.0);
    geometry(&batch, 0, 0.0, parent_y + parent_height);
}
fn opaque_frontiers<S: LayoutScalar>() {
    opaque::<S>(5.0, 2.0);
    opaque::<S>(5.0, 0.0);
    opaque::<S>(20.0, 0.0);
}
#[test]
fn actual_opaque_frontiers_f32() {
    opaque_frontiers::<f32>();
}
#[test]
fn actual_opaque_frontiers_f64() {
    opaque_frontiers::<f64>();
}

fn resident_wrapper<S: LayoutScalar>(right_height: f64, clear: Clear) {
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1, 2, 3])
        .children(3, [4, 7])
        .children(7, [5])
        .style(0, block(None))
        .style(1, float(Float::Left, 50.0, 15.0))
        .style(2, float(Float::Right, 50.0, right_height))
        .style(3, block(None))
        .style(
            4,
            NodeInputOf {
                clear: Clear::Left,
                margin: margins(8.0, 12.0),
                ..block(None)
            },
        )
        .style(7, block(None))
        .style(
            5,
            NodeInputOf {
                clear,
                margin: margins(20.0, 0.0),
                ..block(Some(10.0))
            },
        );
    let batch = compute_layout(&tree, 0, request()).unwrap();
    geometry(&batch, 1, 0.0, 15.0);
    geometry(&batch, 2, 0.0, right_height);
    assert_eq!(output(&batch, 2).location.x, S::from_f64(50.0));
    if clear == Clear::None {
        geometry(&batch, 3, 20.0, 10.0);
        geometry(&batch, 4, 0.0, 0.0);
        geometry(&batch, 7, 0.0, 10.0);
        geometry(&batch, 5, 0.0, 10.0);
        geometry(&batch, 0, 0.0, right_height.max(30.0));
    } else {
        // The frozen earlier E decision leaves through-base7, group-end19.
        // R25: W's exported20 group adjoins, W27/H10/Blocal0.
        // R30: W starts19; B separates at local11, Wheight21.
        let (wrapper_y, wrapper_height, body_y, parent_height) = if right_height == 25.0 {
            (27.0, 10.0, 0.0, 37.0)
        } else {
            (19.0, 21.0, 11.0, 40.0)
        };
        geometry(&batch, 3, 0.0, parent_height);
        geometry(&batch, 4, 15.0, 0.0);
        geometry(&batch, 7, wrapper_y, wrapper_height);
        geometry(&batch, 5, body_y, 10.0);
        geometry(&batch, 0, 0.0, parent_height);
    }
}
fn wrapper_case<S: LayoutScalar>(height: f64) {
    resident_wrapper::<S>(height, Clear::None);
    resident_wrapper::<S>(height, Clear::Right);
}
#[test]
fn resident_wrapper_right25_f32() {
    wrapper_case::<f32>(25.0);
}
#[test]
fn resident_wrapper_right25_f64() {
    wrapper_case::<f64>(25.0);
}
#[test]
fn resident_wrapper_right30_f32() {
    wrapper_case::<f32>(30.0);
}
#[test]
fn resident_wrapper_right30_f64() {
    wrapper_case::<f64>(30.0);
}
