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
            WritingMode::SidewaysRl => text::LineOrientation::Sideways,
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
    let capture =
        LayoutFlowInput::try_new(&tree, 0, vec![binding], WrappingInput::default()).unwrap();
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
