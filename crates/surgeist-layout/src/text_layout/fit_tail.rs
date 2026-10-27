//! Optional repair of the genuine formatted Paragraph-ending pair.

use super::balance::{GroupProof, GroupReference, LineProof, Observations};
use super::*;
use crate::inline_wrapping::quality::NaturalLine;
use crate::inline_wrapping::tail::{TailGoal, TailPair, TailTrial, choose_tail};
use std::ops::Range;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct TailBasis {
    target: Option<TailTarget>,
    goal: Option<TailGoal>,
}

#[derive(Clone, Debug, PartialEq)]
struct TailTarget {
    window: Range<usize>,
    count: usize,
    frozen_prefix: usize,
    // Exactly the two real Auto requests/frames, not a paragraph copy per line.
    reference: Vec<LineProof>,
}

pub(super) fn pair(lines: &[FittedLine]) -> Option<TailPair> {
    let mut formatted = lines
        .iter()
        .rev()
        .filter(|line| line.disposition.is_formatted());
    let line = |line: &FittedLine| {
        let selected = line.finalized.selected_line();
        let request = selected.request();
        NaturalLine::from_components(
            request.context.available_inline?,
            request.context.indent,
            line.natural_advance,
        )
    };
    let last = line(formatted.next()?)?;
    let penultimate = line(formatted.next()?)?;
    Some(TailPair::new(penultimate, last))
}

pub(super) fn formatted_paragraph(lines: &[FittedLine]) -> bool {
    lines.last().is_some_and(|line| {
        line.disposition.is_formatted()
            && line.finalized.selected_line().request().context.end == LineEnd::Paragraph
    })
}

fn attach(lines: &mut [FittedLine], target: Option<TailTarget>, goal: Option<TailGoal>) {
    let basis = Arc::new(SelectionProgram::Tail(TailBasis { target, goal }));
    for line in lines {
        line.program = Some(Arc::clone(&basis));
    }
}

struct TailAttempt<'a> {
    entry: &'a SequenceEntry,
    reference: &'a GroupReference,
    prefix: &'a [LineProof],
    goal: TailGoal,
}

impl<B, C, A, R, X, Accept, Clear> SequenceRunner<'_, B, C, A, R, X, Accept, Clear>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset),
    Accept: FnMut() -> Result<(), FitError>,
    Clear: FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
{
    fn tail_group(
        &mut self,
        attempt: TailAttempt<'_>,
        result: &mut Vec<FittedLine>,
        observations: &mut Observations,
    ) -> Result<Option<TailTrial<GroupProof>>, FitError> {
        let TailAttempt {
            entry,
            reference,
            prefix,
            goal,
        } = attempt;
        let outcome = self.run(
            entry,
            SequenceScope {
                reset: EncounterReset::Group,
                stop_at_forced: true,
                max_formatted: Some(reference.basis.count),
                goal: Some(LineQualityGoal::Tail(goal)),
                goal_after: reference.basis.count - 2,
                goal_before: None,
                opportunity_start: reference.window.start,
                opportunity_end: reference.window.end,
            },
            result,
        )?;
        let lines = &result[entry.prefix_len..];
        observations.record(lines);
        let SequenceOutcome::Complete(outgoing) = outcome else {
            return Ok(None);
        };
        let Some(trial) = balance::authenticate(reference, lines, outgoing)? else {
            return Ok(None);
        };
        if !formatted_paragraph(lines) || trial.value.lines.get(..prefix.len()) != Some(prefix) {
            return Ok(None);
        }
        Ok(pair(lines).map(|pair| TailTrial {
            pair,
            value: trial.value,
        }))
    }
}

