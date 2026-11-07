#![forbid(unsafe_code)]

use surgeist_layout::*;

// Normalized-mode fixtures keep source units separate from layout geometry.
type Input<S> = FragmentationInputOf<u32, S>;
type Report<S> = FragmentationReportOf<u32, S>;

struct Probe<S: LayoutScalar, F> {
    frames: Vec<Vec<FragmentainerOf<S>>>,
    queries: Vec<FragmentationQueryOf<S>>,
    layout: F,
}

impl<S, F> FragmentationProvider<S> for Probe<S, F>
where
    S: LayoutScalar,
    F: FnMut(FragmentationQueryOf<S>) -> Result<FragmentationWindowOf<S>, &'static str>,
{
    type Error = &'static str;

    fn fragmentainer(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<Option<FragmentainerOf<S>>, Self::Error> {
        Ok(self
            .frames
            .get(context)
            .and_then(|frames| frames.get(index))
            .copied())
    }

    fn layout(
        &mut self,
        query: FragmentationQueryOf<S>,
    ) -> Result<FragmentationWindowOf<S>, Self::Error> {
        self.queries.push(query);
        (self.layout)(query)
    }
}

fn probe<S, F>(frames: Vec<Vec<FragmentainerOf<S>>>, layout: F) -> Probe<S, F>
where
    S: LayoutScalar,
    F: FnMut(FragmentationQueryOf<S>) -> Result<FragmentationWindowOf<S>, &'static str>,
{
    Probe {
        frames,
        queries: Vec::new(),
        layout,
    }
}

fn position(occurrence: usize, offset: usize) -> FragmentationPosition {
    FragmentationPosition { occurrence, offset }
}

fn context(kind: FragmentationKind, parent: Option<usize>) -> FragmentationContext {
    FragmentationContext {
        kind,
        parent,
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, Direction::Ltr),
    }
}

fn flow<S: LayoutScalar>(
    context: usize,
    source: &[(u32, usize)],
    initial_block_offset: f64,
) -> FragmentationFlowOf<u32, S> {
    FragmentationFlowOf {
        context,
        start_fragmentainer: 0,
        initial_block_offset: S::from_f64(initial_block_offset),
        occurrences: source
            .iter()
            .map(|&(key, source_extent)| FragmentationOccurrence {
                key,
                source_extent,
                monolithic: false,
            })
            .collect(),
    }
}

fn input<S: LayoutScalar>(source: &[(u32, usize)], initial_block_offset: f64) -> Input<S> {
    FragmentationInputOf {
        contexts: vec![context(FragmentationKind::Page, None)],
        flows: vec![flow(0, source, initial_block_offset)],
    }
}

fn frames<S: LayoutScalar>(
    widths: &[f64],
    block_size: f64,
    crossed: FragmentationBreakKinds,
) -> Vec<FragmentainerOf<S>> {
    widths
        .iter()
        .enumerate()
        .map(|(index, &width)| FragmentainerOf {
            inline_size: S::from_f64(width),
            block_size: S::from_f64(block_size),
            breaks: if index == 0 {
                FragmentationBreakKinds::NONE
            } else {
                crossed
            },
            parent_fragmentainer: None,
            page_side: None,
        })
        .collect()
}

fn window<S: LayoutScalar>(
    block_size: f64,
    head_block_size: f64,
    opportunities: Vec<FragmentationOpportunityOf<S>>,
) -> FragmentationWindowOf<S> {
    FragmentationWindowOf {
        block_size: S::from_f64(block_size),
        head_block_size: S::from_f64(head_block_size),
        fallback: FragmentationFallback::Unsliceable,
        opportunities,
        cloned_block_start: S::ZERO,
        cloned_block_end: S::ZERO,
        emergency_slice: None,
    }
}

fn opportunity<S: LayoutScalar>(
    end: FragmentationPosition,
    offset: f64,
    kind: FragmentationOpportunityKindOf<S>,
) -> FragmentationOpportunityOf<S> {
    FragmentationOpportunityOf {
        position: end,
        block_offset: S::from_f64(offset),
        kind,
        before: Vec::new(),
        after: Vec::new(),
        ancestors: Vec::new(),
    }
}

fn control(value: FragmentationBreak, source_order: usize) -> FragmentationBreakControl {
    FragmentationBreakControl {
        value,
        role: FragmentationRole::Normal,
        source_order,
    }
}

// Only these fixtures declare an identity map between small integer source
// units and used block geometry. Reflow and graphical fixtures below do not.
fn linear_window<S: LayoutScalar>(
    query: FragmentationQueryOf<S>,
    source_extent: usize,
    cuts: &[FragmentationOpportunityOf<S>],
) -> FragmentationWindowOf<S> {
    if query.position.occurrence == 1 {
        return window(0.0, 0.0, Vec::new());
    }
    let offset = query.position.offset;
    let opportunities = cuts
        .iter()
        .filter(|cut| cut.position.offset > offset)
        .map(|cut| {
            let mut cut = cut.clone();
            cut.block_offset = cut.block_offset - S::from_usize(offset);
            cut
        })
        .collect();
    window(
        (source_extent - offset) as f64,
        (source_extent - offset) as f64,
        opportunities,
    )
}

fn coverage<S: LayoutScalar>(report: &Report<S>) -> Vec<(u32, usize, usize)> {
    report
        .pieces
        .iter()
        .flat_map(|piece| {
            piece
                .coverage
                .iter()
                .map(|part| (part.key, part.start, part.end))
        })
        .collect()
}

fn assert_linear_split<S: LayoutScalar>(
    report: &Report<S>,
    key: u32,
    end: usize,
    first_cut: usize,
    first_block: S,
) {
    assert_eq!(report.pieces.len(), 2);
    assert_eq!(report.pieces[0].start, position(0, 0));
    assert_eq!(report.pieces[0].end, position(0, first_cut));
    assert_eq!(report.pieces[0].block_size, first_block);
    assert_eq!(report.pieces[1].start, position(0, first_cut));
    assert_eq!(report.pieces[1].end, position(1, 0));
    assert_eq!(
        coverage(report),
        [(key, 0, first_cut), (key, first_cut, end)]
    );
}

macro_rules! scalars {
    ($test:ident, $f32:ident, $f64:ident) => {
        #[test]
        fn $f32() {
            $test::<f32>();
        }
        #[test]
        fn $f64() {
            $test::<f64>();
        }
    };
}

fn latest_legal_point_without_class_priority<S: LayoutScalar>() {
    let cuts = vec![
        opportunity(
            position(0, 40),
            40.0,
            FragmentationOpportunityKindOf::Sibling,
        ),
        opportunity(
            position(0, 75),
            75.0,
            FragmentationOpportunityKindOf::Lines {
                before: 3,
                after: 3,
                orphans: 2,
                widows: 2,
            },
        ),
        opportunity(
            position(0, 90),
            90.0,
            FragmentationOpportunityKindOf::ContentEdge {
                gap: S::from_f64(5.0),
            },
        ),
    ];
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        move |query| Ok(linear_window(query, 150, &cuts)),
    );
    let report = resolve_fragmentation(&input::<S>(&[(9, 150)], 0.0), &mut provider).unwrap();
    assert_linear_split(&report, 9, 150, 90, S::from_f64(90.0));
    assert_eq!(report.breaks[0].reason, FragmentationBreakReason::Unforced);
    assert_eq!(report.breaks[0].stage, FragmentationStage::Strict);
}
scalars!(
    latest_legal_point_without_class_priority,
    latest_legal_point_without_class_priority_f32,
    latest_legal_point_without_class_priority_f64
);

