//! Explicit native paragraph capture. No tree adjacency or CSS default is inferred.

use std::sync::Arc;

use crate::{
    Affinity, AnalyzedSource, DocumentSegmentBreaks, ProcessedSource, Range, Result, Source, Style,
    Text, Wrap,
};

use super::{ProtocolError, ProtocolHandleKind, protocol_error};

/// Paragraph direction captured before Unicode bidi analysis and preparation.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ParagraphBaseDirection {
    LeftToRight,
    RightToLeft,
}

/// Actual normalized relation of this preparation's end to its containing block.
/// Paragraph completion alone does not establish a CSS block end.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ParagraphTerminalEdge {
    Unknown,
    BlockEnd,
    NonBlockBoundary,
}

impl ParagraphBaseDirection {
    pub(crate) const fn level(self) -> u8 {
        match self {
            Self::LeftToRight => 0,
            Self::RightToLeft => 1,
        }
    }
}

/// The retained origin's coordinate domain; generated text is never a DOM range.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ParagraphOriginKind {
    Authored,
    Generated,
}

/// Actual immutable complete original Text-node content and this occurrence's
/// contributed node-relative UTF-16 range. Resource identity does not imply a node.
#[derive(Clone, Debug)]
pub struct CompleteTextNodeContribution {
    text: Text,
    range: Range,
}

impl CompleteTextNodeContribution {
    /// Captures complete original content. Interior UTF-16 endpoints are valid.
    pub fn new(text: Text, range: Range) -> Result<Self> {
        if range.start > range.end || range.end > text.len() {
            return Err(crate::Error::new(
                crate::ErrorCode::InvalidRange,
                "Text-node contribution is outside its complete original content",
            )
            .with_detail(crate::ErrorDetail::TextTransform {
                reason: crate::TextTransformError::InvalidTextNodeRange {
                    range,
                    text_len: text.len(),
                },
            }));
        }
        Ok(Self { text, range })
    }

    #[must_use]
    pub const fn text(&self) -> &Text {
        &self.text
    }
    #[must_use]
    pub const fn range(&self) -> Range {
        self.range
    }

    pub(crate) fn validate_origin(
        &self,
        source: &Source,
        local: Range,
        origin: usize,
        code: crate::ErrorCode,
    ) -> Result<()> {
        if self.range.len() != local.len()
            || self.text.code_units()[self.range.start..self.range.end]
                != source.text().code_units()[local.start..local.end]
        {
            return Err(crate::Error::new(
                code,
                "Text-node contribution differs from its original occurrence",
            )
            .with_detail(crate::ErrorDetail::TextTransform {
                reason: crate::TextTransformError::TextNodeMismatch { origin },
            }));
        }
        Ok(())
    }
}

/// One explicit captured occurrence. Repeating the same Source creates another occurrence.
#[derive(Clone, Debug)]
pub struct ParagraphOrigin {
    pub source: Source,
    pub local: Range,
    pub aggregate: Range,
    pub kind: ParagraphOriginKind,
    /// Complete original Text-node authority; absence means unknown context.
    pub text_node: Option<CompleteTextNodeContribution>,
}

/// Whether two original occurrences are genuinely text-contiguous.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ParagraphConnectionKind {
    Continuous,
    Disconnected,
}

/// The vector contains exactly one connection for each neighboring origin pair.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParagraphConnection {
    pub at: usize,
    pub kind: ParagraphConnectionKind,
}

/// An actual semantic control, not authored characters manufactured for painting.
#[derive(Clone, Debug, PartialEq)]
pub enum ParagraphControlKind {
    InlineBoundaryStart,
    InlineBoundaryEnd,
    /// An explicit paragraph separator; source BK/NL retain their own classes.
    ForcedBreak,
    /// Genuine wbr occurrence with its complete computed style.
    Wbr {
        style: Box<Style>,
    },
    OptionalBreak {
        wrap: Wrap,
    },
    /// Genuine authored discretionary control with its own complete style.
    DiscretionaryBreak {
        style: Box<Style>,
    },
    LeftToRightEmbedding,
    RightToLeftEmbedding,
    LeftToRightOverride,
    RightToLeftOverride,
    PopDirectionalFormat,
    LeftToRightIsolate,
    RightToLeftIsolate,
    FirstStrongIsolate,
    PopDirectionalIsolate,
}

