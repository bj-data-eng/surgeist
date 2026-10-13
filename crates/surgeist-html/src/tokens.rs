//! HTML §2.3.7 and §2.3.8 native token values.
//!
//! Grammar validity is independent of attribute-specific allowed keywords or
//! further token restrictions. Those defining clauses require their own checks.

use crate::DocumentString;
use std::{collections::HashMap, error::Error, fmt, sync::Arc};

/// A nonempty space-separated token containing no ASCII whitespace.
/// Non-ASCII whitespace, nulls, and isolated surrogates remain exact.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlSpaceToken {
    value: DocumentString,
}

impl HtmlSpaceToken {
    /// Validates one author token without splitting, trimming, or case folding.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlTokenError> {
        let value = value.into();
        if value.is_empty() {
            return Err(HtmlTokenError::EmptySpaceToken);
        }
        if let Some(code_unit_offset) = value.code_units().iter().position(|&u| ascii_whitespace(u))
        {
            return Err(HtmlTokenError::SpaceTokenWhitespace { code_unit_offset });
        }
        Ok(Self { value })
    }

    /// Returns the exact token text.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// A comma-separated token, which may be empty or contain internal whitespace.
/// Commas and leading or trailing ASCII whitespace are excluded from the token.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlCommaToken {
    value: DocumentString,
}

impl HtmlCommaToken {
    /// Validates one author token without discarding surrounding whitespace.
    pub fn new(value: impl Into<DocumentString>) -> Result<Self, HtmlTokenError> {
        let value = value.into();
        let units = value.code_units();
        if units.first().is_some_and(|&u| ascii_whitespace(u)) {
            return Err(HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: 0,
            });
        }
        if let Some(code_unit_offset) = units.iter().position(|&u| u == u16::from(b',')) {
            return Err(HtmlTokenError::CommaTokenComma { code_unit_offset });
        }
        if units.last().is_some_and(|&u| ascii_whitespace(u)) {
            return Err(HtmlTokenError::CommaTokenBoundaryWhitespace {
                code_unit_offset: units.len() - 1,
            });
        }
        Ok(Self { value })
    }

    /// Returns the exact token text.
    pub fn value(&self) -> &DocumentString {
        &self.value
    }
}

/// A rejected native token's grammar error, using UTF-16 code-unit offsets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTokenError {
    EmptySpaceToken,
    SpaceTokenWhitespace { code_unit_offset: usize },
    CommaTokenBoundaryWhitespace { code_unit_offset: usize },
    CommaTokenComma { code_unit_offset: usize },
}

impl fmt::Display for HtmlTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid native HTML token: {self:?}")
    }
}
impl Error for HtmlTokenError {}

/// A grammar-valid space-separated sequence; repetitions are permitted.
/// Uniqueness is a separate contract selected by particular defining clauses.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlSpaceSeparatedTokens {
    tokens: Arc<Vec<HtmlSpaceToken>>,
}

impl HtmlSpaceSeparatedTokens {
    /// Constructs a sequence from already-valid tokens in input order.
    pub fn new(tokens: impl IntoIterator<Item = HtmlSpaceToken>) -> Self {
        Self::from_vec(tokens.into_iter().collect())
    }

    /// Transfers already-admitted storage without another variable allocation.
    pub(crate) fn from_vec(tokens: Vec<HtmlSpaceToken>) -> Self {
        Self {
            tokens: Arc::new(tokens),
        }
    }

    /// Returns every token in input order, including repetitions.
    pub fn tokens(&self) -> &[HtmlSpaceToken] {
        &self.tokens
    }

    /// Serializes exact token spelling with a single space between tokens.
    pub fn as_document_string(&self) -> DocumentString {
        concatenate(self.tokens.iter().map(HtmlSpaceToken::value), b' ')
    }
}

/// A grammar-valid comma-separated sequence, preserving repetitions and empties.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCommaSeparatedTokens {
    tokens: Arc<[HtmlCommaToken]>,
}

impl HtmlCommaSeparatedTokens {
    /// Constructs a sequence from already-valid tokens in input order.
    /// Both zero tokens and one empty token serialize to the empty string.
    pub fn new(tokens: impl IntoIterator<Item = HtmlCommaToken>) -> Self {
        Self {
            tokens: tokens.into_iter().collect(),
        }
    }

