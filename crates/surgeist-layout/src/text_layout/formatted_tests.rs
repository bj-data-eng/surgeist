use super::*;
use crate::inline::{
    ForcedLineBreakControlOf, InlineControlAlignment, InlineFlowOf, LogicalLineBandQueryResultOf,
    MixedInlineParticipantOf, MixedInlineRunInputOf,
    layout_mixed_inline_run_with_formatted_observation,
};
use crate::test_support::text::{PreparedParagraph, ahem_style};
use crate::{AvailableOf, ComputeInputOf, Direction, FlowAxes, Size, WritingMode};
use surgeist_text::PreparedText;

fn prepared() -> PreparedText {
    PreparedParagraph::plain(
        1,
        surgeist_text::Source::new("A AA)"),
        ahem_style(10.0, 10.0),
    )
    .prepared
}

fn input() -> ComputeInputOf<f64> {
    ComputeInputOf::leaf_layout(
        Size::NONE,
        Size::NONE,
        crate::ContainingLayoutContext::new(
            FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            crate::ParentFormattingContext::BlockFlow,
        ),
        Size::new(AvailableOf::definite(40.0), AvailableOf::MAX_CONTENT),
    )
    .unwrap()
}

fn scope() -> ComputeScope<u32, f64> {
    ComputeScope {
        node: 0,
        input: input(),
        anonymous: false,
        frame: None,
    }
}

fn endpoint(prepared: &PreparedText) -> FormattedTailEndpoint {
    FormattedTailEndpoint::new(prepared.boundary_views().last().unwrap().handle(), None)
}

fn state(prepared: &PreparedText) -> (FormattedState<u32, f64>, RunBinding<u32, f64>) {
    let mut state = FormattedState::new();
    state.stack.push(scope());
    let binding = state.binding(
        0,
        &[1],
        &prepared.source_segments().next().unwrap(),
        input(),
        super::super::fit::Available::Definite(40.0),
    );
    (state, binding)
}

#[test]
fn unavailable_and_foreign_snapshots_do_not_manufacture_false_authority() {
    let p = prepared();
    let end = endpoint(&p);
    assert!(
        FormattedTailSnapshot::default()
            .resolve(&p.source_segments().next().unwrap(), &end.boundary, None)
            .is_none()
    );
    let known = FormattedTailSnapshot::known(p.source_segments().next().unwrap(), None);
    assert!(!known.observes());
    assert!(
        !known
            .resolve(&p.source_segments().next().unwrap(), &end.boundary, None)
            .unwrap()
            .last()
    );
    let foreign = prepared();
    assert!(
        known
            .resolve(
                &p.source_segments().next().unwrap(),
                &endpoint(&foreign).boundary,
                None
            )
            .is_none()
    );
}

#[test]
fn external_completion_is_not_target_membership_in_accepted_endpoints() {
    let p = prepared();
    let (mut state, binding) = state(&p);
    let target = p.boundary_views().nth(1).unwrap().handle();
    let ordinary = state
        .snapshot(&binding)
        .resolve(&p.source_segments().next().unwrap(), &target, None)
        .unwrap();
    let external = state
        .snapshot(&binding)
        .resolve_external(&p.source_segments().next().unwrap(), &target, None)
        .unwrap();
    state.record(FormattedFlow {
        frame: None,
        items: vec![FormattedFlowItem::Text(binding.clone(), vec![endpoint(&p)])],
    });
    assert_eq!(state.observe(&binding, &ordinary), Some(false));
    assert_eq!(state.observe(&binding, &external), Some(true));
    // Even a neutral actual run has a genuine completion occurrence. Absence
    // of the prospective target from actual lines is not a false Last oracle.
    state.record(FormattedFlow {
        frame: None,
        items: vec![FormattedFlowItem::Text(binding.clone(), vec![])],
    });
    assert_eq!(state.observe(&binding, &external), Some(true));
}

fn numeric_forced_lines() -> Vec<NumericFormattedLine> {
    let flow = FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr);
    let metrics = crate::InlineMetricsOf::from_ascent_descent(
        crate::InlineBaselineContext::Physical,
        8.0,
        2.0,
    )
    .unwrap();
    let report = layout_mixed_inline_run_with_formatted_observation(
        MixedInlineRunInputOf {
            inline_baseline_context: crate::InlineBaselineContext::Physical,
            groups: crate::group_alignment::Snapshot::default(),
            text_wrap_style: crate::TextWrapStyle::Auto,
            wrapping: None,
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            observe_tail: false,
            available_inline_extent: AvailableOf::definite(40.0),
            flow_axes: flow,
            text_align: crate::TextAlign::Start,
            text_align_last: crate::TextAlignLast::Auto,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            participants: vec![MixedInlineParticipantOf::ForcedLineBreak(
                ForcedLineBreakControlOf::new(
                    0,
                    InlineFlowOf::new(
                        WritingMode::HorizontalTb,
                        Direction::Ltr,
                        AvailableOf::MAX_CONTENT,
                    ),
                    metrics,
                    InlineControlAlignment::Baseline,
                    crate::Clear::None,
                ),
            )],
        },
        |_, _| LogicalLineBandQueryResultOf {
            inline_start: 0.0,
            inline_end: 40.0,
            next_transition: None,
        },
        |block, _| block,
        true,
        crate::inline::unavailable_atomic_receiver,
    )
    .unwrap();
    let lines = report.formatted_lines.unwrap();
    assert_eq!(lines.len(), report.line_bands.len());
    assert_eq!(lines[0].ending, surgeist_text::LineEnd::Forced);
    assert!(!report.indent_state.first_formatted());
    lines
}