impl ParagraphControlKind {
    pub(crate) const fn code_point(&self) -> Option<u32> {
        match self {
            Self::LeftToRightEmbedding => Some(0x202a),
            Self::RightToLeftEmbedding => Some(0x202b),
            Self::PopDirectionalFormat => Some(0x202c),
            Self::LeftToRightOverride => Some(0x202d),
            Self::RightToLeftOverride => Some(0x202e),
            Self::LeftToRightIsolate => Some(0x2066),
            Self::RightToLeftIsolate => Some(0x2067),
            Self::FirstStrongIsolate => Some(0x2068),
            Self::PopDirectionalIsolate => Some(0x2069),
            Self::ForcedBreak
            | Self::OptionalBreak { .. }
            | Self::Wbr { .. }
            | Self::DiscretionaryBreak { .. }
            | Self::InlineBoundaryStart
            | Self::InlineBoundaryEnd => None,
        }
    }
}

/// Same-anchor controls preserve their actual supplied vector order.
#[derive(Clone, Debug, PartialEq)]
pub struct ParagraphControl {
    pub at: usize,
    pub kind: ParagraphControlKind,
}

/// Explicit relative order of actual anchor occurrences. Numeric entries only
/// reference this raw capture and do not issue a prepared capability.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParagraphAnchorItem {
    EmptyOrigin(usize),
    Atomic(usize),
    Control(usize),
}

/// Original anchor precision and its explicit projection to an indivisible unit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParagraphAnchor {
    pub item: ParagraphAnchorItem,
    pub affinity: Affinity,
}

/// Executed semantic anchors; computed causes are not authored input controls.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SemanticAnchorItem {
    EmptyOrigin(usize),
    Atomic(usize),
    Control(usize),
    ComputedLineBoundary(usize),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SemanticAnchor {
    pub(crate) item: SemanticAnchorItem,
    pub(crate) affinity: Affinity,
}
impl From<ParagraphAnchor> for SemanticAnchor {
    fn from(value: ParagraphAnchor) -> Self {
        Self {
            item: match value.item {
                ParagraphAnchorItem::EmptyOrigin(i) => SemanticAnchorItem::EmptyOrigin(i),
                ParagraphAnchorItem::Atomic(i) => SemanticAnchorItem::Atomic(i),
                ParagraphAnchorItem::Control(i) => SemanticAnchorItem::Control(i),
            },
            affinity: value.affinity,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct ComputedBoundaryData {
    pub(crate) at: usize,
    pub(crate) capture_cut: usize,
    pub(crate) causes: Vec<usize>,
}

/// The actual occurrence whose incident boundary owns these wrapping facts.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ParagraphBoundaryDefinition {
    Connection(usize),
    AtomicBefore(usize),
    AtomicAfter(usize),
}

/// Actual captured wrapping facts. The anchor is derived from the specified
/// occurrence, not an unqualified numeric source offset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParagraphBoundaryPolicy {
    pub definition: ParagraphBoundaryDefinition,
    pub preceding_wrap: Wrap,
    pub following_wrap: Wrap,
    pub ancestor_wrap: Wrap,
}

impl ParagraphBoundaryPolicy {
    pub(crate) fn at(self, paragraph: &ParagraphData, boxes: &[crate::InlineBox]) -> usize {
        match self.definition {
            ParagraphBoundaryDefinition::Connection(i) => paragraph.connections[i].at,
            ParagraphBoundaryDefinition::AtomicBefore(i)
            | ParagraphBoundaryDefinition::AtomicAfter(i) => boxes[i].index(),
        }
    }
}

/// The authentic defining occurrence of a computed break replacement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParagraphReplacementDefinition {
    Source {
        origin: usize,
        source: Range,
    },
    Control {
        control: usize,
        origin: Option<usize>,
    },
}

/// Genuine group identity excludes a control's optional source provenance.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum ReplacementOwner {
    Source {
        origin: usize,
        start: usize,
        end: usize,
    },
    Control(usize),
}
impl ParagraphReplacementDefinition {
    pub(crate) const fn owner(self) -> ReplacementOwner {
        match self {
            Self::Source { origin, source } => ReplacementOwner::Source {
                origin,
                start: source.start,
                end: source.end,
            },
            Self::Control { control, .. } => ReplacementOwner::Control(control),
        }
    }
}

