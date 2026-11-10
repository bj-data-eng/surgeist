//! Provisional source-order float encounters for one captured Text paragraph.

use std::collections::BTreeSet;
use std::sync::Arc;

use super::floats::{
    AcceptedFloatPlacement, AcceptedFloatPrefix, FloatExclusions, FloatInlineSpace, PendingFloat,
};
use crate::LayoutScalar;
use crate::text_layout::fit::{EncounterBasis, EncounterReplay, FitError};
use crate::text_layout::scope_alignment::EntryBacking;
use crate::text_layout::scope_alignment::EntryFundingRef;
use surgeist_text::{FinalizedLine, LineEnd, PreparedParticipant, SourcePosition, TextSeam};

mod ancestor_end;
use ancestor_end::ClosingEnds;
pub(super) use ancestor_end::source_interval;

pub(super) struct FloatEvents<Node, S: LayoutScalar> {
    rows: Vec<FloatEvent<Node, S>>,
    backing: EntryBacking,
}
impl<Node: Copy + Eq, S: LayoutScalar> FloatEvents<Node, S> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            backing: EntryBacking::default(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    pub fn push(
        &mut self,
        event: FloatEvent<Node, S>,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<(), FitError> {
        self.backing.reserve_push_ref(&mut self.rows, funding)?;
        self.rows.push(event);
        Ok(())
    }
    pub fn bind<'a>(
        self,
        run: &crate::text_layout::AdmittedRun<'_, Node>,
        associations: &crate::text_layout::capture::ScopeAssociations<Node>,
        controls: impl Iterator<Item = &'a surgeist_text::ControlAllocation> + Clone,
        mut interval: impl FnMut(
            &surgeist_text::InlineBoxSource,
        ) -> Result<Option<(usize, usize)>, FitError>,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<BoundEvents<Node, S>, FitError> {
        let mut backing = EntryBacking::default();
        let mut rows = Vec::new();
        for source in self.rows {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            let closing_ends = ClosingEnds::project(
                &source,
                run,
                associations,
                controls.clone(),
                &mut interval,
                funding,
            )?;
            backing.reserve_push_ref(&mut rows, funding)?;
            rows.push(FloatEncounter {
                source,
                closing_ends,
            });
        }
        Ok(BoundEvents {
            rows,
            _backing: backing,
        })
    }
}

pub(super) struct FloatEncounter<Node, S: LayoutScalar> {
    source: FloatEvent<Node, S>,
    closing_ends: ClosingEnds,
}
pub(super) struct BoundEvents<Node, S: LayoutScalar> {
    rows: Vec<FloatEncounter<Node, S>>,
    _backing: EntryBacking,
}
impl<Node, S: LayoutScalar> std::ops::Deref for BoundEvents<Node, S> {
    type Target = [FloatEncounter<Node, S>];
    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

struct OriginIncidence {
    represented: bool,
    inside: bool,
}

fn visit_source(funding: Option<EntryFundingRef<'_>>) -> Result<(), FitError> {
    if let Some(funding) = funding {
        funding.visit(1)?;
    }
    Ok(())
}

fn source_computed_cut<Node: Copy + Eq>(
    associations: &crate::text_layout::capture::ScopeAssociations<Node>,
    boundary: &surgeist_text::PreparedComputedLineBoundary,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<Option<crate::text_layout::capture::ComputedNativeCut>, FitError> {
    for (actual, cut) in associations.computed_cuts() {
        visit_source(funding)?;
        if actual == boundary {
            return Ok(Some(cut));
        }
    }
    Ok(None)
}

/// Incidence belongs to one preparation and one actual original occurrence.
/// The admitted origin ranges have equal local and aggregate lengths.
fn text_origin_incidence(
    unit: &surgeist_text::PreparedUnit,
    origin: &surgeist_text::PreparedOrigin,
    local: usize,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<Option<OriginIncidence>, FitError> {
    visit_source(funding)?;
    if unit.prepared_text() != origin.prepared_text() {
        return Ok(None);
    }
    let unit = unit.view();
    let original = origin.view();
    if let Some(empty) = unit.empty_origin() {
        return Ok((empty == *origin).then_some(OriginIncidence {
            represented: original.local_range().contains(local)
                || local == original.local_range().end,
            inside: false,
        }));
    }
    let aggregate = original.aggregate_range();
    let range = unit.source_range();
    let start = range.start.max(aggregate.start);
    let end = range.end.min(aggregate.end);
    if start >= end {
        return Ok(None);
    }
    let local_start = original.local_range().start + (start - aggregate.start);
    let local_end = original.local_range().start + (end - aggregate.start);
    let declared = original.local_range();
    let in_origin = declared.start <= local && local <= declared.end;
    let inside = if in_origin {
        let source = aggregate.start + (local - declared.start);
        range.start < source && source < range.end
    } else {
        false
    };
    Ok(Some(OriginIncidence {
        represented: in_origin && local_start <= local && local <= local_end,
        inside,
    }))
}

/// A Cell can span several originals without offering an interior break.
/// Its captured pieces preserve the original endpoint used by Text's prefix.
fn cell_origin_incidence(
    cell: &surgeist_text::PreparedCell,
    origin: &surgeist_text::PreparedOrigin,
    local: usize,
    funding: Option<EntryFundingRef<'_>>,
) -> Result<Option<OriginIncidence>, FitError> {
    let view = origin.view();
    let Some(captured) = view.capture_position() else {
        return Ok(None);
    };
    let mut member = false;
    let mut represented = false;
    let mut before = false;
    let mut after = false;
    for piece in cell.view().source_cell().source_pieces() {
        if let Some(funding) = funding {
            funding.visit(1)?;
        }
        match piece.origin_position().partial_cmp(&captured) {
            Some(std::cmp::Ordering::Less) => before = true,
            Some(std::cmp::Ordering::Greater) => after = true,
            Some(std::cmp::Ordering::Equal) => {
                if let Some(funding) = funding {
                    funding.visit(6)?;
                }
                let range = piece.local();
                let declared = view.local_range();
                member |= (range.start <= declared.end) & (declared.start <= range.end);
                represented |= (range.start <= local) & (local <= range.end);
                before |= range.start < local;
                after |= local < range.end;
            }
            None => {}
        }
    }
    Ok(member.then_some(OriginIncidence {
        represented,
        inside: before && after,
    }))
}

pub(super) struct FloatEvent<Node, S: LayoutScalar> {
    pub pending: PendingFloat<Node, S>,
    pub packet: Arc<crate::engine::FloatChildPacket<Node, S>>,
    seam: TextSeam,
    following: Option<TextSeam>,
    position: usize,
    inside: bool,
    native_offset: usize,
}

impl<Node: Copy + Eq, S: LayoutScalar> FloatEvent<Node, S> {
    pub fn new(
        pending: PendingFloat<Node, S>,
        packet: Arc<crate::engine::FloatChildPacket<Node, S>>,
        run: &crate::text_layout::AdmittedRun<'_, Node>,
        associations: &crate::text_layout::capture::ScopeAssociations<Node>,
        offset: usize,
        request: crate::ComputeInputOf<S>,
        funding: Option<EntryFundingRef<'_>>,
    ) -> Result<Self, FitError> {
        let binding = run.binding;
        if packet.node != pending.node || packet.input != request {
            return Err(FitError::InvalidGeometry);
        }
        let origin_seam = |origin: &surgeist_text::PreparedOrigin,
                           after: bool|
         -> Result<Option<(TextSeam, usize, bool)>, FitError> {
            let view = origin.view();
            let local = if after {
                view.local_range().end
            } else {
                view.local_range().start
            };
            let mut endpoint = None;
            for (index, participant) in binding.prepared.participants().enumerate() {
                visit_source(funding)?;
                let inside = match participant {
                    PreparedParticipant::Text(unit) => {
                        let Some(incidence) = text_origin_incidence(&unit, origin, local, funding)?
                        else {
                            continue;
                        };
                        incidence.inside
                    }
                    PreparedParticipant::Cell(cell) => {
                        let Some(incidence) = cell_origin_incidence(&cell, origin, local, funding)?
                        else {
                            continue;
                        };
                        incidence.inside
                    }
                    _ => continue,
                };
                endpoint = Some((index, inside));
                if !after {
                    break;
                }
            }
            let Some((index, inside)) = endpoint else {
                return Ok(None);
            };
            Ok(Some((
                TextSeam::Origin {
                    origin: origin.clone(),
                    local: SourcePosition::try_new(view.source().text(), local)
                        .map_err(|_| FitError::InvalidGeometry)?,
                },
                index + usize::from(after && !inside),
                inside,
            )))
        };
        let at_node =
            |node: Node, after: bool| -> Result<Option<(TextSeam, usize, bool)>, FitError> {
                for row in &binding.origins {
                    visit_source(funding)?;
                    if row.node == node {
                        return origin_seam(&row.origin, after);
                    }
                }
                let mut participant = None;
                for row in &binding.controls {
                    visit_source(funding)?;
                    if row.node == node {
                        participant = row.occurrence.view().participant();
                        break;
                    }
                }
                if participant.is_none() {
                    for row in &binding.atomics {
                        visit_source(funding)?;
                        if row.node == node {
                            participant = Some(PreparedParticipant::Atomic(row.occurrence.clone()));
                            break;
                        }
                    }
                }
                let Some(participant) = participant else {
                    return Ok(None);
                };
                let mut rank = None;
                for (index, actual) in binding.prepared.participants().enumerate() {
                    visit_source(funding)?;
                    if actual == participant {
                        rank = Some(index);
                        break;
                    }
                }
                let Some(index) = rank else {
                    return Ok(None);
                };
                Ok(Some((
                    if after {
                        TextSeam::AfterParticipant(participant)
                    } else {
                        TextSeam::BeforeParticipant(participant)
                    },
                    index + usize::from(after),
                    false,
                )))
            };
        let mut previous = None;
        for &node in run.nodes()[..offset].iter().rev() {
            visit_source(funding)?;
            previous = at_node(node, true)?;
            if previous.is_some() {
                break;
            }
        }
        let mut following = None;
        for &node in &run.nodes()[offset + 1..] {
            visit_source(funding)?;
            following = at_node(node, false)?;
            if following.is_some() {
                break;
            }
        }
        let lower = previous
            .as_ref()
            .map_or(run.segment.start().view().participant_offset(), |row| row.1);
        let upper = following
            .as_ref()
            .map_or(run.segment.end().view().participant_offset(), |row| row.1);
        let mut actual = previous
            .or_else(|| following.clone())
            .ok_or(FitError::InvalidGeometry)?;
        let mut next_seam = following.map(|value| value.0);
        let mut preceding_computed = None;
        let mut following_computed = None;
        for (index, participant) in binding.prepared.participants().enumerate().take(upper) {
            visit_source(funding)?;
            if index < lower {
                continue;
            }
            let PreparedParticipant::ComputedLineBoundary(ref boundary) = participant else {
                continue;
            };
            let cut = source_computed_cut(associations, boundary, funding)?
                .and_then(|cut| cut.relative_to(run.inventory.clone()))
                .ok_or(FitError::MissingComputedBoundaryIncidence)?;
            match cut.side_of_node(offset) {
                Some(std::cmp::Ordering::Less) => {
                    following_computed.get_or_insert((index, participant.clone()));
                }
                Some(std::cmp::Ordering::Greater) => {
                    preceding_computed = Some((index, participant.clone()))
                }
                Some(std::cmp::Ordering::Equal) | None => {
                    return Err(FitError::MissingComputedBoundaryIncidence);
                }
            }
        }
        if let Some((index, participant)) = preceding_computed {
            actual = (TextSeam::AfterParticipant(participant), index + 1, false);
        } else if let Some((index, participant)) = &following_computed {
            actual = (
                TextSeam::BeforeParticipant(participant.clone()),
                *index,
                false,
            );
        }
        if let Some((_, participant)) = following_computed {
            next_seam = Some(TextSeam::BeforeParticipant(participant));
        }
        Ok(Self {
            pending,
            packet,
            seam: actual.0,
            following: next_seam,
            position: actual.1,
            inside: actual.2,
            native_offset: offset,
        })
    }
}

pub(super) struct AcceptedFloatChild<Node, S: LayoutScalar> {
    pub pending: PendingFloat<Node, S>,
    pub packet: Arc<crate::engine::FloatChildPacket<Node, S>>,
}

pub(super) struct EncounterOutput<Node, S: LayoutScalar> {
    pub ledger: FloatExclusions<S, Node>,
    pub children: Vec<AcceptedFloatChild<Node, S>>,
}

/// An actual accepted prefix is shared by every local group trial. Only rows
/// written after the checkpoint are undone; earlier packet receipts stay put.
struct GroupCheckpoint<Node, S: LayoutScalar> {
    ledger: FloatExclusions<S, Node>,
    accepted_len: usize,
    earlier_float_top: Option<S>,
}

/// Quality selection work index and undo log. Ranks filter work, never decide seam
/// ownership: all equal-boundary and deferred earlier events remain candidates.
struct EncounterJournal<Node, S: LayoutScalar> {
    remaining: BTreeSet<(usize, usize)>,
    tentative: Vec<usize>,
    accepted: Vec<usize>,
    incoming_float_top: Option<S>,
    earlier_float_top: Option<S>,
    group: Option<GroupCheckpoint<Node, S>>,
}

pub(super) struct Encounters<Node, S: LayoutScalar> {
    events: BoundEvents<Node, S>,
    incoming: Option<FloatExclusions<S, Node>>,
    base: FloatExclusions<S, Node>,
    current: FloatExclusions<S, Node>,
    accepted: Vec<Option<PendingFloat<Node, S>>>,
    tentative: Vec<Option<PendingFloat<Node, S>>>,
    cursor: S,
    inline_space: FloatInlineSpace,
    journal: Option<EncounterJournal<Node, S>>,
    #[cfg(test)]
    replay_event_visits: usize,
}

impl<Node: Copy + Eq, S: LayoutScalar> Encounters<Node, S> {
    pub fn new(
        events: BoundEvents<Node, S>,
        base: FloatExclusions<S, Node>,
        cursor: S,
        inline_space: FloatInlineSpace,
        restart_sequence: bool,
        group_replay: bool,
    ) -> Result<Self, FitError> {
        let count = events.len();
        let journal = group_replay.then(|| {
            let earlier_float_top = base.preceding_float_top();
            EncounterJournal {
                remaining: events
                    .iter()
                    .enumerate()
                    .map(|(index, event)| (event.source.position, index))
                    .collect(),
                tentative: Vec::new(),
                accepted: Vec::new(),
                incoming_float_top: earlier_float_top,
                earlier_float_top,
                group: None,
            }
        });
        Ok(Self {
            events,
            incoming: restart_sequence.then(|| base.try_clone()).transpose()?,
            current: base.try_clone()?,
            base,
            accepted: vec![None; count],
            tentative: vec![None; count],
            cursor,
            inline_space,
            journal,
            #[cfg(test)]
            replay_event_visits: 0,
        })
    }
    /// Roll back the entire unpublished branch to this run's incoming ledger.
    /// Finite complete-tail search and group-quality selection retain the snapshot.
    pub fn restart_sequence(&mut self) -> Result<(), FitError> {
        let Some(incoming) = &self.incoming else {
            return self.reset_candidate();
        };
        self.base = incoming.try_clone()?;
        self.current = incoming.try_clone()?;
        if let Some(journal) = &mut self.journal {
            for index in journal.tentative.drain(..) {
                self.tentative[index] = None;
            }
            for index in journal.accepted.drain(..) {
                self.accepted[index] = None;
                journal
                    .remaining
                    .insert((self.events[index].source.position, index));
            }
            journal.earlier_float_top = journal.incoming_float_top;
            journal.group = None;
        } else {
            self.accepted.fill(None);
            self.tentative.fill(None);
        }
        Ok(())
    }
    /// Mark the actual resolved group entry after the preceding winning replay.
    pub fn checkpoint_group(&mut self) -> Result<(), FitError> {
        self.reset_candidate()?;
        if let Some(journal) = &mut self.journal {
            journal.group = Some(GroupCheckpoint {
                ledger: self.base.try_clone()?,
                accepted_len: journal.accepted.len(),
                earlier_float_top: journal.earlier_float_top,
            });
        }
        Ok(())
    }
    /// Restore only this group's accepted suffix and provisional candidate rows.
    pub fn restart_group(&mut self) -> Result<(), FitError> {
        self.reset_candidate()?;
        if let Some(journal) = &mut self.journal {
            let group = journal
                .group
                .as_ref()
                .expect("group reset follows an actual group-entry checkpoint");
            self.base = group.ledger.try_clone()?;
            self.current = group.ledger.try_clone()?;
            for index in journal.accepted.drain(group.accepted_len..) {
                self.accepted[index] = None;
                journal
                    .remaining
                    .insert((self.events[index].source.position, index));
            }
            journal.earlier_float_top = group.earlier_float_top;
        }
        Ok(())
    }
    /// Discard provisional state while retaining the accepted line prefix.
    pub fn reset_candidate(&mut self) -> Result<(), FitError> {
        self.current = self.base.try_clone()?;
        if let Some(journal) = &mut self.journal {
            for index in journal.tentative.drain(..) {
                self.tentative[index] = None;
            }
        } else {
            self.tentative = self.accepted.clone();
        }
        Ok(())
    }
    pub fn band(&self) -> &FloatExclusions<S, Node> {
        &self.current
    }
    pub fn replay<M>(
        &mut self,
        container: Node,
        line: &FinalizedLine,
        replay: EncounterReplay,
        funding: Option<EntryFundingRef<'_>>,
    ) -> crate::LayoutResultOf<Node, Vec<EncounterBasis>, S, M> {
        self.reset_candidate()
            .map_err(|error| super::inline_run::text_run_error(container, error))?;
        let portion_start = line
            .selected_line()
            .request()
            .start
            .view()
            .participant_offset();
        let mut basis = Vec::new();
        let mut earlier_float_top = self.journal.as_ref().map_or_else(
            || self.base.preceding_float_top(),
            |journal| journal.earlier_float_top,
        );
        // The real end rank is only an upper work bound. Keep every unaccepted
        // earlier/equal-rank event, including a deferred predecessor,
        // then visit them in their original native event order.
        let window = self.journal.as_ref().map(|journal| {
            let end = line
                .selected_line()
                .request()
                .end
                .view()
                .participant_offset();
            let mut window: Vec<_> = journal
                .remaining
                .range(..=(end, usize::MAX))
                .map(|&(_, index)| index)
                .collect();
            window.sort_unstable();
            window
        });
        for ordinal in 0..window.as_ref().map_or(self.events.len(), Vec::len) {
            let index = window.as_ref().map_or(ordinal, |window| window[ordinal]);
            let encounter = &self.events[index];
            let event = &encounter.source;
            #[cfg(test)]
            {
                self.replay_event_visits += 1;
            }
            if let Some(previous) = &self.accepted[index] {
                let top = previous
                    .placement
                    .text()
                    .expect("accepted encounter retains its placement")
                    .margin_block_start;
                earlier_float_top = Some(earlier_float_top.map_or(top, |earlier| earlier.max(top)));
                continue;
            }
            let represented = |request: &surgeist_text::LineSelectionRequest,
                               seam: &TextSeam|
             -> Result<bool, FitError> {
                Ok(match seam {
                    TextSeam::BeforeParticipant(participant)
                    | TextSeam::AfterParticipant(participant) => {
                        let mut found = false;
                        for actual in &request.participants {
                            visit_source(funding)?;
                            if actual == participant {
                                found = true;
                                break;
                            }
                        }
                        found
                    }
                    TextSeam::Origin { origin, local } => {
                        let mut found = false;
                        for participant in &request.participants {
                            visit_source(funding)?;
                            found = match participant {
                                PreparedParticipant::Text(unit) => {
                                    text_origin_incidence(unit, origin, local.get(), funding)?
                                        .is_some_and(|incidence| incidence.represented)
                                }
                                PreparedParticipant::Cell(cell) => {
                                    cell_origin_incidence(cell, origin, local.get(), funding)?
                                        .is_some_and(|incidence| incidence.represented)
                                }
                                _ => false,
                            };
                            if found {
                                break;
                            }
                        }
                        found
                    }
                })
            };
            // At an equal participant boundary the real originating seam,
            // rather than the numeric tie, determines prefix ownership.
            let original_request = line.selected_line();
            let use_preceding = replay.preceding.is_some()
                && (event.position < portion_start
                    || event.position == portion_start
                        && !event.inside
                        && !represented(original_request.request(), &event.seam).map_err(
                            |error| super::inline_run::text_run_error(container, error),
                        )?);
            let prefix = replay.preceding.as_ref().filter(|_| use_preceding);
            let boxes = prefix
                .map_or(replay.boxes.as_ref(), |frame| frame.boxes.as_ref())
                .cloned();
            let (line, placed, frame) = prefix
                .map_or((line, replay.placed, &replay.frame), |earlier| {
                    (&earlier.line, earlier.placed, &earlier.frame)
                });
            let selected = line.selected_line();
            let request = selected.request();
            let start = request.start.view().participant_offset();
            let end = if prefix.is_some() {
                portion_start
            } else {
                request.end.view().participant_offset()
            };
            let boundary_side = if event.position == end && !event.inside {
                match boxes
                    .as_ref()
                    .map(|assignment| assignment.outgoing.native())
                {
                    Some(crate::text_layout::boxes::NativeBoxCut::Proven(cut)) => Some(
                        cut.side_of_node(event.native_offset).ok_or_else(|| {
                            super::inline_run::text_run_error(
                                container,
                                FitError::Wrapping(crate::WrappingScopeError::UnsupportedIncidence),
                            )
                        })? == std::cmp::Ordering::Less,
                    ),
                    Some(crate::text_layout::boxes::NativeBoxCut::Unavailable) => {
                        return Err(super::inline_run::text_run_error(
                            container,
                            FitError::Wrapping(crate::WrappingScopeError::UnsupportedIncidence),
                        ));
                    }
                    Some(crate::text_layout::boxes::NativeBoxCut::Prefix) | None => None,
                }
            } else {
                None
            };
            let eligible = if event.inside {
                start <= event.position && event.position < end
            } else {
                start <= event.position
                    && (event.position < end
                        || event.position == end
                            && boundary_side.unwrap_or(request.context.end != LineEnd::Forced))
            };
            if !eligible {
                continue;
            }
            let observation = if represented(request, &event.seam)
                .map_err(|error| super::inline_run::text_run_error(container, error))?
            {
                AcceptedFloatPrefix::Seam(event.seam.clone())
            } else if let Some(seam) = match &event.following {
                Some(seam)
                    if represented(request, seam)
                        .map_err(|error| super::inline_run::text_run_error(container, error))? =>
                {
                    Some(seam)
                }
                _ => None,
            } {
                AcceptedFloatPrefix::Seam(seam.clone())
            } else if start == end
                && event.position == end
                && request.participants.is_empty()
                && request.context.end == surgeist_text::LineEnd::Paragraph
            {
                AcceptedFloatPrefix::EmptyParagraphEnd {
                    boundary: request.end.clone(),
                    original: event.seam.clone(),
                }
            } else {
                return Err(super::inline_run::text_run_error(
                    container,
                    FitError::InvalidGeometry,
                ));
            };
            let (prefix, preceding) = observation
                .observe(line)
                .map_err(|error| super::inline_run::text_run_error(container, error))?;
            let demand = encounter
                .closing_ends
                .demand(line, &observation, funding)
                .map_err(|error| super::inline_run::text_run_error(container, error))?;
            let inline_constraint = crate::text_layout::run::add(
                crate::text_layout::run::add(prefix, request.context.indent)
                    .map_err(|error| super::inline_run::text_run_error(container, error))?,
                demand.extra,
            )
            .map_err(|error| super::inline_run::text_run_error(container, error))?;
            if !inline_constraint.is_finite() {
                return Err(super::inline_run::text_run_error(
                    container,
                    FitError::InvalidGeometry,
                ));
            }
            let mut pending = event.pending.clone();
            let used_height = crate::text_layout::fit::LineDisposition::resolve(line)
                .map_err(|error| super::inline_run::text_run_error(container, error))?
                .height(line);
            let encounter_floor = if preceding && replay.defer_preceding {
                frame.logical_start.max(placed) + used_height
            } else {
                frame.float_source_floor
            };
            pending.block_start = self.cursor
                + crate::text_layout::run::number::<S>(encounter_floor)
                    .map_err(|e| super::inline_run::text_run_error(container, e))?;
            // Rule5 uses only actual earlier source-float receipts. The
            // current float's own text-avoidance retry never raises this floor.
            if let Some(earlier) = earlier_float_top {
                pending.block_start = pending.block_start.max(earlier);
            }
            let encounter_floor = (pending.block_start - self.cursor).to_f64();
            let line_end = self.cursor
                + crate::text_layout::run::number::<S>(
                    frame.logical_start.max(placed) + used_height,
                )
                .map_err(|e| super::inline_run::text_run_error(container, e))?;
            let location = match self.inline_space {
                FloatInlineSpace::Finite => self.current.place_float_for_line(
                    container,
                    &pending,
                    (preceding || demand.participates).then_some((inline_constraint, line_end)),
                )?,
                FloatInlineSpace::IntrinsicMaximum => {
                    self.current.place_float_intrinsic(container, &pending)?
                }
            };
            let logical_origin = location.logical;
            earlier_float_top = Some(
                earlier_float_top.map_or(location.margin_block_start, |earlier| {
                    earlier.max(location.margin_block_start)
                }),
            );
            pending.placement =
                super::floats::PendingPlacement::Text(Box::new(AcceptedFloatPlacement {
                    boxes: boxes.clone(),
                    logical_origin,
                    inline_space: self.inline_space,
                    margin_block_start: location.margin_block_start,
                    intrinsic_inline_demand: location.intrinsic_inline_demand,
                    prefix,
                    observation,
                    line: line.clone(),
                }));
            basis.push(EncounterBasis {
                boxes,
                event: index,
                prefix,
                preceding,
                encounter_floor,
                intrinsic_inline_demand: location
                    .intrinsic_inline_demand
                    .map(|demand| demand.to_f64()),
                inline: logical_origin.inline.to_f64(),
                block: logical_origin.block.to_f64(),
            });
            self.tentative[index] = Some(pending);
            if let Some(journal) = &mut self.journal {
                journal.tentative.push(index);
            }
        }
        Ok(basis)
    }
    pub fn accept(&mut self) -> Result<(), FitError> {
        self.base = self.current.try_clone()?;
        if let Some(journal) = &mut self.journal {
            for index in journal.tentative.drain(..) {
                let pending = self.tentative[index]
                    .take()
                    .expect("touched encounter retains its actual provisional receipt");
                let top = pending
                    .placement
                    .text()
                    .expect("accepted encounter retains its placement")
                    .margin_block_start;
                journal.earlier_float_top = Some(
                    journal
                        .earlier_float_top
                        .map_or(top, |earlier| earlier.max(top)),
                );
                self.accepted[index] = Some(pending);
                journal.accepted.push(index);
                journal
                    .remaining
                    .remove(&(self.events[index].source.position, index));
            }
        } else {
            self.accepted = self.tentative.clone();
        }
        Ok(())
    }
    pub fn finish(self) -> EncounterOutput<Node, S> {
        let children = self
            .accepted
            .into_iter()
            .zip(self.events.rows)
            .filter_map(|(plan, event)| {
                plan.map(|pending| AcceptedFloatChild {
                    pending,
                    packet: event.source.packet,
                })
            })
            .collect();
        EncounterOutput {
            ledger: self.base,
            children,
        }
    }
}

#[cfg(test)]
mod balance_tests;

#[cfg(test)]
mod zero_edge_prefix_tests;

#[cfg(test)]
mod prefix_content_tests;

#[cfg(test)]
mod ancestor_end_tests;

#[cfg(test)]
mod ancestor_end_controls;

#[cfg(test)]
mod generated_end_controls;

#[cfg(test)]
mod cell_prefix_tests;

#[cfg(test)]
mod closing_source_controls;

#[cfg(test)]
mod closing_funding_tests;

#[cfg(test)]
mod source_incidence_tests;

#[cfg(test)]
mod shared_sizing_tests;
