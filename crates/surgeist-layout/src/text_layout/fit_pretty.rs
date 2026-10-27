//! Streaming Auto → eligible Tail → fixed-reference Body at genuine entries.

use super::balance::{GroupProof, LineProof, Observations};
use super::*;
use crate::inline_wrapping::body::{BodyGoal, BodyScore, BodyTrial, choose_body};
use crate::inline_wrapping::quality::NaturalLine;
use crate::inline_wrapping::tail::{TailGoal, TailTrial, choose_tail};
use std::{ops::Range, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrettyBasis {
    disposition: PrettyDisposition,
}

#[derive(Clone, Debug, PartialEq)]
enum PrettyDisposition {
    Stream(Vec<PrettyGroupBasis>),
    OrdinaryClosure(ScopeExhaustion),
}

#[derive(Clone, Debug, PartialEq)]
struct PrettyGroupBasis {
    entry: SequenceEntry,
    window: GroupWindow,
    count: usize,
    ordinary: GroupProof,
    post_tail: Option<GroupProof>,
    tail_goal: Option<TailGoal>,
    body_reference: Option<BodyScore>,
    body_goal: Option<BodyGoal>,
}

struct BuiltGroup {
    basis: PrettyGroupBasis,
    selected: GroupProof,
}

#[derive(Clone, Debug, PartialEq)]
struct GroupWindow {
    range: Range<usize>,
    terminal: PreparedTermination,
    boundary: PreparedBoundary,
    ending: LineEnd,
}

fn windows(opportunities: &[Opportunity]) -> Vec<GroupWindow> {
    let mut start = 0;
    opportunities
        .iter()
        .enumerate()
        .filter_map(|(index, (_, offer, kind, _))| {
            let ending = match kind {
                PreparedBreakKind::Forced => LineEnd::Forced,
                PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
                _ => return None,
            };
            let range = start..index + 1;
            start = index + 1;
            Some(GroupWindow {
                range,
                terminal: PreparedTermination::Ordinary(offer.clone()),
                boundary: offer.view().boundary(),
                ending,
            })
        })
        .collect()
}

fn body_score(lines: &[FittedLine]) -> Option<BodyScore> {
    BodyScore::from_lines(
        lines
            .iter()
            .filter(|line| line.disposition.is_formatted())
            .map(|line| {
                let selected = line.finalized.selected_line();
                let request = selected.request();
                NaturalLine::from_components(
                    request.context.available_inline?,
                    request.context.indent,
                    line.natural_advance,
                )
            }),
    )
}

#[derive(Clone, Copy)]
enum GroupGoal {
    Tail(TailGoal),
    Body(BodyGoal),
}

struct GroupAttempt<'a> {
    entry: &'a SequenceEntry,
    window: &'a GroupWindow,
    count: Option<usize>,
    goal: Option<GroupGoal>,
}

enum GroupOutcome {
    Complete(GroupProof),
    Rejected(SequenceRejection),
}