/// One genuine source/control-owned indication group.
#[derive(Clone, Debug)]
pub struct ParagraphReplacement {
    pub(crate) definition: ParagraphReplacementDefinition,
    pub(crate) insertion_anchor: usize,
    pub(crate) presentation: ReplacementPresentation,
}

#[derive(Clone, Debug)]
pub(crate) enum ReplacementPresentation {
    StyleDriven,
    ResolvedNative(Vec<Text>),
}

impl ParagraphReplacement {
    /// Resolve one actual SHY/control from its own captured Style at preparation.
    /// Analysis validates the genuine definition, anchor and unique owner.
    #[must_use]
    pub const fn style_driven(
        definition: ParagraphReplacementDefinition,
        insertion_anchor: usize,
    ) -> Self {
        Self {
            definition,
            insertion_anchor,
            presentation: ReplacementPresentation::StyleDriven,
        }
    }
    /// Capture a native producer's exact ordered logical-before alternatives.
    /// This route does not reinterpret Style's indication string or convention.
    /// Each string may be empty; the list must contain at least one choice.
    /// Analysis validates the genuine definition, anchor and unique owner.
    ///
    /// # Errors
    /// Returns `HyphenationError::EmptyNativeAlternatives` for an empty list.
    pub fn resolved_native_logical_before(
        definition: ParagraphReplacementDefinition,
        insertion_anchor: usize,
        alternatives: Vec<Text>,
    ) -> Result<Self> {
        if alternatives.is_empty() {
            return Err(crate::hyphenation::error(
                crate::HyphenationError::EmptyNativeAlternatives,
            ));
        }
        Ok(Self {
            definition,
            insertion_anchor,
            presentation: ReplacementPresentation::ResolvedNative(alternatives),
        })
    }
    /// Actual defining occurrence, before analysis issues any capability.
    #[must_use]
    pub const fn definition(&self) -> ParagraphReplacementDefinition {
        self.definition
    }
    /// Captured original insertion anchor, validated with the definition.
    #[must_use]
    pub const fn insertion_anchor(&self) -> usize {
        self.insertion_anchor
    }
    /// None denotes style-driven resolution; Some preserves exact native order.
    #[must_use]
    pub fn native_alternatives(&self) -> Option<&[Text]> {
        match &self.presentation {
            ReplacementPresentation::StyleDriven => None,
            ReplacementPresentation::ResolvedNative(text) => Some(text),
        }
    }

    pub(crate) fn replaced_source(&self) -> Range {
        match self.definition {
            ParagraphReplacementDefinition::Source { source, .. } => source,
            ParagraphReplacementDefinition::Control { .. } => {
                Range::new(self.insertion_anchor, self.insertion_anchor)
            }
        }
    }
}

/// The captured paragraph contribution to every native line's metrics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ParagraphStrut {
    DefaultFont,
    AllocatedContributions,
}

/// Complete native normalized input. Construction of analysis validates this
/// request together; public vector positions do not issue prepared capabilities.
#[derive(Clone, Debug)]
pub struct ParagraphInput {
    pub terminal_edge: ParagraphTerminalEdge,
    pub source: Source,
    pub base_style: Style,
    pub segment_breaks: DocumentSegmentBreaks,
    pub base_direction: ParagraphBaseDirection,
    pub paragraph_strut: ParagraphStrut,
    pub tab_reference: Option<crate::ParagraphTabReference>,
    pub origins: Vec<ParagraphOrigin>,
    pub connections: Vec<ParagraphConnection>,
    pub controls: Vec<ParagraphControl>,
    pub boundary_policies: Vec<ParagraphBoundaryPolicy>,
    pub replacements: Vec<ParagraphReplacement>,
    pub anchor_order: Vec<ParagraphAnchor>,
}

/// One original resource authority: Canonical refers through ProcessedSource,
/// while Captured retains a distinct original occurrence resource.
#[derive(Clone, Debug)]
pub(crate) enum OriginSource {
    Canonical,
    Captured(Source),
}

