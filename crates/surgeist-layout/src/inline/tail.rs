//! Terminal-pair preference over the existing native group walk.
use super::*;
use crate::inline_wrapping::quality::NaturalLine;
use crate::inline_wrapping::tail::{TailAdmission, TailGoal, TailPair, TailTrial, choose_tail};

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

fn trial<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    target: (&NumericGroupEntry<S>, &NumericTailReference<S>),
    goal: Option<TailGoal>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<TailTrial<NumericTailBasis<S>>>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let (entry, reference) = target;
    sequence.restore(*entry);
    let mut remaining = boundaries[entry.state.boundary_index..]
        .iter()
        .copied()
        .peekable();
    let outcome = walk_numeric_sequence(
        request,
        sequence,
        &mut remaining,
        NumericWalkControl {
            goal: goal.map(LineQualityGoal::Tail),
            goal_start: reference.count - 2,
            goal_end: None,
            observe_frontier: Some(reference.count - 2),
            stop_at_group: true,
            max_formatted: Some(reference.count),
        },
        band,
        clear,
    )?;
    if outcome != NumericWalkOutcome::GroupComplete {
        return Ok(None);
    }
    let lines = &sequence.selected_lines[entry.selected_len..];
    let Some(last) = lines.last() else {
        return Ok(None);
    };
    if NumericGroupTerminal::of(last) != reference.terminal
        || lines
            .iter()
            .filter(|line| numeric_is_formatted(line))
            .count()
            != reference.count
        || lines.get(..reference.prefix.len()) != Some(reference.prefix.as_slice())
    {
        return Ok(None);
    }
    let Some(frontier) = sequence.tail_frontier else {
        return Ok(None);
    };
    if reference
        .frontier
        .is_some_and(|expected| expected != frontier)
    {
        return Ok(None);
    }
    let Some(pair) = final_pair(lines) else {
        return Ok(None);
    };
    let formatted = sequence
        .formatted_lines
        .as_ref()
        .map_or_else(Vec::new, |lines| lines[entry.formatted_len..].to_vec());
    Ok(Some(TailTrial {
        pair,
        value: NumericTailBasis {
            group: NumericGroupBasis {
                lines: lines.to_vec(),
                outgoing: sequence.state,
            },
            frontier,
            formatted,
        },
    }))
}

fn prepare_target<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<NumericTailTarget<S>>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    // Genuine ordinary completion records each actual group entry once. No
    // target portfolio ever restarts these unrelated earlier groups.
    let mut final_entry = None;
    loop {
        let entry = sequence.checkpoint();
        let mut remaining = boundaries[entry.state.boundary_index..]
            .iter()
            .copied()
            .peekable();
        let outcome = walk_numeric_sequence(
            request,
            sequence,
            &mut remaining,
            NumericWalkControl::group(None, None),
            &mut *band,
            &mut *clear,
        )?;
        if sequence.selected_lines.len() > entry.selected_len {
            final_entry = Some(entry);
        }
        if outcome == NumericWalkOutcome::Exhausted {
            break;
        }
    }
    let Some(entry) = final_entry else {
        return Ok(None);
    };
    prepare_group_target(request, boundaries, sequence, entry, band, clear)
}