    /// Returns tokens in input order, including empty and repeated tokens.
    pub fn tokens(&self) -> &[HtmlCommaToken] {
        &self.tokens
    }

    /// Serializes exact token spelling with a single comma between tokens.
    pub fn as_document_string(&self) -> DocumentString {
        concatenate(self.tokens.iter().map(HtmlCommaToken::value), b',')
    }
}

/// The comparison rule selected by a token set's defining clause.
/// HTML does not establish one universal token comparison rule.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HtmlTokenComparison {
    CaseSensitive,
    AsciiCaseInsensitive,
}

impl HtmlTokenComparison {
    fn key(self, value: &DocumentString) -> Vec<u16> {
        value
            .code_units()
            .iter()
            .map(|&u| match self {
                Self::AsciiCaseInsensitive if (0x41..=0x5A).contains(&u) => u + 0x20,
                _ => u,
            })
            .collect()
    }

    fn matches(self, left: &DocumentString, right: &DocumentString) -> bool {
        left.len() == right.len()
            && left
                .code_units()
                .iter()
                .zip(right.code_units())
                .all(|(&a, &b)| {
                    let fold = |u: u16| match self {
                        Self::AsciiCaseInsensitive if (0x41..=0x5A).contains(&u) => u + 0x20,
                        _ => u,
                    };
                    fold(a) == fold(b)
                })
    }
}

/// An ordered unique space-separated set under its explicit comparison rule.
/// Exact token spelling is retained; order participates in semantic equality.
#[derive(Clone, Debug)]
pub struct HtmlOrderedTokenSet {
    comparison: HtmlTokenComparison,
    tokens: Arc<[HtmlSpaceToken]>,
    keys: Arc<[Vec<u16>]>,
}

impl HtmlOrderedTokenSet {
    /// Rejects the first duplicate without silently deduplicating authored tokens.
    pub fn new(
        comparison: HtmlTokenComparison,
        tokens: impl IntoIterator<Item = HtmlSpaceToken>,
    ) -> Result<Self, HtmlDuplicateTokenError> {
        let unique = unique_tokens(comparison, tokens)?;
        Ok(Self {
            comparison,
            tokens: unique.tokens,
            keys: unique.keys.into(),
        })
    }

    /// Returns the explicitly selected comparison rule.
    pub fn comparison(&self) -> HtmlTokenComparison {
        self.comparison
    }

    /// Returns tokens in meaningful input order, with exact spelling retained.
    pub fn tokens(&self) -> &[HtmlSpaceToken] {
        &self.tokens
    }

    /// Tests membership using the defining clause's comparison rule.
    pub fn contains(&self, value: &DocumentString) -> bool {
        self.tokens
            .iter()
            .any(|token| self.comparison.matches(token.value(), value))
    }

    /// Serializes tokens in meaningful order, preserving spelling.
    pub fn as_document_string(&self) -> DocumentString {
        concatenate(self.tokens.iter().map(HtmlSpaceToken::value), b' ')
    }
}

impl PartialEq for HtmlOrderedTokenSet {
    fn eq(&self, other: &Self) -> bool {
        self.comparison == other.comparison && self.keys == other.keys
    }
}
impl Eq for HtmlOrderedTokenSet {}

/// An unordered unique space-separated set under its explicit comparison rule.
/// Equality ignores input order. Iteration and serialization retain input order
/// as a spelling choice without assigning it language semantics.
#[derive(Clone, Debug)]
pub struct HtmlUnorderedTokenSet {
    comparison: HtmlTokenComparison,
    tokens: Arc<[HtmlSpaceToken]>,
    keys: Arc<[Vec<u16>]>,
}

impl HtmlUnorderedTokenSet {
    /// Rejects the first duplicate under the explicitly selected comparison rule.
    pub fn new(
        comparison: HtmlTokenComparison,
        tokens: impl IntoIterator<Item = HtmlSpaceToken>,
    ) -> Result<Self, HtmlDuplicateTokenError> {
        let mut unique = unique_tokens(comparison, tokens)?;
        unique.keys.sort_unstable();
        Ok(Self {
            comparison,
            tokens: unique.tokens,
            keys: unique.keys.into(),
        })
    }

    /// Returns the explicitly selected comparison rule.
    pub fn comparison(&self) -> HtmlTokenComparison {
        self.comparison
    }

    /// Returns the retained spelling order, which has no set-order semantics.
    pub fn tokens(&self) -> &[HtmlSpaceToken] {
        &self.tokens
    }

