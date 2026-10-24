use std::collections::BTreeSet;

pub(crate) mod contextual;
mod lexical;
pub use contextual::*;

use crate::{
    Affinity, DocumentSegmentBreakError, Error, ErrorCode, ErrorDetail, InlineBox, InlineBoxKind,
    ProcessingCapacityError, Range, Result, Source, SourcePosition, Span, Style, Text, WhiteSpace,
};

/// Actual document-language delimiter classification in original source units.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentSegmentBreaks {
    /// Explicit fallback: every original LF is a segment break.
    LineFeedDefault,
    /// Supplied ordered delimiter occurrences. Empty means no segment breaks.
    Defined(Vec<Range>),
}

/// A processed UTF-16 endpoint, bounds-checked relative to a result.
///
/// This distinguishes coordinate domains, not global preparation identity.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ProcessedPosition(usize);

impl ProcessedPosition {
    /// Code-unit offset in processed text.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }
}

/// An ordered range in the processed coordinate domain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProcessedRange {
    start: ProcessedPosition,
    end: ProcessedPosition,
}

impl ProcessedRange {
    /// Processed start endpoint.
    #[must_use]
    pub const fn start(self) -> ProcessedPosition {
        self.start
    }
    /// Processed end endpoint.
    #[must_use]
    pub const fn end(self) -> ProcessedPosition {
        self.end
    }
    /// Number of processed code units.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end.0 - self.start.0
    }
    /// Whether both projected endpoints coincide.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start.0 == self.end.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MappingKind {
    Copy,
    Delete(DeletionCause),
    Replace,
}

/// Executed lexical reason, retained for later semantic owners without replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeletionCause {
    Discard,
    SpaceAroundSegmentBreak,
    DuplicateSegmentBreak,
    ContextualSegmentBreak,
    CollapsedSpace,
    Trim,
}

#[derive(Debug)]
pub(crate) struct MappingPiece {
    pub(crate) source: Range,
    pub(crate) processed: usize,
    pub(crate) kind: MappingKind,
}

impl MappingPiece {
    pub(crate) fn processed_end(&self) -> usize {
        self.processed
            + match self.kind {
                MappingKind::Copy => self.source.len(),
                MappingKind::Delete(_) => 0,
                MappingKind::Replace => 1,
            }
    }
}

/// Token facts survive mapping coalescence. In particular PreserveSpaces tab
/// output and a collapsed delimiter's output can both be U+0020 without having
/// the same lexical eligibility or line-context effect.
#[derive(Debug)]
pub(super) struct PlaintextToken {
    pub(super) processed: Range,
    pub(super) domain: usize,
    pub(super) collapsible: bool,
    pub(super) reset: bool,
    pub(super) transparent: bool,
}

/// Immutable captured source and its exact executed whitespace correspondence.
///
/// The source-local front executes Phase I; the contextual front first trims
/// actual scopes and freezes issued paragraph coverage. Neither performs fonts,
/// shaping, opportunity analysis, line selection or document serialization.
/// Actual in-flow boxes remain barriers independently of equal mapped offsets.
#[derive(Debug)]
pub struct ProcessedSource {
    source: Source,
    base_style: Style,
    segment_breaks: DocumentSegmentBreaks,
    processed: Text,
    map: Vec<MappingPiece>,
    box_order: Vec<usize>,
    pub(crate) paragraphs: Vec<contextual::ParagraphRecord>,
    plaintext_tokens: Vec<PlaintextToken>,
}

