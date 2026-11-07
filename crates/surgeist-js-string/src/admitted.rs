//! Optional caller admission for String-owned primitive work.

use crate::{
    JsStr, JsStrVariant, JsString, StaticJsStrings, is_trimmable_whitespace, storage::Units,
};
use std::{cmp::Ordering, collections::TryReserveError, hash::Hasher, mem::size_of};

const BLOCK: usize = 256;

/// Work admitted before the string producer performs it.
///
/// Units count logical reads, writes, and requested payload bytes, not CPU
/// instructions, allocator capacity, or physical peak memory. A refusal stops
/// before the corresponding work; already accepted work is not refunded.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StringWork {
    /// Inspect up to `units` source units (unit pairs for comparisons,
    /// code units otherwise, or bytes for UTF-8 text).
    Scan {
        /// Number of units to inspect.
        units: usize,
    },
    /// Write `units` destination code units (or bytes for UTF-8 text).
    Copy {
        /// Number of units to write.
        units: usize,
    },
    /// Request a buffer or retained descriptor payload of `bytes` bytes.
    Allocate {
        /// Requested logical payload bytes, excluding allocator overhead.
        bytes: usize,
    },
    /// Reserve input work before the opaque decimal-number provider.
    ///
    /// The price is the complete UTF-8 input byte length. Locked fast-float2
    /// 0.2.4 `parse.rs::parse_float` selects fast/binary or long-mantissa paths;
    /// `number.rs` and `decimal.rs::parse_decimal` scan the input, and
    /// `simple.rs::parse_long_mantissa` uses the fixed 768-digit Decimal buffer.
    /// This input-linear native tariff covers every selected branch as a
    /// request price, not a certified count of provider instructions. The
    /// provider has no admission or cancellation hooks: no mid-call stop,
    /// allocation-success, physical-memory, or wall-clock guarantee follows.
    FastFloat {
        /// UTF-8 input bytes reserved before provider entry.
        bytes: usize,
    },
    /// A zero-work cancellation observation, including after opaque work.
    Checkpoint,
}

impl StringWork {
    /// Fixed diagnostic origin and deterministic logical request units.
    #[must_use]
    pub const fn origin_and_units(self) -> (&'static str, u64) {
        match self {
            Self::Scan { units } => ("string-scan", units as u64),
            Self::Copy { units } => ("string-copy", units as u64),
            Self::Allocate { bytes } => ("string-allocate", bytes as u64),
            Self::FastFloat { bytes } => ("string-fast-float", bytes as u64),
            Self::Checkpoint => ("string-checkpoint", 0),
        }
    }
}

/// Failure from an admitted string operation.
#[derive(Debug)]
pub enum StringError<E> {
    /// The caller refused the next work request.
    Refused(E),
    /// A logical extent cannot be represented by the destination allocation.
    SizeOverflow,
    /// A fallible buffer reservation failed.
    ///
    /// Reference-counted descriptors and standard-library shrinking remain
    /// infallible allocation boundaries; this is not a complete OOM guarantee.
    Allocation(TryReserveError),
}

fn request<E>(
    admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    work: StringWork,
) -> Result<(), StringError<E>> {
    admit(work).map_err(StringError::Refused)
}

fn reserve<T, E>(
    units: usize,
    admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
) -> Result<Vec<T>, StringError<E>> {
    let bytes = units
        .checked_mul(size_of::<T>())
        .filter(|n| *n <= isize::MAX as usize)
        .ok_or(StringError::SizeOverflow)?;
    request(admit, StringWork::Allocate { bytes })?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(units)
        .map_err(StringError::Allocation)?;
    Ok(result)
}

fn finish<E>(
    units: Units,
    admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
) -> Result<JsString, StringError<E>> {
    request(
        admit,
        StringWork::Allocate {
            bytes: size_of::<Units>(),
        },
    )?;
    Ok(JsString::from_units(units))
}

fn box_units<T, E>(
    units: Vec<T>,
    admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
) -> Result<Box<[T]>, StringError<E>> {
    // into_boxed_slice may shrink an allocator-overallocated reservation.
    if units.capacity() != units.len() {
        let bytes = units
            .len()
            .checked_mul(size_of::<T>())
            .ok_or(StringError::SizeOverflow)?;
        request(admit, StringWork::Allocate { bytes })?;
        for block in units.chunks(BLOCK) {
            request(admit, StringWork::Copy { units: block.len() })?;
        }
    }
    Ok(units.into_boxed_slice())
}

