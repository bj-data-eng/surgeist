//! Indexed input for the private already-decoded parser. Literal surrogate code
//! points are retained; newline preprocessing is separate from token recovery.

use super::control::{ControlIdentity, Invariant, ParseControl, ParseStop, WorkCategory};
use crate::source::{ConsumedBoundaryAdvance, ConsumedSourceBoundary};
#[cfg(test)]
use crate::{HtmlParseCancellation, HtmlParseLimits, HtmlParseResource};
use crate::{HtmlSource, HtmlSourceId, HtmlSourceRange};
use std::mem::size_of;

/// A Unicode code point, including the surrogate range, but never EOF.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CodePoint(u32);

impl CodePoint {
    pub(crate) fn new(value: u32) -> Option<Self> {
        (value <= 0x10FFFF).then_some(Self(value))
    }

    pub(crate) fn value(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InputDiagnostic {
    Surrogate,
    Noncharacter,
    ControlCharacter,
}

/// The input item's range qualifies this record. A newline record covers both
/// supplied characters for CRLF, or the one supplied CR otherwise.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PreprocessingRecord {
    Diagnostic(InputDiagnostic),
    Newline { was_crlf: bool },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InputCharacter {
    code_point: CodePoint,
    source: HtmlSourceRange,
    preprocessing: Option<PreprocessingRecord>,
}

impl InputCharacter {
    pub(crate) fn code_point(&self) -> CodePoint {
        self.code_point
    }

    pub(crate) fn source(&self) -> &HtmlSourceRange {
        &self.source
    }

    pub(crate) fn preprocessing(&self) -> Option<PreprocessingRecord> {
        self.preprocessing
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InputRead {
    Character(InputCharacter),
    Eof(HtmlSourceRange),
}

/// A source-availability edge is neither a saved character nor EOF.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum InputPoll {
    Ready(InputRead),
    NeedInput,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AsciiMatch {
    Matched,
    Mismatched,
    NeedInput,
}

enum InputAvailability {
    Complete,
    #[cfg(test)]
    Progressive {
        through: usize,
        sealed: bool,
        boundary: ConsumedSourceBoundary,
        current_span: Option<(ConsumedSourceBoundary, ConsumedSourceBoundary)>,
    },
}

struct DecodedRaw {
    code_point: CodePoint,
    end: usize,
    boundary: Option<ConsumedSourceBoundary>,
}

enum RawPoll {
    Ready(Option<DecodedRaw>),
    NeedInput,
}

enum DecodedPoll {
    Ready {
        read: InputRead,
        _boundary: Option<ConsumedSourceBoundary>,
    },
    NeedInput,
}

impl InputRead {
    fn without_record(mut self) -> Self {
        if let Self::Character(character) = &mut self {
            character.preprocessing = None;
        }
        self
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AsciiComparison {
    Sensitive,
    Insensitive,
}

/// A fixed-size private marker. An owner retaining a variable collection of
/// markers must admit that collection; this cursor never allocates a queue.
pub(crate) struct InputCheckpoint {
    owner: ControlIdentity,
    source: HtmlSourceId,
    position: usize,
    current: Option<InputRead>,
    reconsume: bool,
}

/// The progressive marker additionally retains an actual consumed boundary.
/// Keeping it distinct leaves the existing whole-source marker and its fees
/// unchanged; neither marker owns availability or a second progress frontier.
#[cfg(test)]
pub(crate) struct ProgressiveInputCheckpoint {
    checkpoint: InputCheckpoint,
    boundary: ConsumedSourceBoundary,
    current_span: Option<(ConsumedSourceBoundary, ConsumedSourceBoundary)>,
}

pub(crate) struct InputCursor<'source> {
    owner: ControlIdentity,
    source: &'source HtmlSource,
    position: usize,
    current: Option<InputRead>,
    reconsume: bool,
    /// Preprocessing records are emitted on first committed traversal only.
    /// Lookahead does not advance this frontier; replay cannot duplicate errors.
    processed_through: usize,
    availability: InputAvailability,
    #[cfg(test)]
    cancel_after_consumed: Option<(usize, crate::HtmlParseCancellation)>,
}

impl<'source> InputCursor<'source> {
    pub(crate) fn new(
        source: &'source HtmlSource,
        control: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        control.require_source(source)?;
        control.charge(WorkCategory::CopyByte, size_of::<ControlIdentity>() as u64)?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        Ok(Self {
            owner: control.identity(),
            source,
            position: 0,
            current: None,
            reconsume: false,
            processed_through: 0,
            availability: InputAvailability::Complete,
            #[cfg(test)]
            cancel_after_consumed: None,
        })
    }

    /// Owns visibility of one original immutable source. This private producer
    /// does not itself provide tokenizer continuation or parser feeding.
    #[cfg(test)]
    pub(crate) fn progressive(
        source: &'source HtmlSource,
        control: &mut ParseControl,
    ) -> Result<Self, ParseStop> {
        control.require_source(source)?;
        control.charge(WorkCategory::CopyByte, size_of::<ControlIdentity>() as u64)?;
        control.charge(
            WorkCategory::CopyByte,
            size_of::<ConsumedSourceBoundary>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        Ok(Self {
            owner: control.identity(),
            source,
            position: 0,
            current: None,
            reconsume: false,
            processed_through: 0,
            availability: InputAvailability::Progressive {
                through: 0,
                sealed: false,
                boundary: source.initial_consumed_boundary(),
                current_span: None,
            },
            #[cfg(test)]
            cancel_after_consumed: None,
        })
    }

    /// Advances a contiguous visible prefix; it does not inspect the endpoint.
    /// Already-valid UTF-8 may be made available a byte at a time.
    #[cfg(test)]
    pub(crate) fn make_available(
        &mut self,
        through: usize,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let InputAvailability::Progressive {
            through: old,
            sealed,
            ..
        } = &mut self.availability
        else {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        };
        if *sealed || through < *old || through > self.source.len() {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        control.charge(WorkCategory::RecordWrite, 1)?;
        *old = through;
        Ok(())
    }

    /// Only full availability can be final. Exhausting an unsealed prefix is
    /// NeedInput, including a prefix containing the whole supplied source.
    #[cfg(test)]
    pub(crate) fn seal_input(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let InputAvailability::Progressive {
            through, sealed, ..
        } = &mut self.availability
        else {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        };
        if *sealed || *through != self.source.len() {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        control.charge(WorkCategory::RecordWrite, 1)?;
        *sealed = true;
        Ok(())
    }

    pub(crate) fn position(&self, control: &mut ParseControl) -> Result<usize, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.position)
    }

    pub(crate) fn peek(&self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        if self.reconsume {
            return self.saved_current(control);
        }
        Ok(self.decode_at(self.position, control)?.without_record())
    }

    pub(crate) fn consume(&mut self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        let read = if self.reconsume {
            self.saved_current(control)?
        } else {
            self.decode_at(self.position, control)?
        };
        self.commit_read(
            read,
            #[cfg(test)]
            None,
            control,
        )
    }

    pub(crate) fn poll_peek(&self, control: &mut ParseControl) -> Result<InputPoll, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.reconsume {
            return self.saved_current(control).map(InputPoll::Ready);
        }
        match self.decode_poll_at(self.position, self.boundary(), control)? {
            DecodedPoll::Ready { read, .. } => Ok(InputPoll::Ready(read.without_record())),
            DecodedPoll::NeedInput => Ok(InputPoll::NeedInput),
        }
    }

    pub(crate) fn poll_consume(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<InputPoll, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let decoded = if self.reconsume {
            DecodedPoll::Ready {
                read: self.saved_current(control)?,
                _boundary: None,
            }
        } else {
            self.decode_poll_at(self.position, self.boundary(), control)?
        };
        match decoded {
            DecodedPoll::Ready {
                read,
                #[cfg(test)]
                    _boundary: boundary,
                ..
            } => self
                .commit_read(
                    read,
                    #[cfg(test)]
                    boundary,
                    control,
                )
                .map(InputPoll::Ready),
            DecodedPoll::NeedInput => Ok(InputPoll::NeedInput),
        }
    }

    fn commit_read(
        &mut self,
        mut read: InputRead,
        #[cfg(test)] boundary: Option<ConsumedSourceBoundary>,
        control: &mut ParseControl,
    ) -> Result<InputRead, ParseStop> {
        #[cfg(test)]
        let span = if let Some(end) = boundary.as_ref() {
            control.charge(
                WorkCategory::CopyByte,
                (2 * size_of::<ConsumedSourceBoundary>()) as u64,
            )?;
            Some((
                self.boundary().expect("actual progressive start").clone(),
                end.clone(),
            ))
        } else {
            None
        };
        let end = match &read {
            InputRead::Character(character) => character.source.end(),
            InputRead::Eof(range) => range.end(),
        };
        if end <= self.processed_through {
            read = read.without_record();
        }
        control.charge(WorkCategory::CopyByte, size_of::<InputRead>() as u64)?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        #[cfg(test)]
        if span.is_some() {
            control.charge(WorkCategory::RecordWrite, 1)?;
        }
        #[cfg(test)]
        if let InputAvailability::Progressive {
            boundary: current, ..
        } = &mut self.availability
            && let Some(boundary) = boundary
        {
            *current = boundary;
        }
        #[cfg(test)]
        if let InputAvailability::Progressive { current_span, .. } = &mut self.availability {
            // A reconsume has no new proof and preserves the saved span.
            if let Some(span) = span {
                *current_span = Some(span);
            }
        }
        self.current = Some(read.clone().without_record());
        self.position = end;
        self.processed_through = self.processed_through.max(end);
        self.reconsume = false;
        Ok(read)
    }

    /// Replays the saved character/EOF, not its preprocessing emission. The
    /// tokenizer remains responsible for charging its next state dispatch.
    pub(crate) fn reconsume(&mut self, control: &mut ParseControl) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.current.is_none() || self.reconsume {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        control.charge(WorkCategory::RecordWrite, 1)?;
        self.reconsume = true;
        Ok(())
    }

    pub(crate) fn mark(&self, control: &mut ParseControl) -> Result<InputCheckpoint, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        control.charge(WorkCategory::CopyByte, size_of::<InputCheckpoint>() as u64)?;
        Ok(self.checkpoint())
    }

    #[cfg(test)]
    pub(crate) fn mark_progressive(
        &self,
        control: &mut ParseControl,
    ) -> Result<ProgressiveInputCheckpoint, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let Some(boundary) = self.boundary() else {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        };
        control.charge(
            WorkCategory::CopyByte,
            size_of::<ProgressiveInputCheckpoint>() as u64,
        )?;
        Ok(ProgressiveInputCheckpoint {
            checkpoint: self.checkpoint(),
            boundary: boundary.clone(),
            current_span: self.current_span().cloned(),
        })
    }

    fn checkpoint(&self) -> InputCheckpoint {
        InputCheckpoint {
            owner: self.owner.clone(),
            source: self.source.identity().clone(),
            position: self.position,
            current: self.current.clone(),
            reconsume: self.reconsume,
        }
    }

    pub(crate) fn rewind(
        &mut self,
        mark: &InputCheckpoint,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.require_identity(&mark.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        if mark.source != *self.source.identity() || mark.position > self.position {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        control.charge(WorkCategory::CopyByte, size_of::<InputCheckpoint>() as u64)?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        self.position = mark.position;
        self.current = mark.current.clone();
        self.reconsume = mark.reconsume;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn rewind_progressive(
        &mut self,
        mark: &ProgressiveInputCheckpoint,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.require_identity(&mark.checkpoint.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.boundary().is_none()
            || mark.checkpoint.source != *self.source.identity()
            || mark.checkpoint.position > self.position
            || mark.boundary.offset() != mark.checkpoint.position
        {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.source
            .require_consumed_boundary(&mark.boundary)
            .map_err(|_| control.fail_invariant(Invariant::InputCursorState))?;
        control.charge(
            WorkCategory::CopyByte,
            size_of::<ProgressiveInputCheckpoint>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        self.position = mark.checkpoint.position;
        self.current = mark.checkpoint.current.clone();
        self.reconsume = mark.checkpoint.reconsume;
        if let InputAvailability::Progressive { boundary, .. } = &mut self.availability {
            *boundary = mark.boundary.clone();
        }
        if let InputAvailability::Progressive { current_span, .. } = &mut self.availability {
            *current_span = mark.current_span.clone();
        }
        Ok(())
    }

    /// Non-consuming preprocessed ASCII lookahead. Each examined source unit
    /// and comparison is charged again, with a poll before every bounded read.
    pub(crate) fn matches_ascii(
        &self,
        expected: &[u8],
        comparison: AsciiComparison,
        control: &mut ParseControl,
    ) -> Result<bool, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        let mut position = self.position;
        let mut replay = self.reconsume;
        for &byte in expected {
            control.charge(WorkCategory::InspectByte, 1)?;
            if !byte.is_ascii() {
                return Err(control.fail_invariant(Invariant::InputCursorState));
            }
            let read = if replay {
                replay = false;
                self.saved_current(control)?
            } else {
                self.decode_at(position, control)?
            };
            let InputRead::Character(character) = read else {
                return Ok(false);
            };
            position = character.source.end();
            control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
            let value = character.code_point.value();
            if value > 0x7F {
                return Ok(false);
            }
            let actual = value as u8;
            let equal = match comparison {
                AsciiComparison::Sensitive => actual == byte,
                AsciiComparison::Insensitive => actual.eq_ignore_ascii_case(&byte),
            };
            if !equal {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// An available matching prefix cannot decide a mismatch. This lookahead
    /// does not commit cursor progress or preprocessing emissions.
    pub(crate) fn poll_matches_ascii(
        &self,
        expected: &[u8],
        comparison: AsciiComparison,
        control: &mut ParseControl,
    ) -> Result<AsciiMatch, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let mut position = self.position;
        let mut replay = self.reconsume;
        if self.boundary().is_some() {
            control.charge(
                WorkCategory::CopyByte,
                size_of::<ConsumedSourceBoundary>() as u64,
            )?;
        }
        let mut boundary = self.boundary().cloned();
        for expected in expected {
            control.charge(WorkCategory::InspectByte, 1)?;
            let byte = *expected;
            if !byte.is_ascii() {
                return Err(control.fail_invariant(Invariant::InputCursorState));
            }
            let read = if replay {
                replay = false;
                self.saved_current(control)?
            } else {
                match self.decode_poll_at(position, boundary.as_ref(), control)? {
                    DecodedPoll::NeedInput => return Ok(AsciiMatch::NeedInput),
                    DecodedPoll::Ready {
                        read,
                        _boundary: end,
                    } => {
                        boundary = end;
                        read
                    }
                }
            };
            let InputRead::Character(character) = read else {
                return Ok(AsciiMatch::Mismatched);
            };
            position = character.source.end();
            control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
            let value = character.code_point.value();
            if value > 0x7F {
                return Ok(AsciiMatch::Mismatched);
            }
            let actual = value as u8;
            let equal = match comparison {
                AsciiComparison::Sensitive => actual == byte,
                AsciiComparison::Insensitive => actual.eq_ignore_ascii_case(&byte),
            };
            if !equal {
                return Ok(AsciiMatch::Mismatched);
            }
        }
        Ok(AsciiMatch::Matched)
    }

    fn saved_current(&self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.charge(WorkCategory::CopyByte, size_of::<InputRead>() as u64)?;
        self.current
            .clone()
            .map(InputRead::without_record)
            .ok_or_else(|| control.fail_invariant(Invariant::InputCursorState))
    }

    pub(crate) fn is_progressive(&self) -> bool {
        match self.availability {
            InputAvailability::Complete => false,
            #[cfg(test)]
            InputAvailability::Progressive { .. } => true,
        }
    }
    fn current_span(&self) -> Option<&(ConsumedSourceBoundary, ConsumedSourceBoundary)> {
        match &self.availability {
            #[cfg(test)]
            InputAvailability::Progressive { current_span, .. } => current_span.as_ref(),
            InputAvailability::Complete => None,
        }
    }
    /// Actual saved decoded endpoints, including the preprocessed CRLF span.
    pub(crate) fn consumed_span(
        &self,
        c: &mut ParseControl,
    ) -> Result<Option<(ConsumedSourceBoundary, ConsumedSourceBoundary)>, ParseStop> {
        c.require_identity(&self.owner)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if !self.is_progressive() {
            return Ok(None);
        }
        c.charge(
            WorkCategory::CopyByte,
            const { 2 * size_of::<ConsumedSourceBoundary>() } as u64,
        )?;
        Ok(Some(self.current_span().cloned().unwrap_or_else(|| {
            let boundary = self.boundary().expect("progressive boundary");
            (boundary.clone(), boundary.clone())
        })))
    }
    /// A test-only synchronization point on actual consumed progress. The
    /// original handle is polled by the existing input-read admission path.
    #[cfg(test)]
    pub(super) fn cancel_after_consumed(
        &mut self,
        through: usize,
        cancellation: crate::HtmlParseCancellation,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        c.require_identity(&self.owner)?;
        c.charge(
            WorkCategory::CopyByte,
            size_of::<Option<(usize, crate::HtmlParseCancellation)>>() as u64,
        )?;
        c.charge(WorkCategory::RecordWrite, 1)?;
        self.cancel_after_consumed = Some((through, cancellation));
        Ok(())
    }
    /// Does not decode a next item; safe even at an unavailable frontier.
    pub(crate) fn next_anchor(&self, c: &mut ParseControl) -> Result<HtmlSourceRange, ParseStop> {
        c.require_identity(&self.owner)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        if let Some(boundary) = self.boundary() {
            let next = if self.reconsume {
                &self
                    .current_span()
                    .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
                    .0
            } else {
                boundary
            };
            return self.proven_range(next, next, c);
        }
        let position = if self.reconsume {
            match self
                .current
                .as_ref()
                .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?
            {
                InputRead::Character(ch) => ch.source.start(),
                InputRead::Eof(range) => range.start(),
            }
        } else {
            self.position
        };
        self.checked_range(position, position, c)
    }
    pub(crate) fn range_between(
        &self,
        start: &ConsumedSourceBoundary,
        end: &ConsumedSourceBoundary,
        c: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        c.require_identity(&self.owner)?;
        self.proven_range(start, end, c)
    }
    fn boundary(&self) -> Option<&ConsumedSourceBoundary> {
        match &self.availability {
            InputAvailability::Complete => None,
            #[cfg(test)]
            InputAvailability::Progressive { boundary, .. } => Some(boundary),
        }
    }

    fn require_complete(&self, control: &mut ParseControl) -> Result<(), ParseStop> {
        if matches!(self.availability, InputAvailability::Complete) {
            Ok(())
        } else {
            Err(control.fail_invariant(Invariant::InputCursorState))
        }
    }

    /// Produces an exact supplied-input range through this cursor's owner.
    /// Arbitrary offsets require a separately admitted start-boundary check:
    /// the safe source provider stops after an invalid start, whereas the shared
    /// producer admits both endpoints atomically. For a valid interior UTF-8
    /// start, the provider checks that byte again; both actual reads are charged.
    /// Decoding's already-proved starts continue using the shared producer
    /// directly. This operation does not read or preprocess range payload.
    pub(crate) fn checked_range(
        &self,
        start: usize,
        end: usize,
        control: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        control.require_identity(&self.owner)?;
        control.require_source(self.source)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        if start > end || end > self.source.len() {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        if let Some(text) = self.source.as_utf8()
            && start > 0
            && start < text.len()
        {
            control.charge(WorkCategory::InputUnitRead, 1)?;
            if !text.is_char_boundary(start) {
                return Err(control.fail_invariant(Invariant::InputCursorState));
            }
        }
        self.range(start, end, control)
    }

    fn decode_at(
        &self,
        position: usize,
        control: &mut ParseControl,
    ) -> Result<InputRead, ParseStop> {
        match self.decode_poll_at(position, None, control)? {
            DecodedPoll::Ready { read, .. } => Ok(read),
            DecodedPoll::NeedInput => Err(control.fail_invariant(Invariant::InputCursorState)),
        }
    }

    fn decode_poll_at(
        &self,
        position: usize,
        start: Option<&ConsumedSourceBoundary>,
        control: &mut ParseControl,
    ) -> Result<DecodedPoll, ParseStop> {
        // Test synchronization fires only after actual committed progress and
        // before the existing charged read/poll. It adds no production poll.
        #[cfg(test)]
        if let Some((through, cancellation)) = &self.cancel_after_consumed
            && self.position >= *through
        {
            cancellation.cancel();
        }
        let raw = match self.read_raw_at(position, start, control)? {
            RawPoll::NeedInput => return Ok(DecodedPoll::NeedInput),
            RawPoll::Ready(raw) => raw,
        };
        let Some(DecodedRaw {
            mut code_point,
            mut end,
            mut boundary,
        }) = raw
        else {
            let range = match start {
                Some(start) => self.proven_range(start, start, control)?,
                None => self.range(position, position, control)?,
            };
            if start.is_some() {
                control.charge(
                    WorkCategory::CopyByte,
                    size_of::<ConsumedSourceBoundary>() as u64,
                )?;
            }
            return Ok(DecodedPoll::Ready {
                read: InputRead::Eof(range),
                _boundary: start.cloned(),
            });
        };
        control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
        let preprocessing = if code_point.value() == 0x0D {
            let was_crlf = match self.read_raw_at(end, boundary.as_ref(), control)? {
                RawPoll::NeedInput => return Ok(DecodedPoll::NeedInput),
                RawPoll::Ready(Some(next)) if next.code_point.value() == 0x0A => {
                    end = next.end;
                    boundary = next.boundary;
                    true
                }
                _ => false,
            };
            code_point = CodePoint(0x0A);
            Some(PreprocessingRecord::Newline { was_crlf })
        } else {
            match code_point.value() {
                0xD800..=0xDFFF => Some(InputDiagnostic::Surrogate),
                0xFDD0..=0xFDEF => Some(InputDiagnostic::Noncharacter),
                value if value & 0xFFFF >= 0xFFFE => Some(InputDiagnostic::Noncharacter),
                0x01..=0x08 | 0x0B | 0x0E..=0x1F | 0x7F..=0x9F => {
                    Some(InputDiagnostic::ControlCharacter)
                }
                _ => None,
            }
            .map(PreprocessingRecord::Diagnostic)
        };
        let source = match (start, boundary.as_ref()) {
            (Some(start), Some(end)) => self.proven_range(start, end, control)?,
            (None, None) => self.range(position, end, control)?,
            _ => return Err(control.fail_invariant(Invariant::InputCursorState)),
        };
        Ok(DecodedPoll::Ready {
            read: InputRead::Character(InputCharacter {
                code_point,
                source,
                preprocessing,
            }),
            _boundary: boundary,
        })
    }

    fn proven_range(
        &self,
        start: &ConsumedSourceBoundary,
        end: &ConsumedSourceBoundary,
        control: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::CopyByte, size_of::<HtmlSourceRange>() as u64)?;
        self.source
            .range_between_consumed_boundaries(start, end)
            .map_err(|_| control.fail_invariant(Invariant::InputCursorState))
    }

    fn prove_raw_end(
        &self,
        start: &ConsumedSourceBoundary,
        code_point: CodePoint,
        through: usize,
        control: &mut ParseControl,
    ) -> Result<ConsumedSourceBoundary, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
        control.charge(
            WorkCategory::CopyByte,
            size_of::<ConsumedBoundaryAdvance>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        let advance = self
            .source
            .prepare_consumed_boundary(start, code_point.value(), through)
            .map_err(|_| control.fail_invariant(Invariant::InputCursorState))?;
        control.charge(WorkCategory::InspectByte, advance.comparison_bytes() as u64)?;
        control.charge(
            WorkCategory::CopyByte,
            size_of::<ConsumedSourceBoundary>() as u64,
        )?;
        control.charge(WorkCategory::RecordWrite, 1)?;
        // Last fallible admission immediately precedes the bounded source
        // comparison. A denied call performs none of those payload reads.
        control.charge(WorkCategory::InputUnitRead, advance.source_reads() as u64)?;
        self.source
            .complete_consumed_boundary(advance)
            .map_err(|_| control.fail_invariant(Invariant::InputCursorState))
    }

    fn range(
        &self,
        start: usize,
        end: usize,
        control: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        self.require_complete(control)?;
        control.charge(WorkCategory::CopyByte, size_of::<HtmlSourceRange>() as u64)?;
        if start > end || end > self.source.len() {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        if let Some(text) = self.source.as_utf8() {
            // Cursor offsets are scalar boundaries established by the decoder
            // and private checkpoints. HtmlSource::range checks both endpoints:
            // retained std 1.97 reads one byte per interior endpoint, even when
            // start == end; zero/len read none. Admit both atomically immediately
            // before that bounded helper, so a rejected call reads neither.
            let reads =
                u64::from(start > 0 && start < text.len()) + u64::from(end > 0 && end < text.len());
            control.charge(WorkCategory::InputUnitRead, reads)?;
        }
        self.source
            .range(start, end)
            .map_err(|_| control.fail_invariant(Invariant::InputCursorState))
    }

    fn read_raw_at(
        &self,
        position: usize,
        start: Option<&ConsumedSourceBoundary>,
        control: &mut ParseControl,
    ) -> Result<RawPoll, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        let (through, sealed) = match self.availability {
            InputAvailability::Complete => (self.source.len(), true),
            #[cfg(test)]
            InputAvailability::Progressive {
                through, sealed, ..
            } => (through, sealed),
        };
        if start.map(ConsumedSourceBoundary::offset) != self.boundary().map(|_| position) {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        if let Some(start) = start {
            control.charge(WorkCategory::RecordVisit, 1)?;
            self.source
                .require_consumed_boundary(start)
                .map_err(|_| control.fail_invariant(Invariant::InputCursorState))?;
        }
        if position == through {
            return Ok(if sealed {
                RawPoll::Ready(None)
            } else {
                RawPoll::NeedInput
            });
        }
        if position > through {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        }
        if let Some(text) = self.source.as_utf8() {
            let bytes = text.as_bytes();
            control.charge(WorkCategory::InputUnitRead, 1)?;
            let first = bytes[position];
            let (width, mut value) = match first {
                0x00..=0x7F => (1, u32::from(first)),
                0xC2..=0xDF => (2, u32::from(first & 0x1F)),
                0xE0..=0xEF => (3, u32::from(first & 0x0F)),
                0xF0..=0xF4 => (4, u32::from(first & 0x07)),
                _ => return Err(control.fail_invariant(Invariant::InputCursorState)),
            };
            let Some(end) = position
                .checked_add(width)
                .filter(|end| *end <= bytes.len())
            else {
                return Err(control.fail_invariant(Invariant::InputCursorState));
            };
            if end > through {
                return Ok(RawPoll::NeedInput);
            }
            // The immutable source is already-valid UTF-8. Decode each byte
            // exactly once, with no str slicing or iterator boundary/payload
            // rereads hidden behind the advertised physical-read counter.
            let mut offset = position + 1;
            while offset < end {
                control.charge(WorkCategory::InputUnitRead, 1)?;
                let continuation = bytes[offset];
                if continuation & 0xC0 != 0x80 {
                    return Err(control.fail_invariant(Invariant::InputCursorState));
                }
                value = (value << 6) | u32::from(continuation & 0x3F);
                offset += 1;
            }
            let code_point = CodePoint(value);
            let boundary = start
                .map(|start| self.prove_raw_end(start, code_point, through, control))
                .transpose()?;
            return Ok(RawPoll::Ready(Some(DecodedRaw {
                code_point,
                end,
                boundary,
            })));
        }
        let Some(text) = self.source.as_utf16() else {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        };
        control.charge(WorkCategory::InputUnitRead, 1)?;
        let first = text.code_units()[position];
        if (0xD800..=0xDBFF).contains(&first) && position + 1 < text.len() {
            if position + 1 >= through {
                return Ok(RawPoll::NeedInput);
            }
            control.charge(WorkCategory::InputUnitRead, 1)?;
            let second = text.code_units()[position + 1];
            if (0xDC00..=0xDFFF).contains(&second) {
                let value =
                    0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00);
                let code_point = CodePoint(value);
                let boundary = start
                    .map(|start| self.prove_raw_end(start, code_point, through, control))
                    .transpose()?;
                return Ok(RawPoll::Ready(Some(DecodedRaw {
                    code_point,
                    end: position + 2,
                    boundary,
                })));
            }
        }
        if (0xD800..=0xDBFF).contains(&first) && position + 1 == through && !sealed {
            return Ok(RawPoll::NeedInput);
        }
        let code_point = CodePoint(u32::from(first));
        let boundary = start
            .map(|start| self.prove_raw_end(start, code_point, through, control))
            .transpose()?;
        Ok(RawPoll::Ready(Some(DecodedRaw {
            code_point,
            end: position + 1,
            boundary,
        })))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DocumentString, HtmlSourceCoordinates};

    fn limits() -> HtmlParseLimits {
        HtmlParseLimits::builder()
            .source_units(1000)
            .buffered_utf16_units(1000)
            .token_attributes(1000)
            .auxiliary_entries(1000)
            .managed_nodes(1000)
            .managed_edges(1000)
            .managed_attributes(1000)
            .managed_utf16_units(1000)
            .source_map_entries(1000)
            .source_map_payload_units(1000)
            .diagnostics(1000)
            .diagnostic_payload_units(1000)
            .context_effects(1000)
            .open_elements(1000)
            .tree_depth(1000)
            .managed_capacity_bytes(100_000)
            .peak_transient_bytes(100_000)
            .atomic_layout_bytes(100_000)
            .work_credits(100_000)
            .finish()
            .unwrap()
    }

    fn control(source: &HtmlSource) -> ParseControl {
        let mut control = ParseControl::new(limits(), HtmlParseCancellation::new());
        control.admit_source(source).unwrap();
        control
    }

    fn character(read: InputRead) -> InputCharacter {
        match read {
            InputRead::Character(character) => character,
            InputRead::Eof(_) => panic!("expected supplied character"),
        }
    }

    fn ready_character(poll: InputPoll) -> InputCharacter {
        match poll {
            InputPoll::Ready(read) => character(read),
            InputPoll::NeedInput => panic!("expected available character"),
        }
    }

    #[test]
    fn unsealed_end_waits_and_final_seal_yields_only_genuine_eof() {
        for source in [HtmlSource::utf8(""), HtmlSource::utf8("a")] {
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            assert_eq!(
                cursor.poll_peek(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            cursor.make_available(source.len(), &mut control).unwrap();
            if !source.is_empty() {
                let read = ready_character(cursor.poll_consume(&mut control).unwrap());
                assert_eq!(read.code_point().value(), 0x61);
                assert_eq!(read.source(), &source.range(0, 1).unwrap());
            }
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(cursor.current.is_some(), !source.is_empty());
            cursor.seal_input(&mut control).unwrap();
            let before = control.usage().work(WorkCategory::InputUnitRead);
            let eof = InputPoll::Ready(InputRead::Eof(
                source.range(source.len(), source.len()).unwrap(),
            ));
            assert_eq!(cursor.poll_consume(&mut control).unwrap(), eof);
            cursor.reconsume(&mut control).unwrap();
            assert_eq!(cursor.poll_peek(&mut control).unwrap(), eof);
            assert_eq!(cursor.poll_consume(&mut control).unwrap(), eof);
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), before);
        }
    }

    #[test]
    fn utf8_interior_frontiers_wait_without_hidden_continuation_reads() {
        for (text, scalar, width) in [("¢x", 0xA2, 2), ("€x", 0x20AC, 3), ("🙂x", 0x1F642, 4)]
        {
            let source = HtmlSource::utf8(text);
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            for through in 1..width {
                cursor.make_available(through, &mut control).unwrap();
                let before = control.usage().work(WorkCategory::InputUnitRead);
                assert_eq!(
                    cursor.poll_consume(&mut control).unwrap(),
                    InputPoll::NeedInput
                );
                // The available first byte establishes width; no continuation
                // or exclusive-end byte is inspected before full availability.
                assert_eq!(
                    control.usage().work(WorkCategory::InputUnitRead) - before,
                    1
                );
                assert_eq!(cursor.position, 0);
                assert_eq!(cursor.current, None);
                assert_eq!(cursor.processed_through, 0);
            }
            cursor.make_available(width, &mut control).unwrap();
            let before = control.usage().work(WorkCategory::InputUnitRead);
            let read = ready_character(cursor.poll_consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), scalar);
            assert_eq!(read.source(), &source.range(0, width).unwrap());
            // Each available payload byte is decoded once and compared once
            // to prove the consumed boundary. The next hidden 'x' is not read.
            assert_eq!(
                control.usage().work(WorkCategory::InputUnitRead) - before,
                2 * width as u64
            );
            assert_eq!(cursor.boundary().unwrap().offset(), width);
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
        }
    }

    #[test]
    fn ascii_frontier_range_is_proved_without_reading_the_hidden_endpoint() {
        let source = HtmlSource::utf8("ab");
        let mut control = control(&source);
        let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
        cursor.make_available(1, &mut control).unwrap();
        let read = ready_character(cursor.poll_consume(&mut control).unwrap());
        assert_eq!(read.source(), &source.range(0, 1).unwrap());
        assert_eq!(read.code_point().value(), 0x61);
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 2);
        let before = control.usage().work(WorkCategory::InputUnitRead);
        let anchor = cursor
            .proven_range(
                cursor.boundary().unwrap(),
                cursor.boundary().unwrap(),
                &mut control,
            )
            .unwrap();
        assert_eq!(anchor, source.range(1, 1).unwrap());
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), before);
    }

    #[test]
    fn utf16_pairs_wait_and_isolated_surrogates_keep_literal_records() {
        for (units, scalar, end, record) in [
            (vec![0xD83D, 0xDE42], 0x1F642, 2, None),
            (
                vec![0xD800, 0x61],
                0xD800,
                1,
                Some(PreprocessingRecord::Diagnostic(InputDiagnostic::Surrogate)),
            ),
        ] {
            let source = HtmlSource::utf16(DocumentString::from_utf16(units));
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(1, &mut control).unwrap();
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 1);
            assert_eq!(cursor.position, 0);
            cursor.make_available(2, &mut control).unwrap();
            let read = ready_character(cursor.poll_consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), scalar);
            assert_eq!(read.source(), &source.range(0, end).unwrap());
            assert_eq!(read.preprocessing(), record);
        }
        let source = HtmlSource::utf16(DocumentString::from_utf16(vec![0xD800]));
        let mut control = control(&source);
        let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
        cursor.make_available(1, &mut control).unwrap();
        assert_eq!(
            cursor.poll_consume(&mut control).unwrap(),
            InputPoll::NeedInput
        );
        cursor.seal_input(&mut control).unwrap();
        let read = ready_character(cursor.poll_consume(&mut control).unwrap());
        assert_eq!(read.code_point().value(), 0xD800);
        assert_eq!(read.source(), &source.range(0, 1).unwrap());
        assert_eq!(
            read.preprocessing(),
            Some(PreprocessingRecord::Diagnostic(InputDiagnostic::Surrogate))
        );
    }

    #[test]
    fn cr_frontiers_preserve_crlf_and_do_not_consume_a_non_lf_successor() {
        for source in [
            HtmlSource::utf8("\r\nx"),
            HtmlSource::utf16(DocumentString::from("\r\nx")),
        ] {
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(1, &mut control).unwrap();
            assert_eq!(
                cursor.poll_peek(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            assert_eq!(cursor.processed_through, 0);
            cursor.make_available(2, &mut control).unwrap();
            let peek = ready_character(cursor.poll_peek(&mut control).unwrap());
            assert_eq!(peek.preprocessing(), None);
            let read = ready_character(cursor.poll_consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), 0x0A);
            assert_eq!(read.source(), &source.range(0, 2).unwrap());
            assert_eq!(
                read.preprocessing(),
                Some(PreprocessingRecord::Newline { was_crlf: true })
            );
        }
        for text in ["\rx", "\r"] {
            let source = HtmlSource::utf8(text);
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(1, &mut control).unwrap();
            assert_eq!(
                cursor.poll_consume(&mut control).unwrap(),
                InputPoll::NeedInput
            );
            cursor.make_available(source.len(), &mut control).unwrap();
            if text.len() == 1 {
                cursor.seal_input(&mut control).unwrap();
            }
            let read = ready_character(cursor.poll_consume(&mut control).unwrap());
            assert_eq!(read.source(), &source.range(0, 1).unwrap());
            assert_eq!(
                read.preprocessing(),
                Some(PreprocessingRecord::Newline { was_crlf: false })
            );
            assert_eq!(cursor.position, 1);
            if text.len() == 2 {
                assert_eq!(
                    ready_character(cursor.poll_consume(&mut control).unwrap())
                        .code_point()
                        .value(),
                    0x78
                );
            }
        }
    }

    #[test]
    fn ascii_lookahead_distinguishes_matching_prefix_from_established_mismatch() {
        let source = HtmlSource::utf8("Doctype!");
        let mut control = control(&source);
        let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
        assert_eq!(
            cursor
                .poll_matches_ascii(b"", AsciiComparison::Sensitive, &mut control)
                .unwrap(),
            AsciiMatch::Matched
        );
        cursor.make_available(2, &mut control).unwrap();
        assert_eq!(
            cursor
                .poll_matches_ascii(b"DOCTYPE", AsciiComparison::Insensitive, &mut control)
                .unwrap(),
            AsciiMatch::NeedInput
        );
        assert_eq!(
            cursor
                .poll_matches_ascii(b"Dx", AsciiComparison::Sensitive, &mut control)
                .unwrap(),
            AsciiMatch::Mismatched
        );
        cursor.make_available(7, &mut control).unwrap();
        assert_eq!(
            cursor
                .poll_matches_ascii(b"DOCTYPE", AsciiComparison::Insensitive, &mut control)
                .unwrap(),
            AsciiMatch::Matched
        );
        assert_eq!(
            cursor
                .poll_matches_ascii(b"DOCTYPE!", AsciiComparison::Insensitive, &mut control)
                .unwrap(),
            AsciiMatch::NeedInput
        );
        assert_eq!(cursor.position, 0);
        assert_eq!(cursor.current, None);
        assert_eq!(cursor.processed_through, 0);
        cursor.make_available(8, &mut control).unwrap();
        cursor.seal_input(&mut control).unwrap();
        assert_eq!(
            cursor
                .poll_matches_ascii(b"DOCTYPE!!", AsciiComparison::Insensitive, &mut control)
                .unwrap(),
            AsciiMatch::Mismatched
        );
        cursor.poll_consume(&mut control).unwrap();
        cursor.reconsume(&mut control).unwrap();
        assert_eq!(
            cursor
                .poll_matches_ascii(b"Do", AsciiComparison::Sensitive, &mut control)
                .unwrap(),
            AsciiMatch::Matched
        );
        assert_eq!(cursor.position, 1);
        assert!(cursor.reconsume);
    }

    #[test]
    fn progressive_rewind_keeps_visibility_and_suppresses_duplicate_preprocessing() {
        let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
            0xFEFF, 0xD800, 0x61, 0xFEFF,
        ]));
        let mut control = control(&source);
        let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
        let start = cursor.mark_progressive(&mut control).unwrap();
        cursor.make_available(3, &mut control).unwrap();
        assert_eq!(
            ready_character(cursor.poll_consume(&mut control).unwrap())
                .code_point()
                .value(),
            0xFEFF
        );
        let surrogate = ready_character(cursor.poll_consume(&mut control).unwrap());
        assert_eq!(
            surrogate.preprocessing(),
            Some(PreprocessingRecord::Diagnostic(InputDiagnostic::Surrogate))
        );
        cursor.make_available(4, &mut control).unwrap();
        cursor.rewind_progressive(&start, &mut control).unwrap();
        assert_eq!(cursor.boundary().unwrap().offset(), 0);
        assert_eq!(cursor.processed_through, 2);
        assert_eq!(
            ready_character(cursor.poll_consume(&mut control).unwrap())
                .code_point()
                .value(),
            0xFEFF
        );
        let replayed = ready_character(cursor.poll_consume(&mut control).unwrap());
        assert_eq!(replayed.code_point().value(), 0xD800);
        assert_eq!(replayed.preprocessing(), None);
        assert_eq!(
            ready_character(cursor.poll_consume(&mut control).unwrap())
                .code_point()
                .value(),
            0x61
        );
        assert_eq!(
            ready_character(cursor.poll_consume(&mut control).unwrap())
                .code_point()
                .value(),
            0xFEFF
        );
        assert_eq!(
            cursor.poll_consume(&mut control).unwrap(),
            InputPoll::NeedInput
        );
    }

    #[test]
    fn invalid_frontier_transitions_stop_without_partial_mutation() {
        for operation in 0..4 {
            let source = HtmlSource::utf8("ab");
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(1, &mut control).unwrap();
            let result = match operation {
                0 => cursor.make_available(0, &mut control),
                1 => cursor.make_available(3, &mut control),
                2 => cursor.seal_input(&mut control),
                _ => cursor.peek(&mut control).map(|_| ()),
            };
            let stop = ParseStop::InternalInvariant(Invariant::InputCursorState);
            assert_eq!(result, Err(stop));
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.current, None);
            assert_eq!(cursor.poll_consume(&mut control), Err(stop));
            assert_eq!(cursor.make_available(2, &mut control), Err(stop));
            assert!(matches!(
                cursor.availability,
                InputAvailability::Progressive {
                    through: 1,
                    sealed: false,
                    ..
                }
            ));
        }
        let source = HtmlSource::utf8("a");
        let mut control = control(&source);
        let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
        cursor.make_available(1, &mut control).unwrap();
        cursor.seal_input(&mut control).unwrap();
        assert_eq!(
            cursor.make_available(1, &mut control),
            Err(ParseStop::InternalInvariant(Invariant::InputCursorState))
        );
        assert!(matches!(
            cursor.availability,
            InputAvailability::Progressive {
                through: 1,
                sealed: true,
                ..
            }
        ));
    }

    #[test]
    fn paused_cancellation_and_foreign_controls_preserve_first_stop_and_visibility() {
        let source = HtmlSource::utf8("a");
        let cancellation = HtmlParseCancellation::new();
        let mut owner = ParseControl::new(limits(), cancellation.clone());
        owner.admit_source(&source).unwrap();
        let mut cursor = InputCursor::progressive(&source, &mut owner).unwrap();
        assert_eq!(
            cursor.poll_consume(&mut owner).unwrap(),
            InputPoll::NeedInput
        );
        let before = owner.usage().clone();
        cancellation.cancel();
        assert_eq!(
            cursor.make_available(1, &mut owner),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(cursor.poll_consume(&mut owner), Err(ParseStop::Cancelled));
        assert_eq!(cursor.seal_input(&mut owner), Err(ParseStop::Cancelled));
        assert_eq!(
            owner.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert!(matches!(
            cursor.availability,
            InputAvailability::Progressive {
                through: 0,
                sealed: false,
                ..
            }
        ));
        let mut foreign = control(&source);
        let before = foreign.usage().clone();
        assert_eq!(
            cursor.make_available(1, &mut foreign),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(
            foreign.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(foreign.usage().work(WorkCategory::InputUnitRead), 0);
    }

    #[test]
    fn progressive_proof_and_commit_admissions_have_exact_independent_budgets() {
        let source = HtmlSource::utf8("ab");
        let entry = 1
            + size_of::<ControlIdentity>() as u64
            + size_of::<ConsumedSourceBoundary>() as u64
            + 1;
        let available = 2; // One frontier visit and one admitted frontier write.
        let proof_prefix = 12
            + size_of::<ConsumedBoundaryAdvance>() as u64
            + size_of::<ConsumedSourceBoundary>() as u64;
        // Retaining the actual current start/end costs two proved-boundary
        // copies and one additional persistent span write, before commit.
        let complete = 20
            + size_of::<ConsumedBoundaryAdvance>() as u64
            + 3 * size_of::<ConsumedSourceBoundary>() as u64
            + size_of::<HtmlSourceRange>() as u64
            + size_of::<InputRead>() as u64;
        for (extra, success, reads) in [
            (proof_prefix, false, 1),
            (complete - 1, false, 2),
            (complete, true, 2),
        ] {
            let caps = limits()
                .into_builder()
                .work_credits(entry + available + extra)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
            control.admit_source(&source).unwrap();
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(1, &mut control).unwrap();
            let result = cursor.poll_consume(&mut control);
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), reads);
            if success {
                let read = ready_character(result.unwrap());
                assert_eq!(read.code_point().value(), 0x61);
                assert_eq!(cursor.position, 1);
                assert_eq!(control.stop(), None);
            } else {
                let stop = ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: caps.work_credits(),
                    current: caps.work_credits(),
                    requested_delta: 1,
                };
                assert_eq!(result, Err(stop));
                assert_eq!(cursor.position, 0);
                assert_eq!(cursor.current, None);
                assert_eq!(cursor.processed_through, 0);
                assert_eq!(cursor.boundary().unwrap().offset(), 0);
                assert_eq!(cursor.poll_consume(&mut control), Err(stop));
            }
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::WorkCredits)
                    .current,
                caps.work_credits()
            );
        }
    }

