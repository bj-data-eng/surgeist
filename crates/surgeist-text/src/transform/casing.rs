//! Contextual mappings adapted from ICU4X 2.2.0 `internals.rs` and
//! `greek_to_me`. Copyright Unicode, Inc.; Unicode-3.0, retained at
//! `licenses/icu4x/LICENSE`. Text uses the public paired baked trie and checked
//! exception decoder. It owns traversal and records each result at the actual
//! immutable input row, rather than reconstructing edits from an output string.

use icu_casemap::provider::data::{CaseMapDataKind, CaseType, DotType};
use icu_casemap::provider::{Baked, CaseMapV1};
use icu_provider::{DataPayload, DataProvider};
use std::{borrow::Cow, ops::Range};

use super::greek;

pub(super) const CAPITAL_SIGMA: u32 = 0x3a3;
const FINAL_SIGMA: u32 = 0x3c2;

/// Complete locale-independent alternatives of the native lower-Sigma rule.
/// This reads canonical case data; it does not choose a contextual mapping or
/// publish an emission for a component which the request does not own.
pub(super) fn lower_sigma_family() -> Result<[u32; 2], MappingError> {
    let mapper = Mapper::new()?;
    let ordinary = match mapper.ordinary(CAPITAL_SIGMA, Operation::Lower)? {
        Mapping::Scalar(cp) => cp,
        Mapping::Text(text) => {
            let mut chars = text.chars();
            let first = chars.next().ok_or(MappingError::InvalidException)?;
            if chars.next().is_some() {
                return Err(MappingError::InvalidException);
            }
            u32::from(first)
        }
        _ => return Err(MappingError::InvalidException),
    };
    Ok([ordinary, FINAL_SIGMA])
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Operation {
    Lower,
    Upper,
    /// Full title mapping of the selected initial component.
    Title,
    /// Context effects on a later component of the selected initial unit.
    /// Ordinary full mappings do not apply to the unchanged title tail.
    TitleContext,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Locale {
    Root,
    Turkic,
    Lithuanian,
    Greek,
    Dutch,
    Armenian,
    Irish,
}

impl Locale {
    /// Resolve only the actual authored content language. The version-paired
    /// extended likely-subtag data supplies an omitted script, preserving any
    /// explicit script and region; unknown language never becomes English.
    pub(super) fn from_tag(tag: Option<&str>) -> Result<Self, MappingError> {
        let Some(tag) = tag else {
            return Ok(Self::Root);
        };
        let locale: icu_locale::Locale = tag.parse().map_err(|_| MappingError::InvalidLocale)?;
        let mut id = locale.id;
        if id.language.as_str() == "und" {
            return Ok(Self::Root);
        }
        icu_locale::LocaleExpander::new_extended().maximize(&mut id);
        let language = id.language.as_str();
        Ok(
            match id
                .script
                .map(|script| script.as_str().to_owned())
                .as_deref()
            {
                Some("Latn") => match language {
                    "tr" | "az" | "ba" | "crh" | "tt" => Self::Turkic,
                    "lt" => Self::Lithuanian,
                    "nl" => Self::Dutch,
                    "ga" => Self::Irish,
                    _ => Self::Root,
                },
                // For a known language with an unfamiliar combination, Appendix F
                // requires the declared writing system's conventions.
                Some("Grek") => Self::Greek,
                Some("Armn") => Self::Armenian,
                _ => Self::Root,
            },
        )
    }
}

#[derive(Debug)]
pub(super) enum MappingError {
    DataUnavailable,
    InvalidException,
    InvalidScalar,
    InvalidLocale,
    Storage,
    Context(super::context::MissingEdge),
    Source(crate::Error),
}

/// Immutable canonical values with unavailable relationships at real component
/// cuts. Readers check only cuts actually traversed by their native predicate.
pub(super) enum Input<'a> {
    Known {
        values: &'a [u32],
        missing: &'a [(usize, super::context::MissingEdge)],
    },
    Source {
        length: usize,
        value: &'a dyn Fn(usize) -> Result<Option<u32>, MappingError>,
        crossing: &'a dyn Fn(usize, bool) -> Result<bool, MappingError>,
    },
}
impl Input<'_> {
    fn len(&self) -> usize {
        match self {
            Self::Known { values, .. } => values.len(),
            Self::Source { length, .. } => *length,
        }
    }
    fn value(&self, at: usize) -> Result<Option<u32>, MappingError> {
        match self {
            Self::Known { values, .. } => Ok(values.get(at).copied()),
            Self::Source { value, .. } => value(at),
        }
    }
    fn crossing(&self, cut: usize, reverse: bool) -> Result<bool, MappingError> {
        match self {
            Self::Known { missing, .. } => {
                if let Some((_, edge)) = missing.iter().find(|(at, _)| *at == cut) {
                    return Err(MappingError::Context(*edge));
                }
                Ok(false)
            }
            Self::Source { crossing, .. } => crossing(cut, reverse),
        }
    }
    pub(super) fn before(&self, at: usize) -> Cursor<'_> {
        Cursor {
            input: self,
            position: at,
            reverse: true,
        }
    }
    pub(super) fn after(&self, at: usize) -> Cursor<'_> {
        Cursor {
            input: self,
            position: at + 1,
            reverse: false,
        }
    }
}
#[derive(Clone, Copy)]
pub(super) struct Cursor<'a> {
    input: &'a Input<'a>,
    position: usize,
    reverse: bool,
}
impl Iterator for Cursor<'_> {
    type Item = Result<u32, MappingError>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (row, cut) = if self.reverse {
                (self.position.checked_sub(1)?, self.position)
            } else {
                if self.position >= self.input.len() {
                    return None;
                }
                (self.position, self.position)
            };
            match self.input.crossing(cut, self.reverse) {
                Ok(true) => {
                    self.position = if self.reverse { 0 } else { self.input.len() };
                    return None;
                }
                Err(error) => {
                    self.position = if self.reverse { 0 } else { self.input.len() };
                    return Some(Err(error));
                }
                Ok(false) => (),
            }
            self.position = if self.reverse { row } else { row + 1 };
            match self.input.value(row) {
                Ok(Some(cp)) => return Some(Ok(cp)),
                Ok(None) => (),
                Err(error) => return Some(Err(error)),
            }
        }
    }
}

