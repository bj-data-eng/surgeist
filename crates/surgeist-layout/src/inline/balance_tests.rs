use super::*;
use crate::TextWrapStyle;

fn input<S: LayoutScalar>(
    widths: &[f64],
    mandatory: &[usize],
    direction: Direction,
    style: TextWrapStyle,
) -> MixedInlineRunInputOf<S> {
    MixedInlineRunInputOf {
        inline_baseline_context: crate::InlineBaselineContext::Physical,
        groups: crate::group_alignment::Snapshot::default(),
        text_wrap_style: style,
        line_box_context: crate::line_box::LineBoxContext::new(
            crate::LineBoxDirectionPolicy::Normal,
            direction,
        ),
        observe_tail: true,
        wrapping: None,
        available_inline_extent: AvailableOf::definite(S::from_f64(60.0)),
        flow_axes: FlowAxes::new(WritingMode::HorizontalTb, direction),
        text_align: TextAlign::Start,
        text_align_last: crate::TextAlignLast::Auto,
        indent: crate::text_indent::ResolvedTextIndent::ZERO,
        indent_state: crate::text_indent::IndentLineState::default(),
        participants: widths
            .iter()
            .copied()
            .enumerate()
            .map(|(source_index, width)| MixedInlineParticipantOf::Atomic {
                item: AtomicInlineBoxParticipant {
                    baseline_reference: crate::InlineBaselineContext::Physical,
                    source_index,
                    size: Size::new(S::from_f64(width), S::from_f64(10.0)),
                    content_size: Size::new(S::from_f64(width), S::from_f64(10.0)),
                    margin: Edges::ZERO,
                    padding: Edges::ZERO,
                    border: Edges::ZERO,
                    scrollbar_size: Size::ZERO,
                    first_baseline: Some(S::from_f64(8.0)),
                    alignment: InlineControlAlignment::Baseline,
                },
                participation: AtomicInlineParticipation::new(
                    BidiLevel::try_new(u8::from(direction == Direction::Rtl)).unwrap(),
                    if mandatory.contains(&source_index) {
                        InlineBreakOpportunity::Mandatory
                    } else {
                        InlineBreakOpportunity::Allowed
                    },
                ),
            })
            .collect(),
    }
}

fn groups<S: LayoutScalar>(report: &MixedInlineRunReportOf<S>) -> Vec<Vec<usize>> {
    (0..report.line_bands.len())
        .map(|line| {
            report
                .atomics
                .iter()
                .filter(|atomic| atomic.line_index == line)
                .map(|atomic| atomic.item.source_index)
                .collect()
        })
        .collect()
}

