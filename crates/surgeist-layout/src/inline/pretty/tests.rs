use super::*;
use crate::inline::balance_tests::{groups, input};
use crate::inline::tail::tests::{ContourFailure, TriangleTree};
use crate::inline_wrapping::body::BodyAdmission;
use crate::{LayoutInputOf, NodeInputOf, PreferredSizeOf};
use std::cell::RefCell;

#[derive(Clone)]
struct BodyProof {
    entry_source: usize,
    entry_block: f64,
    advances: Vec<f64>,
    capacities: Vec<f64>,
    sources: Vec<Vec<usize>>,
    terminal: surgeist_text::LineEnd,
    terminal_cursor: (usize, usize, bool),
    fixed: BodyScore,
    candidate: BodyScore,
    admission: BodyAdmission,
}

#[derive(Clone)]
struct GroupProof {
    entry_block: f64,
    auto_count: usize,
    auto_advances: Vec<f64>,
    phase_advances: Vec<f64>,
    phase_capacities: Vec<f64>,
    selected_advances: Vec<f64>,
    outgoing_block: f64,
    tail: bool,
    body: bool,
    guards: Option<BodyScore>,
}

thread_local! {
    static BODY: RefCell<Vec<BodyProof>> = const { RefCell::new(Vec::new()) };
    static GROUPS: RefCell<Vec<GroupProof>> = const { RefCell::new(Vec::new()) };
}

fn advances<S: LayoutScalar>(basis: &GroupBasis<S>) -> Vec<f64> {
    basis
        .group
        .lines
        .iter()
        .filter(|line| numeric_is_formatted(line))
        .map(|line| line.used_inline_extent.to_f64())
        .collect()
}

fn capacities<S: LayoutScalar>(basis: &GroupBasis<S>) -> Vec<f64> {
    basis
        .group
        .lines
        .iter()
        .filter(|line| numeric_is_formatted(line))
        .map(|line| (line.band.inline_end - line.band.inline_start).to_f64() - line.indent.to_f64())
        .collect()
}

pub(super) fn observe_trial<S: LayoutScalar>(
    entry: NumericGroupEntry<S>,
    fixed: BodyScore,
    trial: &BodyTrial<GroupBasis<S>>,
) {
    // Called only on complete actual fitter trials with authenticated count,
    // terminal/control/source and optional measurement domain. Observe the
    // actual fixed guard owner; do not synthesize score/program tuples.
    let terminal = trial.value.group.lines.last().unwrap();
    BODY.with(|proofs| {
        proofs.borrow_mut().push(BodyProof {
            entry_source: entry.state.cursor.source,
            entry_block: entry.state.block_cursor.to_f64(),
            advances: advances(&trial.value),
            capacities: capacities(&trial.value),
            sources: trial
                .value
                .group
                .lines
                .iter()
                .filter(|line| numeric_is_formatted(line))
                .map(|line| {
                    line.units
                        .iter()
                        .filter_map(|unit| match unit.participant {
                            MixedInlineParticipantOf::Atomic { item, .. } => {
                                Some(item.source_index)
                            }
                            _ => None,
                        })
                        .collect()
                })
                .collect(),
            terminal: terminal.ending,
            terminal_cursor: (
                terminal.next_cursor.source,
                terminal.next_cursor.descriptor,
                terminal.next_cursor.edge_consumed,
            ),
            fixed,
            candidate: trial.score,
            admission: trial.score.admission(fixed),
        })
    });
}

pub(super) fn observe_group<S: LayoutScalar>(group: &PrettyGroupProgram<S>) {
    GROUPS.with(|proofs| {
        proofs.borrow_mut().push(GroupProof {
            entry_block: group.entry.state.block_cursor.to_f64(),
            auto_count: group.reference.count,
            auto_advances: advances(&group.ordinary),
            phase_advances: advances(&group.phase),
            phase_capacities: capacities(&group.phase),
            selected_advances: advances(&group.selected),
            outgoing_block: group.selected.group.outgoing.block_cursor.to_f64(),
            tail: group.tail_goal.is_some(),
            body: group.body_goal.is_some(),
            guards: group.guards,
        })
    });
}

