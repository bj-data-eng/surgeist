use super::{policy, validation, *};
use crate::LayoutScalar;

/// Schedule all independent flows or return a typed failure without a prefix.
/// Source positions are caller-owned exact units. Only the mode provider may
/// select a semantic cut or map graphical coverage to current-frame geometry.
pub fn resolve_fragmentation<K: Clone + Eq, S: LayoutScalar, P: FragmentationProvider<S>>(
    input: &FragmentationInputOf<K, S>,
    provider: &mut P,
) -> Result<FragmentationReportOf<K, S>, FragmentationError<P::Error>> {
    validation::input(input).map_err(FragmentationError::InvalidInput)?;
    Engine {
        input,
        provider,
        report: FragmentationReportOf {
            pieces: Vec::new(),
            breaks: Vec::new(),
            fragmentainers: Vec::new(),
        },
    }
    .run()
}

struct Engine<'a, K, S: LayoutScalar, P> {
    input: &'a FragmentationInputOf<K, S>,
    provider: &'a mut P,
    report: FragmentationReportOf<K, S>,
}
struct State<S: LayoutScalar> {
    flow: usize,
    frame: usize,
    position: FragmentationPosition,
    used: S,
    graphical: Option<FragmentationGraphicalProgressOf<S>>,
    forced_applied: Option<FragmentationPosition>,
    start_provenance: FragmentationStart,
}
enum Queried<S: LayoutScalar> {
    ControlAtStart(policy::Force),
    Measured(FragmentationWindowOf<S>, S),
}
struct Accepted<S: LayoutScalar> {
    coverage_end: FragmentationPosition,
    consumed_end: FragmentationPosition,
    body: S,
    edges: (S, S),
    stage: FragmentationStage,
    kind: FragmentationPieceKindOf<S>,
}

fn same_controls<'a>(
    left: impl Iterator<Item = &'a FragmentationBreakControl> + Clone,
    right: impl Iterator<Item = &'a FragmentationBreakControl> + Clone,
) -> bool {
    left.clone().count() == right.clone().count()
        && left.clone().all(|item| {
            left.clone().filter(|value| *value == item).count()
                == right.clone().filter(|value| *value == item).count()
        })
}

fn control_kind<S: LayoutScalar>(
    item: &FragmentationOpportunityOf<S>,
) -> Option<FragmentationControlKind> {
    match item.kind {
        FragmentationOpportunityKindOf::Sibling => Some(FragmentationControlKind::Sibling),
        FragmentationOpportunityKindOf::ModeSpecific => {
            Some(FragmentationControlKind::ModeSpecific)
        }
        _ => None,
    }
}

fn same_candidate<S: LayoutScalar>(
    previous: &[FragmentationOpportunityOf<S>],
    replacement: &[FragmentationOpportunityOf<S>],
    position: FragmentationPosition,
) -> bool {
    for kind in [
        FragmentationControlKind::Sibling,
        FragmentationControlKind::ModeSpecific,
    ] {
        let old = previous
            .iter()
            .filter(|item| item.position == position && control_kind(item) == Some(kind));
        let new = replacement
            .iter()
            .filter(|item| item.position == position && control_kind(item) == Some(kind));
        if old.clone().next().is_some() != new.clone().next().is_some()
            || !same_controls(
                old.clone().flat_map(|item| item.before.iter()),
                new.clone().flat_map(|item| item.before.iter()),
            )
            || !same_controls(
                old.flat_map(|item| item.after.iter()),
                new.flat_map(|item| item.after.iter()),
            )
        {
            return false;
        }
    }
    true
}

