//! Indexed input for the private already-decoded parser. Literal surrogate code
//! points are retained; newline preprocessing is separate from token recovery.

use super::control::{ControlIdentity, Invariant, ParseControl, ParseStop, WorkCategory};
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

pub(crate) struct InputCursor<'source> {
    owner: ControlIdentity,
    source: &'source HtmlSource,
    position: usize,
    current: Option<InputRead>,
    reconsume: bool,
    /// Preprocessing records are emitted on first committed traversal only.
    /// Lookahead does not advance this frontier; replay cannot duplicate errors.
    processed_through: usize,
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
        })
    }

    pub(crate) fn position(&self, control: &mut ParseControl) -> Result<usize, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        Ok(self.position)
    }

    pub(crate) fn peek(&self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        if self.reconsume {
            return self.saved_current(control);
        }
        Ok(self.decode_at(self.position, control)?.without_record())
    }

    pub(crate) fn consume(&mut self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.require_identity(&self.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
        let mut read = if self.reconsume {
            self.saved_current(control)?
        } else {
            self.decode_at(self.position, control)?
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
        control.charge(WorkCategory::CopyByte, size_of::<InputCheckpoint>() as u64)?;
        Ok(InputCheckpoint {
            owner: self.owner.clone(),
            source: self.source.identity().clone(),
            position: self.position,
            current: self.current.clone(),
            reconsume: self.reconsume,
        })
    }

    pub(crate) fn rewind(
        &mut self,
        mark: &InputCheckpoint,
        control: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        control.require_identity(&self.owner)?;
        control.require_identity(&mark.owner)?;
        control.charge(WorkCategory::RecordVisit, 1)?;
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

    fn saved_current(&self, control: &mut ParseControl) -> Result<InputRead, ParseStop> {
        control.charge(WorkCategory::CopyByte, size_of::<InputRead>() as u64)?;
        self.current
            .clone()
            .map(InputRead::without_record)
            .ok_or_else(|| control.fail_invariant(Invariant::InputCursorState))
    }

    fn decode_at(
        &self,
        position: usize,
        control: &mut ParseControl,
    ) -> Result<InputRead, ParseStop> {
        let Some((mut code_point, mut end)) = self.read_raw_at(position, control)? else {
            return Ok(InputRead::Eof(self.range(position, position, control)?));
        };
        control.charge(WorkCategory::InspectByte, size_of::<CodePoint>() as u64)?;
        let preprocessing = if code_point.value() == 0x0D {
            let was_crlf = match self.read_raw_at(end, control)? {
                Some((next, next_end)) if next.value() == 0x0A => {
                    end = next_end;
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
        Ok(InputRead::Character(InputCharacter {
            code_point,
            source: self.range(position, end, control)?,
            preprocessing,
        }))
    }

    fn range(
        &self,
        start: usize,
        end: usize,
        control: &mut ParseControl,
    ) -> Result<HtmlSourceRange, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
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
        control: &mut ParseControl,
    ) -> Result<Option<(CodePoint, usize)>, ParseStop> {
        control.charge(WorkCategory::RecordVisit, 1)?;
        if position == self.source.len() {
            return Ok(None);
        }
        if position > self.source.len() {
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
            return Ok(Some((CodePoint(value), end)));
        }
        let Some(text) = self.source.as_utf16() else {
            return Err(control.fail_invariant(Invariant::InputCursorState));
        };
        control.charge(WorkCategory::InputUnitRead, 1)?;
        let first = text.code_units()[position];
        if (0xD800..=0xDBFF).contains(&first) && position + 1 < text.len() {
            control.charge(WorkCategory::InputUnitRead, 1)?;
            let second = text.code_units()[position + 1];
            if (0xDC00..=0xDFFF).contains(&second) {
                let value =
                    0x10000 + ((u32::from(first) - 0xD800) << 10) + (u32::from(second) - 0xDC00);
                return Ok(Some((CodePoint(value), position + 2)));
            }
        }
        Ok(Some((CodePoint(u32::from(first)), position + 1)))
    }
}

#[cfg(test)]
mod tests {
    use super::super::control::{Cancellation, ParseLimits, Resource};
    use super::*;
    use crate::{DocumentString, HtmlSourceCoordinates};

    fn limits() -> ParseLimits {
        ParseLimits {
            source_units: 1000,
            buffered_utf16_units: 1000,
            token_attributes: 1000,
            auxiliary_entries: 1000,
            managed_nodes: 1000,
            managed_edges: 1000,
            managed_attributes: 1000,
            managed_utf16_units: 1000,
            source_map_entries: 1000,
            source_map_payload_units: 1000,
            diagnostics: 1000,
            diagnostic_payload_units: 1000,
            context_effects: 1000,
            open_elements: 1000,
            tree_depth: 1000,
            managed_capacity_bytes: 100_000,
            peak_transient_bytes: 100_000,
            atomic_layout_bytes: 100_000,
            work_credits: 100_000,
        }
    }

    fn control(source: &HtmlSource) -> ParseControl {
        let mut control = ParseControl::new(limits(), Cancellation::new()).unwrap();
        control.admit_source(source).unwrap();
        control
    }

    fn character(read: InputRead) -> InputCharacter {
        match read {
            InputRead::Character(character) => character,
            InputRead::Eof(_) => panic!("expected supplied character"),
        }
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
        assert_eq!(control.usage().resource(Resource::SourceUnits).current, 5);
    }

    #[test]
    fn cancellation_and_work_cap_stop_before_cursor_position_changes() {
        let source = HtmlSource::utf8("x");
        let cancellation = Cancellation::new();
        let mut control = ParseControl::new(limits(), cancellation.clone()).unwrap();
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        cancellation.cancel();
        assert_eq!(cursor.consume(&mut control), Err(ParseStop::Cancelled));
        assert_eq!(cursor.position, 0);

        let mut caps = limits();
        caps.work_credits = 100;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let remaining = caps.work_credits - control.usage().resource(Resource::WorkCredits).current;
        control.charge(WorkCategory::StateStep, remaining).unwrap();
        let stop = cursor.consume(&mut control).unwrap_err();
        assert!(matches!(
            stop,
            ParseStop::LimitExceeded {
                resource: Resource::WorkCredits,
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
        caps.work_credits = entry + before_endpoint;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let stop = ParseStop::LimitExceeded {
            resource: Resource::WorkCredits,
            limit: caps.work_credits,
            current: caps.work_credits,
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
        caps.work_credits = entry + before_endpoint + 1 + size_of::<InputRead>() as u64 + 1;
        let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
        control.admit_source(&source).unwrap();
        let mut cursor = InputCursor::new(&source, &mut control).unwrap();
        let read = character(cursor.consume(&mut control).unwrap());
        assert_eq!(read.code_point().value(), 0x61);
        assert_eq!(read.source(), &source.range(0, 1).unwrap());
        assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 2);
        assert_eq!(
            control.usage().resource(Resource::WorkCredits).current,
            caps.work_credits
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
            caps.work_credits = entry + range_record + remaining;
            let mut control = ParseControl::new(caps, Cancellation::new()).unwrap();
            control.admit_source(&source).unwrap();
            let cursor = InputCursor::new(&source, &mut control).unwrap();
            let result = cursor.range(1, 1, &mut control);
            if remaining == 1 {
                assert_eq!(
                    result,
                    Err(ParseStop::LimitExceeded {
                        resource: Resource::WorkCredits,
                        limit: caps.work_credits,
                        current: entry + range_record,
                        requested_delta: 2,
                    })
                );
                assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 0);
            } else {
                assert_eq!(result, Ok(source.range(1, 1).unwrap()));
                assert_eq!(control.usage().work(WorkCategory::InputUnitRead), 2);
                assert_eq!(
                    control.usage().resource(Resource::WorkCredits).current,
                    caps.work_credits
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
}