fn clear_proofs() {
    BODY.with(|proofs| proofs.borrow_mut().clear());
    GROUPS.with(|proofs| proofs.borrow_mut().clear());
}

fn run<S: LayoutScalar>(input: MixedInlineRunInputOf<S>) -> MixedInlineRunReportOf<S> {
    let supplied = project_inline_participants(&input.participants);
    let available = resolved_inline_available(&input, &supplied);
    layout_mixed_inline_run_with_formatted_observation(
        input,
        |_, _| LogicalLineBandQueryResultOf {
            inline_start: S::ZERO,
            inline_end: available,
            next_transition: None,
        },
        |block, _| block,
        true,
        unavailable_atomic_receiver,
    )
    .unwrap()
}

#[test]
fn native_pretty_preserves_tail_then_improves_body_and_keeps_actual_fixed_guards() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            clear_proofs();
            let tail = run(input::<S>(
                &[30.0, 30.0, 10.0],
                &[],
                direction,
                crate::TextWrapStyle::Pretty,
            ));
            assert_eq!(groups(&tail), [vec![0], vec![1, 2]]);
            let proofs = GROUPS.with(|proofs| proofs.borrow().clone());
            assert!(proofs[0].tail);
            assert_eq!(proofs[0].phase_advances, [30.0, 40.0]);
            assert_eq!(proofs[0].selected_advances, [30.0, 40.0]);

            clear_proofs();
            let body = run(input::<S>(
                &[30.0, 30.0, 20.0, 50.0],
                &[],
                direction,
                crate::TextWrapStyle::Pretty,
            ));
            assert_eq!(groups(&body), [vec![0], vec![1, 2], vec![3]]);
            let proofs = GROUPS.with(|proofs| proofs.borrow().clone());
            assert!(!proofs[0].tail && proofs[0].body);
            assert_eq!(proofs[0].auto_advances, [60.0, 20.0, 50.0]);
            assert_eq!(proofs[0].phase_advances, [60.0, 20.0, 50.0]);
            assert_eq!(proofs[0].selected_advances, [30.0, 50.0, 50.0]);
            let q = proofs[0].guards.unwrap().mean();
            assert!((q - 2.0 / 9.0).abs() < 4.0 * f64::EPSILON);

            for (widths, expected) in [
                (&[10.0, 50.0, 10.0][..], vec![vec![0, 1], vec![2]]),
                (
                    &[160.0, 30.0, 120.0, 50.0][..],
                    vec![vec![0], vec![1, 2], vec![3]],
                ),
            ] {
                let mut request = input::<S>(widths, &[], direction, crate::TextWrapStyle::Pretty);
                if widths[0] == 160.0 {
                    request.available_inline_extent = AvailableOf::definite(S::from_f64(170.0));
                }
                assert_eq!(groups(&run(request)), expected);
            }
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn actual_body_trials_use_one_fixed_reference_instead_of_ratcheting_terminal_guards() {
    fn lane<S: LayoutScalar>() {
        clear_proofs();
        let repaired = run(input::<S>(
            &[40.0, 20.0, 10.0, 10.0, 50.0],
            &[],
            Direction::Ltr,
            crate::TextWrapStyle::Pretty,
        ));
        let trials = BODY.with(|proofs| proofs.borrow().clone());
        let first = trials
            .iter()
            .position(|trial| {
                trial.advances == [40.0, 30.0, 60.0] && trial.admission == BodyAdmission::Improved
            })
            .expect("actual first admissible terminal60 trial");
        let second = trials
            .iter()
            .position(|trial| {
                trial.advances == [40.0, 40.0, 50.0] && trial.admission == BodyAdmission::Improved
            })
            .expect("actual lower-Q terminal50 trial");
        assert!(
            first < second,
            "must execute the order which distinguishes guard ratcheting"
        );
        assert_eq!(trials[first].fixed, trials[second].fixed);
        assert!(trials[second].candidate.mean() < trials[first].candidate.mean());
        assert_eq!(
            trials[second].candidate.admission(trials[first].candidate),
            BodyAdmission::TerminalAdvanceRegression
        );
        assert!(
            trials
                .iter()
                .all(|trial| trial.entry_source == 0 && trial.entry_block == 0.0)
        );
        assert_eq!(groups(&repaired), [vec![0], vec![1, 2, 3], vec![4]]);
    }
    lane::<f32>();
    lane::<f64>();
}

