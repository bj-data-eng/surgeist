//! Borrowed transformation facts from the actual source owner. This inventory
//! has no paragraph direction, strut, or authority to publish a joined output.

use std::{ops::Deref, sync::Arc};

use crate::{
    Error, ErrorCode, ErrorDetail, ParagraphConnectionKind, ProcessedSource, Range, Result,
    SourceContinuityOperation, SourceProcessingError, SourceProcessingEvent, Style, TextTransform,
    TextTransformError,
    engine::{
        OriginalAtom,
        resolve::{self, AnalyzedStyleOwner, AnalyzedStyleRun},
    },
    processing::{
        contextual::ParagraphTopology,
        lexical_inventory::{LexicalContext, LexicalDisposition, LexicalRecordKind, LexicalState},
    },
    projection::SourceProjection,
    protocol::paragraph::{
        ComputedBoundaryData, FlowOriginKey, OriginData, ParagraphControl, ParagraphData,
        SemanticAnchor, SemanticAnchorItem,
    },
};

pub(super) fn capacity() -> Error {
    Error::new(
        ErrorCode::SourceProcessingCapacity,
        "source transformation storage unavailable",
    )
    .with_detail(ErrorDetail::TextTransform {
        reason: TextTransformError::Capacity,
    })
}

pub(super) fn push<T>(rows: &mut Vec<T>, row: T) -> Result<()> {
    rows.try_reserve(1).map_err(|_| capacity())?;
    rows.push(row);
    Ok(())
}
pub(super) fn filled<T: Clone>(length: usize, value: T) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(length).map_err(|_| capacity())?;
    rows.resize(length, value);
    Ok(rows)
}
pub(super) fn collect_rows<T>(values: impl IntoIterator<Item = T>) -> Result<Vec<T>> {
    let mut rows = Vec::new();
    for value in values {
        push(&mut rows, value)?;
    }
    Ok(rows)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct MissingEdge {
    pub(super) before_origin: usize,
    pub(super) after_origin: usize,
    pub(super) at: usize,
    before_order: usize,
    after_order: usize,
}
impl MissingEdge {
    pub(super) fn unavailable(self, operation: SourceContinuityOperation) -> Error {
        Error::new(
            ErrorCode::UnsupportedFeature,
            "transformation needs source continuity",
        )
        .with_detail(ErrorDetail::SourceProcessing {
            reason: SourceProcessingError::MissingSourceContinuity {
                before_origin: self.before_origin,
                after_origin: self.after_origin,
                operation,
            },
        })
    }
}

pub(super) struct Original<'a> {
    pub(super) key: FlowOriginKey,
    pub(super) data: &'a OriginData,
}
impl Deref for Original<'_> {
    type Target = OriginData;
    fn deref(&self) -> &Self::Target {
        self.data
    }
}

pub(super) struct Connection {
    pub(super) at: usize,
    pub(super) kind: ParagraphConnectionKind,
    pub(super) before: usize,
    pub(super) after: usize,
    pub(super) diagnostic: usize,
}

#[derive(Clone, Copy)]
pub(super) struct StyleRef<'a> {
    authored: &'a Style,
}
impl<'a> StyleRef<'a> {
    fn new(authored: &'a Style) -> Self {
        Self { authored }
    }
    pub(super) fn authored(&self) -> &'a Style {
        self.authored
    }
    pub(super) fn text_transform(&self) -> TextTransform {
        self.authored.text_transform
    }
    pub(super) fn locale_tag(&self) -> Option<&str> {
        self.authored.locale.as_deref()
    }
    /// Resolve content language only when an actual phrase context reads it.
    /// A row's language is independent of its own separator request.
    pub(super) fn phrase_language(&self) -> Result<Option<fontique::Language>> {
        self.authored
            .locale
            .as_deref()
            .map(fontique::Language::parse)
            .transpose()
            .map_err(|_| {
                Error::new(ErrorCode::InvalidStyle, "invalid transform content locale").with_detail(
                    ErrorDetail::TextTransform {
                        reason: TextTransformError::InvalidContentLocale,
                    },
                )
            })
    }
}