    /// Tests membership using the defining clause's comparison rule.
    pub fn contains(&self, value: &DocumentString) -> bool {
        self.tokens
            .iter()
            .any(|token| self.comparison.matches(token.value(), value))
    }

    /// Serializes exact token spellings in retained input order.
    pub fn as_document_string(&self) -> DocumentString {
        concatenate(self.tokens.iter().map(HtmlSpaceToken::value), b' ')
    }
}

impl PartialEq for HtmlUnorderedTokenSet {
    fn eq(&self, other: &Self) -> bool {
        self.comparison == other.comparison && self.keys == other.keys
    }
}
impl Eq for HtmlUnorderedTokenSet {}

/// A repeated token rejected by a native unique-set constructor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDuplicateTokenError {
    comparison: HtmlTokenComparison,
    token: DocumentString,
    first_index: usize,
    duplicate_index: usize,
}

impl HtmlDuplicateTokenError {
    /// Returns the comparison rule establishing the duplicate identity.
    pub fn comparison(&self) -> HtmlTokenComparison {
        self.comparison
    }
    /// Returns the exact rejected token, without case folding.
    pub fn token(&self) -> &DocumentString {
        &self.token
    }
    /// Returns the first occurrence's zero-based token index.
    pub fn first_index(&self) -> usize {
        self.first_index
    }
    /// Returns the rejected occurrence's zero-based token index.
    pub fn duplicate_index(&self) -> usize {
        self.duplicate_index
    }
}

impl fmt::Display for HtmlDuplicateTokenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "HTML token at index {} duplicates index {} under {:?} comparison",
            self.duplicate_index, self.first_index, self.comparison
        )
    }
}
impl Error for HtmlDuplicateTokenError {}

struct UniqueTokens {
    tokens: Arc<[HtmlSpaceToken]>,
    keys: Vec<Vec<u16>>,
}

fn unique_tokens(
    comparison: HtmlTokenComparison,
    tokens: impl IntoIterator<Item = HtmlSpaceToken>,
) -> Result<UniqueTokens, HtmlDuplicateTokenError> {
    let mut retained = Vec::new();
    let mut keys = Vec::new();
    let mut seen = HashMap::new();
    for token in tokens {
        let key = comparison.key(token.value());
        let duplicate_index = retained.len();
        if let Some(&first_index) = seen.get(&key) {
            return Err(HtmlDuplicateTokenError {
                comparison,
                token: token.value,
                first_index,
                duplicate_index,
            });
        }
        seen.insert(key.clone(), duplicate_index);
        keys.push(key);
        retained.push(token);
    }
    Ok(UniqueTokens {
        tokens: retained.into(),
        keys,
    })
}

pub(crate) fn ascii_whitespace(unit: u16) -> bool {
    matches!(unit, 0x09 | 0x0A | 0x0C | 0x0D | 0x20)
}

fn concatenate<'a>(
    tokens: impl Iterator<Item = &'a DocumentString>,
    delimiter: u8,
) -> DocumentString {
    let mut units = Vec::new();
    for (index, token) in tokens.enumerate() {
        if index != 0 {
            units.push(u16::from(delimiter));
        }
        units.extend_from_slice(token.code_units());
    }
    DocumentString::from_utf16(units)
}

/// Infra split-on-ASCII-whitespace; no attribute-specific conformance claim.
pub(crate) fn split_space_tokens(input: &DocumentString) -> HtmlSpaceSeparatedTokens {
    HtmlSpaceSeparatedTokens::new(space_token_slices(input).map(|token| HtmlSpaceToken {
        value: DocumentString::from_utf16(token),
    }))
}

/// Shared Infra token boundaries for ordinary and controlled materialization.
pub(crate) fn space_token_slices(input: &DocumentString) -> impl Iterator<Item = &[u16]> + Clone {
    SpaceTokenSlices(SpaceTokenCursor::new(input.code_units()))
}

