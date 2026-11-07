use std::{
    fmt,
    iter::{Fuse, Peekable},
};

use super::{ReadChar, ReadError, ReadUnit};
use crate::frontend::FrontendLimit;

/// Input for UTF-16 encoded slices.
#[derive(Debug)]
pub struct UTF16Input<'a> {
    input: &'a [u16],
    index: usize,
}

impl<'a> UTF16Input<'a> {
    /// Creates a new `UTF16Input` from a UTF-16 encoded slice e.g. <code>[&\[u16\]][slice]</code>.
    ///
    /// [slice]: std::slice
    #[must_use]
    pub const fn new(input: &'a [u16]) -> Self {
        Self { input, index: 0 }
    }
}

impl Units for UTF16Input<'_> {
    fn next_unit(&mut self) -> Option<u16> {
        let unit = self.peek_unit()?;
        self.index += 1;
        Some(unit)
    }

    fn peek_unit(&mut self) -> Option<u16> {
        self.input.get(self.index).copied()
    }

    fn consume_peeked_unit(&mut self) {
        self.index += 1;
    }
}

impl ReadChar for UTF16Input<'_> {
    fn next_char(
        &mut self,
        admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
    ) -> Result<Option<u32>, ReadError> {
        read_code_point(self, admit)
    }
}

/// Input that pulls code units from a borrowed UTF-16 stream.
pub struct UTF16StreamInput<'a> {
    input: Peekable<Fuse<&'a mut dyn Iterator<Item = u16>>>,
}

impl<'a> UTF16StreamInput<'a> {
    pub(super) fn new(input: &'a mut dyn Iterator<Item = u16>) -> Self {
        Self {
            input: input.fuse().peekable(),
        }
    }
}

impl fmt::Debug for UTF16StreamInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("UTF16StreamInput")
            .finish_non_exhaustive()
    }
}

impl Units for UTF16StreamInput<'_> {
    fn next_unit(&mut self) -> Option<u16> {
        self.input.next()
    }

    fn peek_unit(&mut self) -> Option<u16> {
        self.input.peek().copied()
    }

    fn consume_peeked_unit(&mut self) {
        self.input.next();
    }
}

impl ReadChar for UTF16StreamInput<'_> {
    fn next_char(
        &mut self,
        admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
    ) -> Result<Option<u32>, ReadError> {
        read_code_point(self, admit)
    }
}

trait Units {
    fn next_unit(&mut self) -> Option<u16>;
    fn peek_unit(&mut self) -> Option<u16>;
    fn consume_peeked_unit(&mut self);
}

fn read_code_point(
    units: &mut impl Units,
    admit: &mut impl FnMut(ReadUnit) -> Result<(), FrontendLimit>,
) -> Result<Option<u32>, ReadError> {
    admit(ReadUnit::Utf16Unit)?;
    let Some(first) = units.next_unit() else {
        return Ok(None);
    };

    if !is_high_surrogate(first) {
        return Ok(Some(first.into()));
    }

    admit(ReadUnit::Utf16Unit)?;
    Ok(Some(handle_surrogate_pair(units, first)))
}

// Surrogate pairs are rare.
#[cold]
fn handle_surrogate_pair(units: &mut impl Units, first: u16) -> u32 {
    let Some(second) = units.peek_unit().filter(|&unit| is_low_surrogate(unit)) else {
        return first.into();
    };
    units.consume_peeked_unit();
    code_point_from_surrogates(first, second)
}

const SURROGATE_HIGH_START: u16 = 0xD800;
const SURROGATE_HIGH_END: u16 = 0xDBFF;
const SURROGATE_LOW_START: u16 = 0xDC00;
const SURROGATE_LOW_END: u16 = 0xDFFF;

fn is_high_surrogate(b: u16) -> bool {
    (SURROGATE_HIGH_START..=SURROGATE_HIGH_END).contains(&b)
}

fn is_low_surrogate(b: u16) -> bool {
    (SURROGATE_LOW_START..=SURROGATE_LOW_END).contains(&b)
}

fn code_point_from_surrogates(high: u16, low: u16) -> u32 {
    (((u32::from(high & 0x3ff)) << 10) | u32::from(low & 0x3ff)) + 0x1_0000
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, iter};

    use super::*;
    use crate::Source;

    #[test]
    fn a_stream_is_advanced_only_after_unit_admission() {
        for limit in [0, 1] {
            let pulls = Cell::new(0);
            let mut data = [0xD83D, 0xDE00].into_iter();
            let mut units = iter::from_fn(|| {
                pulls.set(pulls.get() + 1);
                data.next()
            });
            let mut input = Source::from_utf16_units(&mut units).reader;
            let mut admitted = 0;
            let result = input.next_char(&mut |unit| {
                assert_eq!(unit, ReadUnit::Utf16Unit);
                assert_eq!(pulls.get(), admitted);
                if admitted == limit {
                    return Err(FrontendLimit::Read(limit));
                }
                admitted += 1;
                Ok(())
            });
            assert!(
                matches!(result, Err(ReadError::Limit(FrontendLimit::Read(cap))) if cap == limit)
            );
            assert_eq!(pulls.get(), limit);
        }
    }

    #[test]
    fn a_stream_preserves_surrogates_and_reuses_the_peeked_unit() {
        let pulls = Cell::new(0);
        let mut data = [0xD83D].into_iter().chain([0xDE00, 0xD800, 0x61]);
        let mut units = iter::from_fn(|| {
            pulls.set(pulls.get() + 1);
            data.next()
        });
        let mut input = Source::from_utf16_units(&mut units).reader;
        let mut attempts = 0;
        for expected in [Some(0x1F600), Some(0xD800), Some(0x61), None] {
            assert_eq!(
                input
                    .next_char(&mut |unit| {
                        assert_eq!(unit, ReadUnit::Utf16Unit);
                        attempts += 1;
                        Ok(())
                    })
                    .unwrap(),
                expected
            );
        }
        // The high surrogate and following non-surrogate each inspect the same
        // unit. The stream pulls it once and keeps it for the next point.
        assert_eq!(attempts, 6);
        assert_eq!(pulls.get(), 5);
    }
}
