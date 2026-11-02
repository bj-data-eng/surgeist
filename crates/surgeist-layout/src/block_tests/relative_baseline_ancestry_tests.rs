//! Private child of block_tests::native_baseline_frames_tests.
//! Reuses that owner's real Ahem Paragraph, binding_at, style, and font premises.
//! Existing preservation cohorts remain in their owning files; see mount note.
use super::*;

fn relative_block_start<S: LayoutScalar>(mode: WritingMode, value: f64) -> Edges<LengthAutoOf<S>> {
    let mut inset = Edges::all(LengthAutoOf::AUTO);
    let value = LengthAutoOf::px(n::<S>(value));
    match mode {
        WritingMode::HorizontalTb => inset.top = value,
        WritingMode::VerticalRl => inset.right = value,
        WritingMode::VerticalLr => inset.left = value,
        _ => panic!("this cohort uses the admitted H/Vrl/Vlr native profiles"),
    }
    inset
}

fn normal_request<S: LayoutScalar>(mode: WritingMode) -> ComputeInputOf<S> {
    let available = if vertical(mode) {
        Size::new(AvailableOf::MAX_CONTENT, AvailableOf::definite(n(100.0)))
    } else {
        Size::new(AvailableOf::definite(n(100.0)), AvailableOf::MAX_CONTENT)
    };
    ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::InherentSize,
        RequestedAxis::Both,
        Size::NONE,
        Size::NONE,
        ContainingLayoutContext::new(
            FlowAxes::new(mode, Direction::Ltr),
            ParentFormattingContext::NoParent,
        ),
        available,
    )
    .with_named_baseline_demand(crate::output::NamedBaselineDemand::Fact {
        name: name(mode),
        set: crate::output::NamedBaselineSet::First,
    })
}

fn packet<S: LayoutScalar>(batch: &CompletedLayoutBatchOf<u32, S>, node: u32) -> NodeOutputOf<S> {
    batch
        .final_entries()
        .iter()
        .find(|entry| entry.node() == node)
        .unwrap()
        .output()
}

fn ordinary_relative_child_exports_normal_baseline<S: LayoutScalar>(mode: WritingMode) {
    let paragraph = Paragraph::new("X", mode);
    let binding = binding_at(&paragraph, 2, 11);
    // H uses the real hhea8/2 reference; the actual upright absent-vertical-em
    // policy supplies5/5. Neither baseline is changed by relative movement.
    let expected_baseline = if vertical(mode) { 5.0 } else { 8.0 };
    for offset in [0.0, 0.72] {
        let mut child = style::<S>(mode);
        child.position = Position::Relative;
        child.size = if vertical(mode) {
            Size::new(px(10.0), px(100.0))
        } else {
            Size::new(px(100.0), px(10.0))
        };
        child.inset = relative_block_start(mode, offset);
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [2])
            .children(2, [11])
            .children(11, [])
            .style(0, style(mode))
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
        {
            let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
            let request = normal_request(mode);
            let output = owner.compute_child(0, request).unwrap();
            // This is an operational ComputeOutput consumed by normal ancestors,
            // not the final NodeOutput named-baseline visual readback.
            assert_facts(output, mode, 10.0, expected_baseline, expected_baseline);
            assert_eq!(owner.compute_child(0, request).unwrap(), output);
            for node in [0, 2, 11] {
                assert!(owner.unrounded(node).is_err());
            }
        }
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::viewport(if vertical(mode) {
                Size::new(AvailableOf::MAX_CONTENT, AvailableOf::definite(n(100.0)))
            } else {
                Size::new(AvailableOf::definite(n(100.0)), AvailableOf::MAX_CONTENT)
            })
            .unwrap(),
        )
        .unwrap();
        let delta = if offset == 0.0 {
            0.0
        } else if mode == WritingMode::VerticalRl {
            -1.0
        } else {
            1.0
        };
        let expected_location = if vertical(mode) {
            Point::new(n(delta), S::ZERO)
        } else {
            Point::new(S::ZERO, n(delta))
        };
        assert_eq!(
            packet(&batch, 2).location,
            expected_location,
            "{mode:?} offset={offset}"
        );
        assert_eq!(packet(&batch, 0).size, physical(mode, 10.0));
        let placed = batch
            .placed_text()
            .iter()
            .find(|entry| entry.container() == 2)
            .unwrap();
        assert_eq!(
            placed.layout().analyzed_source().identity(),
            paragraph.prepared.analyzed_source().identity()
        );
        assert_eq!(placed.layout().source().text(), paragraph.original.text());
        let lines = placed.layout().lines();
        assert_eq!(lines.len(), 1);
        // The actual canonical visual line follows the published child once.
        // No assertion interprets a final named visual packet as normal input.
        assert_eq!(
            lines[0].baseline(),
            expected_baseline + delta,
            "{mode:?} offset={offset}"
        );
    }
}

macro_rules! ordinary_case {
    ($name:ident, $scalar:ty, $mode:expr) => {
        #[test]
        fn $name() {
            ordinary_relative_child_exports_normal_baseline::<$scalar>($mode);
        }
    };
}
ordinary_case!(
    ordinary_relative_block_normal_baseline_h_f32,
    f32,
    WritingMode::HorizontalTb
);
ordinary_case!(
    ordinary_relative_block_normal_baseline_h_f64,
    f64,
    WritingMode::HorizontalTb
);
ordinary_case!(
    ordinary_relative_block_normal_baseline_vrl_f32,
    f32,
    WritingMode::VerticalRl
);
ordinary_case!(
    ordinary_relative_block_normal_baseline_vrl_f64,
    f64,
    WritingMode::VerticalRl
);
ordinary_case!(
    ordinary_relative_block_normal_baseline_vlr_f32,
    f32,
    WritingMode::VerticalLr
);
ordinary_case!(
    ordinary_relative_block_normal_baseline_vlr_f64,
    f64,
    WritingMode::VerticalLr
);