/// Executes source-local Collapse, Discard, Preserve or PreserveSpaces.
///
/// Intrinsic source and document-delimiter validation precedes execution.
/// Collapse uses actual Unicode context and preserves lexical deletion causes;
/// it does not perform selected-line trimming or opportunity analysis. Unused font/style
/// fields are retained without validating them for a later semantic owner.
/// No partial result escapes. Checked capacities and fallible vector reserves
/// do not imply a physical allocator/OOM guarantee or a successful-work quota.
///
/// ```
/// use surgeist_text::{DocumentSegmentBreaks, PlaintextItem, Source, Style,
///     WhiteSpace, process_source_whitespace};
/// let processed = process_source_whitespace(
///     Source::new(" A\tB\n"),
///     Style { white_space: WhiteSpace::Discard, ..Style::default() },
///     DocumentSegmentBreaks::LineFeedDefault,
/// )?;
/// let plaintext: Vec<u16> = processed.plaintext_items(surgeist_text::Affinity::Before).flat_map(|item| match item {
///     PlaintextItem::Text(text) => text.code_units().iter().copied(),
///     PlaintextItem::InlineBox(_) => [].iter().copied(),
/// }).collect();
/// assert_eq!(plaintext, [u16::from(b'A'), u16::from(b'B')]);
/// # Ok::<(), surgeist_text::Error>(())
/// ```
pub fn process_source_whitespace(
    source: Source,
    base_style: Style,
    segment_breaks: DocumentSegmentBreaks,
) -> Result<ProcessedSource> {
    process_with_boundaries(source, base_style, segment_breaks, &[])
}

pub(crate) fn process_paragraph(
    source: Source,
    base_style: Style,
    segment_breaks: DocumentSegmentBreaks,
    paragraph: &crate::protocol::paragraph::ParagraphData,
) -> Result<ProcessedSource> {
    let mut barriers = Vec::new();
    for (connection, row) in paragraph.connections.iter().enumerate() {
        if row.kind != crate::protocol::paragraph::ParagraphConnectionKind::Disconnected {
            continue;
        }
        let at = row.at;
        if delimiter_crosses(&segment_breaks, at) {
            return Err(crate::protocol::protocol_error(
                crate::protocol::ProtocolError::InvalidConnection { connection, at },
            ));
        }
        barriers.push(at);
    }
    for (control, row) in paragraph.controls.iter().enumerate() {
        if row.kind != crate::protocol::paragraph::ParagraphControlKind::ForcedBreak {
            continue;
        }
        let at = row.at;
        if delimiter_crosses(&segment_breaks, at) {
            return Err(crate::protocol::protocol_error(
                crate::protocol::ProtocolError::InvalidControl { control, at },
            ));
        }
        barriers.push(at);
    }
    barriers.sort_unstable();
    barriers.dedup();
    process_with_boundaries(source, base_style, segment_breaks, &barriers)
}

fn delimiter_crosses(breaks: &DocumentSegmentBreaks, at: usize) -> bool {
    let DocumentSegmentBreaks::Defined(delimiters) = breaks else {
        return false;
    };
    let next = delimiters.partition_point(|range| range.end <= at);
    delimiters
        .get(next)
        .is_some_and(|range| range.start < at && at < range.end)
}

fn process_with_boundaries(
    source: Source,
    base_style: Style,
    segment_breaks: DocumentSegmentBreaks,
    barriers: &[usize],
) -> Result<ProcessedSource> {
    crate::source_model::validate_structure(&source)?;
    validate_delimiters(&source, &segment_breaks)?;
    let mut box_order = Vec::new();
    reserve(&mut box_order, source.boxes().len())?;
    box_order.extend(0..source.boxes().len());
    box_order.sort_unstable_by_key(|&ordinal| (source.boxes()[ordinal].index(), ordinal));
    validate_crossings(&source, &segment_breaks, &box_order)?;
    let modes = effective_modes(&source, &base_style)?;
    validate_modes(&modes, &segment_breaks)?;

    let (units, map, _) = lexical::execute(lexical::ValidatedProcessingRequest::SourceLocal {
        source: &source,
        breaks: &segment_breaks,
        modes: &modes,
        box_order: &box_order,
        barriers,
    })?;
    Ok(ProcessedSource {
        source,
        base_style,
        segment_breaks,
        processed: Text::from_code_units(units),
        map,
        box_order,
        paragraphs: Vec::new(),
        plaintext_tokens: Vec::new(),
    })
}

