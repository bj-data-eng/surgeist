use super::{policy, validation, *};
use crate::LayoutScalar;
use std::sync::Arc;

/// Schedule all independent flows or return a typed failure without a prefix.
/// Source positions are caller-owned exact units. Only the mode provider may
/// select a semantic cut or map graphical coverage to current-frame geometry.
pub fn resolve_fragmentation<K: Clone + Eq, S: LayoutScalar, P: FragmentationProvider<S>>(
    input: &FragmentationInputOf<K, S>,
    provider: &mut P,
) -> Result<FragmentationReportOf<K, S>, FragmentationError<P::Error>> {
    resolve_with_feedback(input, provider, |_| Ok(()))
}

/// A completed scheduler step. The mode may retain accepted production or fail
/// the complete operation; it cannot rewrite the selected cuts or transitions.
pub(crate) struct AcceptedStep<'a, K, S: LayoutScalar> {
    pub(crate) flow: usize,
    pub(crate) from_frame: usize,
    pub(crate) to_frame: usize,
    pub(crate) position: FragmentationPosition,
    pub(crate) graphical_progress: Option<FragmentationGraphicalProgressOf<S>>,
    pub(crate) pieces: &'a [FragmentationPieceOf<K, S>],
    pub(crate) breaks: &'a [FragmentationBreakRecord],
    pub(crate) flow_complete: bool,
    pub(crate) appearances: &'a [ScopeAppearanceOf<S>],
}