enum Mapping<'a> {
    Scalar(u32),
    Pair(u32, u32),
    Triple(u32, u32, u32),
    Text(Cow<'a, str>),
    Remove,
}

impl Mapping<'_> {
    fn append_to(self, output: &mut Vec<u32>) -> Result<Range<usize>, MappingError> {
        let length = match &self {
            Self::Scalar(_) => 1,
            Self::Pair(..) => 2,
            Self::Triple(..) => 3,
            Self::Text(text) => text.chars().count(),
            Self::Remove => 0,
        };
        output
            .try_reserve(length)
            .map_err(|_| MappingError::Storage)?;
        let start = output.len();
        match self {
            Self::Scalar(a) => output.push(a),
            Self::Pair(a, b) => output.extend([a, b]),
            Self::Triple(a, b, c) => output.extend([a, b, c]),
            Self::Text(text) => output.extend(text.chars().map(u32::from)),
            Self::Remove => (),
        }
        Ok(start..output.len())
    }
}

pub(super) struct Mapper(DataPayload<CaseMapV1>);

impl Mapper {
    pub(super) fn new() -> Result<Self, MappingError> {
        <Baked as DataProvider<CaseMapV1>>::load(&Baked, Default::default())
            .map(|r| Self(r.payload))
            .map_err(|_| MappingError::DataUnavailable)
    }

    pub(super) fn case_type(&self, cp: u32) -> Option<CaseType> {
        match self.0.get().trie.get32(cp).kind {
            CaseMapDataKind::Exception(case, _) => case,
            CaseMapDataKind::Delta(_, case, _) => Some(case),
            CaseMapDataKind::Uncased(_) => None,
        }
    }

    fn dot(&self, cp: u32) -> Result<DotType, MappingError> {
        Ok(match self.0.get().trie.get32(cp).kind {
            CaseMapDataKind::Exception(_, index) => {
                self.0
                    .get()
                    .exceptions
                    .exceptions
                    .get(usize::from(index))
                    .ok_or(MappingError::InvalidException)?
                    .decode()
                    .bits
                    .dot_type
            }
            CaseMapDataKind::Uncased(data) | CaseMapDataKind::Delta(data, _, _) => data.dot_type,
        })
    }

    fn cased(&self, input: Cursor<'_>) -> Result<bool, MappingError> {
        for cp in input {
            let cp = cp?;
            if !self.0.get().trie.get32(cp).ignoreable {
                return Ok(self.case_type(cp).is_some());
            }
        }
        Ok(false)
    }

