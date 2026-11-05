use std::{
    fmt,
    io::{self, Read},
};

use super::{
    DecodeError, DecodeLimits, FontResource, LimitKind, add,
    buffer::{Budget, Buffer},
    check,
};

/// Failure to read or admit an application-supplied font stream.
#[derive(Debug)]
#[non_exhaustive]
pub enum ReadFontError {
    /// The source failed before reaching end of stream.
    Read(io::Error),
    /// The bytes or the required allocations failed resource admission.
    Decode(DecodeError),
}

impl fmt::Display for ReadFontError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read(error) => write!(f, "could not read font: {error}"),
            Self::Decode(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for ReadFontError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Read(error) => Some(error),
            Self::Decode(error) => Some(error),
        }
    }
}

impl FontResource {
    /// Reads an application-supplied stream through EOF and admits its bytes.
    ///
    /// A file opened by the caller is one possible source. The returned
    /// resource retains the bytes actually read and follows the same limits
    /// and identity contract as [Self::decode]. The reader's own storage and
    /// time spent inside its I/O implementation are caller responsibilities.
    pub fn read_from(reader: impl Read, limits: DecodeLimits) -> Result<Self, ReadFontError> {
        let bytes = read_bytes(reader, limits)?;
        Self::decode(bytes, limits).map_err(ReadFontError::Decode)
    }
}

fn read_bytes(mut reader: impl Read, limits: DecodeLimits) -> Result<Vec<u8>, ReadFontError> {
    let budget = Budget::new(limits, 0).map_err(ReadFontError::Decode)?;
    let mut bytes = Buffer::output(&budget);
    let mut scratch = [0u8; 8192];
    let max = limits.encoded_bytes();
    loop {
        // At the limit, one extra byte distinguishes EOF from an oversized
        // stream without growing the retained buffer.
        let request = scratch.len().min((max - bytes.len()).saturating_add(1));
        let read = match reader.read(&mut scratch[..request]) {
            Ok(0) => return Ok(bytes.into_vec()),
            Ok(read) if read <= request => read,
            Ok(_) => {
                return Err(ReadFontError::Read(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "font reader returned an invalid byte count",
                )));
            }
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(ReadFontError::Read(error)),
        };
        let required = add(bytes.len(), read).map_err(ReadFontError::Decode)?;
        check(LimitKind::EncodedBytes, required, max).map_err(ReadFontError::Decode)?;
        let live = budget.usage().0;
        if required > live {
            // A replacement and its predecessor coexist during growth.
            // Prefer amortized growth when the actual limits allow it.
            let preferred = required.max(live.saturating_mul(2));
            let allowed = max.min(limits.live_bytes() - live);
            bytes
                .reserve(preferred.min(allowed).max(required))
                .map_err(ReadFontError::Decode)?;
            check(LimitKind::EncodedBytes, budget.usage().0, max).map_err(ReadFontError::Decode)?;
        }
        bytes
            .extend(&scratch[..read])
            .map_err(ReadFontError::Decode)?;
    }
}
