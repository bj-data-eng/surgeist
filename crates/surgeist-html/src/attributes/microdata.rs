//! HTML §5.2 exact token observations; URL, vocabulary and tree proof are separate.
use super::*;
use crate::{HtmlSpaceToken, tokens::SpaceTokenStep};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenKind {
    ItemType,
    ItemProp,
    ItemRef,
}
impl HtmlMicrodataTokenKind {
    pub fn attribute(self) -> KnownHtmlAttribute {
        match self {
            Self::ItemType => KnownHtmlAttribute::Itemtype,
            Self::ItemProp => KnownHtmlAttribute::Itemprop,
            Self::ItemRef => KnownHtmlAttribute::Itemref,
        }
    }
}
/// One occurrence in the original UTF-16 backing, not a normalized copy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataToken {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) duplicate_of: Option<usize>,
    pub(crate) dot_or_colon: bool,
}
impl HtmlMicrodataToken {
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
    pub fn duplicate_of(&self) -> Option<usize> {
        self.duplicate_of
    }
    pub fn contains_dot_or_colon(&self) -> bool {
        self.dot_or_colon
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenReport {
    pub(crate) kind: HtmlMicrodataTokenKind,
    pub(crate) raw: Option<DocumentString>,
    pub(crate) tokens: Vec<HtmlMicrodataToken>,
}
impl HtmlMicrodataTokenReport {
    pub fn kind(&self) -> HtmlMicrodataTokenKind {
        self.kind
    }
    pub fn raw(&self) -> Option<&DocumentString> {
        self.raw.as_ref()
    }
    pub fn tokens(&self) -> &[HtmlMicrodataToken] {
        &self.tokens
    }
    pub fn token_units(&self, index: usize) -> Option<&[u16]> {
        self.tokens
            .get(index)
            .map(|t| &self.raw.as_ref().expect("token has raw").code_units()[t.start..t.end])
    }
    pub fn processing_tokens(&self) -> impl Iterator<Item = &[u16]> {
        self.tokens
            .iter()
            .filter(|t| self.kind != HtmlMicrodataTokenKind::ItemProp || t.duplicate_of.is_none())
            .map(|t| &self.raw.as_ref().expect("token has raw").code_units()[t.start..t.end])
    }
    /// Local presence/nonempty/uniqueness only; contextual conformance is separate.
    pub fn local_author_conforming(&self) -> bool {
        self.raw.is_none()
            || ((self.kind == HtmlMicrodataTokenKind::ItemRef || !self.tokens.is_empty())
                && self.tokens.iter().all(|t| t.duplicate_of.is_none()))
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlMicrodataTokenListError {
    Empty,
    Duplicate(crate::HtmlDuplicateTokenError),
}
impl std::fmt::Display for HtmlMicrodataTokenListError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HtmlMicrodataTokenListError {}
/// A unique native token list; type URLs and contextual author rules are unchecked.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlMicrodataTokenList {
    kind: HtmlMicrodataTokenKind,
    tokens: crate::HtmlUnorderedTokenSet,
}
impl HtmlMicrodataTokenList {
    pub fn new(
        kind: HtmlMicrodataTokenKind,
        tokens: Vec<HtmlSpaceToken>,
    ) -> Result<Self, HtmlMicrodataTokenListError> {
        if kind != HtmlMicrodataTokenKind::ItemRef && tokens.is_empty() {
            return Err(HtmlMicrodataTokenListError::Empty);
        }
        let tokens =
            crate::HtmlUnorderedTokenSet::new(crate::HtmlTokenComparison::CaseSensitive, tokens)
                .map_err(HtmlMicrodataTokenListError::Duplicate)?;
        Ok(Self { kind, tokens })
    }
    /// Ordinary allocation, literal single-space separators, no URL validation.
    pub fn into_attribute(self) -> Attribute {
        Attribute::new(
            self.kind.attribute().qualified_name(),
            self.tokens.as_document_string(),
        )
    }
}
impl HtmlValueInterpreter {
    /// Whole raw cap precedes scanning. Ordinary allocation; no semantic work budget.
    pub fn microdata_tokens(
        &self,
        kind: HtmlMicrodataTokenKind,
        raw: Option<&DocumentString>,
    ) -> Result<HtmlMicrodataTokenReport, HtmlValueAdmissionError> {
        self.admit_attribute_raw(raw)?;
        let mut tokens = Vec::new();
        if let Some(raw) = raw {
            let mut cursor = crate::tokens::SpaceTokenCursor::new(raw.code_units());
            loop {
                match cursor.step() {
                    SpaceTokenStep::Continue => {}
                    SpaceTokenStep::Done => break,
                    SpaceTokenStep::Token { start, end } => {
                        let units = &raw.code_units()[start..end];
                        let duplicate_of = tokens.iter().position(|t: &HtmlMicrodataToken| {
                            raw.code_units()[t.start..t.end] == *units
                        });
                        tokens.push(HtmlMicrodataToken {
                            start,
                            end,
                            duplicate_of,
                            dot_or_colon: units.iter().any(|u| matches!(*u, 0x2e | 0x3a)),
                        });
                    }
                }
            }
        }
        Ok(HtmlMicrodataTokenReport {
            kind,
            raw: raw.cloned(),
            tokens,
        })
    }
}