#[derive(Clone, Debug)]
pub(crate) struct OriginData {
    pub(crate) source: OriginSource,
    pub(crate) local: Range,
    pub(crate) aggregate: Range,
    pub(crate) kind: ParagraphOriginKind,
    pub(crate) text_node: Option<CompleteTextNodeContribution>,
}

impl OriginData {
    pub(crate) fn original<'a>(&'a self, processed: &'a ProcessedSource) -> &'a Source {
        match &self.source {
            OriginSource::Canonical => processed.source(),
            OriginSource::Captured(source) => source,
        }
    }
}

/// Validated occurrence/context facts retained by the actual analysis allocation.
#[derive(Clone, Debug)]
pub(crate) struct ParagraphData {
    pub(crate) scope_topology: Option<crate::processing::contextual::ParagraphTopology>,
    pub(crate) terminal_edge: ParagraphTerminalEdge,
    pub(crate) origins: Vec<OriginData>,
    pub(crate) connections: Vec<ParagraphConnection>,
    /// Real preceding/following original endpoints for contextual connections.
    pub(crate) connection_ends: Option<Vec<(usize, usize)>>,
    /// Actual contextual Connection event rank among this paragraph’s anchors.
    pub(crate) connection_anchor_cuts: Option<Vec<usize>>,
    pub(crate) controls: Vec<ParagraphControl>,
    pub(crate) boundary_policies: Vec<ParagraphBoundaryPolicy>,
    pub(crate) replacements: Vec<ParagraphReplacement>,
    pub(crate) anchor_order: Vec<SemanticAnchor>,
    pub(crate) computed_boundaries: Vec<ComputedBoundaryData>,
    /// None is the standalone front's genuine Auto policy.
    pub(crate) base_direction: Option<ParagraphBaseDirection>,
    pub(crate) paragraph_strut: ParagraphStrut,
    pub(crate) tab_reference: Option<crate::ParagraphTabReference>,
    pub(crate) coverage: Option<Vec<Range>>,
    pub(crate) envelope: Option<Range>,
    pub(crate) domain_style: Option<Style>,
}

