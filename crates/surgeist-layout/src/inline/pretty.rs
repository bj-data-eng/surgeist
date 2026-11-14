//! Streaming native group references and replay for Pretty.
use super::*;
use crate::inline_wrapping::body::{BodyGoal, BodyScore, BodySelection, BodyTrial, BodyWinner};
use crate::inline_wrapping::tail::TailGoal;

#[derive(Clone, PartialEq)]
struct GroupBasis<S: LayoutScalar> {
    group: NumericGroupBasis<S>,
    formatted: Vec<crate::text_layout::formatted::NumericFormattedLine>,
}

#[derive(Clone, Copy)]
struct GroupReference<S: LayoutScalar> {
    terminal: NumericGroupTerminal<S>,
    count: usize,
}

/// Invocation-local native custody: real entry and ordinary reference, the
/// fixed post-Tail guard proof, phase goals and selected outgoing program.
/// The normalized Pretty mode stays in the caller's actual input/cache key.
struct PrettyGroupProgram<S: LayoutScalar> {
    entry: NumericGroupEntry<S>,
    ordinary: GroupBasis<S>,
    reference: GroupReference<S>,
    tail_goal: Option<TailGoal>,
    phase: GroupBasis<S>,
    guards: Option<BodyScore>,
    body_goal: Option<BodyGoal>,
    selected: GroupBasis<S>,
}

fn basis<S: LayoutScalar>(
    sequence: &NumericSequence<S>,
    entry: NumericGroupEntry<S>,
) -> GroupBasis<S> {
    GroupBasis {
        group: NumericGroupBasis {
            lines: sequence.selected_lines[entry.selected_len..].to_vec(),
            outgoing: sequence.state,
        },
        formatted: sequence
            .formatted_lines
            .as_ref()
            .map_or_else(Vec::new, |lines| lines[entry.formatted_len..].to_vec()),
    }
}

fn install<S: LayoutScalar>(
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    selected: &GroupBasis<S>,
) {
    sequence.restore(entry);
    sequence
        .selected_lines
        .extend_from_slice(&selected.group.lines);
    if let Some(formatted) = &mut sequence.formatted_lines {
        formatted.extend_from_slice(&selected.formatted);
    }
    sequence.state = selected.group.outgoing;
}

fn score<S: LayoutScalar>(basis: &GroupBasis<S>) -> Option<BodyScore> {
    BodyScore::from_lines(
        basis
            .group
            .lines
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .map(tail::line_measure),
    )
}

fn reference<S: LayoutScalar>(basis: &GroupBasis<S>) -> Option<GroupReference<S>> {
    Some(GroupReference {
        terminal: NumericGroupTerminal::of(basis.group.lines.last()?),
        count: basis
            .group
            .lines
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .count(),
    })
}

fn finish_ordinary_group<S: LayoutScalar>(
    sequence: &NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    outcome: NumericWalkOutcome,
) -> Result<Option<GroupBasis<S>>, NumericWrappingError> {
    if sequence.selected_lines.len() == entry.selected_len {
        return if outcome == NumericWalkOutcome::Exhausted {
            Ok(None)
        } else {
            Err(crate::inline_wrapping::NumericWrappingError::InvalidPrettyBasis)
        };
    }
    if outcome != NumericWalkOutcome::GroupComplete {
        return Err(crate::inline_wrapping::NumericWrappingError::InvalidPrettyBasis);
    }
    Ok(Some(basis(sequence, entry)))
}

fn body_control<S: LayoutScalar>(
    reference: GroupReference<S>,
    goal: BodyGoal,
) -> NumericWalkControl {
    let mut control =
        NumericWalkControl::group(Some(LineQualityGoal::Body(goal)), Some(reference.count));
    // The actual terminal formatted line is ordinary, not a Body score term.
    control.goal_end = Some(reference.count.saturating_sub(1));
    control
}

fn finish_body_trial<S: LayoutScalar>(
    sequence: &NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    reference: GroupReference<S>,
    outcome: NumericWalkOutcome,
) -> Option<BodyTrial<GroupBasis<S>>> {
    if outcome != NumericWalkOutcome::GroupComplete {
        return None;
    }
    let candidate = basis(sequence, entry);
    let actual = self::reference(&candidate)?;
    if actual.terminal != reference.terminal || actual.count != reference.count {
        return None;
    }
    let score = score(&candidate)?;
    Some(BodyTrial {
        score,
        value: candidate,
    })
}

mod continuation;
pub(super) use continuation::NumericPretty;

#[cfg(test)]
mod tests;
