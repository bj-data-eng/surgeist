//! Complete producer captures. The stream supplies order; tables supply facts.
use super::{DocumentSegmentBreaks, ProcessedSource};
use crate::protocol::paragraph::{OriginData, OriginSource, ParagraphData};
use crate::{
    Affinity, AnalyzedSource, InlineBoxKind, ParagraphAnchor, ParagraphAnchorItem,
    ParagraphBaseDirection, ParagraphBoundaryDefinition, ParagraphBoundaryPolicy,
    ParagraphConnection, ParagraphConnectionKind, ParagraphControl, ParagraphControlKind,
    ParagraphOrigin, ParagraphReplacement, ParagraphReplacementDefinition, ParagraphStrut, Range,
    Result, Source, Style,
};
use std::sync::Arc;

/// Computed non-inherited trim facts for one actual inline or block box.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WhiteSpaceTrim {
    pub discard_before: bool,
    pub discard_after: bool,
    pub discard_inner: bool,
}

/// Actual box role. Block domains capture their entire effective base style.
#[derive(Clone, Debug)]
pub enum SourceProcessingScope {
    Inline {
        trim: WhiteSpaceTrim,
    },
    Block {
        trim: WhiteSpaceTrim,
        base_style: Style,
    },
    AtomicBlock {
        trim: WhiteSpaceTrim,
        base_style: Style,
        atomic: usize,
    },
    OutOfFlowBlock {
        trim: WhiteSpaceTrim,
        base_style: Style,
        atomic: usize,
    },
}
impl SourceProcessingScope {
    fn trim(&self) -> WhiteSpaceTrim {
        match self {
            Self::Inline { trim }
            | Self::Block { trim, .. }
            | Self::AtomicBlock { trim, .. }
            | Self::OutOfFlowBlock { trim, .. } => *trim,
        }
    }
    fn style(&self) -> Option<&Style> {
        match self {
            Self::Inline { .. } => None,
            Self::Block { base_style, .. }
            | Self::AtomicBlock { base_style, .. }
            | Self::OutOfFlowBlock { base_style, .. } => Some(base_style),
        }
    }
}

/// Actual paragraph facts, captured before trimming or Unicode analysis.
#[derive(Clone, Copy, Debug)]
pub struct SourceProcessingParagraph {
    pub base_direction: ParagraphBaseDirection,
    pub paragraph_strut: ParagraphStrut,
}

/// A connection has two genuine original sides, including across domain holes.
#[derive(Clone, Copy, Debug)]
pub struct SourceProcessingConnection {
    pub before_origin: usize,
    pub after_origin: usize,
    pub kind: ParagraphConnectionKind,
}

/// Sole ordered occurrence stream. References are capture-local table ordinals.
#[derive(Clone, Copy, Debug)]
pub enum SourceProcessingEvent {
    EnterScope(usize),
    ExitScope(usize),
    EnterParagraph(usize),
    ExitParagraph(usize),
    Origin { origin: usize, affinity: Affinity },
    Connection(usize),
    Atomic { atomic: usize, affinity: Affinity },
    Control { control: usize, affinity: Affinity },
}

/// Whether the producer captured all context needed by exterior trimming.
/// This is a producer contract, not authentication of a live DOM or Style tree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceProcessingEnvelope {
    Complete,
    IncompleteExterior,
}

/// A complete immutable envelope. Source::boxes is the only atomic inventory.
/// Each actual origin, scope, paragraph, control and connection occurs exactly
/// once in `events`; ranges and nesting are not repeated in scope/paragraph facts.
#[derive(Clone, Debug)]
pub struct ContextualSourceInput {
    pub source: Source,
    pub segment_breaks: DocumentSegmentBreaks,
    pub envelope: SourceProcessingEnvelope,
    pub origins: Vec<ParagraphOrigin>,
    pub connections: Vec<SourceProcessingConnection>,
    pub controls: Vec<ParagraphControl>,
    pub boundary_policies: Vec<ParagraphBoundaryPolicy>,
    pub replacements: Vec<ParagraphReplacement>,
    pub scopes: Vec<SourceProcessingScope>,
    pub paragraphs: Vec<SourceProcessingParagraph>,
    pub events: Vec<SourceProcessingEvent>,
}

