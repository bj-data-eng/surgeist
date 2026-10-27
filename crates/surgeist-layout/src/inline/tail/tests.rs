use super::*;
use crate::inline::balance_tests::{groups, input};
use crate::inline_wrapping::tail::TailAdmission;
use crate::test_support::layout_tree::PublicLayoutTreeOf;
use crate::{LayoutInputOf, LayoutTree, NodeInputOf, PreferredSizeOf, Traverse};
use std::cell::RefCell;

#[derive(Clone)]
struct TrialProof {
    sources: Vec<Vec<usize>>,
    cursors: Vec<InlineSourceCursor>,
    bands: Vec<[f64; 4]>,
    advances: Vec<f64>,
    endings: Vec<surgeist_text::LineEnd>,
    admission: TailAdmission,
    frozen_frontier: bool,
}

thread_local! {
    static TRIALS: RefCell<Vec<TrialProof>> = const { RefCell::new(Vec::new()) };
}

pub(super) fn observe<S: LayoutScalar>(
    reference_pair: TailPair,
    reference: &NumericTailReference<S>,
    candidate: &TailTrial<NumericTailBasis<S>>,
) {
    // Observation occurs only AFTER the real native trial has authenticated
    // count, terminal, complete prefix, frontier and optional measure domain.
    // The pair sent to admission is that actual owner's pair, never a fixture
    // tuple. This is cfg(test) instrumentation, not an alternate formatter.
    let lines = &candidate.value.group.lines;
    let sources = lines
        .iter()
        .map(|line| {
            line.units
                .iter()
                .map(|unit| match unit.participant {
                    MixedInlineParticipantOf::Atomic { item, .. } => item.source_index,
                    MixedInlineParticipantOf::Boundary(control) => control.source_index(),
                    MixedInlineParticipantOf::ForcedLineBreak(control) => control.source_index(),
                })
                .collect()
        })
        .collect();
    TRIALS.with(|trials| {
        trials.borrow_mut().push(TrialProof {
            sources,
            cursors: lines.iter().map(|line| line.next_cursor).collect(),
            bands: lines
                .iter()
                .map(|line| {
                    [
                        line.band.inline_start.to_f64(),
                        line.band.inline_end.to_f64(),
                        line.band.block_start.to_f64(),
                        line.band.block_end.to_f64(),
                    ]
                })
                .collect(),
            advances: lines
                .iter()
                .map(|line| line.used_inline_extent.to_f64())
                .collect(),
            endings: lines.iter().map(|line| line.ending).collect(),
            admission: candidate.pair.admission(reference_pair),
            frozen_frontier: reference.frontier == Some(candidate.value.frontier),
        })
    });
}

fn clear_trials() {
    TRIALS.with(|trials| trials.borrow_mut().clear());
}