pub(super) struct StyleRow<'a> {
    pub(super) owner: AnalyzedStyleOwner,
    pub(super) style: StyleRef<'a>,
}
impl StyleRow<'_> {
    pub(super) fn source_range(&self) -> Option<Range> {
        match self.owner {
            AnalyzedStyleOwner::Source(range) => Some(range),
            _ => None,
        }
    }
}

/// Every anchor retains the actual raw owner and its local occurrence. These
/// references, not equal source offsets, authorize final local conversion.
#[derive(Clone, Copy)]
struct AnchorOwner<'a> {
    paragraph: &'a ParagraphData,
    local: usize,
    order: usize,
}

/// A genuine source fence retains its producer occurrence until it is lowered
/// into the current mixed stream. Equal source positions do not alias causes.
pub(super) enum SourceFence {
    Disconnected(usize),
    Anchor(SemanticAnchorItem),
    BlockEvent(usize),
}
impl SourceFence {
    fn source(&self, context: &SourceContext<'_>) -> usize {
        match *self {
            Self::Disconnected(index) => context.connections[index].at,
            Self::Anchor(item) => context.anchor_position(item),
            Self::BlockEvent(order) => {
                context
                    .scope_topology
                    .expect("captured block event")
                    .capture
                    .events[order]
                    .at
            }
        }
    }
}

pub(crate) struct SourceContext<'a> {
    pub(super) processed: &'a ProcessedSource,
    pub(super) requested: &'a ParagraphData,
    pub(super) scope_topology: Option<&'a ParagraphTopology>,
    pub(super) origins: Vec<Original<'a>>,
    pub(super) controls: Vec<&'a ParagraphControl>,
    control_owners: Vec<(&'a ParagraphData, usize)>,
    pub(super) computed_boundaries: Vec<&'a ComputedBoundaryData>,
    pub(super) connections: Vec<Connection>,
    pub(super) connection_ends: Option<Vec<(usize, usize)>>,
    pub(super) connection_anchor_cuts: Option<Vec<usize>>,
    pub(super) anchor_order: Vec<SemanticAnchor>,
    anchor_owners: Vec<AnchorOwner<'a>>,
    pub(super) missing: Vec<MissingEdge>,
}