/// The existing standalone front's actual untransformed correspondence.
pub(crate) fn identity(source: Source, base_style: Style) -> Result<ProcessedSource> {
    crate::source_model::validate_structure(&source)?;
    let mut box_order = Vec::new();
    reserve(&mut box_order, source.boxes().len())?;
    box_order.extend(0..source.boxes().len());
    box_order.sort_unstable_by_key(|&ordinal| (source.boxes()[ordinal].index(), ordinal));
    let map = if source.text().is_empty() {
        Vec::new()
    } else {
        vec![MappingPiece {
            source: Range::new(0, source.text().len()),
            processed: 0,
            kind: MappingKind::Copy,
        }]
    };
    Ok(ProcessedSource {
        processed: source.text().clone(),
        source,
        base_style,
        segment_breaks: DocumentSegmentBreaks::Defined(Vec::new()),
        map,
        box_order,
        paragraphs: Vec::new(),
        plaintext_tokens: Vec::new(),
    })
}

impl ProcessedSource {
    pub(crate) fn operations(&self) -> &[MappingPiece] {
        &self.map
    }
    /// The exact original source, styles, boxes, ID and revision.
    #[must_use]
    pub const fn source(&self) -> &Source {
        &self.source
    }
    /// The actual captured base style; unused fields remain untouched.
    #[must_use]
    pub const fn base_style(&self) -> &Style {
        &self.base_style
    }
    /// Admitted original document delimiter facts.
    #[must_use]
    pub const fn segment_breaks(&self) -> &DocumentSegmentBreaks {
        &self.segment_breaks
    }
    /// The sole exact processed code-unit backing.
    #[must_use]
    pub const fn processed_text(&self) -> &Text {
        &self.processed
    }
    /// Checks a processed endpoint relative to this result.
    pub fn position(&self, at: usize) -> Result<ProcessedPosition> {
        self.range(at, at).map(|range| range.start)
    }
    /// Checks ordered processed bounds relative to this result.
    pub fn range(&self, start: usize, end: usize) -> Result<ProcessedRange> {
        if start > end || end > self.processed.len() {
            return Err(
                Error::new(ErrorCode::InvalidRange, "invalid processed bounds").with_detail(
                    ErrorDetail::InvalidProcessedRange {
                        start,
                        end,
                        text_len: self.processed.len(),
                    },
                ),
            );
        }
        Ok(processed_range(start, end))
    }
    /// Projects an original endpoint with explicit many-to-one affinity.
    ///
    /// Copy is linear; Delete maps every endpoint to one position. Replace maps
    /// its exact endpoints to output endpoints, and a strict original interior
    /// to output start Before / output end After. Original precision is retained
    /// independently for extraction/editing. This is a query policy, not a new
    /// original position or a CSS event.
    pub fn source_to_processed(
        &self,
        at: SourcePosition,
        affinity: Affinity,
    ) -> Result<ProcessedPosition> {
        crate::range::validate_index(self.source.text(), at.get(), "source position")?;
        Ok(ProcessedPosition(self.project(at.get(), affinity)))
    }
    /// Selects the minimum Before / maximum After incident original boundary.
    ///
    /// Copy is linear, replacement edges map to their original exact endpoints,
    /// and a deletion contributes its entire original endpoint interval. This is
    /// not the inverse fiber of either forward affinity: contraction interiors
    /// project outward but are not recovered at replacement edges. For pure
    /// deletion it remains the complete-preimage extrema law. Numeric bounds are
    /// result-relative, without cross-result or stale-identity authority.
    pub fn processed_to_source(
        &self,
        at: ProcessedPosition,
        affinity: Affinity,
    ) -> Result<SourcePosition> {
        self.position(at.0)?;
        let source = if self.map.is_empty() {
            0
        } else {
            let piece = match affinity {
                Affinity::Before => {
                    let next = self
                        .map
                        .partition_point(|piece| piece.processed_end() < at.0);
                    &self.map[next]
                }
                Affinity::After => {
                    let next = self.map.partition_point(|piece| piece.processed <= at.0);
                    &self.map[next - 1]
                }
            };
            match piece.kind {
                MappingKind::Copy => piece.source.start + (at.0 - piece.processed),
                MappingKind::Delete(_) => match affinity {
                    Affinity::Before => piece.source.start,
                    Affinity::After => piece.source.end,
                },
                MappingKind::Replace => {
                    if at.0 == piece.processed {
                        piece.source.start
                    } else {
                        piece.source.end
                    }
                }
            }
        };
        Ok(SourcePosition::from_unchecked(source))
    }
    /// Borrows every original span in original vector order, including empty/deleted spans.
    pub fn spans(&self) -> impl ExactSizeIterator<Item = ProcessedSpan<'_>> {
        self.source
            .spans()
            .iter()
            .enumerate()
            .map(|(ordinal, original)| ProcessedSpan {
                ordinal,
                original,
                source: self,
            })
    }
    /// Borrows every original box in original vector order.
    pub fn inline_boxes(&self) -> impl ExactSizeIterator<Item = ProcessedInlineBox<'_>> {
        self.source
            .boxes()
            .iter()
            .enumerate()
            .map(|(ordinal, original)| self.box_view(ordinal, original))
    }
    /// Visits same-result text slices and actual boxes in original logical anchor order.
    ///
    /// Slicing at an OutOfFlow occurrence is presentation only, never a new
    /// adjacency/grapheme/opportunity context. No separator or alternate content
    /// is inserted, and this partial query does not serialize a whole document.
    /// OutOfFlow anchors inside a contraction appear before its output unit for
    /// Before and after it for After. Both retain the exact original anchor;
    /// neither creates an adjacency barrier. Contextual plaintext additionally
    /// omits leading collapsible tokens at actual block/forced-break starts,
    /// using retained lexical eligibility. These extra deletions leave the
    /// processed backing and layout correspondence unchanged. No block separator,
    /// atomic alternate content or presentation replacement is serialized.
    pub fn plaintext_items(&self, affinity: Affinity) -> PlaintextItems<'_> {
        PlaintextItems {
            source: self,
            affinity,
            next_box: 0,
            text_cursor: 0,
            deletions: self.plaintext_deletions(),
        }
    }

    /// Projects already normalized exact original coverage through this
    /// owner's executed Phase-I and plaintext decisions. Selection does not
    /// repeat trimming/collapse or replace holes with an enclosing range.
    pub(crate) fn plaintext_for_coverage(&self, coverage: &[Range]) -> Result<Text> {
        if coverage
            .iter()
            .any(|range| range.start > range.end || range.end > self.source.text().len())
            || coverage.windows(2).any(|pair| pair[0].end > pair[1].start)
        {
            return Err(Error::new(
                ErrorCode::InvalidRange,
                "invalid normalized plaintext coverage",
            ));
        }
        let mut selected: Vec<Range> = Vec::new();
        for piece in &self.map {
            for range in coverage
                .iter()
                .filter(|range| range.start < piece.source.end && piece.source.start < range.end)
            {
                let start = piece.source.start.max(range.start);
                let end = piece.source.end.min(range.end);
                let mapped = match piece.kind {
                    MappingKind::Copy => Range::new(
                        piece.processed + start - piece.source.start,
                        piece.processed + end - piece.source.start,
                    ),
                    MappingKind::Replace => Range::new(piece.processed, piece.processed + 1),
                    MappingKind::Delete(_) => continue,
                };
                if let Some(previous) = selected
                    .last_mut()
                    .filter(|previous| previous.end >= mapped.start)
                {
                    previous.end = previous.end.max(mapped.end);
                } else {
                    reserve(&mut selected, 1)?;
                    selected.push(mapped);
                }
            }
        }
        let mut units = Vec::new();
        for item in self.plaintext_items(Affinity::Before) {
            let PlaintextItem::Text(text) = item else {
                continue;
            };
            let visible = Range::new(text.range.start.0, text.range.end.0);
            for range in selected
                .iter()
                .filter(|range| range.start < visible.end && visible.start < range.end)
            {
                let start = visible.start.max(range.start);
                let end = visible.end.min(range.end);
                reserve(&mut units, end - start)?;
                units.extend_from_slice(&self.processed.code_units()[start..end]);
            }
        }
        Ok(Text::from_code_units(units))
    }

    fn plaintext_deletions(&self) -> Vec<Range> {
        let mut leading = std::collections::BTreeMap::new();
        let mut deleted = Vec::new();
        for token in &self.plaintext_tokens {
            let start = leading.entry(token.domain).or_insert(true);
            if token.reset {
                *start = true;
            }
            if token.transparent {
                continue;
            }
            if token.processed.is_empty() {
                *start = false;
                continue;
            }
            if *start && token.collapsible {
                deleted.push(token.processed);
            } else if !token.reset {
                *start = false;
            }
        }
        deleted
    }

    pub(crate) fn project(&self, at: usize, affinity: Affinity) -> usize {
        let next = self.map.partition_point(|piece| piece.source.end <= at);
        self.map.get(next).map_or(self.processed.len(), |piece| {
            piece.processed
                + match piece.kind {
                    MappingKind::Copy => at - piece.source.start,
                    MappingKind::Delete(_) => 0,
                    MappingKind::Replace => {
                        usize::from(at > piece.source.start && affinity == Affinity::After)
                    }
                }
        })
    }
    fn box_view<'a>(&'a self, ordinal: usize, original: &'a InlineBox) -> ProcessedInlineBox<'a> {
        ProcessedInlineBox {
            ordinal,
            original,
            source: self,
        }
    }
}

