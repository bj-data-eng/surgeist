//! Checked authored font-feature-values data; no font lookup or cascade.
use crate::component::CssComponentValue;
use crate::rule::font_face::CssFontFaceFamily;
use crate::rule::font_feature_values::{CssFontFeatureDisplayValue, CssFontFeatureValue};
use crate::source::{CssParsedOrigin, CssSourcePosition};

/// A semantic construction failure for font-feature-values.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CssFontFeatureValuesErrorKind {
    InvalidIdentifier,
    EmptyFamilies,
    InvalidIntegerSyntax,
    NegativeIndex,
    InvalidIndexCount,
    KindMismatch,
}
/// A font-feature-values construction failure, with the affected block and member when
/// known, or a checked component-native feature-value descriptor failure.
///
/// Semantic failures are read through the `is_*` predicates. A descriptor-value
/// failure exposes [`Self::origin`] and, when the components were not admitted or the
/// grammar rejected them, [`Self::component_error`] or [`Self::diagnostic`], which is
/// the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFeatureValuesError {
    kind: FontFeatureValuesKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
enum FontFeatureValuesKind {
    Construction {
        kind: CssFontFeatureValuesErrorKind,
        block: Option<CssFontFeatureValueKind>,
        definition_index: Option<usize>,
    },
    Value(crate::rule::descriptor_values::DescriptorValueFailure),
}
impl CssFontFeatureValuesError {
    const fn new(kind: CssFontFeatureValuesErrorKind) -> Self {
        Self {
            kind: FontFeatureValuesKind::Construction {
                kind,
                block: None,
                definition_index: None,
            },
        }
    }
    pub(crate) fn from_component(error: crate::component::CssComponentValueError) -> Self {
        crate::rule::descriptor_values::DescriptorValueFailure::component(error).into()
    }
    pub(crate) fn from_grammar(
        error: crate::diagnostic::CssError,
        serialized: &crate::component::CssSerializedValue,
    ) -> Self {
        crate::rule::descriptor_values::DescriptorValueFailure::grammar(error, serialized).into()
    }
    pub(crate) fn residual(origin: crate::component::CssValueOrigin) -> Self {
        crate::rule::descriptor_values::DescriptorValueFailure::residual(origin).into()
    }
    pub(crate) fn not_pending() -> Self {
        crate::rule::descriptor_values::DescriptorValueFailure::not_pending().into()
    }
    const fn construction(&self) -> Option<CssFontFeatureValuesErrorKind> {
        match self.kind {
            FontFeatureValuesKind::Construction { kind, .. } => Some(kind),
            FontFeatureValuesKind::Value(_) => None,
        }
    }
    const fn value(&self) -> Option<&crate::rule::descriptor_values::DescriptorValueFailure> {
        match &self.kind {
            FontFeatureValuesKind::Value(failure) => Some(failure),
            FontFeatureValuesKind::Construction { .. } => None,
        }
    }
    /// Returns whether a feature-value name is not a representable identifier.
    #[must_use]
    pub const fn is_invalid_identifier(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::InvalidIdentifier)
        )
    }
    /// Returns whether the family list is empty.
    #[must_use]
    pub const fn is_empty_families(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::EmptyFamilies)
        )
    }
    /// Returns whether an index is not an optional sign followed by decimal digits.
    #[must_use]
    pub const fn is_invalid_integer_syntax(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::InvalidIntegerSyntax)
        )
    }
    /// Returns whether an index is negative.
    #[must_use]
    pub const fn is_negative_index(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::NegativeIndex)
        )
    }
    /// Returns whether a feature value has more indexes than its block admits.
    #[must_use]
    pub const fn is_invalid_index_count(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::InvalidIndexCount)
        )
    }
    /// Returns whether a definition belongs to a block of another kind.
    #[must_use]
    pub const fn is_kind_mismatch(&self) -> bool {
        matches!(
            self.construction(),
            Some(CssFontFeatureValuesErrorKind::KindMismatch)
        )
    }
    /// Returns the affected feature-value block, when known.
    #[must_use]
    pub const fn block(&self) -> Option<CssFontFeatureValueKind> {
        match self.kind {
            FontFeatureValuesKind::Construction { block, .. } => block,
            FontFeatureValuesKind::Value(_) => None,
        }
    }
    /// Returns the zero-based index of the affected definition, when known.
    #[must_use]
    pub const fn definition_index(&self) -> Option<usize> {
        match self.kind {
            FontFeatureValuesKind::Construction {
                definition_index, ..
            } => definition_index,
            FontFeatureValuesKind::Value(_) => None,
        }
    }
    /// Returns the responsible input origin of a descriptor-value failure.
    #[must_use]
    pub fn origin(&self) -> Option<&crate::component::CssSerializedOrigin> {
        self.value()
            .map(crate::rule::descriptor_values::DescriptorValueFailure::origin)
    }
    /// Returns the component failure when the supplied components were not admitted.
    #[must_use]
    pub fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        self.value()?.component_error()
    }
    /// Returns the grammar diagnostic when the feature-value grammar rejected the
    /// components. Its position indexes the serialized component input; [`Self::origin`]
    /// locates the responsible supplied component.
    #[must_use]
    pub fn diagnostic(&self) -> Option<&crate::diagnostic::CssError> {
        self.value()?.diagnostic()
    }
    /// Returns whether an implicitly closed component was supplied.
    #[must_use]
    pub fn is_recovered_component(&self) -> bool {
        self.value()
            .is_some_and(|value| value.is_recovered_component())
    }
    /// Returns whether strict reentry still contains a substitution function.
    #[must_use]
    pub fn is_residual_substitution(&self) -> bool {
        self.value()
            .is_some_and(|value| value.is_residual_substitution())
    }
    /// Returns whether reentry was requested for a value that is not pending substitution.
    #[must_use]
    pub fn is_not_pending(&self) -> bool {
        self.value().is_some_and(|value| value.is_not_pending())
    }
}
impl From<crate::rule::descriptor_values::DescriptorValueFailure> for CssFontFeatureValuesError {
    fn from(failure: crate::rule::descriptor_values::DescriptorValueFailure) -> Self {
        Self {
            kind: FontFeatureValuesKind::Value(failure),
        }
    }
}
impl std::fmt::Display for CssFontFeatureValuesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind = match &self.kind {
            FontFeatureValuesKind::Value(failure) => {
                return failure.describe("font-feature-values", f);
            }
            FontFeatureValuesKind::Construction { kind, .. } => kind,
        };
        f.write_str(match kind {
            CssFontFeatureValuesErrorKind::InvalidIdentifier => {
                "invalid font-feature-values: a feature-value name is not an identifier"
            }
            CssFontFeatureValuesErrorKind::EmptyFamilies => {
                "invalid font-feature-values: the family list is empty"
            }
            CssFontFeatureValuesErrorKind::InvalidIntegerSyntax => {
                "invalid font-feature-values: an index is not a decimal integer"
            }
            CssFontFeatureValuesErrorKind::NegativeIndex => {
                "invalid font-feature-values: an index is negative"
            }
            CssFontFeatureValuesErrorKind::InvalidIndexCount => {
                "invalid font-feature-values: a feature value has too many indexes"
            }
            CssFontFeatureValuesErrorKind::KindMismatch => {
                "invalid font-feature-values: a definition belongs to another block kind"
            }
        })
    }
}
impl std::error::Error for CssFontFeatureValuesError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.value()?.source()
    }
}

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
    /// Returns the normalized decimal digits without sign or leading zeroes (`0` for zero).
    pub fn as_decimal_str(&self) -> &str {
        &self.digits
    }
    /// A convenience conversion; overflow never constrains the authored grammar.
    pub fn to_u32(&self) -> Option<u32> {
        self.digits.parse().ok()
    }
    /// Returns the original raw-source window of the index, or `None` when it was constructed programmatically.
    pub const fn origin(&self) -> Option<&CssParsedOrigin> {
        self.origin.as_ref()
    }
    pub(crate) fn with_origin(mut self, origin: CssParsedOrigin) -> Self {
        self.origin = Some(origin);
        self
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
    /// Checks that `value` is a representable identifier and keeps its decoded text.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` cannot be an identifier.
    pub fn try_new(value: impl Into<String>) -> Result<Self, CssFontFeatureValuesError> {
        let value = value.into();
        CssComponentValue::try_ident(value.clone()).map_err(|_| {
            CssFontFeatureValuesError::new(CssFontFeatureValuesErrorKind::InvalidIdentifier)
        })?;
        Ok(Self(value.into_boxed_str()))
    }
    /// Returns the decoded identifier text; it is case-sensitive.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// The seven subsidiary feature-value grammars.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFeatureValueKind {
    /// The `@stylistic` block, whose definitions take exactly one index.
    Stylistic,
    /// The `@historical-forms` block, whose definitions take one or more indexes.
    HistoricalForms,
    /// The `@styleset` block, whose definitions take one or more indexes.
    Styleset,
    /// The `@character-variant` block, whose definitions take one or two indexes.
    CharacterVariant,
    /// The `@swash` block, whose definitions take exactly one index.
    Swash,
    /// The `@ornaments` block, whose definitions take exactly one index.
    Ornaments,
    /// The `@annotation` block, whose definitions take exactly one index.
    Annotation,
}
impl CssFontFeatureValueKind {
    /// Returns the CSS spelling of the block name without the `@`, such as `historical-forms`.
    pub const fn as_str(self) -> &'static str {
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
        .find(|kind| value.eq_ignore_ascii_case(kind.as_str()))
    }
    // Selected Fonts 4 (2026-09-07) section 6.9.2 / frozen WebKit
    // 73aa6c89e2cb77c46184a81aec944e4ab99d114d CSSParser.cpp:800–885 policy.
    // Section 6.9.1 conflicts on CV cardinality and CV/styleset feature ranges;
    // font activation ranges do not constrain these exact authored integers.
    pub(crate) fn validate_index_count(
        self,
        count: usize,
    ) -> Result<(), CssFontFeatureValuesError> {
        let valid_count = match self {
            Self::HistoricalForms | Self::Styleset => count > 0,
            Self::CharacterVariant => matches!(count, 1 | 2),
            _ => count == 1,
        };
        if valid_count {
            Ok(())
        } else {
            Err(CssFontFeatureValuesError {
                kind: FontFeatureValuesKind::Construction {
                    kind: CssFontFeatureValuesErrorKind::InvalidIndexCount,
                    block: Some(self),
                    definition_index: None,
                },
            })
        }
    }
}
/// A named authored subsidiary value. Its carrier owns kind and phase validity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValueDefinition {
    name: CssFontFeatureValueName,
    value: CssFontFeatureValue,
    parsed_name: Option<CssParsedOrigin>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureValueDefinition {
    /// Creates a definition from a name and a value, with no parsed name origin or source position.
    ///
    /// The owning block checks that the value kind matches the block.
    #[must_use]
    pub const fn new(name: CssFontFeatureValueName, value: CssFontFeatureValue) -> Self {
        Self {
            name,
            value,
            parsed_name: None,
            position: None,
        }
    }
    /// Returns the decoded feature-value name.
    pub const fn name(&self) -> &CssFontFeatureValueName {
        &self.name
    }
    /// Returns the definition's indexes or pending value.
    pub const fn value(&self) -> &CssFontFeatureValue {
        &self.value
    }
    /// Returns the original raw-source window of the name, or `None` when it was constructed programmatically.
    pub const fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        self.parsed_name.as_ref()
    }
    /// Returns the source position of the definition, or `None` when it was constructed programmatically.
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
    pub(crate) fn with_parsed_name(mut self, origin: CssParsedOrigin) -> Self {
        self.parsed_name = Some(origin);
        self
    }
}
/// An ordered context-validated block. Empty blocks and duplicate definitions remain authored
/// syntax.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValueBlock {
    kind: CssFontFeatureValueKind,
    definitions: Vec<CssFontFeatureValueDefinition>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureValueBlock {
    /// Creates a block of one kind from its definitions, retained in the given order with duplicates, with no source position.
    ///
    /// # Errors
    ///
    /// Returns an error naming the block kind and the definition index when a definition's value is of another kind.
    pub fn try_new(
        kind: CssFontFeatureValueKind,
        definitions: Vec<CssFontFeatureValueDefinition>,
    ) -> Result<Self, CssFontFeatureValuesError> {
        for (index, definition) in definitions.iter().enumerate() {
            if definition.value().kind() != kind {
                return Err(CssFontFeatureValuesError {
                    kind: FontFeatureValuesKind::Construction {
                        kind: CssFontFeatureValuesErrorKind::KindMismatch,
                        block: Some(kind),
                        definition_index: Some(index),
                    },
                });
            }
        }
        Ok(Self {
            kind,
            definitions,
            position: None,
        })
    }
    /// Returns which feature-value block this is.
    pub const fn kind(&self) -> CssFontFeatureValueKind {
        self.kind
    }
    /// Returns the definitions in authored order, including duplicates.
    pub fn definitions(&self) -> &[CssFontFeatureValueDefinition] {
        &self.definitions
    }
    /// Returns the source position of the block, or `None` when it was constructed programmatically.
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
    value: CssFontFeatureDisplayValue,
    parsed_name: Option<CssParsedOrigin>,
    position: Option<CssSourcePosition>,
}
impl CssFontFeatureDisplayOccurrence {
    /// Creates a font-display occurrence from its value, with no parsed name origin or source position.
    pub const fn new(value: CssFontFeatureDisplayValue) -> Self {
        Self {
            value,
            parsed_name: None,
            position: None,
        }
    }
    /// Returns the font-display value.
    pub const fn value(&self) -> &CssFontFeatureDisplayValue {
        &self.value
    }
    /// Returns the original raw-source window of the descriptor name, or `None` when it was constructed programmatically.
    pub const fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        self.parsed_name.as_ref()
    }
    pub(crate) fn with_parsed_name(mut self, origin: CssParsedOrigin) -> Self {
        self.parsed_name = Some(origin);
        self
    }
    /// Returns the source position of the descriptor, or `None` when it was constructed programmatically.
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
    /// A `font-display` descriptor occurrence.
    FontDisplay(CssFontFeatureDisplayOccurrence),
    /// A feature-value block such as `@swash`.
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
    /// Creates a rule from its family names and body items, retaining the items in the given order with no source position.
    ///
    /// # Errors
    ///
    /// Returns an error when `families` is empty.
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
    /// Returns the family names of the prelude, in authored order; the slice is never empty.
    pub fn families(&self) -> &[CssFontFaceFamily] {
        &self.families
    }
    /// Returns the body items in authored order, with interleaving and duplicates retained.
    pub fn items(&self) -> &[CssFontFeatureValuesItem] {
        &self.items
    }
    /// Returns the source position of the rule, or `None` when it was constructed programmatically.
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
    pub(crate) fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }
}