fn prepare_group_target<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<NumericTailTarget<S>>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let lines = &sequence.selected_lines[entry.selected_len..];
    let Some(last) = lines.last() else {
        return Ok(None);
    };
    // A trailing nonformatted Paragraph receipt is NOT permission to target
    // the preceding Forced occurrence or to walk backward across that receipt.
    if last.ending != surgeist_text::LineEnd::Paragraph || !numeric_is_formatted(last) {
        return Ok(None);
    }
    let formatted = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| numeric_is_formatted(line).then_some(i))
        .collect::<Vec<_>>();
    if formatted.len() < 2 {
        return Ok(None);
    }
    let Some(pair) = final_pair(lines) else {
        return Ok(None);
    };
    if !pair.eligible_auto() {
        return Ok(None);
    }
    let ordinary = NumericGroupBasis {
        lines: lines.to_vec(),
        outgoing: sequence.state,
    };
    let mut reference = NumericTailReference {
        terminal: NumericGroupTerminal::of(last),
        count: formatted.len(),
        prefix: lines[..formatted[formatted.len() - 2]].to_vec(),
        frontier: None,
    };
    // One ordinary FINAL-group replay mints the real pair-frontier state. It
    // authenticates the complete original group, not a reconstructed cursor.
    let baseline = trial(
        request,
        boundaries,
        sequence,
        (&entry, &reference),
        None,
        band,
        clear,
    )?
    .ok_or(crate::inline_wrapping::NumericWrappingError::InvalidTailBasis)?;
    if baseline.value.group != ordinary || baseline.pair != pair {
        return Err(crate::inline_wrapping::NumericWrappingError::InvalidTailBasis);
    }
    reference.frontier = Some(baseline.value.frontier);
    Ok(Some(NumericTailTarget {
        entry,
        reference,
        pair,
        auto: baseline.value,
    }))
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

pub(super) fn repair_numeric_tail<S, B, C>(
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
    let Some(target) = prepare_target(request, boundaries, sequence, band, clear)? else {
        return Ok(());
    };
    repair_target(request, boundaries, sequence, target, band, clear).map(|_| ())
}

fn repair_target<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    target: NumericTailTarget<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<TailGoal>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let winner = choose_tail(target.pair, |goal| {
        let candidate = trial(
            request,
            boundaries,
            sequence,
            (&target.entry, &target.reference),
            Some(goal),
            band,
            clear,
        )?;
        #[cfg(test)]
        if let Some(candidate) = &candidate {
            tests::observe(target.pair, &target.reference, candidate);
        }
        Ok::<_, crate::inline_wrapping::NumericWrappingError>(candidate)
    })?;
    let Some(winner) = winner else {
        restore_program(sequence, target.entry, &target.auto);
        return Ok(None);
    };
    let replay = trial(
        request,
        boundaries,
        sequence,
        (&target.entry, &target.reference),
        Some(winner.goal),
        band,
        clear,
    )?
    .ok_or(crate::inline_wrapping::NumericWrappingError::InvalidTailBasis)?;
    if replay.value != winner.trial.value || replay.pair != winner.trial.pair {
        return Err(crate::inline_wrapping::NumericWrappingError::InvalidTailBasis);
    }
    Ok(Some(winner.goal))
}

/// The same Tail phase at the current group's authentic ordinary entry.
pub(super) fn repair_group_from_auto<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    entry: NumericGroupEntry<S>,
    band: &mut B,
    clear: &mut C,
) -> Result<Option<TailGoal>, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let Some(target) = prepare_group_target(request, boundaries, sequence, entry, band, clear)?
    else {
        return Ok(None);
    };
    repair_target(request, boundaries, sequence, target, band, clear)
}

pub(super) fn replay_group_from_auto<S, B, C>(
    request: NumericWalkInput<'_, S>,
    boundaries: &[InlineSourceCursor],
    sequence: &mut NumericSequence<S>,
    phase: (NumericGroupEntry<S>, TailGoal),
    band: &mut B,
    clear: &mut C,
) -> Result<bool, crate::inline_wrapping::NumericWrappingError>
where
    S: LayoutScalar,
    B: FnMut(S, S) -> LogicalLineBandQueryResultOf<S>,
    C: FnMut(S, PostLineClearIntent) -> S,
{
    let (entry, goal) = phase;
    let Some(target) = prepare_group_target(request, boundaries, sequence, entry, band, clear)?
    else {
        return Ok(false);
    };
    let Some(replay) = trial(
        request,
        boundaries,
        sequence,
        (&entry, &target.reference),
        Some(goal),
        band,
        clear,
    )?
    else {
        return Ok(false);
    };
    Ok(replay.pair.admission(target.pair) == TailAdmission::Improved)
}

#[cfg(test)]
pub(super) mod tests;