impl ParagraphData {
    /// Genuine Wbr separates original decoding, regardless of its replacement.
    /// These positions do not disconnect lexical or typographic context.
    pub(crate) fn wbr_decode_boundaries(&self) -> impl Iterator<Item = usize> + '_ {
        self.controls.iter().filter_map(|control| {
            matches!(control.kind, ParagraphControlKind::Wbr { .. }).then_some(control.at)
        })
    }

    pub(crate) fn base_style<'a>(&'a self, processed: &'a ProcessedSource) -> &'a Style {
        self.domain_style.as_ref().unwrap_or(processed.base_style())
    }
    pub(crate) fn extent_in(&self, source: &Source) -> Range {
        self.envelope.unwrap_or(Range::new(0, source.text().len()))
    }
    pub(crate) fn extent(&self, processed: &ProcessedSource) -> Range {
        self.envelope
            .unwrap_or(Range::new(0, processed.source().text().len()))
    }
    pub(crate) fn contains(&self, source: &Source, range: Range) -> bool {
        self.coverage.as_ref().is_none_or(|pieces| {
            // A real parent occurrence owns its original point even between
            // child domains. It grants no positive text interval or fabricated
            // occurrence identity; geometric callers still qualify ambiguity.
            if range.is_empty()
                && (self
                    .envelope
                    .is_some_and(|extent| range.start == extent.start || range.start == extent.end)
                    || self.anchor_order.iter().any(|anchor| {
                        let at = match anchor.item {
                            SemanticAnchorItem::EmptyOrigin(origin) => {
                                self.origins[origin].aggregate.start
                            }
                            SemanticAnchorItem::Control(control) => self.controls[control].at,
                            SemanticAnchorItem::ComputedLineBoundary(i) => {
                                self.computed_boundaries[i].at
                            }
                            SemanticAnchorItem::Atomic(atomic) => source.boxes()[atomic].index(),
                        };
                        at == range.start
                    }))
            {
                return true;
            }
            let mut cursor = range.start;
            for piece in pieces {
                if piece.start <= cursor && cursor <= piece.end {
                    cursor = cursor.max(piece.end);
                    if cursor >= range.end {
                        return true;
                    }
                }
            }
            false
        })
    }
    pub(crate) fn admit(&self, source: &Source, range: Range) -> Result<()> {
        if self.contains(source, range) {
            Ok(())
        } else {
            Err(crate::processing::contextual::invalid(
                crate::SourceProcessingError::OutsideParagraph,
            ))
        }
    }
    pub(crate) fn intersections(&self, range: Range) -> Vec<Range> {
        match &self.coverage {
            None => vec![range],
            Some(pieces) => {
                let mut result: Vec<Range> = Vec::new();
                for piece in pieces {
                    let start = piece.start.max(range.start);
                    let end = piece.end.min(range.end);
                    if start < end {
                        if let Some(last) = result.last_mut().filter(|last| last.end == start) {
                            last.end = end;
                        } else {
                            result.push(Range::new(start, end));
                        }
                    }
                }
                result
            }
        }
    }

    pub(crate) fn standalone(source: &Source) -> Self {
        let source_len = source.text().len();
        let mut anchors: Vec<_> = source
            .boxes()
            .iter()
            .enumerate()
            .map(|(i, b)| (b.index(), i))
            .collect();
        anchors.sort_unstable();
        let mut anchor_order: Vec<_> = anchors
            .into_iter()
            .map(|(_, i)| SemanticAnchor {
                item: SemanticAnchorItem::Atomic(i),
                affinity: Affinity::After,
            })
            .collect();
        if source_len == 0 {
            anchor_order.insert(
                0,
                SemanticAnchor {
                    item: SemanticAnchorItem::EmptyOrigin(0),
                    affinity: Affinity::Before,
                },
            );
        }
        Self {
            scope_topology: None,
            computed_boundaries: Vec::new(),
            terminal_edge: ParagraphTerminalEdge::Unknown,
            origins: vec![OriginData {
                source: OriginSource::Canonical,
                local: Range::new(0, source_len),
                aggregate: Range::new(0, source_len),
                kind: ParagraphOriginKind::Authored,
                text_node: None,
            }],
            connections: Vec::new(),
            connection_ends: None,
            connection_anchor_cuts: None,
            controls: Vec::new(),
            boundary_policies: Vec::new(),
            replacements: Vec::new(),
            anchor_order,
            base_direction: None,
            paragraph_strut: ParagraphStrut::DefaultFont,
            tab_reference: None,
            coverage: None,
            envelope: None,
            domain_style: None,
        }
    }

    /// Only actual disconnected connections sever original text adjacency.
    /// In-flow box barriers are independently read from the canonical source.
    pub(crate) fn disconnections(&self) -> impl Iterator<Item = usize> + '_ {
        self.connections
            .iter()
            .enumerate()
            .filter(|(_, connection)| connection.kind == ParagraphConnectionKind::Disconnected)
            .flat_map(|(i, connection)| {
                self.connection_ends
                    .as_ref()
                    .map_or([connection.at; 2], |ends| [ends[i].0, ends[i].1])
            })
    }

    /// Candidate origin intersections in original vector order, including
    /// incident empty anchors. Selected participant coverage decides which
    /// same-offset empty occurrences belong to a particular line.
    pub(crate) fn origin_intersections<'a>(
        &'a self,
        processed: &'a ProcessedSource,
        selected: Range,
    ) -> impl Iterator<Item = OriginIntersection> + 'a {
        let first = self.origins.partition_point(|origin| {
            origin.aggregate.end < selected.start
                || (origin.aggregate.end == selected.start && !origin.aggregate.is_empty())
        });
        self.origins
            .iter()
            .enumerate()
            .skip(first)
            .take_while(move |(_, origin)| origin.aggregate.start <= selected.end)
            .filter_map(move |(index, origin)| {
                let aggregate = if origin.aggregate.is_empty() {
                    let at = origin.aggregate.start;
                    if at < selected.start || at > selected.end {
                        return None;
                    }
                    origin.aggregate
                } else {
                    let start = origin.aggregate.start.max(selected.start);
                    let end = origin.aggregate.end.min(selected.end);
                    if start >= end {
                        return None;
                    }
                    Range::new(start, end)
                };
                let local = Range::new(
                    origin.local.start + aggregate.start - origin.aggregate.start,
                    origin.local.start + aggregate.end - origin.aggregate.start,
                );
                // All endpoints come from admitted original ranges in this same
                // owner, and the correspondence preserves their bounds.
                let start = processed.project(aggregate.start, Affinity::Before);
                let end = processed.project(aggregate.end, Affinity::After);
                Some(OriginIntersection {
                    origin: index,
                    aggregate,
                    local,
                    processed: Range::new(start, end),
                })
            })
    }
}

