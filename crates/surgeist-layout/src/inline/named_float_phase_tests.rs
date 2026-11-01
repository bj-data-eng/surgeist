//! A finite exclusion changes available width, not a named baseline reference.
use super::*;

fn common_named_reference<S: LayoutScalar>() {
    let n = S::from_f64;
    let reference = crate::InlineBaselineContext::Named(crate::BaselineName::Central);
    let metrics = InlineMetricsOf::try_new(reference, n(8.0), n(10.0)).unwrap();
    let boundary = InlineBoundaryControlOf::new(
        0,
        InlineBoundaryKind::Start,
        InlineFlowOf::new(
            WritingMode::HorizontalTb,
            Direction::Ltr,
            AvailableOf::MAX_CONTENT,
        ),
        metrics,
        InlineControlAlignment::Baseline,
        InlineBoundaryEdgeOf::ZERO,
        None,
    );
    let item = AtomicInlineBoxParticipant {
        baseline_reference: reference,
        source_index: 1,
        size: Size::new(n(5.0), n(10.0)),
        content_size: Size::new(n(5.0), n(10.0)),
        margin: Edges::ZERO,
        padding: Edges::ZERO,
        border: Edges::ZERO,
        scrollbar_size: Size::ZERO,
        first_baseline: Some(n(6.0)),
        alignment: InlineControlAlignment::Baseline,
    };
    let mut probes = Vec::new();
    let report = layout_mixed_inline_run_with_band_source(
        MixedInlineRunInputOf {
            inline_baseline_context: reference,
            groups: crate::group_alignment::Snapshot::default(),
            text_wrap_style: crate::TextWrapStyle::Auto,
            line_box_context: crate::line_box::LineBoxContext::new(
                crate::LineBoxDirectionPolicy::Normal,
                Direction::Ltr,
            ),
            observe_tail: false,
            wrapping: None,
            available_inline_extent: AvailableOf::definite(n(110.0)),
            flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
            text_align: TextAlign::Start,
            text_align_last: crate::TextAlignLast::Start,
            indent: crate::text_indent::ResolvedTextIndent::ZERO,
            indent_state: crate::text_indent::IndentLineState::default(),
            participants: vec![
                MixedInlineParticipantOf::Boundary(boundary),
                MixedInlineParticipantOf::Atomic {
                    item,
                    participation: crate::AtomicInlineParticipation::new(
                        BidiLevel::try_new(0).unwrap(),
                        crate::InlineBreakOpportunity::Prohibited,
                    ),
                },
            ],
        },
        |start, end| {
            probes.push((start, end));
            // Container[0,110] and a real finite left exclusion occupying
            // inline[0,10], block[0,20]. Any intersecting nonempty line
            // receives[10,110]; the actual next block edge is20.
            let intersects = start < n(20.0) && end > S::ZERO;
            LogicalLineBandQueryResultOf {
                inline_start: if intersects { n(10.0) } else { S::ZERO },
                inline_end: n(110.0),
                next_transition: intersects.then_some(n(20.0)),
            }
        },
        |block, _| block,
    )
    .unwrap();

    let common = report.baselines.numeric().first.unwrap();
    let atomic_reference = crate::inline::expect_logical_position(report.atomics[0].position).block
        + report.atomics[0].item.first_baseline.unwrap();
    // Both actual participants demand the same Central reference. Float
    // exclusion admission cannot give the endpoint a second cross anchor.
    assert_eq!(
        crate::inline::expect_logical_position(report.controls[0].position).block,
        common
    );
    assert_eq!(atomic_reference, common);
    assert!(report.baselines.numeric().first_named && report.baselines.numeric().last_named);
    // max(before8,6)+max(after2,4)=12; the common baseline is8.
    assert_eq!(common, n(8.0));
    assert_eq!(report.block_extent, n(12.0));
    assert_eq!(
        crate::inline::expect_logical_position(report.atomics[0].position).block,
        n(2.0)
    );
    assert_eq!(
        crate::inline::expect_logical_position(report.atomics[0].position).inline,
        n(10.0)
    );
    assert_eq!(report.atomics[0].item.first_baseline, Some(n(6.0)));
    assert_eq!(report.atomics[0].item.size, item.size);
    assert_eq!(report.atomics[0].item.content_size, item.content_size);
    assert_eq!(boundary.metrics(), metrics);
    assert!(
        probes
            .iter()
            .any(|(start, end)| *start == S::ZERO && *end > S::ZERO && *end < n(20.0))
    );
    assert_eq!(report.line_bands.len(), 1);
    assert_eq!(report.line_bands[0].inline_start, n(10.0));
    assert_eq!(report.line_bands[0].inline_end, n(110.0));
}

#[test]
fn float_exclusion_keeps_named_boundary_and_atomic_on_the_same_reference_f32() {
    common_named_reference::<f32>();
}

#[test]
fn float_exclusion_keeps_named_boundary_and_atomic_on_the_same_reference_f64() {
    common_named_reference::<f64>();
}
