use std::collections::BTreeSet;

mod lexical;

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
enum MappingKind {
    Copy,
    Delete(DeletionCause),
    Replace,
}

/// Executed lexical reason, retained for later semantic owners without replay.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DeletionCause {
    Discard,
    SpaceAroundSegmentBreak,
    DuplicateSegmentBreak,
    ContextualSegmentBreak,
    CollapsedSpace,
}

#[derive(Debug)]
struct MappingPiece {
    source: Range,
    processed: usize,
    kind: MappingKind,
}

impl MappingPiece {
    fn processed_end(&self) -> usize {
        self.processed
            + match self.kind {
                MappingKind::Copy => self.source.len(),
                MappingKind::Delete(_) => 0,
                MappingKind::Replace => 1,
            }
    }
}

/// Immutable captured source and its exact executed whitespace correspondence.
///
/// This source-local phase performs no fonts, shaping, opportunity analysis,
/// trimming, line selection or complete document serialization. Actual in-flow
/// boxes remain adjacency barriers independently of equal projected offsets.
#[derive(Debug)]
pub struct ProcessedSource {
    source: Source,
    base_style: Style,
    segment_breaks: DocumentSegmentBreaks,
    processed: Text,
    map: Vec<MappingPiece>,
    box_order: Vec<usize>,
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
    crate::source_model::validate_structure(&source)?;
    validate_delimiters(&source, &segment_breaks)?;
    let mut box_order = Vec::new();
    reserve(&mut box_order, source.boxes().len())?;
    box_order.extend(0..source.boxes().len());
    box_order.sort_unstable_by_key(|&ordinal| (source.boxes()[ordinal].index(), ordinal));
    validate_crossings(&source, &segment_breaks, &box_order)?;
    let modes = effective_modes(&source, &base_style)?;
    validate_modes(&modes, &segment_breaks)?;

    let (units, map) = lexical::execute(&source, &segment_breaks, &modes, &box_order)?;
    Ok(ProcessedSource {
        source,
        base_style,
        segment_breaks,
        processed: Text::from_code_units(units),
        map,
        box_order,
    })
}

impl ProcessedSource {
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
    /// neither creates an adjacency barrier. Text slices partition the same
    /// processed backing exactly once under either choice.
    pub fn plaintext_items(&self, affinity: Affinity) -> PlaintextItems<'_> {
        PlaintextItems {
            source: self,
            affinity,
            next_box: 0,
            text_cursor: 0,
        }
    }

    fn project(&self, at: usize, affinity: Affinity) -> usize {
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
}
impl<'a> Iterator for PlaintextItems<'a> {
    type Item = PlaintextItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&ordinal) = self.source.box_order.get(self.next_box) {
            let original = &self.source.source.boxes()[ordinal];
            let at = self.source.project(original.index(), self.affinity);
            if self.text_cursor < at {
                return Some(self.text_to(at));
            }
            self.next_box += 1;
            return Some(PlaintextItem::InlineBox(
                self.source.box_view(ordinal, original),
            ));
        }
        (self.text_cursor < self.source.processed.len())
            .then(|| self.text_to(self.source.processed.len()))
    }
}
impl<'a> PlaintextItems<'a> {
    fn text_to(&mut self, end: usize) -> PlaintextItem<'a> {
        let start = self.text_cursor;
        self.text_cursor = end;
        PlaintextItem::Text(ProcessedText {
            units: &self.source.processed.code_units()[start..end],
            range: processed_range(start, end),
        })
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
