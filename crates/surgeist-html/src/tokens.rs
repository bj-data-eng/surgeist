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
    tokens: Arc<[HtmlSpaceToken]>,
}

impl HtmlSpaceSeparatedTokens {
    /// Constructs a sequence from already-valid tokens in input order.
    pub fn new(tokens: impl IntoIterator<Item = HtmlSpaceToken>) -> Self {
        Self {
            tokens: tokens.into_iter().collect(),
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

fn ascii_whitespace(unit: u16) -> bool {
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
    HtmlSpaceSeparatedTokens::new(
        input
            .code_units()
            .split(|&u| ascii_whitespace(u))
            .filter(|token| !token.is_empty())
            .map(|token| HtmlSpaceToken {
                value: DocumentString::from_utf16(token),
            }),
    )
}

/// Infra split-on-commas, which omits a field following a final comma and
/// produces no tokens for an empty input; this is not Infra's strict split.
pub(crate) fn split_comma_tokens(input: &DocumentString) -> HtmlCommaSeparatedTokens {
    let mut tokens = Vec::new();
    let units = input.code_units();
    let mut position = 0;
    while position < units.len() {
        let start = position;
        while position < units.len() && units[position] != u16::from(b',') {
            position += 1;
        }
        let mut token = &units[start..position];
        while token.first().is_some_and(|&u| ascii_whitespace(u)) {
            token = &token[1..];
        }
        while token.last().is_some_and(|&u| ascii_whitespace(u)) {
            token = &token[..token.len() - 1];
        }
        tokens.push(HtmlCommaToken {
            value: DocumentString::from_utf16(token),
        });
        if position < units.len() {
            position += 1;
        }
    }
    HtmlCommaSeparatedTokens::new(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

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