/// One bounded shared Infra action; admission belongs to the consuming caller.
#[derive(Clone)]
pub(crate) struct SpaceTokenCursor<'a> {
    units: &'a [u16],
    position: usize,
    start: Option<usize>,
    ready: Option<(usize, usize)>,
    done: bool,
}
pub(crate) enum SpaceTokenStep {
    Continue,
    Token { start: usize, end: usize },
    Done,
}
impl<'a> SpaceTokenCursor<'a> {
    pub(crate) const BEGIN_WORK: usize = 1;
    // Header selection/read/classification/progress or fixed token emission.
    pub(crate) const STEP_WORK: usize = 4;
    /// Header-only classification, before any source-unit operation.
    pub(crate) fn inspects_unit(&self) -> bool {
        self.ready.is_none() && !self.done && self.position < self.units.len()
    }
    pub(crate) fn new(units: &'a [u16]) -> Self {
        Self {
            units,
            position: 0,
            start: None,
            ready: None,
            done: false,
        }
    }
    pub(crate) fn step(&mut self) -> SpaceTokenStep {
        if let Some((start, end)) = self.ready.take() {
            return SpaceTokenStep::Token { start, end };
        }
        if self.done {
            return SpaceTokenStep::Done;
        }
        if self.position == self.units.len() {
            self.done = true;
            return match self.start.take() {
                Some(start) => SpaceTokenStep::Token {
                    start,
                    end: self.position,
                },
                None => SpaceTokenStep::Done,
            };
        }
        let position = self.position;
        let white = ascii_whitespace(self.units[position]);
        self.position += 1;
        if white {
            if let Some(start) = self.start.take() {
                self.ready = Some((start, position));
            }
        } else if self.start.is_none() {
            self.start = Some(position);
        }
        SpaceTokenStep::Continue
    }
}
#[derive(Clone)]
struct SpaceTokenSlices<'a>(SpaceTokenCursor<'a>);
impl<'a> Iterator for SpaceTokenSlices<'a> {
    type Item = &'a [u16];
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.0.step() {
                SpaceTokenStep::Continue => {}
                SpaceTokenStep::Done => return None,
                SpaceTokenStep::Token { start, end } => return Some(&self.0.units[start..end]),
            }
        }
    }
}

