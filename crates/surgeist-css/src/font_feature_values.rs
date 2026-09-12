//! Checked authored font-feature-values data; no font lookup or cascade.
use crate::{
    CssComponentValue, CssFontDisplay, CssFontFaceFamily, CssParsedOrigin, CssSourcePosition,
};

/// A semantic construction failure for font-feature-values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValuesErrorKind {
    InvalidIdentifier,
    EmptyFamilies,
    EmptyIndexes,
    InvalidIntegerSyntax,
    NegativeIndex,
    InvalidIndexCount,
    IndexOutOfRange,
}
/// A checked-construction failure, with the affected block and member when known.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValuesError {
    kind: CssFontFeatureValuesErrorKind,
    block: Option<CssFontFeatureValueKind>,
    definition_index: Option<usize>,
    index: Option<usize>,
}
impl CssFontFeatureValuesError {
    fn new(kind: CssFontFeatureValuesErrorKind) -> Self {
        Self {
            kind,
            block: None,
            definition_index: None,
            index: None,
        }
    }
    pub const fn kind(&self) -> CssFontFeatureValuesErrorKind {
        self.kind
    }
    pub const fn block(&self) -> Option<CssFontFeatureValueKind> {
        self.block
    }
    pub const fn definition_index(&self) -> Option<usize> {
        self.definition_index
    }
    pub const fn index(&self) -> Option<usize> {
        self.index
    }
}
impl std::fmt::Display for CssFontFeatureValuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid font-feature-values: {:?}", self.kind)
    }
}
impl std::error::Error for CssFontFeatureValuesError {}

