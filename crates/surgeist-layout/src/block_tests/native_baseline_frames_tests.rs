//! Native ComputeSize baselines belong to the returned physical border box.
use super::*;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::test_support::text::{ahem_style, in_flat_wrapping_context};
use surgeist_text as text;

const AHEM: &[u8] = include_bytes!("../../../surgeist-text/tests/fixtures/fonts/ahem/Ahem.ttf");

fn n<S: LayoutScalar>(value: f64) -> S {
    S::from_f64(value)
}
fn px<S: LayoutScalar>(value: f64) -> PreferredSizeOf<S> {
    PreferredSizeOf::px(n(value))
}
fn vertical(mode: WritingMode) -> bool {
    mode != WritingMode::HorizontalTb
}
fn name(mode: WritingMode) -> BaselineName {
    if matches!(mode, WritingMode::VerticalRl | WritingMode::VerticalLr) {
        BaselineName::Central
    } else {
        BaselineName::Alphabetic
    }
}
fn physical<S: LayoutScalar>(mode: WritingMode, cross: f64) -> Size<S> {
    if vertical(mode) {
        Size::new(n(cross), n(100.0))
    } else {
        Size::new(n(100.0), n(cross))
    }
}
fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap())
}
fn table(tag: [u8; 4]) -> Option<&'static [u8]> {
    (0..usize::from(u16_at(AHEM, 4))).find_map(|index| {
        let at = 12 + 16 * index;
        if AHEM[at..at + 4] != tag {
            return None;
        }
        let offset = usize::try_from(u32::from_be_bytes(
            AHEM[at + 8..at + 12].try_into().unwrap(),
        ))
        .unwrap();
        let length = usize::try_from(u32::from_be_bytes(
            AHEM[at + 12..at + 16].try_into().unwrap(),
        ))
        .unwrap();
        Some(&AHEM[offset..offset + length])
    })
}
fn font_premises() {
    assert_eq!(u16_at(table(*b"head").unwrap(), 18), 1000);
    assert_eq!(u16_at(table(*b"hmtx").unwrap(), 58 * 4), 1000);
    let hhea = table(*b"hhea").unwrap();
    let signed = |at| i16::from_be_bytes(hhea[at..at + 2].try_into().unwrap());
    assert_eq!((signed(4), signed(6), signed(8)), (800, -200, 0));
    for tag in [*b"BASE", *b"vhea", *b"vmtx"] {
        assert!(table(tag).is_none());
    }
    // Upright's actual absent-vertical-em policy centers one em at its nominal
    // pen. At size10/LH10 that is5/5; H and clockwise use hhea's8/2 instead.
}

