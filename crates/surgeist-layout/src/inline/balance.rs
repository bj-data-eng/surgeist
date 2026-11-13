//! Numeric Balance owns each active walk until its actual receiver is answered.
use super::*;
use crate::inline_wrapping::balance::{GroupSelection, GroupWinner, variance_order};
use std::cmp::Ordering;

#[derive(Clone, Copy)]
enum Phase {
    Reference,
    LocalAuto,
    Goal,
    WinnerReplay,
    ProgramReplay,
    Trailing,
    Fallback,
    Complete,
}

pub(super) struct NumericBalance<S: LayoutScalar> {
    initial: NumericGroupEntry<S>,
    entry: NumericGroupEntry<S>,
    walk: NumericWalk,
    phase: Phase,
    references: Vec<NumericReferenceGroup<S>>,
    program: Vec<NumericGroupProgram<S>>,
    group: usize,
    selection: Option<GroupSelection<NumericGroupBasis<S>>>,
    winner: Option<GroupWinner<NumericGroupBasis<S>>>,
    improved: bool,
    trailing_start: usize,
}

impl<S: LayoutScalar> NumericBalance<S> {
    pub(super) fn new(sequence: &NumericSequence<S>) -> Self {
        let initial = sequence.checkpoint();
        Self {
            initial,
            entry: initial,
            walk: NumericWalk::new(sequence, NumericWalkControl::auto()),
            phase: Phase::Reference,
            references: Vec::new(),
            program: Vec::new(),
            group: 0,
            selection: None,
            winner: None,
            improved: false,
            trailing_start: 0,
        }
    }

    fn start(
        &mut self,
        sequence: &mut NumericSequence<S>,
        entry: NumericGroupEntry<S>,
        phase: Phase,
        control: NumericWalkControl,
    ) {
        sequence.restore(entry);
        self.entry = entry;
        self.walk = NumericWalk::new(sequence, control);
        self.phase = phase;
    }

    fn group_walk(
        &mut self,
        sequence: &mut NumericSequence<S>,
        phase: Phase,
        goal: Option<BalanceGoal>,
    ) {
        self.start(
            sequence,
            self.entry,
            phase,
            NumericWalkControl::group(
                goal.map(LineQualityGoal::Balance),
                Some(self.references[self.group].count),
            ),
        );
    }

    fn fallback(&mut self, sequence: &mut NumericSequence<S>) {
        self.selection = None;
        self.winner = None;
        self.start(
            sequence,
            self.initial,
            Phase::Fallback,
            NumericWalkControl::auto(),
        );
    }

    fn trailing(&mut self, sequence: &mut NumericSequence<S>) {
        self.trailing_start = sequence.selected_lines.len();
        self.start(
            sequence,
            sequence.checkpoint(),
            Phase::Trailing,
            NumericWalkControl::auto(),
        );
    }

    fn next_local(&mut self, sequence: &mut NumericSequence<S>) {
        self.group += 1;
        if self.group < self.references.len() {
            self.entry = sequence.checkpoint();
            self.group_walk(sequence, Phase::LocalAuto, None);
        } else {
            sequence.restore(self.initial);
            self.group = 0;
            self.entry = self.initial;
            if self.references.is_empty() {
                self.trailing(sequence);
            } else {
                self.group_walk(sequence, Phase::ProgramReplay, self.program[0].goal);
            }
        }
    }

    fn next_goal(&mut self, sequence: &mut NumericSequence<S>) -> Result<(), NumericWrappingError> {
        if let Some(goal) = self
            .selection
            .as_mut()
            .expect("eligible portfolio")
            .next_goal()
        {
            self.group_walk(sequence, Phase::Goal, Some(goal));
        } else {
            let winner = self.selection.take().expect("finished portfolio").finish();
            let Some(winner) = winner else {
                self.fallback(sequence);
                return Ok(());
            };
            self.improved |= variance_order(
                &winner.trial.residuals,
                self.references[self.group]
                    .residuals
                    .as_ref()
                    .expect("eligible reference"),
            )
            .ok_or(NumericWrappingError::InvalidBalanceBasis)?
                == Ordering::Less;
            let goal = winner.goal;
            self.winner = Some(winner);
            self.group_walk(sequence, Phase::WinnerReplay, goal);
        }
        Ok(())
    }