#[test]
fn exact_f32_geometry_has_no_breakpoint_tolerance() {
    let below = f32::from_bits(100.0_f32.to_bits() - 1);
    let above = f32::from_bits(100.0_f32.to_bits() + 1);
    for exact_fit in [false, true] {
        let mut cuts = vec![opportunity(
            position(0, 1),
            f64::from(below),
            FragmentationOpportunityKindOf::Sibling,
        )];
        if exact_fit {
            cuts.push(opportunity(
                position(0, 2),
                100.0,
                FragmentationOpportunityKindOf::Sibling,
            ));
        }
        cuts.push(opportunity(
            position(0, 3),
            f64::from(above),
            FragmentationOpportunityKindOf::Sibling,
        ));
        let mut provider = probe(
            vec![frames::<f32>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| {
                if query.position == position(0, 0) {
                    Ok(window(150.0, 150.0, cuts.clone()))
                } else {
                    Ok(window(50.0, 50.0, Vec::new()))
                }
            },
        );
        let report = resolve_fragmentation(&input::<f32>(&[(19, 4)], 0.0), &mut provider).unwrap();
        assert_eq!(
            report.pieces[0].end,
            position(0, if exact_fit { 2 } else { 1 })
        );
        assert_eq!(
            report.pieces[0].block_size,
            if exact_fit { 100.0 } else { below }
        );
    }
}

#[test]
fn exact_f64_geometry_has_no_breakpoint_tolerance() {
    let below = f64::from_bits(100.0_f64.to_bits() - 1);
    let above = f64::from_bits(100.0_f64.to_bits() + 1);
    for exact_fit in [false, true] {
        let mut cuts = vec![opportunity(
            position(0, 1),
            below,
            FragmentationOpportunityKindOf::Sibling,
        )];
        if exact_fit {
            cuts.push(opportunity(
                position(0, 2),
                100.0,
                FragmentationOpportunityKindOf::Sibling,
            ));
        }
        cuts.push(opportunity(
            position(0, 3),
            above,
            FragmentationOpportunityKindOf::Sibling,
        ));
        let mut provider = probe(
            vec![frames::<f64>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| {
                if query.position == position(0, 0) {
                    Ok(window(150.0, 150.0, cuts.clone()))
                } else {
                    Ok(window(50.0, 50.0, Vec::new()))
                }
            },
        );
        let report = resolve_fragmentation(&input::<f64>(&[(19, 4)], 0.0), &mut provider).unwrap();
        assert_eq!(
            report.pieces[0].end,
            position(0, if exact_fit { 2 } else { 1 })
        );
        assert_eq!(
            report.pieces[0].block_size,
            if exact_fit { 100.0 } else { below }
        );
    }
}

fn forced_points_are_not_skipped_to_fill<S: LayoutScalar>() {
    let mut first = opportunity(
        position(0, 30),
        30.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    first.before.push(control(FragmentationBreak::Page, 2));
    first.after.push(control(FragmentationBreak::Avoid, 1));
    first.ancestors.push(FragmentationBreakInside::Avoid);
    let mut second = opportunity(
        position(0, 60),
        60.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    second.before.push(control(FragmentationBreak::Page, 3));
    let cuts = vec![
        first,
        second,
        opportunity(
            position(0, 90),
            90.0,
            FragmentationOpportunityKindOf::Sibling,
        ),
    ];
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        move |query| Ok(linear_window(query, 150, &cuts)),
    );
    let report = resolve_fragmentation(&input::<S>(&[(3, 150)], 0.0), &mut provider).unwrap();
    assert_eq!(
        report
            .pieces
            .iter()
            .map(|piece| piece.block_size)
            .collect::<Vec<_>>(),
        [S::from_f64(30.0), S::from_f64(30.0), S::from_f64(90.0)]
    );
    assert_eq!(coverage(&report), [(3, 0, 30), (3, 30, 60), (3, 60, 150)]);
    assert_eq!(report.breaks.len(), 2);
    for (record, offset) in report.breaks.iter().zip([30, 60]) {
        assert_eq!(record.position, position(0, offset));
        assert_eq!(
            record.reason,
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None
            }
        );
        assert_eq!(record.stage, FragmentationStage::Strict);
    }
}
scalars!(
    forced_points_are_not_skipped_to_fill,
    forced_points_are_not_skipped_to_fill_f32,
    forced_points_are_not_skipped_to_fill_f64
);

fn unmatched_forces_and_absolute_controls_have_no_effect<S: LayoutScalar>() {
    for (value, role) in [
        (FragmentationBreak::Column, FragmentationRole::Normal),
        (FragmentationBreak::Page, FragmentationRole::Absolute),
    ] {
        let mut ignored = opportunity(
            position(0, 30),
            30.0,
            FragmentationOpportunityKindOf::Sibling,
        );
        ignored.before.push(FragmentationBreakControl {
            value,
            role,
            source_order: 2,
        });
        let cuts = vec![
            ignored,
            opportunity(
                position(0, 80),
                80.0,
                FragmentationOpportunityKindOf::Sibling,
            ),
        ];
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| Ok(linear_window(query, 150, &cuts)),
        );
        let report = resolve_fragmentation(&input::<S>(&[(8, 150)], 0.0), &mut provider).unwrap();
        assert_linear_split(&report, 8, 150, 80, S::from_f64(80.0));
        assert_eq!(report.breaks[0].reason, FragmentationBreakReason::Unforced);
    }
}
scalars!(
    unmatched_forces_and_absolute_controls_have_no_effect,
    unmatched_forces_and_absolute_controls_have_no_effect_f32,
    unmatched_forces_and_absolute_controls_have_no_effect_f64
);

fn latest_side_is_flow_order_not_control_vector_order<S: LayoutScalar>() {
    let mut cut = opportunity(
        position(0, 30),
        30.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    cut.before = vec![
        control(FragmentationBreak::Right, 20),
        control(FragmentationBreak::Left, 10),
    ];
    let mut page_frames = frames::<S>(&[80.0, 80.0, 80.0], 100.0, FragmentationBreakKinds::PAGE);
    for (frame, right) in page_frames.iter_mut().zip([true, false, true]) {
        frame.page_side = Some(FragmentationPageSide {
            physical: if right {
                FragmentationPhysicalPageSide::Right
            } else {
                FragmentationPhysicalPageSide::Left
            },
            progression: if right {
                FragmentationProgressionPageSide::Recto
            } else {
                FragmentationProgressionPageSide::Verso
            },
        });
    }
    let mut provider = probe(vec![page_frames], move |query| {
        Ok(linear_window(query, 150, std::slice::from_ref(&cut)))
    });
    let report = resolve_fragmentation(&input::<S>(&[(10, 150)], 0.0), &mut provider).unwrap();
    assert_eq!(report.breaks[0].to_fragmentainer, 2);
    assert_eq!(
        report.breaks[0].reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: Some(FragmentationForcedSide::Right)
        }
    );
    assert_eq!(report.pieces[1].fragmentainer, 2);
}
scalars!(
    latest_side_is_flow_order_not_control_vector_order,
    latest_side_is_flow_order_not_control_vector_order_f32,
    latest_side_is_flow_order_not_control_vector_order_f64
);

fn occupied_frame_defers_once_then_retries_fresh<S: LayoutScalar>() {
    let cuts = vec![opportunity(
        position(0, 80),
        80.0,
        FragmentationOpportunityKindOf::Sibling,
    )];
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        move |query| Ok(linear_window(query, 150, &cuts)),
    );
    let report = resolve_fragmentation(&input::<S>(&[(17, 150)], 40.0), &mut provider).unwrap();
    assert_eq!(report.breaks.len(), 2);
    assert_eq!(report.breaks[0].reason, FragmentationBreakReason::Deferred);
    assert_eq!(report.breaks[0].position, position(0, 0));
    assert_eq!(
        (
            report.breaks[0].from_fragmentainer,
            report.breaks[0].to_fragmentainer
        ),
        (0, 1)
    );
    assert_linear_split(&report, 17, 150, 80, S::from_f64(80.0));
    assert_eq!(
        (
            report.pieces[0].fragmentainer,
            report.pieces[1].fragmentainer
        ),
        (1, 2)
    );
    assert_eq!(report.pieces[0].block_offset, S::ZERO);
    assert!(
        provider
            .queries
            .iter()
            .any(|query| query.fragmentainer_index == 0
                && query.used_block_offset == S::from_f64(40.0)
                && query.available_block_size == S::from_f64(60.0))
    );
    assert!(
        provider
            .queries
            .iter()
            .any(|query| query.fragmentainer_index == 1
                && query.position == position(0, 0)
                && query.stage == FragmentationStage::Strict)
    );
}
scalars!(
    occupied_frame_defers_once_then_retries_fresh,
    occupied_frame_defers_once_then_retries_fresh_f32,
    occupied_frame_defers_once_then_retries_fresh_f64
);