struct Paragraph {
    prepared: text::PreparedText,
    original: text::Source,
}
impl Paragraph {
    fn new(raw: &str, mode: WritingMode) -> Self {
        Self::at_boundary(raw, mode, text::ParagraphTerminalEdge::BlockEnd)
    }
    fn at_boundary(
        raw: &str,
        mode: WritingMode,
        terminal_edge: text::ParagraphTerminalEdge,
    ) -> Self {
        font_premises();
        let original = text::Source::new(raw);
        let range = text::Range::new(0, original.text().len());
        let input = text::ParagraphInput {
            source: original.clone(),
            base_style: text::Style {
                white_space: text::WhiteSpace::Preserve,
                wrap: text::Wrap::None,
                ..ahem_style(10.0, 10.0)
            },
            paragraph_strut: text::ParagraphStrut::DefaultFont,
            terminal_edge,
            tab_reference: None,
            segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
            base_direction: text::ParagraphBaseDirection::LeftToRight,
            origins: vec![text::ParagraphOrigin {
                text_node: None,
                source: original.clone(),
                local: range,
                aggregate: range,
                kind: text::ParagraphOriginKind::Authored,
            }],
            connections: vec![],
            controls: vec![],
            replacements: vec![],
            boundary_policies: vec![],
            anchor_order: vec![],
        };
        let typography = if name(mode) == BaselineName::Central {
            text::AnalysisTypography::Upright {
                computed_direction: text::ParagraphBaseDirection::LeftToRight,
            }
        } else {
            text::AnalysisTypography::HorizontalComposition
        };
        let analyzed = in_flat_wrapping_context(
            text::AnalyzedSource::from_paragraph(input, typography).unwrap(),
            text::Wrap::None,
        );
        let mut system = text::System::new(text::SystemOptions {
            font_catalogue: text::FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let orientation = match mode {
            WritingMode::HorizontalTb => text::LineOrientation::Horizontal,
            WritingMode::SidewaysRl => text::LineOrientation::SidewaysClockwise,
            WritingMode::VerticalRl | WritingMode::VerticalLr => text::LineOrientation::Vertical,
            WritingMode::SidewaysLr => {
                panic!("this fixture uses only admitted native Text profiles")
            }
        };
        let mut options = text::PreparationOptions::default().with_orientation(orientation);
        if name(mode) == BaselineName::Alphabetic {
            options = options.with_baseline_reference(text::FontBaseline::Alphabetic);
        }
        Self {
            prepared: system.prepare(&analyzed, options).unwrap(),
            original,
        }
    }
    fn binding(&self) -> TextParagraphBinding<u32> {
        let origin = self.prepared.origin_views().next().unwrap();
        assert_eq!(origin.source(), &self.original);
        assert_eq!(
            origin.local_range(),
            text::Range::new(0, self.original.text().len())
        );
        TextParagraphBinding {
            inline_box_fragments: crate::InlineBoxFragmentationDemand::NotRequested,
            prepared: self.prepared.clone(),
            container: 0,
            nodes: vec![1],
            origins: vec![TextOriginBinding {
                node: 1,
                origin: origin.handle(),
            }],
            atomics: vec![],
            controls: vec![],
            line_strut: None,
        }
    }
}

fn style<S: LayoutScalar>(mode: WritingMode) -> NodeInputOf<S> {
    NodeInputOf::<S> {
        display: Display::Block,
        writing_mode: mode,
        inline_baseline_context: InlineBaselineContext::Named(name(mode)),
        size: if vertical(mode) {
            Size::new(PreferredSizeOf::AUTO, px(100.0))
        } else {
            Size::new(px(100.0), PreferredSizeOf::AUTO)
        },
        ..NodeInputOf::default()
    }
}

fn measured<S: LayoutScalar>(
    paragraph: &Paragraph,
    mode: WritingMode,
    root: NodeInputOf<S>,
    known_cross: Option<f64>,
    demand: bool,
) -> ComputeOutputOf<S> {
    let binding = paragraph.binding();
    let tree = PublicLayoutTreeOf::<S>::new()
        .children(0, [1])
        .children(1, [])
        .style(0, root)
        .input(
            1,
            LayoutInputOf::inline_text(TextParticipation::new(binding.origins[0].origin.clone())),
        );
    let capture = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![binding],
        WrappingInput::default(),
        Vec::new(),
    )
    .unwrap();
    let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    let known = if vertical(mode) {
        Size::new(known_cross.map(n), None)
    } else {
        Size::new(None, known_cross.map(n))
    };
    let available = if vertical(mode) {
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::definite(n(100.0)))
    } else {
        Size::new(AvailableOf::definite(n(100.0)), AvailableOf::MAX_CONTENT)
    };
    let mut request = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        known,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(mode, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        available,
    );
    if demand {
        request = request.with_named_baseline_demand(crate::output::NamedBaselineDemand::Fact {
            name: name(mode),
            set: crate::output::NamedBaselineSet::First,
        });
    }
    let output = crate::compute_block(&mut owner, 0, request).unwrap();
    // Measurement returns physical facts without staging either the box or
    // its real Text node for publication. Use the existing output read front.
    assert!(owner.unrounded(0).is_err());
    assert!(owner.unrounded(1).is_err());
    let repeat = crate::compute_block(&mut owner, 0, request).unwrap();
    assert_eq!(repeat, output);
    assert!(owner.unrounded(0).is_err());
    assert!(owner.unrounded(1).is_err());
    output
}