fn ordinary<S: LayoutScalar>(input: MixedInlineRunInputOf<S>) -> MixedInlineRunReportOf<S> {
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
fn terminal_pair_repairs_without_changing_real_earlier_prefix_or_crossing_forced() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let auto = ordinary(input::<S>(
                &[30.0, 30.0, 10.0, 30.0, 30.0, 10.0],
                &[2],
                direction,
                crate::TextWrapStyle::Auto,
            ));
            let repaired = ordinary(input::<S>(
                &[30.0, 30.0, 10.0, 30.0, 30.0, 10.0],
                &[2],
                direction,
                crate::TextWrapStyle::AvoidShortLastLine,
            ));
            assert_eq!(groups(&auto), [vec![0, 1], vec![2], vec![3, 4], vec![5]]);
            assert_eq!(
                groups(&repaired),
                [vec![0, 1], vec![2], vec![3], vec![4, 5]]
            );
            assert_eq!(&repaired.atomics[..3], &auto.atomics[..3]);

            // One final Paragraph line cannot borrow from an earlier Forced
            // group; an authentic trailing Forced completion cannot be walked
            // backward across its nonformatted Paragraph receipt either.
            for (widths, forced) in [
                (&[30.0, 30.0, 10.0, 10.0][..], &[2][..]),
                (&[30.0, 30.0, 10.0][..], &[2][..]),
            ] {
                let auto = ordinary(input::<S>(
                    widths,
                    forced,
                    direction,
                    crate::TextWrapStyle::Auto,
                ));
                let repair = ordinary(input::<S>(
                    widths,
                    forced,
                    direction,
                    crate::TextWrapStyle::AvoidShortLastLine,
                ));
                assert_eq!(repair, auto);
            }

            // More than ten final-group formatted lines remain eligible. The
            // eleven accepted prefix rows retain their complete atomic output.
            let widths = [60.0; 11]
                .into_iter()
                .chain([30.0, 30.0, 10.0])
                .collect::<Vec<_>>();
            let auto = ordinary(input::<S>(
                &widths,
                &[],
                direction,
                crate::TextWrapStyle::Auto,
            ));
            let repair = ordinary(input::<S>(
                &widths,
                &[],
                direction,
                crate::TextWrapStyle::AvoidShortLastLine,
            ));
            assert_eq!(&repair.atomics[..11], &auto.atomics[..11]);
            assert_eq!(&groups(&repair)[11..], &[vec![11], vec![12, 13]]);
            assert_eq!(repair.line_bands.len(), 13);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn optional_tail_domain_retains_valid_overflow_and_real_zero_atomics() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let auto = ordinary(input::<S>(
                &[70.0, 10.0],
                &[],
                direction,
                crate::TextWrapStyle::Auto,
            ));
            let repair = ordinary(input::<S>(
                &[70.0, 10.0],
                &[],
                direction,
                crate::TextWrapStyle::AvoidShortLastLine,
            ));
            assert_eq!(repair, auto);
            let mut zero = input::<S>(
                &[0.0],
                &[],
                direction,
                crate::TextWrapStyle::AvoidShortLastLine,
            );
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut zero.participants[0] else {
                unreachable!()
            };
            item.size = Size::ZERO;
            item.content_size = Size::ZERO;
            item.first_baseline = Some(S::ZERO);
            let zero = ordinary(zero);
            assert_eq!(zero.atomics.len(), 1);
            assert_eq!(zero.line_bands.len(), 1);
            assert_eq!(zero.block_extent, S::ZERO);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn unrelated_forced_prefix_work_is_not_repeated_for_terminal_targets() {
    fn work(groups_count: usize, style: crate::TextWrapStyle) -> usize {
        let widths = (0..=groups_count)
            .flat_map(|_| [30.0, 30.0, 10.0])
            .collect::<Vec<_>>();
        let forced = (0..groups_count).map(|i| 3 * i + 2).collect::<Vec<_>>();
        reset_inline_candidate_scan_visits();
        let report = ordinary(input::<f64>(&widths, &forced, Direction::Ltr, style));
        assert_eq!(report.line_bands.len(), 2 * (groups_count + 1));
        inline_candidate_scan_visits()
    }
    let auto8 = work(8, crate::TextWrapStyle::Auto);
    let auto16 = work(16, crate::TextWrapStyle::Auto);
    let tail8 = work(8, crate::TextWrapStyle::AvoidShortLastLine);
    let tail16 = work(16, crate::TextWrapStyle::AvoidShortLastLine);
    assert!(tail8 > auto8, "real terminal trial work must execute");
    assert_eq!(tail16 - tail8, auto16 - auto8);
}

#[test]
fn equal_improved_real_programs_keep_the_first_incumbent() {
    clear_trials();
    let mut request = input::<f64>(
        &[50.0, 20.0, 10.0, 40.0],
        &[],
        Direction::Ltr,
        crate::TextWrapStyle::AvoidShortLastLine,
    );
    request.available_inline_extent = AvailableOf::definite(100.0);
    let repaired = ordinary(request);
    let trials = TRIALS.with(|trials| trials.borrow().clone());
    let first = trials
        .iter()
        .find(|trial| trial.admission == TailAdmission::Improved)
        .unwrap();
    assert_eq!(first.advances, [50.0, 70.0]);
    assert!(
        trials
            .iter()
            .any(|trial| trial.admission == TailAdmission::Improved
                && trial.advances == [70.0, 50.0])
    );
    assert_eq!(groups(&repaired), [vec![0], vec![1, 2, 3]]);
    assert!(trials.iter().all(|trial| trial.frozen_frontier));
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContourFailure {
    CrossesBottomFromWithin,
}

struct TriangleTree<S: LayoutScalar> {
    inner: PublicLayoutTreeOf<S>,
    fail: bool,
    queries: RefCell<Vec<crate::FloatExclusionQueryOf<S>>>,
}

impl<S: LayoutScalar> Traverse for TriangleTree<S> {
    type Node = u32;
    type Scalar = S;
    type Children<'a>
        = std::iter::Copied<std::slice::Iter<'a, u32>>
    where
        Self: 'a;
    fn children(&self, node: u32) -> Self::Children<'_> {
        Traverse::children(&self.inner, node)
    }
    fn child_count(&self, node: u32) -> usize {
        self.inner.child_count(node)
    }
    fn child(&self, node: u32, index: usize) -> u32 {
        self.inner.child(node, index)
    }
}

impl<S: LayoutScalar> LayoutTree for TriangleTree<S> {
    type MeasureError = ContourFailure;
    fn node_input(&self, node: u32) -> &NodeInputOf<S> {
        self.inner.node_input(node)
    }
    fn layout_input(&self, node: u32) -> crate::LayoutInputViewOf<'_, S> {
        self.inner.layout_input(node)
    }
    fn float_exclusion_interval(
        &self,
        node: u32,
        query: crate::FloatExclusionQueryOf<S>,
    ) -> Option<Result<Option<crate::FloatExclusionIntervalOf<S>>, Self::MeasureError>> {
        if node != 1 {
            return None;
        }
        assert_eq!(query.margin_box().origin(), Point::ZERO);
        assert_eq!(query.margin_box().size(), Size::splat(S::from_f64(20.0)));
        self.queries.borrow_mut().push(query);
        let lo = query.band_minimum();
        let hi = query.band_maximum();
        // A fixed geometric inability to evaluate a band crossing the bottom
        // from INSIDE the float. Ordinary0..30 and30..40 remain evaluable;
        // the real alternative continuation10..40 encounters this failure.
        if self.fail && lo > S::ZERO && lo < S::from_f64(20.0) && hi > S::from_f64(20.0) {
            return Some(Err(ContourFailure::CrossesBottomFromWithin));
        }
        if lo >= S::from_f64(20.0) || hi <= S::ZERO {
            return Some(Ok(None));
        }
        Some(Ok(crate::FloatExclusionIntervalOf::try_new(
            query,
            S::ZERO,
            hi.min(S::from_f64(20.0)),
        )
        .unwrap()))
    }
}