impl JsString {
    /// Own borrowed JavaScript code units with optional caller admission.
    /// Encoding and unpaired surrogates are preserved exactly. No static lookup
    /// or consumer-side conversion is needed.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_from_str<E>(
        string: JsStr<'_>,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Self, StringError<E>> {
        Self::try_concat(string, JsStr::EMPTY, admit)
    }

    /// Concatenate JavaScript code units, widening Latin1 only when necessary.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_concat<E>(
        x: JsStr<'_>,
        y: JsStr<'_>,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Self, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let len = x
            .len()
            .checked_add(y.len())
            .ok_or(StringError::SizeOverflow)?;
        if len == 0 {
            return Ok(StaticJsStrings::EMPTY_STRING);
        }
        if x.is_latin1() && y.is_latin1() {
            let mut result = reserve::<u8, _>(len, admit)?;
            for string in [x, y] {
                if let JsStrVariant::Latin1(source) = string.variant() {
                    for block in source.chunks(BLOCK) {
                        request(admit, StringWork::Copy { units: block.len() })?;
                        result.extend_from_slice(block);
                    }
                }
            }
            finish(Units::Latin1(box_units(result, admit)?), admit)
        } else {
            let mut result = reserve::<u16, _>(len, admit)?;
            for string in [x, y] {
                match string.variant() {
                    JsStrVariant::Latin1(source) => {
                        for block in source.chunks(BLOCK) {
                            request(admit, StringWork::Copy { units: block.len() })?;
                            result.extend(block.iter().copied().map(u16::from));
                        }
                    }
                    JsStrVariant::Utf16(source) => {
                        for block in source.chunks(BLOCK) {
                            request(admit, StringWork::Copy { units: block.len() })?;
                            result.extend_from_slice(block);
                        }
                    }
                }
            }
            finish(Units::Utf16(box_units(result, admit)?), admit)
        }
    }

    /// Format a Number using the same ECMAScript scalar provider as `From<f64>`.
    ///
    /// # Errors
    /// Returns refusal or allocation failure.
    pub fn try_from_number<E>(
        value: f64,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Self, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let mut buffer = ryu_js::Buffer::new();
        let text = buffer.format(value);
        request(admit, StringWork::Checkpoint)?;
        Self::try_from_str(JsStr::latin1(text.as_bytes()), admit)
    }

    /// Format an integer using the same fixed scalar provider as `From<i32>`.
    ///
    /// # Errors
    /// Returns refusal or allocation failure.
    pub fn try_from_integer<E>(
        value: i32,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Self, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let mut buffer = itoa::Buffer::new();
        let text = buffer.format(value);
        request(admit, StringWork::Checkpoint)?;
        Self::try_from_str(JsStr::latin1(text.as_bytes()), admit)
    }

    /// Own UTF-8 text as JavaScript UTF-16 code units.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_from_utf8<E>(
        text: &str,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Self, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        if text.is_empty() {
            return Ok(StaticJsStrings::EMPTY_STRING);
        }
        let mut count = 0usize;
        let mut cursor = text;
        while !cursor.is_empty() {
            request(
                admit,
                StringWork::Scan {
                    units: cursor.len().min(4),
                },
            )?;
            let Some(c) = cursor.chars().next() else {
                break;
            };
            count = count
                .checked_add(c.len_utf16())
                .ok_or(StringError::SizeOverflow)?;
            cursor = &cursor[c.len_utf8()..];
        }
        let mut units = reserve::<u16, _>(count, admit)?;
        let mut cursor = text;
        while !cursor.is_empty() {
            request(
                admit,
                StringWork::Scan {
                    units: cursor.len().min(4),
                },
            )?;
            let Some(c) = cursor.chars().next() else {
                break;
            };
            request(
                admit,
                StringWork::Copy {
                    units: c.len_utf16(),
                },
            )?;
            let mut buffer = [0; 2];
            units.extend_from_slice(c.encode_utf16(&mut buffer));
            cursor = &cursor[c.len_utf8()..];
        }
        finish(Units::Utf16(box_units(units, admit)?), admit)
    }
}