impl<'a> SourceContext<'a> {
    pub(super) fn separator_events(
        &self,
        projection: &SourceProjection,
        styles: &[StyleRow<'_>],
    ) -> Result<Vec<crate::engine::analysis::wrapping::LogicalEvent>> {
        let ranges = collect_rows(styles.iter().filter_map(StyleRow::source_range))?;
        self.capture_events(projection, &ranges, true)
    }
    pub(super) fn capture_order(&self, item: SemanticAnchorItem) -> usize {
        self.anchor_order
            .iter()
            .position(|anchor| anchor.item == item)
            .map(|index| self.anchor_owners[index].order)
            .expect("actual source anchor")
    }
    pub(crate) fn capture_events(
        &self,
        projection: &SourceProjection,
        style_ranges: &[Range],
        scalar_seams: bool,
    ) -> Result<Vec<crate::engine::analysis::wrapping::LogicalEvent>> {
        use crate::{
            engine::analysis::wrapping::{Event, LogicalEvent},
            processing::MappingKind,
        };
        let mut events = Vec::new();
        for (atom, value) in projection.atoms.iter().enumerate() {
            push(
                &mut events,
                (atom, 1usize, value.source.start, 0usize, Event::Text(atom)),
            )?;
        }
        let mut cuts = Vec::new();
        for origin in &self.origins {
            push(&mut cuts, origin.aggregate.start)?;
            push(&mut cuts, origin.aggregate.end)?;
        }
        for anchor in &self.anchor_order {
            push(&mut cuts, self.anchor_position(anchor.item))?;
        }
        cuts.sort_unstable();
        cuts.dedup();
        let mut collapsed_cuts = collect_rows(cuts.iter().copied())?;
        for range in style_ranges {
            push(&mut collapsed_cuts, range.start)?;
            push(&mut collapsed_cuts, range.end)?;
        }
        collapsed_cuts.sort_unstable();
        collapsed_cuts.dedup();
        for piece in self.processed.operations() {
            let MappingKind::Delete(cause) = piece.kind else {
                continue;
            };
            let own_cuts = if cause == crate::processing::DeletionCause::CollapsedSpace {
                &collapsed_cuts
            } else {
                &cuts
            };
            for piece_range in self.intersections(piece.source) {
                let first = own_cuts.partition_point(|at| *at <= piece_range.start);
                let last = own_cuts.partition_point(|at| *at < piece_range.end);
                let mut start = piece_range.start;
                for end in own_cuts[first..last]
                    .iter()
                    .copied()
                    .chain(std::iter::once(piece_range.end))
                {
                    let atom = projection
                        .atoms
                        .partition_point(|value| value.source.end <= start);
                    let lexical = self
                        .processed
                        .lexical
                        .deletion(Range::new(start, end), cause);
                    push(
                        &mut events,
                        (
                            atom,
                            0,
                            start,
                            usize::MAX,
                            Event::Deleted(
                                lexical.original_range(&self.processed.lexical),
                                piece.processed,
                                lexical,
                            ),
                        ),
                    )?;
                    start = end;
                }
            }
        }
        for (order, &anchor) in self.anchor_order.iter().enumerate() {
            let at = self.anchor_position(anchor.item);
            let atom = if scalar_seams {
                projection
                    .atoms
                    .partition_point(|atom| atom.source.end <= at)
            } else {
                self.anchor_atom(order, projection)
            };
            push(&mut events, (atom, 0, at, order, Event::Anchor(anchor)))?;
        }
        events.sort_by_key(|event| (event.0, event.1, event.2, event.3));
        Ok(events
            .into_iter()
            .map(|(atom, _, source, _, kind)| LogicalEvent { atom, source, kind })
            .collect())
    }
    /// Existing exact callers use the same event assembly without widening
    /// their source context. The actual analysis request uses `surrounding`.
    pub(crate) fn exact(
        processed: &'a ProcessedSource,
        paragraph: &'a ParagraphData,
    ) -> Result<Self> {
        Self::new(processed, paragraph, false)
    }
    pub(super) fn surrounding(
        processed: &'a ProcessedSource,
        paragraph: &'a ParagraphData,
    ) -> Result<Self> {
        Self::new(processed, paragraph, true)
    }

    fn new(
        processed: &'a ProcessedSource,
        requested: &'a ParagraphData,
        surrounding: bool,
    ) -> Result<Self> {
        let topology = requested.scope_topology.as_ref();
        let mut members = Vec::new();
        if surrounding && let Some(topology) = topology {
            for record in &processed.paragraphs {
                if record.data.scope_topology.as_ref().is_some_and(|other| {
                    other.domain == topology.domain
                        && Arc::ptr_eq(&other.capture, &topology.capture)
                }) {
                    push(&mut members, &record.data)?;
                }
            }
        } else {
            push(&mut members, requested)?;
        }
        let mut result = Self {
            processed,
            requested,
            scope_topology: topology,
            origins: Vec::new(),
            controls: Vec::new(),
            control_owners: Vec::new(),
            computed_boundaries: Vec::new(),
            connections: Vec::new(),
            connection_ends: None,
            connection_anchor_cuts: None,
            anchor_order: Vec::new(),
            anchor_owners: Vec::new(),
            missing: Vec::new(),
        };
        for &member in &members {
            for (index, data) in member.origins.iter().enumerate() {
                let key = member.origin_map.flow_origin(
                    member
                        .origin_map
                        .view(index)
                        .expect("admitted original row"),
                );
                push(&mut result.origins, Original { key, data })?;
            }
            for (index, control) in member.controls.iter().enumerate() {
                push(&mut result.controls, control)?;
                push(&mut result.control_owners, (member, index))?;
            }
            for boundary in &member.computed_boundaries {
                push(&mut result.computed_boundaries, boundary)?;
            }
        }
        result
            .origins
            .sort_unstable_by_key(|origin| match origin.key {
                FlowOriginKey::CapturedEvent(order) | FlowOriginKey::ExactOrdinal(order) => order,
            });
        for &member in &members {
            for (local, &anchor) in member.anchor_order.iter().enumerate() {
                let item = match anchor.item {
                    SemanticAnchorItem::EmptyOrigin(index) => SemanticAnchorItem::EmptyOrigin(
                        result
                            .origin_index(
                                member.origin_map.flow_origin(
                                    member
                                        .origin_map
                                        .view(index)
                                        .expect("actual empty original"),
                                ),
                            )
                            .expect("retained original"),
                    ),
                    SemanticAnchorItem::Control(index) => SemanticAnchorItem::Control(
                        result
                            .control_owners
                            .iter()
                            .position(|(owner, own)| std::ptr::eq(*owner, member) && *own == index)
                            .expect("actual retained control"),
                    ),
                    SemanticAnchorItem::ComputedLineBoundary(index) => {
                        SemanticAnchorItem::ComputedLineBoundary(
                            result
                                .computed_boundaries
                                .iter()
                                .position(|row| {
                                    std::ptr::eq(*row, &member.computed_boundaries[index])
                                })
                                .expect("actual retained computed boundary"),
                        )
                    }
                    SemanticAnchorItem::Atomic(index) => SemanticAnchorItem::Atomic(index),
                };
                let order = if let Some(topology) = &member.scope_topology {
                    match anchor.item {
                        SemanticAnchorItem::EmptyOrigin(index) => member
                            .captured_origin_event(index)
                            .expect("captured original"),
                        SemanticAnchorItem::Control(index) => topology.control_orders[index],
                        SemanticAnchorItem::ComputedLineBoundary(index) => {
                            member.computed_boundaries[index].capture_cut
                        }
                        SemanticAnchorItem::Atomic(index) => topology.capture.atomic_orders[index],
                    }
                } else {
                    local
                };
                push(&mut result.anchor_order, SemanticAnchor { item, ..anchor })?;
                push(
                    &mut result.anchor_owners,
                    AnchorOwner {
                        paragraph: member,
                        local,
                        order,
                    },
                )?;
            }
        }
        let mut order = collect_rows(0..result.anchor_order.len())?;
        order.sort_unstable_by_key(|&index| (result.anchor_owners[index].order, index));
        let anchors = std::mem::take(&mut result.anchor_order);
        let owners = std::mem::take(&mut result.anchor_owners);
        for index in order {
            push(&mut result.anchor_order, anchors[index])?;
            push(&mut result.anchor_owners, AnchorOwner { ..owners[index] })?;
        }
        if surrounding && let Some(topology) = topology {
            let capture = &topology.capture;
            let mut ends = Vec::new();
            let mut cuts = Vec::new();
            for (diagnostic, row) in capture.connections.iter().enumerate() {
                let before_key =
                    FlowOriginKey::CapturedEvent(capture.origin_orders[row.fact.before_origin]);
                let after_key =
                    FlowOriginKey::CapturedEvent(capture.origin_orders[row.fact.after_origin]);
                let (Some(before), Some(after)) = (
                    result.origin_index(before_key),
                    result.origin_index(after_key),
                ) else {
                    continue;
                };
                push(
                    &mut result.connections,
                    Connection {
                        at: result.origins[after].aggregate.start,
                        kind: row.fact.kind,
                        before,
                        after,
                        diagnostic,
                    },
                )?;
                push(
                    &mut ends,
                    (
                        result.origins[before].aggregate.end,
                        result.origins[after].aggregate.start,
                    ),
                )?;
                push(
                    &mut cuts,
                    result
                        .anchor_owners
                        .partition_point(|owner| owner.order < row.order),
                )?;
            }
            for (before, pair) in result.origins.windows(2).enumerate() {
                if result
                    .connections
                    .iter()
                    .any(|edge| edge.before == before && edge.after == before + 1)
                {
                    continue;
                }
                let input = |key: FlowOriginKey| match capture.events
                    [key.captured_event().expect("captured original")]
                .event
                {
                    SourceProcessingEvent::Origin { origin, .. } => origin,
                    _ => unreachable!("origin map retains actual Origin event"),
                };
                push(
                    &mut result.missing,
                    MissingEdge {
                        before_origin: input(pair[0].key),
                        after_origin: input(pair[1].key),
                        at: pair[1].aggregate.start,
                        before_order: pair[0].key.captured_event().expect("captured original"),
                        after_order: pair[1].key.captured_event().expect("captured original"),
                    },
                )?;
            }
            result.connection_ends = Some(ends);
            result.connection_anchor_cuts = Some(cuts);
        } else {
            for (index, edge) in requested.connections.iter().enumerate() {
                push(
                    &mut result.connections,
                    Connection {
                        at: edge.at,
                        kind: edge.kind,
                        before: index,
                        after: index + 1,
                        diagnostic: index,
                    },
                )?;
            }
            result.connection_ends = requested
                .connection_ends
                .as_ref()
                .map(|rows| collect_rows(rows.iter().copied()))
                .transpose()?;
            result.connection_anchor_cuts = requested
                .connection_anchor_cuts
                .as_ref()
                .map(|rows| collect_rows(rows.iter().copied()))
                .transpose()?;
        }
        Ok(result)
    }

    pub(super) fn hard_anchor(&self, item: SemanticAnchorItem) -> bool {
        match item {
            SemanticAnchorItem::Atomic(index) => {
                self.processed.source().boxes()[index].kind() == crate::InlineBoxKind::InFlow
            }
            SemanticAnchorItem::Control(index) => {
                self.controls[index].kind == crate::ParagraphControlKind::ForcedBreak
            }
            SemanticAnchorItem::ComputedLineBoundary(_) => true,
            SemanticAnchorItem::EmptyOrigin(_) => false,
        }
    }

    pub(super) fn fences(&self) -> Result<Vec<SourceFence>> {
        let mut result = Vec::new();
        for (index, connection) in self.connections.iter().enumerate() {
            if connection.kind == ParagraphConnectionKind::Disconnected {
                push(&mut result, SourceFence::Disconnected(index))?;
            }
        }
        for anchor in &self.anchor_order {
            if self.hard_anchor(anchor.item) {
                push(&mut result, SourceFence::Anchor(anchor.item))?;
            }
        }
        if let Some(topology) = self.scope_topology {
            for &order in &topology.capture.flow_orders[topology.domain] {
                let event = &topology.capture.events[order];
                if let SourceProcessingEvent::EnterScope(scope)
                | SourceProcessingEvent::ExitScope(scope) = event.event
                    && topology.capture.scopes[scope].domain != topology.domain
                    && matches!(
                        topology.capture.scopes[scope].definition,
                        Some(crate::processing::contextual::CapturedDomainData::Block)
                    )
                {
                    push(&mut result, SourceFence::BlockEvent(order))?;
                }
            }
        }
        Ok(result)
    }

    /// Original events use their admitted origin identity; anchors retain their
    /// actual captured occurrence. This is never a generated-output lookup.
    fn event_order(
        &self,
        event: &crate::engine::analysis::wrapping::LogicalEvent,
        projection: &SourceProjection,
    ) -> Option<usize> {
        use crate::engine::analysis::wrapping::Event;
        let source = match event.kind {
            Event::Text(atom) => projection.atoms[atom].source,
            Event::Deleted(source, ..) => source,
            Event::Anchor(anchor) => return Some(self.capture_order(anchor.item)),
        };
        self.origins
            .iter()
            .find(|origin| {
                origin.aggregate.start < source.end && source.start < origin.aggregate.end
            })
            .and_then(|origin| origin.key.captured_event())
    }

    pub(super) fn block_event_cut(
        &self,
        order: usize,
        projection: &SourceProjection,
        events: &[crate::engine::analysis::wrapping::LogicalEvent],
    ) -> usize {
        events
            .iter()
            .position(|event| {
                self.event_order(event, projection)
                    .is_some_and(|actual| actual >= order)
            })
            .unwrap_or(events.len())
    }

    pub(super) fn origin_index(&self, key: FlowOriginKey) -> Option<usize> {
        self.origins.iter().position(|row| row.key == key)
    }
    pub(super) fn missing_between(
        &self,
        before: &crate::engine::analysis::wrapping::LogicalEvent,
        after: &crate::engine::analysis::wrapping::LogicalEvent,
        projection: &SourceProjection,
    ) -> Option<MissingEdge> {
        let a = self.event_order(before, projection)?;
        let b = self.event_order(after, projection)?;
        self.missing
            .iter()
            .find(|edge| a.min(b) < edge.after_order && edge.before_order < a.max(b))
            .copied()
    }
    /// Exact output custody comes from the actual capture-cut paragraph. The
    /// same immutable capture owns root demand and final virtual projection.
    pub(super) fn owns_capture_cut(&self, cut: usize) -> bool {
        let Some(topology) = self.scope_topology else {
            return false;
        };
        topology
            .capture
            .events
            .get(cut)
            .is_some_and(|event| event.paragraph == Some(topology.paragraph))
    }
    pub(super) fn intersections(&self, source: Range) -> impl Iterator<Item = Range> + '_ {
        self.origins.iter().filter_map(move |origin| {
            let start = source.start.max(origin.aggregate.start);
            let end = source.end.min(origin.aggregate.end);
            (start < end).then_some(Range::new(start, end))
        })
    }
    pub(super) fn anchor_position(&self, item: SemanticAnchorItem) -> usize {
        match item {
            SemanticAnchorItem::EmptyOrigin(index) => self.origins[index].aggregate.start,
            SemanticAnchorItem::Control(index) => self.controls[index].at,
            SemanticAnchorItem::Atomic(index) => self.processed.source().boxes()[index].index(),
            SemanticAnchorItem::ComputedLineBoundary(index) => self.computed_boundaries[index].at,
        }
    }
    pub(super) fn project_anchor(
        &self,
        order: usize,
        projection: &SourceProjection,
        index: usize,
    ) -> usize {
        let owner = &self.anchor_owners[order];
        crate::engine::analysis::project_anchor_atom(
            owner.local,
            owner.paragraph,
            self.processed.source().boxes(),
            projection,
            index,
        )
    }
    pub(super) fn anchor_atom(&self, order: usize, projection: &SourceProjection) -> usize {
        if let Some(Some(atom)) = projection.anchor_atoms.get(order) {
            return *atom;
        }
        let source = self.anchor_position(self.anchor_order[order].item);
        self.project_anchor(
            order,
            projection,
            projection
                .atoms
                .partition_point(|atom| atom.source.end <= source),
        )
    }
    pub(super) fn local_anchor(&self, order: usize) -> Option<usize> {
        let owner = &self.anchor_owners[order];
        // Requested ParagraphData is an immutable clone of its issued row.
        // Captured event correspondence supplies its actual original occurrence.
        let anchor = owner.paragraph.anchor_order[owner.local];
        self.requested.anchor_order.iter().position(|candidate| {
            match (candidate.item, anchor.item) {
                (SemanticAnchorItem::EmptyOrigin(left), SemanticAnchorItem::EmptyOrigin(right)) => {
                    self.requested.origin_map.flow_origin(
                        self.requested
                            .origin_map
                            .view(left)
                            .expect("actual original"),
                    ) == owner.paragraph.origin_map.flow_origin(
                        owner
                            .paragraph
                            .origin_map
                            .view(right)
                            .expect("actual original"),
                    )
                }
                (SemanticAnchorItem::Atomic(left), SemanticAnchorItem::Atomic(right)) => {
                    left == right
                }
                (SemanticAnchorItem::Control(left), SemanticAnchorItem::Control(right)) => {
                    match (
                        &self.requested.scope_topology,
                        &owner.paragraph.scope_topology,
                    ) {
                        (Some(a), Some(b)) => {
                            a.control_orders[left] == b.control_orders[right]
                                && Arc::ptr_eq(&a.capture, &b.capture)
                        }
                        (None, None) => {
                            std::ptr::eq(self.requested, owner.paragraph) && left == right
                        }
                        _ => false,
                    }
                }
                (
                    SemanticAnchorItem::ComputedLineBoundary(left),
                    SemanticAnchorItem::ComputedLineBoundary(right),
                ) => {
                    let a = &self.requested.computed_boundaries[left];
                    let b = &owner.paragraph.computed_boundaries[right];
                    a.capture_cut == b.capture_cut && a.causes == b.causes
                }
                _ => false,
            }
        })
    }
    pub(super) fn local_control(&self, control: usize) -> Option<usize> {
        self.anchor_order
            .iter()
            .position(|anchor| anchor.item == SemanticAnchorItem::Control(control))
            .and_then(|order| self.local_anchor(order))
            .and_then(|order| match self.requested.anchor_order[order].item {
                SemanticAnchorItem::Control(index) => Some(index),
                _ => None,
            })
    }