fn assert_facts<S: LayoutScalar>(
    output: ComputeOutputOf<S>,
    mode: WritingMode,
    cross: f64,
    first: f64,
    last: f64,
) {
    assert_eq!(output.size, physical(mode, cross));
    let baselines = output.baselines();
    let (axis, first_point, last_point) = if vertical(mode) {
        (
            PhysicalAxis::Horizontal,
            Point::new(Some(n(first)), None),
            Point::new(Some(n(last)), None),
        )
    } else {
        (
            PhysicalAxis::Vertical,
            Point::new(None, Some(n(first))),
            Point::new(None, Some(n(last))),
        )
    };
    assert_eq!(
        baselines.first, first_point,
        "generic facts must be local to the returned border box"
    );
    assert_eq!(baselines.last, last_point);
    let pair = output.named_baselines.get(name(mode));
    assert_eq!(
        pair.first(),
        BaselineFactOf::Available(BaselineCoordinateOf::try_new(axis, n(first)).unwrap())
    );
    assert_eq!(
        pair.last(),
        BaselineFactOf::Available(BaselineCoordinateOf::try_new(axis, n(last)).unwrap())
    );
}

fn auto<S: LayoutScalar>(demand: bool) {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    let output = measured(&paragraph, mode, style(mode), None, demand);
    // No block-axis size was supplied. Its provisional width0 yields an
    // accepted Central-5; completing the actual width10 re-expresses it as5.
    assert_facts::<S>(output, mode, 10.0, 5.0, 5.0);
}
#[test]
fn auto_width_vertical_rl_native_measurement_returns_box_local_baselines_f32() {
    auto::<f32>(false);
}
#[test]
fn auto_width_vertical_rl_native_measurement_returns_box_local_baselines_f64() {
    auto::<f64>(false);
}
#[test]
fn actual_named_demand_retains_the_same_box_local_native_measurement() {
    auto::<f32>(true);
    auto::<f64>(true);
}

fn constrained<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    for minimum in [20.0, 20.25] {
        let mut root = style(mode);
        root.min_size.width = MinSizeOf::px(n(minimum));
        let output = measured(&paragraph, mode, root, None, false);
        // Retain the same accepted anchor-5 and add final extent, not a new
        // midpoint and not another integer snap: 20.25-5 is exactly15.25.
        assert_facts::<S>(output, mode, minimum, minimum - 5.0, minimum - 5.0);
    }
    let known = measured(&paragraph, mode, style(mode), Some(20.0), false);
    assert_facts::<S>(known, mode, 20.0, 15.0, 15.0);
}
#[test]
fn min_width_and_fractional_completion_preserve_the_accepted_anchor_with_known_size_neighbor() {
    constrained::<f32>();
    constrained::<f64>();
}

fn unchanged_axes<S: LayoutScalar>() {
    for (mode, cross, baseline, known) in [
        (WritingMode::HorizontalTb, 10.0, 8.0, None),
        (WritingMode::HorizontalTb, 20.0, 8.0, Some(20.0)),
        (WritingMode::VerticalLr, 10.0, 5.0, None),
        (WritingMode::VerticalLr, 20.0, 5.0, Some(20.0)),
        (WritingMode::SidewaysRl, 20.0, 12.0, Some(20.0)),
    ] {
        let paragraph = Paragraph::new("X", mode);
        assert_facts::<S>(
            measured(&paragraph, mode, style(mode), known, false),
            mode,
            cross,
            baseline,
            baseline,
        );
    }
}
#[test]
fn horizontal_and_vertical_lr_start_side_facts_and_known_clockwise_reference_remain_physical() {
    unchanged_axes::<f32>();
    unchanged_axes::<f64>();
}

fn ordered_and_padded<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let lines = Paragraph::new("X\nX", mode);
    // Two genuine preserved source lines consume20. Source-first is the
    // right-hand line at15 and source-last the left-hand line at5.
    assert_facts::<S>(
        measured(&lines, mode, style(mode), None, false),
        mode,
        20.0,
        15.0,
        5.0,
    );
    let one = Paragraph::new("X", mode);
    let mut padded = style(mode);
    padded.padding.left = LengthOf::px(n(3.0));
    padded.padding.right = LengthOf::px(n(2.0));
    // Final width15 includes the two real padding sides. Its accepted line
    // reference is five units inside the padded right edge:15-2-5=8.
    assert_facts::<S>(
        measured(&one, mode, padded, None, false),
        mode,
        15.0,
        8.0,
        8.0,
    );
}
#[test]
fn source_ordered_lines_and_actual_padding_resolve_in_the_returned_border_box() {
    ordered_and_padded::<f32>();
    ordered_and_padded::<f64>();
}

use std::cell::RefCell;