/// Borrowed original span and its projected range.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedSpan<'a> {
    ordinal: usize,
    original: &'a Span,
    source: &'a ProcessedSource,
}
impl<'a> ProcessedSpan<'a> {
    /// Original span-vector ordinal.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }
    /// Exact original range and style record.
    #[must_use]
    pub const fn original(self) -> &'a Span {
        self.original
    }
    /// Projects both original endpoints with the same chosen affinity.
    ///
    /// This ordered numeric projection is not unique normalized-unit style/font
    /// ownership or conservative painted coverage. Empty spans stay collapsed.
    #[must_use]
    pub fn range(self, affinity: Affinity) -> ProcessedRange {
        processed_range(
            self.source.project(self.original.range().start, affinity),
            self.source.project(self.original.range().end, affinity),
        )
    }
}

/// Borrowed original box and its projected anchor.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedInlineBox<'a> {
    ordinal: usize,
    original: &'a InlineBox,
    source: &'a ProcessedSource,
}
impl<'a> ProcessedInlineBox<'a> {
    /// Original box-vector ordinal, not a preparation identity.
    #[must_use]
    pub const fn ordinal(self) -> usize {
        self.ordinal
    }
    /// Actual original box, including kind, anchor and geometry.
    #[must_use]
    pub const fn original(self) -> &'a InlineBox {
        self.original
    }
    /// Projects the exact original anchor using the explicit contraction choice.
    #[must_use]
    pub fn position(self, affinity: Affinity) -> ProcessedPosition {
        ProcessedPosition(self.source.project(self.original.index(), affinity))
    }
}

