//! CSP hashes over the original Style source after JS string conversion.

use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256, Sha384, Sha512};
use surgeist_dom::DomError;

#[derive(Clone, Copy)]
pub(super) enum Algorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl Algorithm {
    pub(super) const ALL: [Self; 3] = [Self::Sha256, Self::Sha384, Self::Sha512];

    pub(super) fn prefix(self) -> &'static [u8] {
        match self {
            Self::Sha256 => b"'sha256-",
            Self::Sha384 => b"'sha384-",
            Self::Sha512 => b"'sha512-",
        }
    }
}

pub(crate) struct StyleHashes<'a> {
    source: &'a [u16],
    encoded: [Option<Encoded>; 3],
}

impl<'a> StyleHashes<'a> {
    pub(crate) fn new(source: &'a [u16]) -> Self {
        Self {
            source,
            encoded: [None; 3],
        }
    }

    pub(super) fn matches<E: From<DomError>>(
        &mut self,
        algorithm: Algorithm,
        value: &[u16],
        step: &mut impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let length = match algorithm {
            Algorithm::Sha256 => 44,
            Algorithm::Sha384 => 64,
            Algorithm::Sha512 => 88,
        };
        step()?;
        if value.len() != length {
            return Ok(false);
        }
        let slot = &mut self.encoded[algorithm as usize];
        if slot.is_none() {
            *slot = Some(match algorithm {
                Algorithm::Sha256 => encode::<Sha256, E>(self.source, step)?,
                Algorithm::Sha384 => encode::<Sha384, E>(self.source, step)?,
                Algorithm::Sha512 => encode::<Sha512, E>(self.source, step)?,
            });
        }
        let encoded = slot.as_ref().ok_or(DomError::RepresentationExhausted)?;
        let actual = encoded
            .bytes
            .get(..encoded.len)
            .filter(|actual| actual.len() == value.len())
            .ok_or(DomError::RepresentationExhausted)?;
        for (&actual, &expected) in actual.iter().zip(value) {
            step()?;
            let expected = match expected {
                0x2d => u16::from(b'+'),
                0x5f => u16::from(b'/'),
                unit => unit,
            };
            if u16::from(actual) != expected {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

#[derive(Clone, Copy)]
struct Encoded {
    bytes: [u8; 88],
    len: usize,
}

fn encode<H: Digest, E: From<DomError>>(
    source: &[u16],
    step: &mut impl FnMut() -> Result<(), E>,
) -> Result<Encoded, E> {
    let mut hash = H::new();
    let mut units = source.iter();
    while !units.as_slice().is_empty() {
        step()?;
        let Some(&unit) = units.next() else { break };
        let mut scalar = u32::from(unit);
        if (0xd800..=0xdbff).contains(&unit) && !units.as_slice().is_empty() {
            step()?;
            if let Some(&low @ 0xdc00..=0xdfff) = units.as_slice().first() {
                units.next();
                scalar = 0x10000 + ((scalar - 0xd800) << 10) + (u32::from(low) - 0xdc00);
            }
        }
        let mut buffer = [0; 4];
        let bytes = char::from_u32(scalar)
            .unwrap_or(char::REPLACEMENT_CHARACTER)
            .encode_utf8(&mut buffer)
            .as_bytes();
        for _ in bytes {
            step()?;
        }
        hash.update(bytes);
    }
    let mut bytes = [0; 88];
    let expected = <H as Digest>::output_size().div_ceil(3) * 4;
    for _ in 0..expected {
        step()?;
    }
    let digest = hash.finalize();
    let len = STANDARD
        .encode_slice(digest, &mut bytes)
        .map_err(|_| DomError::RepresentationExhausted)?;
    Ok(Encoded { bytes, len })
}