    fn final_sigma(&self, before: Cursor<'_>, after: Cursor<'_>) -> Result<bool, MappingError> {
        match self.cased(before) {
            Ok(false) => Ok(false),
            Ok(true) => self.cased(after).map(|after| !after),
            Err(MappingError::Context(edge)) => match self.cased(after) {
                Ok(true) => Ok(false),
                Ok(false) | Err(MappingError::Context(_)) => Err(MappingError::Context(edge)),
                Err(error) => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    fn after_soft_dotted(&self, before: Cursor<'_>) -> Result<bool, MappingError> {
        for cp in before {
            let cp = cp?;
            match self.dot(cp)? {
                DotType::SoftDotted => return Ok(true),
                DotType::OtherAccent => (),
                _ => return Ok(false),
            }
        }
        Ok(false)
    }

    fn after_i(&self, before: Cursor<'_>) -> Result<bool, MappingError> {
        for cp in before {
            let cp = cp?;
            if cp == 0x49 {
                return Ok(true);
            }
            if self.dot(cp)? != DotType::OtherAccent {
                return Ok(false);
            }
        }
        Ok(false)
    }

    fn more_above(&self, after: Cursor<'_>) -> Result<bool, MappingError> {
        for cp in after {
            let cp = cp?;
            match self.dot(cp)? {
                DotType::Above => return Ok(true),
                DotType::OtherAccent => (),
                _ => return Ok(false),
            }
        }
        Ok(false)
    }

    fn before_dot(&self, after: Cursor<'_>) -> Result<bool, MappingError> {
        for cp in after {
            let cp = cp?;
            if cp == 0x307 {
                return Ok(true);
            }
            if self.dot(cp)? != DotType::OtherAccent {
                return Ok(false);
            }
        }
        Ok(false)
    }

    /// Dutch compound eligibility is read from the original word. The caller
    /// independently decides whether this component's own transform/locale
    /// permits the title-context operation.
    pub(super) fn dutch_j(&self, before: &[u32], after: &[u32]) -> Result<bool, MappingError> {
        let input_before = Input::Known {
            values: before,
            missing: &[],
        };
        let input_after = Input::Known {
            values: after,
            missing: &[],
        };
        self.dutch_context(
            input_before.before(before.len()),
            Cursor {
                input: &input_after,
                position: 0,
                reverse: false,
            },
        )
    }

    fn dutch_context(
        &self,
        mut before: Cursor<'_>,
        after: Cursor<'_>,
    ) -> Result<bool, MappingError> {
        let mut i_acute = false;
        loop {
            match before.next().transpose()? {
                Some(0x69 | 0x49) => break,
                Some(0xed | 0xcd) => {
                    i_acute = true;
                    break;
                }
                Some(0x301) => i_acute = true,
                _ => return Ok(false),
            }
        }
        if before.next().transpose()?.is_some() {
            return Ok(false);
        }
        let mut j_acute = false;
        for cp in after {
            let cp = cp?;
            if cp == 0x301 {
                j_acute = true;
                continue;
            }
            match self.dot(cp)? {
                DotType::NoDot | DotType::SoftDotted => break,
                _ => return Ok(false),
            }
        }
        Ok(i_acute == j_acute)
    }

    /// Writes each actual component once into the caller's flat output buffer.
    pub(super) fn map_in_context(
        &self,
        input: &Input<'_>,
        at: usize,
        operation: Operation,
        locale: Locale,
        output: &mut Vec<u32>,
    ) -> Result<Range<usize>, MappingError> {
        self.mapping(input, at, operation, locale)?
            .append_to(output)
    }

    #[cfg(test)]
    pub(super) fn map(
        &self,
        input: &[u32],
        at: usize,
        operation: Operation,
        locale: Locale,
    ) -> Result<Vec<u32>, MappingError> {
        let mut output = Vec::new();
        self.map_in_context(
            &Input::Known {
                values: input,
                missing: &[],
            },
            at,
            operation,
            locale,
            &mut output,
        )?;
        Ok(output)
    }

    /// `input` is the actual immutable context domain, not progressively mapped
    /// output. Removal is a real result owned by this input row.
    fn mapping(
        &self,
        input: &Input<'_>,
        at: usize,
        operation: Operation,
        locale: Locale,
    ) -> Result<Mapping<'_>, MappingError> {
        let cp = input.value(at)?.ok_or(MappingError::InvalidScalar)?;
        // An isolated UTF-16 unit is opaque. It is not the replacement glyph
        // scalar used by the font-facing UTF-8 projection.
        if (0xd800..=0xdfff).contains(&cp) {
            return Ok(Mapping::Scalar(cp));
        }
        let before = input.before(at);
        let after = input.after(at);
        let text = |s| Mapping::Text(Cow::Borrowed(s));
        let upper = matches!(
            operation,
            Operation::Upper | Operation::Title | Operation::TitleContext
        );
        if operation == Operation::TitleContext {
            if locale == Locale::Dutch
                && matches!(cp, 0x6a | 0x4a)
                && self.dutch_context(before, after)?
            {
                return Ok(Mapping::Scalar(0x4a));
            }
            if locale == Locale::Lithuanian && cp == 0x307 && self.after_soft_dotted(before)? {
                return Ok(Mapping::Remove);
            }
            return Ok(Mapping::Scalar(cp));
        }
        if operation == Operation::Lower {
            if locale == Locale::Lithuanian {
                match cp {
                    0x49 | 0x4a | 0x12e if self.more_above(after)? => {
                        return Ok(Mapping::Pair(
                            if cp == 0x49 {
                                0x69
                            } else if cp == 0x4a {
                                0x6a
                            } else {
                                0x12f
                            },
                            0x307,
                        ));
                    }
                    0xcc => return Ok(text("i\u{307}\u{300}")),
                    0xcd => return Ok(text("i\u{307}\u{301}")),
                    0x128 => return Ok(text("i\u{307}\u{303}")),
                    _ => (),
                }
            }
            if locale == Locale::Turkic {
                if cp == 0x130 {
                    return Ok(Mapping::Scalar(0x69));
                }
                if cp == 0x307 && self.after_i(before)? {
                    return Ok(Mapping::Remove);
                }
                if cp == 0x49 && !self.before_dot(after)? {
                    return Ok(Mapping::Scalar(0x131));
                }
            }
            if cp == 0x130 {
                return Ok(text("i\u{307}"));
            }
            if cp == CAPITAL_SIGMA && self.final_sigma(before, after)? {
                return Ok(Mapping::Scalar(FINAL_SIGMA));
            }
        }
        if upper {
            if locale == Locale::Turkic && cp == 0x69 {
                return Ok(Mapping::Scalar(0x130));
            }
            if locale == Locale::Lithuanian && cp == 0x307 && self.after_soft_dotted(before)? {
                return Ok(Mapping::Remove);
            }
            if cp == 0x587 {
                return Ok(text(match (locale, operation) {
                    (Locale::Armenian, Operation::Upper) => "ԵՎ",
                    (Locale::Armenian, _) => "Եվ",
                    (_, Operation::Upper) => "ԵՒ",
                    _ => "Եւ",
                }));
            }
        }
        if operation == Operation::Upper
            && locale == Locale::Greek
            && let Some(mapped) = self.greek(cp, before, after)?
        {
            return Ok(mapped);
        }
        self.ordinary(cp, operation)
    }

    fn ordinary(&self, cp: u32, operation: Operation) -> Result<Mapping<'_>, MappingError> {
        let relevant = match operation {
            Operation::Lower => {
                matches!(self.case_type(cp), Some(CaseType::Upper | CaseType::Title))
            }
            _ => self.case_type(cp) == Some(CaseType::Lower),
        };
        let scalar = |value: i64| {
            u32::try_from(value)
                .ok()
                .and_then(char::from_u32)
                .map(|ch| Mapping::Scalar(u32::from(ch)))
                .ok_or(MappingError::InvalidScalar)
        };
        match self.0.get().trie.get32(cp).kind {
            CaseMapDataKind::Delta(_, _, delta) if relevant => {
                scalar(i64::from(cp) + i64::from(delta))
            }
            CaseMapDataKind::Exception(_, index) => {
                let exception = self
                    .0
                    .get()
                    .exceptions
                    .exceptions
                    .get(usize::from(index))
                    .ok_or(MappingError::InvalidException)?
                    .decode();
                let slot = match operation {
                    Operation::Lower => 0,
                    Operation::Upper => 2,
                    _ => 3,
                };
                if let Some(full) = exception.full
                    && let Some(value) = full.into_iter().nth(slot)
                    && !value.is_empty()
                {
                    return Ok(Mapping::Text(value));
                }
                if relevant && let Some(delta) = exception.simple_case_delta {
                    return scalar(if exception.bits.negative_delta {
                        i64::from(cp) - i64::from(delta)
                    } else {
                        i64::from(cp) + i64::from(delta)
                    });
                }
                let value = match operation {
                    Operation::Lower => exception.lowercase,
                    Operation::Upper => exception.uppercase,
                    _ => exception.titlecase.or(exception.uppercase),
                };
                Ok(Mapping::Scalar(value.map_or(cp, u32::from)))
            }
            _ => Ok(Mapping::Scalar(cp)),
        }
    }

    fn greek(
        &self,
        cp: u32,
        before: Cursor<'_>,
        after: Cursor<'_>,
    ) -> Result<Option<Mapping<'_>>, MappingError> {
        if greek::removed_mark(cp) && greek::preceded_by_letter(before)? {
            return Ok(Some(Mapping::Remove));
        }
        let Some(letter) = greek::letter(cp) else {
            return Ok(None);
        };
        Ok(match letter {
            greek::Letter::Consonant { rho: true } => Some(Mapping::Scalar(0x3a1)),
            greek::Letter::Consonant { rho: false } => None,
            greek::Letter::Vowel(vowel, mut precomposed) => {
                let mut d = greek::following(precomposed, after)?;
                if !d.dialytika
                    && matches!(vowel, 'Ι' | 'Υ')
                    && let Some((prev, combining)) = greek::preceding(before)?
                    && !prev.dialytika
                    && !combining.dialytika
                {
                    if combining.accented {
                        d.dialytika = true;
                    } else {
                        precomposed.dialytika = prev.accented;
                    }
                }
                let mut result = [0; 3];
                let mut length = 1;
                result[0] = match vowel {
                    'Η' if d.accented
                        && !d.ypogegrammeni
                        && !self.cased(before)?
                        && !self.cased(after)? =>
                    {
                        if precomposed.accented {
                            0x389
                        } else {
                            result[1] = 0x301;
                            length = 2;
                            0x397
                        }
                    }
                    'Ι' | 'Υ' if precomposed.dialytika => {
                        d.dialytika = false;
                        if vowel == 'Ι' { 0x3aa } else { 0x3ab }
                    }
                    _ => u32::from(vowel),
                };
                if d.dialytika {
                    result[length] = 0x308;
                    length += 1;
                }
                if precomposed.ypogegrammeni {
                    result[length] = 0x399;
                    length += 1;
                }
                Some(match length {
                    1 => Mapping::Scalar(result[0]),
                    2 => Mapping::Pair(result[0], result[1]),
                    _ => Mapping::Triple(result[0], result[1], result[2]),
                })
            }
        })
    }
}