fn relative_ancestor_keeps_normal_atomic_line<S: LayoutScalar>() {
    use crate::test_support::text::{ParagraphPart, PreparedParagraph};
    font_premises();
    // The same genuine producer pattern as relative_flow/relative_canonical:
    // original X, actual captured atomic10x10.4, original X. No supplied native
    // bidi/measurement surrogate, generated glyph, or nominal output receipt.
    let text_style = ahem_style(10.0, 10.0);
    let paragraph = PreparedParagraph::new(
        vec![
            ParagraphPart::text(1, text::Source::new("X"), text_style.clone()),
            ParagraphPart::Atomic {
                node: 2,
                size: text::Size::new(10.0, 10.4),
            },
            ParagraphPart::text(3, text::Source::new("X"), text_style.clone()),
        ],
        text_style,
        text::ParagraphBaseDirection::LeftToRight,
    );
    let binding = paragraph.binding(4);
    assert_eq!(binding.nodes, vec![1, 2, 3]);
    assert_eq!(binding.atomics.len(), 1);
    assert_eq!(binding.atomics[0].node, 2);
    assert_eq!(
        binding.atomics[0].occurrence,
        paragraph.prepared.atomic_views().next().unwrap().handle()
    );
    for offset in [0.0, 0.72] {
        let root = NodeInputOf {
            display: Display::Block,
            size: Size::new(px(30.0), PreferredSizeOf::AUTO),
            ..NodeInputOf::<S>::default()
        };
        let ancestor = NodeInputOf {
            position: Position::Relative,
            inset: relative_block_start(WritingMode::HorizontalTb, offset),
            ..root.clone()
        };
        let atomic = NodeInputOf {
            display: Display::InlineBlock,
            vertical_align: VerticalAlignOf::Baseline,
            size: Size::new(px(10.0), px(10.4)),
            ..NodeInputOf::<S>::default()
        };
        assert!(atomic.atomic_inline_participation.is_none());
        let tree = PublicLayoutTreeOf::<S>::new()
            .children(0, [4])
            .children(4, [1, 2, 3])
            .children(1, [])
            .children(2, [])
            .children(3, [])
            .style(0, root)
            .style(4, ancestor)
            .style(2, atomic)
            .input(1, paragraph.text_input(1))
            .input(3, paragraph.text_input(3));
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding.clone()],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(n(30.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
        .unwrap();
        let placed = batch
            .placed_text()
            .iter()
            .find(|entry| entry.container() == 4)
            .unwrap();
        assert_eq!(placed.nodes(), &[1, 2, 3]);
        assert_eq!(
            placed.layout().analyzed_source().identity(),
            paragraph.prepared.analyzed_source().identity()
        );
        assert_eq!(placed.layout().source().text().code_units(), &[0x58, 0x58]);
        for run in placed.layout().glyph_runs() {
            assert_eq!(run.font().bytes(), AHEM);
            assert_eq!(run.font_size(), 10.0);
        }
        let finalized = batch
            .final_inline_fragments()
            .iter()
            .find(|entry| *entry.owner() == InlineFragmentOwner::Node(1))
            .unwrap()
            .fragment()
            .finalized_fragment()
            .finalized_line();
        assert_eq!(finalized.prepared_text(), paragraph.prepared);
        let feedback = finalized.feedback();
        // Before8/after2 real Ahem X plus canonical empty-atomic before10/after0
        // gives normal baseline10/H12. Three10px inline operands fit exactly30.
        assert_eq!(
            (feedback.advance, feedback.baseline, feedback.line_height),
            (30.0, 10.0, 12.0),
            "ancestor offset={offset}"
        );
        assert_eq!(placed.layout().lines().len(), 1);
        assert_eq!(packet(&batch, 0).size, Size::new(n(30.0), n(12.0)));
        assert_eq!(packet(&batch, 4).size, Size::new(n(30.0), n(12.0)));
        let delta = if offset == 0.0 { 0.0 } else { 1.0 };
        assert_eq!(packet(&batch, 4).location, Point::new(S::ZERO, n(delta)));
        assert_eq!(placed.layout().lines()[0].baseline(), 10.0 + delta);
        let native_atomic = packet(&batch, 2);
        let visual_atomic = placed.layout().inline_boxes()[0].rect();
        assert_eq!(visual_atomic.origin.x, native_atomic.location.x.to_f64());
        assert_eq!(
            visual_atomic.origin.y,
            delta + native_atomic.location.y.to_f64()
        );
        assert_eq!(visual_atomic.size.width, native_atomic.size.width.to_f64());
        assert_eq!(
            visual_atomic.size.height,
            native_atomic.size.height.to_f64()
        );
        for node in [1, 3] {
            let fragment = batch
                .final_inline_fragments()
                .iter()
                .find(|entry| *entry.owner() == InlineFragmentOwner::Node(node))
                .unwrap()
                .fragment();
            assert_eq!(fragment.baseline().y, n(10.0));
            assert_eq!(fragment.rect().unwrap().origin().y, n(2.0));
        }
    }
}
#[test]
fn relative_ancestor_keeps_normal_atomic_line_f32() {
    relative_ancestor_keeps_normal_atomic_line::<f32>();
}
#[test]
fn relative_ancestor_keeps_normal_atomic_line_f64() {
    relative_ancestor_keeps_normal_atomic_line::<f64>();
}