/// A nonempty ordered list of decoded literal feature-value family names.
///
/// Parsed lists retain the actual raw prelude and each comma-delimited member
/// (excluding surrounding trivia). Programmatic lists claim no parsed source.
/// Generic-family admission and authored quoting belong to the parser, while
/// serialization uses the literal family writer rather than property-family forms.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFeatureValuesFamilyList {
    families: Vec<CssFontFaceFamily>,
    origin: crate::component::CssValueOrigin,
    family_origins: Vec<CssParsedOrigin>,
}
impl CssFontFeatureValuesFamilyList {
    /// Checks the nonempty literal list; existing literal values are not reparsed.
    pub fn try_new(families: Vec<CssFontFaceFamily>) -> Result<Self, CssFontFeatureValuesError> {
        if families.is_empty() {
            return Err(CssFontFeatureValuesError::new(
                CssFontFeatureValuesErrorKind::EmptyFamilies,
            ));
        }
        Ok(Self {
            families,
            origin: crate::component::CssValueOrigin::Programmatic,
            family_origins: Vec::new(),
        })
    }
    pub(crate) fn from_parsed(
        families: Vec<CssFontFaceFamily>,
        origin: CssParsedOrigin,
        family_origins: Vec<CssParsedOrigin>,
    ) -> Self {
        debug_assert!(!families.is_empty() && families.len() == family_origins.len());
        Self {
            families,
            origin: crate::component::CssValueOrigin::Parsed(origin),
            family_origins,
        }
    }
    /// Returns literal names in input order.
    #[must_use]
    pub fn families(&self) -> &[CssFontFaceFamily] {
        &self.families
    }
    /// Returns the complete original input, or honest programmatic provenance.
    #[must_use]
    pub const fn origin(&self) -> &crate::component::CssValueOrigin {
        &self.origin
    }
    /// Returns an original member occurrence; programmatic lists have none.
    #[must_use]
    pub fn family_origin(&self, index: usize) -> Option<&CssParsedOrigin> {
        self.family_origins.get(index)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssFontFeatureValuesFamilyList
{
    /// Emits the literal comma-separated family list.
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), crate::serialization::CssSerializationError> {
        writer.comma_separated(&self.families, |writer, family| {
            crate::serialization::specified_rule_serialization::SpecifiedProvider::append_to_rule_writer(
                family, writer,
            )
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(
    CssFontFeatureValuesFamilyList
);

impl CssFontFeatureValuesFamilyList {
    /// Parses one complete nonempty nongeneric literal-family prelude, without
    /// constructing an enclosing rule. Retention is independent of report cleanliness.
    ///
    /// The parser supplies the document mode and the cumulative component limits.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssFontFeatureValuesFamilyList>> {
        crate::parser::font_feature_values_family_list::parse_with_context(
            source.as_ref(),
            parser.component_limits(),
            parser.context(),
        )
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
        let mut sheet = crate::rule::CssSheet::new();
        sheet.push_rule(crate::rule::CssRule::FontFeatureValues(rule.clone()));
        let normalized = sheet.normalize(Default::default()).unwrap();
        let [crate::normalize::CssNormalizedItem::Rule(context)] = normalized.items() else {
            panic!("one opaque rule expected")
        };
        assert!(context.position().is_none());
        let crate::normalize::CssRuleContextKindRef::FontFeatureValues(payload) = context.kind()
        else {
            panic!("font payload expected")
        };
        assert_eq!(payload, &rule);
        assert!(context.parent().is_none());
    }
}