/// Typed atomic capture/view rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceProcessingError {
    IncompleteExterior,
    Nesting,
    Occurrence,
    OriginalCoverage,
    OriginCorrespondence,
    Domain,
    DelimiterCrossing,
    Connection,
    BoundaryPolicy,
    Replacement,
    OutsideParagraph,
    ForeignParagraph,
}
pub(crate) fn invalid(reason: SourceProcessingError) -> crate::Error {
    crate::Error::new(
        crate::ErrorCode::InvalidSourceProcessing,
        "invalid contextual source processing",
    )
    .with_detail(crate::ErrorDetail::SourceProcessing { reason })
}

#[derive(Debug)]
pub(crate) struct ParagraphRecord {
    pub(crate) data: ParagraphData,
}

/// Owner-issued view in original envelope coordinates. Clones retain authority.
#[derive(Clone, Debug)]
pub struct ProcessedParagraph {
    owner: Arc<ProcessedSource>,
    index: usize,
}
impl ProcessedParagraph {
    #[must_use]
    pub fn processed_source(&self) -> &Arc<ProcessedSource> {
        &self.owner
    }
    #[must_use]
    /// Original envelope extent; internal child domains can leave holes.
    pub fn source_range(&self) -> Range {
        self.data().extent(&self.owner)
    }
    /// Exact ordered original contributions, including genuine empty origins.
    pub fn source_pieces(&self) -> impl ExactSizeIterator<Item = Range> + '_ {
        self.data()
            .coverage
            .as_ref()
            .expect("issued contextual coverage")
            .iter()
            .copied()
    }
    pub fn plaintext_items(
        &self,
        affinity: Affinity,
    ) -> impl Iterator<Item = super::PlaintextItem<'_>> {
        let mut items = Vec::new();
        for item in self.owner.plaintext_items(affinity) {
            match item {
                super::PlaintextItem::InlineBox(box_) => {
                    if self
                        .data()
                        .anchor_order
                        .iter()
                        .any(|anchor| anchor.item == ParagraphAnchorItem::Atomic(box_.ordinal()))
                    {
                        items.push(item);
                    }
                }
                super::PlaintextItem::Text(text) => {
                    for piece in self.data().intersections(self.source_range()) {
                        let start = text
                            .range
                            .start
                            .0
                            .max(self.owner.project(piece.start, Affinity::Before));
                        let end = text
                            .range
                            .end
                            .0
                            .min(self.owner.project(piece.end, Affinity::After));
                        if start < end {
                            items.push(super::PlaintextItem::Text(super::ProcessedText {
                                units: &self.owner.processed.code_units()[start..end],
                                range: super::processed_range(start, end),
                            }));
                        }
                    }
                }
            }
        }
        items.into_iter()
    }
    pub(crate) fn data(&self) -> &ParagraphData {
        &self.owner.paragraphs[self.index].data
    }
}
impl ProcessedSource {
    pub fn paragraphs(self: &Arc<Self>) -> impl ExactSizeIterator<Item = ProcessedParagraph> + '_ {
        (0..self.paragraphs.len()).map(|index| ProcessedParagraph {
            owner: Arc::clone(self),
            index,
        })
    }
    /// Checks actual owner allocation before admitting an issued paragraph.
    pub fn admit_paragraph(self: &Arc<Self>, paragraph: &ProcessedParagraph) -> Result<()> {
        if Arc::ptr_eq(self, &paragraph.owner) {
            Ok(())
        } else {
            Err(invalid(SourceProcessingError::ForeignParagraph))
        }
    }
}
impl AnalyzedSource {
    /// Analyzes exactly the retained issued coverage and captured domain style.
    pub fn from_processed_paragraph(paragraph: ProcessedParagraph) -> Result<Self> {
        let data = paragraph.data().clone();
        Ok(Self::from_inner(crate::engine::analyze_paragraph(
            paragraph.owner,
            data,
        )?))
    }
}

