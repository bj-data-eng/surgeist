//! Streaming native group references and replay for Pretty.
use super::*;
use crate::inline_wrapping::body::{BodyGoal, BodyScore, BodyTrial, choose_body};
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

fn ordinary_group<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<GroupBasis<S>>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let mut remaining = boundaries[entry.state.boundary_index..]
        .iter()
        .copied()
        .peekable();
    let outcome = walk_numeric_sequence(
        request,
        sequence,
        &mut remaining,
        NumericWalkControl::group(None, None),
        band,
        clear,
    )?;
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

fn body_trial<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    target: (NumericGroupEntry<S>, GroupReference<S>, BodyGoal),
    band: &mut B,
    clear: &mut C,
) -> Result<Option<BodyTrial<GroupBasis<S>>>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let (entry, reference, goal) = target;
    sequence.restore(entry);
    let mut remaining = boundaries[entry.state.boundary_index..]
        .iter()
        .copied()
        .peekable();
    let mut control =
        NumericWalkControl::group(Some(LineQualityGoal::Body(goal)), Some(reference.count));
    // The actual terminal formatted line is ordinary; earlier nonformatted
    // receipts do not consume a quality ordinal or become a score term.
    control.goal_end = Some(reference.count.saturating_sub(1));
    if walk_numeric_sequence(request, sequence, &mut remaining, control, band, clear)?
        != NumericWalkOutcome::GroupComplete
    {
        return Ok(None);
    }
    let candidate = basis(sequence, entry);
    let Some(actual) = self::reference(&candidate) else {
        return Ok(None);
    };
    if actual.terminal != reference.terminal || actual.count != reference.count {
        return Ok(None);
    }
    let Some(score) = score(&candidate) else {
        return Ok(None);
    };
    Ok(Some(BodyTrial {
        score,
        value: candidate,
    }))
}

pub(super) fn layout_pretty_sequence<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<(), crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let invalid = crate::inline_wrapping::NumericWrappingError::InvalidPrettyBasis;
    if request.input.text_wrap_style != crate::TextWrapStyle::Pretty {
        return Err(invalid);
    }
    let initial = sequence.checkpoint();
    let mut program = Vec::new();
    loop {
        // This is the PREVIOUS WINNER'S real outgoing frame. There is no
        // original whole-Auto count or suffix whose geometry could be stale.
        let entry = sequence.checkpoint();
        let Some(ordinary) = ordinary_group(request, boundaries, sequence, entry, band, clear)?
        else {
            break;
        };
        let reference = self::reference(&ordinary).ok_or(invalid)?;
        let tail_goal =
            tail::repair_group_from_auto(request, boundaries, sequence, entry, band, clear)?;
        let phase = basis(sequence, entry);
        let guards = score(&phase);
        let mut body_goal = None;
        let selected = if let Some(guards) = guards {
            // Every callback restores the SAME entry and uses these immutable
            // post-Tail guards. Only Q ranks already-admissible incumbents.
            let winner = choose_body(guards, |goal| {
                let trial = body_trial(
                    request,
                    boundaries,
                    sequence,
                    (entry, reference, goal),
                    band,
                    clear,
                )?;
                #[cfg(test)]
                if let Some(trial) = &trial {
                    tests::observe_trial(entry, guards, trial);
                }
                Ok::<_, crate::inline_wrapping::NumericWrappingError>(trial)
            })?;
            if let Some(winner) = winner {
                let replay = body_trial(
                    request,
                    boundaries,
                    sequence,
                    (entry, reference, winner.goal),
                    band,
                    clear,
                )?
                .ok_or(invalid)?;
                if replay.value != winner.trial.value || replay.score != winner.trial.score {
                    return Err(invalid);
                }
                body_goal = Some(winner.goal);
                replay.value
            } else {
                install(sequence, entry, &phase);
                phase.clone()
            }
        } else {
            phase.clone()
        };
        let group = PrettyGroupProgram {
            entry,
            ordinary,
            reference,
            tail_goal,
            phase,
            guards,
            body_goal,
            selected,
        };
        #[cfg(test)]
        tests::observe_group(&group);
        program.push(group);
    }

    // One complete native program validation. Each group reestablishes its
    // ordinary reference under the replayed selected prefix. Rejected work's
    // provider/dependency side channels are never erased or caught here.
    sequence.restore(initial);
    for group in &program {
        let entry = sequence.checkpoint();
        if entry.state != group.entry.state
            || entry.selected_len != group.entry.selected_len
            || entry.formatted_len != group.entry.formatted_len
        {
            return Err(invalid);
        }
        let actual =
            ordinary_group(request, boundaries, sequence, entry, band, clear)?.ok_or(invalid)?;
        if actual != group.ordinary {
            return Err(invalid);
        }
        if let Some(goal) = group.tail_goal
            && !tail::replay_group_from_auto(
                request,
                boundaries,
                sequence,
                (entry, goal),
                band,
                clear,
            )?
        {
            return Err(invalid);
        }
        let phase = basis(sequence, entry);
        if phase != group.phase || score(&phase) != group.guards {
            return Err(invalid);
        }
        if let Some(goal) = group.body_goal {
            let replay = body_trial(
                request,
                boundaries,
                sequence,
                (entry, group.reference, goal),
                band,
                clear,
            )?
            .ok_or(invalid)?;
            if replay.value != group.selected {
                return Err(invalid);
            }
        } else if phase != group.selected {
            return Err(invalid);
        }
    }
    let end = sequence.checkpoint();
    if ordinary_group(request, boundaries, sequence, end, band, clear)?.is_some() {
        return Err(invalid);
    }
    Ok(())
}

#[cfg(test)]
mod tests;
