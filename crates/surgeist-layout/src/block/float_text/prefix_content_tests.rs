//! Actual prefix content stays distinct from nominal metric participation.
use crate::block::floats::AcceptedFloatPrefix;
use crate::test_support::inline_endpoint::Fixture;
use crate::test_support::readback::{Phase, single_box_output};
use crate::test_support::text::{ParagraphPart, PreparedParagraph, ahem_style};
use crate::*;
use surgeist_text as text;

fn sizing<S: LayoutScalar>() -> SizingRequestOf<S> {
    SizingRequestOf::new(
        Size::new(
            AxisSizingRequestOf::new(
                AvailableSpaceOf::Definite(NonNegativeFiniteOf::new(S::from_f64(30.0)).unwrap()),
                SizingOperation::Normal,
            ),
            AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
        ),
        OrthogonalFallbackOf::new(
            Size::NONE,
            NearestScrollportOf::Absent,
            Size::splat(OrthogonalFallbackLimitOf::Unbounded),
        ),
    )
}

fn quality_nominal_prefix<S: LayoutScalar>() {
    for policy in [
        TextWrapStyle::Auto,
        TextWrapStyle::Balance,
        TextWrapStyle::Pretty,
        TextWrapStyle::AvoidShortLastLine,
    ] {
        let fixture = Fixture::new("A B C D", (0, 7), text::WhiteSpace::Collapse, false, false);
        let mut binding = fixture.binding.clone();
        binding.nodes.insert(1, 2);
        let tree = fixture
            .tree::<S>(
                InlineBoundaryEdgeOf::ZERO,
                InlineBoundaryEdgeOf::ZERO,
                false,
                Some(30.0),
                true,
            )
            .children(0, binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    text_wrap_style: policy,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..Default::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::standalone(
                sizing(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::NONE,
            ),
        )
        .unwrap();
        // Source starts at0. F25 fits width30 beside the empty prefix but
        // leaves5px, so no Ahem10 word fits until its20px bottom.
        assert_eq!(
            single_box_output(&batch, 2, Phase::Final).location.y,
            S::ZERO
        );
        assert_eq!(
            batch.placed_text()[0]
                .layout()
                .lines()
                .iter()
                .map(|line| line.bounds().origin.y)
                .collect::<Vec<_>>(),
            [20.0, 30.0],
            "{policy:?}",
        );
        assert_eq!(
            single_box_output(&batch, 0, Phase::Final).size.height,
            S::from_f64(40.0),
        );
    }
}

#[test]
fn nominal_prefix_float_reflows_under_each_quality_policy_f32() {
    quality_nominal_prefix::<f32>();
}

#[test]
fn nominal_prefix_float_reflows_under_each_quality_policy_f64() {
    quality_nominal_prefix::<f64>();
}

#[test]
fn prefix_consumer_keeps_each_individual_edge_even_when_advance_cancels() {
    for (margin, border, padding) in [(0.0, 0.0, 0.0), (-1.0, 1.0, 0.0), (-1.0, 0.0, 1.0)] {
        let fixture = Fixture::new("X", (0, 1), text::WhiteSpace::Collapse, false, false);
        let opening = fixture.binding.controls[0].occurrence.clone();
        let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
        let scope = opening.captured_control().unwrap().scope();
        let inline = GeneratedBoxRef::Inline(InlineBoxRef::Scope(scope.clone()));
        let child = |box_| GeneratedChildBoxInput {
            box_,
            participation: GeneratedBoxParticipation::InFlow,
        };
        let order = |parent, children| InlineChildOrderInput {
            parent,
            children,
            prefix: ChildOrderCompleteness::Complete,
            suffix: ChildOrderCompleteness::Complete,
            formatting_context: 0,
        };
        let wrapping = WrappingInput {
            scopes: vec![WrappingScopeInput::Captured {
                scope,
                wrap_inside: WrapInside::Auto,
                edges: WrapEdges::AUTO,
                metric_contribution: None,
            }],
            child_order: vec![
                order(GeneratedBoxRef::Node(0), vec![child(inline.clone())]),
                order(inline, vec![child(GeneratedBoxRef::Node(10))]),
            ],
            ..Default::default()
        };
        let mut tree = fixture.tree::<f64>(
            InlineBoundaryEdgeOf::ZERO,
            InlineBoundaryEdgeOf::ZERO,
            false,
            Some(30.0),
            false,
        );
        for (node, kind, edge) in [
            (
                20,
                InlineBoundaryKind::Start,
                InlineBoundaryEdgeOf::try_new(margin, border, padding).unwrap(),
            ),
            (21, InlineBoundaryKind::End, InlineBoundaryEdgeOf::ZERO),
        ] {
            tree = tree.input(
                node,
                LayoutInputOf::inline_boundary(InlineBoundaryInputOf::box_edge(
                    kind,
                    InlineMetricsOf::try_new(InlineBaselineContext::Physical, 8.0, 10.0).unwrap(),
                    edge,
                    axes,
                )),
            );
        }
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![fixture.binding.clone()],
            wrapping,
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::standalone(sizing(), axes, Size::NONE),
        )
        .unwrap();
        let line = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let seam = text::TextSeam::AfterParticipant(text::PreparedParticipant::Control(opening));
        let prefix = line.prefix_at(&seam).unwrap();
        assert_eq!(prefix.advance(), 0.0);
        assert!(prefix.has_retained_flow_participation());
        let participates = margin != 0.0 || border != 0.0 || padding != 0.0;
        assert_eq!(prefix.has_in_flow_content(), participates);
        assert_eq!(
            AcceptedFloatPrefix::Seam(seam)
                .observe(&line)
                .map_err(|error| {
                    crate::block::inline_run::text_run_error::<u32, f64, core::convert::Infallible>(
                        0, error,
                    )
                })
                .unwrap(),
            (0.0, participates),
        );
    }
}

