//! Pretty retains either its active walk or its actual nested Tail owner.
use super::*;

#[derive(Clone, Copy)]
enum Phase {
    Ordinary,
    Tail,
    Goal,
    Winner,
    ProgramOrdinary,
    ProgramTail,
    ProgramBody,
    Exhaustion,
    Complete,
}

enum Active<S: LayoutScalar> {
    Walk(NumericWalk),
    Tail(Box<tail::NumericTail<S>>),
    Ready,
}

pub(in crate::inline) struct NumericPretty<S: LayoutScalar> {
    initial: NumericGroupEntry<S>,
    entry: NumericGroupEntry<S>,
    phase: Phase,
    active: Active<S>,
    tail_goal: Option<TailGoal>,
    ordinary: Option<GroupBasis<S>>,
    reference: Option<GroupReference<S>>,
    current: Option<PrettyGroupProgram<S>>,
    selection: Option<BodySelection<GroupBasis<S>>>,
    winner: Option<BodyWinner<GroupBasis<S>>>,
    program: Vec<PrettyGroupProgram<S>>,
    index: usize,
}

impl<S: LayoutScalar> NumericPretty<S> {
    pub(in crate::inline) fn new(sequence: &NumericSequence<S>) -> Self {
        let initial = sequence.checkpoint();
        Self {
            initial,
            entry: initial,
            phase: Phase::Ordinary,
            active: Active::Walk(NumericWalk::new(
                sequence,
                NumericWalkControl::group(None, None),
            )),
            tail_goal: None,
            ordinary: None,
            reference: None,
            current: None,
            selection: None,
            winner: None,
            program: Vec::new(),
            index: 0,
        }
    }

    fn walk(&mut self, sequence: &NumericSequence<S>, phase: Phase, control: NumericWalkControl) {
        self.active = Active::Walk(NumericWalk::new(sequence, control));
        self.phase = phase;
    }

    fn body_walk(
        &mut self,
        sequence: &mut NumericSequence<S>,
        phase: Phase,
        reference: GroupReference<S>,
        goal: BodyGoal,
    ) {
        sequence.restore(self.entry);
        self.walk(sequence, phase, body_control(reference, goal));
    }

    fn start_tail(
        &mut self,
        sequence: &mut NumericSequence<S>,
        phase: Phase,
        goal: Option<TailGoal>,
    ) {
        let tail = tail::NumericTail::from_auto(sequence, self.entry, goal);
        self.tail_goal = None;
        self.active = if let Some(goal) = tail.completed_goal() {
            // An ineligible Tail has no active walk to retain or allocate.
            self.tail_goal = goal;
            Active::Ready
        } else {
            Active::Tail(Box::new(tail))
        };
        self.phase = phase;
    }

    fn finish_current(&mut self, sequence: &NumericSequence<S>) {
        let group = self.current.take().expect("selected current Pretty group");
        #[cfg(test)]
        super::tests::observe_group(&group);
        self.program.push(group);
        self.entry = sequence.checkpoint();
        self.walk(
            sequence,
            Phase::Ordinary,
            NumericWalkControl::group(None, None),
        );
    }

    fn next_goal(&mut self, sequence: &mut NumericSequence<S>) {
        if let Some(goal) = self
            .selection
            .as_mut()
            .expect("fixed Body portfolio")
            .next_goal()
        {
            let reference = self
                .current
                .as_ref()
                .expect("current Pretty group")
                .reference;
            self.body_walk(sequence, Phase::Goal, reference, goal);
        } else {
            self.winner = self
                .selection
                .take()
                .expect("finished Body portfolio")
                .finish();
            if let Some(winner) = &self.winner {
                let goal = winner.goal;
                let reference = self
                    .current
                    .as_ref()
                    .expect("current Pretty group")
                    .reference;
                self.body_walk(sequence, Phase::Winner, reference, goal);
            } else {
                install(
                    sequence,
                    self.entry,
                    &self.current.as_ref().expect("saved post-Tail phase").phase,
                );
                self.finish_current(sequence);
            }
        }
    }

    fn replay_group(&mut self, sequence: &NumericSequence<S>) -> Result<(), NumericWrappingError> {
        self.entry = sequence.checkpoint();
        if self.index == self.program.len() {
            self.walk(
                sequence,
                Phase::Exhaustion,
                NumericWalkControl::group(None, None),
            );
        } else {
            let expected = self.program[self.index].entry;
            if self.entry.state != expected.state
                || self.entry.selected_len != expected.selected_len
                || self.entry.formatted_len != expected.formatted_len
            {
                return Err(NumericWrappingError::InvalidPrettyBasis);
            }
            self.walk(
                sequence,
                Phase::ProgramOrdinary,
                NumericWalkControl::group(None, None),
            );
        }
        Ok(())
    }