/// An actual selected child transition participated even when the parent
/// checkpoint did not advance. This is private adoption evidence, not coverage.
#[derive(Clone, Copy)]
pub(crate) struct ScopeAppearanceOf<S: LayoutScalar> {
    pub(crate) owner: FragmentationPosition,
    pub(crate) checkpoint: FragmentationCutOf<S>,
    pub(crate) normal_end: S,
    pub(crate) occupied_end: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum ScopeEndpointOf<S: LayoutScalar> {
    Advance(FragmentationCutOf<S>),
    Retain(FragmentationCutOf<S>),
}
impl<S: LayoutScalar> ScopeEndpointOf<S> {
    pub(crate) fn checkpoint(self) -> FragmentationCutOf<S> {
        match self {
            Self::Advance(cut) | Self::Retain(cut) => cut,
        }
    }
    fn retained(self) -> bool {
        matches!(self, Self::Retain(_))
    }
}

pub(crate) fn resolve_with_feedback<K, S, P, F>(
    input: &FragmentationInputOf<K, S>,
    provider: &mut P,
    feedback: F,
) -> Result<FragmentationReportOf<K, S>, FragmentationError<P::Error>>
where
    K: Clone + Eq,
    S: LayoutScalar,
    P: FragmentationProvider<S>,
    F: FnMut(AcceptedStep<'_, K, S>) -> Result<(), P::Error>,
{
    let mut execution =
        Execution::new(Arc::new(input.clone())).map_err(FragmentationError::InvalidInput)?;
    let mut feedback = feedback;
    while !execution.poll(provider, &mut feedback)? {}
    Ok(execution.finish().into_report())
}

#[derive(Debug)]
struct ExecutionIdentity;

/// A scheduler-certified initial preparation point, never a public resume token.
pub(crate) struct ScopeTicket<S: LayoutScalar> {
    execution: Arc<ExecutionIdentity>,
    generation: usize,
    query: FragmentationQueryOf<S>,
    owner: FragmentationPosition,
    principal_end: FragmentationPosition,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScopeRegistrationError {
    StalePlacement,
    InvalidOwner,
    InvalidProjection,
    Allocation,
}

/// Block's geometric projection of an actual selected child transition.
/// The offset is an occupied-prefix coordinate in that fragmentainer, not a
/// distance from a coalesced parent query's changing source cursor.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScopeProjectionOf<S: LayoutScalar> {
    pub(crate) child_break_index: usize,
    pub(crate) endpoint: ScopeEndpointOf<S>,
    pub(crate) block_offset: S,
    pub(crate) normal_end: S,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScopeHeadProjectionOf<S: LayoutScalar> {
    pub(crate) child_piece_index: usize,
    pub(crate) cut: FragmentationCutOf<S>,
    pub(crate) block_offset: S,
}

/// Complete scheduler evidence; consumers can read but cannot mint its report.
pub(crate) struct CompletedExecution<K, S: LayoutScalar> {
    identity: Arc<ExecutionIdentity>,
    input: Arc<FragmentationInputOf<K, S>>,
    report: FragmentationReportOf<K, S>,
    qualified_heads: Vec<bool>,
    selected_forces: Vec<Option<policy::Force>>,
    selected_origins: Vec<Option<policy::SelectedPolicyOf<S>>>,
    accepted_source: Vec<usize>,
    source_frames: Vec<AcceptedSourceFrame>,
}
/// Minted only by a successful step that accepted actual source or a completed
/// nested source window. Merely advancing a fragmentainer cannot issue one.
struct AcceptedSourceFrame {
    flow: usize,
    frame: usize,
}
struct AcceptedOwner {
    flow: usize,
    owner: FragmentationPosition,
    child: Arc<ExecutionIdentity>,
}
impl<K, S: LayoutScalar> CompletedExecution<K, S> {
    pub(crate) fn report(&self) -> &FragmentationReportOf<K, S> {
        &self.report
    }
    /// Read Engine-issued indivisibility evidence for this report's piece.
    /// A graphical kind alone does not qualify a piece as an accepted head.
    pub(crate) fn head_qualified(&self, piece_index: usize) -> bool {
        self.qualified_heads
            .get(piece_index)
            .copied()
            .unwrap_or(false)
    }
    pub(crate) fn into_report(self) -> FragmentationReportOf<K, S> {
        self.report
    }
}

struct RegisteredScope<K, S: LayoutScalar> {
    flow: usize,
    initial_frame: usize,
    owner: FragmentationPosition,
    principal_end: FragmentationPosition,
    child: Arc<CompletedExecution<K, S>>,
    projections: Vec<ScopeProjectionOf<S>>,
    heads: Vec<ScopeHeadProjectionOf<S>>,
    applied: Vec<bool>,
    source_windows: Vec<bool>,
    active: bool,
    retired: bool,
}

/// Only state actually changed by this unpublished step needs rollback.
enum ScopeUndo {
    SourceWindow {
        scope: usize,
        window: usize,
    },
    Lifecycle {
        scope: usize,
        active: bool,
        retired: bool,
    },
    Applied {
        scope: usize,
        projection: usize,
        previous: bool,
    },
}
impl ScopeUndo {
    fn restore<K, S: LayoutScalar>(journal: &mut Vec<Self>, scopes: &mut [RegisteredScope<K, S>]) {
        while let Some(change) = journal.pop() {
            match change {
                Self::SourceWindow { scope, window } => {
                    scopes[scope].source_windows[window] = false
                }
                Self::Lifecycle {
                    scope,
                    active,
                    retired,
                } => {
                    scopes[scope].active = active;
                    scopes[scope].retired = retired;
                }
                Self::Applied {
                    scope,
                    projection,
                    previous,
                } => {
                    scopes[scope].applied[projection] = previous;
                }
            }
        }
    }
}

/// Invocation-owned scheduling state. A failed provider preparation leaves the
/// current step unaccepted so an internal scope driver can satisfy its request
/// and replay against the same immutable input.
pub(crate) struct Execution<K, S: LayoutScalar> {
    input: Arc<FragmentationInputOf<K, S>>,
    report: FragmentationReportOf<K, S>,
    qualified_heads: Vec<bool>,
    selected_forces: Vec<Option<policy::Force>>,
    selected_origins: Vec<Option<policy::SelectedPolicyOf<S>>>,
    accepted_source: Vec<usize>,
    source_frames: Vec<AcceptedSourceFrame>,
    accepted_owners: Vec<AcceptedOwner>,
    flow: usize,
    state: Option<State<S>>,
    identity: Arc<ExecutionIdentity>,
    generation: usize,
    paused_query: Option<FragmentationQueryOf<S>>,
    scopes: Vec<RegisteredScope<K, S>>,
}
impl<K: Clone + Eq, S: LayoutScalar> Execution<K, S> {
    pub(crate) fn new(
        input: Arc<FragmentationInputOf<K, S>>,
    ) -> Result<Self, FragmentationInputError> {
        validation::input(&input)?;
        Ok(Self {
            input,
            report: FragmentationReportOf {
                pieces: Vec::new(),
                breaks: Vec::new(),
                fragmentainers: Vec::new(),
            },
            qualified_heads: Vec::new(),
            selected_forces: Vec::new(),
            selected_origins: Vec::new(),
            accepted_source: Vec::new(),
            source_frames: Vec::new(),
            accepted_owners: Vec::new(),
            flow: 0,
            state: None,
            identity: Arc::new(ExecutionIdentity),
            generation: 0,
            paused_query: None,
            scopes: Vec::new(),
        })
    }
    pub(crate) fn scope_ticket(
        &self,
        query: FragmentationQueryOf<S>,
        owner: FragmentationPosition,
        principal_end: FragmentationPosition,
    ) -> Result<ScopeTicket<S>, ScopeRegistrationError> {
        if self.paused_query != Some(query) {
            return Err(ScopeRegistrationError::StalePlacement);
        }
        let flow = &self.input.flows[query.flow];
        if owner.offset != 0
            || principal_end.offset != 0
            || owner < query.position
            || owner.occurrence.checked_add(1) != Some(principal_end.occurrence)
            || [owner, principal_end].iter().any(|position| {
                flow.occurrences
                    .get(position.occurrence)
                    .is_none_or(|item| item.source_extent != 0 || item.monolithic)
            })
        {
            return Err(ScopeRegistrationError::InvalidOwner);
        }
        Ok(ScopeTicket {
            execution: self.identity.clone(),
            generation: self.generation,
            query,
            owner,
            principal_end,
        })
    }

    pub(crate) fn scope_participated(
        &self,
        owner: FragmentationPosition,
        child: &CompletedExecution<K, S>,
    ) -> bool {
        self.accepted_owners.iter().any(|accepted| {
            accepted.flow == self.flow
                && accepted.owner == owner
                && Arc::ptr_eq(&accepted.child, &child.identity)
        }) || self.scopes.iter().any(|scope| {
            scope.flow == self.flow
                && scope.owner == owner
                && scope.active
                && Arc::ptr_eq(&scope.child.identity, &child.identity)
        })
    }

    pub(crate) fn register_scope(
        &mut self,
        ticket: ScopeTicket<S>,
        child: Arc<CompletedExecution<K, S>>,
        projections: Vec<ScopeProjectionOf<S>>,
        heads: Vec<ScopeHeadProjectionOf<S>>,
    ) -> Result<(), ScopeRegistrationError> {
        if !Arc::ptr_eq(&self.identity, &ticket.execution)
            || self.generation != ticket.generation
            || self.paused_query != Some(ticket.query)
        {
            return Err(ScopeRegistrationError::StalePlacement);
        }
        let replacement = self.scopes.iter().position(|scope| {
            !scope.retired && scope.flow == ticket.query.flow && scope.owner == ticket.owner
        });
        if replacement.is_some_and(|index| self.scopes[index].active) {
            return Err(ScopeRegistrationError::InvalidOwner);
        }
        if child.input.contexts != self.input.contexts {
            return Err(ScopeRegistrationError::InvalidProjection);
        }
        let mut previous: Option<&ScopeProjectionOf<S>> = None;
        let mut extent = None;
        let mut child_flow = None;
        for (index, head) in heads.iter().enumerate() {
            let Some(piece) = child.report.pieces.get(head.child_piece_index) else {
                return Err(ScopeRegistrationError::InvalidProjection);
            };
            let flow = &child.input.flows[piece.flow];
            if !validation::nonnegative(head.block_offset)
                || piece.context != ticket.query.context
                || piece.fragmentainer < ticket.query.fragmentainer_index
                || flow.context != ticket.query.context
                || flow.start_fragmentainer != ticket.query.fragmentainer_index
                || child_flow.is_some_and(|old| old != piece.flow)
                || heads[..index].iter().any(|old| {
                    old.child_piece_index == head.child_piece_index || old.cut == head.cut
                })
            {
                return Err(ScopeRegistrationError::InvalidProjection);
            }
            child_flow = Some(piece.flow);
            match head.cut {
                FragmentationCutOf::Source(position) if position == ticket.principal_end => {}
                FragmentationCutOf::Graphical { anchor, progress }
                    if anchor == ticket.owner
                        && validation::nonnegative(progress.consumed)
                        && validation::nonnegative(progress.extent)
                        && progress.consumed > S::ZERO
                        && progress.consumed < progress.extent
                        && extent.is_none_or(|old| old == progress.extent) =>
                {
                    extent = Some(progress.extent);
                }
                _ => return Err(ScopeRegistrationError::InvalidProjection),
            }
        }
        for projection in &projections {
            let Some(selected) = child.report.breaks.get(projection.child_break_index) else {
                return Err(ScopeRegistrationError::InvalidProjection);
            };
            if !validation::nonnegative(projection.block_offset)
                || !projection.normal_end.is_finite()
                || projection.normal_end > projection.block_offset
                || selected.context != ticket.query.context
                || selected.from_fragmentainer < ticket.query.fragmentainer_index
                || selected.to_fragmentainer <= selected.from_fragmentainer
                || child_flow.is_some_and(|flow| flow != selected.flow)
                || previous.is_some_and(|old| {
                    old.child_break_index >= projection.child_break_index
                        || cut_order(old.endpoint.checkpoint(), projection.endpoint.checkpoint())
                            == core::cmp::Ordering::Greater
                        || (old.endpoint.checkpoint() == projection.endpoint.checkpoint()
                            && !projection.endpoint.retained())
                })
            {
                return Err(ScopeRegistrationError::InvalidProjection);
            }
            child_flow = Some(selected.flow);
            let flow = &child.input.flows[selected.flow];
            if flow.context != ticket.query.context
                || flow.start_fragmentainer != ticket.query.fragmentainer_index
            {
                return Err(ScopeRegistrationError::InvalidProjection);
            }
            let deferred = matches!(selected.reason, FragmentationBreakReason::Deferred);
            let initial_deferral = deferred
                && !projection.endpoint.retained()
                && projection.endpoint.checkpoint() == FragmentationCutOf::Source(ticket.owner);
            if projection.endpoint.retained() {
                // The kernel minted this ordinal only after actual source
                // acceptance (or authenticated nested child acceptance). Frame
                // advancement and zero-extent owner tokens cannot mint one.
                let before = previous.map_or(0, |old| child.accepted_source[old.child_break_index]);
                if child.accepted_source[projection.child_break_index] <= before
                    || projection.endpoint.checkpoint()
                        != previous.map_or(FragmentationCutOf::Source(ticket.owner), |old| {
                            old.endpoint.checkpoint()
                        })
                {
                    return Err(ScopeRegistrationError::InvalidProjection);
                }
            }
            if initial_deferral
                && (projection.endpoint.checkpoint() != FragmentationCutOf::Source(ticket.owner)
                    || projection.block_offset <= S::ZERO
                    || (ticket.owner == ticket.query.position
                        && projection.block_offset != ticket.query.used_block_offset)
                    || flow.initial_block_offset <= S::ZERO
                    || selected.position
                        != (FragmentationPosition {
                            occurrence: 0,
                            offset: 0,
                        })
                    || selected.from_fragmentainer != flow.start_fragmentainer
                    || selected.stage != FragmentationStage::Strict
                    || child.report.breaks[..projection.child_break_index]
                        .iter()
                        .any(|record| record.flow == selected.flow)
                    || child.report.pieces.iter().any(|piece| {
                        piece.flow == selected.flow
                            && piece.fragmentainer <= selected.from_fragmentainer
                    }))
            {
                return Err(ScopeRegistrationError::InvalidProjection);
            }
            if deferred
                && !initial_deferral
                && !projection.endpoint.retained()
                && !heads.iter().any(|head| {
                    let piece = &child.report.pieces[head.child_piece_index];
                    head.cut == projection.endpoint.checkpoint()
                        && head.block_offset == projection.block_offset
                        && piece.flow == selected.flow
                        && piece.context == selected.context
                        && piece.fragmentainer == selected.from_fragmentainer
                        && piece.end == selected.position
                })
            {
                return Err(ScopeRegistrationError::InvalidProjection);
            }
            match projection.endpoint.checkpoint() {
                FragmentationCutOf::Source(position)
                    if (initial_deferral || projection.endpoint.retained())
                        && position == ticket.owner => {}
                FragmentationCutOf::Source(position) if position == ticket.principal_end => {}
                FragmentationCutOf::Graphical { anchor, progress }
                    if anchor == ticket.owner
                        && validation::nonnegative(progress.consumed)
                        && validation::nonnegative(progress.extent)
                        && progress.consumed > S::ZERO
                        && progress.consumed < progress.extent
                        && extent.is_none_or(|old| old == progress.extent) =>
                {
                    extent = Some(progress.extent);
                }
                _ => return Err(ScopeRegistrationError::InvalidProjection),
            }
            previous = Some(projection);
        }
        if (!projections.is_empty() || !heads.is_empty())
            && !child.report.fragmentainers.iter().any(|frame| {
                frame.context == ticket.query.context
                    && frame.index == ticket.query.fragmentainer_index
                    && frame.frame == ticket.query.fragmentainer
            })
        {
            return Err(ScopeRegistrationError::InvalidProjection);
        }
        let mut applied = Vec::new();
        applied
            .try_reserve(projections.len())
            .map_err(|_| ScopeRegistrationError::Allocation)?;
        applied.resize(projections.len(), false);
        let mut source_windows = Vec::new();
        source_windows
            .try_reserve(child.source_frames.len())
            .map_err(|_| ScopeRegistrationError::Allocation)?;
        source_windows.resize(child.source_frames.len(), false);
        if replacement.is_none() {
            self.scopes
                .try_reserve(1)
                .map_err(|_| ScopeRegistrationError::Allocation)?;
        }
        let prepared = RegisteredScope {
            flow: ticket.query.flow,
            initial_frame: ticket.query.fragmentainer_index,
            owner: ticket.owner,
            principal_end: ticket.principal_end,
            child,
            projections,
            heads,
            applied,
            source_windows,
            active: false,
            retired: false,
        };
        if let Some(index) = replacement {
            self.scopes[index] = prepared;
        } else {
            self.scopes.push(prepared);
        }
        Ok(())
    }
    pub(crate) fn poll<P, F>(
        &mut self,
        provider: &mut P,
        feedback: &mut F,
    ) -> Result<bool, FragmentationError<P::Error>>
    where
        P: FragmentationProvider<S>,
        F: FnMut(AcceptedStep<'_, K, S>) -> Result<(), P::Error>,
    {
        while self.flow < self.input.flows.len() {
            let input = &self.input.flows[self.flow];
            let mut state = self.state.clone().unwrap_or(State {
                flow: self.flow,
                frame: input.start_fragmentainer,
                position: FragmentationPosition {
                    occurrence: 0,
                    offset: 0,
                },
                used: input.initial_block_offset,
                graphical: None,
                forced_applied: None,
                start_provenance: FragmentationStart::FlowStart,
                source_acceptance: 0,
                accepted_boundary: None,
            });
            let end = FragmentationPosition {
                occurrence: input.occurrences.len(),
                offset: 0,
            };
            if state.position == end {
                self.flow += 1;
                self.state = None;
                continue;
            }
            let from_frame = state.frame;
            let prior_source = state.source_acceptance;
            let first_piece = self.report.pieces.len();
            let first_break = self.report.breaks.len();
            let next_generation = self
                .generation
                .checked_add(1)
                .ok_or(FragmentationError::Arithmetic { flow: self.flow })?;
            let mut scope_undo = Vec::new();
            let mut appearances = Vec::new();
            self.source_frames
                .try_reserve(1)
                .map_err(|_| FragmentationError::Allocation)?;
            self.accepted_owners
                .try_reserve(self.scopes.len())
                .map_err(|_| FragmentationError::Allocation)?;
            self.paused_query = None;
            let result = Engine {
                input: &self.input,
                provider,
                report: &mut self.report,
                qualified_heads: &mut self.qualified_heads,
                selected_forces: &mut self.selected_forces,
                selected_origins: &mut self.selected_origins,
                accepted_source: &mut self.accepted_source,
                appearances: &mut appearances,
                paused_query: &mut self.paused_query,
                scopes: &mut self.scopes,
                scope_undo: &mut scope_undo,
            }
            .step(&mut state);
            if let Err(error) = result {
                ScopeUndo::restore(&mut scope_undo, &mut self.scopes);
                self.report.pieces.truncate(first_piece);
                self.qualified_heads.truncate(first_piece);
                self.report.breaks.truncate(first_break);
                self.selected_forces.truncate(first_break);
                self.selected_origins.truncate(first_break);
                self.accepted_source.truncate(first_break);
                return Err(error);
            }
            let accepted = feedback(AcceptedStep {
                flow: self.flow,
                from_frame,
                to_frame: state.frame,
                position: state.position,
                graphical_progress: state.graphical,
                pieces: &self.report.pieces[first_piece..],
                breaks: &self.report.breaks[first_break..],
                flow_complete: state.position == end,
                appearances: &appearances,
            });
            if let Err(error) = accepted {
                ScopeUndo::restore(&mut scope_undo, &mut self.scopes);
                self.report.pieces.truncate(first_piece);
                self.qualified_heads.truncate(first_piece);
                self.report.breaks.truncate(first_break);
                self.selected_forces.truncate(first_break);
                self.selected_origins.truncate(first_break);
                self.accepted_source.truncate(first_break);
                return Err(FragmentationError::Provider(error));
            }
            if state.source_acceptance > prior_source
                && !self
                    .source_frames
                    .iter()
                    .any(|accepted| accepted.flow == self.flow && accepted.frame == from_frame)
            {
                self.source_frames.push(AcceptedSourceFrame {
                    flow: self.flow,
                    frame: from_frame,
                });
            }
            // Placement participation outlives transition scheduling. A retired
            // owner can still occur in the current frame's final Block replay.
            // Only successful feedback makes this immutable history visible.
            for scope in self.scopes.iter().filter(|scope| scope.active) {
                if !self.accepted_owners.iter().any(|accepted| {
                    accepted.flow == scope.flow
                        && accepted.owner == scope.owner
                        && Arc::ptr_eq(&accepted.child, &scope.child.identity)
                }) {
                    self.accepted_owners.push(AcceptedOwner {
                        flow: scope.flow,
                        owner: scope.owner,
                        child: scope.child.identity.clone(),
                    });
                }
            }
            self.state = Some(state);
            self.generation = next_generation;
            // No scope index escapes this step. After feedback succeeds the
            // journal can no longer roll back, and obsolete payloads can drop.
            drop(scope_undo);
            self.scopes.retain(|scope| !scope.retired);
            return Ok(false);
        }
        Ok(true)
    }
    pub(crate) fn finish(self) -> CompletedExecution<K, S> {
        assert!(
            self.flow == self.input.flows.len(),
            "only a completed execution produces a report"
        );
        CompletedExecution {
            identity: self.identity,
            input: self.input,
            report: self.report,
            qualified_heads: self.qualified_heads,
            selected_forces: self.selected_forces,
            selected_origins: self.selected_origins,
            accepted_source: self.accepted_source,
            source_frames: self.source_frames,
        }
    }
}

struct Engine<'a, K, S: LayoutScalar, P> {
    input: &'a FragmentationInputOf<K, S>,
    provider: &'a mut P,
    report: &'a mut FragmentationReportOf<K, S>,
    qualified_heads: &'a mut Vec<bool>,
    selected_forces: &'a mut Vec<Option<policy::Force>>,
    selected_origins: &'a mut Vec<Option<policy::SelectedPolicyOf<S>>>,
    accepted_source: &'a mut Vec<usize>,
    appearances: &'a mut Vec<ScopeAppearanceOf<S>>,
    paused_query: &'a mut Option<FragmentationQueryOf<S>>,
    scopes: &'a mut [RegisteredScope<K, S>],
    scope_undo: &'a mut Vec<ScopeUndo>,
}
#[derive(Clone)]
struct State<S: LayoutScalar> {
    flow: usize,
    frame: usize,
    position: FragmentationPosition,
    used: S,
    graphical: Option<FragmentationGraphicalProgressOf<S>>,
    forced_applied: Option<FragmentationPosition>,
    start_provenance: FragmentationStart,
    source_acceptance: usize,
    accepted_boundary: Option<AcceptedBoundary<S>>,
}
#[derive(Clone)]
struct AcceptedBoundary<S: LayoutScalar> {
    frame: usize,
    cut: FragmentationCutOf<S>,
    policy: policy::SelectedPolicyOf<S>,
}
enum Queried<S: LayoutScalar> {
    ControlAtStart(policy::Force),
    PlacementDeferred,
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

/// The public checkpoint projects a site into report/cursor space. A retained
/// interior has its own authenticated source order and occupied geometry.
#[derive(Clone, Copy)]
enum SiteKind {
    Offered {
        opportunity: Option<usize>,
        receipt: Option<(usize, usize)>,
    },
    Interior {
        receipt: (usize, usize),
    },
}
#[derive(Clone, Copy)]
struct SelectionSite<S: LayoutScalar> {
    cut: FragmentationCutOf<S>,
    body: S,
    kind: SiteKind,
}
impl<S: LayoutScalar> SelectionSite<S> {
    fn receipt(self) -> Option<(usize, usize)> {
        match self.kind {
            SiteKind::Offered { receipt, .. } => receipt,
            SiteKind::Interior { receipt } => Some(receipt),
        }
    }
    fn interior(self) -> bool {
        matches!(self.kind, SiteKind::Interior { .. })
    }
}
struct SiteFit<S: LayoutScalar> {
    edges: (S, S),
    stage: FragmentationStage,
    overflow: bool,
}
#[derive(Clone, Copy)]
struct ForcedCut<S: LayoutScalar> {
    site: SelectionSite<S>,
    force: policy::Force,
    floor: FragmentationStage,
}
impl<S: LayoutScalar> ForcedCut<S> {
    fn reason(self) -> FragmentationBreakReason {
        FragmentationBreakReason::Forced {
            required: self.force.required,
            side: self.force.side,
        }
    }
}

struct CertifiedHead<S: LayoutScalar> {
    cut: FragmentationCutOf<S>,
    body: S,
    qualified: bool,
}

fn current_cut<S: LayoutScalar>(state: &State<S>) -> FragmentationCutOf<S> {
    state
        .graphical
        .map_or(FragmentationCutOf::Source(state.position), |progress| {
            FragmentationCutOf::Graphical {
                anchor: state.position,
                progress,
            }
        })
}

fn stage_rank(stage: FragmentationStage) -> u8 {
    match stage {
        FragmentationStage::Strict => 0,
        FragmentationStage::RelaxLineCounts => 1,
        FragmentationStage::RelaxAvoidance => 2,
        FragmentationStage::TruncateBlockEnd => 3,
        FragmentationStage::TruncateBlockStart => 4,
        FragmentationStage::Emergency => 5,
    }
}

fn cut_order<S: LayoutScalar>(
    left: FragmentationCutOf<S>,
    right: FragmentationCutOf<S>,
) -> core::cmp::Ordering {
    left.source_position()
        .cmp(&right.source_position())
        .then_with(|| match (left, right) {
            (FragmentationCutOf::Source(_), FragmentationCutOf::Graphical { .. }) => {
                core::cmp::Ordering::Less
            }
            (FragmentationCutOf::Graphical { .. }, FragmentationCutOf::Source(_)) => {
                core::cmp::Ordering::Greater
            }
            (
                FragmentationCutOf::Graphical { progress: a, .. },
                FragmentationCutOf::Graphical { progress: b, .. },
            ) => a
                .consumed
                .partial_cmp(&b.consumed)
                .expect("validated finite graphical cuts"),
            _ => core::cmp::Ordering::Equal,
        })
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
        let old = previous.iter().filter(|item| {
            item.cut.source_position() == position && control_kind(item) == Some(kind)
        });
        let new = replacement.iter().filter(|item| {
            item.cut.source_position() == position && control_kind(item) == Some(kind)
        });
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
    fn scope_lifecycle(
        &mut self,
        index: usize,
        active: bool,
        retired: bool,
    ) -> Result<(), FragmentationError<P::Error>> {
        let scope = &self.scopes[index];
        if scope.active == active && scope.retired == retired {
            return Ok(());
        }
        self.scope_undo
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.scope_undo.push(ScopeUndo::Lifecycle {
            scope: index,
            active: scope.active,
            retired: scope.retired,
        });
        self.scopes[index].active = active;
        self.scopes[index].retired = retired;
        Ok(())
    }

    fn scope_application(
        &mut self,
        scope: usize,
        projection: usize,
    ) -> Result<(), FragmentationError<P::Error>> {
        let previous = self.scopes[scope].applied[projection];
        if previous {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidBoundary,
            ));
        }
        self.scope_undo
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.scope_undo.push(ScopeUndo::Applied {
            scope,
            projection,
            previous,
        });
        self.scopes[scope].applied[projection] = true;
        Ok(())
    }

    fn adopt_source_window(
        &mut self,
        scope: usize,
        state: &mut State<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        let registered = &self.scopes[scope];
        let Some(window) = registered.child.source_frames.iter().position(|accepted| {
            accepted.frame == state.frame
                && registered.child.input.flows[accepted.flow].context == self.context(state)
        }) else {
            return Ok(());
        };
        if registered.source_windows[window] {
            return Ok(());
        }
        self.scope_undo
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        let next = state
            .source_acceptance
            .checked_add(1)
            .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
        self.scope_undo
            .push(ScopeUndo::SourceWindow { scope, window });
        self.scopes[scope].source_windows[window] = true;
        state.source_acceptance = next;
        Ok(())
    }

    fn end(&self, flow: usize) -> FragmentationPosition {
        FragmentationPosition {
            occurrence: self.input.flows[flow].occurrences.len(),
            offset: 0,
        }
    }
    fn anchor_end(state: &State<S>) -> FragmentationPosition {
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
        self.advance_record(state, reason, stage, None)
    }

    fn advance_record(
        &mut self,
        state: &mut State<S>,
        reason: FragmentationBreakReason,
        stage: FragmentationStage,
        force: Option<policy::Force>,
    ) -> Result<(), FragmentationError<P::Error>> {
        let provenance_matches = match (reason, force) {
            (FragmentationBreakReason::Forced { required, side }, Some(force)) => {
                required == force.required && side == force.side
            }
            (FragmentationBreakReason::Forced { .. }, None) => false,
            (_, None) => true,
            (_, Some(_)) => false,
        };
        if !provenance_matches {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidBoundary,
            ));
        }
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
        for index in 0..self.scopes.len() {
            let scope = &self.scopes[index];
            if scope.flow != state.flow || scope.active || scope.retired {
                continue;
            }
            // A candidate child was prepared for an owner not yet accepted.
            // A parent relocation requires a fresh preparation and ticket.
            self.scope_lifecycle(index, false, true)?;
        }
        state.used = S::ZERO;
        state.accepted_boundary = None;
        state.start_provenance = FragmentationStart::AfterBreak(reason);
        self.report
            .breaks
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.selected_forces
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.selected_origins
            .try_reserve(1)
            .map_err(|_| FragmentationError::Allocation)?;
        self.accepted_source
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
        self.selected_forces.push(force);
        self.selected_origins.push(None);
        self.accepted_source.push(state.source_acceptance);
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
            let reply = match self.provider.layout(query) {
                Ok(reply) => reply,
                Err(error) => {
                    *self.paused_query = Some(query);
                    return Err(FragmentationError::Provider(error));
                }
            };
            match reply {
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
                    if stage == FragmentationStage::Strict
                        && state.used > S::ZERO
                        && self.placement_limit(state) == Some(state.position)
                    {
                        return Ok(Queried::PlacementDeferred);
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
                    self.validate_scope_window(state, &window, covered_end)?;
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
                    for item in &window.opportunities {
                        if let FragmentationCutOf::Graphical { anchor, progress } = item.cut
                            && anchor == state.position
                            && let Some(old) = state.graphical
                        {
                            if old.extent != progress.extent {
                                return Err(FragmentationError::InvalidInput(
                                    FragmentationInputError::InvalidSlice,
                                ));
                            }
                            if progress.consumed <= old.consumed {
                                return Err(FragmentationError::NoProgress {
                                    flow: state.flow,
                                    occurrence: anchor.occurrence,
                                });
                            }
                        }
                    }
                    let force = self.first_force(state, &window);
                    if scan_after.is_some_and(|bound| {
                        force.is_some_and(|force| {
                            cut_order(force.site.cut, FragmentationCutOf::Source(bound))
                                != core::cmp::Ordering::Greater
                        })
                    }) {
                        return Err(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidWindow,
                        ));
                    }
                    if matches!(window.end, FragmentationWindowEnd::FlowEnd)
                        || force.is_some()
                        || self.pending_cut_limit(state).is_some_and(|limit| {
                            cut_order(limit, FragmentationCutOf::Source(covered_end))
                                != core::cmp::Ordering::Greater
                        })
                        || self
                            .placement_limit(state)
                            .is_some_and(|limit| limit <= covered_end)
                    {
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
        let position = opportunity.cut.source_position();
        match opportunity.cut {
            FragmentationCutOf::Source(_) => {
                position.offset == 0
                    || !self.input.flows[state.flow].occurrences[position.occurrence].monolithic
            }
            FragmentationCutOf::Graphical { progress, .. } => {
                position >= state.position
                    && (position != state.position
                        || state.graphical.is_none_or(|old| {
                            old.extent == progress.extent && progress.consumed > old.consumed
                        }))
            }
        }
    }
    fn projection_site(&self, state: &State<S>, scope: usize, index: usize) -> SelectionSite<S> {
        let projection = self.scopes[scope].projections[index];
        let receipt = (scope, index);
        SelectionSite {
            cut: projection.endpoint.checkpoint(),
            body: projection.block_offset - state.used,
            kind: if projection.endpoint.retained() {
                SiteKind::Interior { receipt }
            } else {
                SiteKind::Offered {
                    opportunity: None,
                    receipt: Some(receipt),
                }
            },
        }
    }
    fn site_order(&self, a: SelectionSite<S>, b: SelectionSite<S>) -> core::cmp::Ordering {
        cut_order(a.cut, b.cut).then_with(|| match (a.kind, b.kind) {
            (SiteKind::Offered { .. }, SiteKind::Interior { .. }) => core::cmp::Ordering::Less,
            (SiteKind::Interior { .. }, SiteKind::Offered { .. }) => core::cmp::Ordering::Greater,
            (SiteKind::Interior { receipt: a }, SiteKind::Interior { receipt: b }) => a.cmp(&b),
            _ => core::cmp::Ordering::Equal,
        })
    }
    fn first_force(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
    ) -> Option<ForcedCut<S>> {
        let available = self.available_kinds(self.context(state));
        let pending_limit = self.pending_cut_limit(state);
        let mut result: Option<(FragmentationPosition, S, policy::Force)> = None;
        for opportunity in &window.opportunities {
            if !matches!(opportunity.cut, FragmentationCutOf::Source(_)) {
                continue;
            }
            if opportunity.cut.source_position() == self.end(state.flow)
                || state.forced_applied == Some(opportunity.cut.source_position())
                || !self.usable(state, opportunity)
                || pending_limit.is_some_and(|limit| {
                    cut_order(opportunity.cut, limit) == core::cmp::Ordering::Greater
                })
            {
                continue;
            }
            let Some(force) = policy::force(opportunity, available) else {
                continue;
            };
            match &mut result {
                Some((position, _, current)) if *position == opportunity.cut.source_position() => {
                    current.merge(force)
                }
                Some((position, _, _)) if *position < opportunity.cut.source_position() => {}
                _ => {
                    result = Some((
                        opportunity.cut.source_position(),
                        opportunity.block_offset,
                        force,
                    ))
                }
            }
        }
        let mut result = result.map(|(position, body, force)| ForcedCut {
            site: SelectionSite {
                cut: FragmentationCutOf::Source(position),
                body,
                kind: SiteKind::Offered {
                    opportunity: None,
                    receipt: None,
                },
            },
            force,
            floor: FragmentationStage::Strict,
        });
        let covered_end = match window.end {
            FragmentationWindowEnd::FlowEnd => self.end(state.flow),
            FragmentationWindowEnd::PotentialControl(position) => position,
        };
        for (scope_index, scope) in self.scopes.iter().enumerate() {
            if scope.flow != state.flow
                || scope.retired
                || (!scope.active && state.frame != scope.initial_frame)
            {
                continue;
            }
            for (projection_index, projection) in scope.projections.iter().enumerate() {
                let selected = &scope.child.report.breaks[projection.child_break_index];
                if scope.applied[projection_index]
                    || pending_limit.is_some_and(|limit| {
                        cut_order(projection.endpoint.checkpoint(), limit)
                            == core::cmp::Ordering::Greater
                    })
                    || selected.from_fragmentainer != state.frame
                    || !matches!(selected.reason, FragmentationBreakReason::Forced { .. })
                    || projection.endpoint.checkpoint().source_position() < state.position
                    || projection.endpoint.checkpoint().source_position() > covered_end
                    || matches!(projection.endpoint.checkpoint(), FragmentationCutOf::Graphical { anchor, .. } if anchor == covered_end)
                    || projection.block_offset < state.used
                    || projection.block_offset - state.used > window.block_size
                    || matches!(projection.endpoint.checkpoint(), FragmentationCutOf::Graphical { anchor, progress }
                        if anchor == state.position && state.graphical.is_some_and(|old|
                            old.extent != progress.extent || old.consumed > progress.consumed || (old.consumed == progress.consumed && !projection.endpoint.retained())))
                {
                    continue;
                }
                let site = self.projection_site(state, scope_index, projection_index);
                let child_force = scope.child.selected_forces[projection.child_break_index]
                    .expect("advance_record retains provenance for every selected Forced break");
                if result.is_some_and(|old| {
                    self.site_order(site, old.site) == core::cmp::Ordering::Equal
                }) {
                    let old = result.as_mut().expect("the matching forced cut exists");
                    old.force.merge(child_force);
                    old.site.body = projection.block_offset - state.used;
                    if stage_rank(selected.stage) > stage_rank(old.floor) {
                        old.floor = selected.stage;
                    }
                    old.site = self.projection_site(state, scope_index, projection_index);
                } else if result
                    .is_none_or(|old| self.site_order(site, old.site) == core::cmp::Ordering::Less)
                {
                    result = Some(ForcedCut {
                        site: self.projection_site(state, scope_index, projection_index),
                        force: child_force,
                        floor: selected.stage,
                    });
                }
            }
        }
        result
    }
    fn pending_head_force(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
    ) -> Option<ForcedCut<S>> {
        self.first_force(state, window)
            .filter(|force| force.site.cut.source_position() < window.head.end)
    }

    fn projected_transition(
        &self,
        state: &State<S>,
        cut: FragmentationCutOf<S>,
    ) -> Option<(usize, usize)> {
        self.scopes
            .iter()
            .enumerate()
            .find_map(|(scope_index, scope)| {
                if scope.flow != state.flow || scope.retired {
                    return None;
                }
                scope
                    .projections
                    .iter()
                    .enumerate()
                    .find_map(|(index, projection)| {
                        let selected = &scope.child.report.breaks[projection.child_break_index];
                        (!scope.applied[index]
                            && projection.endpoint.checkpoint() == cut
                            && !projection.endpoint.retained()
                            && !matches!(selected.reason, FragmentationBreakReason::Deferred)
                            && selected.from_fragmentainer == state.frame)
                            .then_some((scope_index, index))
                    })
            })
    }

    fn placement_limit(&self, state: &State<S>) -> Option<FragmentationPosition> {
        self.scopes
            .iter()
            .filter(|scope| {
                scope.flow == state.flow
                    && !scope.active
                    && !scope.retired
                    && scope.initial_frame == state.frame
            })
            .filter_map(|scope| {
                scope
                    .projections
                    .iter()
                    .any(|projection| {
                        projection.endpoint.checkpoint() == FragmentationCutOf::Source(scope.owner)
                            && !projection.endpoint.retained()
                            && matches!(
                                scope.child.report.breaks[projection.child_break_index].reason,
                                FragmentationBreakReason::Deferred
                            )
                    })
                    .then_some(scope.owner)
            })
            .min()
    }

    fn deferred_transition(&self, state: &State<S>) -> Option<(usize, usize)> {
        let cut = current_cut(state);
        self.scopes
            .iter()
            .enumerate()
            .find_map(|(scope_index, scope)| {
                if scope.flow != state.flow || !scope.active || scope.retired {
                    return None;
                }
                scope
                    .projections
                    .iter()
                    .enumerate()
                    .find_map(|(index, projection)| {
                        let selected = &scope.child.report.breaks[projection.child_break_index];
                        (!scope.applied[index]
                            && projection.endpoint.checkpoint() == cut
                            && !projection.endpoint.retained()
                            && projection.block_offset == state.used
                            && selected.from_fragmentainer == state.frame
                            && matches!(selected.reason, FragmentationBreakReason::Deferred))
                        .then_some((scope_index, index))
                    })
            })
    }

    fn projection_policy(
        &mut self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
        site: SelectionSite<S>,
        stage: FragmentationStage,
    ) -> Result<Option<policy::SelectedPolicyOf<S>>, FragmentationError<P::Error>> {
        let receipt = site
            .receipt()
            .expect("projected policy carries its exact child receipt");
        let scope = &self.scopes[receipt.0];
        let projection = &scope.projections[receipt.1];
        let selected = &scope.child.report.breaks[projection.child_break_index];
        if stage_rank(stage) < stage_rank(selected.stage) {
            return Ok(None);
        }
        let origin = scope.child.selected_origins[projection.child_break_index]
            .clone()
            .ok_or(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidBoundary,
            ))?;
        let cut = projection.endpoint.checkpoint();
        let interior = projection.endpoint.retained();
        // A retained checkpoint represents accepted source inside the owner.
        // Raw before/after controls at its Source anchor still belong to the
        // separate before-owner boundary; only parent ancestry applies here.
        let parent = match site.kind {
            SiteKind::Offered {
                opportunity: Some(index),
                ..
            } => {
                let offered =
                    window
                        .opportunities
                        .get(index)
                        .ok_or(FragmentationError::InvalidInput(
                            FragmentationInputError::InvalidWindow,
                        ))?;
                if offered.cut != site.cut || offered.block_offset != site.body {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidWindow,
                    ));
                }
                Some(offered)
            }
            // These are authenticated interior/current-boundary contexts, not
            // a choice among offered candidates. A selected offered site keeps
            // its exact index above, including duplicate cut/offset policies.
            SiteKind::Interior { .. }
            | SiteKind::Offered {
                opportunity: None, ..
            } => window
                .opportunities
                .iter()
                .find(|opportunity| opportunity.cut == cut),
        };
        let accepted_parent = state.accepted_boundary.as_ref().filter(|site| {
            site.frame == state.frame && site.cut == cut && cut == current_cut(state)
        });
        let next = state
            .frame
            .checked_add(1)
            .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
        let crossed = self.transition(self.context(state), state.frame, next)?;
        let available = self.available_kinds(self.context(state));
        match (parent, accepted_parent) {
            (Some(parent), _) if interior => Ok(policy::legal_projected_interior(
                &origin, parent, crossed, available, stage,
            )
            .then(|| origin.project_interior(parent, available))),
            (Some(parent), _) => Ok(policy::legal_projected(
                &origin, parent, crossed, available, stage,
            )
            .then(|| origin.project(parent, available))),
            (None, Some(parent)) if interior => Ok(policy::legal_projected_accepted_interior(
                &origin,
                &parent.policy,
                crossed,
                stage,
            )
            .then(|| origin.project_accepted_interior(&parent.policy))),
            (None, Some(parent)) => {
                Ok(
                    policy::legal_projected_accepted(&origin, &parent.policy, crossed, stage)
                        .then(|| origin.project_accepted(&parent.policy)),
                )
            }
            (None, None) => Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidBoundary,
            )),
        }
    }

    fn defer(
        &mut self,
        state: &mut State<S>,
        window: &FragmentationWindowOf<S>,
        stage: FragmentationStage,
    ) -> Result<bool, FragmentationError<P::Error>> {
        let receipt = self.deferred_transition(state);
        let origin = if let Some(receipt) = receipt {
            let Some(origin) = self.projection_policy(
                state,
                window,
                self.projection_site(state, receipt.0, receipt.1),
                stage,
            )?
            else {
                return Ok(false);
            };
            Some(origin)
        } else {
            // Deferred metadata records the real queried boundary. Capture is
            // provenance only; an adopting parent still checks cumulative policy.
            window
                .opportunities
                .iter()
                .find(|opportunity| {
                    opportunity.cut == current_cut(state) && opportunity.block_offset == S::ZERO
                })
                .map(|opportunity| {
                    policy::SelectedPolicyOf::capture(
                        opportunity,
                        self.available_kinds(self.context(state)),
                    )
                })
                .transpose()
                .map_err(|_| FragmentationError::Allocation)?
                .or_else(|| {
                    state
                        .accepted_boundary
                        .as_ref()
                        .filter(|site| site.frame == state.frame && site.cut == current_cut(state))
                        .map(|site| site.policy.clone())
                })
        };
        self.advance_selected(
            state,
            FragmentationBreakReason::Deferred,
            stage,
            receipt,
            None,
        )?;
        *self
            .selected_origins
            .last_mut()
            .expect("Deferred recorded its transition") = origin;
        Ok(true)
    }

    fn certified_head(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
    ) -> Option<CertifiedHead<S>> {
        let current = current_cut(state);
        self.scopes
            .iter()
            .filter(|scope| scope.flow == state.flow && !scope.retired)
            .flat_map(|scope| {
                scope.heads.iter().filter_map(move |head| {
                    let piece = &scope.child.report.pieces[head.child_piece_index];
                    (piece.fragmentainer == state.frame
                        && cut_order(current, head.cut) == core::cmp::Ordering::Less
                        && cut_order(head.cut, FragmentationCutOf::Source(window.head.end))
                            != core::cmp::Ordering::Greater
                        && head.block_offset >= state.used)
                        .then_some(CertifiedHead {
                            cut: head.cut,
                            body: head.block_offset - state.used,
                            qualified: scope.child.qualified_heads[head.child_piece_index],
                        })
                })
            })
            .min_by(|a, b| cut_order(a.cut, b.cut))
    }

    fn validate_scope_window(
        &self,
        state: &State<S>,
        window: &FragmentationWindowOf<S>,
        covered_end: FragmentationPosition,
    ) -> Result<(), FragmentationError<P::Error>> {
        for scope in self
            .scopes
            .iter()
            .filter(|scope| scope.flow == state.flow && !scope.retired)
        {
            for (index, projection) in scope.projections.iter().enumerate() {
                let selected = &scope.child.report.breaks[projection.child_break_index];
                if scope.applied[index]
                    || selected.from_fragmentainer != state.frame
                    || projection.endpoint.checkpoint().source_position() > covered_end
                    || matches!(projection.endpoint.checkpoint(), FragmentationCutOf::Graphical { anchor, .. } if anchor == covered_end)
                {
                    continue;
                }
                if projection.endpoint.checkpoint().source_position() < state.position
                    || projection.block_offset < state.used
                    || projection.block_offset - state.used > window.block_size
                    || (!projection.endpoint.retained()
                        && projection.endpoint.checkpoint()
                            == FragmentationCutOf::Source(state.position)
                        && projection.block_offset != state.used)
                    || (!projection.endpoint.retained()
                        && projection.endpoint.checkpoint()
                            == FragmentationCutOf::Source(covered_end)
                        && projection.block_offset - state.used != window.block_size)
                    || (!projection.endpoint.retained()
                        && window.opportunities.iter().any(|opportunity| {
                            opportunity.cut == projection.endpoint.checkpoint()
                                && opportunity.block_offset != projection.block_offset - state.used
                        }))
                {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidWindow,
                    ));
                }
                if !projection.endpoint.retained()
                    && matches!(selected.reason, FragmentationBreakReason::Deferred)
                    && projection.endpoint.checkpoint() == FragmentationCutOf::Source(scope.owner)
                    && projection.endpoint.checkpoint().source_position() > state.position
                    && !window.opportunities.iter().any(|opportunity| {
                        opportunity.cut == projection.endpoint.checkpoint()
                            && opportunity.block_offset == projection.block_offset - state.used
                    })
                {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidWindow,
                    ));
                }
                if let FragmentationCutOf::Graphical { anchor, progress } =
                    projection.endpoint.checkpoint()
                    && anchor == state.position
                    && state.graphical.is_some_and(|old| {
                        old.extent != progress.extent
                            || old.consumed > progress.consumed
                            || (old.consumed == progress.consumed
                                && !projection.endpoint.retained()
                                && !matches!(selected.reason, FragmentationBreakReason::Deferred))
                    })
                {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
            }
            for head in &scope.heads {
                let piece = &scope.child.report.pieces[head.child_piece_index];
                if piece.fragmentainer != state.frame
                    || cut_order(head.cut, current_cut(state)) != core::cmp::Ordering::Greater
                    || head.cut.source_position() > covered_end
                {
                    continue;
                }
                let child_frame = scope
                    .child
                    .report
                    .fragmentainers
                    .iter()
                    .find(|frame| frame.context == piece.context && frame.index == state.frame);
                let actual_frame = self
                    .report
                    .fragmentainers
                    .iter()
                    .find(|frame| frame.context == piece.context && frame.index == state.frame);
                if head.block_offset < state.used
                    || head.block_offset - state.used > window.block_size
                    || child_frame.is_none()
                    || child_frame != actual_frame
                {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidWindow,
                    ));
                }
            }
        }
        Ok(())
    }

    fn pending_projection_before(&self, state: &State<S>, cut: FragmentationCutOf<S>) -> bool {
        self.scopes.iter().any(|scope| {
            scope.flow == state.flow
                && !scope.retired
                && scope
                    .projections
                    .iter()
                    .enumerate()
                    .any(|(index, projection)| {
                        !scope.applied[index]
                            && cut_order(projection.endpoint.checkpoint(), cut)
                                == core::cmp::Ordering::Less
                    })
        })
    }

    fn pending_cut_limit(&self, state: &State<S>) -> Option<FragmentationCutOf<S>> {
        self.scopes
            .iter()
            .filter(|scope| scope.flow == state.flow && !scope.retired)
            .flat_map(|scope| {
                scope
                    .projections
                    .iter()
                    .enumerate()
                    .filter_map(|(index, projection)| {
                        (!scope.applied[index]).then_some(projection.endpoint.checkpoint())
                    })
            })
            .min_by(|a, b| cut_order(*a, *b))
    }

    fn stage_allows_projection(
        &self,
        state: &State<S>,
        cut: FragmentationCutOf<S>,
        stage: FragmentationStage,
    ) -> bool {
        self.projected_transition(state, cut)
            .is_none_or(|(scope, index)| {
                let scope = &self.scopes[scope];
                stage_rank(stage)
                    >= stage_rank(
                        scope.child.report.breaks[scope.projections[index].child_break_index].stage,
                    )
            })
    }

    fn advance_selected(
        &mut self,
        state: &mut State<S>,
        reason: FragmentationBreakReason,
        stage: FragmentationStage,
        receipt: Option<(usize, usize)>,
        force: Option<policy::Force>,
    ) -> Result<(), FragmentationError<P::Error>> {
        if let Some((scope, projection)) = receipt {
            let registered = &self.scopes[scope];
            let selected = &registered.projections[projection];
            let retained = selected.endpoint.retained();
            let before = projection.checked_sub(1).map_or(0, |index| {
                registered.child.accepted_source[registered.projections[index].child_break_index]
            });
            let has_source = registered.child.accepted_source[selected.child_break_index] > before;
            let appearance = ScopeAppearanceOf {
                owner: registered.owner,
                checkpoint: selected.endpoint.checkpoint(),
                normal_end: selected.normal_end,
                occupied_end: selected.block_offset,
            };
            if retained && !has_source {
                return Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidBoundary,
                ));
            }
            // Activate before frame advancement can retire an unaccepted
            // preparation. Both lifecycle and receipt consumption are journaled.
            if has_source {
                self.scope_lifecycle(scope, true, false)?;
                if retained {
                    self.appearances
                        .try_reserve(1)
                        .map_err(|_| FragmentationError::Allocation)?;
                    self.appearances.push(appearance);
                }
                self.adopt_source_window(scope, state)?;
            }
            self.scope_application(scope, projection)?;
        }
        self.advance_record(state, reason, stage, force)?;
        if let Some((scope, projection)) = receipt {
            let registered = &self.scopes[scope];
            let selected = &registered.child.report.breaks
                [registered.projections[projection].child_break_index];
            let actual = self
                .report
                .breaks
                .last()
                .expect("advance recorded its transition");
            let reason_compatible = match (actual.reason, selected.reason) {
                (
                    FragmentationBreakReason::Forced {
                        required: actual, ..
                    },
                    FragmentationBreakReason::Forced {
                        required: child, ..
                    },
                ) => actual.contains(child),
                _ => actual.reason == selected.reason,
            };
            if actual.context != selected.context
                || actual.from_fragmentainer != selected.from_fragmentainer
                || actual.to_fragmentainer != selected.to_fragmentainer
                || actual.crossed != selected.crossed
                || !reason_compatible
                || stage_rank(actual.stage) < stage_rank(selected.stage)
                || [selected.from_fragmentainer, selected.to_fragmentainer]
                    .iter()
                    .any(|&index| {
                        let child = registered.child.report.fragmentainers.iter().find(|frame| {
                            frame.context == selected.context && frame.index == index
                        });
                        let parent = self.report.fragmentainers.iter().find(|frame| {
                            frame.context == selected.context && frame.index == index
                        });
                        child.is_none() || parent.is_none() || child != parent
                    })
            {
                return Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidBoundary,
                ));
            }
            // Every successful adoption of this exact selected transition
            // carries its policy custody, including emergency acceptance.
            // A normal selection subsequently adds its current parent facts.
            let inherited = registered.child.selected_origins
                [registered.projections[projection].child_break_index]
                .clone();
            *self
                .selected_origins
                .last_mut()
                .expect("advance recorded its policy slot") = inherited;
        }
        Ok(())
    }

    fn accept_site(
        &mut self,
        state: &mut State<S>,
        site: SelectionSite<S>,
        window: &FragmentationWindowOf<S>,
        fit: SiteFit<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        if site.interior() {
            let receipt = site
                .receipt()
                .expect("interior sites carry their exact receipt");
            let projection = self.scopes[receipt.0].projections[receipt.1];
            if site.cut.source_position() > state.position {
                // An ordinary prefix ends before the private owner interior.
                // Its measured extent is independent of the child's peak.
                let prefix = window
                    .opportunities
                    .iter()
                    .find(|item| item.cut == site.cut)
                    .ok_or(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidWindow,
                    ))?;
                self.accept(
                    state,
                    Accepted {
                        coverage_end: site.cut.source_position(),
                        consumed_end: site.cut.source_position(),
                        body: prefix.block_offset,
                        edges: fit.edges,
                        stage: fit.stage,
                        kind: FragmentationPieceKindOf::Content,
                    },
                )?;
            }
            // The authenticated child window supplies source progress even
            // without an ordinary prefix. Preserve its normal endpoint apart
            // from the occupied peak used to evaluate physical fit.
            let normal = projection.normal_end + fit.edges.0 + fit.edges.1;
            if !normal.is_finite() {
                return Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidGeometry,
                ));
            }
            state.used = normal;
            state.graphical = match site.cut {
                FragmentationCutOf::Graphical { progress, .. } => Some(progress),
                FragmentationCutOf::Source(_) => None,
            };
            return Ok(());
        }
        let (coverage_end, consumed_end, kind, graphical) = match site.cut {
            FragmentationCutOf::Source(position) => {
                let kind = if let Some(progress) = state.graphical {
                    FragmentationPieceKindOf::GraphicalSlice {
                        anchor: state.position,
                        coverage: FragmentationGraphicalCoverageOf {
                            start: progress.consumed,
                            end: progress.extent,
                            extent: progress.extent,
                        },
                    }
                } else if fit.overflow {
                    FragmentationPieceKindOf::UnsliceableOverflow
                } else {
                    FragmentationPieceKindOf::Content
                };
                (position, position, kind, None)
            }
            FragmentationCutOf::Graphical { anchor, progress } => {
                let start = if anchor == state.position {
                    state.graphical.map_or(S::ZERO, |old| old.consumed)
                } else {
                    S::ZERO
                };
                (
                    FragmentationPosition {
                        occurrence: anchor.occurrence + 1,
                        offset: 0,
                    },
                    anchor,
                    FragmentationPieceKindOf::GraphicalSlice {
                        anchor,
                        coverage: FragmentationGraphicalCoverageOf {
                            start,
                            end: progress.consumed,
                            extent: progress.extent,
                        },
                    },
                    Some(progress),
                )
            }
        };
        if coverage_end > state.position {
            self.accept(
                state,
                Accepted {
                    coverage_end,
                    consumed_end,
                    body: site.body,
                    edges: fit.edges,
                    stage: fit.stage,
                    kind,
                },
            )?;
        } else if site.body != S::ZERO {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidWindow,
            ));
        }
        state.graphical = graphical;
        Ok(())
    }

    fn accept_forced(
        &mut self,
        state: &mut State<S>,
        forced: ForcedCut<S>,
        window: &FragmentationWindowOf<S>,
        fit: SiteFit<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        let stage = fit.stage;
        self.accept_site(state, forced.site, window, fit)?;
        // A private retained force is inside the owner; it does not discharge
        // the separate raw before-owner control at the same public anchor.
        if !forced.site.interior()
            && let FragmentationCutOf::Source(position) = forced.site.cut
        {
            state.forced_applied = Some(position);
        }
        self.advance_selected(
            state,
            forced.reason(),
            stage,
            forced.site.receipt(),
            Some(forced.force),
        )
    }

    fn accept_certified_head(
        &mut self,
        state: &mut State<S>,
        head: CertifiedHead<S>,
        available: S,
        force: Option<ForcedCut<S>>,
        window: &FragmentationWindowOf<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        if head.body > available && !head.qualified {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        let frame = state.frame;
        let boundary = window
            .opportunities
            .iter()
            .find(|opportunity| opportunity.cut == head.cut)
            .map(|opportunity| {
                policy::SelectedPolicyOf::capture(
                    opportunity,
                    self.available_kinds(self.context(state)),
                )
            })
            .transpose()
            .map_err(|_| FragmentationError::Allocation)?;
        if let Some(force) = force.filter(|force| force.site.cut == head.cut) {
            self.accept_forced(
                state,
                force,
                window,
                SiteFit {
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    overflow: head.body > available,
                },
            )?;
        } else {
            let (coverage_end, consumed_end, kind, graphical) = match head.cut {
                FragmentationCutOf::Source(position) => (
                    position,
                    position,
                    if head.body > available {
                        FragmentationPieceKindOf::UnsliceableOverflow
                    } else {
                        FragmentationPieceKindOf::Content
                    },
                    None,
                ),
                FragmentationCutOf::Graphical { anchor, progress } => {
                    let start = if anchor == state.position {
                        state.graphical.map_or(S::ZERO, |old| old.consumed)
                    } else {
                        S::ZERO
                    };
                    (
                        FragmentationPosition {
                            occurrence: anchor.occurrence + 1,
                            offset: 0,
                        },
                        anchor,
                        FragmentationPieceKindOf::GraphicalSlice {
                            anchor,
                            coverage: FragmentationGraphicalCoverageOf {
                                start,
                                end: progress.consumed,
                                extent: progress.extent,
                            },
                        },
                        Some(progress),
                    )
                }
            };
            self.accept(
                state,
                Accepted {
                    coverage_end,
                    consumed_end,
                    body: head.body,
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    kind,
                },
            )?;
            state.graphical = graphical;
        }
        if head.qualified {
            *self
                .qualified_heads
                .last_mut()
                .expect("the certified endpoint accepted one head") = true;
        }
        if state.frame == frame {
            state.accepted_boundary = boundary.map(|policy| AcceptedBoundary {
                frame,
                cut: head.cut,
                policy,
            });
        }
        Ok(())
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
        let accepted_cut = match accepted.kind {
            FragmentationPieceKindOf::GraphicalSlice { anchor, coverage }
                if coverage.end < coverage.extent =>
            {
                FragmentationCutOf::Graphical {
                    anchor,
                    progress: FragmentationGraphicalProgressOf {
                        consumed: coverage.end,
                        extent: coverage.extent,
                    },
                }
            }
            _ => FragmentationCutOf::Source(accepted.consumed_end),
        };
        if self.pending_projection_before(state, accepted_cut)
            || !self.stage_allows_projection(state, accepted_cut, accepted.stage)
        {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
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
        self.qualified_heads
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
        self.qualified_heads.push(matches!(
            accepted.kind,
            FragmentationPieceKindOf::UnsliceableOverflow
        ));
        if self.report.pieces.last().is_some_and(|piece| {
            piece
                .coverage
                .iter()
                .any(|coverage| coverage.end > coverage.start)
        }) {
            state.source_acceptance = state
                .source_acceptance
                .checked_add(1)
                .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
        }
        for index in 0..self.scopes.len() {
            let scope = &self.scopes[index];
            if scope.flow != state.flow || scope.retired {
                continue;
            }
            let active = scope.active || (start <= scope.owner && end > scope.owner);
            let retired = accepted.consumed_end.occurrence > scope.principal_end.occurrence;
            self.scope_lifecycle(index, active, retired)?;
            if active {
                self.adopt_source_window(index, state)?;
            }
        }
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
                    return self.advance_record(
                        state,
                        FragmentationBreakReason::Forced {
                            required: force.required,
                            side: force.side,
                        },
                        stage,
                        Some(force),
                    );
                }
                Queried::PlacementDeferred => {
                    return self.advance(state, FragmentationBreakReason::Deferred, stage);
                }
            };
            if state.graphical.is_some()
                && state.used
                    > self
                        .frame(self.context(state), state.frame)?
                        .block_size
                        .max(S::ONE)
            {
                if self.defer(state, &window, stage)? {
                    return Ok(());
                }
                continue;
            }
            if state.graphical.is_some() {
                return self.graphical(state, window, available, stage);
            }
            let force = self.first_force(state, &window);
            if let Some(force) = force
                && force.site.cut == FragmentationCutOf::Source(state.position)
                && force.site.body == S::ZERO
                && stage_rank(stage) >= stage_rank(force.floor)
            {
                return self.accept_forced(
                    state,
                    force,
                    &window,
                    SiteFit {
                        edges: (S::ZERO, S::ZERO),
                        stage,
                        overflow: false,
                    },
                );
            }
            if state.used
                > self
                    .frame(self.context(state), state.frame)?
                    .block_size
                    .max(S::ONE)
            {
                if self.defer(state, &window, stage)? {
                    return Ok(());
                }
                continue;
            }
            if matches!(window.end, FragmentationWindowEnd::FlowEnd)
                && force.is_none()
                && !self.pending_projection_before(
                    state,
                    FragmentationCutOf::Source(self.end(state.flow)),
                )
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
            if let Some(forced) = force
                && stage_rank(stage) >= stage_rank(forced.floor)
                && let Some(edges) = policy::edges(&window, forced.site.body, available, stage)
            {
                return self.accept_forced(
                    state,
                    forced,
                    &window,
                    SiteFit {
                        edges,
                        stage,
                        overflow: false,
                    },
                );
            }
            if self.select_cut(state, &window, available, stage)? {
                return Ok(());
            }
            // Ordered zero-size boxes make source progress without an empty slice.
            if window.head.block_size == S::ZERO
                && !self
                    .pending_projection_before(state, FragmentationCutOf::Source(window.head.end))
                && force.is_none_or(|force| force.site.cut.source_position() >= window.head.end)
                && let Some(edges) = policy::edges(&window, S::ZERO, available, stage)
            {
                return self.accept(
                    state,
                    Accepted {
                        coverage_end: window.head.end,
                        consumed_end: window.head.end,
                        body: S::ZERO,
                        edges,
                        stage,
                        kind: FragmentationPieceKindOf::Content,
                    },
                );
            }
            if stage == FragmentationStage::Strict && state.used > S::ZERO {
                if self.defer(state, &window, stage)? {
                    return Ok(());
                }
                continue;
            }
            if stage == FragmentationStage::Emergency {
                return self.emergency(state, window, available);
            }
        }
        unreachable!("the final scheduling stage accepts progress or returns a typed failure")
    }

    fn select_cut(
        &mut self,
        state: &mut State<S>,
        window: &FragmentationWindowOf<S>,
        available: S,
        stage: FragmentationStage,
    ) -> Result<bool, FragmentationError<P::Error>> {
        let force = self.first_force(state, window);
        let placement_limit = self.placement_limit(state);
        let pending_limit = self.pending_cut_limit(state);
        let mut candidates = Vec::new();
        let capacity = window
            .opportunities
            .len()
            .checked_add(self.scopes.len())
            .ok_or(FragmentationError::Allocation)?;
        candidates
            .try_reserve(capacity)
            .map_err(|_| FragmentationError::Allocation)?;
        for (index, item) in window.opportunities.iter().enumerate() {
            let position = item.cut.source_position();
            if (position > state.position
                || matches!(item.cut, FragmentationCutOf::Graphical { .. }))
                && item.block_offset > S::ZERO
                && position < self.end(state.flow)
                && self.usable(state, item)
                && placement_limit.is_none_or(|limit| {
                    cut_order(item.cut, FragmentationCutOf::Source(limit))
                        != core::cmp::Ordering::Greater
                })
                && pending_limit
                    .is_none_or(|limit| cut_order(item.cut, limit) != core::cmp::Ordering::Greater)
            {
                candidates.push(SelectionSite {
                    cut: item.cut,
                    body: item.block_offset,
                    kind: SiteKind::Offered {
                        opportunity: Some(index),
                        receipt: self.projected_transition(state, item.cut),
                    },
                });
            }
        }
        for (scope_index, scope) in self.scopes.iter().enumerate() {
            if scope.flow != state.flow
                || scope.retired
                || (!scope.active && scope.initial_frame != state.frame)
            {
                continue;
            }
            // Ordered receipts cannot skip a prior unapplied transition.
            let Some((index, projection)) = scope
                .projections
                .iter()
                .enumerate()
                .find(|(index, _)| !scope.applied[*index])
            else {
                continue;
            };
            let selected = &scope.child.report.breaks[projection.child_break_index];
            if projection.endpoint.retained()
                && selected.from_fragmentainer == state.frame
                && !matches!(selected.reason, FragmentationBreakReason::Forced { .. })
                && projection.block_offset >= state.used
                && cut_order(projection.endpoint.checkpoint(), current_cut(state))
                    != core::cmp::Ordering::Less
            {
                candidates.push(self.projection_site(state, scope_index, index));
            }
        }
        candidates.retain(|site| {
            force
                .is_none_or(|force| self.site_order(*site, force.site) == core::cmp::Ordering::Less)
                && site.receipt().is_none_or(|(scope, index)| {
                    stage_rank(stage)
                        >= stage_rank(
                            self.scopes[scope].child.report.breaks
                                [self.scopes[scope].projections[index].child_break_index]
                                .stage,
                        )
                })
                && policy::edges(window, site.body, available, stage).is_some()
        });
        if candidates.is_empty() {
            return Ok(false);
        }
        candidates.sort_unstable_by(|a, b| {
            b.body
                .partial_cmp(&a.body)
                .expect("validated finite sites")
                .then_with(|| self.site_order(*b, *a))
                .then_with(|| match (a.kind, b.kind) {
                    (
                        SiteKind::Offered { opportunity: a, .. },
                        SiteKind::Offered { opportunity: b, .. },
                    ) => b.cmp(&a),
                    _ => core::cmp::Ordering::Equal,
                })
        });
        let next = state
            .frame
            .checked_add(1)
            .ok_or(FragmentationError::Arithmetic { flow: state.flow })?;
        let crossed = self.transition(self.context(state), state.frame, next)?;
        for site in candidates {
            let (origin, reason) = if let Some(receipt) = site.receipt() {
                let Some(origin) = self.projection_policy(state, window, site, stage)? else {
                    continue;
                };
                let scope = &self.scopes[receipt.0];
                (
                    origin,
                    scope.child.report.breaks[scope.projections[receipt.1].child_break_index]
                        .reason,
                )
            } else {
                let SiteKind::Offered {
                    opportunity: Some(index),
                    ..
                } = site.kind
                else {
                    unreachable!("ordinary soft candidates carry their exact offered site");
                };
                let item = &window.opportunities[index];
                let available = self.available_kinds(self.context(state));
                if !policy::legal(item, crossed, available, stage) {
                    continue;
                }
                (
                    policy::SelectedPolicyOf::capture(item, available)
                        .map_err(|_| FragmentationError::Allocation)?,
                    FragmentationBreakReason::Unforced,
                )
            };
            let edges = policy::edges(window, site.body, available, stage)
                .expect("selected site fit was evaluated");
            self.accept_site(
                state,
                site,
                window,
                SiteFit {
                    edges,
                    stage,
                    overflow: false,
                },
            )?;
            self.advance_selected(state, reason, stage, site.receipt(), None)?;
            *self
                .selected_origins
                .last_mut()
                .expect("selection recorded its transition") = Some(origin);
            return Ok(true);
        }
        Ok(false)
    }

    fn emergency(
        &mut self,
        state: &mut State<S>,
        window: FragmentationWindowOf<S>,
        available: S,
    ) -> Result<(), FragmentationError<P::Error>> {
        let pending = self.pending_head_force(state, &window);
        if let Some(force) = pending
            && window.fallback == FragmentationFallback::Unsliceable
        {
            return self.accept_forced(
                state,
                force,
                &window,
                SiteFit {
                    edges: (S::ZERO, S::ZERO),
                    stage: FragmentationStage::Emergency,
                    overflow: true,
                },
            );
        }
        if let Some(head) = self.certified_head(state, &window) {
            let force = self.first_force(state, &window);
            return self.accept_certified_head(state, head, available, force, &window);
        }
        if pending.is_some() && window.head.block_size <= available {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        if window.head.block_size <= available
            && self
                .projected_transition(state, FragmentationCutOf::Source(window.head.end))
                .is_none()
        {
            return self.accept(
                state,
                Accepted {
                    coverage_end: window.head.end,
                    consumed_end: window.head.end,
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
                    coverage_end: window.head.end,
                    consumed_end: window.head.end,
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
                if end > window.head.end
                    || pending.is_some_and(|force| {
                        end > force.site.cut.source_position()
                            || (end == force.site.cut.source_position()
                                && block_size != force.site.body)
                    })
                    || (end == window.head.end && block_size != window.head.block_size)
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
                let receipt = self.projected_transition(state, FragmentationCutOf::Source(end));
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
                if end.offset != 0 || receipt.is_some() {
                    self.advance_selected(
                        state,
                        FragmentationBreakReason::Unforced,
                        FragmentationStage::Emergency,
                        receipt,
                        None,
                    )?;
                }
                Ok(())
            }
            FragmentationEmergencySliceOf::Graphical {
                end,
                extent,
                block_size,
            } => self.graphical_slice(state, end, extent, block_size, available, &window),
        }
    }
    fn finish_graphical_head(
        &mut self,
        state: &mut State<S>,
        window: &FragmentationWindowOf<S>,
        available: S,
        stage: FragmentationStage,
        overflow: bool,
    ) -> Result<bool, FragmentationError<P::Error>> {
        let progress = state.graphical.ok_or(FragmentationError::InvalidInput(
            FragmentationInputError::InvalidSlice,
        ))?;
        let force = self.first_force(state, window).filter(|force| {
            force.site.cut.source_position() < window.head.end
                || (force.site.receipt().is_some()
                    && force.site.cut == FragmentationCutOf::Source(window.head.end))
        });
        if let Some(force) = force
            && force.site.receipt().is_some()
        {
            if stage_rank(stage) < stage_rank(force.floor) {
                return Ok(false);
            }
            let edges = match policy::edges(window, force.site.body, available, stage) {
                Some(edges) => edges,
                None if overflow => (S::ZERO, S::ZERO),
                None => return Ok(false),
            };
            self.accept_forced(
                state,
                force,
                window,
                SiteFit {
                    edges,
                    stage,
                    overflow,
                },
            )?;
            return Ok(true);
        }
        if force.is_some_and(|force| force.site.cut.source_position() < Self::anchor_end(state)) {
            // Graphical coverage cannot discharge a semantic cut inside its
            // anchor. A source force after the anchor is an ordinary boundary.
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        if force.is_none()
            && (self.pending_projection_before(state, FragmentationCutOf::Source(window.head.end))
                || self
                    .projected_transition(state, FragmentationCutOf::Source(window.head.end))
                    .is_some())
        {
            return Ok(false);
        }
        let (end, body) = force
            .as_ref()
            .map_or((window.head.end, window.head.block_size), |force| {
                (force.site.cut.source_position(), force.site.body)
            });
        let edges = match policy::edges(window, body, available, stage) {
            Some(edges) => edges,
            None if overflow => (S::ZERO, S::ZERO),
            None => return Ok(false),
        };
        self.accept(
            state,
            Accepted {
                coverage_end: end,
                consumed_end: end,
                body,
                edges,
                stage,
                kind: FragmentationPieceKindOf::GraphicalSlice {
                    anchor: state.position,
                    coverage: FragmentationGraphicalCoverageOf {
                        start: progress.consumed,
                        end: progress.extent,
                        extent: progress.extent,
                    },
                },
            },
        )?;
        state.graphical = None;
        if let Some(forced) = force {
            let position = forced.site.cut.source_position();
            state.forced_applied = Some(position);
            self.advance_selected(
                state,
                forced.reason(),
                stage,
                forced.site.receipt(),
                Some(forced.force),
            )?;
        }
        Ok(true)
    }
    fn graphical(
        &mut self,
        state: &mut State<S>,
        window: FragmentationWindowOf<S>,
        available: S,
        stage: FragmentationStage,
    ) -> Result<(), FragmentationError<P::Error>> {
        if self.finish_graphical_head(state, &window, available, stage, false)? {
            return Ok(());
        }
        if self.select_cut(state, &window, available, stage)? {
            return Ok(());
        }
        if state.used > S::ZERO {
            return if self.defer(state, &window, stage)? {
                Ok(())
            } else {
                Err(FragmentationError::InvalidInput(
                    FragmentationInputError::InvalidBoundary,
                ))
            };
        }
        for next_stage in [
            FragmentationStage::RelaxLineCounts,
            FragmentationStage::RelaxAvoidance,
            FragmentationStage::TruncateBlockEnd,
            FragmentationStage::TruncateBlockStart,
            FragmentationStage::Emergency,
        ] {
            let (next, available) = match self.query(state, next_stage)? {
                Queried::Measured(window, available) => (window, available),
                Queried::ControlAtStart(_) | Queried::PlacementDeferred => {
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
            };
            if self.finish_graphical_head(state, &next, available, next_stage, false)? {
                return Ok(());
            }
            if self.select_cut(state, &next, available, next_stage)? {
                return Ok(());
            }
            if next_stage == FragmentationStage::Emergency {
                if let Some(head) = self.certified_head(state, &next) {
                    let force = self.first_force(state, &next);
                    return self.accept_certified_head(state, head, available, force, &next);
                }
                if next.fallback == FragmentationFallback::Unsliceable {
                    if self.finish_graphical_head(state, &next, available, next_stage, true)? {
                        return Ok(());
                    }
                    return Err(FragmentationError::InvalidInput(
                        FragmentationInputError::InvalidSlice,
                    ));
                }
                let Some(FragmentationEmergencySliceOf::Graphical {
                    end,
                    extent,
                    block_size,
                }) = next.emergency_slice
                else {
                    return Err(FragmentationError::MissingSlice { flow: state.flow });
                };
                return self.graphical_slice(state, end, extent, block_size, available, &next);
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
        window: &FragmentationWindowOf<S>,
    ) -> Result<(), FragmentationError<P::Error>> {
        // Incomplete graphical progress can precede a source force beyond the
        // anchor. Completion must use the ordinary forced-boundary path, and
        // graphical coverage cannot discharge a force inside its own anchor.
        if self.pending_head_force(state, window).is_some_and(|force| {
            force.site.cut.source_position() < Self::anchor_end(state) || end >= extent
        }) {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        let head = window.head;
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
            || (end == extent && body != head.block_size)
            || !validation::nonnegative(body)
            || body <= S::ZERO
            || body > available
            || state.graphical.is_some_and(|old| old.extent != extent)
        {
            return Err(FragmentationError::InvalidInput(
                FragmentationInputError::InvalidSlice,
            ));
        }
        let receipt = self.projected_transition(
            state,
            FragmentationCutOf::Graphical {
                anchor: state.position,
                progress: FragmentationGraphicalProgressOf {
                    consumed: end,
                    extent,
                },
            },
        );
        self.accept(
            state,
            Accepted {
                coverage_end: if end == extent {
                    head.end
                } else {
                    Self::anchor_end(state)
                },
                consumed_end: if end == extent {
                    head.end
                } else {
                    state.position
                },
                body,
                edges: (S::ZERO, S::ZERO),
                stage: FragmentationStage::Emergency,
                kind: FragmentationPieceKindOf::GraphicalSlice {
                    anchor: state.position,
                    coverage: FragmentationGraphicalCoverageOf { start, end, extent },
                },
            },
        )?;
        if end < extent {
            state.graphical = Some(FragmentationGraphicalProgressOf {
                consumed: end,
                extent,
            });
            self.advance_selected(
                state,
                FragmentationBreakReason::Unforced,
                FragmentationStage::Emergency,
                receipt,
                None,
            )?;
        } else {
            state.graphical = None;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "feedback_tests.rs"]
mod feedback_tests;

#[cfg(test)]
#[path = "scope_transition_tests.rs"]
mod scope_transition_tests;
