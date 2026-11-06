//! Boa's lexer cursor that manages the input byte stream.

use super::Error;
use crate::{
    frontend::{Control, FrontendControl, FrontendWork},
    source::{ReadChar, UTF8Input},
};
use std::io::{self, ErrorKind};
use surgeist_js_ast::{LinearPosition, Position, PositionGroup, SourceText};

/// Cursor over the source code.
#[derive(Debug)]
pub(super) struct Cursor<'control, R> {
    iter: R,
    control: Control<'control>,
    pos: Position,
    module: bool,
    strict: bool,
    peeked: [Option<u32>; 4],
    source_collector: SourceText,
}

impl<'control, R> Cursor<'control, R> {
    /// Gets the current position of the cursor in the source code.
    #[inline]
    pub(super) fn pos_group(&self) -> PositionGroup {
        PositionGroup::new(self.pos, self.linear_pos())
    }

    /// Gets the current position of the cursor in the source code.
    #[inline]
    pub(super) const fn pos(&self) -> Position {
        self.pos
    }

    /// Gets the current linear position of the cursor in the source code.
    #[inline]
    pub(super) fn linear_pos(&self) -> LinearPosition {
        self.source_collector.cur_linear_position()
    }

    pub(super) fn take_source(&mut self) -> SourceText {
        let replace_with = SourceText::with_capacity(0);
        std::mem::replace(&mut self.source_collector, replace_with)
    }

    /// Advances the position to the next column.
    fn next_column(&mut self) {
        let current_line = self.pos.line_number();
        let next_column = self.pos.column_number() + 1;
        self.pos = Position::new(current_line, next_column);
    }

    /// Advances the position to the next line.
    fn next_line(&mut self) {
        let next_line = self.pos.line_number() + 1;
        self.pos = Position::new(next_line, 1);
    }

    /// Returns if strict mode is currently active.
    pub(super) const fn strict(&self) -> bool {
        self.strict
    }

    /// Sets the current strict mode.
    pub(super) fn set_strict(&mut self, strict: bool) {
        self.strict = strict;
    }

    /// Returns if the module mode is currently active.
    pub(super) const fn module(&self) -> bool {
        self.module
    }

    /// Sets the current goal symbol to module.
    pub(super) fn set_module(&mut self, module: bool) {
        self.module = module;
        self.strict = module;
    }
}

impl<'control, R: ReadChar> Cursor<'control, R> {
    /// Creates a new Lexer cursor.
    pub(super) fn new(inner: R, control: Option<&'control mut FrontendControl<'control>>) -> Self {
        Self {
            iter: inner,
            control: Control(control),
            pos: Position::new(1, 1),
            strict: false,
            module: false,
            peeked: [None; 4],
            source_collector: SourceText::default(),
        }
    }

    #[inline]
    fn read_decoded(&mut self) -> Result<Option<u32>, Error> {
        Ok(self
            .iter
            .next_char(&mut |unit| self.control.admit(FrontendWork::Read(unit)))?)
    }

    #[inline]
    pub(super) fn admit(&mut self, work: FrontendWork) -> Result<(), Error> {
        self.control.admit(work).map_err(Error::FrontendLimit)
    }

    /// Ensures up to n decoded points are available in the four-entry lookahead cache.
    pub(super) fn peek_n(&mut self, n: u8) -> Result<&[Option<u32>; 4], Error> {
        self.admit(FrontendWork::Lookahead)?;
        let cached = self.peeked.iter().filter(|point| point.is_some()).count();

        for index in cached..usize::from(n) {
            self.peeked[index] = self.read_decoded()?;
        }

        Ok(&self.peeked)
    }

    /// Peeks the next decoded point.
    pub(super) fn peek_char(&mut self) -> Result<Option<u32>, Error> {
        self.admit(FrontendWork::Lookahead)?;
        if let Some(c) = self.peeked[0] {
            return Ok(Some(c));
        }

        let next = self.read_decoded()?;
        self.peeked[0] = next;
        Ok(next)
    }