enum StreamOutcome {
    Complete(Vec<BuiltGroup>),
    ClosureExhausted(ScopeExhaustion),
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
    fn pretty_group(
        &mut self,
        attempt: GroupAttempt<'_>,
        result: &mut Vec<FittedLine>,
    ) -> Result<GroupOutcome, FitError> {
        let GroupAttempt {
            entry,
            window,
            count,
            goal,
        } = attempt;
        let (goal, after, before) = match goal {
            None => (None, 0, None),
            Some(GroupGoal::Tail(goal)) => (
                Some(LineQualityGoal::Tail(goal)),
                count
                    .ok_or(FitError::InvalidPrettyBasis)?
                    .checked_sub(2)
                    .ok_or(FitError::InvalidPrettyBasis)?,
                None,
            ),
            Some(GroupGoal::Body(goal)) => (
                Some(LineQualityGoal::Body(goal)),
                0,
                Some(
                    count
                        .ok_or(FitError::InvalidPrettyBasis)?
                        .checked_sub(1)
                        .ok_or(FitError::InvalidPrettyBasis)?,
                ),
            ),
        };
        let outcome = self.run(
            entry,
            SequenceScope {
                reset: EncounterReset::Group,
                stop_at_forced: true,
                max_formatted: count,
                goal,
                goal_after: after,
                goal_before: before,
                opportunity_start: window.range.start,
                opportunity_end: window.range.end,
            },
            result,
        )?;
        let outgoing = match outcome {
            SequenceOutcome::Complete(outgoing) => outgoing,
            SequenceOutcome::Inadmissible(reason) => return Ok(GroupOutcome::Rejected(reason)),
        };
        let lines = &result[entry.prefix_len..];
        let last = lines.last().ok_or(FitError::InvalidPrettyBasis)?;
        let selected = last.finalized.selected_line();
        let request = selected.request();
        if request.termination != window.terminal
            || request.end != window.boundary
            || request.context.end != window.ending
        {
            return Err(FitError::InvalidPrettyBasis);
        }
        if count.is_some_and(|count| {
            lines
                .iter()
                .filter(|line| line.disposition.is_formatted())
                .count()
                != count
        }) {
            return Ok(GroupOutcome::Rejected(SequenceRejection::CountExceeded));
        }
        Ok(GroupOutcome::Complete(GroupProof {
            lines: lines.iter().map(LineProof::of).collect(),
            outgoing,
        }))
    }

    fn pretty_replay(
        &mut self,
        attempt: GroupAttempt<'_>,
        expected: &GroupProof,
        result: &mut Vec<FittedLine>,
    ) -> Result<GroupProof, FitError> {
        let GroupOutcome::Complete(proof) = self.pretty_group(attempt, result)? else {
            // A previously selected proof must replay. This is not normal
            // optional closure exhaustion and cannot trigger ordinary fallback.
            return Err(FitError::InvalidPrettyBasis);
        };
        if &proof != expected {
            return Err(FitError::InvalidPrettyBasis);
        }
        Ok(proof)
    }

