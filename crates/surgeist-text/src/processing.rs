use std::collections::BTreeSet;

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

#[derive(Debug)]
struct MappingPiece {
    source: Range,
    processed: usize,
    retained: bool,
}

impl MappingPiece {
    fn processed_end(&self) -> usize {
        self.processed + if self.retained { self.source.len() } else { 0 }
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

/// Executes source-only `Discard` and captures one complete immutable result.
///
/// Intrinsic source and document-delimiter validation precedes mode admission.
/// Only nonempty effective Discard intervals are supported. Unused font/style
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
/// let plaintext: Vec<u16> = processed.plaintext_items().flat_map(|item| match item {
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
    validate_modes(&source, &base_style)?;

    let mut units = Vec::new();
    let mut map: Vec<MappingPiece> = Vec::new();
    let mut delimiter = 0;
    for (index, &unit) in source.text().code_units().iter().enumerate() {
        let in_delimiter = match &segment_breaks {
            DocumentSegmentBreaks::LineFeedDefault => unit == 0x000a,
            DocumentSegmentBreaks::Defined(ranges) => {
                while ranges
                    .get(delimiter)
                    .is_some_and(|range| range.end <= index)
                {
                    delimiter = delimiter.checked_add(1).ok_or_else(overflow)?;
                }
                ranges
                    .get(delimiter)
                    .is_some_and(|range| range.contains(index))
            }
        };
        let retained = !matches!(unit, 0x0020 | 0x0009 | 0x000d) && !in_delimiter;
        let end = index.checked_add(1).ok_or_else(overflow)?;
        if let Some(last) = map.last_mut().filter(|piece| piece.retained == retained) {
            last.source.end = end;
        } else {
            reserve(&mut map, 1)?;
            map.push(MappingPiece {
                source: Range::new(index, end),
                processed: units.len(),
                retained,
            });
        }
        if retained {
            reserve(&mut units, 1)?;
            units.push(unit);
        }
    }
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
    /// Counts retained original units strictly before the supplied endpoint.
    pub fn source_to_processed(&self, at: SourcePosition) -> Result<ProcessedPosition> {
        crate::range::validate_index(self.source.text(), at.get(), "source position")?;
        Ok(ProcessedPosition(self.project(at.get())))
    }
    /// Returns the complete preimage's earliest or latest original endpoint.
    ///
    /// Removed interiors do not uniquely round-trip. Numeric values are checked
    /// against this result, without claiming cross-result or stale identity.
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
            if piece.retained {
                piece.source.start + (at.0 - piece.processed)
            } else {
                match affinity {
                    Affinity::Before => piece.source.start,
                    Affinity::After => piece.source.end,
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
                range: processed_range(
                    self.project(original.range().start),
                    self.project(original.range().end),
                ),
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
    pub fn plaintext_items(&self) -> PlaintextItems<'_> {
        PlaintextItems {
            source: self,
            next_box: 0,
            text_cursor: 0,
        }
    }

    fn project(&self, at: usize) -> usize {
        let next = self.map.partition_point(|piece| piece.source.end <= at);
        self.map.get(next).map_or(self.processed.len(), |piece| {
            piece.processed
                + if piece.retained {
                    at - piece.source.start
                } else {
                    0
                }
        })
    }
    fn box_view<'a>(&self, ordinal: usize, original: &'a InlineBox) -> ProcessedInlineBox<'a> {
        ProcessedInlineBox {
            ordinal,
            original,
            position: ProcessedPosition(self.project(original.index())),
        }
    }
}

/// Borrowed original span and its projected range.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedSpan<'a> {
    ordinal: usize,
    original: &'a Span,
    range: ProcessedRange,
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
    /// Projected range, possibly empty after deletion.
    #[must_use]
    pub const fn range(self) -> ProcessedRange {
        self.range
    }
}

/// Borrowed original box and its projected anchor.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedInlineBox<'a> {
    ordinal: usize,
    original: &'a InlineBox,
    position: ProcessedPosition,
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
    /// Projected numeric anchor; distinct original anchors may coincide here.
    #[must_use]
    pub const fn position(self) -> ProcessedPosition {
        self.position
    }
}

/// One borrowed slice of the same processed text.
#[derive(Clone, Copy, Debug)]
pub struct ProcessedText<'a> {
    units: &'a [u16],
    range: ProcessedRange,
}
impl<'a> ProcessedText<'a> {
    /// Exact processed units without repair or inserted content.
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
    next_box: usize,
    text_cursor: usize,
}
impl<'a> Iterator for PlaintextItems<'a> {
    type Item = PlaintextItem<'a>;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(&ordinal) = self.source.box_order.get(self.next_box) {
            let original = &self.source.source.boxes()[ordinal];
            let at = self.source.project(original.index());
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

fn validate_modes(source: &Source, base: &Style) -> Result<()> {
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
    let mut pending: Option<(Range, Option<usize>, WhiteSpace)> = None;
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
        if let Some((range, _, _)) =
            pending
                .as_mut()
                .filter(|(_, previous_origin, previous_mode)| {
                    *previous_origin == origin && *previous_mode == mode
                })
        {
            range.end = end;
        } else {
            if let Some((range, span, mode)) = pending {
                admit_mode(range, span, mode)?;
            }
            pending = Some((Range::new(at, end), origin, mode));
        }
        at = end;
    }
    if let Some((range, span, mode)) = pending {
        admit_mode(range, span, mode)?;
    }
    Ok(())
}

fn admit_mode(range: Range, span: Option<usize>, mode: WhiteSpace) -> Result<()> {
    if mode != WhiteSpace::Discard {
        return Err(Error::new(
            ErrorCode::UnsupportedFeature,
            "unsupported effective source-processing mode",
        )
        .with_detail(ErrorDetail::UnsupportedProcessingMode { range, mode, span }));
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