impl JsStr<'_> {
    /// Hash exact JavaScript code units with optional caller admission.
    ///
    /// This preserves the ordinary `Hash` write sequence: `write_usize(len)`
    /// followed by one `write_u16` per code unit, including Latin1 widening and
    /// unpaired surrogates. The initial checkpoint precedes length metadata and
    /// its hasher write; each admitted scan block covers at most 256 source
    /// reads and corresponding code-unit writes. A final checkpoint observes
    /// cancellation after the accepted work.
    ///
    /// Arbitrary `Hasher` implementation work remains caller-owned. Accepted
    /// hasher writes are not rolled back after a later refusal, including a
    /// refusal at the final checkpoint. Logical scan admission provides no
    /// physical allocator or elapsed-time guarantee.
    ///
    /// # Errors
    /// Returns the caller's refusal before the next block or at a checkpoint.
    pub fn try_hash<H: Hasher, E>(
        self,
        state: &mut H,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<(), StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        state.write_usize(self.len());
        match self.variant() {
            JsStrVariant::Latin1(source) => {
                for block in source.chunks(BLOCK) {
                    request(admit, StringWork::Scan { units: block.len() })?;
                    for &unit in block {
                        state.write_u16(u16::from(unit));
                    }
                }
            }
            JsStrVariant::Utf16(source) => {
                for block in source.chunks(BLOCK) {
                    request(admit, StringWork::Scan { units: block.len() })?;
                    for &unit in block {
                        state.write_u16(unit);
                    }
                }
            }
        }
        request(admit, StringWork::Checkpoint)
    }

    /// Export exact UTF-16 code units, widening Latin1 without Unicode decoding.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_to_vec<E>(
        &self,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Vec<u16>, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let mut result = reserve::<u16, _>(self.len(), admit)?;
        for start in (0..self.len()).step_by(BLOCK) {
            let end = self.len().min(start.saturating_add(BLOCK));
            request(admit, StringWork::Copy { units: end - start })?;
            result.extend(self.get_expect(start..end).iter());
        }
        Ok(result)
    }

    /// Compare UTF-16 code units lexicographically, including unpaired surrogates.
    ///
    /// # Errors
    /// Returns the caller's refusal before reading the next comparison block.
    pub fn try_cmp<E>(
        &self,
        other: JsStr<'_>,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Ordering, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let common = self.len().min(other.len());
        for start in (0..common).step_by(BLOCK) {
            let end = common.min(start.saturating_add(BLOCK));
            request(admit, StringWork::Scan { units: end - start })?;
            let order = self
                .get_expect(start..end)
                .cmp(&other.get_expect(start..end));
            if order != Ordering::Equal {
                return Ok(order);
            }
        }
        Ok(self.len().cmp(&other.len()))
    }

    /// Test exact code-unit equality across both storage encodings.
    ///
    /// # Errors
    /// Returns the caller's refusal before reading the next comparison block.
    pub fn try_eq<E>(
        &self,
        other: JsStr<'_>,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<bool, StringError<E>> {
        if self.len() != other.len() {
            request(admit, StringWork::Checkpoint)?;
            return Ok(false);
        }
        self.try_cmp(other, admit)
            .map(|order| order == Ordering::Equal)
    }

    /// Decode Unicode text, returning `None` for an unpaired surrogate.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_to_std_string<E>(
        &self,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<Option<String>, StringError<E>> {
        request(admit, StringWork::Checkpoint)?;
        let bound = self
            .len()
            .checked_mul(3)
            .filter(|n| *n <= isize::MAX as usize)
            .ok_or(StringError::SizeOverflow)?;
        request(admit, StringWork::Allocate { bytes: bound })?;
        let mut result = String::new();
        result
            .try_reserve_exact(bound)
            .map_err(StringError::Allocation)?;
        let mut input = self.iter();
        while let Some(unit) = {
            // Admit before the iterator reads its first unit, including invalid input.
            if input.len() != 0 {
                request(admit, StringWork::Scan { units: 1 })?;
            }
            input.next()
        } {
            let c = if (0xD800..=0xDBFF).contains(&unit) {
                if input.len() == 0 {
                    return Ok(None);
                }
                request(admit, StringWork::Scan { units: 1 })?;
                let Some(low @ 0xDC00..=0xDFFF) = input.next() else {
                    return Ok(None);
                };
                let point = 0x10000 + ((u32::from(unit) - 0xD800) << 10) + u32::from(low) - 0xDC00;
                let Some(c) = char::from_u32(point) else {
                    return Ok(None);
                };
                c
            } else {
                let Some(c) = char::from_u32(u32::from(unit)) else {
                    return Ok(None);
                };
                c
            };
            request(
                admit,
                StringWork::Copy {
                    units: c.len_utf8(),
                },
            )?;
            result.push(c);
        }
        Ok(Some(result))
    }

    /// Convert a String to a Number with producer-controlled admission.
    ///
    /// # Errors
    /// Returns refusal, unrepresentable size, or buffer-reservation failure.
    pub fn try_to_number<E>(
        &self,
        admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
    ) -> Result<f64, StringError<E>> {
        let Some(text) = self.try_to_std_string(admit)? else {
            return Ok(f64::NAN);
        };
        number_text(&text, admit)
    }
}