    pub(super) fn projection(&self, exact: SourceProjection) -> Result<SourceProjection> {
        let (
            Some(topology),
            LexicalState::Executed {
                context: LexicalContext::Captured(capture),
                records,
            },
        ) = (self.scope_topology, &self.processed.lexical)
        else {
            return Ok(exact);
        };
        if !Arc::ptr_eq(capture, &topology.capture) {
            return Err(crate::processing::contextual::invalid(
                SourceProcessingError::ForeignParagraph,
            ));
        }
        let mut atoms = Vec::new();
        for record in records {
            if record.domain != topology.domain {
                continue;
            }
            let LexicalRecordKind::Character {
                source,
                disposition: LexicalDisposition::Emitted { processed },
                ..
            } = record.kind
            else {
                continue;
            };
            let units =
                &self.processed.processed_text().code_units()[processed.start..processed.end];
            let mut offset = 0;
            for decoded in char::decode_utf16(units.iter().copied()) {
                let (code_point, glyph, length) = match decoded {
                    Ok(ch) => (u32::from(ch), ch, ch.len_utf16()),
                    Err(error) => (u32::from(error.unpaired_surrogate()), '\u{fffd}', 1),
                };
                push(
                    &mut atoms,
                    OriginalAtom {
                        source,
                        processed: Range::new(
                            processed.start + offset,
                            processed.start + offset + length,
                        ),
                        presentation: Range::new(
                            processed.start + offset,
                            processed.start + offset + length,
                        ),
                        code_point,
                        glyph,
                        glyph_bytes: 0..0,
                    },
                )?;
                offset += length;
            }
        }
        let mut barriers = Vec::new();
        for fence in self.fences()? {
            push(&mut barriers, fence.source(self))?;
        }
        barriers.sort_unstable();
        barriers.dedup();
        let cuts = collect_rows(
            barriers
                .iter()
                .map(|&at| atoms.partition_point(|atom| atom.source.end <= at)),
        )?;
        let end = atoms.last().map_or(0, |atom| atom.presentation.end);
        SourceProjection::from_atoms_at_barriers(atoms, end, barriers, cuts)
    }

