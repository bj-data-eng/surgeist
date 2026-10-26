//! Pure en-US2005-05-30 resource observations, not lexical/Text cut authority.
//!
//! Provider: exact Hypher0.1.8 alloc+english. Source revision:
//! d81cc506416bffef2a75ee2dc969d4b41bc36613/patterns/hyph-en-us.tex.
//! The exceptions below are derived from that source's hyphenation section.
//! Copyright (C) 1990, 2004, 2005 Gerard D.C. Kuiken.
//! Copying and distribution of this file, with or without modification,
//! are permitted in any medium without royalty provided the copyright
//! notice and this notice are preserved.
//! Exact source, data notice and provider licenses are retained in
//! surgeist-text/licenses/hypher; see docs/hyphenation-resources.md.
//! The democrat correction is separately Text-owned; upstream stays unchanged.

use std::{collections::TryReserveError, fmt};

/// Derivation of this complete result; an empty exception result is still an
/// actual source override and differs from no matching exception.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Provenance {
    Pattern,
    SourceException { entry: &'static str },
    TextCorrection { entry: &'static str },
}

/// A checked interior UTF8 byte boundary with borrowed original query slices.
/// This proves no lexical, UTF16, grapheme, style, count or selection authority.
#[derive(Clone, Copy, Debug)]
pub(crate) struct ByteCut<'word> {
    byte: usize,
    before: &'word str,
    after: &'word str,
}

impl<'word> ByteCut<'word> {
    pub(crate) fn byte(self) -> usize {
        self.byte
    }
    pub(crate) fn before(self) -> &'word str {
        self.before
    }
    pub(crate) fn after(self) -> &'word str {
        self.after
    }
}

#[derive(Debug)]
pub(crate) struct Observation<'word> {
    word: &'word str,
    provenance: Provenance,
    cuts: Vec<ByteCut<'word>>,
}

impl<'word> Observation<'word> {
    pub(crate) fn word(&self) -> &'word str {
        self.word
    }
    pub(crate) fn provenance(&self) -> Provenance {
        self.provenance
    }
    pub(crate) fn cuts(&self) -> &[ByteCut<'word>] {
        &self.cuts
    }
}

/// No failed observation publishes a successful partial list. Allocation inside
/// Hypher itself retains the approved provider's ordinary alloc semantics.
#[derive(Debug)]
pub(crate) enum ObservationError {
    LengthOverflow,
    InvalidProviderSlice { start: usize, length: usize },
    IncompleteProvider { observed: usize, expected: usize },
    InvalidCut { byte: usize },
    Storage(TryReserveError),
}

impl fmt::Display for ObservationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LengthOverflow => f.write_str("resource offset is not representable"),
            Self::InvalidProviderSlice { start, length } => {
                write!(
                    f,
                    "invalid original provider slice at {start}, length {length}"
                )
            }
            Self::IncompleteProvider { observed, expected } => {
                write!(
                    f,
                    "provider covered {observed} of {expected} original bytes"
                )
            }
            Self::InvalidCut { byte } => write!(f, "invalid interior UTF8 cut at {byte}"),
            Self::Storage(error) => write!(f, "resource observation storage: {error}"),
        }
    }
}

impl std::error::Error for ObservationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

// Exact whole-word entries and UTF8 byte cuts from the pinned source. All keys
// are ASCII; eq_ignore_ascii_case preserves their byte offsets and spelling.
const EXCEPTIONS: &[(&str, &[usize])] = &[
    ("associate", &[2, 4]),
    ("associates", &[2, 4]),
    ("declination", &[3, 5, 7]),
    ("obligatory", &[5, 6]),
    ("philanthropic", &[4, 6]),
    ("present", &[]),
    ("presents", &[]),
    ("project", &[]),
    ("projects", &[]),
    ("reciprocity", &[4]),
    ("recognizance", &[2, 5, 7]),
    ("reformation", &[3, 5, 7]),
    ("retribution", &[3, 5, 7]),
    ("table", &[2]),
];

/// Query one already chosen resource. The caller owns whether this is a word
/// and whether en-US is applicable. No normalization or authored storage change.
pub(crate) fn observe(word: &str) -> Result<Observation<'_>, ObservationError> {
    // Hypher constructs its dotted byte view using word.len()+2. Check that
    // actual provider requirement, without imposing a separate word quota.
    word.len()
        .checked_add(2)
        .ok_or(ObservationError::LengthOverflow)?;
    if word.eq_ignore_ascii_case("democrat") {
        return from_offsets(
            word,
            Provenance::TextCorrection { entry: "democrat" },
            &[3, 4],
        );
    }
    if let Some(&(entry, offsets)) = EXCEPTIONS
        .iter()
        .find(|(entry, _)| word.eq_ignore_ascii_case(entry))
    {
        return from_offsets(word, Provenance::SourceException { entry }, offsets);
    }
    let mut cuts = Vec::new();
    let mut cursor = 0usize;
    for part in hypher::hyphenate_bounded(word, hypher::Lang::English, 1, 1) {
        let end = cursor
            .checked_add(part.len())
            .ok_or(ObservationError::LengthOverflow)?;
        let valid = word
            .get(cursor..end)
            .is_some_and(|original| std::ptr::eq(original, part));
        if part.is_empty() || !valid {
            return Err(ObservationError::InvalidProviderSlice {
                start: cursor,
                length: part.len(),
            });
        }
        if end < word.len() {
            push_cut(word, end, &mut cuts)?;
        }
        cursor = end;
    }
    if cursor != word.len() {
        return Err(ObservationError::IncompleteProvider {
            observed: cursor,
            expected: word.len(),
        });
    }
    Ok(Observation {
        word,
        provenance: Provenance::Pattern,
        cuts,
    })
}

fn from_offsets<'word>(
    word: &'word str,
    provenance: Provenance,
    offsets: &[usize],
) -> Result<Observation<'word>, ObservationError> {
    let mut cuts = Vec::new();
    for &byte in offsets {
        push_cut(word, byte, &mut cuts)?;
    }
    Ok(Observation {
        word,
        provenance,
        cuts,
    })
}

fn push_cut<'word>(
    word: &'word str,
    byte: usize,
    cuts: &mut Vec<ByteCut<'word>>,
) -> Result<(), ObservationError> {
    if byte == 0 || byte >= word.len() || cuts.last().is_some_and(|prior| prior.byte >= byte) {
        return Err(ObservationError::InvalidCut { byte });
    }
    let before = word
        .get(..byte)
        .ok_or(ObservationError::InvalidCut { byte })?;
    let after = word
        .get(byte..)
        .ok_or(ObservationError::InvalidCut { byte })?;
    cuts.try_reserve(1).map_err(ObservationError::Storage)?;
    cuts.push(ByteCut {
        byte,
        before,
        after,
    });
    Ok(())
}

#[cfg(test)]
#[path = "en_us/tests.rs"]
mod tests;
