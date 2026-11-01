//! A real comparison replay must refuse contradictory external band facts.

use super::*;
use crate::test_support::inline_endpoint::Fixture;
use crate::*;
use std::cell::Cell;
use surgeist_text::{ControlAllocation, ControlStrut, InlineBoundaryAllocation, LineBoxAlignment};

#[test]
fn a_changed_comparison_band_cannot_publish_a_scoped_choice() {
    let fixture = Fixture::new(
        "A B C",
        (2, 5),
        surgeist_text::WhiteSpace::Collapse,
        false,
        false,
    );
    let tree = fixture.tree::<f64>(
        InlineBoundaryEdgeOf::ZERO,
        InlineBoundaryEdgeOf::ZERO,
        false,
        Some(40.0),
        false,
    );
    let input = LayoutFlowInput::try_new(
        &tree,
        0,
        vec![fixture.binding.clone()],
        crate::WrappingInput {
            gaps: Vec::new(),
            scopes: vec![WrappingScopeInput::NativePair {
                metric_contribution: None,
                edges: Default::default(),
                start: 20,
                end: 21,
                wrap_inside: WrapInside::Avoid,
            }],
            child_order: Vec::new(),
        },
        Vec::new(),
    )
    .unwrap();
    let [row] = input.associations.runs() else {
        panic!("endpoint fixture has one actual admitted native run");
    };
    let admitted = input.associations.admitted_run(&input.paragraphs, row);
    let binding = admitted.binding;
    let controls = binding
        .controls
        .iter()
        .map(|row| {
            let endpoint = tree.layout_input(row.node).as_inline_boundary().unwrap();
            Control {
                node: row.node,
                source_index: binding
                    .nodes
                    .iter()
                    .position(|node| *node == row.node)
                    .unwrap(),
                kind: if row.node == 20 {
                    InlineParticipantLayoutKind::InlineBoundaryStart
                } else {
                    InlineParticipantLayoutKind::InlineBoundaryEnd
                },
                allocation: ControlAllocation {
                    occurrence: row.occurrence.clone(),
                    boundary: Some(InlineBoundaryAllocation::CONTRIBUTION_ZERO),
                    strut: Some(ControlStrut {
                        baseline: 8.0,
                        line_extent: 10.0,
                        alignment: LineBoxAlignment::Baseline,
                    }),
                },
                endpoint: Some(crate::inline::InlineEndpointOf::from_input(endpoint)),
            }
        })
        .collect::<Vec<_>>();
    // The first ordinary scan sees two fitting soft cuts at capacity 40. The
    // actual overflowing completion is then the external provider's change
    // point. A second comparison scan must observe and reject capacity 30.
    let capacity = Cell::new(40.0);
    let changed = Cell::new(false);
    let result = layout(
        &admitted,
        &input.associations,
        &[],
        &controls,
        Context {
            scope_alignments: crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
                input.scope_alignments.clone(),
                crate::text_layout::fragmentation::InlineBoxFamilies::default(),
            ),
            orientation: text_orientation(WritingMode::HorizontalTb, &binding.prepared)
                .unwrap_or_else(|_| panic!("horizontal fixture must resolve its Text orientation")),
            inline_baseline_context: crate::InlineBaselineContext::Physical,
            groups: crate::group_alignment::Snapshot::default(),
            wrap_style: crate::TextWrapStyle::Auto,
            wrapping: WrappingProjection::new(
                &input.wrapping,
                &input.graph,
                &admitted,
                &input.associations,
            ),
            formatted: Default::default(),
            line_box_context: crate::line_box::LineBoxContext::new(
                LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            frame: Frame::Measurement,
            containing_size: Size::new(40.0, 100.0),
            content_insets: crate::Edges::ZERO,
            cursor_block: 0.0,
            axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            available: AvailableOf::definite(40.0),
            alignment: TextAlign::Start,
            last_alignment: TextAlignLast::Auto,
            justification: TextJustification::None,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            hyphenation_state: crate::hyphenation::LineHyphenationState::default(),
            hyphenate_limit_zone: LengthPercentageOf::ZERO,
            tail: fit::TailContext::Unrestricted,
        },
        &[],
        LineEnvironment {
            checkpoint_group: || {},
            band: |_, _| {
                Ok(Band {
                    start: 0.0,
                    end: capacity.get(),
                    intrinsic_end_occupation: None,
                    next_transition: None,
                })
            },
            clear: |_, _: &surgeist_text::LineSelectionRequest| Ok(None),
            replay: |line: &FinalizedLine, _| {
                if line.selected_line().request().context.end == surgeist_text::LineEnd::Paragraph {
                    capacity.set(30.0);
                    changed.set(true);
                }
                Ok(Vec::new())
            },
            accept: || Ok(()),
            reset: |_| {},
        },
    );
    assert!(changed.get(), "the actual completion was evaluated");
    assert!(matches!(result, Err(FitError::FeedbackCycle)));
}