    pub(super) fn styles(&self) -> Result<Vec<StyleRow<'a>>> {
        let default = if let Some(topology) = self.scope_topology {
            topology.capture.scopes[topology.capture.domain_scopes[topology.domain]]
                .style
                .as_ref()
                .expect("domain root retains actual base style")
        } else {
            self.requested.base_style(self.processed)
        };
        // Source membership and effective style seams survive lexical deletion.
        // This inventory is independent of which source characters emit atoms.
        let extent = self
            .origins
            .first()
            .zip(self.origins.last())
            .map_or(self.requested.extent(self.processed), |(first, last)| {
                Range::new(first.aggregate.start, last.aggregate.end)
            });
        let mut cuts = collect_rows([extent.start, extent.end])?;
        for span in self.processed.source().spans() {
            push(
                &mut cuts,
                span.range().start.clamp(extent.start, extent.end),
            )?;
            push(&mut cuts, span.range().end.clamp(extent.start, extent.end))?;
        }
        cuts.sort_unstable();
        cuts.dedup();
        let mut rows: Vec<StyleRow<'a>> = Vec::new();
        for pair in cuts.windows(2) {
            for range in self.intersections(Range::new(pair[0], pair[1])) {
                let authored =
                    resolve::winning_span(self.processed.source().spans(), range, |span| {
                        span.range()
                    })
                    .map_or(default, |span| span.style());
                if (self.scope_topology.is_some() || self.requested.coverage.is_some())
                    && let Some(last) = rows.last_mut()
                    && last.style.authored() == authored
                    && let AnalyzedStyleOwner::Source(own) = &mut last.owner
                    && own.end == range.start
                {
                    own.end = range.end;
                } else {
                    push(
                        &mut rows,
                        StyleRow {
                            owner: AnalyzedStyleOwner::Source(range),
                            style: StyleRef::new(authored),
                        },
                    )?;
                }
            }
        }
        if rows.is_empty() {
            push(
                &mut rows,
                StyleRow {
                    owner: AnalyzedStyleOwner::Source(Range::new(extent.start, extent.start)),
                    style: StyleRef::new(default),
                },
            )?;
        }
        for (index, control) in self.controls.iter().enumerate() {
            if let crate::ParagraphControlKind::Wbr { style }
            | crate::ParagraphControlKind::DiscretionaryBreak { style } = &control.kind
            {
                push(
                    &mut rows,
                    StyleRow {
                        owner: AnalyzedStyleOwner::Control(index),
                        style: StyleRef::new(style),
                    },
                )?;
            }
        }
        Ok(rows)
    }

    pub(super) fn append_scope_style(
        &self,
        rows: &mut Vec<StyleRow<'a>>,
        scope: usize,
    ) -> Result<usize> {
        if let Some(index) = rows
            .iter()
            .position(|row| matches!(row.owner, AnalyzedStyleOwner::Scope(own) if own == scope))
        {
            return Ok(index);
        }
        let topology = self
            .scope_topology
            .expect("actual generated separator topology");
        let authored = topology.capture.scopes[scope]
            .style
            .as_ref()
            .expect("qualified own scope style");
        let index = rows.len();
        push(
            rows,
            StyleRow {
                owner: AnalyzedStyleOwner::Scope(scope),
                style: StyleRef::new(authored),
            },
        )?;
        Ok(index)
    }

    pub(super) fn local_style(
        &self,
        row: &StyleRow<'_>,
        source: Range,
        output: &mut Vec<AnalyzedStyleRun>,
    ) -> Result<usize> {
        match row.owner {
            AnalyzedStyleOwner::Source(_) => Ok(resolve::source_style(output, source.start)),
            AnalyzedStyleOwner::Control(index) => {
                let control = self
                    .local_control(index)
                    .expect("published generating control belongs to requested view");
                Ok(output.iter().position(|row| matches!(row.owner, AnalyzedStyleOwner::Control(own) if own == control)).expect("validated generating control style"))
            }
            AnalyzedStyleOwner::Scope(scope) => {
                if let Some(index) = output.iter().position(
                    |row| matches!(row.owner, AnalyzedStyleOwner::Scope(own) if own == scope),
                ) {
                    return Ok(index);
                }
                let index = output.len();
                push(
                    output,
                    AnalyzedStyleRun {
                        owner: AnalyzedStyleOwner::Scope(scope),
                        style: crate::ValidatedStyle::for_processed(row.style.authored().clone())?,
                    },
                )?;
                Ok(index)
            }
        }
    }
}

pub(super) fn source_style(styles: &[StyleRow<'_>], at: usize) -> usize {
    let count = styles.partition_point(|row| row.source_range().is_some());
    styles[..count]
        .partition_point(|row| row.source_range().is_some_and(|range| range.end <= at))
        .min(count - 1)
}