/// One borrowed slice of the same processed text.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedText<'a> {
    units: &'a [u16],
    range: ProcessedRange,
}
impl<'a> ProcessedText<'a> {
    /// Exact operation output units, without scalar repair.
    #[must_use]
    pub const fn code_units(self) -> &'a [u16] {
        self.units
    }
    /// Processed-domain extent of this slice.
    #[must_use]
    pub const fn range(self) -> ProcessedRange {
        self.range
    }
}

/// A partial plaintext contribution; not a shaping or analysis event.
#[derive(Clone, Copy, Debug)]
pub enum PlaintextItem<'a> {
    /// Same-result processed text.
    Text(ProcessedText<'a>),
    /// Actual original occurrence; its original kind governs adjacency.
    InlineBox(ProcessedInlineBox<'a>),
}

/// Borrowing logical traversal of the processed text and original box occurrences.
pub struct PlaintextItems<'a> {
    source: &'a ProcessedSource,
    affinity: Affinity,
    next_box: usize,
    text_cursor: usize,
    deletions: Vec<Range>,
}
impl<'a> Iterator for PlaintextItems<'a> {
    type Item = PlaintextItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        let next_box = self.source.box_order.get(self.next_box).copied();
        let end = next_box.map_or(self.source.processed.len(), |ordinal| {
            self.source
                .project(self.source.source.boxes()[ordinal].index(), self.affinity)
        });
        if self.text_cursor < end
            && let Some(text) = self.text_to(end)
        {
            return Some(text);
        }
        if let Some(ordinal) = next_box {
            self.next_box += 1;
            return Some(PlaintextItem::InlineBox(
                self.source
                    .box_view(ordinal, &self.source.source.boxes()[ordinal]),
            ));
        }
        None
    }
}

