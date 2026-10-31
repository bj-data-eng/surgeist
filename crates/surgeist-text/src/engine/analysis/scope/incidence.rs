//! Certified original placements of one fixed executed mixed prefix.

use std::{collections::BTreeMap, sync::Arc};

use super::{ScopeIndexCapacity, anchor_exact};
use crate::engine::AnalyzedCore;
use crate::processing::contextual::canonical::CanonicalFlow;
use crate::protocol::paragraph::SemanticAnchorItem;
use crate::transform::relation::{ComponentOwner, RelationEvent};
use crate::{SourceBoxSide, SourceProcessingEvent};

// A cut before an original event sorts before that event's split body. Rank
// removes only nonparticipating connection bookkeeping in the same domain.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct Point(usize, bool);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum Placement {
    Cut(usize),
    Body { first: usize, last: usize },
}
impl Placement {
    fn point(self) -> Point {
        match self {
            Self::Cut(rank) => Point(rank, false),
            Self::Body { first, .. } => Point(first, true),
        }
    }
}

struct Occurrence {
    first: usize,
    last: usize,
}

#[derive(Clone, Copy)]
struct Cut {
    lower: Point,
    upper: Point,
    split: Option<(usize, usize)>,
    pin: Option<usize>,
    valid: bool,
}

pub(crate) struct IncidenceIndex {
    cuts: Vec<Cut>,
    canonical: Arc<CanonicalFlow>,
    domain: usize,
    edges: Vec<usize>,
    represented: bool,
}

#[cfg(test)]
impl IncidenceIndex {
    pub(super) fn payload(&self, census: &mut super::IndexPayloadCensus) {
        census.buffer(&self.cuts);
        census.buffer(&self.edges);
        self.canonical.payload(census);
    }
}

fn allocated<T>(len: usize, value: T) -> Result<Vec<T>, ScopeIndexCapacity>
where
    T: Clone,
{
    let mut values = Vec::new();
    values
        .try_reserve_exact(len)
        .map_err(|_| ScopeIndexCapacity)?;
    values.resize(len, value);
    Ok(values)
}