#[test]
fn genuine_empty_forced_numeric_lines_disprove_both_tail_predicates() {
    let p = prepared();
    let (mut state, binding) = state(&p);
    let end = endpoint(&p);
    let ordinary = state
        .snapshot(&binding)
        .resolve(&p.source_segments().next().unwrap(), &end.boundary, None)
        .unwrap();
    let external = state
        .snapshot(&binding)
        .resolve_external(&p.source_segments().next().unwrap(), &end.boundary, None)
        .unwrap();
    let lines = numeric_forced_lines();
    state.record(FormattedFlow {
        frame: None,
        items: vec![
            FormattedFlowItem::Text(binding.clone(), vec![end]),
            FormattedFlowItem::Numeric(vec![2], lines),
        ],
    });
    assert_eq!(state.observe(&binding, &ordinary), Some(false));
    assert_eq!(state.observe(&binding, &external), Some(false));
}

#[test]
fn summary_receipts_exist_only_in_the_actual_invocation_and_replay() {
    let p = prepared();
    let (mut state, binding) = state(&p);
    let end = endpoint(&p);
    let assignment = state
        .snapshot(&binding)
        .resolve(&p.source_segments().next().unwrap(), &end.boundary, None)
        .unwrap();
    assert!(state.receipt(0, &input()).is_none());
    state.merge(
        binding.clone(),
        FormattedTailUses {
            demands: vec![assignment.clone()],
            roles: vec![],
            accepted: vec![end.clone()],
        },
    );
    state.record(FormattedFlow {
        frame: None,
        items: vec![FormattedFlowItem::Text(binding.clone(), vec![end])],
    });
    state.finish_block(scope(), true);
    state.next_pass();
    assert!(state.restore(0, &input()));
    assert_eq!(state.observe(&binding, &assignment), Some(true));
    // The same immutable host inputs and Text handles do not transfer a
    // semantic receipt to a new computation session.
    let fresh = FormattedState::<u32, f64>::new();
    assert!(fresh.receipt(0, &input()).is_none());
}

#[test]
fn no_demand_keeps_summary_collection_and_cache_requirements_absent() {
    let p = prepared();
    let paragraphs = [];
    let mut computation = super::super::state::TextComputation::<u32, f64>::new(
        &paragraphs,
        Arc::new(super::super::capture::ScopeAssociations::default()),
        crate::text_layout::scope_alignment::ScopeAlignmentResources::new(
            crate::text_layout::scope_alignment::ScopeAlignmentSnapshot::admit(
                Vec::new(),
                &paragraphs,
                false,
            )
            .unwrap()
            .freeze(),
            crate::text_layout::fragmentation::InlineBoxFamilies::default(),
        ),
    );
    let snapshot = computation.formatted_snapshot(
        0,
        &[1],
        &p.source_segments().next().unwrap(),
        input(),
        super::super::fit::Available::Definite(40.0),
    );
    assert!(!snapshot.observes());
    assert!(
        computation
            .formatted_merge_run(
                0,
                &[1],
                &p.source_segments().next().unwrap(),
                input(),
                super::super::fit::Available::Definite(40.0),
                FormattedTailUses::default()
            )
            .is_none()
    );
    assert!(computation.formatted.is_none());
    assert!(computation.formatted_new_flow().is_none());
    assert!(!computation.formatted_cache_required(123));
    assert!(computation.formatted_cache_available(123, &input()));
}

#[test]
fn a_summary_receipt_cannot_authenticate_different_geometry_or_canonical_frames() {
    let p = prepared();
    let (mut state, binding) = state(&p);
    state.record(FormattedFlow {
        frame: None,
        items: vec![FormattedFlowItem::Text(binding, vec![endpoint(&p)])],
    });
    state.finish_block(scope(), true);
    let context = crate::CacheKeyContext::new();
    let output = crate::ComputeOutputOf::from_sizes(Size::new(10.0, 10.0), Size::new(10.0, 10.0));
    let different =
        crate::ComputeOutputOf::from_sizes(Size::new(20.0, 10.0), Size::new(20.0, 10.0));
    assert!(!state.cache_matches(0, &input(), context, output));
    state.bind_output(0, &input(), context, output);
    assert!(state.cache_matches(0, &input(), context, output));
    assert!(!state.cache_matches(0, &input(), context, different));
    state.frames = Arc::from([super::super::state::CanonicalFrame {
        node: 0,
        origin: crate::Point::new(1.0, 0.0),
        output: crate::NodeOutputOf::new(),
    }]);
    assert!(!state.cache_matches(0, &input(), context, output));
}
