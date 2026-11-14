//! Terminal-pair preference over the existing native group walk.
use super::*;
use crate::inline_wrapping::quality::NaturalLine;
use crate::inline_wrapping::tail::{
    TailAdmission, TailGoal, TailPair, TailSelection, TailTrial, TailWinner,
};

struct NumericTailReference<S: LayoutScalar> {
    terminal: NumericGroupTerminal<S>,
    count: usize,
    prefix: Vec<SelectedInlineLineOf<S>>,
    frontier: Option<NumericFlowState<S>>,
}

#[derive(Clone, PartialEq)]
struct NumericTailBasis<S: LayoutScalar> {
    group: NumericGroupBasis<S>,
    frontier: NumericFlowState<S>,
    formatted: Vec<crate::text_layout::formatted::NumericFormattedLine>,
}

struct NumericTailTarget<S: LayoutScalar> {
    entry: NumericGroupEntry<S>,
    reference: NumericTailReference<S>,
    pair: TailPair,
    auto: NumericTailBasis<S>,
}

pub(super) fn line_measure<S: LayoutScalar>(line: &SelectedInlineLineOf<S>) -> Option<NaturalLine> {
    NaturalLine::from_components(
        (line.band.inline_end - line.band.inline_start).to_f64(),
        line.indent.to_f64(),
        line.used_inline_extent.to_f64(),
    )
}

fn final_pair<S: LayoutScalar>(lines: &[SelectedInlineLineOf<S>]) -> Option<TailPair> {
    let mut formatted = lines.iter().rev().filter(|line| numeric_is_formatted(line));
    let last = line_measure(formatted.next()?)?;
    let penultimate = line_measure(formatted.next()?)?;
    Some(TailPair::new(penultimate, last))
}

fn trial_control<S: LayoutScalar>(
    reference: &NumericTailReference<S>,
    goal: Option<TailGoal>,
) -> NumericWalkControl {
    NumericWalkControl {
        goal: goal.map(LineQualityGoal::Tail),
        goal_start: reference.count - 2,
        goal_end: None,
        observe_frontier: Some(reference.count - 2),
        stop_at_group: true,
        max_formatted: Some(reference.count),
    }
}

fn finish_trial<S: LayoutScalar>(
    sequence: &NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    reference: &NumericTailReference<S>,
    outcome: NumericWalkOutcome,
) -> Option<TailTrial<NumericTailBasis<S>>> {
    if outcome != NumericWalkOutcome::GroupComplete {
        return None;
    }
    let lines = &sequence.selected_lines[entry.selected_len..];
    let last = lines.last()?;
    if NumericGroupTerminal::of(last) != reference.terminal
        || lines
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .count()
            != reference.count
        || lines.get(..reference.prefix.len()) != Some(reference.prefix.as_slice())
    {
        return None;
    }
    let frontier = sequence.tail_frontier?;
    if reference
        .frontier
        .is_some_and(|expected| expected != frontier)
    {
        return None;
    }
    let pair = final_pair(lines)?;
    let formatted = sequence
        .formatted_lines
        .as_ref()
        .map_or_else(Vec::new, |lines| lines[entry.formatted_len..].to_vec());
    Some(TailTrial {
        pair,
        value: NumericTailBasis {
            group: NumericGroupBasis {
                lines: lines.to_vec(),
                outgoing: sequence.state,
            },
            frontier,
            formatted,
        },
    })
}

struct NumericTailPreparation<S: LayoutScalar> {
    entry: NumericGroupEntry<S>,
    reference: NumericTailReference<S>,
    pair: TailPair,
    ordinary: NumericGroupBasis<S>,
}

fn prepare_group_reference<S: LayoutScalar>(
    sequence: &NumericSequence<S>,
    entry: NumericGroupEntry<S>,
) -> Option<NumericTailPreparation<S>> {
    let lines = &sequence.selected_lines[entry.selected_len..];
    let last = lines.last()?;
    // A trailing nonformatted Paragraph receipt is NOT permission to target
    // the preceding Forced occurrence or to walk backward across that receipt.
    if last.ending != surgeist_text::LineEnd::Paragraph || !numeric_is_formatted(last) {
        return None;
    }
    let formatted = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| numeric_is_formatted(line).then_some(i))
        .collect::<Vec<_>>();
    if formatted.len() < 2 {
        return None;
    }
    let pair = final_pair(lines)?;
    if !pair.eligible_auto() {
        return None;
    }
    let ordinary = NumericGroupBasis {
        lines: lines.to_vec(),
        outgoing: sequence.state,
    };
    let reference = NumericTailReference {
        terminal: NumericGroupTerminal::of(last),
        count: formatted.len(),
        prefix: lines[..formatted[formatted.len() - 2]].to_vec(),
        frontier: None,
    };
    Some(NumericTailPreparation {
        entry,
        reference,
        pair,
        ordinary,
    })
}

fn restore_program<S: LayoutScalar>(
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    basis: &NumericTailBasis<S>,
) {
    sequence.restore(entry);
    sequence
        .selected_lines
        .extend_from_slice(&basis.group.lines);
    if let Some(formatted) = &mut sequence.formatted_lines {
        formatted.extend_from_slice(&basis.formatted);
    }
    sequence.state = basis.group.outgoing;
    sequence.tail_frontier = Some(basis.frontier);
}

mod continuation;
pub(super) use continuation::NumericTail;

/// Pretty retains its synchronous group ownership, draining the same Tail owner.
pub(super) fn repair_group_from_auto<S, B, C, E>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<TailGoal>, NumericRunError<E>>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> Result<LogicalLineBandQueryResultOf<S>, E>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    NumericTail::from_auto(sequence, entry, None).drain(request, sequence, boundaries, band, clear)
}

pub(super) fn replay_group_from_auto<S, B, C, E>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    phase: (NumericGroupEntry<S>, TailGoal),
    band: &mut B,
    clear: &mut C,
) -> Result<bool, NumericRunError<E>>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> Result<LogicalLineBandQueryResultOf<S>, E>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    NumericTail::from_auto(sequence, phase.0, Some(phase.1))
        .drain(request, sequence, boundaries, band, clear)
        .map(|goal| goal.is_some())
}

#[cfg(test)]
pub(super) mod tests;