#[test]
fn numeric_balance_uses_real_capacity_without_a_wrapping_context() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let auto = layout_mixed_inline_run(input::<S>(
                &[30.0, 30.0, 10.0],
                &[],
                direction,
                TextWrapStyle::Auto,
            ))
            .unwrap();
            let balance = layout_mixed_inline_run(input::<S>(
                &[30.0, 30.0, 10.0],
                &[],
                direction,
                TextWrapStyle::Balance,
            ))
            .unwrap();
            assert_eq!(groups(&auto), [vec![0, 1], vec![2]]);
            assert_eq!(groups(&balance), [vec![0], vec![1, 2]]);
            assert_eq!(auto.inline_extent, S::from_f64(60.0));
            assert_eq!(balance.inline_extent, S::from_f64(40.0));
            assert_eq!(balance.block_extent, S::from_f64(20.0));
            assert_eq!(balance.first_baseline, Some(S::from_f64(8.0)));
            assert_eq!(balance.last_baseline, Some(S::from_f64(18.0)));
            assert!(balance.line_bands.iter().all(|band| {
                band.inline_start == S::ZERO && band.inline_end == S::from_f64(60.0)
            }));
            assert_eq!(
                balance
                    .lines
                    .iter()
                    .map(|line| line.end)
                    .collect::<Vec<_>>(),
                [
                    surgeist_text::LineEnd::Soft,
                    surgeist_text::LineEnd::Paragraph
                ]
            );
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_balance_keeps_forced_groups_intrinsics_and_real_zero_atomics() {
    fn lane<S: LayoutScalar>() {
        for direction in [Direction::Ltr, Direction::Rtl] {
            let report = layout_mixed_inline_run(input::<S>(
                &[30.0, 30.0, 10.0, 30.0, 30.0, 10.0],
                &[2],
                direction,
                TextWrapStyle::Balance,
            ))
            .unwrap();
            assert_eq!(groups(&report), [vec![0], vec![1, 2], vec![3], vec![4, 5]]);
            assert_eq!(report.block_extent, S::from_f64(40.0));
            assert_eq!(
                report.lines.iter().map(|line| line.end).collect::<Vec<_>>(),
                [
                    surgeist_text::LineEnd::Soft,
                    surgeist_text::LineEnd::Forced,
                    surgeist_text::LineEnd::Soft,
                    surgeist_text::LineEnd::Paragraph,
                ]
            );
            for available in [AvailableOf::MIN_CONTENT, AvailableOf::MAX_CONTENT] {
                let mut auto = input::<S>(&[30.0, 30.0, 10.0], &[], direction, TextWrapStyle::Auto);
                auto.available_inline_extent = available;
                let mut balance = auto.clone();
                balance.text_wrap_style = TextWrapStyle::Balance;
                assert_eq!(
                    layout_mixed_inline_run(auto).unwrap(),
                    layout_mixed_inline_run(balance).unwrap()
                );
            }
            let mut zero = input::<S>(&[0.0], &[], direction, TextWrapStyle::Balance);
            let MixedInlineParticipantOf::Atomic { item, .. } = &mut zero.participants[0] else {
                unreachable!()
            };
            item.size = Size::ZERO;
            item.content_size = Size::ZERO;
            item.first_baseline = Some(S::ZERO);
            let zero = layout_mixed_inline_run(zero).unwrap();
            assert_eq!(zero.atomics.len(), 1);
            assert_eq!(zero.line_bands.len(), 1);
            assert_eq!(zero.first_baseline, Some(S::ZERO));
            assert_eq!(zero.block_extent, S::ZERO);
        }
    }
    lane::<f32>();
    lane::<f64>();
}

#[test]
fn numeric_balance_does_not_replay_a_long_group_for_other_groups_targets() {
    let widths = [30.0, 30.0, 10.0]
        .into_iter()
        .chain([60.0; 11])
        .chain([30.0, 30.0, 10.0])
        .collect::<Vec<_>>();
    let queries = std::cell::RefCell::new(Vec::new());
    let report = layout_mixed_inline_run_with_band_source(
        input::<f64>(&widths, &[2, 13], Direction::Ltr, TextWrapStyle::Balance),
        |start, end| {
            queries.borrow_mut().push((start, end));
            LogicalLineBandQueryResultOf {
                inline_start: 0.0,
                inline_end: 60.0,
                next_transition: None,
            }
        },
        |block, _| block,
    )
    .unwrap();
    assert_eq!(report.line_bands.len(), 15);
    assert_eq!(report.block_extent, 150.0);
    assert_eq!(groups(&report)[..2], [vec![0], vec![1, 2]]);
    assert_eq!(groups(&report)[13..], [vec![14], vec![15, 16]]);
    // The eleven-line group is ordinary Auto. Its seven middle rows cannot
    // occur in a short group's <=3-line rejected trial. Each is queried once
    // in original Auto, once in local Auto and once in final whole replay.
    // The provider receives actual geometric ranges, with no source-aware or
    // candidate-aware answer and no invented clearance transition.
    let middle = queries
        .borrow()
        .iter()
        .filter(|(start, _)| (40.0..=100.0).contains(start))
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(middle.len(), 21);
    assert!(middle.iter().all(|(start, end)| end - start == 10.0));
}
