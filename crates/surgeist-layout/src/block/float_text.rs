//! Provisional source-order float encounters for one captured Text paragraph.

use std::sync::Arc;

use super::floats::{
    AcceptedFloatPlacement, AcceptedFloatPrefix, FloatExclusions, FloatInlineSpace, PendingFloat,
    ProviderBandContext,
};
use crate::LayoutScalar;
use crate::engine::contracts::Compute;
use crate::text_layout::fit::{EncounterBasis, EncounterReplay, FitError};
use surgeist_text::{FinalizedLine, LineEnd, PreparedParticipant, SourcePosition, TextSeam};

pub(super) struct FloatEvent<Node, S: LayoutScalar> {
    pub pending: PendingFloat<Node, S>,
    pub packet: Arc<crate::engine::FloatChildPacket<Node, S>>,
    seam: TextSeam,
    following: Option<TextSeam>,
    position: usize,
    inside: bool,
}

impl<Node: Copy + Eq, S: LayoutScalar> FloatEvent<Node, S> {
    pub fn new(
        pending: PendingFloat<Node, S>,
        packet: Arc<crate::engine::FloatChildPacket<Node, S>>,
        binding: &crate::TextParagraphBinding<Node>,
        associations: &crate::text_layout::capture::ScopeAssociations<Node>,
        offset: usize,
        request: crate::ComputeInputOf<S>,
    ) -> Result<Self, FitError> {
        if packet.node != pending.node || packet.input != request {
            return Err(FitError::InvalidGeometry);
        }
        let participants: Vec<_> = binding.prepared.participants().collect();
        let origin_seam = |origin: &surgeist_text::PreparedOrigin,
                           after: bool|
         -> Option<(TextSeam, usize, bool)> {
            let view = origin.view();
            let local = if after {
                view.local_range().end
            } else {
                view.local_range().start
            };
            let source = if after {
                view.aggregate_range().end
            } else {
                view.aggregate_range().start
            };
            let positions = participants
                .iter()
                .enumerate()
                .filter_map(|(index, participant)| {
                    let PreparedParticipant::Text(unit) = participant else {
                        return None;
                    };
                    let unit_view = unit.view();
                    (unit_view.empty_origin().as_ref() == Some(origin)
                        || unit_view.origins().any(|row| row.origin() == *origin))
                    .then_some((index, unit_view.source_range()))
                })
                .collect::<Vec<_>>();
            let &(index, range) = if after {
                positions.last()?
            } else {
                positions.first()?
            };
            let inside = range.start < source && source < range.end;
            Some((
                TextSeam::Origin {
                    origin: origin.clone(),
                    local: SourcePosition::try_new(view.source().text(), local).ok()?,
                },
                index + usize::from(after && !inside),
                inside,
            ))
        };
        let at_node = |node: Node, after: bool| -> Option<(TextSeam, usize, bool)> {
            if let Some(row) = binding.origins.iter().find(|row| row.node == node) {
                return origin_seam(&row.origin, after);
            }
            let participant = binding
                .controls
                .iter()
                .find(|row| row.node == node)
                .map(|row| PreparedParticipant::Control(row.occurrence.clone()))
                .or_else(|| {
                    binding
                        .atomics
                        .iter()
                        .find(|row| row.node == node)
                        .map(|row| PreparedParticipant::Atomic(row.occurrence.clone()))
                })?;
            let index = participants.iter().position(|p| p == &participant)?;
            Some((
                if after {
                    TextSeam::AfterParticipant(participant)
                } else {
                    TextSeam::BeforeParticipant(participant)
                },
                index + usize::from(after),
                false,
            ))
        };
        let previous = binding.nodes[..offset]
            .iter()
            .rev()
            .find_map(|&node| at_node(node, true));
        let following = binding.nodes[offset + 1..]
            .iter()
            .find_map(|&node| at_node(node, false));
        let lower = previous.as_ref().map_or(0, |row| row.1);
        let upper = following.as_ref().map_or(participants.len(), |row| row.1);
        let mut actual = previous
            .or_else(|| following.clone())
            .ok_or(FitError::InvalidGeometry)?;
        let mut next_seam = following.map(|value| value.0);
        let mut preceding_computed = None;
        let mut following_computed = None;
        for (index, participant) in participants.iter().enumerate().take(upper).skip(lower) {
            let PreparedParticipant::ComputedLineBoundary(boundary) = participant else {
                continue;
            };
            let cut = associations
                .computed_cut(boundary)
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

pub(super) struct Encounters<Node, S: LayoutScalar> {
    pub events: Vec<FloatEvent<Node, S>>,
    incoming: Option<FloatExclusions<S, Node>>,
    base: FloatExclusions<S, Node>,
    current: FloatExclusions<S, Node>,
    accepted: Vec<Option<PendingFloat<Node, S>>>,
    tentative: Vec<Option<PendingFloat<Node, S>>>,
    cursor: S,
    inline_space: FloatInlineSpace,
}

impl<Node: Copy + Eq, S: LayoutScalar> Encounters<Node, S> {
    pub fn new(
        events: Vec<FloatEvent<Node, S>>,
        base: FloatExclusions<S, Node>,
        cursor: S,
        inline_space: FloatInlineSpace,
        reselect_tail: bool,
    ) -> Self {
        let count = events.len();
        Self {
            events,
            incoming: reselect_tail.then(|| base.clone()),
            current: base.clone(),
            base,
            accepted: vec![None; count],
            tentative: vec![None; count],
            cursor,
            inline_space,
        }
    }
    /// Roll back the entire unpublished branch to this run's incoming ledger.
    /// Only finite complete-tail search retains the additional snapshot.
    pub fn restart_sequence(&mut self) {
        let Some(incoming) = &self.incoming else {
            self.reset_candidate();
            return;
        };
        self.base = incoming.clone();
        self.current = incoming.clone();
        self.accepted.fill(None);
        self.tentative.fill(None);
    }
    /// Discard provisional state while retaining the accepted line prefix.
    pub fn reset_candidate(&mut self) {
        self.current = self.base.clone();
        self.tentative = self.accepted.clone();
    }
    pub fn band(&self) -> &FloatExclusions<S, Node> {
        &self.current
    }
    pub fn replay<'input, Tree, M>(
        &mut self,
        tree: &Tree,
        container: Node,
        line: &FinalizedLine,
        replay: EncounterReplay,
        provider_enabled: bool,
    ) -> crate::LayoutResultOf<Node, Vec<EncounterBasis>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.current = self.base.clone();
        self.tentative = self.accepted.clone();
        let portion_start = line
            .selected_line()
            .request()
            .start
            .view()
            .participant_offset();
        let mut basis = Vec::new();
        let mut earlier_float_top = self.base.preceding_float_top();
        for (index, event) in self.events.iter().enumerate() {
            if let Some(previous) = &self.accepted[index] {
                let top = previous
                    .accepted
                    .as_ref()
                    .expect("accepted encounter retains its placement")
                    .margin_block_start;
                earlier_float_top = Some(earlier_float_top.map_or(top, |earlier| earlier.max(top)));
                continue;
            }
            let represented =
                |request: &surgeist_text::LineSelectionRequest, seam: &TextSeam| match seam {
                    TextSeam::BeforeParticipant(participant)
                    | TextSeam::AfterParticipant(participant) => {
                        request.participants.contains(participant)
                    }
                    TextSeam::Origin { origin, local } => {
                        request
                            .participants
                            .iter()
                            .any(|participant| match participant {
                                PreparedParticipant::Text(unit) => {
                                    unit.view().empty_origin().as_ref() == Some(origin)
                                        || unit.view().origins().any(|row| {
                                            row.origin() == *origin
                                                && row.local_range().start <= local.get()
                                                && local.get() <= row.local_range().end
                                        })
                                }
                                _ => false,
                            })
                    }
                };
            // At an equal participant boundary the real originating seam,
            // rather than the numeric tie, determines prefix ownership.
            let original_request = line.selected_line();
            let prefix = replay.preceding.as_ref().filter(|_| {
                event.position < portion_start
                    || event.position == portion_start
                        && !event.inside
                        && !represented(original_request.request(), &event.seam)
            });
            let (line, placed) =
                prefix.map_or((line, replay.placed), |frame| (&frame.line, frame.placed));
            let selected = line.selected_line();
            let request = selected.request();
            let start = request.start.view().participant_offset();
            let end = if prefix.is_some() {
                portion_start
            } else {
                request.end.view().participant_offset()
            };
            let eligible = if event.inside {
                start <= event.position && event.position < end
            } else {
                start <= event.position
                    && (event.position < end
                        || event.position == end && request.context.end != LineEnd::Forced)
            };
            if !eligible {
                continue;
            }
            let observation = if represented(request, &event.seam) {
                AcceptedFloatPrefix::Seam(event.seam.clone())
            } else if let Some(seam) = event
                .following
                .as_ref()
                .filter(|seam| represented(request, seam))
            {
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
            let inline_constraint = prefix + request.context.indent;
            if !inline_constraint.is_finite() {
                return Err(super::inline_run::text_run_error(
                    container,
                    FitError::InvalidGeometry,
                ));
            }
            let mut pending = event.pending.clone();
            let encounter_floor = if preceding && replay.defer_preceding {
                replay.logical.max(placed) + line.feedback().line_height
            } else {
                replay.source_floor
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
                    replay.logical.max(placed) + line.feedback().line_height,
                )
                .map_err(|e| super::inline_run::text_run_error(container, e))?;
            let provider = ProviderBandContext {
                tree,
                container,
                enabled: provider_enabled,
            };
            let location = match self.inline_space {
                FloatInlineSpace::Finite => self.current.place_float_for_line(
                    provider,
                    &pending,
                    preceding.then_some((inline_constraint, line_end)),
                )?,
                FloatInlineSpace::IntrinsicMaximum => {
                    self.current.place_float_intrinsic(provider, &pending)?
                }
            };
            let logical_origin = location.logical;
            earlier_float_top = Some(
                earlier_float_top.map_or(location.margin_block_start, |earlier| {
                    earlier.max(location.margin_block_start)
                }),
            );
            pending.accepted = Some(AcceptedFloatPlacement {
                logical_origin,
                inline_space: self.inline_space,
                margin_block_start: location.margin_block_start,
                intrinsic_inline_demand: location.intrinsic_inline_demand,
                prefix,
                observation,
                line: line.clone(),
            });
            basis.push(EncounterBasis {
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
        }
        Ok(basis)
    }
    pub fn accept(&mut self) {
        self.base = self.current.clone();
        self.accepted = self.tentative.clone();
    }
    pub fn finish(self) -> EncounterOutput<Node, S> {
        let children = self
            .accepted
            .into_iter()
            .zip(self.events)
            .filter_map(|(plan, event)| {
                plan.map(|pending| AcceptedFloatChild {
                    pending,
                    packet: event.packet,
                })
            })
            .collect();
        EncounterOutput {
            ledger: self.base,
            children,
        }
    }
}