/// Exact actual source occurrence association used by finalized fragment views.
#[derive(Clone, Copy, Debug)]
pub(crate) struct OriginIntersection {
    pub(crate) origin: usize,
    pub(crate) aggregate: Range,
    pub(crate) local: Range,
    pub(crate) processed: Range,
}

impl AnalyzedSource {
    /// Executes the captured source processing and analyzes its actual semantic
    /// input with the supplied paragraph base before font matching or shaping.
    /// Upright requires that already-resolved used base to be LTR; its separate
    /// computed direction does not replace contradictory capture authority.
    pub fn from_paragraph(
        input: ParagraphInput,
        typography: crate::AnalysisTypography,
    ) -> Result<Self> {
        validate(&input)?;
        let ParagraphInput {
            terminal_edge,
            source,
            base_style,
            segment_breaks,
            base_direction,
            paragraph_strut,
            tab_reference,
            origins,
            connections,
            controls,
            boundary_policies,
            replacements,
            anchor_order,
        } = input;
        let paragraph = ParagraphData {
            scope_topology: None,
            computed_boundaries: Vec::new(),
            terminal_edge,
            origins: origins
                .into_iter()
                .map(|origin| OriginData {
                    source: OriginSource::Captured(origin.source),
                    local: origin.local,
                    aggregate: origin.aggregate,
                    kind: origin.kind,
                    text_node: origin.text_node,
                })
                .collect(),
            connections,
            connection_ends: None,
            connection_anchor_cuts: None,
            controls,
            boundary_policies,
            replacements,
            anchor_order: anchor_order.into_iter().map(SemanticAnchor::from).collect(),
            base_direction: Some(base_direction),
            paragraph_strut,
            tab_reference,
            coverage: None,
            envelope: None,
            domain_style: None,
        };
        let processed = Arc::new(crate::processing::process_paragraph(
            source,
            base_style,
            segment_breaks,
            &paragraph,
        )?);
        Ok(Self::from_inner(crate::engine::analyze_paragraph(
            processed, paragraph, typography,
        )?))
    }
}

fn validate(input: &ParagraphInput) -> Result<()> {
    crate::source_model::validate_structure(&input.source)?;
    validate_origins(input)?;
    validate_connections(input)?;
    validate_controls(&input.source, &input.controls)?;
    validate_anchor_order(input)?;
    validate_boundaries(
        &input.source,
        &input.connections,
        |i| {
            input
                .anchor_order
                .iter()
                .any(|a| a.item == ParagraphAnchorItem::Atomic(i))
        },
        &input.boundary_policies,
    )?;
    validate_replacements(
        &input.source,
        |i| input.origins.get(i).map(|origin| origin.aggregate),
        &input.controls,
        &input.replacements,
    )
}

fn validate_anchor_order(input: &ParagraphInput) -> Result<()> {
    let mut origins = vec![false; input.origins.len()];
    let mut atomics = vec![false; input.source.boxes().len()];
    let mut controls = vec![false; input.controls.len()];
    let mut previous = 0;
    for (position, item) in input.anchor_order.iter().enumerate() {
        let (occurrence, kind, at, seen) = match item.item {
            ParagraphAnchorItem::EmptyOrigin(i) => (
                i,
                ProtocolHandleKind::Origin,
                input
                    .origins
                    .get(i)
                    .filter(|o| o.aggregate.is_empty())
                    .map(|o| o.aggregate.start),
                origins.get_mut(i),
            ),
            ParagraphAnchorItem::Atomic(i) => (
                i,
                ProtocolHandleKind::Atomic,
                input.source.boxes().get(i).map(|b| b.index()),
                atomics.get_mut(i),
            ),
            ParagraphAnchorItem::Control(i) => (
                i,
                ProtocolHandleKind::Control,
                input.controls.get(i).map(|c| c.at),
                controls.get_mut(i),
            ),
        };
        let invalid = || {
            protocol_error(ProtocolError::InvalidAnchorOrder {
                position,
                kind,
                occurrence,
            })
        };
        let (Some(at), Some(seen)) = (at, seen) else {
            return Err(invalid());
        };
        if *seen || at < previous {
            return Err(invalid());
        }
        *seen = true;
        previous = at;
    }
    let missing = origins
        .iter()
        .enumerate()
        .find(|(i, seen)| input.origins[*i].aggregate.is_empty() && !**seen)
        .map(|(i, _)| (ProtocolHandleKind::Origin, i))
        .or_else(|| {
            atomics
                .iter()
                .position(|seen| !seen)
                .map(|i| (ProtocolHandleKind::Atomic, i))
        })
        .or_else(|| {
            controls
                .iter()
                .position(|seen| !seen)
                .map(|i| (ProtocolHandleKind::Control, i))
        });
    if let Some((kind, occurrence)) = missing {
        return Err(protocol_error(ProtocolError::InvalidAnchorOrder {
            position: input.anchor_order.len(),
            kind,
            occurrence,
        }));
    }
    Ok(())
}

