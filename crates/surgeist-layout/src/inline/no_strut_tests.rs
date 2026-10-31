//! A selected named baseline may lie outside its actual box without a strut.
use super::*;

fn no_strut<S: LayoutScalar>() {
    let reference = crate::InlineBaselineContext::Named(crate::BaselineName::Central);
    let participant = AtomicInlineBoxParticipant {
        baseline_reference: reference,
        source_index: 0,
        size: Size::new(S::from_f64(5.0), S::from_f64(10.0)),
        content_size: Size::new(S::from_f64(5.0), S::from_f64(10.0)),
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_size: Size::ZERO,
        // This is the actual selected named coordinate, not InlineMetrics:
        // finite named baseline facts are not constrained to their box extent.
        first_baseline: Some(S::from_f64(20.0)),
        alignment: InlineControlAlignment::Baseline,
    };
    let report = layout_mixed_inline_run(MixedInlineRunInputOf {
        inline_baseline_context: reference,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: crate::TextWrapStyle::Auto,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            Direction::Ltr,
        ),
        observe_tail: false,
        wrapping: None,
        available_inline_extent: AvailableOf::definite(S::from_f64(100.0)),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Start,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        // There is no parent strut or control contributing a reference point.
        participants: vec![MixedInlineParticipantOf::Atomic {
            item: participant,
            participation: AtomicInlineParticipation::new(
                BidiLevel::try_new(0).unwrap(),
                crate::InlineBreakOpportunity::Prohibited,
            ),
        }],
    })
    .unwrap();

    // Relative to the named reference, the only box occupies[-20,-10]. Its
    // envelope is10; the absent parent contributes no additional point at0.
    assert_eq!(report.block_extent, S::from_f64(10.0));
    assert_eq!(report.baselines.numeric().first, Some(S::from_f64(20.0)));
    assert_eq!(report.baselines.numeric().last, Some(S::from_f64(20.0)));
    assert!(report.baselines.numeric().first_named && report.baselines.numeric().last_named);
    assert_eq!(report.atomics.len(), 1);
    assert_eq!(
        crate::inline::expect_logical_position(report.atomics[0].position).block,
        S::ZERO
    );
    assert_eq!(
        report.atomics[0].item.first_baseline,
        participant.first_baseline
    );
    assert_eq!(report.atomics[0].item.size, participant.size);
    assert_eq!(
        report.atomics[0].item.content_size,
        participant.content_size
    );
    assert_eq!(report.line_bands[0].block_start, S::ZERO);
    assert_eq!(report.line_bands[0].block_end, S::from_f64(10.0));
}

#[test]
fn named_baseline_outside_box_without_parent_strut_preserves_only_actual_bounds_f32() {
    no_strut::<f32>();
}

#[test]
fn named_baseline_outside_box_without_parent_strut_preserves_only_actual_bounds_f64() {
    no_strut::<f64>();
}