/// Actual output order, lending each scalar's existing emitting row/subposition.
/// Unicode SpecialCasing requires a capital iota originating in ypogegrammeni
/// to follow the complete combining sequence. No normalization or source-owner
/// rewrite is needed: the same row can contribute separated output pieces.
#[cfg(test)]
pub(super) fn emission_order(
    input: &[u32],
    operations: &[Operation],
    mapped: &[Vec<u32>],
) -> Result<Vec<(usize, usize)>, MappingError> {
    use icu_normalizer::properties::CanonicalCombiningClassMap;
    if input.len() != operations.len() || input.len() != mapped.len() {
        return Err(MappingError::InvalidScalar);
    }
    let classes = CanonicalCombiningClassMap::new();
    let mut output = Vec::new();
    let mut pending = Vec::new();
    for (row, &cp) in input.iter().enumerate() {
        if classes.get32_u8(cp) == 0 {
            output.append(&mut pending);
        }
        let postpone = postpones_iota(cp, Some(operations[row]), mapped[row].last().copied());
        for subposition in 0..mapped[row].len() {
            if postpone && subposition + 1 == mapped[row].len() {
                pending.push((row, subposition));
            } else {
                output.push((row, subposition));
            }
        }
    }
    output.append(&mut pending);
    Ok(output)
}

pub(super) fn postpones_iota(cp: u32, operation: Option<Operation>, last: Option<u32>) -> bool {
    use icu_normalizer::properties::{CanonicalDecomposition, Decomposed};
    operation == Some(Operation::Upper)
        && last == Some(0x399)
        && (cp == 0x345
            || char::from_u32(cp).is_some_and(|ch| {
                matches!(
                    CanonicalDecomposition::new().decompose(ch),
                    Decomposed::Expansion(_, '\u{345}')
                )
            }))
}

#[cfg(test)]
mod tests;