impl IncidenceIndex {
    pub(crate) fn new(core: &AnalyzedCore) -> Result<Self, ScopeIndexCapacity> {
        let topology = core
            .paragraph
            .scope_topology
            .as_ref()
            .expect("authenticated capture");
        let capture = &topology.capture;
        let flow = capture.canonical_flow().map_err(|_| ScopeIndexCapacity)?;
        let rank = |cut| flow.rank(topology.domain, cut);
        let events = &core.transform.events;
        let count = events.len().checked_add(1).ok_or(ScopeIndexCapacity)?;
        let transitions = flow.transitions(topology.domain);
        let mut occurrences: BTreeMap<usize, Occurrence> = BTreeMap::new();
        let mut points = Vec::new();
        let mut exact_aliases = BTreeMap::new();
        let mut represented = true;
        let mut atom_cut = 0;
        for (event_index, event) in events.iter().enumerate() {
            let mut support = Vec::new();
            match *event {
                RelationEvent::Text { atom } => {
                    atom_cut += 1;
                    match core.transform.component_for_atom(atom).owner {
                        ComponentOwner::Original(row) => support.extend(
                            core.transform.pieces[core.transform.rows[row].pieces.clone()]
                                .iter()
                                .map(|piece| {
                                    piece
                                        .origin
                                        .captured_event()
                                        .expect("captured transform origin retains its event")
                                }),
                        ),
                        ComponentOwner::Wbr(control) => {
                            support.push(topology.control_orders[control])
                        }
                        ComponentOwner::Virtual(index) => points.push((
                            event_index,
                            rank(core.transform.virtuals[index].capture_cut),
                            false,
                        )),
                    }
                    if support.is_empty()
                        && !matches!(
                            core.transform.component_for_atom(atom).owner,
                            ComponentOwner::Virtual(_)
                        )
                    {
                        represented = false;
                    }
                }
                RelationEvent::Removal { row, .. } => {
                    support.extend(
                        core.transform.pieces[core.transform.rows[row].pieces.clone()]
                            .iter()
                            .map(|piece| {
                                piece
                                    .origin
                                    .captured_event()
                                    .expect("captured transform removal retains its event")
                            }),
                    );
                    if support.is_empty() {
                        represented = false;
                    }
                }
                RelationEvent::GeneratedSuppression { component, .. } => match core
                    .transform
                    .components[component]
                    .owner
                {
                    ComponentOwner::Original(row) => {
                        support.extend(core.transform.original_pieces(row).iter().map(|piece| {
                            piece
                                .origin
                                .captured_event()
                                .expect("actual source occurrence")
                        }));
                    }
                    ComponentOwner::Wbr(control) => support.push(topology.control_orders[control]),
                    ComponentOwner::Virtual(index) => points.push((
                        event_index,
                        rank(core.transform.virtuals[index].capture_cut),
                        false,
                    )),
                },
                RelationEvent::PhaseIDeleted { source, .. } => {
                    support.extend(
                        core.paragraph
                            .origin_intersections(&core.processed, source)
                            .filter(|piece| !piece.aggregate.is_empty())
                            .map(|piece| {
                                core.paragraph
                                    .captured_origin_event(piece.origin)
                                    .expect("captured deletion retains its original occurrence")
                            }),
                    );
                    if support.is_empty() {
                        represented = false;
                    }
                }
                RelationEvent::Anchor { order } => match core.paragraph.anchor_order[order].item {
                    SemanticAnchorItem::EmptyOrigin(origin) => support.push(
                        core.paragraph
                            .captured_origin_event(origin)
                            .expect("captured empty origin retains its event"),
                    ),
                    SemanticAnchorItem::Atomic(atomic) => {
                        support.push(capture.atomic_orders[atomic])
                    }
                    SemanticAnchorItem::ComputedLineBoundary(index) => {
                        for &cause in &core.paragraph.computed_boundaries[index].causes {
                            points.push((
                                event_index,
                                rank(capture.computed[cause].capture_cut),
                                true,
                            ));
                        }
                    }
                    SemanticAnchorItem::Control(control) => {
                        let original = topology.control_orders[control];
                        let alias = match capture.events[original].event {
                            SourceProcessingEvent::Control { control, .. } => {
                                flow.endpoint(control)
                            }
                            _ => None,
                        };
                        if let Some(alias) = alias {
                            if anchor_exact(core, order, atom_cut) {
                                exact_aliases.insert(original, alias);
                                support.push(original);
                            }
                            // A snapped physical endpoint cannot move a
                            // canonical edge. Real original support remains.
                        } else {
                            support.push(original);
                        }
                    }
                },
            }
            for order in support {
                occurrences
                    .entry(order)
                    .and_modify(|value| value.last = event_index)
                    .or_insert(Occurrence {
                        first: event_index,
                        last: event_index,
                    });
            }
        }
        let limit = flow.len(topology.domain);
        let mut lower = allocated(count, Point(0, false))?;
        let mut upper = allocated(count, Point(limit, false))?;
        let mut changes = allocated(count, Vec::<(usize, bool)>::new())?;
        for (order, occurrence) in occurrences {
            let mut consumed = Point(rank(order + 1), false);
            let mut unconsumed = Point(rank(order), false);
            if let Some(&(edge, side)) = exact_aliases.get(&order) {
                match side {
                    SourceBoxSide::Before => unconsumed = Point(edge, false),
                    SourceBoxSide::After => consumed = Point(edge, false),
                }
            }
            lower[occurrence.last + 1] = lower[occurrence.last + 1].max(consumed);
            upper[occurrence.first] = upper[occurrence.first].min(unconsumed);
            if occurrence.first != occurrence.last {
                changes[occurrence.first + 1].push((rank(order), true));
                changes[occurrence.last + 1].push((rank(order), false));
            }
        }
        let mut pins = allocated(count, None)?;
        for (event, point, computed) in points {
            lower[event + 1] = lower[event + 1].max(Point(point, false));
            upper[event] = upper[event].min(Point(point, false));
            if computed {
                pins[event + 1] = Some(point);
            }
        }
        for i in 1..count {
            lower[i] = lower[i].max(lower[i - 1]);
        }
        for i in (0..count - 1).rev() {
            upper[i] = upper[i].min(upper[i + 1]);
        }
        let mut active = BTreeMap::<usize, usize>::new();
        let mut cuts = Vec::new();
        cuts.try_reserve_exact(count)
            .map_err(|_| ScopeIndexCapacity)?;
        for i in 0..count {
            for &(body, add) in &changes[i] {
                if add {
                    *active.entry(body).or_default() += 1;
                } else if let Some(count) = active.get_mut(&body) {
                    *count -= 1;
                    if *count == 0 {
                        active.remove(&body);
                    }
                }
            }
            let split = active
                .first_key_value()
                .zip(active.last_key_value())
                .map(|((&first, _), (&last, _))| (first, last));
            let valid = split.is_none_or(|(first, last)| {
                transitions.partition_point(|&edge| Point(edge, false) < Point(first, true))
                    == transitions.partition_point(|&edge| Point(edge, false) < Point(last, true))
            });
            cuts.push(Cut {
                lower: lower[i],
                upper: upper[i],
                split,
                pin: pins[i],
                valid,
            });
        }
        let mut edges = Vec::new();
        for ordinal in 0..topology.owner.len() {
            let raw = topology.owner.member(ordinal).index;
            let member_edges = flow.edges(raw);
            edges
                .try_reserve(member_edges.len())
                .map_err(|_| ScopeIndexCapacity)?;
            edges.extend_from_slice(member_edges);
        }
        edges.sort_unstable();
        edges.dedup();
        Ok(Self {
            cuts,
            canonical: Arc::clone(&flow),
            domain: topology.domain,
            edges,
            represented,
        })
    }