#[test]
fn prefix_consumer_keeps_zero_size_in_flow_atomic_and_ignores_out_of_flow_atomic() {
    for in_flow in [true, false] {
        let atomic = if in_flow {
            ParagraphPart::Atomic {
                node: 1,
                size: text::Size::new(0.0, 0.0),
            }
        } else {
            ParagraphPart::OutOfFlow {
                node: 1,
                size: text::Size::new(0.0, 0.0),
            }
        };
        let style = ahem_style(10.0, 10.0);
        let paragraph = PreparedParagraph::new(
            vec![
                atomic,
                ParagraphPart::text(10, text::Source::new("X"), style.clone()),
            ],
            style,
            text::ParagraphBaseDirection::LeftToRight,
        );
        let binding = paragraph.binding(0);
        let occurrence = binding.atomics[0].occurrence.clone();
        let tree = crate::test_support::layout_tree::PublicLayoutTreeOf::new()
            .children(0, [1, 10])
            .children(1, [])
            .children(10, [])
            .input(10, paragraph.text_input(10))
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(PreferredSizeOf::px(30.0), PreferredSizeOf::AUTO),
                    ..Default::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: Display::InlineBlock,
                    position: if in_flow {
                        Position::Relative
                    } else {
                        Position::Absolute
                    },
                    size: Size::new(PreferredSizeOf::px(0.0), PreferredSizeOf::px(0.0)),
                    ..Default::default()
                },
            );
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let batch = compute_layout_with_flow(
            &capture,
            0,
            LayoutRootRequestOf::standalone(
                sizing::<f64>(),
                FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
                Size::NONE,
            ),
        )
        .unwrap();
        let line = batch.final_inline_fragments()[0]
            .fragment()
            .finalized_fragment()
            .finalized_line();
        let seam = text::TextSeam::AfterParticipant(text::PreparedParticipant::Atomic(occurrence));
        assert_eq!(
            AcceptedFloatPrefix::Seam(seam)
                .observe(&line)
                .map_err(|error| {
                    crate::block::inline_run::text_run_error::<u32, f64, core::convert::Infallible>(
                        0, error,
                    )
                })
                .unwrap(),
            (0.0, in_flow),
        );
    }
}
