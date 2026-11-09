//! Native Wbr seams retain actual generated Text before a float.
use super::generated_binding;
use crate::block::float_text::FloatEvent;
use crate::block::floats::{AcceptedFloatPrefix, PendingFloat, PendingPlacement};
use crate::engine::contracts::Compute;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::*;
use surgeist_text as text;

fn finalized(prepared: &text::PreparedText) -> text::FinalizedLine {
    let segment = prepared.source_segments().next().unwrap();
    let participants = prepared.participants().collect::<Vec<_>>();
    // A generated Wbr contributes through its real Text participant. Only
    // actual selected Control participants require control allocations here.
    let controls = participants
        .iter()
        .filter_map(|participant| {
            let text::PreparedParticipant::Control(occurrence) = participant else {
                return None;
            };
            assert!(matches!(
                occurrence.view().kind(),
                text::ParagraphControlKind::InlineBoundaryStart
                    | text::ParagraphControlKind::InlineBoundaryEnd
            ));
            Some(text::ControlAllocation {
                occurrence: occurrence.clone(),
                boundary: Some(text::InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                strut: Some(text::ControlStrut {
                    baseline: 8.0,
                    line_extent: 10.0,
                    alignment: text::LineBoxAlignment::Baseline,
                }),
            })
        })
        .collect();
    prepared
        .select(text::LineSelectionRequest {
            inline_boxes: None,
            segment: segment.clone(),
            start: segment.start(),
            end: segment.end(),
            participants,
            termination: text::PreparedTermination::Ordinary(segment.end_opportunity()),
            alternative: None,
            incoming: None,
            atomics: Vec::new(),
            controls,
            boundary_context: Vec::new(),
            line_strut: None,
            line_box_struts: Vec::new(),
            context: text::LineContext {
                alignment_insets: text::LineAlignmentInsets::ZERO,
                formatted_line: text::FormattedLineContext::default(),
                base_direction: text::ParagraphBaseDirection::LeftToRight,
                line_box_direction: text::LineBoxDirection::LeftToRight,
                orientation: prepared.orientation(),
                start: text::LineStart::Paragraph,
                end: text::LineEnd::Paragraph,
                available_inline: Some(30.0),
                indent: 0.0,
                alignment: text::Alignment::Start,
                tabs: text::TabContext::NoTabs,
                spacing: text::SpacingPolicy::None,
            },
        })
        .unwrap()
        .finalize(text::LineAllocations {
            spacing: text::SpacingAllocations::None,
            scopes: text::ScopeAllocations::None,
        })
        .unwrap()
}

fn generated_prefix<S: LayoutScalar>() {
    let prefix_error = |error| crate::block::inline_run::text_run_error::<u32, S, ()>(0, error);
    let binding = generated_binding();
    assert!(binding.origins.is_empty());
    let generated = binding
        .controls
        .iter()
        .find(|row| row.node == 24)
        .unwrap()
        .occurrence
        .clone();
    let end = binding
        .controls
        .iter()
        .find(|row| row.node == 21)
        .unwrap()
        .occurrence
        .clone();
    let text = binding
        .prepared
        .participants()
        .find(|participant| {
            matches!(participant, text::PreparedParticipant::Text(unit)
            if unit.view().controls().any(|control| control == generated))
        })
        .expect("actual generated Wbr Text participant");
    let text::PreparedParticipant::Text(unit) = &text else {
        unreachable!()
    };
    assert!(unit.view().empty_origin().is_none());
    assert!(unit.view().processed_range().is_empty());
    assert_eq!(unit.view().origins().count(), 0);
    assert_eq!(
        unit.view().controls().collect::<Vec<_>>().as_slice(),
        std::slice::from_ref(&generated)
    );
    let line = finalized(&binding.prepared);
    assert_eq!(line.feedback().advance, 10.0);
    for (seam, expected) in [
        (
            text::TextSeam::BeforeParticipant(text.clone()),
            (0.0, false),
        ),
        (text::TextSeam::AfterParticipant(text), (10.0, true)),
        (
            text::TextSeam::BeforeParticipant(text::PreparedParticipant::Control(end)),
            (10.0, true),
        ),
    ] {
        assert_eq!(
            AcceptedFloatPrefix::Seam(seam)
                .observe(&line)
                .map_err(prefix_error)
                .unwrap(),
            expected
        );
    }

    let axes = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    for (nodes, offset, expected) in [
        (vec![20, 2, 24, 21], 1, (0.0, false)),
        (vec![20, 24, 2, 21], 2, (10.0, true)),
    ] {
        let mut binding = binding.clone();
        binding.nodes = nodes;
        let mut tree = PublicLayoutTreeOf::<S>::new()
            .children(0, binding.nodes.iter().copied())
            .style(
                0,
                NodeInputOf {
                    display: Display::Block,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(30.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..Default::default()
                },
            )
            .children(24, [])
            .input(
                24,
                LayoutInputOf::text_control(TextControlParticipation::new(generated.clone())),
            )
            .children(2, [])
            .style(
                2,
                NodeInputOf {
                    display: Display::Block,
                    float: Float::Left,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(10.0)),
                        PreferredSizeOf::px(S::from_f64(10.0)),
                    ),
                    ..Default::default()
                },
            );
        for (node, kind) in [
            (20, InlineBoundaryKind::Start),
            (21, InlineBoundaryKind::End),
        ] {
            tree = tree.children(node, []).input(
                node,
                LayoutInputOf::inline_boundary(
                    InlineBoundaryInputOf::contribution(
                        kind,
                        InlineMetricsOf::try_new(
                            InlineBaselineContext::Physical,
                            S::from_f64(8.0),
                            S::from_f64(10.0),
                        )
                        .unwrap(),
                        InlineBoundaryEdgeOf::ZERO,
                    )
                    .unwrap(),
                ),
            );
        }
        let capture = LayoutFlowInput::try_new(
            &tree,
            0,
            vec![binding],
            WrappingInput::default(),
            Vec::new(),
        )
        .unwrap();
        let [row] = capture.associations.runs() else {
            panic!("one actual generated-source run")
        };
        let admitted = capture.associations.admitted_run(&capture.paragraphs, row);
        let request = ComputeInputOf::for_child(
            RunMode::PerformLayout,
            SizingMode::InherentSize,
            RequestedAxis::Both,
            Size::NONE,
            Size::new(Some(S::from_f64(30.0)), None),
            ContainingLayoutContext::new(axes, ParentFormattingContext::BlockFlow),
            SizingRequestOf::new(
                Size::new(
                    AxisSizingRequestOf::new(
                        AvailableSpaceOf::Definite(
                            NonNegativeFiniteOf::new(S::from_f64(30.0)).unwrap(),
                        ),
                        SizingOperation::Normal,
                    ),
                    AxisSizingRequestOf::new(AvailableSpaceOf::Indefinite, SizingOperation::Normal),
                ),
                OrthogonalFallbackOf::new(
                    Size::NONE,
                    NearestScrollportOf::Absent,
                    Size::splat(OrthogonalFallbackLimitOf::Unbounded),
                ),
            ),
        );
        let mut owner = crate::engine::ComputeSession::with_flow(&capture, Vec::new());
        let packet = owner
            .probe_float_child(2, request, crate::engine::FloatChildTrial::Detached)
            .unwrap();
        assert_eq!(packet.output.size, Size::splat(S::from_f64(10.0)));
        let pending = PendingFloat {
            placement: PendingPlacement::Unresolved,
            node: 2,
            source_index: offset,
            side: Float::Left,
            clear: Clear::None,
            block_start: S::ZERO,
            size: packet.output.size,
            named_baselines: packet.output.named_baselines,
            content_size: packet.output.content_size,
            border: Edges::ZERO,
            padding: Edges::ZERO,
            margin: Edges::ZERO,
            float_exclusion: FloatExclusion::MarginBox,
            child_compute_geometry: packet.output.scroll_geometry,
        };
        let event = FloatEvent::new(
            pending,
            packet,
            &admitted,
            &capture.associations,
            offset,
            request,
            None,
        )
        .unwrap_or_else(|_| panic!("actual native generated-source float event"));
        assert_eq!(
            AcceptedFloatPrefix::Seam(event.seam.clone())
                .observe(&line)
                .map_err(prefix_error)
                .unwrap(),
            expected,
            "native Wbr prefix retains its actual generated Text extent and content"
        );
    }
}

#[test]
fn generated_wbr_before_float_retains_actual_prefix_f32() {
    generated_prefix::<f32>();
}
#[test]
fn generated_wbr_before_float_retains_actual_prefix_f64() {
    generated_prefix::<f64>();
}