#[derive(Clone, Copy)]
struct Frame {
    scope: usize,
    domain: usize,
}
#[derive(Default)]
struct Capture {
    origins: Vec<usize>,
    connections: Vec<usize>,
    anchors: Vec<ParagraphAnchor>,
    controls: Vec<usize>,
    connection_orders: Vec<usize>,
    start: usize,
    domain: usize,
}
pub(super) struct Domain {
    pub(super) style: Style,
}
pub(super) struct TrimBoundary {
    pub(super) scope: usize,
    pub(super) domain: usize,
    pub(super) at: usize,
    pub(super) order: usize,
    pub(super) exterior_domain: usize,
    pub(super) enter: bool,
    pub(super) trim: WhiteSpaceTrim,
    pub(super) block: bool,
    pub(super) atomic: Option<usize>,
}
pub(super) struct ProcessingBarrier {
    pub(super) at: usize,
    pub(super) domain: usize,
    pub(super) order: usize,
    pub(super) plaintext_reset: bool,
    pub(super) atomic: Option<usize>,
    pub(super) scope: Option<usize>,
}
pub(super) struct ProcessingContext {
    pub(super) domains: Vec<Domain>,
    pub(super) origins: Vec<(Range, usize)>,
    pub(super) boundaries: Vec<TrimBoundary>,
    pub(super) barriers: Vec<ProcessingBarrier>,
}

fn take(seen: &mut [bool], index: usize) -> Result<()> {
    let item = seen
        .get_mut(index)
        .ok_or_else(|| invalid(SourceProcessingError::Occurrence))?;
    if *item {
        return Err(invalid(SourceProcessingError::Occurrence));
    }
    *item = true;
    Ok(())
}