/// Shared StringNumericValue kernel: ordinary and admitted callers use the
/// identical whitespace, radix and decimal semantics.
pub(crate) fn number_text<E>(
    mut text: &str,
    admit: &mut (impl FnMut(StringWork) -> Result<(), E> + ?Sized),
) -> Result<f64, StringError<E>> {
    while !text.is_empty() {
        request(
            admit,
            StringWork::Scan {
                units: text.len().min(4),
            },
        )?;
        let Some(c) = text.chars().next() else {
            break;
        };
        if !is_trimmable_whitespace(c) {
            break;
        }
        text = &text[c.len_utf8()..];
    }
    while !text.is_empty() {
        request(
            admit,
            StringWork::Scan {
                units: text.len().min(4),
            },
        )?;
        let Some(c) = text.chars().next_back() else {
            break;
        };
        if !is_trimmable_whitespace(c) {
            break;
        }
        text = &text[..text.len() - c.len_utf8()];
    }
    request(
        admit,
        StringWork::Scan {
            units: text.len().min(9),
        },
    )?;
    match text {
        "" => return Ok(0.0),
        "-Infinity" => return Ok(f64::NEG_INFINITY),
        "Infinity" | "+Infinity" => return Ok(f64::INFINITY),
        _ => {}
    }
    let mut bytes = text.bytes();
    let radix = match (bytes.next(), bytes.next()) {
        (Some(b'0'), Some(b'b' | b'B')) => Some(2),
        (Some(b'0'), Some(b'o' | b'O')) => Some(8),
        (Some(b'0'), Some(b'x' | b'X')) => Some(16),
        (Some(b'i' | b'I'), _) => return Ok(f64::NAN),
        _ => None,
    };
    if let Some(radix) = radix {
        if text.len() == 2 {
            return Ok(f64::NAN);
        }
        // Preserve the ordinary u32 parser's optional leading plus, including
        // its overflow fallback to the original (unstripped) spelling.
        let original = &text.as_bytes()[2..];
        let digits = original.strip_prefix(b"+").unwrap_or(original);
        if digits.is_empty() {
            return Ok(f64::NAN);
        }
        let mut integer = 0u32;
        let mut overflow = false;
        'integer: for block in digits.chunks(BLOCK) {
            request(admit, StringWork::Scan { units: block.len() })?;
            for &byte in block {
                let Some(digit) = char::from(byte).to_digit(radix) else {
                    return Ok(f64::NAN);
                };
                let Some(next) = integer
                    .checked_mul(radix)
                    .and_then(|n| n.checked_add(digit))
                else {
                    overflow = true;
                    break 'integer;
                };
                integer = next;
            }
        }
        if !overflow {
            return Ok(f64::from(integer));
        }
        let mut value = 0.0f64;
        for block in original.chunks(BLOCK) {
            request(admit, StringWork::Scan { units: block.len() })?;
            for &byte in block {
                let Some(digit) = char::from(byte).to_digit(radix) else {
                    return Ok(f64::NAN);
                };
                value = value.mul_add(f64::from(radix), f64::from(digit));
            }
        }
        return Ok(value);
    }
    request(admit, StringWork::FastFloat { bytes: text.len() })?;
    let value = fast_float2::parse(text).unwrap_or(f64::NAN);
    request(admit, StringWork::Checkpoint)?;
    Ok(value)
}

#[cfg(test)]
mod hash_tests {
    use super::*;
    use std::{cell::Cell, hash::Hash, rc::Rc};

    #[derive(Debug, PartialEq, Eq)]
    enum Write {
        Length(usize),
        Unit(u16),
    }

    #[derive(Default)]
    struct RecordingHasher {
        writes: Vec<Write>,
        refused: Rc<Cell<bool>>,
    }

    impl Hasher for RecordingHasher {
        fn finish(&self) -> u64 {
            0
        }
        fn write(&mut self, _: &[u8]) {
            panic!("canonical JsStr hashing uses typed writes");
        }
        fn write_usize(&mut self, value: usize) {
            assert!(!self.refused.get(), "hasher write after refusal");
            self.writes.push(Write::Length(value));
        }
        fn write_u16(&mut self, value: u16) {
            assert!(!self.refused.get(), "hasher write after refusal");
            self.writes.push(Write::Unit(value));
        }
    }