fn binding_at(paragraph: &Paragraph, container: u32, node: u32) -> TextParagraphBinding<u32> {
    let mut binding = paragraph.binding();
    binding.container = container;
    binding.nodes = vec![node];
    binding.origins[0].node = node;
    binding
}

fn scene_request<S: LayoutScalar>(known: Size<Option<S>>, inline: f64) -> ComputeInputOf<S> {
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        known,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::VerticalRl, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::definite(n(inline))),
    )
}

fn scene_measure<Tree: LayoutTree<Node = u32>>(
    tree: &Tree,
    bindings: Vec<TextParagraphBinding<u32>>,
    nodes: &[u32],
) -> ComputeOutputOf<Tree::Scalar>
where
    Tree::MeasureError: core::fmt::Debug,
{
    let capture =
        LayoutFlowInput::try_new(tree, 0, bindings, WrappingInput::default(), Vec::new()).unwrap();
    let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
    let output = owner
        .compute_child(0, scene_request(Size::NONE, 100.0))
        .unwrap();
    for &node in nodes {
        assert!(owner.unrounded(node).is_err());
    }
    output
}

fn central<S: LayoutScalar>(value: f64) -> BaselineFactOf<S> {
    BaselineFactOf::Available(
        BaselineCoordinateOf::try_new(PhysicalAxis::Horizontal, n(value)).unwrap(),
    )
}

fn mixed_numeric_and_native<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    for numeric_first in [true, false] {
        // A following ordinary sibling means this preparation does not reach
        // its containing block end. Only the final Native run owns BlockEnd.
        let terminal = if numeric_first {
            text::ParagraphTerminalEdge::BlockEnd
        } else {
            text::ParagraphTerminalEdge::NonBlockBoundary
        };
        let paragraph = Paragraph::at_boundary("X", mode, terminal);
        let binding = paragraph.binding();
        for (minimum, width) in [(None, 14.0), (Some(20.0), 20.0)] {
            let mut root = style(mode);
            if let Some(minimum) = minimum {
                root.min_size.width = MinSizeOf::px(n(minimum));
            }
            let mut ordinary = style(mode);
            ordinary.size.width = px(4.0);
            // This ordinary child really lays out its normalized line break.
            // Its declared Central2/extent4 metrics qualify its numeric line;
            // no Available baseline output is supplied by this fixture.
            let metrics = InlineMetricsOf::try_new(
                InlineBaselineContext::Named(BaselineName::Central),
                n(2.0),
                n(4.0),
            )
            .unwrap();
            let tree = PublicLayoutTreeOf::<S>::new()
                .children(0, if numeric_first { [2, 1] } else { [1, 2] })
                .children(1, [])
                .children(2, [3])
                .children(3, [])
                .style(0, root)
                .style(2, ordinary)
                .input(
                    1,
                    LayoutInputOf::inline_text(TextParticipation::new(
                        binding.origins[0].origin.clone(),
                    )),
                )
                .input(
                    3,
                    LayoutInputOf::line_break(
                        LineBreakInputOf::new()
                            .with_writing_mode(mode)
                            .with_metrics(metrics),
                    ),
                );
            let output = scene_measure(&tree, vec![binding.clone()], &[0, 1, 2, 3]);
            // Source-first numeric: x=F-4+2; following Native: x=F-4-5.
            // Source-first Native: x=F-5; following numeric: x=F-10-4+2.
            let (first, last) = if numeric_first {
                (width - 2.0, width - 9.0)
            } else {
                (width - 5.0, width - 12.0)
            };
            assert_facts::<S>(output, mode, width, first, last);
        }
    }
}
#[test]
fn mixed_real_numeric_child_and_native_line_keep_source_order_through_final_extent() {
    mixed_numeric_and_native::<f32>();
    mixed_numeric_and_native::<f64>();
}