fn intact_line_counts_are_not_relaxed_for_fuller_pages<S: LayoutScalar>() {
    let cuts = [60, 90].map(|offset| {
        opportunity(
            position(0, offset),
            offset as f64,
            FragmentationOpportunityKindOf::Lines {
                before: offset / 30,
                after: 5 - offset / 30,
                orphans: 2,
                widows: 3,
            },
        )
    });
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        move |query| Ok(linear_window(query, 150, &cuts)),
    );
    let report = resolve_fragmentation(&input::<S>(&[(18, 150)], 0.0), &mut provider).unwrap();
    assert_linear_split(&report, 18, 150, 60, S::from_f64(60.0));
    assert_eq!(report.pieces[0].stage, FragmentationStage::Strict);
}
scalars!(
    intact_line_counts_are_not_relaxed_for_fuller_pages,
    intact_line_counts_are_not_relaxed_for_fuller_pages_f32,
    intact_line_counts_are_not_relaxed_for_fuller_pages_f64
);

fn line_counts_relax_before_avoidance<S: LayoutScalar>() {
    for counts_candidate in [false, true] {
        let mut avoided = opportunity(
            position(0, 80),
            80.0,
            FragmentationOpportunityKindOf::Sibling,
        );
        avoided.before.push(control(FragmentationBreak::Avoid, 1));
        let mut cuts = Vec::new();
        if counts_candidate {
            cuts.push(opportunity(
                position(0, 60),
                60.0,
                FragmentationOpportunityKindOf::Lines {
                    before: 3,
                    after: 1,
                    orphans: 2,
                    widows: 2,
                },
            ));
        }
        cuts.push(avoided);
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| Ok(linear_window(query, 150, &cuts)),
        );
        let report = resolve_fragmentation(&input::<S>(&[(21, 150)], 0.0), &mut provider).unwrap();
        let cut = if counts_candidate { 60 } else { 80 };
        assert_linear_split(&report, 21, 150, cut, S::from_usize(cut));
        let stage = if counts_candidate {
            FragmentationStage::RelaxLineCounts
        } else {
            FragmentationStage::RelaxAvoidance
        };
        assert_eq!(report.pieces[0].stage, stage);
        assert_eq!(report.breaks[0].stage, stage);
    }
}
scalars!(
    line_counts_relax_before_avoidance,
    line_counts_relax_before_avoidance_f32,
    line_counts_relax_before_avoidance_f64
);

fn cloned_block_end_is_truncated_before_start<S: LayoutScalar>() {
    for (cut, stage, retained_start) in [
        (90, FragmentationStage::TruncateBlockEnd, 10.0),
        (100, FragmentationStage::TruncateBlockStart, 0.0),
    ] {
        let cuts = vec![opportunity(
            position(0, cut),
            cut as f64,
            FragmentationOpportunityKindOf::Sibling,
        )];
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| {
                let mut result = linear_window(query, 150, &cuts);
                if query.position == position(0, 0) {
                    result.cloned_block_start = S::from_f64(10.0);
                    result.cloned_block_end = S::from_f64(15.0);
                }
                Ok(result)
            },
        );
        let report = resolve_fragmentation(&input::<S>(&[(22, 150)], 0.0), &mut provider).unwrap();
        assert_linear_split(&report, 22, 150, cut, S::from_usize(cut));
        let piece = &report.pieces[0];
        assert_eq!(piece.stage, stage);
        assert_eq!(
            (piece.cloned_block_start, piece.cloned_block_end),
            (S::from_f64(retained_start), S::ZERO)
        );
        assert_eq!(
            piece.block_size + piece.cloned_block_start + piece.cloned_block_end,
            S::from_f64(100.0)
        );
    }
}
scalars!(
    cloned_block_end_is_truncated_before_start,
    cloned_block_end_is_truncated_before_start_f32,
    cloned_block_end_is_truncated_before_start_f64
);

fn equal_coordinate_zero_boxes_belong_to_earlier_fragment<S: LayoutScalar>() {
    // The two zero-height boxes still each own one source unit. Their keys are
    // deliberately unordered; membership follows source, not key or geometry.
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            let at = query.position.occurrence;
            let cuts = (at + 1..=3)
                .map(|next| {
                    opportunity(
                        position(next, 0),
                        if at == 0 { 80.0 } else { 0.0 },
                        FragmentationOpportunityKindOf::Sibling,
                    )
                })
                .collect();
            Ok(window(
                if at == 0 { 110.0 } else { 30.0 },
                if at == 0 {
                    80.0
                } else if at < 3 {
                    0.0
                } else {
                    30.0
                },
                cuts,
            ))
        },
    );
    let report = resolve_fragmentation(
        &input::<S>(&[(79, 80), (1, 1), (7, 1), (3, 30)], 0.0),
        &mut provider,
    )
    .unwrap();
    assert_eq!(report.pieces.len(), 2);
    assert_eq!(report.pieces[0].end, position(3, 0));
    assert_eq!(report.pieces[0].block_size, S::from_f64(80.0));
    assert_eq!(
        report.pieces[0]
            .coverage
            .iter()
            .map(|part| (part.key, part.start, part.end))
            .collect::<Vec<_>>(),
        [(79, 0, 80), (1, 0, 1), (7, 0, 1)]
    );
    assert_eq!(
        report.pieces[1].coverage,
        [FragmentationCoverage {
            key: 3,
            start: 0,
            end: 30
        }]
    );
    assert_eq!(report.pieces[1].block_size, S::from_f64(30.0));
}
scalars!(
    equal_coordinate_zero_boxes_belong_to_earlier_fragment,
    equal_coordinate_zero_boxes_belong_to_earlier_fragment_f32,
    equal_coordinate_zero_boxes_belong_to_earlier_fragment_f64
);