impl<'a> PlaintextItems<'a> {
    fn text_to(&mut self, end: usize) -> Option<PlaintextItem<'a>> {
        loop {
            let next = self
                .deletions
                .iter()
                .find(|range| range.end > self.text_cursor);
            if let Some(range) = next.filter(|range| range.start <= self.text_cursor) {
                self.text_cursor = range.end.min(end);
                if self.text_cursor == end {
                    return None;
                }
                continue;
            }
            let start = self.text_cursor;
            self.text_cursor = next.map_or(end, |range| end.min(range.start));
            return (start < self.text_cursor).then(|| {
                PlaintextItem::Text(ProcessedText {
                    units: &self.source.processed.code_units()[start..self.text_cursor],
                    range: processed_range(start, self.text_cursor),
                })
            });
        }
    }
}

fn processed_range(start: usize, end: usize) -> ProcessedRange {
    ProcessedRange {
        start: ProcessedPosition(start),
        end: ProcessedPosition(end),
    }
}

fn validate_delimiters(source: &Source, breaks: &DocumentSegmentBreaks) -> Result<()> {
    let DocumentSegmentBreaks::Defined(ranges) = breaks else {
        return Ok(());
    };
    for (occurrence, &range) in ranges.iter().enumerate() {
        let reason = if range.start > range.end {
            Some(DocumentSegmentBreakError::Reversed)
        } else if range.is_empty() {
            Some(DocumentSegmentBreakError::Empty)
        } else if range.end > source.text().len() {
            Some(DocumentSegmentBreakError::OutOfBounds)
        } else if occurrence > 0 && range.start < ranges[occurrence - 1].start {
            Some(DocumentSegmentBreakError::Unordered)
        } else if occurrence > 0 && range.start < ranges[occurrence - 1].end {
            Some(DocumentSegmentBreakError::Overlap)
        } else {
            None
        };
        if let Some(reason) = reason {
            return Err(
                Error::new(ErrorCode::InvalidRange, "invalid document segment break").with_detail(
                    ErrorDetail::InvalidDocumentSegmentBreak {
                        occurrence,
                        range,
                        reason,
                    },
                ),
            );
        }
    }
    Ok(())
}