fn unknown_and_absent<S: LayoutScalar>() {
    use super::fixtures::PublicBlockTree;
    let mode = WritingMode::VerticalRl;
    for opaque_leaf in [false, true] {
        for native_first in [false, true] {
            let terminal = if native_first {
                text::ParagraphTerminalEdge::NonBlockBoundary
            } else {
                text::ParagraphTerminalEdge::BlockEnd
            };
            let paragraph = Paragraph::at_boundary("X", mode, terminal);
            let binding = paragraph.binding();
            let mut child = style(mode);
            if !opaque_leaf {
                child.size.width = px(4.0);
            }
            let mut tree = PublicBlockTree::<S>::default()
                .with_children(0, if native_first { [1, 2] } else { [2, 1] })
                .with_children(1, [])
                .with_children(2, [])
                .with_style(0, style(mode))
                .with_style(1, NodeInputOf::non_box())
                .with_style(2, child)
                .with_layout_input(
                    1,
                    LayoutInputOf::inline_text(TextParticipation::new(
                        binding.origins[0].origin.clone(),
                    )),
                );
            if opaque_leaf {
                // The actual public host leaf supplies size only. Its used
                // width is genuinely measured, not a supplied baseline fact.
                tree = tree.with_measurement(2, Size::new(n(4.0), n(100.0)));
            }
            let capture = LayoutFlowInput::try_new(
                &tree,
                0,
                vec![binding.clone()],
                WrappingInput::default(),
                Vec::new(),
            )
            .unwrap();
            let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
            let request = scene_request(Size::NONE, 100.0);
            let child_output = owner.compute_child(2, request).unwrap();
            assert_eq!(child_output.size, Size::new(n(4.0), n(100.0)));
            assert_eq!(child_output.baselines(), BaselinesOf::NONE);
            let own = child_output.named_baselines.get(BaselineName::Central);
            let actual_kind = if opaque_leaf {
                BaselineFactOf::Unknown
            } else {
                BaselineFactOf::Absent
            };
            assert_eq!((own.first(), own.last()), (actual_kind, actual_kind));
            let output = owner.compute_child(0, request).unwrap();
            assert_eq!(output.size, physical(mode, 14.0));
            let native = if native_first { 9.0 } else { 5.0 };
            assert_eq!(output.first_baselines, Point::new(Some(n(native)), None));
            assert_eq!(output.last_baselines, Point::new(Some(n(native)), None));
            let pair = output.named_baselines.get(BaselineName::Central);
            let first = if opaque_leaf && !native_first {
                BaselineFactOf::Unknown
            } else {
                central(native)
            };
            let last = if opaque_leaf && native_first {
                BaselineFactOf::Unknown
            } else {
                central(native)
            };
            assert_eq!((pair.first(), pair.last()), (first, last));
            for node in [0, 1, 2] {
                assert!(owner.unrounded(node).is_err());
            }
        }
    }
}
#[test]
fn real_size_only_leaf_unknown_is_a_source_barrier_while_empty_block_absence_is_skipped() {
    unknown_and_absent::<f32>();
    unknown_and_absent::<f64>();
}

fn nested<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    let binding = binding_at(&paragraph, 2, 11);
    for (minimum, width, baseline) in [(None, 25.0, 17.0), (Some(35.0), 35.0, 27.0)] {
        let mut root = style(mode);
        root.padding.left = LengthOf::px(n(2.0));
        root.padding.right = LengthOf::px(n(3.0));
        if let Some(minimum) = minimum {
            root.min_size.width = MinSizeOf::px(n(minimum));
        }
        let mut child = style(mode);
        child.min_size.width = MinSizeOf::px(n(20.0));
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [2])
            .children(2, [11])
            .children(11, [])
            .style(0, root)
            .style(2, child)
            .input(
                11,
                LayoutInputOf::inline_text(TextParticipation::new(
                    binding.origins[0].origin.clone(),
                )),
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
        let request = scene_request(Size::NONE, 100.0);
        // Resolve the actual child in its own20px box first. Inheritance then
        // applies only its parent placement: F-right3-child20+local15=F-8.
        let child_output = owner.compute_child(2, request).unwrap();
        assert_facts::<S>(child_output, mode, 20.0, 15.0, 15.0);
        let output = owner.compute_child(0, request).unwrap();
        assert_facts::<S>(output, mode, width, baseline, baseline);
        for node in [0, 2, 11] {
            assert!(owner.unrounded(node).is_err());
        }
    }
}
#[test]
fn child_local_native_fact_is_resolved_before_exactly_one_parent_placement() {
    nested::<f32>();
    nested::<f64>();
}