fn zero_box_after_overflow_moves_with_following_content<S: LayoutScalar>() {
    let mut fixture = input::<S>(&[(4, 7), (5, 1), (6, 2)], 0.0);
    fixture.flows[0].occurrences[0].monolithic = true;
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            Ok(match query.position.occurrence {
                0 => window(
                    120.0,
                    110.0,
                    vec![
                        opportunity(
                            position(1, 0),
                            110.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                        opportunity(
                            position(2, 0),
                            110.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                    ],
                ),
                1 => window(
                    10.0,
                    0.0,
                    vec![opportunity(
                        position(2, 0),
                        0.0,
                        FragmentationOpportunityKindOf::Sibling,
                    )],
                ),
                _ => window(10.0, 10.0, Vec::new()),
            })
        },
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    assert_eq!(report.pieces.len(), 2);
    assert_eq!(report.pieces[0].end, position(1, 0));
    assert_eq!(report.pieces[0].block_size, S::from_f64(110.0));
    assert_eq!(
        report.pieces[0].kind,
        FragmentationPieceKindOf::UnsliceableOverflow
    );
    assert_eq!(
        report.pieces[1]
            .coverage
            .iter()
            .map(|part| part.key)
            .collect::<Vec<_>>(),
        [5, 6]
    );
    assert_eq!(report.pieces[1].block_size, S::from_f64(10.0));
}
scalars!(
    zero_box_after_overflow_moves_with_following_content,
    zero_box_after_overflow_moves_with_following_content_f32,
    zero_box_after_overflow_moves_with_following_content_f64
);

fn graphical_window<S: LayoutScalar>(
    query: FragmentationQueryOf<S>,
    extent: S,
) -> FragmentationWindowOf<S> {
    let consumed = query
        .graphical_progress
        .map_or(S::ZERO, |progress| progress.consumed);
    let available = if query.available_block_size < S::ONE {
        S::ONE
    } else {
        query.available_block_size
    };
    let end = if consumed + available < extent {
        consumed + available
    } else {
        extent
    };
    FragmentationWindowOf {
        block_size: extent - consumed,
        head_block_size: extent - consumed,
        fallback: FragmentationFallback::Sliceable,
        opportunities: Vec::new(),
        cloned_block_start: S::ZERO,
        cloned_block_end: S::ZERO,
        emergency_slice: Some(FragmentationEmergencySliceOf::Graphical {
            end,
            extent,
            block_size: end - consumed,
        }),
    }
}

fn graphical_slices_keep_semantic_source_whole<S: LayoutScalar>() {
    let mut fixture = input::<S>(&[(44, 7)], 0.0);
    fixture.flows[0].occurrences[0].monolithic = true;
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 40.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| Ok(graphical_window(query, S::from_f64(150.0))),
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    assert_eq!(report.pieces.len(), 2);
    for (piece, (start, end, size)) in report
        .pieces
        .iter()
        .zip([(0.0, 100.0, 100.0), (100.0, 150.0, 50.0)])
    {
        assert_eq!(
            piece.kind,
            FragmentationPieceKindOf::GraphicalSlice {
                coverage: FragmentationGraphicalCoverageOf {
                    start: S::from_f64(start),
                    end: S::from_f64(end),
                    extent: S::from_f64(150.0)
                }
            }
        );
        assert_eq!(piece.block_size, S::from_f64(size));
        assert_eq!(
            piece.coverage,
            [FragmentationCoverage {
                key: 44,
                start: 0,
                end: 7
            }]
        );
    }
    assert_eq!(
        (report.pieces[0].start, report.pieces[0].end),
        (position(0, 0), position(0, 0))
    );
    assert_eq!(report.pieces[1].end, position(1, 0));
    assert!(
        provider
            .queries
            .iter()
            .any(|query| query.fragmentainer_index == 1
                && query.position == position(0, 0)
                && query.graphical_progress
                    == Some(FragmentationGraphicalProgressOf {
                        consumed: S::from_f64(100.0),
                        extent: S::from_f64(150.0)
                    }))
    );
}
scalars!(
    graphical_slices_keep_semantic_source_whole,
    graphical_slices_keep_semantic_source_whole_f32,
    graphical_slices_keep_semantic_source_whole_f64
);

fn fragmentainer_progress_floor_is_one_pixel<S: LayoutScalar>() {
    for capacity in [0.0, 0.5] {
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                capacity,
                FragmentationBreakKinds::PAGE,
            )],
            |query| Ok(graphical_window(query, S::from_f64(2.0))),
        );
        let report = resolve_fragmentation(&input::<S>(&[(45, 3)], 0.0), &mut provider).unwrap();
        assert_eq!(report.pieces.len(), 2);
        assert_eq!(
            report
                .pieces
                .iter()
                .map(|piece| piece.block_size)
                .collect::<Vec<_>>(),
            [S::ONE, S::ONE]
        );
        assert_eq!(coverage(&report), [(45, 0, 3), (45, 0, 3)]);
    }
}
scalars!(
    fragmentainer_progress_floor_is_one_pixel,
    fragmentainer_progress_floor_is_one_pixel_f32,
    fragmentainer_progress_floor_is_one_pixel_f64
);

fn emergency_source_mapping_is_mode_owned<S: LayoutScalar>() {
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 40.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            let mut result = if query.position == position(0, 0) {
                window(150.0, 150.0, Vec::new())
            } else {
                window(50.0, 50.0, Vec::new())
            };
            result.fallback = FragmentationFallback::Sliceable;
            if query.position == position(0, 0) {
                result.emergency_slice = Some(FragmentationEmergencySliceOf::Source {
                    end: position(0, 1),
                    block_size: S::from_f64(100.0),
                });
            }
            Ok(result)
        },
    );
    let report = resolve_fragmentation(&input::<S>(&[(46, 3)], 0.0), &mut provider).unwrap();
    assert_linear_split(&report, 46, 3, 1, S::from_f64(100.0));
    assert_eq!(report.pieces[1].block_size, S::from_f64(50.0));
}
scalars!(
    emergency_source_mapping_is_mode_owned,
    emergency_source_mapping_is_mode_owned_f32,
    emergency_source_mapping_is_mode_owned_f64
);

