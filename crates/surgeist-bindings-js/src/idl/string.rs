//! WebIDL strings after ECMAScript `ToString`, preserving UTF-16 code units.
//!
//! Implements ordinary `#js-DOMString`, `#js-USVString`, and `#js-ByteString`
//! at WebIDL revision `0620cf171d1b97138847088748e363cae4005156`.

use crate::Error;

/// An IDL DOMString, including any unmatched UTF-16 surrogates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomString(Vec<u16>);

impl DomString {
    pub(crate) fn new(units: Vec<u16>) -> Self {
        Self(units)
    }

    /// Exact UTF-16 code units, without replacement or UTF-8 conversion.
    pub fn as_code_units(&self) -> &[u16] {
        &self.0
    }

    /// Consumes the string and returns its exact UTF-16 code units.
    pub fn into_code_units(self) -> Vec<u16> {
        self.0
    }
}

/// An IDL USVString, represented by well-formed UTF-16 code units.
///
/// Each unmatched surrogate in the original DOMString becomes U+FFFD.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UsvString(Vec<u16>);

impl UsvString {
    pub(crate) fn normalize(mut units: Vec<u16>) -> Self {
        let mut remaining = units.iter_mut().peekable();
        while let Some(unit) = remaining.next() {
            match *unit {
                0xd800..=0xdbff => {
                    if remaining
                        .peek()
                        .is_some_and(|next| (0xdc00..=0xdfff).contains(&**next))
                    {
                        // A valid pair encodes one scalar and is retained verbatim.
                        let _ = remaining.next();
                    } else {
                        *unit = 0xfffd;
                    }
                }
                0xdc00..=0xdfff => *unit = 0xfffd,
                _ => {}
            }
        }
        Self(units)
    }

    /// Well-formed UTF-16 encoding of the scalar value sequence.
    pub fn as_code_units(&self) -> &[u16] {
        &self.0
    }

    /// Consumes the string and returns its well-formed UTF-16 encoding.
    pub fn into_code_units(self) -> Vec<u16> {
        self.0
    }
}

/// An IDL ByteString: one byte for each original code unit in 0..=255.
///
/// Bytes are not UTF-8 and are never truncated from larger code units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ByteString(Vec<u8>);

pub(crate) enum ByteStringStep {
    Inspect,
    Allocate(usize),
    Copy(usize),
}

impl ByteString {
    #[cfg(test)]
    pub(crate) fn convert(units: Vec<u16>) -> Result<Self, Error> {
        Self::convert_with(units, |_| Ok(()))
    }

    pub(crate) fn convert_with(
        units: Vec<u16>,
        mut before: impl FnMut(ByteStringStep) -> Result<(), Error>,
    ) -> Result<Self, Error> {
        for unit in &units {
            before(ByteStringStep::Inspect)?;
            if *unit > 255 {
                return Err(Error::ByteStringOutOfRange);
            }
        }
        before(ByteStringStep::Allocate(units.len()))?;
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(units.len())
            .map_err(|_| Error::Allocation)?;
        before(ByteStringStep::Copy(units.len()))?;
        // Validation above establishes that every conversion is lossless.
        bytes.extend(units.into_iter().map(|unit| unit as u8));
        Ok(Self(bytes))
    }

    /// Exact byte sequence, without text decoding.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Consumes the string and returns its exact bytes.
    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{ByteString, DomString, UsvString};
    use crate::Error;

    #[test]
    fn dom_string_retains_unmatched_surrogates_and_embedded_nul() {
        let units = vec![0, 0xd800, 0x61, 0xdc00, 0xd83d, 0xde00, 0xffff];
        let string = DomString::new(units.clone());
        assert_eq!(string.as_code_units(), units);
        assert_eq!(string.into_code_units(), units);
    }

    #[test]
    fn usv_string_replaces_only_unmatched_surrogates() {
        for (input, expected) in [
            (vec![], vec![]),
            (vec![0, 0x61, 0xffff], vec![0, 0x61, 0xffff]),
            (
                vec![0xd800, 0xdbff, 0xdfff, 0xdc00],
                vec![0xfffd, 0xdbff, 0xdfff, 0xfffd],
            ),
            (
                vec![0xdc00, 0xd800, 0x61, 0xdfff],
                vec![0xfffd, 0xfffd, 0x61, 0xfffd],
            ),
            (
                vec![0xd800, 0xdc00, 0xdbff, 0xdfff],
                vec![0xd800, 0xdc00, 0xdbff, 0xdfff],
            ),
        ] {
            let string = UsvString::normalize(input);
            assert_eq!(string.as_code_units(), expected);
            assert_eq!(string.into_code_units(), expected);
        }
    }

    #[test]
    fn byte_string_preserves_bytes_and_rejects_larger_code_units() -> Result<(), Error> {
        let string = ByteString::convert(vec![0, 0x7f, 0x80, 0xff])?;
        assert_eq!(string.as_bytes(), [0, 0x7f, 0x80, 0xff]);
        assert_eq!(string.into_bytes(), [0, 0x7f, 0x80, 0xff]);
        assert!(ByteString::convert(vec![])?.as_bytes().is_empty());
        for input in [
            vec![0x100],
            vec![0xd800],
            vec![0xd83d, 0xde00],
            vec![0, 0xffff],
        ] {
            assert!(matches!(
                ByteString::convert(input),
                Err(Error::ByteStringOutOfRange)
            ));
        }
        Ok(())
    }
}