// A faithful host cache stores only outputs actually obtained from the owning
// computation. These normalized ParagraphInput fixtures have nonempty Flow
// bindings and no captured semantic slots, scoped wrapping/metric effects or
// formatted-tail demands. The existing with_flow owner therefore admits host
// lookup for this profile. Semantic capture ancestors may intentionally bypass
// it; they are not represented by these cache-hit cases. Public lookup results
// distinguish actual host hits from repeat equality.
struct MemoTree<S: LayoutScalar> {
    tree: PublicLayoutTreeOf<S>,
    cache: RefCell<CacheOf<S>>,
    lookups: RefCell<Vec<(ComputeInputOf<S>, bool)>>,
}
impl<S: LayoutScalar> Traverse for MemoTree<S> {
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
impl<S: LayoutScalar> LayoutTree for MemoTree<S> {
    type MeasureError = core::convert::Infallible;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.tree.node_input(node)
    }
    fn layout_input(&self, node: u32) -> LayoutInputViewOf<'_, S> {
        self.tree.layout_input(node)
    }
    fn cache_get(
        &self,
        node: u32,
        input: &ComputeInputOf<S>,
        context: CacheKeyContext,
    ) -> Option<ComputeOutputOf<S>> {
        if node != 0 {
            return None;
        }
        let output = self.cache.borrow().get_with_context(input, context);
        self.lookups.borrow_mut().push((*input, output.is_some()));
        output
    }
}
impl<S: LayoutScalar> MemoTree<S> {
    fn new(paragraph: &Paragraph, root: NodeInputOf<S>) -> Self {
        assert!(paragraph.prepared.captured_owner().is_none());
        let binding = paragraph.binding();
        Self {
            tree: PublicLayoutTreeOf::new()
                .children(0, [1])
                .children(1, [])
                .style(0, root)
                .input(
                    1,
                    LayoutInputOf::inline_text(TextParticipation::new(
                        binding.origins[0].origin.clone(),
                    )),
                ),
            cache: RefCell::new(CacheOf::new()),
            lookups: RefCell::new(vec![]),
        }
    }
    fn compute(
        &self,
        paragraph: &Paragraph,
        request: ComputeInputOf<S>,
        hit: bool,
    ) -> ComputeOutputOf<S> {
        let capture = LayoutFlowInput::try_new(
            self,
            0,
            vec![paragraph.binding()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
        self.lookups.borrow_mut().clear();
        let output = owner.compute_child(0, request).unwrap();
        let observed = self.lookups.borrow();
        assert!(
            !observed.is_empty(),
            "this plain Flow profile must reach its admitted host lookup"
        );
        assert!(
            observed
                .iter()
                .all(|(actual, found)| *actual == request && *found == hit)
        );
        assert!(owner.unrounded(0).is_err());
        assert!(owner.unrounded(1).is_err());
        output
    }
    fn retain(&self, request: ComputeInputOf<S>, output: ComputeOutputOf<S>) {
        self.cache
            .borrow_mut()
            .store_with_context(&request, CacheKeyContext::new(), output);
    }
}

fn cache_frames<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    let tree = MemoTree::<S>::new(&paragraph, style(mode));
    let auto = scene_request(Size::NONE, 100.0);
    let cold = tree.compute(&paragraph, auto, false);
    assert_facts::<S>(cold, mode, 10.0, 5.0, 5.0);
    tree.retain(auto, cold);
    let warm = tree.compute(&paragraph, auto, true);
    assert_facts::<S>(warm, mode, 10.0, 5.0, 5.0);
    // The existing cache explicitly permits known==cached returned size.
    // This must reuse a resolved local5, never a provisional-frame value-5.
    let equal_size = scene_request(Size::new(Some(n(10.0)), None), 100.0);
    assert_facts::<S>(
        tree.compute(&paragraph, equal_size, true),
        mode,
        10.0,
        5.0,
        5.0,
    );
    let wider = scene_request(Size::new(Some(n(20.0)), None), 100.0);
    assert_facts::<S>(
        tree.compute(&paragraph, wider, false),
        mode,
        20.0,
        15.0,
        15.0,
    );
    let demand = auto.with_named_baseline_demand(crate::output::NamedBaselineDemand::Fact {
        name: BaselineName::Central,
        set: crate::output::NamedBaselineSet::First,
    });
    let refined = tree.compute(&paragraph, demand, false);
    assert_facts::<S>(refined, mode, 10.0, 5.0, 5.0);
    tree.retain(demand, refined);
    assert_facts::<S>(tree.compute(&paragraph, demand, true), mode, 10.0, 5.0, 5.0);
    assert!(
        tree.cache
            .borrow()
            .get_with_context(
                &auto.with_run_mode(RunMode::PerformLayout),
                CacheKeyContext::new()
            )
            .is_none()
    );
}
#[test]
fn actual_native_measurement_cache_retains_local_facts_and_separates_size_demand_and_publication() {
    cache_frames::<f32>();
    cache_frames::<f64>();
}

fn wrapping_paragraph() -> Paragraph {
    font_premises();
    let original = text::Source::new("X X");
    let range = text::Range::new(0, 3);
    let input = text::ParagraphInput {
        source: original.clone(),
        base_style: text::Style {
            white_space: text::WhiteSpace::Collapse,
            wrap: text::Wrap::Word,
            ..ahem_style(10.0, 10.0)
        },
        paragraph_strut: text::ParagraphStrut::DefaultFont,
        terminal_edge: text::ParagraphTerminalEdge::BlockEnd,
        tab_reference: None,
        segment_breaks: text::DocumentSegmentBreaks::LineFeedDefault,
        base_direction: text::ParagraphBaseDirection::LeftToRight,
        origins: vec![text::ParagraphOrigin {
            text_node: None,
            source: original.clone(),
            local: range,
            aggregate: range,
            kind: text::ParagraphOriginKind::Authored,
        }],
        connections: vec![],
        controls: vec![],
        replacements: vec![],
        boundary_policies: vec![],
        anchor_order: vec![],
    };
    let analyzed = in_flat_wrapping_context(
        text::AnalyzedSource::from_paragraph(
            input,
            text::AnalysisTypography::Upright {
                computed_direction: text::ParagraphBaseDirection::LeftToRight,
            },
        )
        .unwrap(),
        text::Wrap::Word,
    );
    let mut system = text::System::new(text::SystemOptions {
        font_catalogue: text::FontCatalogueMode::RegisteredOnly,
    })
    .unwrap();
    system.register_fonts(AHEM.to_vec()).unwrap();
    Paragraph {
        prepared: system
            .prepare(
                &analyzed,
                text::PreparationOptions::default()
                    .with_orientation(text::LineOrientation::Vertical),
            )
            .unwrap(),
        original,
    }
}

fn reflow<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = wrapping_paragraph();
    let mut root = style(mode);
    root.size.height = PreferredSizeOf::AUTO;
    let tree = MemoTree::<S>::new(&paragraph, root);
    let wide = scene_request(Size::new(None, Some(n(30.0))), 30.0);
    let narrow = scene_request(Size::new(None, Some(n(10.0))), 10.0);
    let assert_output = |output: ComputeOutputOf<S>, inline, width, first, last| {
        assert_eq!(output.size, Size::new(n(width), n(inline)));
        assert_eq!(output.first_baselines, Point::new(Some(n(first)), None));
        assert_eq!(output.last_baselines, Point::new(Some(n(last)), None));
        let pair = output.named_baselines.get(BaselineName::Central);
        assert_eq!((pair.first(), pair.last()), (central(first), central(last)));
    };
    // Ahem's X/space/X occupies30 along the inline axis. At30 it is one
    // genuine10px column; at10 the actual word boundary produces two columns,
    // with the collapsed trailing separator excluded from the next line.
    let first = tree.compute(&paragraph, wide, false);
    assert_output(first, 30.0, 10.0, 5.0, 5.0);
    tree.retain(wide, first);
    assert_output(tree.compute(&paragraph, wide, true), 30.0, 10.0, 5.0, 5.0);
    let changed = tree.compute(&paragraph, narrow, false);
    assert_output(changed, 10.0, 20.0, 15.0, 5.0);
    tree.retain(narrow, changed);
    assert_output(
        tree.compute(&paragraph, narrow, true),
        10.0,
        20.0,
        15.0,
        5.0,
    );
    assert_output(tree.compute(&paragraph, wide, false), 30.0, 10.0, 5.0, 5.0);
}
#[test]
fn changed_real_inline_constraint_reselects_native_lines_instead_of_reusing_old_affine_evidence() {
    reflow::<f32>();
    reflow::<f64>();
}