pub(super) fn fit<B, C, A, R, X, Accept, Clear>(
    runner: &mut SequenceRunner<'_, B, C, A, R, X, Accept, Clear>,
    initial: SequenceEntry,
    mut checkpoint: impl FnMut(),
) -> Result<Vec<FittedLine>, FitError>
where
    B: FnMut(f64, f64) -> Result<Band, FitError>,
    C: FnMut(BlockPosition, f64) -> Result<f64, FitError>,
    A: FnMut(
        &FinalizedLine,
        Band,
        f64,
        &[surgeist_text::LineBoxStrut],
    ) -> Result<LineAllocations, FitError>,
    R: FnMut(&FinalizedLine, EncounterReplay) -> Result<Vec<EncounterBasis>, FitError>,
    X: FnMut(EncounterReset),
    Accept: FnMut() -> Result<(), FitError>,
    Clear: FnMut(f64, &LineSelectionRequest) -> Result<Option<f64>, FitError>,
{
    let whole = SequenceScope {
        reset: EncounterReset::Sequence,
        stop_at_forced: false,
        max_formatted: None,
        goal: None,
        goal_after: 0,
        goal_before: None,
        opportunity_start: 0,
        opportunity_end: runner.opportunities.len(),
    };
    let mut result = Vec::new();
    if !matches!(
        runner.run(&initial, whole, &mut result)?,
        SequenceOutcome::Complete(_)
    ) {
        return Err(FitError::InvalidTailBasis);
    }
    // A trailing nonformatted Paragraph cannot turn a preceding Forced line
    // into a target. The semantic terminal occurrence itself must be formatted.
    if !formatted_paragraph(&result) {
        attach(&mut result, None, None);
        return Ok(result);
    }
    let final_start = result[..result.len() - 1]
        .iter()
        .rposition(|line| line.finalized.selected_line().request().context.end != LineEnd::Soft)
        .map_or(0, |index| index + 1);
    let group_lines = &result[final_start..];
    let Some(reference_pair) = pair(group_lines).filter(|pair| pair.eligible_auto()) else {
        attach(&mut result, None, None);
        return Ok(result);
    };
    let mut tail_indices = group_lines
        .iter()
        .enumerate()
        .rev()
        .filter(|(_, line)| line.disposition.is_formatted());
    let last = tail_indices.next().expect("authenticated pair").0;
    let penultimate = tail_indices.next().expect("authenticated pair").0;
    let original = result.iter().map(LineProof::of).collect::<Vec<_>>();
    let references = balance::references(&result, &runner.opportunities)?;
    let final_reference = references.last().ok_or(FitError::InvalidTailBasis)?;
    let target = TailTarget {
        window: final_reference.window.clone(),
        count: final_reference.basis.count,
        frozen_prefix: final_start + penultimate,
        reference: vec![
            original[final_start + penultimate].clone(),
            original[final_start + last].clone(),
        ],
    };
    let prefix = &original[final_start..final_start + penultimate];
    let mut observations = Observations::default();
    observations.record(&result);

    // One rolling ordinary replay constructs the actual incoming float ledger
    // and complete state of the final group. No earlier group is walked per
    // target, nor is an entry fabricated from its ending source offset.
    (runner.environment.reset)(EncounterReset::Sequence);
    result.clear();
    let mut entry = initial.clone();
    let mut ordinary_valid = true;
    for reference in &references[..references.len() - 1] {
        checkpoint();
        let start = entry.prefix_len;
        let Some(trial) = runner.group(&entry, reference, None, &mut result, &mut observations)?
        else {
            ordinary_valid = false;
            break;
        };
        let end = trial.value.outgoing.prefix_len;
        if original.get(start..end) != Some(trial.value.lines.as_slice()) {
            ordinary_valid = false;
            break;
        }
        entry = trial.value.outgoing;
    }
    let mut selected_goal = None;
    if ordinary_valid && entry.prefix_len == final_start {
        checkpoint();
        if let Some(winner) = choose_tail(reference_pair, |goal| {
            runner.tail_group(
                TailAttempt {
                    entry: &entry,
                    reference: final_reference,
                    prefix,
                    goal,
                },
                &mut result,
                &mut observations,
            )
        })? {
            let Some(replayed) = runner.tail_group(
                TailAttempt {
                    entry: &entry,
                    reference: final_reference,
                    prefix,
                    goal: winner.goal,
                },
                &mut result,
                &mut observations,
            )?
            else {
                return Err(FitError::InvalidTailBasis);
            };
            if replayed != winner.trial {
                return Err(FitError::InvalidTailBasis);
            }
            selected_goal = Some(winner.goal);
        }
    }

    if let Some(goal) = selected_goal {
        let chosen = result.iter().map(LineProof::of).collect::<Vec<_>>();
        (runner.environment.reset)(EncounterReset::Sequence);
        result.clear();
        let mut entry = initial.clone();
        let mut valid = true;
        for (index, reference) in references.iter().enumerate() {
            checkpoint();
            let outgoing = if index + 1 == references.len() {
                runner
                    .tail_group(
                        TailAttempt {
                            entry: &entry,
                            reference,
                            prefix,
                            goal,
                        },
                        &mut result,
                        &mut observations,
                    )?
                    .map(|trial| trial.value.outgoing)
            } else {
                runner
                    .group(&entry, reference, None, &mut result, &mut observations)?
                    .map(|trial| trial.value.outgoing)
            };
            let Some(outgoing) = outgoing else {
                valid = false;
                break;
            };
            entry = outgoing;
        }
        if !valid {
            selected_goal = None;
        } else if result.iter().map(LineProof::of).ne(chosen) {
            return Err(FitError::InvalidTailBasis);
        }
    }
    if selected_goal.is_none()
        && (!matches!(
            runner.run(&initial, whole, &mut result)?,
            SequenceOutcome::Complete(_)
        ) || result.iter().map(LineProof::of).ne(original))
    {
        return Err(FitError::InvalidTailBasis);
    }
    attach(&mut result, Some(target), selected_goal);
    observations.attach(&mut result);
    Ok(result)
}