    #[test]
    fn admitted_hash_preserves_canonical_writes_across_storage_encodings() {
        let latin1 = [0, 97, 127, 128, 255];
        let utf16 = [0, 97, 127, 128, 255];
        let expected = vec![
            Write::Length(5),
            Write::Unit(0),
            Write::Unit(97),
            Write::Unit(127),
            Write::Unit(128),
            Write::Unit(255),
        ];
        for text in [JsStr::latin1(&latin1), JsStr::utf16(&utf16)] {
            let mut admitted = RecordingHasher::default();
            text.try_hash(&mut admitted, &mut |_| Ok::<_, ()>(()))
                .unwrap();
            assert_eq!(admitted.writes, expected);
            let mut ordinary = RecordingHasher::default();
            text.hash(&mut ordinary);
            assert_eq!(ordinary.writes, expected);
        }
    }

    #[test]
    fn admitted_hash_preserves_unpaired_surrogates_and_empty_length_prefix() {
        for (text, expected) in [
            (
                JsStr::utf16(&[0xD800, 97, 0xDC00, 0xDFFF, 0xD800]),
                vec![
                    Write::Length(5),
                    Write::Unit(0xD800),
                    Write::Unit(97),
                    Write::Unit(0xDC00),
                    Write::Unit(0xDFFF),
                    Write::Unit(0xD800),
                ],
            ),
            (JsStr::latin1(&[]), vec![Write::Length(0)]),
            (JsStr::utf16(&[]), vec![Write::Length(0)]),
        ] {
            let mut state = RecordingHasher::default();
            let mut checkpoints = 0;
            text.try_hash(&mut state, &mut |work| {
                if work == StringWork::Checkpoint {
                    checkpoints += 1;
                }
                Ok::<_, ()>(())
            })
            .unwrap();
            assert_eq!(state.writes, expected);
            assert_eq!(checkpoints, 2);
        }
    }

    #[test]
    fn admitted_hash_exact_and_one_short_limits_bound_accepted_prefixes() {
        for length in [0, 1, 255, 256, 257, 512, 513, 4097] {
            let units: Vec<u16> = [0, 255, 0xD800, 0xDC00]
                .into_iter()
                .cycle()
                .take(length)
                .collect();
            for limit in [length, length.saturating_sub(1)] {
                let mut state = RecordingHasher::default();
                let refused = state.refused.clone();
                let mut accepted = 0;
                let result = JsStr::utf16(&units).try_hash(&mut state, &mut |work| {
                    if let StringWork::Scan { units } = work {
                        assert!((1..=256).contains(&units));
                        if units > limit - accepted {
                            refused.set(true);
                            return Err("scan allowance");
                        }
                        accepted += units;
                    }
                    Ok(())
                });
                let prefix = if limit == length {
                    length
                } else {
                    limit / 256 * 256
                };
                assert_eq!(accepted, prefix);
                assert_eq!(state.writes.first(), Some(&Write::Length(length)));
                assert_eq!(state.writes.len(), prefix + 1);
                assert_eq!(
                    &state.writes[1..],
                    units[..prefix]
                        .iter()
                        .copied()
                        .map(Write::Unit)
                        .collect::<Vec<_>>()
                );
                if limit == length {
                    assert!(result.is_ok());
                } else {
                    assert!(matches!(
                        result,
                        Err(StringError::Refused("scan allowance"))
                    ));
                }
            }
        }
    }

    #[test]
    fn admitted_hash_stops_on_each_checkpoint_and_block_refusal() {
        let units = [0xD800; 600];
        // Initial checkpoint, three scan blocks, final checkpoint.
        for (denied, written_units) in [None, Some(0), Some(256), Some(512), Some(600)]
            .into_iter()
            .enumerate()
        {
            let mut state = RecordingHasher::default();
            let refused = state.refused.clone();
            let mut calls = 0;
            let result = JsStr::utf16(&units).try_hash(&mut state, &mut |_| {
                let current = calls;
                calls += 1;
                if current == denied {
                    refused.set(true);
                    return Err(denied);
                }
                Ok(())
            });
            assert!(matches!(result, Err(StringError::Refused(index)) if index == denied));
            assert_eq!(calls, denied + 1);
            assert_eq!(
                state.writes.len(),
                written_units.map_or(0, |units| units + 1)
            );
        }
    }
}