fn validate_origins(input: &ParagraphInput) -> Result<()> {
    let mut cursor = 0;
    for (index, origin) in input.origins.iter().enumerate() {
        let invalid = || {
            protocol_error(ProtocolError::InvalidOriginRange {
                origin: index,
                local: origin.local,
                aggregate: origin.aggregate,
            })
        };
        if origin.local.start > origin.local.end
            || origin.local.end > origin.source.text().len()
            || origin.aggregate.start > origin.aggregate.end
            || origin.aggregate.end > input.source.text().len()
            || origin.local.len() != origin.aggregate.len()
        {
            return Err(invalid());
        }
        if let Some(node) = &origin.text_node {
            node.validate_origin(
                &origin.source,
                origin.local,
                index,
                crate::ErrorCode::InvalidTextProtocol,
            )?;
        }
        if origin.aggregate.start != cursor {
            return Err(protocol_error(ProtocolError::IncompleteOriginCoverage {
                expected: cursor,
                actual: origin.aggregate.start,
            }));
        }
        let local_range: std::ops::Range<usize> = origin.local.into();
        let aggregate_range: std::ops::Range<usize> = origin.aggregate.into();
        for (offset, (&local, &aggregate)) in origin.source.text().code_units()[local_range]
            .iter()
            .zip(&input.source.text().code_units()[aggregate_range])
            .enumerate()
        {
            if local != aggregate {
                return Err(protocol_error(ProtocolError::OriginTextMismatch {
                    origin: index,
                    local: origin.local.start + offset,
                    aggregate: origin.aggregate.start + offset,
                }));
            }
        }
        cursor = origin.aggregate.end;
    }
    if cursor != input.source.text().len() {
        return Err(protocol_error(ProtocolError::IncompleteOriginCoverage {
            expected: input.source.text().len(),
            actual: cursor,
        }));
    }
    Ok(())
}

fn validate_connections(input: &ParagraphInput) -> Result<()> {
    let needed = input.origins.len().saturating_sub(1);
    for index in 0..needed {
        let Some(connection) = input.connections.get(index) else {
            return Err(protocol_error(ProtocolError::MissingConnection {
                before: index,
                after: index + 1,
            }));
        };
        let at = input.origins[index].aggregate.end;
        if connection.at != at || input.origins[index + 1].aggregate.start != at {
            return Err(protocol_error(ProtocolError::InvalidConnection {
                connection: index,
                at: connection.at,
            }));
        }
    }
    if input.connections.len() != needed {
        return Err(protocol_error(ProtocolError::InvalidConnection {
            connection: needed,
            at: input.connections[needed].at,
        }));
    }
    Ok(())
}

fn validate_controls(source: &Source, controls: &[ParagraphControl]) -> Result<()> {
    let mut previous = 0;
    for (index, control) in controls.iter().enumerate() {
        if control.at > source.text().len() || control.at < previous {
            return Err(protocol_error(ProtocolError::InvalidControl {
                control: index,
                at: control.at,
            }));
        }
        previous = control.at;
    }
    Ok(())
}