fn fullness_veto_rectangle<S: LayoutScalar>() -> TriangleTree<S> {
    let mut tree = TriangleTree::new(crate::TextWrapStyle::Pretty, false);
    tree.inner.insert_children(0, [1, 2, 3, 4, 5, 6]);
    tree.inner.insert_input(
        1,
        LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            size: Size::new(
                PreferredSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(20.0)).unwrap()),
                PreferredSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(50.0)).unwrap()),
            ),
            ..NodeInputOf::default()
        }),
    );
    for (id, width, height) in [
        (2, 40.0, 30.0),
        (3, 20.0, 30.0),
        (4, 10.0, 10.0),
        (5, 10.0, 10.0),
        (6, 50.0, 10.0),
    ] {
        tree.inner.insert_input(
            id,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::InlineBlock,
                direction: Direction::Ltr,
                vertical_align: VerticalAlign::Baseline,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                size: Size::new(
                    PreferredSizeOf::value(
                        crate::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                    ),
                    PreferredSizeOf::value(
                        crate::LengthPercentageOf::px(S::from_f64(height)).unwrap(),
                    ),
                ),
                ..NodeInputOf::default()
            }),
        );
    }
    tree
}

#[test]
fn real_fitter_trials_independently_veto_fixed_terminal_advance_fullness_and_pair_regressions() {
    fn lane<S: LayoutScalar>() {
        for case in 0..3 {
            clear_proofs();
            match case {
                0 => {
                    // Actual Auto100/100/10 first undergoes eligible Tail to
                    // 100/50/60. Body goal1 can genuinely revisit100/100/10.
                    let mut request = input::<S>(
                        &[100.0, 50.0, 50.0, 10.0],
                        &[],
                        Direction::Ltr,
                        crate::TextWrapStyle::Pretty,
                    );
                    request.available_inline_extent = AvailableOf::definite(S::from_f64(100.0));
                    run(request);
                }
                1 => {
                    // Genuine rectangle height50: the alternative places its
                    // same50-unit last atom below the float in an80-unit span.
                    fullness_veto_rectangle::<S>().compute().unwrap();
                }
                2 => {
                    // Auto60/30/50/50 has no Tail phase. Goal11/16 visits the
                    // same-count50/40/40/60 candidate before Body admission.
                    run(input::<S>(
                        &[50.0, 10.0, 30.0, 40.0, 10.0, 30.0, 10.0, 10.0],
                        &[],
                        Direction::Ltr,
                        crate::TextWrapStyle::Pretty,
                    ));
                }
                _ => unreachable!(),
            }
            let group = GROUPS.with(|proofs| proofs.borrow()[0].clone());
            let proofs = BODY.with(|proofs| proofs.borrow().clone());
            let (
                expected,
                candidate_capacities,
                phase,
                phase_capacities,
                sources,
                end,
                reason,
                reference_q,
                candidate_q,
            ) = match case {
                0 => (
                    vec![100.0, 100.0, 10.0],
                    vec![100.0; 3],
                    vec![100.0, 50.0, 60.0],
                    vec![100.0; 3],
                    vec![vec![0], vec![1, 2], vec![3]],
                    4,
                    BodyAdmission::TerminalAdvanceRegression,
                    1.0 / 8.0,
                    0.0,
                ),
                1 => (
                    vec![40.0, 40.0, 50.0],
                    vec![60.0, 60.0, 80.0],
                    vec![60.0, 20.0, 50.0],
                    vec![60.0; 3],
                    vec![vec![1], vec![2, 3, 4], vec![5]],
                    5,
                    BodyAdmission::TerminalFullnessRegression,
                    2.0 / 9.0,
                    1.0 / 9.0,
                ),
                2 => (
                    vec![50.0, 40.0, 40.0, 60.0],
                    vec![60.0; 4],
                    vec![60.0, 30.0, 50.0, 50.0],
                    vec![60.0; 4],
                    vec![vec![0], vec![1, 2], vec![3], vec![4, 5, 6, 7]],
                    8,
                    BodyAdmission::PairRegression,
                    5.0 / 54.0,
                    1.0 / 12.0,
                ),
                _ => unreachable!(),
            };
            let trial = proofs
                .iter()
                .find(|trial| trial.advances == expected)
                .expect("the complete legitimate candidate must actually reach Body admission");
            assert_eq!(group.phase_advances, phase);
            assert_eq!(group.phase_capacities, phase_capacities);
            assert_eq!(group.tail, case == 0);
            assert_eq!(group.auto_count, expected.len());
            assert_eq!(trial.fixed, group.guards.unwrap());
            assert_eq!(trial.entry_source, 0);
            assert_eq!(trial.entry_block, 0.0);
            assert_eq!(trial.terminal, surgeist_text::LineEnd::Paragraph);
            assert_eq!(trial.terminal_cursor, (end, end, false));
            assert_eq!(trial.sources, sources);
            assert_eq!(trial.capacities, candidate_capacities);
            assert_eq!(trial.advances.len(), group.auto_count);
            for (advance, capacity) in trial
                .advances
                .iter()
                .zip(&trial.capacities)
                .chain(group.phase_advances.iter().zip(&group.phase_capacities))
            {
                assert!(capacity.is_finite() && *capacity > 0.0);
                assert!(advance.is_finite() && *advance >= 0.0 && advance <= capacity);
            }
            // Q is the real owner's score from the complete authenticated
            // trial, compared to its ACTUAL fixed post-Tail phase record.
            assert!(trial.candidate.mean() < trial.fixed.mean());
            assert!((trial.fixed.mean() - reference_q).abs() <= 4.0 * f64::EPSILON);
            assert!((trial.candidate.mean() - candidate_q).abs() <= 4.0 * f64::EPSILON);
            let last = expected.len() - 1;
            let reference_tail = phase[last] / phase_capacities[last];
            let candidate_tail = expected[last] / candidate_capacities[last];
            if case > 0 {
                assert!(
                    expected[last] >= phase[last],
                    "earlier advance guard passes"
                );
            }
            if case == 2 {
                assert!(
                    candidate_tail >= reference_tail,
                    "earlier fullness guard passes"
                );
                assert!(
                    (expected[last - 1] / candidate_capacities[last - 1]).min(candidate_tail)
                        < (phase[last - 1] / phase_capacities[last - 1]).min(reference_tail)
                );
            }
            if case == 1 {
                assert!(candidate_tail < reference_tail);
                assert!(
                    (expected[last - 1] / candidate_capacities[last - 1]).min(candidate_tail)
                        >= (phase[last - 1] / phase_capacities[last - 1]).min(reference_tail)
                );
            }
            assert_eq!(trial.admission, reason);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

fn rectangle<S: LayoutScalar>() -> TriangleTree<S> {
    let mut tree = TriangleTree::new(crate::TextWrapStyle::Pretty, false);
    tree.inner.insert_children(0, [1, 2, 3, 4, 5, 6, 7, 8, 9]);
    tree.inner.insert_input(
        1,
        LayoutInputOf::box_input(NodeInputOf {
            display: crate::Display::Block,
            float: crate::Float::Left,
            size: Size::new(
                PreferredSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(20.0)).unwrap()),
                PreferredSizeOf::value(crate::LengthPercentageOf::px(S::from_f64(60.0)).unwrap()),
            ),
            ..NodeInputOf::default()
        }),
    );
    for (id, width, height) in [
        (2, 30.0, 10.0),
        (3, 30.0, 30.0),
        (4, 20.0, 30.0),
        (5, 50.0, 10.0),
        (7, 30.0, 10.0),
        (8, 30.0, 10.0),
        (9, 10.0, 10.0),
    ] {
        tree.inner.insert_input(
            id,
            LayoutInputOf::box_input(NodeInputOf {
                display: crate::Display::InlineBlock,
                direction: Direction::Ltr,
                vertical_align: VerticalAlign::Baseline,
                atomic_inline_participation: Some(AtomicInlineParticipation::new(
                    BidiLevel::try_new(0).unwrap(),
                    InlineBreakOpportunity::Allowed,
                )),
                size: Size::new(
                    PreferredSizeOf::value(
                        crate::LengthPercentageOf::px(S::from_f64(width)).unwrap(),
                    ),
                    PreferredSizeOf::value(
                        crate::LengthPercentageOf::px(S::from_f64(height)).unwrap(),
                    ),
                ),
                ..NodeInputOf::default()
            }),
        );
    }
    tree.inner.insert_input(
        6,
        LayoutInputOf::line_break(
            crate::LineBreakInputOf::new().with_metrics(
                InlineMetricsOf::try_new(
                    crate::InlineBaselineContext::Physical,
                    S::from_f64(10.0),
                    S::from_f64(10.0),
                )
                .unwrap(),
            ),
        ),
    );
    tree
}

#[test]
fn current_entry_pretty_reference_can_change_later_group_count_after_real_float_body() {
    fn lane<S: LayoutScalar>() {
        clear_proofs();
        let tree = rectangle::<S>();
        let output = tree.compute().unwrap();
        for entries in [output.unrounded_entries(), output.final_entries()] {
            let node = |id| {
                entries
                    .iter()
                    .find(|entry| entry.node() == id)
                    .unwrap()
                    .output()
            };
            assert_eq!(entries.len(), 10);
            assert_eq!(
                node(0).size,
                Size::new(S::from_f64(80.0), S::from_f64(70.0))
            );
            assert_eq!(node(1).location, Point::ZERO);
            assert_eq!(
                node(1).size,
                Size::new(S::from_f64(20.0), S::from_f64(60.0))
            );
            for (id, x, y) in [
                (2, 20.0, 0.0),
                (3, 20.0, 10.0),
                (4, 50.0, 10.0),
                (5, 20.0, 40.0),
                (6, 70.0, 50.0),
                (7, 20.0, 50.0),
                (8, 0.0, 60.0),
                (9, 30.0, 60.0),
            ] {
                assert_eq!(
                    node(id).location,
                    Point::new(S::from_f64(x), S::from_f64(y))
                );
                assert_eq!(
                    node(id).source_index,
                    crate::SourceIndex::new(id as usize - 1)
                );
            }
        }
        let proof = GROUPS.with(|proofs| proofs.borrow().clone());
        assert_eq!(proof.len(), 2);
        assert_eq!(proof[0].entry_block, 0.0);
        assert_eq!(proof[0].auto_count, 3);
        assert_eq!(proof[0].auto_advances, [60.0, 20.0, 50.0]);
        assert!(!proof[0].tail && proof[0].body);
        assert_eq!(proof[0].outgoing_block, 50.0);
        assert_eq!(proof[1].entry_block, 50.0);
        assert_eq!(proof[1].auto_count, 2);
        assert_eq!(proof[1].auto_advances, [60.0, 10.0]);
        assert!(proof[1].tail);
        assert_eq!(proof[1].phase_advances, [30.0, 40.0]);
        assert_eq!(proof[1].selected_advances, [30.0, 40.0]);
        assert_eq!(proof[1].outgoing_block, 70.0);
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn pretty_has_no_ten_line_cap_and_preserves_real_forced_phantom_and_optional_domain() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let widths = [60.0; 9]
                .into_iter()
                .chain([30.0, 30.0, 20.0, 50.0])
                .collect::<Vec<_>>();
            let auto = run(input::<S>(
                &widths,
                &[],
                direction,
                crate::TextWrapStyle::Auto,
            ));
            let pretty = run(input::<S>(
                &widths,
                &[],
                direction,
                crate::TextWrapStyle::Pretty,
            ));
            assert_eq!(pretty.line_bands.len(), 12);
            assert_eq!(&pretty.atomics[..9], &auto.atomics[..9]);
            assert_eq!(&groups(&pretty)[9..], &[vec![9], vec![10, 11], vec![12]]);

            let forced = run(input::<S>(
                &[30.0, 30.0, 20.0, 50.0],
                &[3],
                direction,
                crate::TextWrapStyle::Pretty,
            ));
            assert_eq!(groups(&forced), [vec![0], vec![1, 2], vec![3]]);
            assert_eq!(
                forced.lines.last().unwrap().end,
                surgeist_text::LineEnd::Forced
            );
            assert_eq!(forced.line_bands.len(), 3);
            let overflow = run(input::<S>(
                &[70.0, 10.0],
                &[],
                direction,
                crate::TextWrapStyle::Pretty,
            ));
            let auto = run(input::<S>(
                &[70.0, 10.0],
                &[],
                direction,
                crate::TextWrapStyle::Auto,
            ));
            assert_eq!(overflow, auto);
            let mut zero = input::<S>(&[0.0], &[], direction, crate::TextWrapStyle::Pretty);
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut zero.participants[0] else {
                unreachable!()
            };
            item.size = Size::ZERO;
            item.content_size = Size::ZERO;
            item.first_baseline = Some(S::ZERO);
            let zero = run(zero);
            assert_eq!(zero.atomics.len(), 1);
            assert_eq!(zero.line_bands.len(), 1);
            assert_eq!(zero.block_extent, S::ZERO);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn pretty_does_not_replay_unrelated_prefix_groups_for_body_targets() {
    fn work(groups_count: usize, style: crate::TextWrapStyle) -> (usize, Vec<usize>) {
        let widths = [60.0]
            .into_iter()
            .cycle()
            .take(groups_count)
            .chain([30.0, 30.0, 20.0, 50.0])
            .collect::<Vec<_>>();
        let forced = (0..groups_count).collect::<Vec<_>>();
        let mut visits = vec![0; groups_count];
        let request = input::<f64>(&widths, &forced, Direction::Ltr, style);
        let result = layout_mixed_inline_run_with_formatted_observation(
            request,
            |start, _| {
                let index = (start / 10.0) as usize;
                if index < visits.len() {
                    visits[index] += 1;
                }
                LogicalLineBandQueryResultOf {
                    inline_start: 0.0,
                    inline_end: 60.0,
                    next_transition: None,
                }
            },
            |block, _| block,
            true,
            unavailable_atomic_receiver,
        )
        .unwrap();
        (result.line_bands.len(), visits)
    }
    for count in [8, 16] {
        let (auto_rows, auto) = work(count, crate::TextWrapStyle::Auto);
        let (pretty_rows, pretty) = work(count, crate::TextWrapStyle::Pretty);
        assert_eq!(auto_rows, pretty_rows);
        // The declared ordinary establishment and final validation each visit
        // these real one-line Forced groups once; later target portfolios do not.
        assert_eq!(
            pretty,
            auto.iter().map(|visits| 2 * visits).collect::<Vec<_>>()
        );
    }
}

#[test]
fn pretty_triangle_differs_from_real_balance_and_keeps_losing_trial_provider_error() {
    fn lane<S: LayoutScalar>() {
        let auto = TriangleTree::<S>::new(crate::TextWrapStyle::Auto, false)
            .compute()
            .unwrap();
        let pretty = TriangleTree::<S>::new(crate::TextWrapStyle::Pretty, false)
            .compute()
            .unwrap();
        let balance = TriangleTree::<S>::new(crate::TextWrapStyle::Balance, false)
            .compute()
            .unwrap();
        assert_eq!(auto.unrounded_entries(), pretty.unrounded_entries());
        assert_eq!(auto.final_entries(), pretty.final_entries());
        assert_ne!(auto.final_entries(), balance.final_entries());
        let prior = auto.final_entries().to_vec();
        let failure = TriangleTree::<S>::new(crate::TextWrapStyle::Pretty, true)
            .compute()
            .unwrap_err();
        assert_eq!(
            failure.operation(),
            crate::LayoutOperation::FloatExclusionQuery
        );
        assert_eq!(
            failure.site(),
            crate::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert!(matches!(
            failure.kind(),
            crate::LayoutErrorKindOf::Measurement(ContourFailure::CrossesBottomFromWithin)
        ));
        assert_eq!(auto.final_entries(), prior);
    }
    lane::<f32>();
    lane::<f64>();
}