    fn replay_phase(
        &mut self,
        sequence: &mut NumericSequence<S>,
    ) -> Result<(), NumericWrappingError> {
        let group = &self.program[self.index];
        let phase = basis(sequence, self.entry);
        if phase != group.phase || score(&phase) != group.guards {
            return Err(NumericWrappingError::InvalidPrettyBasis);
        }
        if let Some(goal) = group.body_goal {
            self.body_walk(sequence, Phase::ProgramBody, group.reference, goal);
        } else {
            if phase != group.selected {
                return Err(NumericWrappingError::InvalidPrettyBasis);
            }
            self.index += 1;
            self.replay_group(sequence)?;
        }
        Ok(())
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
        let invalid = NumericWrappingError::InvalidPrettyBasis;
        if request.input.text_wrap_style != crate::TextWrapStyle::Pretty {
            return Err(invalid.into());
        }
        loop {
            if matches!(self.phase, Phase::Complete) {
                return Ok(NumericWalkStep::Complete(NumericWalkOutcome::Exhausted));
            }
            let step = match &mut self.active {
                Active::Walk(walk) => {
                    let mut remaining = boundaries[sequence.state.boundary_index..]
                        .iter()
                        .copied()
                        .peekable();
                    walk_numeric_sequence_step(
                        request,
                        sequence,
                        &mut remaining,
                        walk,
                        &mut band,
                        &mut clear,
                    )?
                }
                Active::Tail(tail) => {
                    let step =
                        tail.advance(request, sequence, boundaries, &mut band, &mut clear)?;
                    if matches!(step, NumericWalkStep::Complete(_)) {
                        self.tail_goal = tail.completed_goal().expect("completed nested Tail");
                    }
                    step
                }
                Active::Ready => NumericWalkStep::Complete(NumericWalkOutcome::Exhausted),
            };
            let outcome = match step {
                NumericWalkStep::Receiver(demand) => return Ok(NumericWalkStep::Receiver(demand)),
                NumericWalkStep::Complete(outcome) => outcome,
            };
            match self.phase {
                Phase::Ordinary => {
                    if let Some(ordinary) = finish_ordinary_group(sequence, self.entry, outcome)? {
                        self.reference = Some(reference(&ordinary).ok_or(invalid)?);
                        self.ordinary = Some(ordinary);
                        self.start_tail(sequence, Phase::Tail, None);
                    } else {
                        sequence.restore(self.initial);
                        self.index = 0;
                        self.replay_group(sequence)?;
                    }
                }
                Phase::Tail => {
                    let phase = basis(sequence, self.entry);
                    let guards = score(&phase);
                    self.current = Some(PrettyGroupProgram {
                        entry: self.entry,
                        ordinary: self.ordinary.take().expect("ordinary Pretty reference"),
                        reference: self
                            .reference
                            .take()
                            .expect("actual ordinary terminal and count"),
                        tail_goal: self.tail_goal,
                        selected: phase.clone(),
                        phase,
                        guards,
                        body_goal: None,
                    });
                    if let Some(guards) = guards {
                        self.selection = Some(BodySelection::new(guards));
                        self.next_goal(sequence);
                    } else {
                        self.finish_current(sequence);
                    }
                }
                Phase::Goal => {
                    let group = self.current.as_ref().expect("pending Body trial");
                    let trial = finish_body_trial(sequence, self.entry, group.reference, outcome);
                    #[cfg(test)]
                    if let Some(trial) = &trial {
                        super::tests::observe_trial(
                            self.entry,
                            group.guards.expect("fixed Body guards"),
                            trial,
                        );
                    }
                    self.selection
                        .as_mut()
                        .expect("pending Body goal")
                        .accept(trial);
                    self.next_goal(sequence);
                }
                Phase::Winner => {
                    let group = self.current.as_mut().expect("selected Body group");
                    let replay = finish_body_trial(sequence, self.entry, group.reference, outcome)
                        .ok_or(invalid)?;
                    let winner = self.winner.take().expect("selected Body winner");
                    if replay.value != winner.trial.value || replay.score != winner.trial.score {
                        return Err(invalid.into());
                    }
                    group.body_goal = Some(winner.goal);
                    group.selected = replay.value;
                    self.finish_current(sequence);
                }
                Phase::ProgramOrdinary => {
                    let actual =
                        finish_ordinary_group(sequence, self.entry, outcome)?.ok_or(invalid)?;
                    if actual != self.program[self.index].ordinary {
                        return Err(invalid.into());
                    }
                    if let Some(goal) = self.program[self.index].tail_goal {
                        self.start_tail(sequence, Phase::ProgramTail, Some(goal));
                    } else {
                        self.replay_phase(sequence)?;
                    }
                }
                Phase::ProgramTail => {
                    if self.tail_goal.is_none() {
                        return Err(invalid.into());
                    }
                    self.replay_phase(sequence)?;
                }
                Phase::ProgramBody => {
                    let group = &self.program[self.index];
                    let replay = finish_body_trial(sequence, self.entry, group.reference, outcome)
                        .ok_or(invalid)?;
                    if replay.value != group.selected {
                        return Err(invalid.into());
                    }
                    self.index += 1;
                    self.replay_group(sequence)?;
                }
                Phase::Exhaustion => {
                    if finish_ordinary_group(sequence, self.entry, outcome)?.is_some() {
                        return Err(invalid.into());
                    }
                    self.phase = Phase::Complete;
                    self.active = Active::Ready;
                }
                Phase::Complete => unreachable!("completed Pretty owner has no active work"),
            }
        }
    }
}
