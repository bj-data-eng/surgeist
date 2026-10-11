//! Immutable Unicode sources and validated coordinates, independent of URL context.

use crate::DocumentString;
use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// Opaque identity of one supplied source, independent of equal text or URLs.
#[derive(Clone, Debug)]
pub struct HtmlSourceId(Arc<()>);

impl PartialEq for HtmlSourceId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}
impl Eq for HtmlSourceId {}
impl Hash for HtmlSourceId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}

/// Coordinates in the caller's already-decoded input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlSourceCoordinates {
    /// Byte offsets in supplied valid UTF-8, at scalar boundaries.
    Utf8Bytes,
    /// Offsets between exact supplied UTF-16 code units.
    Utf16CodeUnits,
}

#[derive(Clone, Debug)]
enum SourceText {
    Utf8(Arc<str>),
    Utf16(DocumentString),
}

/// An immutable, independently identified already-decoded Unicode source.
///
/// A literal U+FEFF remains supplied content. This is not a byte decoder, encoding
/// sniffer, document URL, base URL, security origin or permission policy.
#[derive(Clone, Debug)]
pub struct HtmlSource {
    identity: HtmlSourceId,
    text: SourceText,
}

impl HtmlSource {
    /// Supplies valid UTF-8 with UTF-8 byte coordinates and a fresh identity.
    pub fn utf8(text: impl Into<Arc<str>>) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf8(text.into()),
        }
    }

    /// Supplies exact UTF-16 units with unit coordinates and a fresh identity.
    pub fn utf16(text: DocumentString) -> Self {
        Self {
            identity: HtmlSourceId(Arc::new(())),
            text: SourceText::Utf16(text),
        }
    }

    /// Returns the source identity; cloning the source retains this identity.
    pub fn identity(&self) -> &HtmlSourceId {
        &self.identity
    }

    /// Returns the supplied coordinate space.
    pub fn coordinates(&self) -> HtmlSourceCoordinates {
        match self.text {
            SourceText::Utf8(_) => HtmlSourceCoordinates::Utf8Bytes,
            SourceText::Utf16(_) => HtmlSourceCoordinates::Utf16CodeUnits,
        }
    }

    /// Returns the supplied length in the explicit coordinate space.
    pub fn len(&self) -> usize {
        match &self.text {
            SourceText::Utf8(text) => text.len(),
            SourceText::Utf16(text) => text.len(),
        }
    }

    /// Reports whether the supplied source contains no units.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Borrows the supplied UTF-8 when that is the source's input representation.
    pub fn as_utf8(&self) -> Option<&str> {
        match &self.text {
            SourceText::Utf8(text) => Some(text),
            SourceText::Utf16(_) => None,
        }
    }

    /// Borrows the exact UTF-16 when that is the source's input representation.
    pub fn as_utf16(&self) -> Option<&DocumentString> {
        match &self.text {
            SourceText::Utf8(_) => None,
            SourceText::Utf16(text) => Some(text),
        }
    }

    /// Validates a half-open source range in the supplied coordinate space.
    ///
    /// UTF-16 ranges may separate the units of a surrogate pair: they describe
    /// supplied units, not an assertion that the selected units are scalars.
    pub fn range(&self, start: usize, end: usize) -> Result<HtmlSourceRange, HtmlSourceRangeError> {
        if start > end {
            return Err(HtmlSourceRangeError::Reversed { start, end });
        }
        if end > self.len() {
            return Err(HtmlSourceRangeError::OutOfBounds {
                end,
                length: self.len(),
            });
        }
        if let SourceText::Utf8(text) = &self.text {
            for offset in [start, end] {
                if !text.is_char_boundary(offset) {
                    return Err(HtmlSourceRangeError::Utf8Boundary { offset });
                }
            }
        }
        Ok(HtmlSourceRange {
            source: self.identity.clone(),
            coordinates: self.coordinates(),
            start,
            end,
        })
    }

    /// Obtains exact text for a validated range of this source without normalization.
    pub fn text(&self, range: &HtmlSourceRange) -> Result<DocumentString, HtmlSourceRangeError> {
        if range.source != self.identity {
            return Err(HtmlSourceRangeError::ForeignSource);
        }
        match &self.text {
            SourceText::Utf8(text) => Ok(DocumentString::from(&text[range.start..range.end])),
            SourceText::Utf16(text) => Ok(DocumentString::from_utf16(
                &text.code_units()[range.start..range.end],
            )),
        }
    }
}

/// A validated half-open range, qualified by source identity and coordinate space.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlSourceRange {
    source: HtmlSourceId,
    coordinates: HtmlSourceCoordinates,
    start: usize,
    end: usize,
}

impl HtmlSourceRange {
    /// Returns the owning source identity.
    pub fn source(&self) -> &HtmlSourceId {
        &self.source
    }
    /// Returns the input coordinate space.
    pub fn coordinates(&self) -> HtmlSourceCoordinates {
        self.coordinates
    }
    /// Returns the inclusive starting offset.
    pub fn start(&self) -> usize {
        self.start
    }
    /// Returns the exclusive ending offset.
    pub fn end(&self) -> usize {
        self.end
    }
    /// Returns the number of input units in this range.
    pub fn len(&self) -> usize {
        self.end - self.start
    }
    /// Reports whether this is an empty range or end anchor.
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// A rejectable range or source-identity mismatch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlSourceRangeError {
    /// The supplied start followed its end.
    Reversed { start: usize, end: usize },
    /// The range ended beyond the source.
    OutOfBounds { end: usize, length: usize },
    /// An offset splits a UTF-8 scalar encoding.
    Utf8Boundary { offset: usize },
    /// The range belongs to a different source, even if its text is equal.
    ForeignSource,
}

impl fmt::Display for HtmlSourceRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Reversed { start, end } => {
                write!(f, "source range start {start} follows end {end}")
            }
            Self::OutOfBounds { end, length } => {
                write!(f, "source range end {end} exceeds length {length}")
            }
            Self::Utf8Boundary { offset } => {
                write!(f, "source offset {offset} splits a UTF-8 scalar")
            }
            Self::ForeignSource => f.write_str("range belongs to another source"),
        }
    }
}
impl Error for HtmlSourceRangeError {}