fn unsliceable_monolithic_box_overflows_once<S: LayoutScalar>() {
    let mut fixture = input::<S>(&[(47, 7)], 0.0);
    fixture.flows[0].occurrences[0].monolithic = true;
    let mut internal = opportunity(
        position(0, 3),
        30.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    internal.before.push(control(FragmentationBreak::Page, 1));
    let mut provider = probe(
        vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
        move |_| Ok(window(150.0, 150.0, vec![internal.clone()])),
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    assert_eq!(report.pieces.len(), 1);
    assert_eq!(report.pieces[0].block_size, S::from_f64(150.0));
    assert_eq!(
        report.pieces[0].kind,
        FragmentationPieceKindOf::UnsliceableOverflow
    );
    assert_eq!(coverage(&report), [(47, 0, 7)]);
    assert!(report.breaks.is_empty());
}
scalars!(
    unsliceable_monolithic_box_overflows_once,
    unsliceable_monolithic_box_overflows_once_f32,
    unsliceable_monolithic_box_overflows_once_f64
);

fn nonrepresentable_graphical_progress<S: LayoutScalar>(base: f64) {
    let base = S::from_f64(base);
    let extent = base + S::from_f64(4.0);
    assert_eq!(base + S::ONE, base);
    let mut frame_list = frames::<S>(&[80.0, 40.0], 1.0, FragmentationBreakKinds::PAGE);
    frame_list[0].block_size = base;
    let mut provider = probe(vec![frame_list], move |query| {
        let consumed = query
            .graphical_progress
            .map_or(S::ZERO, |progress| progress.consumed);
        let mut result = graphical_window(query, extent);
        // The mode requests a positive one-unit slice whose end is no longer
        // representable. A finite loop or successful prefix would lose coverage.
        if consumed == base {
            result.emergency_slice = Some(FragmentationEmergencySliceOf::Graphical {
                end: consumed + S::ONE,
                extent,
                block_size: S::ONE,
            });
        }
        Ok(result)
    });
    assert_eq!(
        resolve_fragmentation(&input::<S>(&[(48, 7)], 0.0), &mut provider),
        Err(FragmentationError::NoProgress {
            flow: 0,
            occurrence: 0
        })
    );
    assert!(
        provider
            .queries
            .iter()
            .any(|query| query.graphical_progress.map(|progress| progress.consumed) == Some(base))
    );
}
#[test]
fn nonrepresentable_graphical_progress_f32() {
    nonrepresentable_graphical_progress::<f32>(16_777_216.0);
}
#[test]
fn nonrepresentable_graphical_progress_f64() {
    nonrepresentable_graphical_progress::<f64>(9_007_199_254_740_992.0);
}

#[test]
fn integer_source_cuts_remain_distinct_beyond_f32_precision() {
    let mut provider = probe(
        vec![frames::<f32>(
            &[80.0, 40.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            Ok(match query.position.offset {
                0 => window(
                    150.0,
                    150.0,
                    vec![
                        opportunity(
                            position(0, 16_777_216),
                            60.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                        opportunity(
                            position(0, 16_777_217),
                            80.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                    ],
                ),
                16_777_216 => window(90.0, 90.0, Vec::new()),
                _ => window(70.0, 70.0, Vec::new()),
            })
        },
    );
    let report =
        resolve_fragmentation(&input::<f32>(&[(49, 16_777_218)], 0.0), &mut provider).unwrap();
    assert_linear_split(&report, 49, 16_777_218, 16_777_217, 80.0);
    assert_eq!(report.pieces[1].block_size, 70.0);
}

fn unequal_widths_requery_remaining_source<S: LayoutScalar>() {
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 40.0, 120.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            Ok(
                match (query.position.offset, query.fragmentainer.inline_size) {
                    (0, width) if width == S::from_f64(80.0) => window(
                        150.0,
                        150.0,
                        vec![opportunity(
                            position(0, 1),
                            60.0,
                            FragmentationOpportunityKindOf::Sibling,
                        )],
                    ),
                    (1, width) if width == S::from_f64(40.0) => window(
                        140.0,
                        140.0,
                        vec![opportunity(
                            position(0, 2),
                            80.0,
                            FragmentationOpportunityKindOf::Sibling,
                        )],
                    ),
                    (2, width) if width == S::from_f64(120.0) => window(30.0, 30.0, Vec::new()),
                    _ => return Err("unexpected source/frame pair"),
                },
            )
        },
    );
    let report = resolve_fragmentation(&input::<S>(&[(50, 3)], 0.0), &mut provider).unwrap();
    assert_eq!(coverage(&report), [(50, 0, 1), (50, 1, 2), (50, 2, 3)]);
    assert_eq!(
        report
            .pieces
            .iter()
            .map(|piece| (piece.fragmentainer, piece.block_size))
            .collect::<Vec<_>>(),
        [
            (0, S::from_f64(60.0)),
            (1, S::from_f64(80.0)),
            (2, S::from_f64(30.0))
        ]
    );
    for (index, width) in [80.0, 40.0, 120.0].into_iter().enumerate() {
        assert!(
            provider
                .queries
                .iter()
                .any(|query| query.fragmentainer_index == index
                    && query.position == position(0, index)
                    && query.fragmentainer.inline_size == S::from_f64(width)
                    && query.used_block_offset == S::ZERO
                    && query.available_block_size == S::from_f64(100.0))
        );
    }
}
scalars!(
    unequal_widths_requery_remaining_source,
    unequal_widths_requery_remaining_source_f32,
    unequal_widths_requery_remaining_source_f64
);

fn parallel_flows_choose_breaks_independently<S: LayoutScalar>() {
    let fixture = FragmentationInputOf {
        contexts: vec![context(FragmentationKind::Page, None)],
        flows: vec![
            flow::<S>(0, &[(51, 120)], 0.0),
            flow::<S>(0, &[(52, 150)], 0.0),
        ],
    };
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            let (extent, offset) = if query.flow == 0 {
                (120, 40)
            } else {
                (150, 70)
            };
            let mut cut = opportunity(
                position(0, offset),
                offset as f64,
                FragmentationOpportunityKindOf::Sibling,
            );
            if query.flow == 0 {
                cut.before.push(control(FragmentationBreak::Page, 1));
            }
            Ok(linear_window(query, extent, &[cut]))
        },
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    for (flow_index, key, cut, reason) in [
        (
            0,
            51,
            40,
            FragmentationBreakReason::Forced {
                required: FragmentationBreakKinds::PAGE,
                side: None,
            },
        ),
        (1, 52, 70, FragmentationBreakReason::Unforced),
    ] {
        let pieces = report
            .pieces
            .iter()
            .filter(|piece| piece.flow == flow_index)
            .collect::<Vec<_>>();
        assert_eq!(pieces.len(), 2);
        assert_eq!(
            (
                pieces[0].fragmentainer,
                pieces[0].block_offset,
                pieces[0].end
            ),
            (0, S::ZERO, position(0, cut))
        );
        assert_eq!(
            (
                pieces[1].fragmentainer,
                pieces[1].block_offset,
                pieces[1].block_size
            ),
            (1, S::ZERO, S::from_f64(80.0))
        );
        assert_eq!(
            pieces[0].coverage,
            [FragmentationCoverage {
                key,
                start: 0,
                end: cut
            }]
        );
        assert_eq!(
            report
                .breaks
                .iter()
                .find(|record| record.flow == flow_index)
                .unwrap()
                .reason,
            reason
        );
    }
}
scalars!(
    parallel_flows_choose_breaks_independently,
    parallel_flows_choose_breaks_independently_f32,
    parallel_flows_choose_breaks_independently_f64
);

fn nested_frames<S: LayoutScalar>(kind: FragmentationKind) -> Vec<Vec<FragmentainerOf<S>>> {
    let outer = frames::<S>(&[100.0, 100.0], 200.0, FragmentationBreakKinds::PAGE);
    let mut inner = frames::<S>(&[80.0, 70.0, 60.0], 100.0, kind.breaks());
    inner[0].parent_fragmentainer = Some(0);
    inner[1].parent_fragmentainer = Some(0);
    inner[2].parent_fragmentainer = Some(1);
    // Entry 1 crosses the inner boundary; entry 2 crosses the parent Page.
    // The latter also starts a new column row, but does not itself introduce
    // a Region boundary. Traversing both entries accumulates both types.
    inner[2].breaks = if kind == FragmentationKind::Column {
        FragmentationBreakKinds::PAGE.union(FragmentationBreakKinds::COLUMN)
    } else {
        FragmentationBreakKinds::PAGE
    };
    vec![outer, inner]
}

fn nested_forces_cross_all_required_types<S: LayoutScalar>() {
    for (kind, force_page) in [
        (FragmentationKind::Column, false),
        (FragmentationKind::Column, true),
        (FragmentationKind::Region, true),
    ] {
        let fixture = FragmentationInputOf {
            contexts: vec![
                context(FragmentationKind::Page, None),
                context(kind, Some(0)),
            ],
            flows: vec![flow::<S>(1, &[(53, 120)], 0.0)],
        };
        let mut cut = opportunity(
            position(0, 40),
            40.0,
            FragmentationOpportunityKindOf::Sibling,
        );
        let mut required = FragmentationBreakKinds::NONE;
        if kind == FragmentationKind::Column {
            cut.before.push(control(FragmentationBreak::Column, 1));
            required = required.union(FragmentationBreakKinds::COLUMN);
        }
        if force_page {
            cut.after.push(control(FragmentationBreak::Page, 2));
            required = required.union(FragmentationBreakKinds::PAGE);
        }
        let mut frame_inventory = nested_frames::<S>(kind);
        if kind == FragmentationKind::Region {
            // Direct outer split: the same Region advances to parent Page1,
            // with no intervening Region boundary in this declared path.
            frame_inventory[1].remove(1);
        }
        let mut provider = probe(frame_inventory, move |query| {
            Ok(linear_window(query, 120, std::slice::from_ref(&cut)))
        });
        let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
        assert_linear_split(&report, 53, 120, 40, S::from_f64(40.0));
        let target = if force_page && kind == FragmentationKind::Column {
            2
        } else {
            1
        };
        assert_eq!(report.pieces[1].fragmentainer, target);
        assert_eq!(report.breaks[0].to_fragmentainer, target);
        assert_eq!(report.breaks[0].crossed, required);
        assert_eq!(
            report.breaks[0].reason,
            FragmentationBreakReason::Forced {
                required,
                side: None
            }
        );
    }
}
scalars!(
    nested_forces_cross_all_required_types,
    nested_forces_cross_all_required_types_f32,
    nested_forces_cross_all_required_types_f64
);

fn intervening_region_boundary_is_retained_in_crossed_types<S: LayoutScalar>() {
    let fixture = FragmentationInputOf {
        contexts: vec![
            context(FragmentationKind::Page, None),
            context(FragmentationKind::Region, Some(0)),
        ],
        flows: vec![flow::<S>(1, &[(66, 120)], 0.0)],
    };
    let mut cut = opportunity(
        position(0, 40),
        40.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    cut.before.push(control(FragmentationBreak::Page, 1));
    let mut provider = probe(
        nested_frames::<S>(FragmentationKind::Region),
        move |query| Ok(linear_window(query, 120, std::slice::from_ref(&cut))),
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    assert_linear_split(&report, 66, 120, 40, S::from_f64(40.0));
    assert_eq!(report.pieces[1].fragmentainer, 2);
    // Required Page is reached only after crossing a real Region boundary at
    // entry1. That crossing is retained even though the force requests Page.
    assert_eq!(
        report.breaks[0].crossed,
        FragmentationBreakKinds::REGION.union(FragmentationBreakKinds::PAGE)
    );
    assert_eq!(
        report.breaks[0].reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
}
scalars!(
    intervening_region_boundary_is_retained_in_crossed_types,
    intervening_region_boundary_is_retained_in_crossed_types_f32,
    intervening_region_boundary_is_retained_in_crossed_types_f64
);

fn source_advancing_zero_coordinate_is_not_a_top_break<S: LayoutScalar>() {
    for positive_candidate in [false, true] {
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| {
                if query.position.offset == 2 {
                    return Ok(window(70.0, 70.0, Vec::new()));
                }
                let mut cuts = Vec::new();
                if query.position.offset == 0 {
                    cuts.push(opportunity(
                        position(0, 1),
                        0.0,
                        FragmentationOpportunityKindOf::Sibling,
                    ));
                }
                if positive_candidate {
                    cuts.push(opportunity(
                        position(0, 2),
                        80.0,
                        FragmentationOpportunityKindOf::Sibling,
                    ));
                }
                Ok(window(150.0, 150.0, cuts))
            },
        );
        let report = resolve_fragmentation(&input::<S>(&[(67, 3)], 0.0), &mut provider).unwrap();
        if positive_candidate {
            assert_linear_split(&report, 67, 3, 2, S::from_f64(80.0));
        } else {
            // No legal non-top point: the indivisible head overflows whole,
            // rather than publishing an empty top fragment with source loss.
            assert_eq!(report.pieces.len(), 1);
            assert_eq!(report.pieces[0].block_size, S::from_f64(150.0));
            assert_eq!(
                report.pieces[0].kind,
                FragmentationPieceKindOf::UnsliceableOverflow
            );
            assert_eq!(coverage(&report), [(67, 0, 3)]);
            assert!(report.breaks.is_empty());
        }
    }
}
scalars!(
    source_advancing_zero_coordinate_is_not_a_top_break,
    source_advancing_zero_coordinate_is_not_a_top_break_f32,
    source_advancing_zero_coordinate_is_not_a_top_break_f64
);

fn context_qualified_controls_preserve_conditional_ancestor_rule<S: LayoutScalar>() {
    // In a Page-only context, unmatched/absolute controls have effective Auto
    // applicability. The common Avoid ancestor therefore still prohibits A.
    for (value, role) in [
        (FragmentationBreak::Column, FragmentationRole::Normal),
        (FragmentationBreak::AvoidColumn, FragmentationRole::Normal),
        (FragmentationBreak::Page, FragmentationRole::Absolute),
    ] {
        let mut cut = opportunity(
            position(0, 80),
            80.0,
            FragmentationOpportunityKindOf::Sibling,
        );
        cut.before.push(FragmentationBreakControl {
            value,
            role,
            source_order: 1,
        });
        cut.ancestors.push(FragmentationBreakInside::Avoid);
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |query| Ok(linear_window(query, 150, std::slice::from_ref(&cut))),
        );
        let report = resolve_fragmentation(&input::<S>(&[(54, 150)], 0.0), &mut provider).unwrap();
        assert_eq!(report.pieces[0].end, position(0, 80));
        assert_eq!(report.pieces[0].stage, FragmentationStage::RelaxAvoidance);
    }
    // AvoidPage is applicable to this nested context, but does not prohibit a
    // Column crossing. Its non-Auto value disables conditional class-A Rule2;
    // the unconditional B/C ancestor restriction remains in force.
    for sibling in [true, false] {
        let fixture = FragmentationInputOf {
            contexts: vec![
                context(FragmentationKind::Page, None),
                context(FragmentationKind::Column, Some(0)),
            ],
            flows: vec![flow::<S>(1, &[(55, 150)], 0.0)],
        };
        let mut cut = opportunity(
            position(0, 80),
            80.0,
            if sibling {
                FragmentationOpportunityKindOf::Sibling
            } else {
                FragmentationOpportunityKindOf::Lines {
                    before: 3,
                    after: 3,
                    orphans: 2,
                    widows: 2,
                }
            },
        );
        cut.before.push(control(FragmentationBreak::AvoidPage, 1));
        cut.ancestors.push(FragmentationBreakInside::Avoid);
        let mut provider = probe(
            nested_frames::<S>(FragmentationKind::Column),
            move |query| Ok(linear_window(query, 150, std::slice::from_ref(&cut))),
        );
        let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
        assert_eq!(report.breaks[0].crossed, FragmentationBreakKinds::COLUMN);
        assert_eq!(
            report.pieces[0].stage,
            if sibling {
                FragmentationStage::Strict
            } else {
                FragmentationStage::RelaxAvoidance
            }
        );
    }
}
scalars!(
    context_qualified_controls_preserve_conditional_ancestor_rule,
    context_qualified_controls_preserve_conditional_ancestor_rule_f32,
    context_qualified_controls_preserve_conditional_ancestor_rule_f64
);

fn source_order_does_not_require_monotone_geometry<S: LayoutScalar>() {
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            if query.position == position(0, 0) {
                Ok(window(
                    150.0,
                    150.0,
                    vec![
                        opportunity(
                            position(0, 1),
                            80.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                        opportunity(
                            position(0, 2),
                            60.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                        opportunity(
                            position(0, 3),
                            110.0,
                            FragmentationOpportunityKindOf::Sibling,
                        ),
                    ],
                ))
            } else {
                Ok(window(70.0, 70.0, Vec::new()))
            }
        },
    );
    let report = resolve_fragmentation(&input::<S>(&[(56, 4)], 0.0), &mut provider).unwrap();
    assert_linear_split(&report, 56, 4, 1, S::from_f64(80.0));
}
scalars!(
    source_order_does_not_require_monotone_geometry,
    source_order_does_not_require_monotone_geometry_f32,
    source_order_does_not_require_monotone_geometry_f64
);

fn invalid_inventory_and_source_membership_fail_completely<S: LayoutScalar>() {
    for case in 0..4 {
        let mut fixture = input::<S>(&[(57, 3)], 0.0);
        match case {
            0 => {
                let duplicate = fixture.flows[0].occurrences[0].clone();
                fixture.flows[0].occurrences.push(duplicate);
            }
            1 => fixture.flows[0].context = 1,
            2 => fixture.contexts[0].parent = Some(0),
            _ => fixture.contexts[0].parent = Some(1),
        }
        let mut provider = probe(
            vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
            |_| Ok(window(10.0, 10.0, Vec::new())),
        );
        assert!(
            matches!(
                resolve_fragmentation(&fixture, &mut provider),
                Err(FragmentationError::InvalidInput(_))
            ),
            "inventory case {case}"
        );
    }
    for cuts in [
        vec![opportunity(
            position(2, 0),
            40.0,
            FragmentationOpportunityKindOf::Sibling,
        )],
        vec![opportunity(
            position(0, 4),
            40.0,
            FragmentationOpportunityKindOf::Sibling,
        )],
        vec![opportunity(
            position(1, 1),
            40.0,
            FragmentationOpportunityKindOf::Sibling,
        )],
        vec![
            opportunity(
                position(0, 2),
                40.0,
                FragmentationOpportunityKindOf::Sibling,
            ),
            opportunity(
                position(0, 1),
                80.0,
                FragmentationOpportunityKindOf::Sibling,
            ),
        ],
    ] {
        let mut provider = probe(
            vec![frames::<S>(
                &[80.0, 80.0],
                100.0,
                FragmentationBreakKinds::PAGE,
            )],
            move |_| Ok(window(150.0, 150.0, cuts.clone())),
        );
        assert!(matches!(
            resolve_fragmentation(&input::<S>(&[(58, 3)], 0.0), &mut provider),
            Err(FragmentationError::InvalidInput(_))
        ));
    }
}
scalars!(
    invalid_inventory_and_source_membership_fail_completely,
    invalid_inventory_and_source_membership_fail_completely_f32,
    invalid_inventory_and_source_membership_fail_completely_f64
);

fn invalid_nonfinite_geometry_is_typed_failure<S: LayoutScalar>() {
    for value in [S::NAN, S::INFINITY, S::ZERO - S::INFINITY, S::ZERO - S::ONE] {
        for field in 0..6 {
            // Signed used coordinates can retreat; only nonfinite point
            // geometry is invalid independently of whether it fits a frame.
            if field == 5 && value == S::ZERO - S::ONE {
                continue;
            }
            let mut frame_list = frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE);
            if field == 0 {
                frame_list[0].block_size = value;
            }
            if field == 1 {
                frame_list[0].inline_size = value;
            }
            let mut provider = probe(vec![frame_list], move |_| {
                let mut result = window(
                    150.0,
                    150.0,
                    vec![opportunity(
                        position(0, 1),
                        80.0,
                        FragmentationOpportunityKindOf::Sibling,
                    )],
                );
                match field {
                    2 => result.block_size = value,
                    3 => result.head_block_size = value,
                    4 => result.cloned_block_end = value,
                    5 => result.opportunities[0].block_offset = value,
                    _ => {}
                }
                Ok(result)
            });
            assert!(
                matches!(
                    resolve_fragmentation(&input::<S>(&[(59, 3)], 0.0), &mut provider),
                    Err(FragmentationError::InvalidInput(_))
                ),
                "geometry field {field}"
            );
        }
    }
}
scalars!(
    invalid_nonfinite_geometry_is_typed_failure,
    invalid_nonfinite_geometry_is_typed_failure_f32,
    invalid_nonfinite_geometry_is_typed_failure_f64
);

fn late_provider_and_missing_frame_errors_return_no_prefix<S: LayoutScalar>() {
    let fixture = FragmentationInputOf {
        contexts: vec![context(FragmentationKind::Page, None)],
        flows: vec![flow::<S>(0, &[(60, 1)], 0.0), flow::<S>(0, &[(61, 1)], 0.0)],
    };
    let mut provider = probe(
        vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
        |query| {
            if query.flow == 0 {
                Ok(window(10.0, 10.0, Vec::new()))
            } else {
                Err("later flow failed")
            }
        },
    );
    assert_eq!(
        resolve_fragmentation(&fixture, &mut provider),
        Err(FragmentationError::Provider("later flow failed"))
    );
    assert!(provider.queries.iter().any(|query| query.flow == 0));
    assert!(provider.queries.iter().any(|query| query.flow == 1));
    let cut = opportunity(
        position(0, 80),
        80.0,
        FragmentationOpportunityKindOf::Sibling,
    );
    let mut provider = probe(
        vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
        move |query| Ok(linear_window(query, 150, std::slice::from_ref(&cut))),
    );
    assert_eq!(
        resolve_fragmentation(&input::<S>(&[(62, 150)], 0.0), &mut provider),
        Err(FragmentationError::MissingFragmentainer {
            context: 0,
            index: 1
        })
    );
}
scalars!(
    late_provider_and_missing_frame_errors_return_no_prefix,
    late_provider_and_missing_frame_errors_return_no_prefix_f32,
    late_provider_and_missing_frame_errors_return_no_prefix_f64
);

fn zero_source_extent_retains_occurrence_identity<S: LayoutScalar>() {
    let mut provider = probe(
        vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
        |_| Ok(window(10.0, 0.0, Vec::new())),
    );
    let report =
        resolve_fragmentation(&input::<S>(&[(63, 0), (64, 3)], 0.0), &mut provider).unwrap();
    assert_eq!(report.pieces.len(), 1);
    assert_eq!(report.pieces[0].end, position(2, 0));
    assert_eq!(report.pieces[0].block_size, S::from_f64(10.0));
    assert_eq!(coverage(&report), [(63, 0, 0), (64, 0, 3)]);
}
scalars!(
    zero_source_extent_retains_occurrence_identity,
    zero_source_extent_retains_occurrence_identity_f32,
    zero_source_extent_retains_occurrence_identity_f64
);

fn false_whole_head_completion_is_rejected<S: LayoutScalar>() {
    for graphical in [false, true] {
        let mut provider = probe(
            vec![frames::<S>(&[80.0], 100.0, FragmentationBreakKinds::PAGE)],
            move |_| {
                let mut result = window(150.0, 150.0, Vec::new());
                result.fallback = FragmentationFallback::Sliceable;
                result.emergency_slice = Some(if graphical {
                    FragmentationEmergencySliceOf::Graphical {
                        end: S::from_f64(150.0),
                        extent: S::from_f64(150.0),
                        block_size: S::from_f64(100.0),
                    }
                } else {
                    FragmentationEmergencySliceOf::Source {
                        end: position(1, 0),
                        block_size: S::from_f64(100.0),
                    }
                });
                Ok(result)
            },
        );
        assert!(
            matches!(
                resolve_fragmentation(&input::<S>(&[(65, 7)], 0.0), &mut provider),
                Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidSlice | FragmentationInputError::InvalidWindow
                ))
            ),
            "graphical={graphical}"
        );
    }
}
scalars!(
    false_whole_head_completion_is_rejected,
    false_whole_head_completion_is_rejected_f32,
    false_whole_head_completion_is_rejected_f64
);

fn pending_force_frames<S: LayoutScalar>() -> Vec<Vec<FragmentainerOf<S>>> {
    let mut page_frames = frames::<S>(&[80.0, 80.0, 80.0], 100.0, FragmentationBreakKinds::PAGE);
    page_frames[0].block_size = S::from_f64(50.0);
    vec![page_frames]
}

fn pending_page_force<S: LayoutScalar>(block_offset: f64) -> FragmentationOpportunityOf<S> {
    let mut cut = opportunity(
        position(0, 2),
        block_offset,
        FragmentationOpportunityKindOf::Sibling,
    );
    cut.before.push(control(FragmentationBreak::Page, 1));
    cut
}

fn emergency_progress_cannot_bypass_pending_internal_force<S: LayoutScalar>() {
    let mut outcomes = Vec::new();
    for graphical in [false, true] {
        let mut provider = probe(pending_force_frames::<S>(), move |query| {
            if graphical {
                let consumed = query
                    .graphical_progress
                    .map_or(S::ZERO, |progress| progress.consumed);
                let mut result = graphical_window(query, S::from_f64(150.0));
                let mut cut = pending_page_force::<S>(80.0);
                cut.block_offset = cut.block_offset - consumed;
                result.opportunities.push(cut);
                Ok(result)
            } else if query.position == position(0, 0) {
                let mut result = window(150.0, 150.0, vec![pending_page_force(80.0)]);
                result.fallback = FragmentationFallback::Sliceable;
                // Source units are independent of used geometry: this partial
                // mode-provided cut crosses source2, whose Page force is pending.
                result.emergency_slice = Some(FragmentationEmergencySliceOf::Source {
                    end: position(0, 3),
                    block_size: S::from_f64(50.0),
                });
                Ok(result)
            } else {
                Ok(window(100.0, 100.0, Vec::new()))
            }
        });
        outcomes.push((
            graphical,
            resolve_fragmentation(&input::<S>(&[(68, 4)], 0.0), &mut provider),
        ));
    }
    for (graphical, result) in outcomes {
        // A graphical slice denotes the complete remaining semantic head; it
        // cannot represent discharge of an internal source boundary. Neither
        // representation may silently pass that mandatory boundary.
        assert!(
            matches!(
                &result,
                Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidSlice | FragmentationInputError::InvalidWindow
                ))
            ),
            "graphical={graphical}, result={result:?}"
        );
    }
}
scalars!(
    emergency_progress_cannot_bypass_pending_internal_force,
    emergency_progress_cannot_bypass_pending_internal_force_f32,
    emergency_progress_cannot_bypass_pending_internal_force_f64
);