fn genuine_refinement<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    let binding = paragraph.binding();
    let mut child = style(mode);
    child.size.width = px(4.0);
    let tree = MemoTree::<S> {
        tree: PublicLayoutTreeOf::new()
            .children(0, [2, 1])
            .children(1, [])
            .children(2, [3])
            .children(3, [])
            .style(0, style(mode))
            .style(2, child)
            .style(
                3,
                NodeInputOf::<S> {
                    display: Display::Block,
                    position: Position::Absolute,
                    writing_mode: mode,
                    size: Size::new(px(0.0), px(0.0)),
                    ..NodeInputOf::default()
                },
            )
            .input(
                1,
                LayoutInputOf::inline_text(TextParticipation::new(
                    binding.origins[0].origin.clone(),
                )),
            ),
        cache: RefCell::new(CacheOf::new()),
        lookups: RefCell::new(vec![]),
    };
    let plain = scene_request(Size::NONE, 100.0);
    let demanded = plain.with_named_baseline_demand(crate::output::NamedBaselineDemand::Fact {
        name: BaselineName::Central,
        set: crate::output::NamedBaselineSet::First,
    });
    {
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
        // The existing definite-size front may skip baseline inspection when
        // its only child is out of flow. A genuine named demand inspects the
        // actual flow and proves absence without changing the returned size.
        let unknown = owner.compute_child(2, plain).unwrap();
        let clarified = owner.compute_child(2, demanded).unwrap();
        assert_eq!(unknown.size, Size::new(n(4.0), n(100.0)));
        assert_eq!(clarified.size, unknown.size);
        let before = unknown.named_baselines.get(BaselineName::Central);
        let after = clarified.named_baselines.get(BaselineName::Central);
        assert_eq!(
            (before.first(), before.last()),
            (BaselineFactOf::Unknown, BaselineFactOf::Unknown)
        );
        assert_eq!(
            (after.first(), after.last()),
            (BaselineFactOf::Absent, BaselineFactOf::Absent)
        );
        for node in [0, 1, 2, 3] {
            assert!(owner.unrounded(node).is_err());
        }
    }
    let unresolved = tree.compute(&paragraph, plain, false);
    assert_eq!(unresolved.size, physical(mode, 14.0));
    let pair = unresolved.named_baselines.get(BaselineName::Central);
    assert_eq!(
        (pair.first(), pair.last()),
        (BaselineFactOf::Unknown, central(5.0))
    );
    tree.retain(plain, unresolved);
    // The named-demand key must miss the real cached Unknown barrier. Once
    // the earlier child is genuinely Absent, the following Native5 is first.
    let resolved = tree.compute(&paragraph, demanded, false);
    assert_facts::<S>(resolved, mode, 14.0, 5.0, 5.0);
    tree.retain(demanded, resolved);
    assert_facts::<S>(
        tree.compute(&paragraph, demanded, true),
        mode,
        14.0,
        5.0,
        5.0,
    );
}
#[test]
fn real_named_demand_refines_dimension_only_unknown_to_absence_before_following_native_fact() {
    genuine_refinement::<f32>();
    genuine_refinement::<f64>();
}

fn real_border_and_padding<S: LayoutScalar>() {
    let mode = WritingMode::VerticalRl;
    let paragraph = Paragraph::new("X", mode);
    for (minimum, width, baseline) in [(None, 18.0, 9.0), (Some(20.0), 20.0, 11.0)] {
        let mut root = style(mode);
        root.padding.left = LengthOf::px(n(3.0));
        root.padding.right = LengthOf::px(n(2.0));
        root.border.left = LengthOf::px(n(1.0));
        root.border.right = LengthOf::px(n(2.0));
        if let Some(minimum) = minimum {
            root.min_size.width = MinSizeOf::px(n(minimum));
        }
        // Used left/right insets are4/4, so the natural border width is18.
        // The actual right-affine line reference is F-right4-over5=F-9.
        let output = measured(&paragraph, mode, root, None, false);
        assert_facts::<S>(output, mode, width, baseline, baseline);
    }
}
#[test]
fn actual_border_and_padding_edges_rebase_the_retained_native_reference_in_the_final_box() {
    real_border_and_padding::<f32>();
    real_border_and_padding::<f64>();
}