    fn pretty_stream(
        &mut self,
        initial: &SequenceEntry,
        windows: &[GroupWindow],
        checkpoint: &mut impl FnMut(),
        result: &mut Vec<FittedLine>,
    ) -> Result<StreamOutcome, FitError> {
        let mut groups = Vec::with_capacity(windows.len());
        let mut entry = initial.clone();
        for window in windows {
            checkpoint();
            let ordinary = match self.pretty_group(
                GroupAttempt {
                    entry: &entry,
                    window,
                    count: None,
                    goal: None,
                },
                result,
            )? {
                GroupOutcome::Complete(proof) => proof,
                GroupOutcome::Rejected(SequenceRejection::ScopeExhausted(
                    ScopeExhaustion::TerminalConstraint,
                )) => {
                    return Ok(StreamOutcome::ClosureExhausted(
                        ScopeExhaustion::TerminalConstraint,
                    ));
                }
                GroupOutcome::Rejected(SequenceRejection::ScopeExhausted(
                    ScopeExhaustion::NoAdmissibleCandidate,
                )) => {
                    return Err(FitError::FeedbackCycle);
                }
                GroupOutcome::Rejected(SequenceRejection::CountExceeded) => {
                    return Err(FitError::InvalidPrettyBasis);
                }
            };
            let count = result[entry.prefix_len..]
                .iter()
                .filter(|line| line.disposition.is_formatted())
                .count();
            let mut incumbent = ordinary.clone();
            let mut tail_goal = None;
            let lines = &result[entry.prefix_len..];
            if tail::formatted_paragraph(lines)
                && let Some(pair) = tail::pair(lines).filter(|pair| pair.eligible_auto())
            {
                let penultimate = lines
                    .iter()
                    .enumerate()
                    .rev()
                    .filter(|(_, line)| line.disposition.is_formatted())
                    .nth(1)
                    .expect("actual pair")
                    .0;
                let prefix = &ordinary.lines[..penultimate];
                if let Some(winner) = choose_tail(pair, |goal| {
                    let GroupOutcome::Complete(proof) = self.pretty_group(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: Some(GroupGoal::Tail(goal)),
                        },
                        result,
                    )?
                    else {
                        return Ok::<_, FitError>(None);
                    };
                    if !tail::formatted_paragraph(&result[entry.prefix_len..])
                        || proof.lines.get(..prefix.len()) != Some(prefix)
                    {
                        return Ok(None);
                    }
                    Ok(tail::pair(&result[entry.prefix_len..])
                        .map(|pair| TailTrial { pair, value: proof }))
                })? {
                    incumbent = self.pretty_replay(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: Some(GroupGoal::Tail(winner.goal)),
                        },
                        &winner.trial.value,
                        result,
                    )?;
                    tail_goal = Some(winner.goal);
                } else {
                    incumbent = self.pretty_replay(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: None,
                        },
                        &ordinary,
                        result,
                    )?;
                }
            }
            let post_tail = tail_goal.map(|_| incumbent.clone());
            let body_reference = body_score(&result[entry.prefix_len..]);
            let mut body_goal = None;
            if let Some(reference) = body_reference {
                if let Some(winner) = choose_body(reference, |goal| {
                    let GroupOutcome::Complete(proof) = self.pretty_group(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: Some(GroupGoal::Body(goal)),
                        },
                        result,
                    )?
                    else {
                        return Ok::<_, FitError>(None);
                    };
                    Ok(
                        body_score(&result[entry.prefix_len..]).map(|score| BodyTrial {
                            score,
                            value: proof,
                        }),
                    )
                })? {
                    incumbent = self.pretty_replay(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: Some(GroupGoal::Body(winner.goal)),
                        },
                        &winner.trial.value,
                        result,
                    )?;
                    body_goal = Some(winner.goal);
                } else {
                    incumbent = self.pretty_replay(
                        GroupAttempt {
                            entry: &entry,
                            window,
                            count: Some(count),
                            goal: tail_goal.map(GroupGoal::Tail),
                        },
                        &incumbent,
                        result,
                    )?;
                }
            }
            #[cfg(test)]
            observe_group(&result[entry.prefix_len..]);
            let outgoing = incumbent.outgoing.clone();
            groups.push(BuiltGroup {
                basis: PrettyGroupBasis {
                    entry,
                    window: window.clone(),
                    count,
                    ordinary,
                    post_tail,
                    tail_goal,
                    body_reference,
                    body_goal,
                },
                selected: incumbent,
            });
            // This selected group's actual outgoing geometry/state establishes
            // the NEXT ordinary reference, including its possibly new count.
            entry = outgoing;
        }
        Ok(StreamOutcome::Complete(groups))
    }

    pub(super) fn pretty(
        &mut self,
        initial: SequenceEntry,
        mut checkpoint: impl FnMut(),
    ) -> Result<Vec<FittedLine>, FitError> {
        self.environment.observed = Some(Observations::default());
        (self.environment.reset)(EncounterReset::Sequence);
        let windows = windows(&self.opportunities);
        if windows
            .last()
            .is_none_or(|window| window.ending != LineEnd::Paragraph)
        {
            return Err(FitError::InvalidPrettyBasis);
        }
        let mut result = Vec::new();
        let basis = match self.pretty_stream(&initial, &windows, &mut checkpoint, &mut result)? {
            StreamOutcome::ClosureExhausted(reason) => {
                #[cfg(test)]
                observe_closure(reason);
                // Once-only ordinary closure from the same invocation input.
                // Journals roll back, but the observation stream and actual
                // work/cancellation charges are not reset.
                let outcome = self.run(
                    &initial,
                    SequenceScope {
                        reset: EncounterReset::Sequence,
                        stop_at_forced: false,
                        max_formatted: None,
                        goal: None,
                        goal_after: 0,
                        goal_before: None,
                        opportunity_start: 0,
                        opportunity_end: self.opportunities.len(),
                    },
                    &mut result,
                )?;
                if !matches!(outcome, SequenceOutcome::Complete(_)) {
                    return Err(FitError::FeedbackCycle);
                }
                PrettyBasis {
                    disposition: PrettyDisposition::OrdinaryClosure(reason),
                }
            }
            StreamOutcome::Complete(groups) => {
                // Reestablish ordinary and phase references under the same real
                // entries, then verify the selected goals. No stale global-Auto
                // count/frame is imported into the streaming program.
                (self.environment.reset)(EncounterReset::Sequence);
                result.clear();
                let mut entry = initial.clone();
                for group in &groups {
                    if entry != group.basis.entry {
                        return Err(FitError::InvalidPrettyBasis);
                    }
                    checkpoint();
                    let mut phase = self.pretty_replay(
                        GroupAttempt {
                            entry: &entry,
                            window: &group.basis.window,
                            count: None,
                            goal: None,
                        },
                        &group.basis.ordinary,
                        &mut result,
                    )?;
                    if let Some(goal) = group.basis.tail_goal {
                        phase = self.pretty_replay(
                            GroupAttempt {
                                entry: &entry,
                                window: &group.basis.window,
                                count: Some(group.basis.count),
                                goal: Some(GroupGoal::Tail(goal)),
                            },
                            group
                                .basis
                                .post_tail
                                .as_ref()
                                .ok_or(FitError::InvalidPrettyBasis)?,
                            &mut result,
                        )?;
                    }
                    if body_score(&result[entry.prefix_len..]) != group.basis.body_reference {
                        return Err(FitError::InvalidPrettyBasis);
                    }
                    let proof = if let Some(goal) = group.basis.body_goal {
                        self.pretty_replay(
                            GroupAttempt {
                                entry: &entry,
                                window: &group.basis.window,
                                count: Some(group.basis.count),
                                goal: Some(GroupGoal::Body(goal)),
                            },
                            &group.selected,
                            &mut result,
                        )?
                    } else {
                        if phase != group.selected {
                            return Err(FitError::InvalidPrettyBasis);
                        }
                        phase
                    };
                    entry = proof.outgoing;
                }
                PrettyBasis {
                    disposition: PrettyDisposition::Stream(
                        groups.into_iter().map(|group| group.basis).collect(),
                    ),
                }
            }
        };
        let program = Arc::new(SelectionProgram::Pretty(basis));
        for line in &mut result {
            line.program = Some(Arc::clone(&program));
        }
        self.environment
            .observed
            .take()
            .ok_or(FitError::InvalidPrettyBasis)?
            .attach(&mut result);
        Ok(result)
    }
}

#[cfg(test)]
thread_local! {
    static CLOSURES: std::cell::RefCell<Vec<ScopeExhaustion>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn observe_closure(reason: ScopeExhaustion) {
    CLOSURES.with_borrow_mut(|closures| closures.push(reason));
}

#[cfg(test)]
thread_local! {
    static GROUPS: std::cell::RefCell<Vec<Vec<(usize, f64)>>> = const { std::cell::RefCell::new(Vec::new()) };
}

#[cfg(test)]
fn observe_group(lines: &[FittedLine]) {
    GROUPS.with_borrow_mut(|groups| {
        groups.push(
            lines
                .iter()
                .map(|line| {
                    (
                        line.finalized
                            .selected_line()
                            .request()
                            .end
                            .view()
                            .source_after()
                            .get(),
                        line.natural_advance,
                    )
                })
                .collect(),
        )
    });
}

#[cfg(test)]
#[path = "fit_pretty_tests.rs"]
mod tests;