impl<S: LayoutScalar> TriangleTree<S> {
    fn new(style: crate::TextWrapStyle, fail: bool) -> Self {
        let mut inner = PublicLayoutTreeOf::new()
            .children(0, [1, 2, 3, 4])
            .style(
                0,
                NodeInputOf {
                    display: crate::Display::Block,
                    direction: Direction::Ltr,
                    text_align: TextAlign::Start,
                    text_align_last: crate::TextAlignLast::Start,
                    text_wrap_style: style,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(80.0)),
                        PreferredSizeOf::AUTO,
                    ),
                    ..NodeInputOf::default()
                },
            )
            .style(
                1,
                NodeInputOf {
                    display: crate::Display::Block,
                    float: crate::Float::Left,
                    float_exclusion: crate::FloatExclusion::Shape,
                    size: Size::new(
                        PreferredSizeOf::px(S::from_f64(20.0)),
                        PreferredSizeOf::px(S::from_f64(20.0)),
                    ),
                    ..NodeInputOf::default()
                },
            );
        for (id, width, height) in [(2, 60.0, 10.0), (3, 0.0, 30.0), (4, 10.0, 10.0)] {
            inner = inner.children(id, []).input(
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
                        PreferredSizeOf::px(S::from_f64(width)),
                        PreferredSizeOf::px(S::from_f64(height)),
                    ),
                    ..NodeInputOf::default()
                }),
            );
        }
        Self {
            inner,
            fail,
            queries: RefCell::new(Vec::new()),
        }
    }
    fn compute(
        &self,
    ) -> Result<crate::CompletedLayoutBatchOf<u32, S>, crate::LayoutErrorOf<u32, S, ContourFailure>>
    {
        crate::compute_layout(
            self,
            0,
            crate::LayoutRootRequestOf::viewport(Size::new(
                AvailableOf::definite(S::from_f64(80.0)),
                AvailableOf::MAX_CONTENT,
            ))
            .unwrap(),
        )
    }
}

