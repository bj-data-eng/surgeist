//! One Tail owner for discovery, preparation and the finite target portfolio.
use super::*;

#[derive(Clone, Copy)]
enum Phase {
    Discover,
    Baseline,
    Goal,
    WinnerReplay,
    ReplayGoal,
    Complete,
}

pub(in crate::inline) struct NumericTail<S: LayoutScalar> {
    walk: NumericWalk,
    phase: Phase,
    entry: NumericGroupEntry<S>,
    final_entry: Option<NumericGroupEntry<S>>,
    preparation: Option<NumericTailPreparation<S>>,
    target: Option<NumericTailTarget<S>>,
    selection: Option<TailSelection<NumericTailBasis<S>>>,
    winner: Option<TailWinner<NumericTailBasis<S>>>,
    replay: Option<TailGoal>,
    result: Option<TailGoal>,
}

impl<S: LayoutScalar> NumericTail<S> {
    pub(in crate::inline) fn new(sequence: &NumericSequence<S>) -> Self {
        Self {
            walk: NumericWalk::new(sequence, NumericWalkControl::group(None, None)),
            phase: Phase::Discover,
            entry: sequence.checkpoint(),
            final_entry: None,
            preparation: None,
            target: None,
            selection: None,
            winner: None,
            replay: None,
            result: None,
        }
    }

    pub(super) fn from_auto(
        sequence: &mut NumericSequence<S>,
        entry: NumericGroupEntry<S>,
        replay: Option<TailGoal>,
    ) -> Self {
        let mut owner = Self::new(sequence);
        owner.replay = replay;
        owner.prepare(sequence, entry);
        owner
    }

    fn prepare(&mut self, sequence: &mut NumericSequence<S>, entry: NumericGroupEntry<S>) {
        let Some(preparation) = prepare_group_reference(sequence, entry) else {
            self.phase = Phase::Complete;
            return;
        };
        let control = trial_control(&preparation.reference, None);
        self.entry = preparation.entry;
        self.preparation = Some(preparation);
        sequence.restore(self.entry);
        self.walk = NumericWalk::new(sequence, control);
        self.phase = Phase::Baseline;
    }

    fn trial(&mut self, sequence: &mut NumericSequence<S>, phase: Phase, goal: TailGoal) {
        let target = self.target.as_ref().expect("prepared Tail target");
        sequence.restore(target.entry);
        self.walk = NumericWalk::new(sequence, trial_control(&target.reference, Some(goal)));
        self.phase = phase;
    }

    fn next_goal(&mut self, sequence: &mut NumericSequence<S>) {
        if let Some(goal) = self
            .selection
            .as_mut()
            .expect("eligible Tail portfolio")
            .next_goal()
        {
            self.trial(sequence, Phase::Goal, goal);
        } else {
            self.winner = self
                .selection
                .take()
                .expect("finished Tail portfolio")
                .finish();
            if let Some(winner) = &self.winner {
                self.trial(sequence, Phase::WinnerReplay, winner.goal);
            } else {
                let target = self.target.as_ref().expect("saved ordinary Tail basis");
                restore_program(sequence, target.entry, &target.auto);
                self.phase = Phase::Complete;
            }
        }
    }

    pub(in crate::inline) fn advance<B, C, E>(
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
        let invalid = NumericWrappingError::InvalidTailBasis;
        loop {
            if matches!(self.phase, Phase::Complete) {
                return Ok(NumericWalkStep::Complete(NumericWalkOutcome::Exhausted));
            }
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
                Phase::Discover => {
                    // The final entry includes a real trailing nonformatted
                    // receipt. Never search backward for a preferable group.
                    if sequence.selected_lines.len() > self.entry.selected_len {
                        self.final_entry = Some(self.entry);
                    }
                    if outcome == NumericWalkOutcome::Exhausted {
                        if let Some(entry) = self.final_entry {
                            self.prepare(sequence, entry);
                        } else {
                            self.phase = Phase::Complete;
                        }
                    } else {
                        self.entry = sequence.checkpoint();
                        self.walk =
                            NumericWalk::new(sequence, NumericWalkControl::group(None, None));
                    }
                }
                Phase::Baseline => {
                    let mut preparation = self
                        .preparation
                        .take()
                        .expect("baseline owns its ordinary group");
                    let baseline =
                        finish_trial(sequence, preparation.entry, &preparation.reference, outcome)
                            .ok_or(invalid)?;
                    if baseline.value.group != preparation.ordinary
                        || baseline.pair != preparation.pair
                    {
                        return Err(invalid.into());
                    }
                    preparation.reference.frontier = Some(baseline.value.frontier);
                    self.target = Some(NumericTailTarget {
                        entry: preparation.entry,
                        reference: preparation.reference,
                        pair: preparation.pair,
                        auto: baseline.value,
                    });
                    if let Some(goal) = self.replay {
                        self.trial(sequence, Phase::ReplayGoal, goal);
                    } else {
                        self.selection = TailSelection::new(preparation.pair);
                        self.next_goal(sequence);
                    }
                }
                Phase::Goal => {
                    let target = self.target.as_ref().expect("pending Tail goal");
                    let candidate =
                        finish_trial(sequence, target.entry, &target.reference, outcome);
                    #[cfg(test)]
                    if let Some(candidate) = &candidate {
                        super::tests::observe(target.pair, &target.reference, candidate);
                    }
                    self.selection
                        .as_mut()
                        .expect("pending Tail portfolio")
                        .accept(candidate);
                    self.next_goal(sequence);
                }
                Phase::WinnerReplay => {
                    let target = self.target.as_ref().expect("selected Tail target");
                    let replay = finish_trial(sequence, target.entry, &target.reference, outcome)
                        .ok_or(invalid)?;
                    let winner = self.winner.take().expect("selected Tail winner");
                    if replay.value != winner.trial.value || replay.pair != winner.trial.pair {
                        return Err(invalid.into());
                    }
                    self.result = Some(winner.goal);
                    self.phase = Phase::Complete;
                }
                Phase::ReplayGoal => {
                    let target = self.target.as_ref().expect("requested Tail replay");
                    if let Some(replay) =
                        finish_trial(sequence, target.entry, &target.reference, outcome)
                        && replay.pair.admission(target.pair) == TailAdmission::Improved
                    {
                        self.result = self.replay;
                    }
                    self.phase = Phase::Complete;
                }
                Phase::Complete => unreachable!("completed Tail owner has no active walk"),
            }
        }
    }

    pub(super) fn drain<B, C, E>(
        mut self,
        request: NumericWalkInput<'_, S>,
        sequence: &mut NumericSequence<S>,
        boundaries: &[InlineSourceCursor],
        band: B,
        clear: C,
    ) -> Result<Option<TailGoal>, NumericRunError<E>>
    where
        B: FnMut(S, S) -> Result<LogicalLineBandQueryResultOf<S>, E>,
        C: FnMut(S, PostLineClearIntent) -> S,
    {
        match self.advance(request, sequence, boundaries, band, clear)? {
            NumericWalkStep::Complete(_) => Ok(self.result),
            NumericWalkStep::Receiver(_) => {
                unreachable!("synchronous receiver callback resolves its demand")
            }
        }
    }
}