fn validate_crossings(
    source: &Source,
    breaks: &DocumentSegmentBreaks,
    order: &[usize],
) -> Result<()> {
    let DocumentSegmentBreaks::Defined(ranges) = breaks else {
        return Ok(());
    };
    let mut next = 0;
    for (occurrence, range) in ranges.iter().enumerate() {
        while let Some(&ordinal) = order.get(next) {
            let box_ = source.boxes()[ordinal];
            if box_.index() >= range.end {
                break;
            }
            next += 1;
            if range.start < box_.index() && box_.kind() == InlineBoxKind::InFlow {
                return Err(Error::new(
                    ErrorCode::UnsupportedFeature,
                    "document delimiter crosses atomic inline box",
                )
                .with_detail(ErrorDetail::SegmentBreakCrossesInlineBox {
                    occurrence,
                    box_ordinal: ordinal,
                    id: box_.id(),
                    anchor: box_.index(),
                }));
            }
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct EffectiveInterval {
    range: Range,
    span: Option<usize>,
    mode: WhiteSpace,
}

fn effective_modes(source: &Source, base: &Style) -> Result<Vec<EffectiveInterval>> {
    let mut events = Vec::new();
    reserve(
        &mut events,
        source.spans().len().checked_mul(2).ok_or_else(overflow)?,
    )?;
    for (ordinal, span) in source.spans().iter().enumerate() {
        let range = span.range();
        if !range.is_empty() {
            events.push((range.start, true, ordinal));
            events.push((range.end, false, ordinal));
        }
    }
    events.sort_unstable();
    let mut active = BTreeSet::new();
    let mut next = 0;
    let mut at = 0;
    let mut intervals: Vec<EffectiveInterval> = Vec::new();
    while at < source.text().len() {
        while events.get(next).is_some_and(|event| event.0 == at) {
            let (_, start, ordinal) = events[next];
            if start {
                active.insert(ordinal);
            } else {
                active.remove(&ordinal);
            }
            next += 1;
        }
        let end = events
            .get(next)
            .map_or(source.text().len(), |event| event.0);
        let origin = active.last().copied();
        let mode = origin.map_or(base.white_space, |ordinal| {
            source.spans()[ordinal].style().white_space
        });
        if let Some(previous) = intervals
            .last_mut()
            .filter(|previous| previous.span == origin && previous.mode == mode)
        {
            previous.range.end = end;
        } else {
            reserve(&mut intervals, 1)?;
            intervals.push(EffectiveInterval {
                range: Range::new(at, end),
                span: origin,
                mode,
            });
        }
        at = end;
    }
    Ok(intervals)
}

fn validate_modes(intervals: &[EffectiveInterval], breaks: &DocumentSegmentBreaks) -> Result<()> {
    let mut delimiter = 0;
    for (ordinal, interval) in intervals.iter().enumerate() {
        if ordinal == 0 || intervals[ordinal - 1].mode == interval.mode {
            continue;
        }
        if let DocumentSegmentBreaks::Defined(ranges) = breaks {
            let at = interval.range.start;
            while ranges.get(delimiter).is_some_and(|range| range.end <= at) {
                delimiter += 1;
            }
            if let Some(&range) = ranges
                .get(delimiter)
                .filter(|range| range.start < at && at < range.end)
            {
                return Err(Error::new(
                    ErrorCode::UnsupportedFeature,
                    "document delimiter crosses effective processing mode",
                )
                .with_detail(ErrorDetail::SegmentBreakCrossesProcessingMode {
                    occurrence: delimiter,
                    range,
                    at,
                    before: intervals[ordinal - 1].mode,
                    after: interval.mode,
                }));
            }
        }
    }
    Ok(())
}

fn reserve<T>(buffer: &mut Vec<T>, additional: usize) -> Result<()> {
    let count = buffer.len().checked_add(additional).ok_or_else(overflow)?;
    let bytes = count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or_else(overflow)?;
    if bytes > isize::MAX as usize {
        return Err(overflow());
    }
    buffer.try_reserve(additional).map_err(|_| {
        Error::new(
            ErrorCode::SourceProcessingCapacity,
            "source-processing reservation refused",
        )
        .with_detail(ErrorDetail::ProcessingCapacity {
            reason: ProcessingCapacityError::Allocation,
        })
    })
}

fn overflow() -> Error {
    Error::new(
        ErrorCode::SourceProcessingCapacity,
        "source-processing capacity overflow",
    )
    .with_detail(ErrorDetail::ProcessingCapacity {
        reason: ProcessingCapacityError::Overflow,
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod contextual_tests;