/// Trims first, executes the shared Phase I lexer once, and freezes views.
pub fn process_contextual_source(input: ContextualSourceInput) -> Result<ProcessedSource> {
    if input.envelope != SourceProcessingEnvelope::Complete {
        return Err(invalid(SourceProcessingError::IncompleteExterior));
    }
    crate::source_model::validate_structure(&input.source)?;
    super::validate_delimiters(&input.source, &input.segment_breaks)?;
    let mut scopes = vec![false; input.scopes.len()];
    let mut paragraphs = vec![false; input.paragraphs.len()];
    let mut origins = vec![false; input.origins.len()];
    let mut connections = vec![false; input.connections.len()];
    let mut atomics = vec![false; input.source.boxes().len()];
    let mut controls = vec![false; input.controls.len()];
    let mut associations = vec![false; input.source.boxes().len()];
    let mut stack: Vec<Frame> = Vec::new();
    let mut active_paragraphs: Vec<Option<usize>> = Vec::new();
    let mut captures: Vec<Capture> = (0..input.paragraphs.len())
        .map(|_| Capture::default())
        .collect();
    let mut context = ProcessingContext {
        domains: Vec::new(),
        origins: Vec::new(),
        boundaries: Vec::new(),
        barriers: Vec::new(),
    };
    let mut cursor = 0;
    let mut paragraph_order = Vec::new();
    let mut atomic_order = Vec::new();
    for (order, event) in input.events.iter().enumerate() {
        let domain = stack.last().map(|frame| frame.domain);
        let paragraph = domain.and_then(|domain| active_paragraphs[domain]);
        match *event {
            SourceProcessingEvent::EnterScope(index) => {
                take(&mut scopes, index)?;
                let scope = &input.scopes[index];
                let own = if let Some(style) = scope.style() {
                    if matches!(scope, SourceProcessingScope::Block { .. }) && paragraph.is_some() {
                        return Err(invalid(SourceProcessingError::Domain));
                    }
                    if let SourceProcessingScope::AtomicBlock { atomic, .. }
                    | SourceProcessingScope::OutOfFlowBlock { atomic, .. } = scope
                    {
                        take(&mut associations, *atomic)?;
                        let expected = if matches!(scope, SourceProcessingScope::AtomicBlock { .. })
                        {
                            InlineBoxKind::InFlow
                        } else {
                            InlineBoxKind::OutOfFlow
                        };
                        if paragraph.is_none()
                            || !atomics.get(*atomic).copied().unwrap_or(false)
                            || input.source.boxes()[*atomic].index() != cursor
                            || input.source.boxes()[*atomic].kind() != expected
                        {
                            return Err(invalid(SourceProcessingError::Domain));
                        }
                    }
                    if matches!(scope, SourceProcessingScope::Block { .. })
                        && let Some(parent) = domain
                    {
                        context.barriers.push(ProcessingBarrier {
                            at: cursor,
                            domain: parent,
                            order,
                            plaintext_reset: false,
                            atomic: None,
                            scope: Some(index),
                        });
                    }
                    let own = context.domains.len();
                    context.domains.push(Domain {
                        style: style.clone(),
                    });
                    active_paragraphs.push(None);
                    own
                } else {
                    domain.ok_or_else(|| invalid(SourceProcessingError::Domain))?
                };
                context.boundaries.push(TrimBoundary {
                    scope: index,
                    domain: own,
                    at: cursor,
                    order,
                    exterior_domain: domain.unwrap_or(own),
                    enter: true,
                    trim: scope.trim(),
                    block: scope.style().is_some(),
                    atomic: match scope {
                        SourceProcessingScope::AtomicBlock { atomic, .. }
                        | SourceProcessingScope::OutOfFlowBlock { atomic, .. } => Some(*atomic),
                        _ => None,
                    },
                });
                stack.push(Frame {
                    scope: index,
                    domain: own,
                });
            }
            SourceProcessingEvent::ExitScope(index) => {
                let Some(Frame {
                    scope: actual,
                    domain: own,
                }) = stack.pop()
                else {
                    return Err(invalid(SourceProcessingError::Nesting));
                };
                if actual != index {
                    return Err(invalid(SourceProcessingError::Nesting));
                }
                let scope = &input.scopes[index];
                if scope.style().is_some() && active_paragraphs[own].is_some() {
                    return Err(invalid(SourceProcessingError::Nesting));
                }
                if matches!(scope, SourceProcessingScope::Block { .. })
                    && let Some(parent) = stack.last().map(|frame| frame.domain)
                {
                    context.barriers.push(ProcessingBarrier {
                        at: cursor,
                        domain: parent,
                        order,
                        plaintext_reset: true,
                        atomic: None,
                        scope: Some(index),
                    });
                }
                context.boundaries.push(TrimBoundary {
                    scope: index,
                    domain: own,
                    at: cursor,
                    order,
                    exterior_domain: stack.last().map(|frame| frame.domain).unwrap_or(own),
                    enter: false,
                    trim: scope.trim(),
                    block: scope.style().is_some(),
                    atomic: match scope {
                        SourceProcessingScope::AtomicBlock { atomic, .. }
                        | SourceProcessingScope::OutOfFlowBlock { atomic, .. } => Some(*atomic),
                        _ => None,
                    },
                });
            }
            SourceProcessingEvent::EnterParagraph(index) => {
                take(&mut paragraphs, index)?;
                paragraph_order.push(index);
                let own = domain.ok_or_else(|| invalid(SourceProcessingError::Domain))?;
                if paragraph.is_some() {
                    return Err(invalid(SourceProcessingError::Nesting));
                }
                captures[index].start = cursor;
                captures[index].domain = own;
                active_paragraphs[own] = Some(index);
            }
            SourceProcessingEvent::ExitParagraph(index) => {
                let own = domain.ok_or_else(|| invalid(SourceProcessingError::Nesting))?;
                if active_paragraphs[own] != Some(index) {
                    return Err(invalid(SourceProcessingError::Nesting));
                }
                active_paragraphs[own] = None;
            }
            SourceProcessingEvent::Origin { origin, affinity } => {
                take(&mut origins, origin)?;
                let p = paragraph.ok_or_else(|| invalid(SourceProcessingError::Domain))?;
                let row = &input.origins[origin];
                if row.aggregate.start != cursor
                    || row.aggregate.start > row.aggregate.end
                    || row.aggregate.end > input.source.text().len()
                    || row.local.start > row.local.end
                    || row.local.end > row.source.text().len()
                    || row.local.len() != row.aggregate.len()
                {
                    return Err(invalid(SourceProcessingError::OriginalCoverage));
                }
                if row.source.text().code_units()[row.local.start..row.local.end]
                    != input.source.text().code_units()[row.aggregate.start..row.aggregate.end]
                {
                    return Err(invalid(SourceProcessingError::OriginCorrespondence));
                }
                let local = captures[p].origins.len();
                captures[p].origins.push(origin);
                if row.aggregate.is_empty() {
                    captures[p].anchors.push(ParagraphAnchor {
                        item: ParagraphAnchorItem::EmptyOrigin(local),
                        affinity,
                    });
                } else {
                    context
                        .origins
                        .push((row.aggregate, domain.expect("admitted domain")));
                }
                cursor = row.aggregate.end;
            }
            SourceProcessingEvent::Connection(index) => {
                take(&mut connections, index)?;
                let p = paragraph.ok_or_else(|| invalid(SourceProcessingError::Domain))?;
                let row = input.connections[index];
                if captures[p].origins.last() != Some(&row.before_origin)
                    || input
                        .origins
                        .get(row.after_origin)
                        .is_none_or(|origin| origin.aggregate.start != cursor)
                {
                    return Err(invalid(SourceProcessingError::Connection));
                }
                captures[p].connections.push(index);
                captures[p].connection_orders.push(order);
            }
            SourceProcessingEvent::Atomic { atomic, affinity } => {
                take(&mut atomics, atomic)?;
                atomic_order.push(atomic);
                let p = paragraph.ok_or_else(|| invalid(SourceProcessingError::Domain))?;
                let box_ = input.source.boxes()[atomic];
                if box_.index() != cursor {
                    return Err(invalid(SourceProcessingError::Occurrence));
                }
                captures[p].anchors.push(ParagraphAnchor {
                    item: ParagraphAnchorItem::Atomic(atomic),
                    affinity,
                });
                if box_.kind() == InlineBoxKind::InFlow {
                    context.barriers.push(ProcessingBarrier {
                        at: cursor,
                        domain: domain.expect("admitted domain"),
                        order,
                        plaintext_reset: false,
                        atomic: Some(atomic),
                        scope: None,
                    });
                }
            }
            SourceProcessingEvent::Control { control, affinity } => {
                take(&mut controls, control)?;
                let p = paragraph.ok_or_else(|| invalid(SourceProcessingError::Domain))?;
                if input.controls[control].at != cursor {
                    return Err(invalid(SourceProcessingError::Occurrence));
                }
                let local = captures[p].controls.len();
                captures[p].controls.push(control);
                captures[p].anchors.push(ParagraphAnchor {
                    item: ParagraphAnchorItem::Control(local),
                    affinity,
                });
                if input.controls[control].kind == ParagraphControlKind::ForcedBreak {
                    context.barriers.push(ProcessingBarrier {
                        at: cursor,
                        domain: domain.expect("admitted domain"),
                        order,
                        plaintext_reset: true,
                        atomic: None,
                        scope: None,
                    });
                }
            }
        }
    }
    if !stack.is_empty() || active_paragraphs.iter().any(Option::is_some) {
        return Err(invalid(SourceProcessingError::Nesting));
    }
    if cursor != input.source.text().len() {
        return Err(invalid(SourceProcessingError::OriginalCoverage));
    }
    if [
        &scopes,
        &paragraphs,
        &origins,
        &connections,
        &atomics,
        &controls,
    ]
    .iter()
    .any(|seen| seen.iter().any(|seen| !seen))
    {
        return Err(invalid(SourceProcessingError::Occurrence));
    }
    if context.domains.is_empty() {
        return Err(invalid(SourceProcessingError::Domain));
    }
    for barrier in &context.barriers {
        if super::delimiter_crosses(&input.segment_breaks, barrier.at) {
            return Err(invalid(SourceProcessingError::DelimiterCrossing));
        }
    }
    for boundary in &context.boundaries {
        if super::delimiter_crosses(&input.segment_breaks, boundary.at) {
            return Err(invalid(SourceProcessingError::DelimiterCrossing));
        }
    }
    for domain in 0..context.domains.len() {
        if !captures.iter().any(|capture| capture.domain == domain) {
            return Err(invalid(SourceProcessingError::Domain));
        }
    }
    if let DocumentSegmentBreaks::Defined(ranges) = &input.segment_breaks {
        for range in ranges {
            let mut domain = None;
            for (piece, own) in &context.origins {
                if piece.start < range.end && range.start < piece.end {
                    if domain.is_some_and(|domain| domain != *own) {
                        return Err(invalid(SourceProcessingError::DelimiterCrossing));
                    }
                    domain = Some(*own);
                }
            }
        }
    }
    let mut records = Vec::new();
    let mut used_policies = vec![false; input.boundary_policies.len()];
    let mut used_replacements = vec![false; input.replacements.len()];
    for (index, capture) in captures.into_iter().enumerate() {
        if capture.connections.len() != capture.origins.len().saturating_sub(1) {
            return Err(invalid(SourceProcessingError::Connection));
        }
        let mut native_connections = Vec::new();
        let mut ends = Vec::new();
        for ((pair, &connection), &order) in capture
            .origins
            .windows(2)
            .zip(&capture.connections)
            .zip(&capture.connection_orders)
        {
            let row = input.connections[connection];
            if row.before_origin != pair[0] || row.after_origin != pair[1] {
                return Err(invalid(SourceProcessingError::Connection));
            }
            let before = input.origins[pair[0]].aggregate.end;
            let after = input.origins[pair[1]].aggregate.start;
            if super::delimiter_crosses(&input.segment_breaks, before)
                || super::delimiter_crosses(&input.segment_breaks, after)
            {
                return Err(invalid(SourceProcessingError::DelimiterCrossing));
            }
            if row.kind == ParagraphConnectionKind::Disconnected {
                context.barriers.push(ProcessingBarrier {
                    at: after,
                    domain: capture.domain,
                    order,
                    plaintext_reset: false,
                    atomic: None,
                    scope: None,
                });
            }
            native_connections.push(ParagraphConnection {
                at: after,
                kind: row.kind,
            });
            ends.push((before, after));
        }
        let mut policies = Vec::new();
        for (policy, row) in input.boundary_policies.iter().enumerate() {
            let definition = match row.definition {
                ParagraphBoundaryDefinition::Connection(global) => capture
                    .connections
                    .iter()
                    .position(|i| *i == global)
                    .map(ParagraphBoundaryDefinition::Connection),
                ParagraphBoundaryDefinition::AtomicBefore(i)
                | ParagraphBoundaryDefinition::AtomicAfter(i) => capture
                    .anchors
                    .iter()
                    .any(|a| a.item == ParagraphAnchorItem::Atomic(i))
                    .then_some(row.definition),
            };
            if let Some(definition) = definition {
                if used_policies[policy] {
                    return Err(invalid(SourceProcessingError::BoundaryPolicy));
                }
                used_policies[policy] = true;
                policies.push(ParagraphBoundaryPolicy { definition, ..*row });
            }
        }
        let mut replacements = Vec::new();
        for (i, row) in input.replacements.iter().enumerate() {
            let definition = match row.definition {
                ParagraphReplacementDefinition::Source { origin, source } => capture
                    .origins
                    .iter()
                    .position(|i| *i == origin)
                    .map(|origin| ParagraphReplacementDefinition::Source { origin, source }),
                ParagraphReplacementDefinition::Control { control, origin } => capture
                    .controls
                    .iter()
                    .position(|i| *i == control)
                    .and_then(|control| match origin {
                        None => Some(ParagraphReplacementDefinition::Control {
                            control,
                            origin: None,
                        }),
                        Some(global) => {
                            capture
                                .origins
                                .iter()
                                .position(|i| *i == global)
                                .map(|origin| ParagraphReplacementDefinition::Control {
                                    control,
                                    origin: Some(origin),
                                })
                        }
                    }),
            };
            if let Some(definition) = definition {
                used_replacements[i] = true;
                replacements.push(ParagraphReplacement {
                    definition,
                    ..row.clone()
                });
            }
        }
        let coverage: Vec<_> = capture
            .origins
            .iter()
            .map(|i| input.origins[*i].aggregate)
            .collect();
        // Structural traversal through a child domain transfers no ownership.
        // Include original ranges before processing and every genuine zero-width
        // occurrence, retaining the entry point only for a truly itemless view.
        let mut owned = coverage.clone();
        owned.extend(capture.anchors.iter().map(|anchor| {
            let at = match anchor.item {
                ParagraphAnchorItem::EmptyOrigin(local) => {
                    input.origins[capture.origins[local]].aggregate.start
                }
                ParagraphAnchorItem::Atomic(atomic) => input.source.boxes()[atomic].index(),
                ParagraphAnchorItem::Control(control) => {
                    input.controls[capture.controls[control]].at
                }
            };
            Range::new(at, at)
        }));
        let extent = Range::new(
            owned
                .iter()
                .map(|range| range.start)
                .min()
                .unwrap_or(capture.start),
            owned
                .iter()
                .map(|range| range.end)
                .max()
                .unwrap_or(capture.start),
        );
        let data = ParagraphData {
            origins: capture
                .origins
                .iter()
                .map(|i| {
                    let row = &input.origins[*i];
                    OriginData {
                        source: OriginSource::Captured(row.source.clone()),
                        local: row.local,
                        aggregate: row.aggregate,
                        kind: row.kind,
                    }
                })
                .collect(),
            connections: native_connections,
            connection_ends: Some(ends),
            controls: capture
                .controls
                .iter()
                .map(|i| input.controls[*i].clone())
                .collect(),
            boundary_policies: policies,
            replacements,
            anchor_order: capture.anchors,
            base_direction: Some(input.paragraphs[index].base_direction),
            paragraph_strut: input.paragraphs[index].paragraph_strut,
            coverage: Some(coverage),
            envelope: Some(extent),
            domain_style: Some(context.domains[capture.domain].style.clone()),
        };
        for replacement in &data.replacements {
            data.admit(&input.source, replacement.replaced_source())?;
        }
        validate_facts(&input.source, &data)?;
        records.push(Some(ParagraphRecord { data }));
    }
    if used_policies.iter().any(|used| !used) {
        return Err(invalid(SourceProcessingError::BoundaryPolicy));
    }
    if used_replacements.iter().any(|used| !used) {
        return Err(invalid(SourceProcessingError::Replacement));
    }
    let box_order = atomic_order;
    let records = paragraph_order
        .into_iter()
        .map(|index| {
            records[index]
                .take()
                .expect("admitted exactly-once paragraph")
        })
        .collect();
    super::validate_crossings(&input.source, &input.segment_breaks, &box_order)?;
    let (units, map, plaintext_tokens) =
        super::lexical::execute(super::lexical::ValidatedProcessingRequest::Contextual {
            source: &input.source,
            breaks: &input.segment_breaks,
            context: &context,
        })?;
    Ok(ProcessedSource {
        source: input.source,
        base_style: context.domains[0].style.clone(),
        segment_breaks: input.segment_breaks,
        processed: crate::Text::from_code_units(units),
        map,
        box_order,
        paragraphs: records,
        plaintext_tokens,
    })
}

fn validate_facts(source: &Source, data: &ParagraphData) -> Result<()> {
    // Reuse native fact validators on the same original owner. Contextual
    // coverage/order were admitted above, so only owner-local facts run here.
    crate::protocol::paragraph::validate_contextual_facts(source, data)
}