#[test]
fn genuine_triangle_trial_reaches_and_fails_only_the_natural_tail_guard() {
    fn lane<S: LayoutScalar>() {
        clear_trials();
        let tree = TriangleTree::<S>::new(crate::TextWrapStyle::AvoidShortLastLine, false);
        let output = tree.compute().unwrap();
        let trials = TRIALS.with(|trials| trials.borrow().clone());
        let trial = trials
            .iter()
            .find(|trial| trial.sources == [vec![1], vec![2, 3]])
            .expect("the real triangle alternative must actually run through admission");
        assert_eq!(
            trial.bands,
            [[10.0, 80.0, 0.0, 10.0], [20.0, 80.0, 10.0, 40.0]]
        );
        assert_eq!(trial.advances, [60.0, 10.0]);
        assert_eq!(
            trial.endings,
            [
                surgeist_text::LineEnd::Soft,
                surgeist_text::LineEnd::Paragraph
            ]
        );
        assert_eq!(
            trial
                .cursors
                .iter()
                .map(|cursor| (cursor.source, cursor.descriptor, cursor.edge_consumed))
                .collect::<Vec<_>>(),
            [(1, 1, false), (3, 3, false)]
        );
        assert!(trial.frozen_frontier);
        // The ACTUAL shared controller checked last fullness and pair minimum
        // first; this reason proves both passed before natural10==10 rejected.
        assert_eq!(trial.admission, TailAdmission::NoNaturalTailGain);
        for entries in [output.unrounded_entries(), output.final_entries()] {
            let node = |id| {
                entries
                    .iter()
                    .find(|entry| entry.node() == id)
                    .unwrap()
                    .output()
            };
            assert_eq!(
                node(0).size,
                Size::new(S::from_f64(80.0), S::from_f64(40.0))
            );
            assert_eq!(node(1).location, Point::ZERO);
            assert_eq!(node(1).size, Size::splat(S::from_f64(20.0)));
            assert_eq!(
                node(2).location,
                Point::new(S::from_f64(20.0), S::from_f64(20.0))
            );
            assert_eq!(node(3).location, Point::new(S::from_f64(80.0), S::ZERO));
            assert_eq!(node(4).location, Point::new(S::ZERO, S::from_f64(30.0)));
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn actual_provider_failure_during_a_losing_tail_trial_keeps_original_context() {
    fn lane<S: LayoutScalar>() {
        let prior = TriangleTree::<S>::new(crate::TextWrapStyle::Auto, true)
            .compute()
            .unwrap();
        let snapshot = prior.final_entries().to_vec();
        let tree = TriangleTree::<S>::new(crate::TextWrapStyle::AvoidShortLastLine, true);
        let error = tree.compute().unwrap_err();
        assert!(
            tree.queries
                .borrow()
                .iter()
                .any(|query| query.band_minimum() == S::from_f64(10.0)
                    && query.band_maximum() == S::from_f64(40.0))
        );
        assert_eq!(
            error.site(),
            crate::LayoutErrorSiteOf::ContainerSubject {
                container: 0,
                subject: 1
            }
        );
        assert_eq!(
            error.operation(),
            crate::LayoutOperation::FloatExclusionQuery
        );
        assert!(matches!(
            error.kind(),
            crate::LayoutErrorKindOf::Measurement(ContourFailure::CrossesBottomFromWithin)
        ));
        assert_eq!(prior.final_entries(), snapshot);
    }
    lane::<f32>();
    lane::<f64>();
}