    pub(super) fn next_if(&mut self, c: u32) -> Result<bool, Error> {
        if self.peek_char()? == Some(c) {
            self.next_char()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Applies the predicate to the next character and returns the result.
    /// Returns false if the next character is not a valid ascii or there is no next character.
    /// Otherwise returns the result from the predicate on the ascii in char
    ///
    /// The buffer is not incremented.
    pub(super) fn next_is_ascii_pred<F>(&mut self, pred: &F) -> Result<bool, Error>
    where
        F: Fn(char) -> bool,
    {
        Ok(match self.peek_char()? {
            Some(byte) if (0..=0x7F).contains(&byte) =>
            {
                #[allow(clippy::cast_possible_truncation)]
                pred(char::from(byte as u8))
            }
            Some(_) | None => false,
        })
    }

    /// Fills the buffer with all bytes until the stop byte is found.
    /// Returns error when reaching the end of the buffer.
    ///
    /// Note that all bytes up until the stop byte are added to the buffer, including the byte right before.
    pub(super) fn take_until(&mut self, stop: u32, buf: &mut Vec<u32>) -> Result<(), Error> {
        loop {
            if self.next_if(stop)? {
                return Ok(());
            } else if let Some(c) = self.next_char()? {
                buf.push(c);
            } else {
                return Err(io::Error::new(
                    ErrorKind::UnexpectedEof,
                    format!("Unexpected end of file when looking for character {stop}"),
                )
                .into());
            }
        }
    }

    /// Fills a mutable slice up to the ends while characters are alphabetic. Returns
    /// the number of characters read, or `N+1` if the buffer was filled but there were
    /// still characters after.
    pub(super) fn take_array_alphabetic<const N: usize>(
        &mut self,
        arr: &mut [u32; N],
    ) -> Result<usize, Error> {
        for (i, out) in arr.iter_mut().enumerate() {
            match self.peek_char()? {
                // A..Z | a..z
                Some(0x41..=0x5A | 0x61..=0x7A) => {
                    *out = self.next_char()?.expect("Already checked.");
                }
                _ => return Ok(i),
            }
        }
        // Check the next character and return N+1 if it's alphabetic.
        match self.peek_char()? {
            // A..Z | a..z
            Some(0x41..=0x5A | 0x61..=0x7A) => Ok(N + 1),
            _ => Ok(N),
        }
    }

    /// Retains a decoded point and advances the lookahead buffer after admission.
    #[inline]
    fn consume(&mut self, point: u32) -> Result<(), Error> {
        self.source_collector
            .collect_code_point(point, |retained_units| {
                self.control.admit(FrontendWork::Consume { retained_units })
            })
            .map_err(Error::FrontendLimit)?;
        if self.peeked[0].is_some() {
            self.peeked[0] = None;
            self.peeked.rotate_left(1);
        }
        Ok(())
    }

    /// Retrieves the next decoded character.
    pub(crate) fn next_char(&mut self) -> Result<Option<u32>, Error> {
        let candidate = match self.peeked[0] {
            Some(point) => Some(point),
            None => self.read_decoded()?,
        };
        let Some(ch) = candidate else {
            return Ok(None);
        };
        if let Err(error) = self.consume(ch) {
            self.peeked[0] = Some(ch);
            return Err(error);
        }
        match ch {
            0xD => {
                self.next_line();
                // Try to take a newline if it's next, for windows "\r\n" newlines
                // Otherwise, treat as a Mac OS9 bare '\r' newline
                if self.peek_char()? == Some(0xA) {
                    self.consume(0xA)?;
                }
            }
            // '\n' | '\u{2028}' | '\u{2029}'
            0xA | 0x2028 | 0x2029 => self.next_line(),
            _ => self.next_column(),
        }

        Ok(Some(ch))
    }
}

impl<'a> From<&'a [u8]> for Cursor<'_, UTF8Input<&'a [u8]>> {
    fn from(input: &'a [u8]) -> Self {
        Self::new(UTF8Input::new(input), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        frontend::FrontendLimit,
        source::{ReadError, ReadUnit},
    };

    struct FaultAfterCharacter(Option<u32>);

    impl ReadChar for FaultAfterCharacter {
        fn next_char(
            &mut self,
            admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
        ) -> Result<Option<u32>, ReadError> {
            admit(ReadUnit::HostRead)?;
            self.0.take().map(Some).ok_or_else(|| {
                io::Error::new(ErrorKind::InvalidData, "source fault after character").into()
            })
        }
    }

    #[test]
    fn alphabetic_lookahead_propagates_a_fault_after_filling_the_buffer() {
        let mut cursor = Cursor::new(FaultAfterCharacter(Some(u32::from(b'a'))), None);
        let mut letters = [0];
        let error = cursor.take_array_alphabetic(&mut letters).unwrap_err();
        assert_eq!(letters, [u32::from(b'a')]);
        let Error::IO(error) = error else {
            panic!("expected a reader fault");
        };
        assert_eq!(error.kind(), ErrorKind::InvalidData);
        assert_eq!(error.to_string(), "source fault after character");
    }

    #[test]
    fn carriage_return_advances_the_accepted_prefix_before_a_following_read_fault() {
        let mut cursor = Cursor::new(FaultAfterCharacter(Some(u32::from(b'\r'))), None);
        let error = cursor.next_char().unwrap_err();
        assert!(matches!(error, Error::IO(_)));
        assert_eq!(cursor.linear_pos(), LinearPosition::new(1));
        assert_eq!(cursor.pos(), Position::new(2, 1));
    }

    #[test]
    fn refusing_lf_keeps_the_admitted_carriage_return_and_cached_lf() {
        let mut consumed = 0;
        let mut control = |work| {
            if matches!(work, FrontendWork::Consume { .. }) {
                consumed += 1;
                if consumed == 2 {
                    return Err(FrontendLimit::Retention(1));
                }
            }
            Ok(())
        };
        let mut cursor = Cursor::new(UTF8Input::new(&b"\r\n"[..]), Some(&mut control));
        assert!(matches!(
            cursor.next_char(),
            Err(Error::FrontendLimit(FrontendLimit::Retention(1)))
        ));
        assert_eq!(cursor.linear_pos(), LinearPosition::new(1));
        assert_eq!(cursor.pos(), Position::new(2, 1));
        assert_eq!(cursor.peeked[0], Some(u32::from(b'\n')));
    }

    #[test]
    fn shorter_lookahead_reuses_a_longer_cached_prefix() {
        let mut seen = Vec::new();
        let mut control = |work| {
            seen.push(work);
            Ok(())
        };
        {
            let mut cursor = Cursor::new(UTF8Input::new(&b"abc"[..]), Some(&mut control));
            let cached = [
                Some(u32::from(b'a')),
                Some(u32::from(b'b')),
                Some(u32::from(b'c')),
                None,
            ];
            assert_eq!(*cursor.peek_n(3).unwrap(), cached);
            assert_eq!(*cursor.peek_n(2).unwrap(), cached);
            assert_eq!(cursor.linear_pos(), LinearPosition::new(0));
        }
        assert_eq!(
            seen,
            [
                FrontendWork::Lookahead,
                FrontendWork::Read(ReadUnit::Utf8Byte),
                FrontendWork::Read(ReadUnit::Utf8Byte),
                FrontendWork::Read(ReadUnit::Utf8Byte),
                FrontendWork::Lookahead,
            ]
        );
    }

    #[test]
    fn lookahead_refusal_precedes_uncached_and_cached_inspection() {
        for prefetched in [false, true] {
            let mut admitted_probes = usize::from(prefetched);
            let mut reads = 0;
            let mut control = |work| match work {
                FrontendWork::Lookahead if admitted_probes == 0 => Err(FrontendLimit::Lookahead(0)),
                FrontendWork::Lookahead => {
                    admitted_probes -= 1;
                    Ok(())
                }
                FrontendWork::Read(_) => {
                    reads += 1;
                    Ok(())
                }
                FrontendWork::Consume { .. } | FrontendWork::LexicalDispatch => {
                    panic!("lookahead must not consume source or dispatch lexing")
                }
            };
            {
                let mut cursor = Cursor::new(UTF8Input::new(&b"abc"[..]), Some(&mut control));
                if prefetched {
                    assert_eq!(cursor.peek_n(2).unwrap()[1], Some(u32::from(b'b')));
                }
                let cached = cursor.peeked;
                assert!(matches!(
                    cursor.peek_char(),
                    Err(Error::FrontendLimit(FrontendLimit::Lookahead(0)))
                ));
                assert!(matches!(
                    cursor.peek_n(3),
                    Err(Error::FrontendLimit(FrontendLimit::Lookahead(0)))
                ));
                assert_eq!(cursor.peeked, cached);
                assert_eq!(cursor.linear_pos(), LinearPosition::new(0));
            }
            assert_eq!(reads, usize::from(prefetched) * 2);
        }
    }

    #[test]
    fn lookahead_does_not_consume_or_reread_a_refused_astral_point() {
        for prefetched in [false, true] {
            let mut seen = Vec::new();
            let mut control = |work| {
                seen.push(work);
                match work {
                    FrontendWork::Consume { .. } => Err(FrontendLimit::Retention(1)),
                    FrontendWork::Read(_) | FrontendWork::Lookahead => Ok(()),
                    FrontendWork::LexicalDispatch => panic!("cursor must not dispatch lexing"),
                }
            };
            {
                let mut cursor = Cursor::new(UTF8Input::new("😀".as_bytes()), Some(&mut control));
                if prefetched {
                    assert_eq!(cursor.peek_char().unwrap(), Some(0x1F600));
                    assert_eq!(cursor.peek_char().unwrap(), Some(0x1F600));
                }
                assert!(matches!(
                    cursor.next_char(),
                    Err(Error::FrontendLimit(FrontendLimit::Retention(1)))
                ));
                assert_eq!(cursor.peeked[0], Some(0x1F600));
                assert_eq!(cursor.linear_pos(), LinearPosition::new(0));
                assert_eq!(cursor.pos(), Position::new(1, 1));
            }
            let mut expected = Vec::new();
            if prefetched {
                expected.push(FrontendWork::Lookahead);
            }
            expected.extend([FrontendWork::Read(ReadUnit::Utf8Byte); 4]);
            if prefetched {
                expected.push(FrontendWork::Lookahead);
            }
            expected.push(FrontendWork::Consume { retained_units: 2 });
            assert_eq!(seen, expected);
        }
    }
}