/// An exact nonnegative authored integer, without a machine-integer upper bound.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValueIndex {
    digits: Box<str>,
    origin: Option<CssParsedOrigin>,
}
impl CssFontFeatureValueIndex {
    /// Accepts an optional ASCII sign and decimal digits; normalizes signs and leading zeroes.
    /// This semantic constructor does not claim its input was already a CSS token.
    pub fn try_from_decimal(value: &str) -> Result<Self, CssFontFeatureValuesError> {
        let digits = value.strip_prefix(['+', '-']).unwrap_or(value);
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
            return Err(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::InvalidIntegerSyntax,
            ));
        }
        let normalized = digits.trim_start_matches('0');
        if value.starts_with('-') && !normalized.is_empty() {
            return Err(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::NegativeIndex,
            ));
        }
        Ok(Self {
            digits: if normalized.is_empty() {
                "0"
            } else {
                normalized
            }
            .into(),
            origin: None,
        })
    }
    pub fn as_decimal_str(&self) -> &str {
        &self.digits
    }
    /// A convenience conversion; overflow never constrains the authored grammar.
    pub fn to_u32(&self) -> Option<u32> {
        self.digits.parse().ok()
    }
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
    pub(crate) fn with_origin(mut self, origin: CssParsedOrigin) -> Self {
        self.origin = Some(origin);
        self
    }
    fn exceeds(&self, bound: &str) -> bool {
        self.digits.len() > bound.len()
            || (self.digits.len() == bound.len() && self.digits.as_ref() > bound)
    }
}
impl From<u32> for CssFontFeatureValueIndex {
    fn from(value: u32) -> Self {
        Self {
            digits: value.to_string().into_boxed_str(),
            origin: None,
        }
    }
}
/// A decoded case-sensitive identifier, including escaped punctuation and keyword names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValueName(Box<str>);
impl CssFontFeatureValueName {
    pub fn try_new(value: impl Into<String>) -> Result<Self, CssFontFeatureValuesError> {
        let value = value.into();
        CssComponentValue::try_ident(value.clone()).map_err(|_| {
            CssFontFeatureValuesError::new(CssFontFeatureValuesErrorKind::InvalidIdentifier)
        })?;
        Ok(Self(value.into_boxed_str()))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// The seven subsidiary feature-value grammars.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValueKind {
    Stylistic,
    HistoricalForms,
    Styleset,
    CharacterVariant,
    Swash,
    Ornaments,
    Annotation,
}
impl CssFontFeatureValueKind {
    pub const fn css_name(self) -> &'static str {
        match self {
            Self::Stylistic => "stylistic",
            Self::HistoricalForms => "historical-forms",
            Self::Styleset => "styleset",
            Self::CharacterVariant => "character-variant",
            Self::Swash => "swash",
            Self::Ornaments => "ornaments",
            Self::Annotation => "annotation",
        }
    }
    pub(crate) fn from_css_name(value: &str) -> Option<Self> {
        [
            Self::Stylistic,
            Self::HistoricalForms,
            Self::Styleset,
            Self::CharacterVariant,
            Self::Swash,
            Self::Ornaments,
            Self::Annotation,
        ]
        .into_iter()
        .find(|kind| value.eq_ignore_ascii_case(kind.css_name()))
    }
    // Provisional Fonts 4 (2026-09-07) section 6.9.1 policy. Section 6.9.2
    // conflicts on character-variant cardinality/first range and styleset range.
    pub(crate) fn validate(
        self,
        definition: &CssFontFeatureValueDefinition,
    ) -> Result<(), CssFontFeatureValuesError> {
        let count = definition.indexes.len();
        let valid_count = match self {
            Self::HistoricalForms | Self::Styleset => count > 0,
            Self::CharacterVariant => count == 2,
            _ => count == 1,
        };
        let mut error = if !valid_count {
            Some(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::InvalidIndexCount,
            ))
        } else {
            None
        };
        if error.is_none() {
            let invalid = match self {
                Self::Styleset => definition
                    .indexes
                    .iter()
                    .position(|index| index.exceeds("20")),
                Self::CharacterVariant if definition.indexes[0].exceeds("99") => Some(0),
                _ => None,
            };
            if let Some(index) = invalid {
                let mut range =
                    CssFontFeatureValuesError::new(CssFontFeatureValuesErrorKind::IndexOutOfRange);
                range.index = Some(index);
                error = Some(range);
            }
        }
        match error {
            Some(mut error) => {
                error.block = Some(self);
                Err(error)
            }
            None => Ok(()),
        }
    }
}
/// A generic nonempty index definition. A block additionally validates its kind's constraints.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValueDefinition {
    name: CssFontFeatureValueName,
    indexes: Vec<CssFontFeatureValueIndex>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureValueDefinition {
    pub fn try_new(
        name: CssFontFeatureValueName,
        indexes: Vec<CssFontFeatureValueIndex>,
    ) -> Result<Self, CssFontFeatureValuesError> {
        if indexes.is_empty() {
            return Err(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::EmptyIndexes,
            ));
        }
        Ok(Self {
            name,
            indexes,
            position: None,
        })
    }
    pub const fn name(&self) -> &CssFontFeatureValueName {
        &self.name
    }
    pub fn indexes(&self) -> &[CssFontFeatureValueIndex] {
        &self.indexes
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}
/// An ordered context-validated block. Empty blocks and duplicate definitions remain authored syntax.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValueBlock {
    kind: CssFontFeatureValueKind,
    definitions: Vec<CssFontFeatureValueDefinition>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureValueBlock {
    pub fn try_new(
        kind: CssFontFeatureValueKind,
        definitions: Vec<CssFontFeatureValueDefinition>,
    ) -> Result<Self, CssFontFeatureValuesError> {
        for (index, definition) in definitions.iter().enumerate() {
            kind.validate(definition).map_err(|mut error| {
                error.definition_index = Some(index);
                error
            })?;
        }
        Ok(Self {
            kind,
            definitions,
            position: None,
        })
    }
    pub const fn kind(&self) -> CssFontFeatureValueKind {
        self.kind
    }
    pub fn definitions(&self) -> &[CssFontFeatureValueDefinition] {
        &self.definitions
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}
/// One ordered font-display descriptor occurrence.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureDisplayOccurrence {
    value: CssFontDisplay,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureDisplayOccurrence {
    pub const fn new(value: CssFontDisplay) -> Self {
        Self {
            value,
            position: None,
        }
    }
    pub const fn value(&self) -> CssFontDisplay {
        self.value
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}
/// An outer body member; interleaving and duplicates are retained.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValuesItem {
    FontDisplay(CssFontFeatureDisplayOccurrence),
    Block(CssFontFeatureValueBlock),
}
/// An ordered authored rule. It neither resolves names nor chooses winning definitions.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValuesRule {
    families: Vec<CssFontFaceFamily>,
    items: Vec<CssFontFeatureValuesItem>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureValuesRule {
    pub fn try_new(
        families: Vec<CssFontFaceFamily>,
        items: Vec<CssFontFeatureValuesItem>,
    ) -> Result<Self, CssFontFeatureValuesError> {
        if families.is_empty() {
            return Err(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::EmptyFamilies,
            ));
        }
        Ok(Self {
            families,
            items,
            position: None,
        })
    }
    pub fn families(&self) -> &[CssFontFaceFamily] {
        &self.families
    }
    pub fn items(&self) -> &[CssFontFeatureValuesItem] {
        &self.items
    }
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn programmatic_payload_normalizes_without_source_positions() {
        let rule = CssFontFeatureValuesRule::try_new(
            vec![CssFontFaceFamily::try_new("serif").unwrap()],
            vec![],
        )
        .unwrap();
        let mut sheet = crate::CssSheet::new();
        sheet.push_rule(crate::CssRule::FontFeatureValues(rule.clone()));
        let normalized = crate::normalize_sheet(&sheet).unwrap();
        let [crate::CssNormalizedItem::Rule(context)] = normalized.items() else {
            panic!("one opaque rule expected")
        };
        assert!(context.position().is_none());
        let crate::CssRuleContextKindRef::FontFeatureValues(payload) = context.kind() else {
            panic!("font payload expected")
        };
        assert_eq!(payload, &rule);
        assert!(context.parent().is_none());
    }
}