fn validate_boundaries(
    source: &Source,
    connections: &[ParagraphConnection],
    contains_atomic: impl Fn(usize) -> bool,
    policies: &[ParagraphBoundaryPolicy],
) -> Result<()> {
    let mut definitions = std::collections::BTreeSet::new();
    for (policy, row) in policies.iter().enumerate() {
        let at = match row.definition {
            ParagraphBoundaryDefinition::Connection(i) => connections.get(i).map(|c| c.at),
            ParagraphBoundaryDefinition::AtomicBefore(i)
            | ParagraphBoundaryDefinition::AtomicAfter(i) => {
                source.boxes().get(i).map(|b| b.index())
            }
        };
        if at.is_none() || !definitions.insert(row.definition) {
            return Err(protocol_error(ProtocolError::InvalidBoundaryPolicy {
                policy,
                at: at.unwrap_or(source.text().len()),
            }));
        }
    }
    for (i, connection) in connections.iter().enumerate() {
        if connection.kind == ParagraphConnectionKind::Continuous
            && !definitions.contains(&ParagraphBoundaryDefinition::Connection(i))
        {
            return Err(protocol_error(ProtocolError::InvalidBoundaryPolicy {
                policy: policies.len(),
                at: connection.at,
            }));
        }
    }
    for (i, box_) in source
        .boxes()
        .iter()
        .enumerate()
        .filter(|(i, _)| contains_atomic(*i))
    {
        if box_.kind() == crate::InlineBoxKind::InFlow {
            for definition in [
                ParagraphBoundaryDefinition::AtomicBefore(i),
                ParagraphBoundaryDefinition::AtomicAfter(i),
            ] {
                if !definitions.contains(&definition) {
                    return Err(protocol_error(ProtocolError::InvalidBoundaryPolicy {
                        policy: policies.len(),
                        at: box_.index(),
                    }));
                }
            }
        }
    }
    Ok(())
}

fn validate_replacements(
    owner_source: &Source,
    origin_range: impl Fn(usize) -> Option<Range>,
    controls: &[ParagraphControl],
    replacements: &[ParagraphReplacement],
) -> Result<()> {
    let mut owners = std::collections::BTreeMap::new();
    for (index, replacement) in replacements.iter().enumerate() {
        let range = replacement.replaced_source();
        let valid = match replacement.definition {
            ParagraphReplacementDefinition::Source { origin, source } => {
                source.start < source.end
                    && source.end - source.start == 1
                    && source.start < owner_source.text().len()
                    && owner_source.text().code_units()[source.start] == 0xad
                    && origin_range(origin).is_some_and(|origin| {
                        origin.start <= source.start && source.end <= origin.end
                    })
            }
            ParagraphReplacementDefinition::Control { control, origin } => {
                controls.get(control).is_some_and(|control| {
                    control.at == replacement.insertion_anchor
                        && matches!(
                            control.kind,
                            ParagraphControlKind::DiscretionaryBreak { .. }
                        )
                }) && origin.is_none_or(|i| {
                    origin_range(i).is_some_and(|origin| {
                        origin.start <= replacement.insertion_anchor
                            && replacement.insertion_anchor <= origin.end
                    })
                })
            }
        };
        if !valid || replacement.insertion_anchor != range.end {
            return Err(protocol_error(ProtocolError::InvalidReplacement {
                alternative: index,
                source: range,
            }));
        }
        if let Some(prior) = owners.insert(replacement.definition.owner(), index) {
            return Err(protocol_error(ProtocolError::DuplicateReplacement {
                alternative: index,
                prior,
            }));
        }
    }
    Ok(())
}

/// Actual primary fragment authority; an original association is optional for
/// source-less native controls and their computed presentation alternatives.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FragmentOwner {
    Origin(usize),
    Control(usize),
    Alternative(usize),
    VirtualSeparator(usize),
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FragmentAssociation {
    pub(crate) owner: FragmentOwner,
    pub(crate) source: Option<OriginIntersection>,
}

pub(crate) fn validate_contextual_facts(source: &Source, data: &ParagraphData) -> Result<()> {
    validate_controls(source, &data.controls)?;
    validate_boundaries(
        source,
        &data.connections,
        |i| {
            data.anchor_order
                .iter()
                .any(|a| a.item == SemanticAnchorItem::Atomic(i))
        },
        &data.boundary_policies,
    )?;
    validate_replacements(
        source,
        |i| data.origins.get(i).map(|origin| origin.aggregate),
        &data.controls,
        &data.replacements,
    )
}