impl<K: Clone + Eq, S: LayoutScalar, P: FragmentationProvider<S>> Engine<'_, K, S, P> {
    fn run(mut self) -> Result<FragmentationReportOf<K, S>, FragmentationError<P::Error>> {
        for flow in 0..self.input.flows.len() {
            let input = &self.input.flows[flow];
            let mut state = State {
                flow,
                frame: input.start_fragmentainer,
                position: FragmentationPosition {
                    occurrence: 0,
                    offset: 0,
                },
                used: input.initial_block_offset,
                graphical: None,
                forced_applied: None,
                start_provenance: FragmentationStart::FlowStart,
            };
            while state.position != self.end(flow) {
                self.step(&mut state)?;
            }
        }
        Ok(self.report)
    }

    fn end(&self, flow: usize) -> FragmentationPosition {
        FragmentationPosition {
            occurrence: self.input.flows[flow].occurrences.len(),
            offset: 0,
        }
    }
    fn head_end(state: &State<S>) -> FragmentationPosition {
        FragmentationPosition {
            occurrence: state.position.occurrence + 1,
            offset: 0,
        }
    }
    fn context(&self, state: &State<S>) -> usize {
        self.input.flows[state.flow].context
    }
    fn available_kinds(&self, mut context: usize) -> FragmentationBreakKinds {
        let mut result = FragmentationBreakKinds::NONE;
        loop {
            let item = self.input.contexts[context];
            result = result.union(if item.kind == FragmentationKind::Page {
                FragmentationBreakKinds::SPREAD
            } else {
                item.kind.breaks()
            });
            let Some(parent) = item.parent else {
                return result;
            };
            context = parent;
        }
    }

    fn obtain_frame(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<FragmentainerOf<S>, FragmentationError<P::Error>> {
        if let Some(stored) = self
            .report
            .fragmentainers
            .iter()
            .find(|item| item.context == context && item.index == index)
        {
            return Ok(stored.frame);
        }
        let frame = self
            .provider
            .fragmentainer(context, index)
            .map_err(FragmentationError::Provider)?
            .ok_or(FragmentationError::MissingFragmentainer { context, index })?;
        if !validation::nonnegative(frame.inline_size) || !validation::nonnegative(frame.block_size)
        {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidGeometry,
            ));
        }
        if (index != 0 && frame.breaks.is_empty())
            || !self.available_kinds(context).contains(frame.breaks)
        {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidBoundary,
            ));
        }
        if self.input.contexts[context].parent.is_some() != frame.parent_fragmentainer.is_some() {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidParentFragmentainer,
            ));
        }
        self.report
            .fragmentainers
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.report.fragmentainers.push(FragmentationFrameOf {
            context,
            index,
            frame,
        });
        Ok(frame)
    }
    fn frame(
        &mut self,
        context: usize,
        index: usize,
    ) -> Result<FragmentainerOf<S>, FragmentationError<P::Error>> {
        let first = self.obtain_frame(context, index)?;
        let mut current = first;
        let mut current_context = context;
        while let Some(parent) = self.input.contexts[current_context].parent {
            let parent_index =
                current
                    .parent_fragmentainer
                    .ok_or(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidParentFragmentainer,
                    ))?;
            current = self.obtain_frame(parent, parent_index)?;
            current_context = parent;
        }
        Ok(first)
    }
    fn transition(
        &mut self,
        mut context: usize,
        from: usize,
        to: usize,
    ) -> Result<FragmentationBreakKinds, FragmentationError<P::Error>> {
        let mut old = self.frame(context, from)?;
        let mut new = self.frame(context, to)?;
        let mut crossed = new.breaks;
        while let Some(parent) = self.input.contexts[context].parent {
            let first = old
                .parent_fragmentainer
                .ok_or(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidParentFragmentainer,
                ))?;
            let last = new
                .parent_fragmentainer
                .ok_or(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidParentFragmentainer,
                ))?;
            if last < first {
                return Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidParentFragmentainer,
                ));
            }
            if last > first {
                for index in first + 1..=last {
                    crossed = crossed.union(self.frame(parent, index)?.breaks);
                }
            }
            old = self.frame(parent, first)?;
            new = self.frame(parent, last)?;
            context = parent;
        }
        Ok(crossed)
    }
    fn side_matches(
        &mut self,
        mut context: usize,
        mut index: usize,
        side: FragmentationForcedSide,
    ) -> Result<bool, FragmentationError<P::Error>> {
        loop {
            let frame = self.frame(context, index)?;
            if self.input.contexts[context].kind == FragmentationKind::Page {
                let actual = frame
                    .page_side
                    .ok_or(FragmentationError::MissingPageSide { context, index })?;
                return Ok(match side {
                    FragmentationForcedSide::Left => {
                        actual.physical == FragmentationPhysicalPageSide::Left
                    }
                    FragmentationForcedSide::Right => {
                        actual.physical == FragmentationPhysicalPageSide::Right
                    }
                    FragmentationForcedSide::Recto => {
                        actual.progression == FragmentationProgressionPageSide::Recto
                    }
                    FragmentationForcedSide::Verso => {
                        actual.progression == FragmentationProgressionPageSide::Verso
                    }
                });
            }
            context =
                self.input.contexts[context]
                    .parent
                    .ok_or(FragmentationError::InvalidInput(
                        FragmentationInputError::ContextReference,
                    ))?;
            index = frame
                .parent_fragmentainer
                .ok_or(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidParentFragmentainer,
                ))?;
        }
    }
    fn advance(
        &mut self,
        state: &mut State<S>,
        reason: FragmentationBreakReason,
        stage: FragmentationStage,
    ) -> Result<(), FragmentationError<P::Error>> {
        let from = state.frame;
        let context = self.context(state);
        let mut crossed = FragmentationBreakKinds::NONE;
        loop {
            let next = state
                .frame
                .checked_add(1)
                .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
            crossed = crossed.union(self.transition(context, state.frame, next)?);
            state.frame = next;
            let satisfied = match reason {
                FragmentationBreakReason::Forced { required, side } => {
                    crossed.contains(required)
                        && match side {
                            Some(side) => self.side_matches(context, state.frame, side)?,
                            None => true,
                        }
                }
                _ => true,
            };
            if satisfied {
                break;
            }
        }
        state.used = S::ZERO;
        state.start_provenance = FragmentationStart::AfterBreak(reason);
        self.report
            .breaks
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.report.breaks.push(FragmentationBreakRecord {
            flow: state.flow,
            context,
            from_fragmentainer: from,
            to_fragmentainer: state.frame,
            position: state.position,
            crossed,
            reason,
            stage,
        });
        Ok(())
    }

    fn query(
        &mut self,
        state: &State<S>,
        stage: FragmentationStage,
    ) -> Result<Queried<S>, FragmentationError<P::Error>> {
        let context = self.context(state);
        let frame = self.frame(context, state.frame)?;
        // Break3 guarantees at least one CSS pixel of fragmentainer progress.
        let available = (frame.block_size.max(S::ONE) - state.used).max(S::ZERO);
        let mut scan_after = None;
        let mut scanned_candidates = Vec::new();
        let mut previous: Option<FragmentationWindowOf<S>> = None;
        loop {
            let query = FragmentationQueryOf {
                flow: state.flow,
                context,
                fragmentainer_index: state.frame,
                fragmentainer: frame,
                position: state.position,
                graphical_progress: state.graphical,
                used_block_offset: state.used,
                available_block_size: available,
                stage,
                scan_after,
                start_provenance: state.start_provenance,
            };
            match self
                .provider
                .layout(query)
                .map_err(FragmentationError::Provider)?
            {
                FragmentationReplyOf::ControlAtStart(controls) => {
                    if scan_after.is_some() {
                        return Err(FragmentationError::NoProgress {
                            flow: state.flow,
                            occurrence: state.position.occurrence,
                        });
                    }
                    let usable = state.position.offset == 0
                        || !self.input.flows[state.flow].occurrences[state.position.occurrence]
                            .monolithic;
                    if usable
                        && state.forced_applied != Some(state.position)
                        && let Some(force) = policy::force_controls(
                            &controls.before,
                            &controls.after,
                            self.available_kinds(context),
                        )
                    {
                        return Ok(Queried::ControlAtStart(force));
                    }
                    scan_after = Some(state.position);
                }
                FragmentationReplyOf::Measured(mut window) => {
                    let covered_end = validation::window(
                        &self.input.flows[state.flow],
                        state.position,
                        &mut window,
                    )
                    .map_err(FragmentationError::InvalidInput)?;
                    if scan_after.is_some_and(|previous| covered_end <= previous) {
                        return Err(FragmentationError::NoProgress {
                            flow: state.flow,
                            occurrence: state.position.occurrence,
                        });
                    }
                    if previous.as_ref().is_some_and(|old| {
                        scanned_candidates.iter().any(|&position| {
                            !same_candidate(&old.opportunities, &window.opportunities, position)
                        })
                    }) {
                        return Err(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidWindow,
                        ));
                    }
                    if let (
                        Some(progress),
                        Some(FragmentationEmergencySliceOf::Graphical { extent, .. }),
                    ) = (state.graphical, window.emergency_slice)
                        && extent != progress.extent
                    {
                        return Err(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidSlice,
                        ));
                    }
                    let force = self.first_force(state, &window);
                    if scan_after.is_some_and(|bound| {
                        force.is_some_and(|(position, _, _)| position <= bound)
                    }) {
                        return Err(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidWindow,
                        ));
                    }
                    if matches!(window.end, FragmentationWindowEnd::FlowEnd) || force.is_some() {
                        return Ok(Queried::Measured(window, available));
                    }
                    // A rejected terminal control authorizes discovery only.
                    // The next reply replaces this entire from-origin packet.
                    scan_after = Some(covered_end);
                    scanned_candidates
                        .try_reserve(1)
                        .map_err(|_| FragmentationError::Allocation)?;
                    scanned_candidates.push(covered_end);
                    previous = Some(window);
                }
            }
        }
    }
    fn usable(&self, state: &State<S>, opportunity: &FragmentationOpportunityOf<S>) -> bool {
        let position = opportunity.position;
        position.offset == 0
            || !self.input.flows[state.flow].occurrences[position.occurrence].monolithic
    }
    fn first_force(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
    ) -> Option<(FragmentationPosition, S, policy::Force)> {
        let available = self.available_kinds(self.context(state));
        let mut result: Option<(FragmentationPosition, S, policy::Force)> = None;
        for opportunity in &window.opportunities {
            if opportunity.position == self.end(state.flow)
                || state.forced_applied == Some(opportunity.position)
                || !self.usable(state, opportunity)
            {
                continue;
            }
            let Some(force) = policy::force(opportunity, available) else {
                continue;
            };
            match &mut result {
                Some((position, _, current)) if *position == opportunity.position => {
                    current.merge(force)
                }
                Some((position, _, _)) if *position < opportunity.position => {}
                _ => result = Some((opportunity.position, opportunity.block_offset, force)),
            }
        }
        result
    }
    fn pending_head_force(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
    ) -> Option<(FragmentationPosition, S, policy::Force)> {
        self.first_force(state, window)
            .filter(|(position, _, _)| *position < Self::head_end(state))
    }
    fn accept(
        &mut self,
        state: &mut State<S>,
        accepted: Accepted<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        let start = state.position;
        let end = accepted.coverage_end;
        if end <= start {
            return Err(FragmentationError::NoProgress {
                flow: state.flow,
                occurrence: start.occurrence,
            });
        }
        let flow = &self.input.flows[state.flow];
        let mut coverage = Vec::new();
        let count = end.occurrence - start.occurrence + usize::from(end.offset != 0);
        coverage
            .try_reserve(count)
            .map_err(|_| FragmentationError::Allocation)?;
        for occurrence in start.occurrence..=end.occurrence {
            if occurrence == end.occurrence && end.offset == 0 {
                break;
            }
            let item = &flow.occurrences[occurrence];
            coverage.push(FragmentationCoverage {
                key: item.key.clone(),
                start: if occurrence == start.occurrence {
                    start.offset
                } else {
                    0
                },
                end: if occurrence == end.occurrence {
                    end.offset
                } else {
                    item.source_extent
                },
            });
        }
        let occupied = accepted.body + accepted.edges.0 + accepted.edges.1;
        let used = state.used + occupied;
        if !used.is_finite() || (occupied > S::ZERO && used <= state.used) {
            return Err(FragmentationError::NoProgress {
                flow: state.flow,
                occurrence: start.occurrence,
            });
        }
        self.report
            .pieces
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.report.pieces.push(FragmentationPieceOf {
            flow: state.flow,
            context: flow.context,
            fragmentainer: state.frame,
            start,
            start_provenance: state.start_provenance,
            end: accepted.consumed_end,
            coverage,
            block_offset: state.used,
            block_size: accepted.body,
            cloned_block_start: accepted.edges.0,
            cloned_block_end: accepted.edges.1,
            stage: accepted.stage,
            kind: accepted.kind,
        });
        state.position = accepted.consumed_end;
        state.used = used;
        state.start_provenance = FragmentationStart::InFrame;
        Ok(())
    }

    fn step(&mut self, state: &mut State<S>) -> Result<(), FragmentationError<P::Error>> {
        for stage in [
            FragmentationStage::Strict,
            FragmentationStage::RelaxLineCounts,
            FragmentationStage::RelaxAvoidance,
            FragmentationStage::TruncateBlockEnd,
            FragmentationStage::TruncateBlockStart,
            FragmentationStage::Emergency,
        ] {
            let (window, available) = match self.query(state, stage)? {
                Queried::Measured(window, available) => (window, available),
                Queried::ControlAtStart(force) => {
                    if state.graphical.is_some() {
                        return Err(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidSlice,
                        ));
                    }
                    state.forced_applied = Some(state.position);
                    return self.advance(
                        state,
                        FragmentationBreakReason::Forced {
                            required: force.required,
                            side: force.side,
                        },
                        stage,
                    );
                }
            };
            if state.graphical.is_some() {
                return self.graphical(state, window, available, stage);
            }
            let force = self.first_force(state, &window);
            if let Some((position, _, force)) = force
                && position == state.position
            {
                state.forced_applied = Some(position);
                return self.advance(
                    state,
                    FragmentationBreakReason::Forced {
                        required: force.required,
                        side: force.side,
                    },
                    stage,
                );
            }
            if state.used
                > self
                    .frame(self.context(state), state.frame)?
                    .block_size
                    .max(S::ONE)
            {
                return self.advance(state, FragmentationBreakReason::Deferred, stage);
            }
            if matches!(window.end, FragmentationWindowEnd::FlowEnd)
                && force.is_none()
                && let Some(edges) = policy::edges(&window, window.block_size, available, stage)
            {
                return self.accept(
                    state,
                    Accepted {
                        coverage_end: self.end(state.flow),
                        consumed_end: self.end(state.flow),
                        body: window.block_size,
                        edges,
                        stage,
                        kind: FragmentationPieceKindOf::Content,
                    },
                );
            }
            if let Some((position, body, forced)) = force
                && let Some(edges) = policy::edges(&window, body, available, stage)
            {
                self.accept(
                    state,
                    Accepted {
                        coverage_end: position,
                        consumed_end: position,
                        body,
                        edges,
                        stage,
                        kind: FragmentationPieceKindOf::Content,
                    },
                )?;
                state.forced_applied = Some(position);
                return self.advance(
                    state,
                    FragmentationBreakReason::Forced {
                        required: forced.required,
                        side: forced.side,
                    },
                    stage,
                );
            }
            let has_candidate = window.opportunities.iter().any(|item| {
                item.position > state.position
                    && item.block_offset > S::ZERO
                    && item.position < self.end(state.flow)
                    && self.usable(state, item)
                    && force.is_none_or(|forced| item.position < forced.0)
                    && policy::edges(&window, item.block_offset, available, stage).is_some()
            });
            if has_candidate {
                let next = state
                    .frame
                    .checked_add(1)
                    .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
                let crossed = self.transition(self.context(state), state.frame, next)?;
                let mut selected: Option<&FragmentationOpportunityOf<S>> = None;
                for opportunity in &window.opportunities {
                    if opportunity.position <= state.position
                        || opportunity.block_offset <= S::ZERO
                        || opportunity.position == self.end(state.flow)
                        || !self.usable(state, opportunity)
                        || force.is_some_and(|forced| opportunity.position >= forced.0)
                        || !policy::legal(
                            opportunity,
                            crossed,
                            self.available_kinds(self.context(state)),
                            stage,
                        )
                        || policy::edges(&window, opportunity.block_offset, available, stage)
                            .is_none()
                    {
                        continue;
                    }
                    if selected.is_none_or(|old| {
                        opportunity.block_offset > old.block_offset
                            || (opportunity.block_offset == old.block_offset
                                && opportunity.position > old.position)
                    }) {
                        selected = Some(opportunity);
                    }
                }
                if let Some(selected) = selected {
                    let edges = policy::edges(&window, selected.block_offset, available, stage)
                        .ok_or(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidWindow,
                        ))?;
                    self.accept(
                        state,
                        Accepted {
                            coverage_end: selected.position,
                            consumed_end: selected.position,
                            body: selected.block_offset,
                            edges,
                            stage,
                            kind: FragmentationPieceKindOf::Content,
                        },
                    )?;
                    return self.advance(state, FragmentationBreakReason::Unforced, stage);
                }
            }
            // Ordered zero-size boxes make source progress without an empty slice.
            if window.head.block_size == S::ZERO
                && force.is_none_or(|(position, _, _)| position >= Self::head_end(state))
                && let Some(edges) = policy::edges(&window, S::ZERO, available, stage)
            {
                return self.accept(
                    state,
                    Accepted {
                        coverage_end: Self::head_end(state),
                        consumed_end: Self::head_end(state),
                        body: S::ZERO,
                        edges,
                        stage,
                        kind: FragmentationPieceKindOf::Content,
                    },
                );
            }
            if stage == FragmentationStage::Strict && state.used > S::ZERO {
                return self.advance(state, FragmentationBreakReason::Deferred, stage);
            }
            if stage == FragmentationStage::Emergency {
                return self.emergency(state, window, available);
            }
        }
        unreachable!("the final scheduling stage accepts progress or returns a typed failure")
    }

    fn emergency(
        &mut self,
        state: &mut State<S>,
        window: FragmentationWindowOf<S>,
        available: S,
    ) -> Result<(), FragmentationError<P::Error>> {
        let pending = self.pending_head_force(state, &window);
        if let Some((position, body, force)) = pending
            && window.fallback == FragmentationFallback::Unsliceable
        {
            self.accept(
                state,
                Accepted {
                    coverage_end: position,
                    consumed_end: position,
                    body,
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    kind: FragmentationPieceKindOf::UnsliceableOverflow,
                },
            )?;
            state.forced_applied = Some(position);
            return self.advance(
                state,
                FragmentationBreakReason::Forced {
                    required: force.required,
                    side: force.side,
                },
                FragmentationStage::Emergency,
            );
        }
        if pending.is_some() && window.head.block_size <= available {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        if window.head.block_size <= available {
            return self.accept(
                state,
                Accepted {
                    coverage_end: Self::head_end(state),
                    consumed_end: Self::head_end(state),
                    body: window.head.block_size,
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    kind: FragmentationPieceKindOf::Content,
                },
            );
        }
        if window.fallback == FragmentationFallback::Unsliceable {
            return self.accept(
                state,
                Accepted {
                    coverage_end: Self::head_end(state),
                    consumed_end: Self::head_end(state),
                    body: window.head.block_size,
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    kind: FragmentationPieceKindOf::UnsliceableOverflow,
                },
            );
        }
        let slice = window
            .emergency_slice
            .ok_or(FragmentationError::MissingSlice { flow: state.flow })?;
        match slice {
            FragmentationEmergencySliceOf::Source { end, block_size } => {
                let end = validation::position(&self.input.flows[state.flow], end)
                    .map_err(FragmentationError::InvalidInput)?;
                if end <= state.position || block_size == S::ZERO {
                    return Err(FragmentationError::NoProgress {
                        flow: state.flow,
                        occurrence: state.position.occurrence,
                    });
                }
                if end > Self::head_end(state)
                    || pending.is_some_and(|(position, body, _)| {
                        end > position || (end == position && block_size != body)
                    })
                    || (end == Self::head_end(state) && block_size != window.head.block_size)
                    || !validation::nonnegative(block_size)
                    || block_size <= S::ZERO
                    || block_size > available
                    || (end.offset != 0
                        && self.input.flows[state.flow].occurrences[end.occurrence].monolithic)
                {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
                self.accept(
                    state,
                    Accepted {
                        coverage_end: end,
                        consumed_end: end,
                        body: block_size,
                        edges: (S::ZERO, S::ZERO),
                        stage: FragmentationStage::Emergency,
                        kind: FragmentationPieceKindOf::Content,
                    },
                )?;
                if end.offset != 0 {
                    self.advance(
                        state,
                        FragmentationBreakReason::Unforced,
                        FragmentationStage::Emergency,
                    )?;
                }
                Ok(())
            }
            FragmentationEmergencySliceOf::Graphical {
                end,
                extent,
                block_size,
            } => {
                // Graphical coverage cannot discharge an internal semantic force.
                if pending.is_some() {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
                self.graphical_slice(
                    state,
                    end,
                    extent,
                    block_size,
                    available,
                    window.head.block_size,
                )
            }
        }
    }
    fn graphical(
        &mut self,
        state: &mut State<S>,
        window: FragmentationWindowOf<S>,
        available: S,
        stage: FragmentationStage,
    ) -> Result<(), FragmentationError<P::Error>> {
        if self.pending_head_force(state, &window).is_some() {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        let progress = state.graphical.ok_or(FragmentationError::InvalidInput(
            FragmentationInputError::InvalidSlice,
        ))?;
        if let Some(edges) = policy::edges(&window, window.head.block_size, available, stage) {
            let kind = FragmentationPieceKindOf::GraphicalSlice {
                coverage: FragmentationGraphicalCoverageOf {
                    start: progress.consumed,
                    end: progress.extent,
                    extent: progress.extent,
                },
            };
            self.accept(
                state,
                Accepted {
                    coverage_end: Self::head_end(state),
                    consumed_end: Self::head_end(state),
                    body: window.head.block_size,
                    edges,
                    stage,
                    kind,
                },
            )?;
            state.graphical = None;
            return Ok(());
        }
        if state.used > S::ZERO {
            return self.advance(state, FragmentationBreakReason::Deferred, stage);
        }
        // Truncation and slicing still follow the common stage order.
        for next_stage in [
            FragmentationStage::RelaxLineCounts,
            FragmentationStage::RelaxAvoidance,
            FragmentationStage::TruncateBlockEnd,
            FragmentationStage::TruncateBlockStart,
            FragmentationStage::Emergency,
        ] {
            let (next, available) = match self.query(state, next_stage)? {
                Queried::Measured(window, available) => (window, available),
                Queried::ControlAtStart(_) => {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
            };
            if self.pending_head_force(state, &next).is_some() {
                return Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidSlice,
                ));
            }
            if let Some(edges) = policy::edges(&next, next.head.block_size, available, next_stage) {
                self.accept(
                    state,
                    Accepted {
                        coverage_end: Self::head_end(state),
                        consumed_end: Self::head_end(state),
                        body: next.head.block_size,
                        edges,
                        stage: next_stage,
                        kind: FragmentationPieceKindOf::GraphicalSlice {
                            coverage: FragmentationGraphicalCoverageOf {
                                start: progress.consumed,
                                end: progress.extent,
                                extent: progress.extent,
                            },
                        },
                    },
                )?;
                state.graphical = None;
                return Ok(());
            }
            if next_stage == FragmentationStage::Emergency {
                let Some(FragmentationEmergencySliceOf::Graphical {
                    end,
                    extent,
                    block_size,
                }) = next.emergency_slice
                else {
                    return Err(FragmentationError::MissingSlice { flow: state.flow });
                };
                return self.graphical_slice(
                    state,
                    end,
                    extent,
                    block_size,
                    available,
                    next.head.block_size,
                );
            }
        }
        unreachable!("graphical continuation has a final emergency stage")
    }
    fn graphical_slice(
        &mut self,
        state: &mut State<S>,
        end: S,
        extent: S,
        body: S,
        available: S,
        head_size: S,
    ) -> Result<(), FragmentationError<P::Error>> {
        let start = state.graphical.map_or(S::ZERO, |value| value.consumed);
        if end <= start || body == S::ZERO {
            return Err(FragmentationError::NoProgress {
                flow: state.flow,
                occurrence: state.position.occurrence,
            });
        }
        if !validation::nonnegative(extent)
            || extent <= S::ZERO
            || !validation::nonnegative(end)
            || end <= start
            || end > extent
            || (end == extent && body != head_size)
            || !validation::nonnegative(body)
            || body <= S::ZERO
            || body > available
            || state.graphical.is_some_and(|old| old.extent != extent)
        {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        self.accept(
            state,
            Accepted {
                coverage_end: Self::head_end(state),
                consumed_end: if end == extent {
                    Self::head_end(state)
                } else {
                    state.position
                },
                body,
                edges: (S::ZERO, S::ZERO),
                stage: FragmentationStage::Emergency,
                kind: FragmentationPieceKindOf::GraphicalSlice {
                    coverage: FragmentationGraphicalCoverageOf { start, end, extent },
                },
            },
        )?;
        if end < extent {
            state.graphical = Some(FragmentationGraphicalProgressOf {
                consumed: end,
                extent,
            });
            self.advance(
                state,
                FragmentationBreakReason::Unforced,
                FragmentationStage::Emergency,
            )?;
        } else {
            state.graphical = None;
        }
        Ok(())
    }
}