    pub(super) fn advance<B, C, E>(
        &mut self,
        request: NumericWalkInput<'_, S>,
        sequence: &mut NumericSequence<S>,
        boundaries: &[InlineSourceCursor],
        mut band: B,
        mut clear: C,
    ) -> Result<NumericWalkStep, NumericRunError<E>>
    where
        B: FnMut(S, S) -> Result<LogicalLineBandQueryResultOf<S>, E>,
        C: FnMut(S, PostLineClearIntent) -> S,
    {
        let invalid = NumericWrappingError::InvalidBalanceBasis;
        loop {
            if matches!(self.phase, Phase::Complete) {
                return Ok(NumericWalkStep::Complete(NumericWalkOutcome::Exhausted));
            }
            // Resume the active walk itself. A receiver pause is not a new
            // trial: no restore, direction resampling or formatted-count reset.
            let mut remaining = boundaries[sequence.state.boundary_index..]
                .iter()
                .copied()
                .peekable();
            let outcome = match walk_numeric_sequence_step(
                request,
                sequence,
                &mut remaining,
                &mut self.walk,
                &mut band,
                &mut clear,
            )? {
                NumericWalkStep::Receiver(demand) => return Ok(NumericWalkStep::Receiver(demand)),
                NumericWalkStep::Complete(outcome) => outcome,
            };
            match self.phase {
                Phase::Reference => {
                    self.references = numeric_reference_groups(&sequence.selected_lines)?;
                    self.program = Vec::with_capacity(self.references.len());
                    sequence.restore(self.initial);
                    if self.references.is_empty() {
                        self.trailing(sequence);
                    } else {
                        self.group_walk(sequence, Phase::LocalAuto, None);
                    }
                }
                Phase::LocalAuto => {
                    let reference = &self.references[self.group];
                    let trial =
                        finish_numeric_group_trial(sequence, self.entry, reference, outcome)?;
                    if let Some(residuals) = &reference.residuals {
                        self.selection = GroupSelection::new(residuals, trial);
                        self.next_goal(sequence)?;
                    } else if let Some(trial) = trial {
                        self.program.push(NumericGroupProgram {
                            goal: None,
                            basis: trial.value,
                        });
                        self.next_local(sequence);
                    } else {
                        self.fallback(sequence);
                    }
                }
                Phase::Goal => {
                    let trial = finish_numeric_group_trial(
                        sequence,
                        self.entry,
                        &self.references[self.group],
                        outcome,
                    )?;
                    self.selection
                        .as_mut()
                        .expect("pending finite goal")
                        .accept(trial);
                    self.next_goal(sequence)?;
                }
                Phase::WinnerReplay => {
                    let replay = finish_numeric_group_trial(
                        sequence,
                        self.entry,
                        &self.references[self.group],
                        outcome,
                    )?;
                    let Some(replay) = replay else {
                        self.fallback(sequence);
                        continue;
                    };
                    let winner = self.winner.take().expect("selected group winner");
                    if replay.value != winner.trial.value {
                        return Err(invalid.into());
                    }
                    self.program.push(NumericGroupProgram {
                        goal: winner.goal,
                        basis: replay.value,
                    });
                    self.next_local(sequence);
                }
                Phase::ProgramReplay => {
                    let reference = &self.references[self.group];
                    let replay =
                        finish_numeric_group_trial(sequence, self.entry, reference, outcome)?;
                    let Some(replay) = replay else {
                        self.fallback(sequence);
                        continue;
                    };
                    if replay.value != self.program[self.group].basis {
                        return Err(invalid.into());
                    }
                    if let Some(residuals) = &reference.residuals
                        && variance_order(&replay.residuals, residuals).ok_or(invalid)?
                            == Ordering::Greater
                    {
                        self.fallback(sequence);
                        continue;
                    }
                    self.group += 1;
                    if self.group == self.references.len() {
                        self.trailing(sequence);
                    } else {
                        self.entry = sequence.checkpoint();
                        self.group_walk(
                            sequence,
                            Phase::ProgramReplay,
                            self.program[self.group].goal,
                        );
                    }
                }
                Phase::Trailing => {
                    if outcome != NumericWalkOutcome::Exhausted
                        || sequence.selected_lines.len() != self.trailing_start
                    {
                        return Err(invalid.into());
                    }
                    if self.improved {
                        self.phase = Phase::Complete;
                    } else {
                        self.fallback(sequence);
                    }
                }
                Phase::Fallback => self.phase = Phase::Complete,
                Phase::Complete => unreachable!("completed owner has no active walk"),
            }
        }
    }
}