fn emergency_progress_preserves_force_and_monolithic_boundaries<S: LayoutScalar>() {
    let mut provider = probe(pending_force_frames::<S>(), |query| {
        Ok(match query.position.offset {
            0 => {
                let mut result = window(150.0, 150.0, vec![pending_page_force(80.0)]);
                result.fallback = FragmentationFallback::Sliceable;
                result.emergency_slice = Some(FragmentationEmergencySliceOf::Source {
                    end: position(0, 1),
                    block_size: S::from_f64(50.0),
                });
                result
            }
            // The mode's explicit source1 cut consumed 50 used units. Its
            // remaining forced boundary is therefore at 80-50=30, followed by 70.
            1 => window(100.0, 100.0, vec![pending_page_force(30.0)]),
            _ => window(70.0, 70.0, Vec::new()),
        })
    });
    let report = resolve_fragmentation(&input::<S>(&[(69, 4)], 0.0), &mut provider).unwrap();
    assert_eq!(
        report
            .pieces
            .iter()
            .map(|piece| (piece.fragmentainer, piece.block_offset, piece.block_size))
            .collect::<Vec<_>>(),
        [
            (0, S::ZERO, S::from_f64(50.0)),
            (1, S::ZERO, S::from_f64(30.0)),
            (2, S::ZERO, S::from_f64(70.0))
        ]
    );
    assert_eq!(coverage(&report), [(69, 0, 1), (69, 1, 2), (69, 2, 4)]);
    let forced = report
        .breaks
        .iter()
        .find(|record| record.position == position(0, 2))
        .unwrap();
    assert_eq!((forced.from_fragmentainer, forced.to_fragmentainer), (1, 2));
    assert_eq!(
        forced.reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
    assert_eq!(forced.stage, FragmentationStage::Strict);

    let mut fixture = input::<S>(&[(70, 4)], 0.0);
    fixture.flows[0].occurrences[0].monolithic = true;
    let mut provider = probe(pending_force_frames::<S>(), |query| {
        let consumed = query
            .graphical_progress
            .map_or(S::ZERO, |progress| progress.consumed);
        let mut result = graphical_window(query, S::from_f64(150.0));
        let mut cut = pending_page_force::<S>(80.0);
        cut.block_offset = cut.block_offset - consumed;
        result.opportunities.push(cut);
        Ok(result)
    });
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    assert_eq!(report.pieces.len(), 2);
    for (piece, (start, end, size)) in report
        .pieces
        .iter()
        .zip([(0.0, 50.0, 50.0), (50.0, 150.0, 100.0)])
    {
        assert_eq!(
            piece.kind,
            FragmentationPieceKindOf::GraphicalSlice {
                coverage: FragmentationGraphicalCoverageOf {
                    start: S::from_f64(start),
                    end: S::from_f64(end),
                    extent: S::from_f64(150.0)
                }
            }
        );
        assert_eq!(piece.block_size, S::from_f64(size));
        assert_eq!(
            piece.coverage,
            [FragmentationCoverage {
                key: 70,
                start: 0,
                end: 4
            }]
        );
    }
    assert_eq!(report.pieces[1].end, position(1, 0));
    assert!(
        !report
            .breaks
            .iter()
            .any(|record| matches!(record.reason, FragmentationBreakReason::Forced { .. }))
    );
}
scalars!(
    emergency_progress_preserves_force_and_monolithic_boundaries,
    emergency_progress_preserves_force_and_monolithic_boundaries_f32,
    emergency_progress_preserves_force_and_monolithic_boundaries_f64
);

fn oversized_unsliceable_prefix_honors_internal_force<S: LayoutScalar>() {
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |query| {
            Ok(if query.position == position(0, 0) {
                window(200.0, 200.0, vec![pending_page_force(150.0)])
            } else {
                window(50.0, 50.0, Vec::new())
            })
        },
    );
    let report = resolve_fragmentation(&input::<S>(&[(71, 4)], 0.0), &mut provider).unwrap();
    // The supplied forced source boundary is legal even though its indivisible
    // prefix overflows by 150-100=50. Unsliceable fallback does not erase it.
    assert_linear_split(&report, 71, 4, 2, S::from_f64(150.0));
    assert_eq!(
        (
            report.pieces[0].fragmentainer,
            report.pieces[1].fragmentainer
        ),
        (0, 1)
    );
    assert_eq!(report.pieces[0].block_offset, S::ZERO);
    assert_eq!(
        report.pieces[0].kind,
        FragmentationPieceKindOf::UnsliceableOverflow
    );
    assert_eq!(
        report.pieces[0].block_size - S::from_f64(100.0),
        S::from_f64(50.0)
    );
    assert_eq!(report.pieces[1].block_offset, S::ZERO);
    assert_eq!(report.pieces[1].block_size, S::from_f64(50.0));
    assert_eq!(report.pieces[1].kind, FragmentationPieceKindOf::Content);
    assert_eq!(report.breaks.len(), 1);
    let forced = &report.breaks[0];
    assert_eq!(forced.position, position(0, 2));
    assert_eq!((forced.from_fragmentainer, forced.to_fragmentainer), (0, 1));
    assert_eq!(forced.crossed, FragmentationBreakKinds::PAGE);
    assert_eq!(
        forced.reason,
        FragmentationBreakReason::Forced {
            required: FragmentationBreakKinds::PAGE,
            side: None
        }
    );
}
scalars!(
    oversized_unsliceable_prefix_honors_internal_force,
    oversized_unsliceable_prefix_honors_internal_force_f32,
    oversized_unsliceable_prefix_honors_internal_force_f64
);

