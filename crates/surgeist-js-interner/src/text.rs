use alloc::{string::String, vec::Vec};

use crate::{InternerBuffer, InternerEncoding, InternerWork};

const TEXT_CHUNK: usize = 16;
const INITIAL_TEXT_CAPACITY: usize = 64;

pub(crate) fn equal<E, F, T: PartialEq>(
    left: &[T],
    right: &[T],
    encoding: InternerEncoding,
    admit: &mut F,
) -> Result<bool, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    if left.len() != right.len() {
        return Ok(false);
    }

    for (left, right) in left.chunks(TEXT_CHUNK).zip(right.chunks(TEXT_CHUNK)) {
        admit(InternerWork::Compare {
            encoding,
            units: left.len(),
        })?;
        if left != right {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn equal_latin1<E, F>(left: &[u8], right: &[u16], admit: &mut F) -> Result<bool, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    if left.len() != right.len() {
        return Ok(false);
    }

    for (left, right) in left.chunks(TEXT_CHUNK).zip(right.chunks(TEXT_CHUNK)) {
        admit(InternerWork::Compare {
            encoding: InternerEncoding::Latin1,
            units: left.len(),
        })?;
        if !left
            .iter()
            .copied()
            .map(u16::from)
            .eq(right.iter().copied())
        {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn encode_utf16<E, F>(text: &str, admit: &mut F) -> Result<Vec<u16>, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    if text.is_empty() {
        return Ok(Vec::new());
    }

    // A valid UTF-8 string needs at most one UTF-16 unit per input byte.
    let capacity_bound = text.len().min(isize::MAX as usize / size_of::<u16>());
    let initial_capacity = capacity_bound.min(INITIAL_TEXT_CAPACITY);
    admit(InternerWork::Reserve {
        buffer: InternerBuffer::TemporaryUtf16,
        additional: initial_capacity,
        existing: 0,
    })?;
    let mut encoded = Vec::with_capacity(initial_capacity);
    let mut scalars = text.chars();
    while let Some(lead) = scalars.as_str().as_bytes().first() {
        admit(InternerWork::Convert {
            encoding: InternerEncoding::Utf8,
            units: 1,
        })?;
        let lead = *lead;
        if !lead.is_ascii() {
            // The valid string has a complete scalar at this boundary.
            let tail = if lead < 0xE0 {
                1
            } else if lead < 0xF0 {
                2
            } else {
                3
            };
            admit(InternerWork::Convert {
                encoding: InternerEncoding::Utf8,
                units: tail,
            })?;
        }
        // Advance the admitted scalar without testing the next byte as a
        // string-slice boundary.
        let scalar = scalars.next().expect("nonempty UTF-8 scalar");
        let mut buffer = [0; 2];
        let units = scalar.encode_utf16(&mut buffer);
        if units.len() > encoded.capacity() - encoded.len() {
            let additional = encoded
                .capacity()
                .min(capacity_bound - encoded.len())
                .max(units.len());
            admit(InternerWork::Reserve {
                buffer: InternerBuffer::TemporaryUtf16,
                additional,
                existing: encoded.len(),
            })?;
            encoded.reserve_exact(additional);
        }
        encoded.extend_from_slice(units);
    }
    Ok(encoded)
}

pub(crate) fn encode_utf8<E, F>(mut text: &[u16], admit: &mut F) -> Result<Option<String>, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    let initial_capacity = text.len().min(INITIAL_TEXT_CAPACITY);
    let mut encoded = String::new();
    while let Some((first, remaining)) = text.split_first() {
        admit(InternerWork::Convert {
            encoding: InternerEncoding::Utf16,
            units: 1,
        })?;
        let first = u32::from(*first);
        text = remaining;
        let code_point = if (0xD800..=0xDBFF).contains(&first) {
            let Some((second, remaining)) = text.split_first() else {
                return Ok(None);
            };
            admit(InternerWork::Convert {
                encoding: InternerEncoding::Utf16,
                units: 1,
            })?;
            let second = u32::from(*second);
            if !(0xDC00..=0xDFFF).contains(&second) {
                return Ok(None);
            }
            text = remaining;
            0x1_0000 + ((first - 0xD800) << 10) + (second - 0xDC00)
        } else {
            first
        };
        let Some(character) = char::from_u32(code_point) else {
            return Ok(None);
        };

        let width = character.len_utf8();
        if width > encoded.capacity() - encoded.len() {
            // Reserve from the initialized length. Once allocated, use the old capacity as
            // the growth increment to avoid reallocating for every character.
            let additional = initial_capacity
                .max(encoded.capacity())
                .min(isize::MAX as usize - encoded.len())
                .max(width);
            admit(InternerWork::Reserve {
                buffer: InternerBuffer::TemporaryUtf8,
                additional,
                existing: encoded.len(),
            })?;
            encoded.reserve_exact(additional);
        }
        encoded.push(character);
    }
    Ok(Some(encoded))
}

pub(crate) fn is_latin1<E, F>(text: &[u16], admit: &mut F) -> Result<bool, E>
where
    F: FnMut(InternerWork) -> Result<(), E>,
{
    for chunk in text.chunks(TEXT_CHUNK) {
        admit(InternerWork::Classify { units: chunk.len() })?;
        if !chunk.iter().all(|&unit| unit <= 0xFF) {
            return Ok(false);
        }
    }
    Ok(true)
}