/// One borrowed Infra split-on-commas field. The separator is the actual
/// preceding comma; a trailing final comma does not produce another field.
#[derive(Clone, Copy, Debug)]
pub(crate) struct CommaTokenRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) separator: Option<usize>,
}
#[derive(Clone, Copy)]
enum CommaStep {
    Start,
    Scan,
    Leading,
    Trailing,
    Emit,
    Done,
}
/// Allocation-free boundary owner shared with controlled Email sanitization.
pub(crate) struct CommaTokenCursor<'a> {
    units: &'a [u16],
    position: usize,
    start: usize,
    end: usize,
    separator: Option<usize>,
    next_separator: Option<usize>,
    step: CommaStep,
}
impl<'a> CommaTokenCursor<'a> {
    pub(crate) fn new(units: &'a [u16]) -> Self {
        Self {
            units,
            position: 0,
            start: 0,
            end: 0,
            separator: None,
            next_separator: None,
            step: CommaStep::Start,
        }
    }
    /// Some(index) is the sole source-unit inspection in the next action.
    pub(crate) fn inspection(&self) -> Option<usize> {
        match self.step {
            CommaStep::Scan if self.position < self.units.len() => Some(self.position),
            CommaStep::Leading if self.start < self.end => Some(self.start),
            CommaStep::Trailing if self.start < self.end => Some(self.end - 1),
            _ => None,
        }
    }
    pub(crate) fn finished(&self) -> bool {
        matches!(self.step, CommaStep::Done)
    }
    pub(crate) fn advance(&mut self) -> Option<CommaTokenRange> {
        match self.step {
            CommaStep::Start => {
                if self.position == self.units.len() {
                    self.step = CommaStep::Done;
                } else {
                    self.start = self.position;
                    self.separator = self.next_separator;
                    self.step = CommaStep::Scan;
                }
            }
            CommaStep::Scan => {
                if self.position < self.units.len() && self.units[self.position] != 0x2c {
                    self.position += 1;
                } else {
                    self.end = self.position;
                    self.step = CommaStep::Leading;
                }
            }
            CommaStep::Leading => {
                if self.start < self.end && ascii_whitespace(self.units[self.start]) {
                    self.start += 1;
                } else {
                    self.step = CommaStep::Trailing;
                }
            }
            CommaStep::Trailing => {
                if self.start < self.end && ascii_whitespace(self.units[self.end - 1]) {
                    self.end -= 1;
                } else {
                    self.step = CommaStep::Emit;
                }
            }
            CommaStep::Emit => {
                let range = CommaTokenRange {
                    start: self.start,
                    end: self.end,
                    separator: self.separator,
                };
                if self.position < self.units.len() {
                    self.next_separator = Some(self.position);
                    self.position += 1;
                }
                self.step = CommaStep::Start;
                return Some(range);
            }
            CommaStep::Done => {}
        }
        None
    }
}
/// Infra split-on-commas, which omits a field following a final comma and
/// produces no tokens for an empty input; this is not Infra's strict split.
pub(crate) fn split_comma_tokens(input: &DocumentString) -> HtmlCommaSeparatedTokens {
    let mut tokens = Vec::new();
    let mut cursor = CommaTokenCursor::new(input.code_units());
    while !cursor.finished() {
        if let Some(range) = cursor.advance() {
            tokens.push(HtmlCommaToken {
                value: DocumentString::from_utf16(&input.code_units()[range.start..range.end]),
            });
        }
    }
    HtmlCommaSeparatedTokens::new(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admitted_space_token_storage_transfers_without_variable_reallocation() {
        let mut values = Vec::with_capacity(7);
        values.push(HtmlSpaceToken::new("A").unwrap());
        values.push(HtmlSpaceToken::new("A").unwrap());
        let capacity = values.capacity();
        let pointer = values.as_ptr();
        let tokens = HtmlSpaceSeparatedTokens::from_vec(values);
        assert_eq!(tokens.tokens.capacity(), capacity);
        assert_eq!(tokens.tokens.as_ptr(), pointer);
        assert_eq!(
            tokens,
            HtmlSpaceSeparatedTokens::new([
                HtmlSpaceToken::new("A").unwrap(),
                HtmlSpaceToken::new("A").unwrap(),
            ])
        );
        assert_eq!(tokens.as_document_string(), DocumentString::from("A A"));
        let cloned = tokens.clone();
        assert!(Arc::ptr_eq(&tokens.tokens, &cloned.tokens));
    }

    #[test]
    fn space_splitting_uses_only_the_five_ascii_whitespace_units_and_preserves_raw_text() {
        let input = DocumentString::from_utf16(vec![
            0x20, 0x09, 0xD800, 0x0B, 0xA0, 0x0A, 0x41, 0x0C, 0x41, 0x0D,
        ]);
        let result = split_space_tokens(&input);
        assert_eq!(
            result
                .tokens()
                .iter()
                .map(|t| t.value().code_units())
                .collect::<Vec<_>>(),
            [vec![0xD800, 0x0B, 0xA0], vec![0x41], vec![0x41]]
        );
        assert_eq!(
            input.code_units(),
            [0x20, 0x09, 0xD800, 0x0B, 0xA0, 0x0A, 0x41, 0x0C, 0x41, 0x0D]
        );
        assert!(
            split_space_tokens(&DocumentString::from(" \t\n\r\u{C}"))
                .tokens()
                .is_empty()
        );
    }

    #[test]
    fn comma_splitting_observes_infra_empty_and_trailing_field_rules() {
        for (raw, expected) in [
            ("", vec![]),
            (",", vec![""]),
            (",,", vec!["", ""]),
            ("a,", vec!["a"]),
            (" a ,b,,d d ", vec!["a", "b", "", "d d"]),
            ("a, ", vec!["a", ""]),
            (" \t\n\r\u{C}", vec![""]),
        ] {
            let result = split_comma_tokens(&DocumentString::from(raw));
            assert_eq!(
                result
                    .tokens()
                    .iter()
                    .map(|t| t.value().to_utf8().unwrap())
                    .collect::<Vec<_>>(),
                expected
            );
        }
    }

    #[test]
    fn comma_splitting_preserves_non_ascii_whitespace_and_surrogates() {
        let input = DocumentString::from_utf16(vec![
            0x20, 0xA0, 0xD800, 0x0B, 0x20, 0x2C, 0x20, 0xDC00, 0x20,
        ]);
        let result = split_comma_tokens(&input);
        assert_eq!(
            result.tokens()[0].value().code_units(),
            [0xA0, 0xD800, 0x0B]
        );
        assert_eq!(result.tokens()[1].value().code_units(), [0xDC00]);
        assert_eq!(
            input.code_units(),
            [0x20, 0xA0, 0xD800, 0x0B, 0x20, 0x2C, 0x20, 0xDC00, 0x20]
        );
    }
}