fn monolithic_unsliceable_head_ignores_internal_force<S: LayoutScalar>() {
    let mut fixture = input::<S>(&[(72, 4)], 0.0);
    fixture.flows[0].occurrences[0].monolithic = true;
    let mut provider = probe(
        vec![frames::<S>(
            &[80.0, 80.0],
            100.0,
            FragmentationBreakKinds::PAGE,
        )],
        |_| Ok(window(200.0, 200.0, vec![pending_page_force(150.0)])),
    );
    let report = resolve_fragmentation(&fixture, &mut provider).unwrap();
    // Monolithic authority suppresses the internal source2 opportunity. The
    // entire 200-unit head is consumed once, overflowing capacity 100 by 100.
    assert_eq!(report.pieces.len(), 1);
    let piece = &report.pieces[0];
    assert_eq!((piece.fragmentainer, piece.block_offset), (0, S::ZERO));
    assert_eq!((piece.start, piece.end), (position(0, 0), position(1, 0)));
    assert_eq!(piece.block_size, S::from_f64(200.0));
    assert_eq!(piece.kind, FragmentationPieceKindOf::UnsliceableOverflow);
    assert_eq!(coverage(&report), [(72, 0, 4)]);
    assert!(report.breaks.is_empty());
    assert!(
        provider
            .queries
            .iter()
            .all(|query| query.fragmentainer_index == 0)
    );
}
scalars!(
    monolithic_unsliceable_head_ignores_internal_force,
    monolithic_unsliceable_head_ignores_internal_force_f32,
    monolithic_unsliceable_head_ignores_internal_force_f64
);