    #[test]
    fn frontier_and_progressive_marker_one_short_admissions_are_atomic() {
        let source = HtmlSource::utf8("ab");
        let entry = 1
            + size_of::<ControlIdentity>() as u64
            + size_of::<ConsumedSourceBoundary>() as u64
            + 1;
        for (extra, success) in [(1, false), (2, true)] {
            let caps = limits()
                .into_builder()
                .work_credits(entry + extra)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
            control.admit_source(&source).unwrap();
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            let result = cursor.make_available(1, &mut control);
            if success {
                assert_eq!(result, Ok(()));
            } else {
                let stop = ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: caps.work_credits(),
                    current: caps.work_credits(),
                    requested_delta: 1,
                };
                assert_eq!(result, Err(stop));
                assert_eq!(cursor.make_available(2, &mut control), Err(stop));
            }
            assert!(
                matches!(cursor.availability, InputAvailability::Progressive { through, sealed: false, .. } if through == usize::from(success))
            );
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::SourceUnits)
                    .current,
                2
            );
        }
        let marker_bytes = size_of::<ProgressiveInputCheckpoint>() as u64;
        for (extra, success) in [(marker_bytes, false), (1 + marker_bytes, true)] {
            let caps = limits()
                .into_builder()
                .work_credits(entry + extra)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
            control.admit_source(&source).unwrap();
            let cursor = InputCursor::progressive(&source, &mut control).unwrap();
            let result = cursor.mark_progressive(&mut control);
            if success {
                let marker = result.unwrap();
                assert_eq!(marker.checkpoint.position, 0);
                assert_eq!(marker.boundary.offset(), 0);
                assert_eq!(control.stop(), None);
            } else {
                assert!(matches!(result, Err(ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    requested_delta,
                    ..
                }) if requested_delta == marker_bytes));
            }
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.current, None);
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
        }
    }

    #[test]
    fn progressive_ranges_and_markers_cannot_use_whole_source_shortcuts() {
        let source = HtmlSource::utf8("a");
        for operation in 0..4 {
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            let stop = ParseStop::InternalInvariant(Invariant::InputCursorState);
            let result = match operation {
                0 => cursor.checked_range(0, 1, &mut control).map(|_| ()),
                1 => cursor.mark(&mut control).map(|_| ()),
                2 => cursor.consume(&mut control).map(|_| ()),
                _ => cursor
                    .matches_ascii(b"a", AsciiComparison::Sensitive, &mut control)
                    .map(|_| ()),
            };
            assert_eq!(result, Err(stop));
            assert_eq!(cursor.poll_consume(&mut control), Err(stop));
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            assert_eq!(cursor.position, 0);
        }
        let mut owner = control(&source);
        let cursor = InputCursor::progressive(&source, &mut owner).unwrap();
        let marker = cursor.mark_progressive(&mut owner).unwrap();
        let mut foreign = control(&source);
        let mut other = InputCursor::progressive(&source, &mut foreign).unwrap();
        let before = foreign.usage().clone();
        assert_eq!(
            other.rewind_progressive(&marker, &mut foreign),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        );
        assert_eq!(
            foreign.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(foreign.usage().work(WorkCategory::InputUnitRead), 0);
        assert_eq!(other.position, 0);
        assert_eq!(owner.stop(), None);
    }

    #[test]
    fn utf8_scalars_null_and_bom_keep_original_byte_spans() {
        let source = HtmlSource::utf8("A🙂\0\u{FEFF}");
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        for (value, start, end) in [(0x41, 0, 1), (0x1F642, 1, 5), (0, 5, 6), (0xFEFF, 6, 9)] {
            let read = character(cursor.consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), value);
            assert_eq!((read.source().start(), read.source().end()), (start, end));
            assert_eq!(read.source().source(), source.identity());
            assert_eq!(
                read.source().coordinates(),
                HtmlSourceCoordinates::Utf8Bytes
            );
            assert_eq!(read.preprocessing(), None);
        }
        assert_eq!(
            cursor.consume(&mut control).unwrap(),
            InputRead::Eof(source.range(9, 9).unwrap())
        );
        // Nine decoding bytes, plus both sides of three interior boundaries.
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 15);
    }

    #[test]
    fn utf16_pairs_isolates_and_both_boms_are_lossless() {
        let units = [0xFEFF, 0xD83D, 0xDE42, 0xD800, 0x61, 0xDC00, 0xFEFF];
        let source = HtmlSource::utf16(DocumentString::from_utf16(units.to_vec()));
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        for (value, start, end, diagnostic) in [
            (0xFEFF, 0, 1, None),
            (0x1F642, 1, 3, None),
            (0xD800, 3, 4, Some(InputDiagnostic::Surrogate)),
            (0x61, 4, 5, None),
            (0xDC00, 5, 6, Some(InputDiagnostic::Surrogate)),
            (0xFEFF, 6, 7, None),
        ] {
            let read = character(cursor.consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), value);
            assert_eq!((read.source().start(), read.source().end()), (start, end));
            assert_eq!(
                read.source().coordinates(),
                HtmlSourceCoordinates::Utf16CodeUnits
            );
            assert_eq!(
                read.preprocessing(),
                diagnostic.map(PreprocessingRecord::Diagnostic)
            );
        }
        assert_eq!(source.as_utf16().unwrap().code_units(), units);
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 8);
    }

    #[test]
    fn crlf_and_lone_cr_each_emit_one_lf_with_exact_normalization_record() {
        for source in [
            HtmlSource::utf8("\r\n\rx\n"),
            HtmlSource::utf16(DocumentString::from("\r\n\rx\n")),
        ] {
            let mut control = control(&source);
            let mut cursor = InputCursor::new(&source, &mut control).unwrap();
            for (value, start, end, preprocessing) in [
                (
                    0x0A,
                    0,
                    2,
                    Some(PreprocessingRecord::Newline { was_crlf: true }),
                ),
                (
                    0x0A,
                    2,
                    3,
                    Some(PreprocessingRecord::Newline { was_crlf: false }),
                ),
                (0x78, 3, 4, None),
                (0x0A, 4, 5, None),
            ] {
                let read = character(cursor.consume(&mut control).unwrap());
                assert_eq!(read.code_point().value(), value);
                assert_eq!((read.source().start(), read.source().end()), (start, end));
                assert_eq!(read.preprocessing(), preprocessing);
            }
        }
    }

    #[test]
    fn preprocessing_errors_exclude_null_and_ascii_whitespace() {
        let source = HtmlSource::utf16(DocumentString::from_utf16(vec![
            0, 9, 10, 12, 1, 11, 0x7F, 0x9F, 0xFDD0, 0xDBFF, 0xDFFF,
        ]));
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        for expected in [
            None,
            None,
            None,
            None,
            Some(InputDiagnostic::ControlCharacter),
            Some(InputDiagnostic::ControlCharacter),
            Some(InputDiagnostic::ControlCharacter),
            Some(InputDiagnostic::ControlCharacter),
            Some(InputDiagnostic::Noncharacter),
            Some(InputDiagnostic::Noncharacter),
        ] {
            let read = character(cursor.consume(&mut control).unwrap());
            assert_eq!(
                read.preprocessing(),
                expected.map(PreprocessingRecord::Diagnostic)
            );
        }
    }

    #[test]
    fn lookahead_reconsume_and_rewind_charge_work_without_duplicate_records_or_source() {
        let source = HtmlSource::utf8("\r\nAb!");
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let start = cursor.mark(&mut control).unwrap();
        assert!(
            cursor
                .matches_ascii(b"\nAb", AsciiComparison::Sensitive, &mut control)
                .unwrap()
        );
        assert!(
            cursor
                .matches_ascii(b"\naB", AsciiComparison::Insensitive, &mut control)
                .unwrap()
        );
        assert!(
            !cursor
                .matches_ascii(b"\nab", AsciiComparison::Sensitive, &mut control)
                .unwrap()
        );
        assert_eq!(cursor.position(&mut control).unwrap(), 0);
        assert_eq!(
            character(cursor.peek(&mut control).unwrap()).preprocessing(),
            None
        );
        let first = character(cursor.consume(&mut control).unwrap());
        assert_eq!(
            first.preprocessing(),
            Some(PreprocessingRecord::Newline { was_crlf: true })
        );
        cursor.reconsume(&mut control).unwrap();
        let replay = character(cursor.consume(&mut control).unwrap());
        assert_eq!(replay.code_point(), first.code_point());
        assert_eq!(replay.source(), first.source());
        assert_eq!(replay.preprocessing(), None);
        cursor.rewind(&start, &mut control).unwrap();
        let before = control.usage().work(WorkCategory::InputUnitRead);
        assert_eq!(
            character(cursor.consume(&mut control).unwrap()).preprocessing(),
            None
        );
        assert_eq!(
            control.usage().work(WorkCategory::InputUnitRead) - before,
            3
        );
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::SourceUnits)
                .current,
            5
        );
    }

    #[test]
    fn cancellation_and_work_cap_stop_before_cursor_position_changes() {
        let source = HtmlSource::utf8("x");
        let cancellation = HtmlParseCancellation::new();
        let mut control = ParseControl::new(limits(), cancellation.clone());
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        cancellation.cancel();
        assert_eq!(cursor.consume(&mut control), Err(ParseStop::Cancelled));
        assert_eq!(cursor.position, 0);

        let mut caps = limits();
        caps = caps.into_builder().work_credits(100).finish().unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let remaining = caps.work_credits()
            - control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current;
        control.charge(WorkCategory::StateStep, remaining).unwrap();
        let stop = cursor.consume(&mut control).unwrap_err();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: HtmlParseResource::WorkCredits,
                ..
            }
        ));
        assert_eq!(cursor.position, 0);
        assert_eq!(cursor.consume(&mut control), Err(stop));
    }

    #[test]
    fn invalid_reconsume_is_a_latched_invariant_and_codepoints_are_checked() {
        assert_eq!(CodePoint::new(0xD800).unwrap().value(), 0xD800);
        assert_eq!(CodePoint::new(0x10FFFF).unwrap().value(), 0x10FFFF);
        assert_eq!(CodePoint::new(0x110000), None);
        let source = HtmlSource::utf8("");
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let stop = ParseStop::InternalInvariant(Invariant::InputCursorState);
        assert_eq!(cursor.reconsume(&mut control), Err(stop));
        assert_eq!(cursor.peek(&mut control), Err(stop));
    }

    #[test]
    fn a_fresh_owner_with_the_same_source_cannot_use_cursor_or_checkpoint() {
        let source = HtmlSource::utf8("x");
        let mut first = control(&source);
        let mut cursor = InputCursor::new(&source, &mut first).unwrap();
        let mark = cursor.mark(&mut first).unwrap();
        let stop = ParseStop::InternalInvariant(Invariant::ForeignControl);
        for operation in 0..7 {
            let mut other = control(&source.clone());
            let result = match operation {
                0 => cursor.consume(&mut other).map(|_| ()),
                1 => cursor.peek(&mut other).map(|_| ()),
                2 => cursor.reconsume(&mut other),
                3 => cursor.mark(&mut other).map(|_| ()),
                4 => cursor
                    .matches_ascii(b"x", AsciiComparison::Sensitive, &mut other)
                    .map(|_| ()),
                5 => cursor.position(&mut other).map(|_| ()),
                _ => cursor.rewind(&mark, &mut other),
            };
            assert_eq!(result, Err(stop));
            assert_eq!(other.usage().work(WorkCategory::InputUnitRead), 0);
            assert_eq!(cursor.position, 0);
        }
        let mut second = control(&source);
        let mut second_cursor = InputCursor::new(&source, &mut second).unwrap();
        assert_eq!(second_cursor.rewind(&mark, &mut second), Err(stop));
        assert_eq!(first.stop(), None);
    }

    #[test]
    fn final_cr_and_high_surrogate_are_processed_before_distinct_eof() {
        for source in [
            HtmlSource::utf8("\u{FEFF}\r"),
            HtmlSource::utf16(DocumentString::from_utf16(vec![0xFEFF, 0xD800])),
        ] {
            let mut control = control(&source);
            let mut cursor = InputCursor::new(&source, &mut control).unwrap();
            assert_eq!(
                character(cursor.consume(&mut control).unwrap())
                    .code_point()
                    .value(),
                0xFEFF
            );
            let final_character = character(cursor.consume(&mut control).unwrap());
            let expected = if source.as_utf8().is_some() {
                (0x0A, PreprocessingRecord::Newline { was_crlf: false })
            } else {
                (
                    0xD800,
                    PreprocessingRecord::Diagnostic(InputDiagnostic::Surrogate),
                )
            };
            assert_eq!(final_character.code_point().value(), expected.0);
            assert_eq!(final_character.preprocessing(), Some(expected.1));
            let eof = InputRead::Eof(source.range(source.len(), source.len()).unwrap());
            assert_eq!(cursor.consume(&mut control).unwrap(), eof);
            cursor.reconsume(&mut control).unwrap();
            assert_eq!(cursor.peek(&mut control).unwrap(), eof);
            assert_eq!(cursor.consume(&mut control).unwrap(), eof);
        }
    }

    #[test]
    fn empty_source_yields_source_qualified_eof_without_input_reads() {
        let source = HtmlSource::utf8("");
        let mut control = control(&source);
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        assert!(
            cursor
                .matches_ascii(b"", AsciiComparison::Sensitive, &mut control)
                .unwrap()
        );
        assert!(
            !cursor
                .matches_ascii(b"x", AsciiComparison::Sensitive, &mut control)
                .unwrap()
        );
        assert_eq!(
            cursor.consume(&mut control).unwrap(),
            InputRead::Eof(source.range(0, 0).unwrap())
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
    }

    #[test]
    fn utf8_actual_reads_include_each_interior_source_range_endpoint() {
        let source = HtmlSource::utf8("ab");
        let mut control = control(&source);
        let cursor = InputCursor::new(&source, &mut control).unwrap();
        // range() checks start then end independently, including identical
        // interior endpoints. Zero and source length require no byte read.
        for (start, end, reads) in [(0, 0, 0), (0, 1, 1), (1, 1, 2), (1, 2, 1), (2, 2, 0)] {
            let before = control.usage().work(WorkCategory::InputUnitRead);
            assert_eq!(
                cursor.range(start, end, &mut control).unwrap(),
                source.range(start, end).unwrap()
            );
            assert_eq!(
                control.usage().work(WorkCategory::InputUnitRead) - before,
                reads
            );
        }
    }

    #[test]
    fn utf8_actual_reads_cover_payload_and_interior_endpoints_without_decoder_rereads() {
        for (text, value, end, reads, preprocessing) in [
            ("ab", 0x61, 1, 2, None),
            ("🙂x", 0x1F642, 4, 5, None),
            (
                "\r\nx",
                0x0A,
                2,
                3,
                Some(PreprocessingRecord::Newline { was_crlf: true }),
            ),
            (
                "\rx",
                0x0A,
                1,
                3,
                Some(PreprocessingRecord::Newline { was_crlf: false }),
            ),
        ] {
            let source = HtmlSource::utf8(text);
            let mut control = control(&source);
            let mut cursor = InputCursor::new(&source, &mut control).unwrap();
            let read = character(cursor.consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), value);
            assert_eq!(read.source(), &source.range(0, end).unwrap());
            assert_eq!(read.preprocessing(), preprocessing);
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), reads);
            assert_eq!(source.as_utf8(), Some(text));
        }
    }

    #[test]
    fn utf8_actual_reads_stop_before_an_unadmitted_interior_boundary_check() {
        let source = HtmlSource::utf8("ab");
        let mut caps = limits();
        // Entry: one source metadata visit, one identity copy, one cursor write.
        // Before range endpoints: consume/raw/range visits, decoding 'a',
        // classifying its u32 value, and copying the fixed range record.
        let entry = 1 + size_of::<ControlIdentity>() as u64 + 1;
        let before_endpoint =
            3 + 1 + size_of::<CodePoint>() as u64 + size_of::<HtmlSourceRange>() as u64;
        caps = caps
            .into_builder()
            .work_credits(entry + before_endpoint)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let stop = ParseStop::LimitExceeded {
            resource: HtmlParseResource::WorkCredits,
            limit: caps.work_credits(),
            current: caps.work_credits(),
            requested_delta: 1,
        };
        assert_eq!(cursor.consume(&mut control), Err(stop));
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 1);
        assert_eq!(cursor.position, 0);
        assert_eq!(cursor.processed_through, 0);
        assert_eq!(cursor.current, None);
        assert_eq!(source.as_utf8(), Some("ab"));
        assert_eq!(cursor.consume(&mut control), Err(stop));

        // The same first character succeeds at the exact complete budget: two
        // input reads, plus storing the fixed current-input record and cursor.
        caps = caps
            .into_builder()
            .work_credits(entry + before_endpoint + 1 + size_of::<InputRead>() as u64 + 1)
            .finish()
            .unwrap();
        let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let read = character(cursor.consume(&mut control).unwrap());
        assert_eq!(read.code_point().value(), 0x61);
        assert_eq!(read.source(), &source.range(0, 1).unwrap());
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 2);
        assert_eq!(
            control
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            caps.work_credits()
        );
        assert_eq!(control.stop(), None);
        assert_eq!(cursor.position, 1);
        assert_eq!(source.as_utf8(), Some("ab"));
    }

    #[test]
    fn utf8_actual_reads_admit_both_interior_endpoints_atomically() {
        let source = HtmlSource::utf8("ab");
        let entry = 1 + size_of::<ControlIdentity>() as u64 + 1;
        let range_record = 1 + size_of::<HtmlSourceRange>() as u64;
        for remaining in [1, 2] {
            let mut caps = limits();
            caps = caps
                .into_builder()
                .work_credits(entry + range_record + remaining)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
            control.admit_source(&source).unwrap();
            let cursor = InputCursor::new(&source, &mut control).unwrap();
            let result = cursor.range(1, 1, &mut control);
            if remaining == 1 {
                assert_eq!(
                    result,
                    Err(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        limit: caps.work_credits(),
                        current: entry + range_record,
                        requested_delta: 2,
                    })
                );
                assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            } else {
                assert_eq!(result, Ok(source.range(1, 1).unwrap()));
                assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 2);
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    caps.work_credits()
                );
            }
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.processed_through, 0);
            assert_eq!(cursor.current, None);
            assert_eq!(source.as_utf8(), Some("ab"));
        }
    }

    #[test]
    fn direct_utf8_decoding_preserves_scalar_encoding_boundaries() {
        for (text, value, bytes) in [
            ("\u{7F}", 0x7F, 1),
            ("\u{80}", 0x80, 2),
            ("\u{7FF}", 0x7FF, 2),
            ("\u{800}", 0x800, 3),
            ("\u{D7FF}", 0xD7FF, 3),
            ("\u{E000}", 0xE000, 3),
            ("\u{FFFF}", 0xFFFF, 3),
            ("\u{10000}", 0x10000, 4),
            ("\u{10FFFF}", 0x10FFFF, 4),
        ] {
            let source = HtmlSource::utf8(text);
            let mut control = control(&source);
            let mut cursor = InputCursor::new(&source, &mut control).unwrap();
            let read = character(cursor.consume(&mut control).unwrap());
            assert_eq!(read.code_point().value(), value);
            assert_eq!(read.source(), &source.range(0, bytes).unwrap());
            assert_eq!(
                control.usage().work(WorkCategory::InputUnitRead),
                bytes as u64
            );
            assert_eq!(source.as_utf8(), Some(text));
        }
    }

    #[test]
    fn checked_utf8_ranges_charge_start_preflight_and_each_provider_endpoint() {
        let source = HtmlSource::utf8("a🙂b");
        let mut control = control(&source);
        let cursor = InputCursor::new(&source, &mut control).unwrap();
        for (start, end, reads) in [
            (0, 0, 0),
            (0, 1, 1),
            (1, 1, 3),
            (1, 5, 3),
            (5, 6, 2),
            (6, 6, 0),
        ] {
            let before = control.usage().clone();
            let range = cursor.checked_range(start, end, &mut control).unwrap();
            assert_eq!(range.source(), source.identity());
            assert_eq!(range.coordinates(), HtmlSourceCoordinates::Utf8Bytes);
            assert_eq!((range.start(), range.end()), (start, end));
            for (category, expected) in [
                (WorkCategory::InputUnitRead, reads),
                (WorkCategory::RecordVisit, 2),
                (WorkCategory::CopyByte, size_of::<HtmlSourceRange>() as u64),
                (WorkCategory::InspectByte, 0),
                (WorkCategory::StateStep, 0),
                (WorkCategory::RecordWrite, 0),
                (WorkCategory::ReserveByte, 0),
                (WorkCategory::ProviderCall, 0),
            ] {
                assert_eq!(
                    control.usage().work(category) - before.work(category),
                    expected
                );
            }
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::SourceUnits)
                    .current,
                6
            );
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.current, None);
            assert_eq!(cursor.processed_through, 0);
        }
    }

    #[test]
    fn checked_utf16_ranges_allow_surrogate_unit_boundaries_without_payload_reads() {
        let source = HtmlSource::utf16(DocumentString::from_utf16(vec![0xD83D, 0xDE42, 0xD800]));
        let mut control = control(&source);
        let cursor = InputCursor::new(&source, &mut control).unwrap();
        for (start, end) in [(0, 1), (1, 2), (2, 3), (1, 1), (3, 3)] {
            let before = control.usage().clone();
            let range = cursor.checked_range(start, end, &mut control).unwrap();
            assert_eq!(range.source(), source.identity());
            assert_eq!(range.coordinates(), HtmlSourceCoordinates::Utf16CodeUnits);
            assert_eq!((range.start(), range.end()), (start, end));
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            assert_eq!(control.usage().work(WorkCategory::InspectByte), 0);
            assert_eq!(
                control.usage().work(WorkCategory::RecordVisit)
                    - before.work(WorkCategory::RecordVisit),
                2
            );
            assert_eq!(
                control.usage().work(WorkCategory::CopyByte) - before.work(WorkCategory::CopyByte),
                size_of::<HtmlSourceRange>() as u64
            );
            assert_eq!(
                control
                    .usage()
                    .resource(HtmlParseResource::SourceUnits)
                    .current,
                3
            );
        }
    }

    #[test]
    fn checked_ranges_reject_foreign_controls_before_work_or_source_access() {
        let source = HtmlSource::utf8("a🙂b");
        let mut owner = control(&source);
        let cursor = InputCursor::new(&source, &mut owner).unwrap();
        for other_source in [source.clone(), HtmlSource::utf8("a🙂b")] {
            let mut other = control(&other_source);
            let before = other.usage().clone();
            let stop = ParseStop::InternalInvariant(Invariant::ForeignControl);
            assert_eq!(cursor.checked_range(1, 5, &mut other), Err(stop));
            assert_eq!(
                other.usage().resource(HtmlParseResource::WorkCredits),
                before.resource(HtmlParseResource::WorkCredits)
            );
            assert_eq!(other.usage().work(WorkCategory::InputUnitRead), 0);
            assert_eq!(cursor.checked_range(0, 0, &mut other), Err(stop));
        }
        assert_eq!(owner.stop(), None);
    }

    #[test]
    fn checked_ranges_preserve_cancelled_and_foreign_source_stops_without_work() {
        let source = HtmlSource::utf8("a🙂b");
        let cancellation = HtmlParseCancellation::new();
        let mut control = ParseControl::new(limits(), cancellation.clone());
        control.admit_source(&source).unwrap();
        let cursor = InputCursor::new(&source, &mut control).unwrap();
        let before = control.usage().clone();
        cancellation.cancel();
        assert_eq!(
            cursor.checked_range(1, 5, &mut control),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            cursor.checked_range(0, 0, &mut control),
            Err(ParseStop::Cancelled)
        );
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);

        let mut control = self::control(&source);
        let cursor = InputCursor::new(&source, &mut control).unwrap();
        let stop = ParseStop::InternalInvariant(Invariant::ForeignSource);
        assert_eq!(control.admit_source(&HtmlSource::utf8("a🙂b")), Err(stop));
        let before = control.usage().clone();
        assert_eq!(cursor.checked_range(1, 5, &mut control), Err(stop));
        assert_eq!(
            control.usage().resource(HtmlParseResource::WorkCredits),
            before.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
    }

    #[test]
    fn checked_ranges_reject_bounds_and_stop_at_the_first_invalid_scalar_endpoint() {
        let source = HtmlSource::utf8("a🙂b");
        for (start, end, reads) in [
            (5, 1, 0),
            (0, 7, 0),
            (usize::MAX, usize::MAX, 0),
            (0, 2, 1),
            (1, 2, 3),
            (2, 5, 1),
            (2, 2, 1),
        ] {
            let mut control = control(&source);
            let cursor = InputCursor::new(&source, &mut control).unwrap();
            let stop = ParseStop::InternalInvariant(Invariant::InputCursorState);
            assert_eq!(cursor.checked_range(start, end, &mut control), Err(stop));
            assert_eq!(control.usage().work(WorkCategory::InputUnitRead), reads);
            let before = control.usage().clone();
            assert_eq!(cursor.checked_range(0, 0, &mut control), Err(stop));
            assert_eq!(
                control.usage().resource(HtmlParseResource::WorkCredits),
                before.resource(HtmlParseResource::WorkCredits)
            );
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.current, None);
        }
    }

    #[test]
    fn checked_ranges_stop_at_entry_or_before_both_endpoint_reads_with_exact_caps() {
        let source = HtmlSource::utf8("ab");
        let entry = 1 + size_of::<ControlIdentity>() as u64 + 1;
        let copy = size_of::<HtmlSourceRange>() as u64;
        for available in [0, 1, 2, 3, 3 + copy, 4 + copy, 5 + copy] {
            let mut caps = limits();
            caps = caps
                .into_builder()
                .work_credits(entry + available)
                .finish()
                .unwrap();
            let mut control = ParseControl::new(caps, HtmlParseCancellation::new());
            control.admit_source(&source).unwrap();
            let cursor = InputCursor::new(&source, &mut control).unwrap();
            let result = cursor.checked_range(1, 1, &mut control);
            if available == 5 + copy {
                let range = result.unwrap();
                assert_eq!((range.start(), range.end()), (1, 1));
                assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 3);
                assert_eq!(
                    control
                        .usage()
                        .resource(HtmlParseResource::WorkCredits)
                        .current,
                    caps.work_credits()
                );
            } else {
                let (current, requested_delta) = if available < 3 {
                    (entry + available, 1)
                } else if available == 3 {
                    (entry + 3, copy)
                } else {
                    (entry + 3 + copy, 2)
                };
                let stop = ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits,
                    limit: caps.work_credits(),
                    current,
                    requested_delta,
                };
                assert_eq!(result, Err(stop));
                assert_eq!(
                    control.usage().work(WorkCategory::InputUnitRead),
                    u64::from(available >= 2)
                );
                assert_eq!(
                    control.usage().work(WorkCategory::RecordVisit),
                    1 + u64::from(available > 0) + u64::from(available >= 3)
                );
                assert_eq!(cursor.checked_range(0, 0, &mut control), Err(stop));
            }
            assert_eq!(cursor.position, 0);
            assert_eq!(cursor.current, None);
            assert_eq!(cursor.processed_through, 0);
        }
    }

    #[test]
    fn a_frontier_beyond_the_source_stops_instead_of_indexing_past_the_end() {
        for source in [
            HtmlSource::utf8("ab"),
            HtmlSource::utf16(DocumentString::from_utf16(vec![0x61, 0x62])),
        ] {
            let mut control = control(&source);
            let mut cursor = InputCursor::progressive(&source, &mut control).unwrap();
            cursor.make_available(source.len(), &mut control).unwrap();
            for _ in 0..2 {
                ready_character(cursor.poll_consume(&mut control).unwrap());
            }
            // The private frontier claims more input than the source holds.
            let InputAvailability::Progressive { through, .. } = &mut cursor.availability else {
                panic!("progressive cursor");
            };
            *through = source.len() + 98;
            let stop = ParseStop::InternalInvariant(Invariant::InputCursorState);
            assert_eq!(cursor.poll_consume(&mut control), Err(stop));
            assert_eq!(control.stop(), Some(stop));
            assert_eq!(cursor.position, 2);
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::SourceMapEntries,
                HtmlParseResource::ManagedCapacityBytes,
            ] {
                assert_eq!(
                    control.usage().resource(resource).current,
                    0,
                    "{resource:?}"
                );
            }
        }
    }
}