    /// The issued section restricts real canonical placements, including when
    /// several sections share one prepared participant cut. No byte offset or
    /// guessed paragraph label can establish this association.
    pub(crate) fn placements_in(
        &self,
        cut: usize,
        section: &crate::engine::analysis::structural::Section,
    ) -> Option<Vec<Placement>> {
        let (Some(entry), Some(exit), Some(span)) = (section.entry, section.exit, section.span)
        else {
            return None;
        };
        self.placements_within(
            cut,
            (
                self.canonical.rank(self.domain, span.entry),
                self.canonical.rank(self.domain, span.exit),
            ),
            (
                self.canonical.rank(self.domain, entry),
                self.canonical.rank(self.domain, exit),
            ),
        )
    }

    fn placements_within(
        &self,
        cut: usize,
        structural: (usize, usize),
        bounds: (usize, usize),
    ) -> Option<Vec<Placement>> {
        if !self.represented {
            return None;
        }
        let mut state = *self.cuts.get(cut)?;
        state.lower = state.lower.max(Point(structural.0, false));
        state.upper = state.upper.min(Point(structural.1, false));
        if !state.valid || state.lower > state.upper {
            return None;
        }
        let fits = |point| state.lower <= point && point <= state.upper;
        if let Some((first, last)) = state.split {
            return (state.pin.is_none() && fits(Point(first, true)) && fits(Point(last, true)))
                .then(|| vec![Placement::Body { first, last }]);
        }
        if let Some(pin) = state.pin {
            return fits(Point(pin, false)).then(|| vec![Placement::Cut(pin)]);
        }
        let edges = &self.edges;
        let start = edges.partition_point(|&edge| Point(edge, false) < state.lower);
        let end = edges.partition_point(|&edge| Point(edge, false) <= state.upper);
        if start < end {
            return Some(
                edges[start..end]
                    .iter()
                    .copied()
                    .map(Placement::Cut)
                    .collect(),
            );
        }
        let ordinary = state.lower.max(Point(bounds.0, false));
        (ordinary <= state.upper.min(Point(bounds.1, false)))
            .then(|| vec![Placement::Cut(ordinary.0)])
    }

    pub(crate) fn relation(
        &self,
        placement: Placement,
        scope: usize,
    ) -> Option<crate::CapturedScopeIncidence> {
        use crate::CapturedScopeIncidence as R;
        let (before, after) = self.canonical.scope_ranks(scope)?;
        let point = placement.point();
        Some(if point == Point(before, false) {
            R::AtBefore
        } else if point == Point(after, false) {
            R::AtAfter
        } else if point < Point(before, false) {
            R::Before
        } else if point > Point(after, false) {
            R::After
        } else {
            R::Inside
        })
    }
}
