pub(crate) mod property;
pub(crate) mod retained_backing;
pub(crate) mod source_resolution;
use crate::color::CssColorComponentName;
use crate::property::CssPropertyName;
use crate::query::media::CssMediaFeatureName;
use crate::rule::{CssAtRuleName, CssDescriptorName};
/// Defines one documented `is_*` predicate per pattern of an error's private kind.
///
/// `field` names the kind field; `*field` names a boxed one.
macro_rules! kind_predicates {
    ($field:ident; $($(#[doc = $doc:expr])+ $name:ident => $pattern:pat,)+) => {
        $(
            $(#[doc = $doc])+
            #[must_use]
            pub const fn $name(&self) -> bool {
                matches!(self.$field, $pattern)
            }
        )+
    };
    (*$field:ident; $($(#[doc = $doc:expr])+ $name:ident => $pattern:pat,)+) => {
        $(
            $(#[doc = $doc])+
            #[must_use]
            pub fn $name(&self) -> bool {
                matches!(*self.$field, $pattern)
            }
        )+
    };
}
pub(crate) use kind_predicates;

use std::fmt;

use cssparser::{
    BasicParseError, BasicParseErrorKind, ParseError, ParseErrorKind, Parser, ToCss, Token,
};

use crate::declaration::CssCustomPropertyName;
use crate::property::CssKnownProperty;
use crate::source::CssSourcePosition;
use crate::validation::property_for_supported_name;

/// A stable machine-readable category for a CSS grammar or conformance diagnostic.
///
/// A code classifies the authored invariant violation; it does not select recovery policy or
/// resolve authored values.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssErrorCode {
    /// Escape processing encountered a newline or exhausted its code-point input.
    EscapeParseError,
    /// The parser required more authored syntax but reached the end of input.
    UnexpectedEnd,
    /// The parser encountered an authored token that the active grammar did not accept.
    UnexpectedToken,
    /// An authored at-rule appeared outside its permitted grammar context.
    InvalidAtRulePlacement,
    /// An authored at-rule prelude did not satisfy the rule's grammar.
    InvalidAtRulePrelude,
    /// An authored at-rule body did not satisfy the rule's grammar.
    InvalidAtRuleBody,
    /// A retained declaration repeated a namespace prefix or default binding.
    NamespaceRedeclaration,
    /// A retained navigation target string begins with underscore.
    LegacyNavigationTarget,
    /// The authored at-rule name is not recognized by the CSS catalog.
    UnknownAtRule,
    /// The authored at-rule is recognized but is outside this crate's supported subset.
    UnsupportedAtRule,
    /// An authored qualified rule did not satisfy its grammar.
    InvalidQualifiedRule,
    /// An authored selector did not satisfy the supported selector grammar.
    InvalidSelector,
    /// An authored media query did not satisfy the supported media-query grammar.
    InvalidMediaQuery,
    /// The authored property name is not recognized by the CSS catalog.
    UnknownProperty,
    /// The authored property is recognized but is outside this crate's supported subset.
    UnsupportedProperty,
    /// An authored value did not satisfy its known property's grammar.
    InvalidPropertyValue,
    /// A retained declaration value exceeded component limits or contained invalid tokens.
    InvalidComponentValue,
    /// An authored declaration annotation was malformed or misplaced.
    InvalidDeclarationAnnotation,
    /// The authored descriptor name is not recognized for its owning at-rule.
    UnknownDescriptor,
    /// The authored descriptor is recognized but is outside this crate's supported subset.
    UnsupportedDescriptor,
    /// An authored value did not satisfy its known descriptor's grammar.
    InvalidDescriptorValue,
    /// Authored descriptors formed a combination rejected by their at-rule grammar.
    InvalidDescriptorCombination,
    /// Authored color syntax did not satisfy the supported color grammar.
    InvalidColorSyntax,
    /// Authored nesting exceeded the parser's configured structural limit.
    NestingLimit,
    /// Input nested deeper than the caller's stack is trusted with was refused
    /// because the bounded larger-stack parser thread could not be started.
    ParserUnavailable,
}

/// A static diagnostic-metadata identifier for one CSS grammar production.
///
/// The identifier names the grammar active at a strict parse failure; it does not parse,
/// recover, or resolve authored syntax.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssProductionId(&'static str);

impl CssProductionId {
    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }

    #[must_use]
    /// Returns the stable production identifier.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// A static diagnostic-phase description of the grammar expected at a failure position.
///
/// The description records an invariant the authored syntax violated; it is not a recovery
/// instruction or a downstream validation rule.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssGrammarExpectation(&'static str);

impl CssGrammarExpectation {
    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }

    #[must_use]
    /// Returns the stable expectation description.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// A stable diagnostic-metadata identity for recognized but unsupported authored syntax.
///
/// The identity distinguishes catalogued support from unknown syntax; it does not enable the
/// feature, choose a fallback, or perform downstream resolution.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct CssFeatureId(&'static str);

impl CssFeatureId {
    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }

    #[must_use]
    /// Returns the stable support-catalog identifier.
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

macro_rules! owned_name {
    ($name:ident, $docs:literal) => {
        #[doc = $docs]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name(pub(crate) String);

        impl $name {
            pub(crate) fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            /// Returns the decoded authored identifier retained by the diagnostic.
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}
pub(crate) use owned_name;

/// The diagnostic-phase CSS token category retained by an error summary.
///
/// The non-exhaustive category describes authored syntax without exposing parser tokens; it does
/// not drive tokenization, recovery, or downstream value resolution.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssTokenKind {
    /// An identifier token.
    Ident,
    /// An at-keyword token.
    AtKeyword,
    /// An unrestricted hash token.
    Hash,
    /// An identifier-like hash token.
    IdHash,
    /// A string token.
    String,
    /// A URL token.
    Url,
    /// A delimiter token.
    Delim,
    /// A number token.
    Number,
    /// A percentage token.
    Percentage,
    /// A dimension token.
    Dimension,
    /// A whitespace token.
    Whitespace,
    /// A comment token.
    Comment,
    /// A colon token.
    Colon,
    /// A semicolon token.
    Semicolon,
    /// A comma token.
    Comma,
    /// A CSS `<!--` token.
    Cdo,
    /// A CSS `-->` token.
    Cdc,
    /// A function token.
    Function,
    /// A parenthesis block token.
    ParenthesisBlock,
    /// A square-bracket block token.
    SquareBracketBlock,
    /// A curly-bracket block token.
    CurlyBracketBlock,
    /// A malformed URL token.
    BadUrl,
    /// A malformed string token.
    BadString,
    /// An unmatched closing-parenthesis token.
    CloseParenthesis,
    /// An unmatched closing-square-bracket token.
    CloseSquareBracket,
    /// An unmatched closing-curly-bracket token.
    CloseCurlyBracket,
}

/// A diagnostic-phase token category paired with its exact authored source slice.
///
/// The pair preserves the responsible authored token without exposing parser internals; it is not
/// a token-stream input and does not choose or perform recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssTokenSummary {
    pub(crate) kind: CssTokenKind,
    pub(crate) authored: String,
}

impl CssTokenSummary {
    fn clone_controlled(
        &self,
        control: &crate::values::color_adjust::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        control.step()?;
        Ok(Self {
            kind: self.kind,
            authored: control.string(&self.authored)?,
        })
    }

    #[must_use]
    /// Returns the semantic category of the encountered authored token.
    pub const fn kind(&self) -> CssTokenKind {
        self.kind
    }

    #[must_use]
    /// Returns the exact authored source spelling of the encountered token.
    pub fn authored(&self) -> &str {
        &self.authored
    }

    fn kind_from_component(component: &crate::component::CssComponentValue) -> CssTokenKind {
        use crate::component::CssBlockKind as Block;
        use crate::component::CssComponentValueRef as Component;
        use crate::component::CssValueTokenRef as Value;
        match component.view() {
            Component::Token(token) => match token {
                Value::Ident(_) => CssTokenKind::Ident,
                Value::AtKeyword(_) => CssTokenKind::AtKeyword,
                Value::Hash {
                    flag: crate::component::CssHashFlag::Id,
                    ..
                } => CssTokenKind::IdHash,
                Value::Hash {
                    flag: crate::component::CssHashFlag::Unrestricted,
                    ..
                } => CssTokenKind::Hash,
                Value::String(_) => CssTokenKind::String,
                Value::Url(_) => CssTokenKind::Url,
                Value::Delim(_) => CssTokenKind::Delim,
                Value::Number(_) => CssTokenKind::Number,
                Value::Percentage(_) => CssTokenKind::Percentage,
                Value::Dimension { .. } => CssTokenKind::Dimension,
                Value::Whitespace(_) => CssTokenKind::Whitespace,
                Value::Colon => CssTokenKind::Colon,
                Value::Semicolon => CssTokenKind::Semicolon,
                Value::Comma => CssTokenKind::Comma,
                Value::Cdo => CssTokenKind::Cdo,
                Value::Cdc => CssTokenKind::Cdc,
            },
            Component::Function(_) => CssTokenKind::Function,
            Component::Block(block) => match block.kind() {
                Block::Parenthesis => CssTokenKind::ParenthesisBlock,
                Block::SquareBracket => CssTokenKind::SquareBracketBlock,
                Block::CurlyBracket => CssTokenKind::CurlyBracketBlock,
            },
            Component::Comment(_) => CssTokenKind::Comment,
        }
    }
    fn from_checked_controlled(
        component: &crate::component::CssComponentValue,
        closing: bool,
        authored: &str,
        control: &crate::values::color_adjust::utf16_value::Control<'_>,
    ) -> Result<Self, ()> {
        control.step()?;
        let kind = if closing {
            match component.view() {
                crate::component::CssComponentValueRef::Function(_) => {
                    CssTokenKind::CloseParenthesis
                }
                crate::component::CssComponentValueRef::Block(block) => match block.kind() {
                    crate::component::CssBlockKind::Parenthesis => CssTokenKind::CloseParenthesis,
                    crate::component::CssBlockKind::SquareBracket => {
                        CssTokenKind::CloseSquareBracket
                    }
                    crate::component::CssBlockKind::CurlyBracket => CssTokenKind::CloseCurlyBracket,
                },
                _ => unreachable!("actual checked closing component"),
            }
        } else {
            Self::kind_from_component(component)
        };
        Ok(Self {
            kind,
            authored: control.string(authored)?,
        })
    }
    fn from_component(component: &crate::component::CssComponentValue) -> Self {
        let kind = Self::kind_from_component(component);
        let (authored, _) = component
            .structural_lexeme(false)
            .expect("a component owns its leaf or opening lexeme");
        Self {
            kind,
            authored: authored.to_owned(),
        }
    }

    fn from_token(token: &Token<'_>) -> Self {
        Self {
            kind: token_kind(token),
            authored: crate::tokenization::combined_operator_prefix(token)
                .map_or_else(|| token.to_css_string(), |first| first.to_string()),
        }
    }

    /// The summary of a token of the parser crate's token tree, in its
    /// canonical spelling (Syntax 3 section 9).
    pub(crate) fn from_syntax_token(token: surgeist_css_parser::Token<'_>) -> Self {
        Self {
            kind: syntax_token_kind(token),
            authored: token.to_string(),
        }
    }

    fn from_authored_token(token: &Token<'_>, authored: &str) -> Self {
        let (token, range) =
            crate::tokenization::source_token_parts(authored, token.clone(), 0..authored.len())
                .into_iter()
                .flatten()
                .next()
                .expect("an authored provider capture has a first canonical token");
        Self {
            kind: token_kind(&token),
            authored: authored[range].to_owned(),
        }
    }

    fn bang() -> Self {
        Self {
            kind: CssTokenKind::Delim,
            authored: "!".to_owned(),
        }
    }
}

pub(crate) const EXPECT_CSS_SYNTAX: CssGrammarExpectation =
    CssGrammarExpectation::new("valid CSS syntax");
const EXPECT_DECLARATION_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a declaration value");
pub(crate) const EXPECT_PROPERTY_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a value accepted by the property's grammar");
pub(crate) const EXPECT_DESCRIPTOR_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a value accepted by the descriptor grammar");
pub(crate) const EXPECT_SELECTOR: CssGrammarExpectation =
    CssGrammarExpectation::new("a supported selector");
pub(crate) const EXPECT_MEDIA_QUERY: CssGrammarExpectation =
    CssGrammarExpectation::new("a supported media query");
pub(crate) const EXPECT_COLOR: CssGrammarExpectation =
    CssGrammarExpectation::new("valid color syntax");

pub(crate) const QUALIFIED_RULE: CssProductionId = CssProductionId::new("css.qualified-rule");
const SELECTOR_LIST: CssProductionId = CssProductionId::new("baseline.selector.complex");

/// Diagnostic detail for strict parsing that reached EOF before an expected production completed.
///
/// The stored expectation identifies the violated grammar invariant; this value does not insert
/// missing syntax or perform browser-style recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnexpectedEndError {
    pub(crate) expectation: CssGrammarExpectation,
}

impl CssUnexpectedEndError {
    #[must_use]
    /// Returns the grammar expectation that remained unsatisfied at EOF.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }
}

/// Diagnostic detail for an authored token rejected by the active strict grammar.
///
/// The expectation and exact encountered token identify the violation; this value does not skip,
/// replace, or otherwise recover from the token.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnexpectedTokenError {
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: CssTokenSummary,
}

impl CssUnexpectedTokenError {
    #[must_use]
    /// Returns the grammar expectation violated by the token.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the exact authored token rejected by the grammar.
    pub const fn encountered(&self) -> &CssTokenSummary {
        &self.encountered
    }
}

/// Diagnostic detail for an authored at-rule rejected in its current grammar context.
///
/// The decoded name and expected context identify the placement invariant; this value does not
/// move, drop, or recover the at-rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssAtRulePlacementError {
    pub(crate) name: CssAtRuleName,
    pub(crate) expected_context: CssGrammarExpectation,
}

impl CssAtRulePlacementError {
    /// Returns the rule production associated with its decoded name.
    #[must_use]
    pub fn production(&self) -> CssProductionId {
        production_for_at_rule(self.name.as_str())
    }
    #[must_use]
    /// Returns the decoded authored at-rule name.
    pub const fn name(&self) -> &CssAtRuleName {
        &self.name
    }

    #[must_use]
    /// Returns the grammar context in which the at-rule was expected to appear.
    pub const fn expected_context(&self) -> CssGrammarExpectation {
        self.expected_context
    }
}

/// Diagnostic detail for an authored at-rule prelude or body rejected by strict parsing.
///
/// The enclosing [`ErrorKind`] variant supplies the prelude/body phase, while this detail retains
/// the rule, production, expectation, and responsible token. It does not recover or evaluate the
/// at-rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssAtRuleSyntaxError {
    pub(crate) name: CssAtRuleName,
    pub(crate) production: CssProductionId,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl CssAtRuleSyntaxError {
    #[must_use]
    /// Returns the decoded authored at-rule name.
    pub const fn name(&self) -> &CssAtRuleName {
        &self.name
    }

    #[must_use]
    /// Returns the grammar production being parsed when the failure occurred.
    pub const fn production(&self) -> CssProductionId {
        self.production
    }

    #[must_use]
    /// Returns the grammar expectation violated by the authored syntax.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the production ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for a retained namespace declaration repeating an existing binding.
///
/// A missing prefix identifies the default binding. The last declaration remains
/// effective, while the stylesheet is nonconforming even when the names agree.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssNamespaceRedeclarationError {
    pub(crate) prefix: Option<crate::rule::CssNamespacePrefix>,
    pub(crate) previous_position: CssSourcePosition,
}

impl CssNamespaceRedeclarationError {
    /// Returns the exact decoded prefix, or `None` for a repeated default binding.
    #[must_use]
    pub const fn prefix(&self) -> Option<&crate::rule::CssNamespacePrefix> {
        self.prefix.as_ref()
    }

    /// Returns the preceding retained declaration's at-keyword position.
    #[must_use]
    pub const fn previous_position(&self) -> CssSourcePosition {
        self.previous_position
    }
}

/// A retained underscore-leading symbolic navigation target; execution remains downstream.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssLegacyNavigationTargetError {
    pub(crate) target: String,
}
impl CssLegacyNavigationTargetError {
    /// Returns the decoded original string without case folding or frame interpretation.
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// Diagnostic detail for an authored at-rule name absent from the CSS catalog.
///
/// The decoded name preserves authored identity; this value does not reinterpret the unknown rule
/// or select a recovery action.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnknownAtRuleError {
    pub(crate) name: CssAtRuleName,
}

impl CssUnknownAtRuleError {
    #[must_use]
    /// Returns the decoded authored at-rule name.
    pub const fn name(&self) -> &CssAtRuleName {
        &self.name
    }
}

/// Diagnostic detail for a recognized authored at-rule outside the supported subset.
///
/// The decoded name and catalog feature identity distinguish unsupported syntax from unknown
/// syntax; this value does not enable, lower, or recover the rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnsupportedAtRuleError {
    pub(crate) name: CssAtRuleName,
    pub(crate) feature: CssFeatureId,
}

impl CssUnsupportedAtRuleError {
    #[must_use]
    /// Returns the decoded authored at-rule name.
    pub const fn name(&self) -> &CssAtRuleName {
        &self.name
    }

    #[must_use]
    /// Returns the support-catalog identity for the recognized at-rule.
    pub const fn feature(&self) -> CssFeatureId {
        self.feature
    }
}

/// Diagnostic detail for an authored qualified rule rejected by strict parsing.
///
/// The production, expectation, and optional responsible token identify the syntax invariant;
/// this value does not discard or recover the rule.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssQualifiedRuleError {
    pub(crate) production: CssProductionId,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl CssQualifiedRuleError {
    #[must_use]
    /// Returns the qualified-rule grammar production that rejected the syntax.
    pub const fn production(&self) -> CssProductionId {
        self.production
    }

    #[must_use]
    /// Returns the grammar expectation violated by the authored rule.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the rule ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for an authored selector rejected by the supported strict grammar.
///
/// The optional production, expectation, and token preserve parse provenance; this value does not
/// match selectors, recover forgiving selector lists, or evaluate document state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssSelectorError {
    pub(crate) production: Option<CssProductionId>,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl CssSelectorError {
    #[must_use]
    /// Returns the responsible selector production when one is available.
    pub const fn production(&self) -> Option<CssProductionId> {
        self.production
    }

    #[must_use]
    /// Returns the selector grammar expectation violated by the authored syntax.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the selector ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for an authored media query rejected by the supported strict grammar.
///
/// The optional feature name, expectation, and token preserve parse provenance; this value does
/// not evaluate the query against an environment or select recovery policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssMediaQueryError {
    pub(crate) committed_context: bool,
    pub(crate) feature: Option<CssMediaFeatureName>,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl CssMediaQueryError {
    pub(crate) const fn is_committed_context(&self) -> bool {
        self.committed_context
    }
    #[must_use]
    /// Returns the decoded authored media-feature name when one caused the failure.
    pub const fn feature(&self) -> Option<&CssMediaFeatureName> {
        self.feature.as_ref()
    }

    #[must_use]
    /// Returns the media-query grammar expectation violated by the authored syntax.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the query ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for an authored property name absent from the CSS catalog.
///
/// The decoded name preserves authored identity; this value does not reinterpret the declaration,
/// apply cascade, or select recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnknownPropertyError {
    pub(crate) name: CssPropertyName,
}

impl CssUnknownPropertyError {
    #[must_use]
    /// Returns the decoded authored property name.
    pub const fn name(&self) -> &CssPropertyName {
        &self.name
    }
}

/// Diagnostic detail for a recognized authored property outside the supported subset.
///
/// The decoded name and catalog feature identity distinguish unsupported syntax from unknown
/// syntax; this value does not enable, resolve, or apply the property.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnsupportedPropertyError {
    pub(crate) name: CssPropertyName,
    pub(crate) feature: CssFeatureId,
}

impl CssUnsupportedPropertyError {
    #[must_use]
    /// Returns the decoded authored property name.
    pub const fn name(&self) -> &CssPropertyName {
        &self.name
    }

    #[must_use]
    /// Returns the support-catalog identity for the recognized property.
    pub const fn feature(&self) -> CssFeatureId {
        self.feature
    }
}

/// Diagnostic detail for an authored value rejected by a known property's strict grammar.
///
/// The canonical property, expectation, and optional token identify the parse invariant; this
/// value does not substitute variables, resolve context, compute cascade, or recover the value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PropertyValueRejection {
    pub(crate) property: CssKnownProperty,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl PropertyValueRejection {
    #[must_use]
    /// Returns the canonical property whose value grammar rejected the authored syntax.
    pub const fn property(&self) -> CssKnownProperty {
        self.property
    }

    #[must_use]
    /// Returns the property-value grammar expectation that was not met.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the value ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CssDeclarationContext {
    SvgGlyph,
    KeyframeSvgGlyph,
    OrdinaryKnown(CssKnownProperty),
    OrdinaryCustom(CssCustomPropertyName),
    Keyframe(CssKnownProperty),
    KeyframeCustom(CssCustomPropertyName),
    Descriptor {
        at_rule: CssAtRuleName,
        descriptor: CssDescriptorName,
    },
}

/// A borrowed diagnostic-phase view of the grammar context for an authored declaration.
///
/// Exactly one variant identifies the context that owns the declaration invariant. The view does
/// not apply declarations, compute cascade, substitute custom properties, or choose recovery.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssDeclarationContextRef<'a> {
    /// The independent SVG glyph definition in an ordinary declaration.
    SvgGlyphOrientationVertical,
    /// The independent SVG glyph definition inside an authored keyframe.
    KeyframeSvgGlyphOrientationVertical,
    /// A declaration for a known ordinary property.
    KnownProperty(CssKnownProperty),
    /// A declaration for a case-sensitive authored custom property.
    CustomProperty(&'a CssCustomPropertyName),
    /// A declaration for a known property inside an authored keyframe.
    Keyframe(CssKnownProperty),
    /// A case-sensitive custom property declaration inside an authored keyframe.
    KeyframeCustomProperty(&'a CssCustomPropertyName),
    /// A descriptor declaration owned by an authored at-rule.
    Descriptor {
        /// The decoded authored name of the owning at-rule.
        at_rule: &'a CssAtRuleName,
        /// The decoded authored descriptor name.
        descriptor: &'a CssDescriptorName,
    },
}

impl CssDeclarationContext {
    const fn as_ref(&self) -> CssDeclarationContextRef<'_> {
        match self {
            Self::SvgGlyph => CssDeclarationContextRef::SvgGlyphOrientationVertical,
            Self::KeyframeSvgGlyph => CssDeclarationContextRef::KeyframeSvgGlyphOrientationVertical,
            Self::OrdinaryKnown(property) => CssDeclarationContextRef::KnownProperty(*property),
            Self::OrdinaryCustom(property) => CssDeclarationContextRef::CustomProperty(property),
            Self::Keyframe(property) => CssDeclarationContextRef::Keyframe(*property),
            Self::KeyframeCustom(property) => {
                CssDeclarationContextRef::KeyframeCustomProperty(property)
            }
            Self::Descriptor {
                at_rule,
                descriptor,
            } => CssDeclarationContextRef::Descriptor {
                at_rule,
                descriptor,
            },
        }
    }
}

/// Diagnostic detail for a malformed or misplaced authored declaration annotation.
///
/// The declaration context and exact token identify the strict grammar violation; this value does
/// not apply importance, mutate the declaration, or recover it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssDeclarationAnnotationError {
    pub(crate) context: CssDeclarationContext,
    pub(crate) encountered: CssTokenSummary,
}

impl CssDeclarationAnnotationError {
    #[must_use]
    /// Returns the grammar context that owns the declaration.
    pub const fn context(&self) -> CssDeclarationContextRef<'_> {
        self.context.as_ref()
    }

    #[must_use]
    /// Returns the exact authored annotation token rejected by the grammar.
    pub const fn encountered(&self) -> &CssTokenSummary {
        &self.encountered
    }
}

/// Diagnostic detail for an authored descriptor name unknown to its at-rule grammar.
///
/// The owning rule and descriptor names preserve authored identity; this value does not
/// reinterpret,
/// apply, or recover the descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnknownDescriptorError {
    pub(crate) at_rule: CssAtRuleName,
    pub(crate) descriptor: CssDescriptorName,
}

impl CssUnknownDescriptorError {
    #[must_use]
    /// Returns the decoded authored name of the owning at-rule.
    pub const fn at_rule(&self) -> &CssAtRuleName {
        &self.at_rule
    }

    #[must_use]
    /// Returns the decoded authored descriptor name.
    pub const fn descriptor(&self) -> &CssDescriptorName {
        &self.descriptor
    }
}

/// Diagnostic detail for a recognized authored descriptor outside the supported subset.
///
/// The owning rule, descriptor, and catalog identity distinguish unsupported syntax from unknown
/// syntax; this value does not enable, apply, or recover the descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssUnsupportedDescriptorError {
    pub(crate) at_rule: CssAtRuleName,
    pub(crate) descriptor: CssDescriptorName,
    pub(crate) feature: CssFeatureId,
}

impl CssUnsupportedDescriptorError {
    #[must_use]
    /// Returns the decoded authored name of the owning at-rule.
    pub const fn at_rule(&self) -> &CssAtRuleName {
        &self.at_rule
    }

    #[must_use]
    /// Returns the decoded authored descriptor name.
    pub const fn descriptor(&self) -> &CssDescriptorName {
        &self.descriptor
    }

    #[must_use]
    /// Returns the support-catalog identity for the recognized descriptor.
    pub const fn feature(&self) -> CssFeatureId {
        self.feature
    }
}

/// Diagnostic detail for an authored value rejected by a known descriptor's strict grammar.
///
/// The owning rule, descriptor, expectation, and optional token identify the parse invariant; this
/// value does not resolve resources, apply descriptor effects, or recover the value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssDescriptorValueError {
    pub(crate) origin: DiagnosticOrigin,
    pub(crate) at_rule: CssAtRuleName,
    // Keep the largest error payload compact when retaining explicit origins.
    pub(crate) descriptor: Box<CssDescriptorName>,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticOrigin {
    Inferred,
    Token,
    EndOfInput,
}

impl CssDescriptorValueError {
    #[must_use]
    /// Returns the decoded authored name of the owning at-rule.
    pub const fn at_rule(&self) -> &CssAtRuleName {
        &self.at_rule
    }

    #[must_use]
    /// Returns the decoded authored descriptor name.
    pub const fn descriptor(&self) -> &CssDescriptorName {
        &self.descriptor
    }

    #[must_use]
    /// Returns the descriptor-value grammar expectation that was not met.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the value ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for authored descriptors rejected as an invalid combination.
///
/// The owning rule, responsible descriptor, and conflict set identify the validation invariant;
/// this value does not choose a descriptor, apply rule effects, or recover the block.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssDescriptorCombinationError {
    pub(crate) at_rule: CssAtRuleName,
    pub(crate) responsible: CssDescriptorName,
    pub(crate) conflicting: Vec<CssDescriptorName>,
}

impl CssDescriptorCombinationError {
    #[must_use]
    /// Returns the decoded authored name of the owning at-rule.
    pub const fn at_rule(&self) -> &CssAtRuleName {
        &self.at_rule
    }

    #[must_use]
    /// Returns the descriptor that made the authored combination invalid.
    pub const fn responsible(&self) -> &CssDescriptorName {
        &self.responsible
    }

    #[must_use]
    /// Returns the descriptors that conflict with the responsible descriptor.
    pub fn conflicting(&self) -> &[CssDescriptorName] {
        &self.conflicting
    }
}

/// Diagnostic detail for authored color syntax rejected by strict parsing.
///
/// The optional component, expectation, and token identify the color grammar violation; this value
/// does not resolve system colors, evaluate relative channels, mix, convert, or recover colors.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssColorSyntaxError {
    pub(crate) component: Option<CssColorComponentName>,
    pub(crate) expectation: CssGrammarExpectation,
    pub(crate) encountered: Option<CssTokenSummary>,
}

impl CssColorSyntaxError {
    #[must_use]
    /// Returns the semantic color component responsible for the failure, when known.
    pub const fn component(&self) -> Option<&CssColorComponentName> {
        self.component.as_ref()
    }

    #[must_use]
    /// Returns the color grammar expectation that was not met.
    pub const fn expectation(&self) -> CssGrammarExpectation {
        self.expectation
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` when the color ended at EOF.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.encountered.as_ref()
    }
}

/// Diagnostic detail for authored CSS nesting beyond the parser's structural limit.
///
/// The configured limit and enclosing production identify the rejected parse invariant; this value
/// does not flatten additional nesting, recover the construct, or perform selector matching.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CssNestingLimitError {
    pub(crate) limit: u32,
    pub(crate) enclosing_production: CssProductionId,
}

impl CssNestingLimitError {
    #[must_use]
    /// Returns the maximum nesting depth accepted by the strict parser.
    ///
    /// For a nesting limit this is ordinarily the 256-level structural limit.
    /// For a parser-thread refusal it is the depth trusted on the caller's
    /// stack: 128 for stylesheets and style attributes, 63 for fragments.
    pub const fn limit(&self) -> u32 {
        self.limit
    }

    #[must_use]
    /// Returns the grammar production enclosing the excessive nesting.
    pub const fn enclosing_production(&self) -> CssProductionId {
        self.enclosing_production
    }
}

/// The tokenizer's parse-error condition while processing an authored escape.
///
/// These are lexical facts independent of surrounding grammar retention. EOF
/// still satisfies valid-escape lookahead in CSS Syntax 3 §4.3.8; consuming its
/// escaped code point reports a parse error and returns U+FFFD (§4.3.7).
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssEscapeFailure {
    /// Backslash-newline is not a valid escape; recovery returns a backslash delimiter.
    Newline,
    /// An escape reached EOF; recovery returns the replacement character U+FFFD.
    EndOfInput,
}

/// The structured diagnostic-phase detail for one CSS grammar or conformance violation.
///
/// Every variant carries the payload required by its stable [`CssErrorCode`] and preserves authored
/// provenance without a free-form catch-all. [`CssError`] exposes the payload through its fact
/// accessors; the representation stays private to the crate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ErrorKind {
    /// Tokenizer escape processing required lexical recovery.
    EscapeParseError(CssEscapeFailure),
    /// Strict parsing reached EOF before an expected production completed.
    UnexpectedEnd(CssUnexpectedEndError),
    /// Strict parsing encountered a token rejected by the active grammar.
    UnexpectedToken(CssUnexpectedTokenError),
    /// An authored at-rule appeared outside its permitted grammar context.
    InvalidAtRulePlacement(CssAtRulePlacementError),
    /// An authored at-rule prelude was invalid.
    InvalidAtRulePrelude(CssAtRuleSyntaxError),
    /// An authored at-rule body was invalid.
    InvalidAtRuleBody(CssAtRuleSyntaxError),
    /// A retained namespace declaration repeated a named or default binding.
    NamespaceRedeclaration(CssNamespaceRedeclarationError),
    /// Browser parsing retained a nonconforming symbolic navigation target.
    LegacyNavigationTarget(CssLegacyNavigationTargetError),
    /// An authored at-rule name was not recognized.
    UnknownAtRule(CssUnknownAtRuleError),
    /// An authored at-rule was recognized but unsupported.
    UnsupportedAtRule(CssUnsupportedAtRuleError),
    /// An authored qualified rule was invalid.
    InvalidQualifiedRule(CssQualifiedRuleError),
    /// An authored selector was invalid or outside the supported grammar.
    InvalidSelector(CssSelectorError),
    /// An authored media query was invalid or outside the supported grammar.
    InvalidMediaQuery(CssMediaQueryError),
    /// An authored property name was not recognized.
    UnknownProperty(CssUnknownPropertyError),
    /// An authored property was recognized but unsupported.
    UnsupportedProperty(CssUnsupportedPropertyError),
    /// An authored property value was invalid.
    InvalidPropertyValue(PropertyValueRejection),
    /// A retained declaration value could not form a checked component sequence.
    InvalidComponentValue(Box<crate::component::CssComponentValueError>),
    /// An authored declaration annotation was malformed or misplaced.
    InvalidDeclarationAnnotation(CssDeclarationAnnotationError),
    /// An authored descriptor name was not recognized for its at-rule.
    UnknownDescriptor(CssUnknownDescriptorError),
    /// An authored descriptor was recognized but unsupported.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no descriptor grammar reports this public code yet; CssError keeps its facts"
        )
    )]
    UnsupportedDescriptor(CssUnsupportedDescriptorError),
    /// An authored descriptor value was invalid.
    InvalidDescriptorValue(CssDescriptorValueError),
    /// Authored descriptors formed an invalid combination.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "no descriptor grammar reports this public code yet; CssError keeps its facts"
        )
    )]
    InvalidDescriptorCombination(CssDescriptorCombinationError),
    /// Authored color syntax was invalid or outside the supported grammar.
    InvalidColorSyntax(CssColorSyntaxError),
    /// Authored nesting exceeded the parser's configured structural limit.
    NestingLimit(CssNestingLimitError),
    /// Deep input was refused because the bounded parser thread could not start.
    /// The limit is the depth trusted on the caller's stack.
    ParserUnavailable(CssNestingLimitError),
}

/// A structured grammar or conformance diagnostic at one semantic authored-source position.
///
/// [`Self::code`] classifies the violated authored invariant and the fact accessors expose its
/// payload; each accessor names the codes that populate it. Match on [`Self::code`] with a
/// wildcard arm, since [`CssErrorCode`] is non-exhaustive. This error reports the failure but
/// does not perform browser-style recovery, cascade, substitution, selector matching,
/// contextual resolution, or resource loading.
///
/// ```
/// use surgeist_css::diagnostic::CssErrorCode;
/// use surgeist_css::CssParser;
///
/// let report = CssParser::default().parse_block_contents("width: red");
/// let error = report.diagnostics()[0].error();
/// assert_eq!(error.code(), CssErrorCode::InvalidPropertyValue);
/// assert_eq!(error.encountered().map(|token| token.authored()), Some("red"));
/// assert!(!error.is_resource_refusal());
/// ```
///
/// The structured kind behind the accessors is private (CF23):
///
/// ```compile_fail,E0432
/// use surgeist_css::diagnostic::ErrorKind;
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssError {
    pub(crate) kind: ErrorKind,
    pub(crate) position: CssSourcePosition,
}

impl CssError {
    pub(crate) fn resolve_original_coordinates(&mut self, source: &str) {
        self.position =
            CssSourcePosition::from_byte_offset_in(source, self.position.byte_offset().value());
    }

    fn at(location: cssparser::SourceLocation, kind: ErrorKind) -> Self {
        Self {
            kind,
            position: CssSourcePosition::from_source_location(location),
        }
    }

    fn at_exact_nonzero_byte_offset(position: CssSourcePosition, kind: ErrorKind) -> Self {
        debug_assert_ne!(position.byte_offset().value(), 0);
        Self { kind, position }
    }

    /// The private structured detail behind the public fact accessors.
    pub(crate) const fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    #[must_use]
    /// Returns the stable machine-readable category of the violated invariant.
    pub const fn code(&self) -> CssErrorCode {
        self.kind.code()
    }

    #[must_use]
    /// Returns the semantic authored-source position of the responsible token or missing token.
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }

    #[must_use]
    /// Returns the grammar expectation that the authored syntax did not meet.
    ///
    /// Populated for `UnexpectedEnd`, `UnexpectedToken`, `InvalidAtRulePlacement` (the expected
    /// context), `InvalidAtRulePrelude`, `InvalidAtRuleBody`, `InvalidQualifiedRule`,
    /// `InvalidSelector`, `InvalidMediaQuery`, `InvalidPropertyValue`, `InvalidDescriptorValue`
    /// and `InvalidColorSyntax`.
    pub const fn expectation(&self) -> Option<CssGrammarExpectation> {
        self.kind.expectation()
    }

    #[must_use]
    /// Returns the responsible authored token, or `None` at EOF or for codes without one.
    ///
    /// Populated for `UnexpectedToken` and `InvalidDeclarationAnnotation`, and, when a token was
    /// responsible, for `InvalidAtRulePrelude`, `InvalidAtRuleBody`, `InvalidQualifiedRule`,
    /// `InvalidSelector`, `InvalidMediaQuery`, `InvalidPropertyValue`, `InvalidDescriptorValue`
    /// and `InvalidColorSyntax`.
    pub const fn encountered(&self) -> Option<&CssTokenSummary> {
        self.kind.encountered()
    }

    #[must_use]
    /// Returns the decoded authored at-rule name that owns the failure.
    ///
    /// Populated for `InvalidAtRulePlacement`, `InvalidAtRulePrelude`, `InvalidAtRuleBody`,
    /// `UnknownAtRule`, `UnsupportedAtRule`, `UnknownDescriptor`, `UnsupportedDescriptor`,
    /// `InvalidDescriptorValue` and `InvalidDescriptorCombination`.
    pub fn at_rule(&self) -> Option<&CssAtRuleName> {
        self.kind.at_rule()
    }

    #[must_use]
    /// Returns the grammar production active at the failure.
    ///
    /// Populated for `InvalidAtRulePlacement`, `InvalidAtRulePrelude`, `InvalidAtRuleBody`,
    /// `InvalidQualifiedRule`, `NestingLimit` and `ParserUnavailable` (the enclosing
    /// production), and for `InvalidSelector` when a selector production is known.
    pub fn production(&self) -> Option<CssProductionId> {
        self.kind.production()
    }

    #[must_use]
    /// Returns the repeated namespace prefix of a `NamespaceRedeclaration`.
    ///
    /// `None` both for a repeated default namespace and for every other code, so read it after
    /// checking [`Self::code`].
    pub const fn namespace_prefix(&self) -> Option<&crate::rule::CssNamespacePrefix> {
        match &self.kind {
            ErrorKind::NamespaceRedeclaration(detail) => detail.prefix(),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the at-keyword position of the earlier declaration a `NamespaceRedeclaration`
    /// repeats.
    pub const fn previous_position(&self) -> Option<CssSourcePosition> {
        match &self.kind {
            ErrorKind::NamespaceRedeclaration(detail) => Some(detail.previous_position()),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the decoded underscore-leading target of a `LegacyNavigationTarget`, without
    /// case folding or frame interpretation.
    pub fn navigation_target(&self) -> Option<&str> {
        match &self.kind {
            ErrorKind::LegacyNavigationTarget(detail) => Some(detail.target()),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the support-catalog identity of recognized but unsupported syntax.
    ///
    /// Populated for `UnsupportedAtRule`, `UnsupportedProperty` and `UnsupportedDescriptor`.
    pub const fn feature(&self) -> Option<CssFeatureId> {
        match &self.kind {
            ErrorKind::UnsupportedAtRule(detail) => Some(detail.feature()),
            ErrorKind::UnsupportedProperty(detail) => Some(detail.feature()),
            ErrorKind::UnsupportedDescriptor(detail) => Some(detail.feature()),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the decoded media-feature name that caused an `InvalidMediaQuery`, when known.
    pub const fn media_feature(&self) -> Option<&CssMediaFeatureName> {
        match &self.kind {
            ErrorKind::InvalidMediaQuery(detail) => detail.feature(),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the decoded authored property name of an `UnknownProperty` or
    /// `UnsupportedProperty`.
    pub const fn property_name(&self) -> Option<&CssPropertyName> {
        match &self.kind {
            ErrorKind::UnknownProperty(detail) => Some(detail.name()),
            ErrorKind::UnsupportedProperty(detail) => Some(detail.name()),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the canonical property whose value grammar rejected an `InvalidPropertyValue`.
    pub const fn known_property(&self) -> Option<CssKnownProperty> {
        self.kind.known_property()
    }

    #[must_use]
    /// Returns the grammar context owning an `InvalidDeclarationAnnotation`.
    pub const fn declaration_context(&self) -> Option<CssDeclarationContextRef<'_>> {
        match &self.kind {
            ErrorKind::InvalidDeclarationAnnotation(detail) => Some(detail.context()),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the decoded authored descriptor name.
    ///
    /// Populated for `UnknownDescriptor`, `UnsupportedDescriptor` and `InvalidDescriptorValue`,
    /// and for `InvalidDescriptorCombination`, where it is the descriptor that made the
    /// combination invalid.
    pub fn descriptor(&self) -> Option<&CssDescriptorName> {
        self.kind.descriptor()
    }

    #[must_use]
    /// Returns the descriptors an `InvalidDescriptorCombination` conflicts with; empty for
    /// other codes.
    pub fn conflicting_descriptors(&self) -> &[CssDescriptorName] {
        match &self.kind {
            ErrorKind::InvalidDescriptorCombination(detail) => detail.conflicting(),
            _ => &[],
        }
    }

    #[must_use]
    /// Returns the colour component responsible for an `InvalidColorSyntax`, when known.
    pub const fn color_component(&self) -> Option<&CssColorComponentName> {
        self.kind.color_component()
    }

    #[must_use]
    /// Returns the depth limit of a `NestingLimit`, or the depth trusted on the caller's stack
    /// for a `ParserUnavailable` refusal.
    pub const fn nesting_limit(&self) -> Option<u32> {
        match &self.kind {
            ErrorKind::NestingLimit(detail) | ErrorKind::ParserUnavailable(detail) => {
                Some(detail.limit())
            }
            _ => None,
        }
    }

    #[must_use]
    /// Returns the lexical escape failure of an `EscapeParseError`.
    pub const fn escape_failure(&self) -> Option<CssEscapeFailure> {
        match self.kind {
            ErrorKind::EscapeParseError(failure) => Some(failure),
            _ => None,
        }
    }

    #[must_use]
    /// Returns the component failure of an `InvalidComponentValue`.
    pub fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match &self.kind {
            ErrorKind::InvalidComponentValue(detail) => Some(detail),
            _ => None,
        }
    }

    #[must_use]
    /// Returns whether a parser or component resource refused the input, rather than the input
    /// violating a grammar.
    ///
    /// True for `NestingLimit`, `ParserUnavailable`, and an `InvalidComponentValue` whose
    /// component error is a resource limit ([`crate::component::CssComponentValueError::is_resource_limit`]).
    /// A consumer that publishes only complete results aborts on these.
    pub fn is_resource_refusal(&self) -> bool {
        self.kind.is_resource_refusal()
    }

    /// Maps a checked grammar failure back to its supplied component input.
    /// Component failures already retain their responsible original token;
    /// their authored positions must not index the shorter serialized input.
    pub(crate) fn component_input_origin(
        &self,
        serialized: &crate::component::CssSerializedValue,
    ) -> crate::component::CssSerializedOrigin {
        if let ErrorKind::InvalidComponentValue(detail) = self.kind() {
            return crate::component::CssSerializedOrigin::Token(detail.origin().clone());
        }
        serialized
            .origin_at(self.position().byte_offset().value())
            .expect(
                "checked grammar reports a serialized input cursor or retains a component origin",
            )
            .clone()
    }
}

impl ErrorKind {
    pub(crate) const fn code(&self) -> CssErrorCode {
        match self {
            Self::EscapeParseError(_) => CssErrorCode::EscapeParseError,
            Self::UnexpectedEnd(_) => CssErrorCode::UnexpectedEnd,
            Self::UnexpectedToken(_) => CssErrorCode::UnexpectedToken,
            Self::InvalidAtRulePlacement(_) => CssErrorCode::InvalidAtRulePlacement,
            Self::InvalidAtRulePrelude(_) => CssErrorCode::InvalidAtRulePrelude,
            Self::InvalidAtRuleBody(_) => CssErrorCode::InvalidAtRuleBody,
            Self::NamespaceRedeclaration(_) => CssErrorCode::NamespaceRedeclaration,
            Self::LegacyNavigationTarget(_) => CssErrorCode::LegacyNavigationTarget,
            Self::UnknownAtRule(_) => CssErrorCode::UnknownAtRule,
            Self::UnsupportedAtRule(_) => CssErrorCode::UnsupportedAtRule,
            Self::InvalidQualifiedRule(_) => CssErrorCode::InvalidQualifiedRule,
            Self::InvalidSelector(_) => CssErrorCode::InvalidSelector,
            Self::InvalidMediaQuery(_) => CssErrorCode::InvalidMediaQuery,
            Self::UnknownProperty(_) => CssErrorCode::UnknownProperty,
            Self::UnsupportedProperty(_) => CssErrorCode::UnsupportedProperty,
            Self::InvalidPropertyValue(_) => CssErrorCode::InvalidPropertyValue,
            Self::InvalidComponentValue(_) => CssErrorCode::InvalidComponentValue,
            Self::InvalidDeclarationAnnotation(_) => CssErrorCode::InvalidDeclarationAnnotation,
            Self::UnknownDescriptor(_) => CssErrorCode::UnknownDescriptor,
            Self::UnsupportedDescriptor(_) => CssErrorCode::UnsupportedDescriptor,
            Self::InvalidDescriptorValue(_) => CssErrorCode::InvalidDescriptorValue,
            Self::InvalidDescriptorCombination(_) => CssErrorCode::InvalidDescriptorCombination,
            Self::InvalidColorSyntax(_) => CssErrorCode::InvalidColorSyntax,
            Self::NestingLimit(_) => CssErrorCode::NestingLimit,
            Self::ParserUnavailable(_) => CssErrorCode::ParserUnavailable,
        }
    }

    pub(crate) const fn expectation(&self) -> Option<CssGrammarExpectation> {
        match self {
            Self::UnexpectedEnd(detail) => Some(detail.expectation()),
            Self::UnexpectedToken(detail) => Some(detail.expectation()),
            Self::InvalidAtRulePlacement(detail) => Some(detail.expected_context()),
            Self::InvalidAtRulePrelude(detail) | Self::InvalidAtRuleBody(detail) => {
                Some(detail.expectation())
            }
            Self::InvalidQualifiedRule(detail) => Some(detail.expectation()),
            Self::InvalidSelector(detail) => Some(detail.expectation()),
            Self::InvalidMediaQuery(detail) => Some(detail.expectation()),
            Self::InvalidPropertyValue(detail) => Some(detail.expectation()),
            Self::InvalidDescriptorValue(detail) => Some(detail.expectation()),
            Self::InvalidColorSyntax(detail) => Some(detail.expectation()),
            _ => None,
        }
    }

    pub(crate) const fn encountered(&self) -> Option<&CssTokenSummary> {
        match self {
            Self::UnexpectedToken(detail) => Some(detail.encountered()),
            Self::InvalidAtRulePrelude(detail) | Self::InvalidAtRuleBody(detail) => {
                detail.encountered()
            }
            Self::InvalidQualifiedRule(detail) => detail.encountered(),
            Self::InvalidSelector(detail) => detail.encountered(),
            Self::InvalidMediaQuery(detail) => detail.encountered(),
            Self::InvalidPropertyValue(detail) => detail.encountered(),
            Self::InvalidDeclarationAnnotation(detail) => Some(detail.encountered()),
            Self::InvalidDescriptorValue(detail) => detail.encountered(),
            Self::InvalidColorSyntax(detail) => detail.encountered(),
            _ => None,
        }
    }

    pub(crate) fn at_rule(&self) -> Option<&CssAtRuleName> {
        match self {
            Self::InvalidAtRulePlacement(detail) => Some(detail.name()),
            Self::InvalidAtRulePrelude(detail) | Self::InvalidAtRuleBody(detail) => {
                Some(detail.name())
            }
            Self::UnknownAtRule(detail) => Some(detail.name()),
            Self::UnsupportedAtRule(detail) => Some(detail.name()),
            Self::UnknownDescriptor(detail) => Some(detail.at_rule()),
            Self::UnsupportedDescriptor(detail) => Some(detail.at_rule()),
            Self::InvalidDescriptorValue(detail) => Some(detail.at_rule()),
            Self::InvalidDescriptorCombination(detail) => Some(detail.at_rule()),
            _ => None,
        }
    }

    pub(crate) fn production(&self) -> Option<CssProductionId> {
        match self {
            Self::InvalidAtRulePlacement(detail) => Some(detail.production()),
            Self::InvalidAtRulePrelude(detail) | Self::InvalidAtRuleBody(detail) => {
                Some(detail.production())
            }
            Self::InvalidQualifiedRule(detail) => Some(detail.production()),
            Self::InvalidSelector(detail) => detail.production(),
            Self::NestingLimit(detail) | Self::ParserUnavailable(detail) => {
                Some(detail.enclosing_production())
            }
            _ => None,
        }
    }

    pub(crate) const fn known_property(&self) -> Option<CssKnownProperty> {
        match self {
            Self::InvalidPropertyValue(detail) => Some(detail.property()),
            _ => None,
        }
    }

    pub(crate) fn descriptor(&self) -> Option<&CssDescriptorName> {
        match self {
            Self::UnknownDescriptor(detail) => Some(detail.descriptor()),
            Self::UnsupportedDescriptor(detail) => Some(detail.descriptor()),
            Self::InvalidDescriptorValue(detail) => Some(detail.descriptor()),
            Self::InvalidDescriptorCombination(detail) => Some(detail.responsible()),
            _ => None,
        }
    }

    pub(crate) const fn color_component(&self) -> Option<&CssColorComponentName> {
        match self {
            Self::InvalidColorSyntax(detail) => detail.component(),
            _ => None,
        }
    }

    pub(crate) fn is_resource_refusal(&self) -> bool {
        match self {
            Self::NestingLimit(_) | Self::ParserUnavailable(_) => true,
            Self::InvalidComponentValue(detail) => detail.is_resource_limit(),
            _ => false,
        }
    }

    /// One summary sentence built from the payload's facts, without a position.
    pub(crate) fn describe(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EscapeParseError(CssEscapeFailure::Newline) => {
                formatter.write_str("invalid CSS escape: a backslash precedes a newline")
            }
            Self::EscapeParseError(CssEscapeFailure::EndOfInput) => {
                formatter.write_str("invalid CSS escape: the input ends after a backslash")
            }
            Self::UnexpectedEnd(_) => formatter.write_str("unexpected end of CSS input"),
            Self::UnexpectedToken(_) => formatter.write_str("unexpected CSS token"),
            Self::InvalidAtRulePlacement(detail) => write!(
                formatter,
                "@{} appears outside its permitted context",
                detail.name().as_str()
            ),
            Self::InvalidAtRulePrelude(detail) => {
                write!(formatter, "invalid @{} prelude", detail.name().as_str())
            }
            Self::InvalidAtRuleBody(detail) => {
                write!(formatter, "invalid @{} body", detail.name().as_str())
            }
            Self::NamespaceRedeclaration(detail) => match detail.prefix() {
                Some(prefix) => write!(
                    formatter,
                    "namespace prefix `{}` is declared again",
                    prefix.as_str()
                ),
                None => formatter.write_str("the default namespace is declared again"),
            },
            Self::LegacyNavigationTarget(detail) => write!(
                formatter,
                "navigation target `{}` begins with an underscore",
                detail.target()
            ),
            Self::UnknownAtRule(detail) => {
                write!(formatter, "unknown at-rule @{}", detail.name().as_str())
            }
            Self::UnsupportedAtRule(detail) => write!(
                formatter,
                "unsupported at-rule @{} ({})",
                detail.name().as_str(),
                detail.feature().as_str()
            ),
            Self::InvalidQualifiedRule(_) => formatter.write_str("invalid qualified rule"),
            Self::InvalidSelector(_) => formatter.write_str("invalid selector"),
            Self::InvalidMediaQuery(detail) => match detail.feature() {
                Some(feature) => write!(
                    formatter,
                    "invalid media query at feature `{}`",
                    feature.as_str()
                ),
                None => formatter.write_str("invalid media query"),
            },
            Self::UnknownProperty(detail) => {
                write!(formatter, "unknown property `{}`", detail.name().as_str())
            }
            Self::UnsupportedProperty(detail) => write!(
                formatter,
                "unsupported property `{}` ({})",
                detail.name().as_str(),
                detail.feature().as_str()
            ),
            Self::InvalidPropertyValue(detail) => write!(
                formatter,
                "invalid value for property `{}`",
                detail.property().canonical_name()
            ),
            Self::InvalidComponentValue(detail) => {
                formatter.write_str("invalid component value: ")?;
                detail.describe(formatter)
            }
            Self::InvalidDeclarationAnnotation(_) => {
                formatter.write_str("invalid declaration annotation")
            }
            Self::UnknownDescriptor(detail) => write!(
                formatter,
                "unknown descriptor `{}` in @{}",
                detail.descriptor().as_str(),
                detail.at_rule().as_str()
            ),
            Self::UnsupportedDescriptor(detail) => write!(
                formatter,
                "unsupported descriptor `{}` in @{} ({})",
                detail.descriptor().as_str(),
                detail.at_rule().as_str(),
                detail.feature().as_str()
            ),
            Self::InvalidDescriptorValue(detail) => write!(
                formatter,
                "invalid value for descriptor `{}` in @{}",
                detail.descriptor().as_str(),
                detail.at_rule().as_str()
            ),
            Self::InvalidDescriptorCombination(detail) => {
                write!(
                    formatter,
                    "descriptor `{}` in @{} conflicts with",
                    detail.responsible().as_str(),
                    detail.at_rule().as_str()
                )?;
                for (index, conflicting) in detail.conflicting().iter().enumerate() {
                    let separator = if index == 0 { " " } else { ", " };
                    write!(formatter, "{separator}`{}`", conflicting.as_str())?;
                }
                Ok(())
            }
            Self::InvalidColorSyntax(detail) => match detail.component() {
                Some(component) => write!(
                    formatter,
                    "invalid color syntax in component `{}`",
                    component.as_str()
                ),
                None => formatter.write_str("invalid color syntax"),
            },
            Self::NestingLimit(detail) => write!(
                formatter,
                "CSS nesting exceeds the limit of {} levels",
                detail.limit()
            ),
            Self::ParserUnavailable(detail) => write!(
                formatter,
                "input nested deeper than {} levels was refused because the CSS parser thread \
                 is unavailable",
                detail.limit()
            ),
        }?;
        if let Some(expectation) = self.expectation()
            && !matches!(self, Self::InvalidAtRulePlacement(_))
        {
            write!(formatter, "; expected {}", expectation.as_str())?;
        }
        if let Self::InvalidAtRulePlacement(detail) = self {
            write!(
                formatter,
                "; permitted: {}",
                detail.expected_context().as_str()
            )?;
        }
        if let Some(token) = self.encountered() {
            write!(formatter, ", found `{}`", token.authored())?;
        }
        Ok(())
    }
}

impl fmt::Display for CssError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.describe(formatter)?;
        write!(formatter, " at {}", self.position)
    }
}

impl std::error::Error for CssError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}

pub(crate) fn from_parse_error(source: &str, error: ParseError<'_, CssError>) -> CssError {
    match error.kind {
        ParseErrorKind::Custom(error) => error,
        ParseErrorKind::Basic(kind) => basic_error(error.location, kind),
    }
    .resolve_source(source)
}

pub(crate) fn is_nesting_limit_error(error: &ParseError<'_, CssError>) -> bool {
    matches!(
        error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::NestingLimit(_),
            ..
        })
    )
}

pub(crate) fn nesting_limit<'i>(
    source: &str,
    byte_offset: usize,
    limit: u32,
    enclosing_production: &'static str,
) -> ParseError<'i, CssError> {
    depth_error(
        source,
        byte_offset,
        ErrorKind::NestingLimit(CssNestingLimitError {
            limit,
            enclosing_production: CssProductionId::new(enclosing_production),
        }),
    )
}

/// The refusal of input deeper than `direct_depth` when the bounded parser
/// thread cannot start (#1277), at the first opening beyond that depth.
pub(crate) fn parser_unavailable<'i>(
    source: &str,
    byte_offset: usize,
    direct_depth: u32,
    enclosing_production: &'static str,
) -> ParseError<'i, CssError> {
    depth_error(
        source,
        byte_offset,
        ErrorKind::ParserUnavailable(CssNestingLimitError {
            limit: direct_depth,
            enclosing_production: CssProductionId::new(enclosing_production),
        }),
    )
}

fn depth_error<'i>(source: &str, byte_offset: usize, kind: ErrorKind) -> ParseError<'i, CssError> {
    let position = CssSourcePosition::from_byte_offset_in(source, byte_offset);
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value().saturating_add(1),
    };
    ParseError {
        kind: ParseErrorKind::Custom(CssError { kind, position }),
        location,
    }
}

pub(crate) fn legacy_navigation_target(target: &str, position: CssSourcePosition) -> CssError {
    CssError {
        kind: ErrorKind::LegacyNavigationTarget(CssLegacyNavigationTargetError {
            target: target.to_owned(),
        }),
        position,
    }
}

pub(crate) fn namespace_redeclaration(
    prefix: Option<crate::rule::CssNamespacePrefix>,
    position: CssSourcePosition,
    previous_position: CssSourcePosition,
) -> CssError {
    CssError {
        kind: ErrorKind::NamespaceRedeclaration(CssNamespaceRedeclarationError {
            prefix,
            previous_position,
        }),
        position,
    }
}

pub(crate) fn implicit_eof(source: &str) -> CssError {
    CssError {
        kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_CSS_SYNTAX,
        }),
        position: CssSourcePosition::from_byte_offset_in(source, source.len()),
    }
}

pub(crate) fn unterminated_at_rule(source: &str, byte_offset: usize) -> CssError {
    CssError {
        kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: CssGrammarExpectation::new("a semicolon or block terminating an at-rule"),
        }),
        position: CssSourcePosition::from_byte_offset_in(source, byte_offset),
    }
}

pub(crate) fn escape_parse_error(
    source: &str,
    byte_offset: usize,
    detail: CssEscapeFailure,
) -> CssError {
    CssError {
        kind: ErrorKind::EscapeParseError(detail),
        position: CssSourcePosition::from_byte_offset_in(source, byte_offset),
    }
}

/// A name-free grammar failure from an actual retained source token or opener.
pub(crate) fn unexpected_component_value<'i>(
    component: &crate::component::CssComponentValue,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    let crate::component::CssValueOrigin::Parsed(origin) = component.origin() else {
        unreachable!("source value grammar owns parsed component origins")
    };
    unexpected_component_value_at(component, expectation, origin.span().start())
}

pub(crate) fn unexpected_component_value_at<'i>(
    component: &crate::component::CssComponentValue,
    expectation: &'static str,
    position: CssSourcePosition,
) -> ParseError<'i, CssError> {
    let encountered = CssTokenSummary::from_component(component);
    ParseError {
        location: cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        },
        kind: ParseErrorKind::Custom(CssError {
            position,
            kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                expectation: CssGrammarExpectation::new(expectation),
                encountered,
            }),
        }),
    }
}

pub(crate) fn unexpected_end_at<'i>(
    location: cssparser::SourceLocation,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: CssGrammarExpectation::new(expectation),
        }),
    )
}

/// Whether `unit` holds a `<{-token>`. A `{` byte alone is not one: an escaped
/// `\{` is part of an ident (Syntax 3 (2026 CRD) §4.3.7), and a string,
/// comment or URL can hold the byte too.
fn contains_curly_block(unit: &str) -> bool {
    let mut offset = 0;
    while let Some((_, end, token)) = crate::tokenization::next_source_token(unit, offset) {
        if matches!(token, Token::CurlyBracketBlock) {
            return true;
        }
        offset = end;
    }
    false
}

pub(crate) fn from_rule_parse_error(
    source: &str,
    failed_unit: &str,
    error: ParseError<'_, CssError>,
) -> CssError {
    let mut error = from_parse_error(source, error);
    let unit = failed_unit.trim_start();
    // The adopted conditional grammar owns its prelude and terminal component
    // failures. Only a valid prelude's basic missing-body failure is contextualized.
    // The decoded at-keyword is the unit's name, so escaped at-keywords keep the
    // same ownership, and an escape such as `\{` belongs to the name (Syntax 3
    // (2026 CRD) §4.3.7) where a text split would end it.
    let first = crate::tokenization::next_source_token(unit, 0);
    let at_keyword = match &first {
        Some((_, _, Token::AtKeyword(name))) => Some(name.as_ref()),
        _ => None,
    };
    if let Some(name) = at_keyword
        && (name.eq_ignore_ascii_case("when") || name.eq_ignore_ascii_case("else"))
    {
        if matches!(
            error.kind,
            ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidAtRuleBody(_)
        ) {
            error.kind = ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: production_for_at_rule(name),
                expectation: CssGrammarExpectation::new("a block body for this at-rule"),
                encountered: None,
            });
        }
        return error;
    }
    if unit.starts_with('@') {
        if let Some(name) = at_keyword
            && unit.trim_end().ends_with(';')
            && at_rule_requires_block(name)
        {
            error.kind = ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: production_for_at_rule(name),
                expectation: CssGrammarExpectation::new("a block body for this at-rule"),
                encountered: None,
            });
        }
    } else if !contains_curly_block(unit)
        && matches!(
            error.kind,
            ErrorKind::UnexpectedEnd(_) | ErrorKind::UnexpectedToken(_)
        )
    {
        error.kind = ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
            production: QUALIFIED_RULE,
            expectation: EXPECT_CSS_SYNTAX,
            encountered: None,
        });
    }
    error
}

fn basic_error(location: cssparser::SourceLocation, kind: BasicParseErrorKind<'_>) -> CssError {
    match kind {
        BasicParseErrorKind::EndOfInput => CssError::at(
            location,
            ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                expectation: EXPECT_CSS_SYNTAX,
            }),
        ),
        BasicParseErrorKind::UnexpectedToken(token) => CssError::at(
            location,
            ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                expectation: EXPECT_CSS_SYNTAX,
                encountered: CssTokenSummary::from_token(&token),
            }),
        ),
        BasicParseErrorKind::AtRuleInvalid(name) => {
            let name = name.to_string();
            if let Some(feature) = unsupported_at_rule_feature(&name) {
                CssError::at(
                    location,
                    ErrorKind::UnsupportedAtRule(CssUnsupportedAtRuleError {
                        name: CssAtRuleName::new(name),
                        feature,
                    }),
                )
            } else {
                CssError::at(
                    location,
                    ErrorKind::UnknownAtRule(CssUnknownAtRuleError {
                        name: CssAtRuleName::new(name),
                    }),
                )
            }
        }
        BasicParseErrorKind::QualifiedRuleInvalid => CssError::at(
            location,
            ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
                production: QUALIFIED_RULE,
                expectation: EXPECT_CSS_SYNTAX,
                encountered: None,
            }),
        ),
        BasicParseErrorKind::AtRuleBodyInvalid => CssError::at(
            location,
            ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new("at-rule"),
                production: CssProductionId::new("css.at-rule"),
                expectation: EXPECT_CSS_SYNTAX,
                encountered: None,
            }),
        ),
    }
}

pub(crate) fn basic<'i>(error: BasicParseError<'i>) -> ParseError<'i, CssError> {
    error.into()
}

pub(crate) fn selector_basic<'i>(error: BasicParseError<'i>) -> ParseError<'i, CssError> {
    let location = error.location;
    let encountered = match error.kind {
        BasicParseErrorKind::UnexpectedToken(token) => Some(CssTokenSummary::from_token(&token)),
        _ => None,
    };
    error_at(
        location,
        ErrorKind::InvalidSelector(CssSelectorError {
            production: Some(SELECTOR_LIST),
            expectation: EXPECT_SELECTOR,
            encountered,
        }),
    )
}

/// Keep lexical envelope rejection in the selector domain without inferring a
/// different token from a bounded closing delimiter. Resource failures retain
/// their component error and original provenance instead of becoming grammar.
pub(crate) fn selector_component_error<'i>(
    fallback: cssparser::SourceLocation,
    detail: crate::component::CssComponentValueError,
) -> ParseError<'i, CssError> {
    let crate::component::CssValueOrigin::Parsed(origin) = detail.origin() else {
        return invalid_component_value(fallback, detail);
    };
    let authored = &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()];
    let kind = match detail.kind {
        crate::component::values::CssComponentValueErrorKind::BadString => CssTokenKind::BadString,
        crate::component::values::CssComponentValueErrorKind::BadUrl => CssTokenKind::BadUrl,
        crate::component::values::CssComponentValueErrorKind::UnmatchedClosingDelimiter => {
            match authored {
                ")" => CssTokenKind::CloseParenthesis,
                "]" => CssTokenKind::CloseSquareBracket,
                "}" => CssTokenKind::CloseCurlyBracket,
                _ => return invalid_component_value(fallback, detail),
            }
        }
        _ => return invalid_component_value(fallback, detail),
    };
    let position = origin.span().start();
    ParseError {
        location: cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value().saturating_add(1),
        },
        kind: ParseErrorKind::Custom(CssError {
            position,
            kind: ErrorKind::InvalidSelector(CssSelectorError {
                production: Some(SELECTOR_LIST),
                expectation: EXPECT_SELECTOR,
                encountered: Some(CssTokenSummary {
                    kind,
                    authored: authored.to_owned(),
                }),
            }),
        }),
    }
}

pub(crate) fn invalid_component_value<'i>(
    location: cssparser::SourceLocation,
    detail: crate::component::CssComponentValueError,
) -> ParseError<'i, CssError> {
    let position = match detail.origin() {
        crate::component::CssValueOrigin::Parsed(origin) => origin.span().start(),
        crate::component::CssValueOrigin::ImplicitClosure { opening, .. } => opening.span().start(),
        crate::component::CssValueOrigin::Programmatic
        | crate::component::CssValueOrigin::UnretainedInput { .. } => {
            CssSourcePosition::from_source_location(location)
        }
    };
    ParseError {
        location,
        kind: ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidComponentValue(Box::new(detail)),
            position,
        }),
    }
}

pub(crate) fn invalid_syntax<'i>(location: cssparser::SourceLocation) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
            production: QUALIFIED_RULE,
            expectation: EXPECT_CSS_SYNTAX,
            encountered: None,
        }),
    )
}

pub(crate) fn invalid_when_prelude<'i>(
    position: CssSourcePosition,
    name: &str,
    token: Option<&Token<'_>>,
) -> ParseError<'i, CssError> {
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    };
    ParseError {
        location,
        kind: ParseErrorKind::Custom(CssError {
            position,
            kind: ErrorKind::InvalidAtRulePrelude(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: production_for_at_rule(name),
                expectation: CssGrammarExpectation::new("a condition in the adopted when grammar"),
                encountered: token.map(CssTokenSummary::from_token),
            }),
        }),
    }
}
pub(crate) fn invalid_free_else(source: &str, position: CssSourcePosition) -> CssError {
    let name = crate::tokenization::next_source_token(source, position.byte_offset().value())
        .and_then(|(_, _, token)| match token {
            Token::AtKeyword(name) => Some(name),
            _ => None,
        })
        .expect("parsed Else starts at its original at-keyword");
    CssError {
        position,
        kind: ErrorKind::InvalidAtRulePlacement(CssAtRulePlacementError {
            name: CssAtRuleName::new(name.as_ref()),
            expected_context: CssGrammarExpectation::new(
                "after a conditional group separated only by whitespace or comments",
            ),
        }),
    }
}
pub(crate) fn invalid_at_rule_placement<'i>(
    location: cssparser::SourceLocation,
    name: &str,
    expected_context: &'static str,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::InvalidAtRulePlacement(CssAtRulePlacementError {
            name: CssAtRuleName::new(name),
            expected_context: CssGrammarExpectation::new(expected_context),
        }),
    )
}

pub(crate) fn with_at_rule_prelude_context<'i>(
    mut error: ParseError<'i, CssError>,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    let encountered = take_encountered(&mut error.kind);
    if !matches!(
        error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidMediaQuery(_),
            ..
        })
    ) {
        error.kind = ParseErrorKind::Custom(CssError::at(
            error.location,
            ErrorKind::InvalidAtRulePrelude(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: CssProductionId::new(production),
                expectation: CssGrammarExpectation::new(expectation),
                encountered,
            }),
        ));
    }
    error
}

pub(crate) fn invalid_at_rule_body<'i, 't>(
    input: &Parser<'i, 't>,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    invalid_at_rule_body_at(
        CssSourcePosition::from_cssparser(input.position(), input.current_source_location()),
        input.current_source_location(),
        name,
        production,
        expectation,
    )
}

pub(crate) fn invalid_at_rule_block<'i, 't>(
    input: &Parser<'i, 't>,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    let position =
        CssSourcePosition::from_cssparser(input.position(), input.current_source_location())
            .previous_ascii_byte();
    let mut location = input.current_source_location();
    location.column = location.column.saturating_sub(1).max(1);
    invalid_at_rule_body_at(position, location, name, production, expectation)
}

pub(crate) fn invalid_at_rule_body_at<'i>(
    position: CssSourcePosition,
    location: cssparser::SourceLocation,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, CssError> {
    ParseError {
        kind: ParseErrorKind::Custom(CssError::at_exact_nonzero_byte_offset(
            position,
            ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: CssProductionId::new(production),
                expectation: CssGrammarExpectation::new(expectation),
                encountered: None,
            }),
        )),
        location,
    }
}

pub(crate) fn invalid_selector<'i, 't>(input: &Parser<'i, 't>) -> ParseError<'i, CssError> {
    invalid_selector_at(input.current_source_location())
}

pub(crate) fn invalid_selector_at<'i>(
    location: cssparser::SourceLocation,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::InvalidSelector(CssSelectorError {
            production: Some(SELECTOR_LIST),
            expectation: EXPECT_SELECTOR,
            encountered: None,
        }),
    )
}

/// A recognized property refused by the active grammar, such as `size` in a
/// margin rule, names that property's support-catalog record.
pub(crate) fn property_name_error<'i>(
    location: cssparser::SourceLocation,
    name: &str,
) -> ParseError<'i, CssError> {
    match property_for_supported_name(name) {
        Some(property) => error_at(
            location,
            ErrorKind::UnsupportedProperty(CssUnsupportedPropertyError {
                feature: CssFeatureId::new(property.stable_id()),
                name: CssPropertyName::new(name),
            }),
        ),
        None => error_at(
            location,
            ErrorKind::UnknownProperty(CssUnknownPropertyError {
                name: CssPropertyName::new(name),
            }),
        ),
    }
}

pub(crate) fn descriptor_name_error<'i>(
    location: cssparser::SourceLocation,
    at_rule: &str,
    name: &str,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::UnknownDescriptor(CssUnknownDescriptorError {
            at_rule: CssAtRuleName::new(at_rule),
            descriptor: CssDescriptorName::new(name),
        }),
    )
}

pub(crate) fn color_syntax_kind(component: Option<&str>) -> ErrorKind {
    ErrorKind::InvalidColorSyntax(CssColorSyntaxError {
        component: component.map(CssColorComponentName::new),
        expectation: EXPECT_COLOR,
        encountered: None,
    })
}

pub(crate) fn with_media_query_context<'i>(
    mut error: ParseError<'i, CssError>,
    feature: Option<&str>,
) -> ParseError<'i, CssError> {
    if matches!(
        error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidMediaQuery(_),
            ..
        })
    ) {
        return error;
    }
    let encountered = take_encountered(&mut error.kind);
    error.kind = ParseErrorKind::Custom(CssError::at(
        error.location,
        ErrorKind::InvalidMediaQuery(CssMediaQueryError {
            committed_context: false,
            feature: feature.map(CssMediaFeatureName::new),
            expectation: EXPECT_MEDIA_QUERY,
            encountered,
        }),
    ));
    error
}

pub(crate) fn custom_media_context_error<'i>(
    location: cssparser::SourceLocation,
    name: &str,
) -> ParseError<'i, CssError> {
    let mut error = with_media_query_context(invalid_syntax(location), Some(name));
    if let ParseErrorKind::Custom(CssError {
        kind: ErrorKind::InvalidMediaQuery(detail),
        ..
    }) = &mut error.kind
    {
        detail.committed_context = true;
    }
    error
}

pub(crate) fn with_property_context<'i>(
    mut error: ParseError<'i, CssError>,
    property: &str,
) -> ParseError<'i, CssError> {
    let Some(property) = property_for_supported_name(property) else {
        return error;
    };
    if matches!(
        &error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidColorSyntax(_),
            ..
        })
    ) || matches!(
        &error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidComponentValue(detail),
            ..
        }) if is_component_resource_error(detail)
    ) {
        return error;
    }
    let encountered = take_encountered(&mut error.kind);
    error.kind = ParseErrorKind::Custom(CssError::at(
        error.location,
        ErrorKind::InvalidPropertyValue(PropertyValueRejection {
            property,
            expectation: EXPECT_PROPERTY_VALUE,
            encountered,
        }),
    ));
    error
}

/// Resource failures are component invariants even when found during a
/// property-specific preflight. Keep their typed kind and original token origin.
pub(crate) fn is_component_resource_error(
    detail: &crate::component::CssComponentValueError,
) -> bool {
    detail.is_resource_limit()
}

/// A speculative grammar alternative must not erase resource failures.
pub(crate) fn is_resource_parse_error(error: &ParseError<'_, CssError>) -> bool {
    is_nesting_limit_error(error)
        || matches!(
            &error.kind,
            ParseErrorKind::Custom(CssError { kind: ErrorKind::InvalidComponentValue(detail), .. })
                if is_component_resource_error(detail)
        )
}

/// Names the owning at-rule and descriptor on a standalone descriptor-value
/// grammar failure, as the enclosing rule body does. Resource and component
/// failures keep their own kinds.
pub(crate) fn with_descriptor_value_context<'i>(
    error: ParseError<'i, CssError>,
    at_rule: &str,
    descriptor: &str,
) -> ParseError<'i, CssError> {
    if is_resource_parse_error(&error)
        || matches!(
            &error.kind,
            ParseErrorKind::Custom(CssError {
                kind: ErrorKind::InvalidComponentValue(_),
                ..
            })
        )
    {
        error
    } else {
        with_descriptor_context(error, at_rule, descriptor)
    }
}

pub(crate) fn with_descriptor_context<'i>(
    mut error: ParseError<'i, CssError>,
    at_rule: &str,
    descriptor: &str,
) -> ParseError<'i, CssError> {
    if matches!(
        error.kind,
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::UnknownDescriptor(_)
                | ErrorKind::UnsupportedDescriptor(_)
                | ErrorKind::InvalidDeclarationAnnotation(_),
            ..
        })
    ) {
        return error;
    }
    let origin = match &error.kind {
        ParseErrorKind::Custom(CssError {
            kind: ErrorKind::InvalidDescriptorValue(detail),
            ..
        }) => detail.origin,
        _ => DiagnosticOrigin::Inferred,
    };
    let encountered = take_encountered(&mut error.kind);
    let contextual = CssError::at(
        error.location,
        ErrorKind::InvalidDescriptorValue(CssDescriptorValueError {
            origin,
            at_rule: CssAtRuleName::new(at_rule),
            descriptor: Box::new(CssDescriptorName::new(descriptor)),
            expectation: EXPECT_DESCRIPTOR_VALUE,
            encountered,
        }),
    );
    error.kind = ParseErrorKind::Custom(contextual);
    error
}

pub(crate) fn invalid_descriptor_token_at<'i>(
    location: cssparser::SourceLocation,
    at_rule: &str,
    descriptor: &str,
    token: &Token<'_>,
    authored: &str,
) -> ParseError<'i, CssError> {
    explicit_descriptor_error(
        location,
        at_rule,
        descriptor,
        Some(CssTokenSummary::from_authored_token(token, authored)),
        DiagnosticOrigin::Token,
    )
}

pub(crate) fn incomplete_descriptor_at<'i>(
    location: cssparser::SourceLocation,
    at_rule: &str,
    descriptor: &str,
) -> ParseError<'i, CssError> {
    explicit_descriptor_error(
        location,
        at_rule,
        descriptor,
        None,
        DiagnosticOrigin::EndOfInput,
    )
}

fn explicit_descriptor_error<'i>(
    location: cssparser::SourceLocation,
    at_rule: &str,
    descriptor: &str,
    encountered: Option<CssTokenSummary>,
    origin: DiagnosticOrigin,
) -> ParseError<'i, CssError> {
    let error = CssError::at(
        location,
        ErrorKind::InvalidDescriptorValue(CssDescriptorValueError {
            origin,
            at_rule: CssAtRuleName::new(at_rule),
            descriptor: Box::new(CssDescriptorName::new(descriptor)),
            expectation: EXPECT_DESCRIPTOR_VALUE,
            encountered,
        }),
    );
    ParseError {
        location,
        kind: ParseErrorKind::Custom(error),
    }
}

pub(crate) fn invalid_svg_glyph_declaration_annotation<'i>(
    location: cssparser::SourceLocation,
    keyframe: bool,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::InvalidDeclarationAnnotation(CssDeclarationAnnotationError {
            context: if keyframe {
                CssDeclarationContext::KeyframeSvgGlyph
            } else {
                CssDeclarationContext::SvgGlyph
            },
            encountered: CssTokenSummary::bang(),
        }),
    )
}

pub(crate) fn invalid_known_declaration_annotation<'i>(
    location: cssparser::SourceLocation,
    property: CssKnownProperty,
    keyframe: bool,
) -> ParseError<'i, CssError> {
    let context = if keyframe {
        CssDeclarationContext::Keyframe(property)
    } else {
        CssDeclarationContext::OrdinaryKnown(property)
    };
    error_at(
        location,
        ErrorKind::InvalidDeclarationAnnotation(CssDeclarationAnnotationError {
            context,
            encountered: CssTokenSummary::bang(),
        }),
    )
}

pub(crate) fn invalid_custom_declaration_annotation<'i>(
    location: cssparser::SourceLocation,
    property: &CssCustomPropertyName,
    keyframe: bool,
) -> ParseError<'i, CssError> {
    let context = if keyframe {
        CssDeclarationContext::KeyframeCustom(property.clone())
    } else {
        CssDeclarationContext::OrdinaryCustom(property.clone())
    };
    error_at(
        location,
        ErrorKind::InvalidDeclarationAnnotation(CssDeclarationAnnotationError {
            context,
            encountered: CssTokenSummary::bang(),
        }),
    )
}

pub(crate) fn invalid_descriptor_annotation<'i>(
    location: cssparser::SourceLocation,
    at_rule: &str,
    descriptor: &str,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::InvalidDeclarationAnnotation(CssDeclarationAnnotationError {
            context: CssDeclarationContext::Descriptor {
                at_rule: CssAtRuleName::new(at_rule),
                descriptor: CssDescriptorName::new(descriptor),
            },
            encountered: CssTokenSummary::bang(),
        }),
    )
}

pub(crate) fn error_at<'i>(
    location: cssparser::SourceLocation,
    kind: ErrorKind,
) -> ParseError<'i, CssError> {
    ParseError {
        kind: ParseErrorKind::Custom(CssError::at(location, kind)),
        location,
    }
}

pub(crate) fn unexpected_at<'i>(location: cssparser::SourceLocation) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_DECLARATION_VALUE,
        }),
    )
}

/// The error of a token stream that met `token` where its grammar needed
/// another, as a cursor that reads tokens one at a time reports it.
pub(crate) fn unexpected_syntax_token<'i>(
    location: cssparser::SourceLocation,
    token: surgeist_css_parser::Token<'_>,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
            expectation: EXPECT_CSS_SYNTAX,
            encountered: CssTokenSummary::from_syntax_token(token),
        }),
    )
}

/// The error of a token stream that ended where its grammar needed a token.
pub(crate) fn unexpected_syntax_end<'i>(
    location: cssparser::SourceLocation,
) -> ParseError<'i, CssError> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_CSS_SYNTAX,
        }),
    )
}

fn take_encountered(kind: &mut ParseErrorKind<'_, CssError>) -> Option<CssTokenSummary> {
    match kind {
        ParseErrorKind::Basic(BasicParseErrorKind::UnexpectedToken(token)) => {
            Some(CssTokenSummary::from_token(token))
        }
        ParseErrorKind::Custom(error) => encountered_mut(&mut error.kind).cloned(),
        _ => None,
    }
}

fn encountered_mut(kind: &mut ErrorKind) -> Option<&mut CssTokenSummary> {
    match kind {
        ErrorKind::UnexpectedToken(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidAtRulePrelude(detail) | ErrorKind::InvalidAtRuleBody(detail) => {
            detail.encountered.as_mut()
        }
        ErrorKind::InvalidQualifiedRule(detail) => detail.encountered.as_mut(),
        ErrorKind::InvalidSelector(detail) => detail.encountered.as_mut(),
        ErrorKind::InvalidMediaQuery(detail) => detail.encountered.as_mut(),
        ErrorKind::InvalidPropertyValue(detail) => detail.encountered.as_mut(),
        ErrorKind::InvalidDeclarationAnnotation(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidDescriptorValue(detail) => detail.encountered.as_mut(),
        ErrorKind::InvalidColorSyntax(detail) => detail.encountered.as_mut(),
        _ => None,
    }
}

fn optional_encountered_mut(kind: &mut ErrorKind) -> Option<&mut Option<CssTokenSummary>> {
    match kind {
        ErrorKind::InvalidAtRulePrelude(detail) | ErrorKind::InvalidAtRuleBody(detail) => {
            Some(&mut detail.encountered)
        }
        ErrorKind::InvalidQualifiedRule(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidSelector(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidMediaQuery(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidPropertyValue(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidDescriptorValue(detail) => Some(&mut detail.encountered),
        ErrorKind::InvalidColorSyntax(detail) => Some(&mut detail.encountered),
        _ => None,
    }
}

/// The kind of a token of the parser crate's token tree.
pub(crate) fn syntax_token_kind(token: surgeist_css_parser::Token<'_>) -> CssTokenKind {
    use surgeist_css_parser::Token as Syntax;
    use surgeist_css_parser::token::HashKind;
    match token {
        Syntax::Ident(_) | Syntax::UnicodeRange(_) => CssTokenKind::Ident,
        Syntax::AtKeyword(_) => CssTokenKind::AtKeyword,
        Syntax::Hash {
            kind: HashKind::Id, ..
        } => CssTokenKind::IdHash,
        Syntax::Hash {
            kind: HashKind::Unrestricted,
            ..
        } => CssTokenKind::Hash,
        Syntax::String(_) => CssTokenKind::String,
        Syntax::Url(_) => CssTokenKind::Url,
        Syntax::Delim(_) => CssTokenKind::Delim,
        Syntax::Number(_) => CssTokenKind::Number,
        Syntax::Percentage(_) => CssTokenKind::Percentage,
        Syntax::Dimension { .. } => CssTokenKind::Dimension,
        Syntax::Whitespace => CssTokenKind::Whitespace,
        Syntax::Comment => CssTokenKind::Comment,
        Syntax::Colon => CssTokenKind::Colon,
        Syntax::Semicolon => CssTokenKind::Semicolon,
        Syntax::Comma => CssTokenKind::Comma,
        Syntax::Cdo => CssTokenKind::Cdo,
        Syntax::Cdc => CssTokenKind::Cdc,
        Syntax::Function(_) => CssTokenKind::Function,
        Syntax::OpenParen => CssTokenKind::ParenthesisBlock,
        Syntax::OpenSquare => CssTokenKind::SquareBracketBlock,
        Syntax::OpenCurly => CssTokenKind::CurlyBracketBlock,
        Syntax::BadUrl => CssTokenKind::BadUrl,
        Syntax::BadString => CssTokenKind::BadString,
        Syntax::CloseParen => CssTokenKind::CloseParenthesis,
        Syntax::CloseSquare => CssTokenKind::CloseSquareBracket,
        Syntax::CloseCurly => CssTokenKind::CloseCurlyBracket,
    }
}

fn token_kind(token: &Token<'_>) -> CssTokenKind {
    match token {
        Token::Ident(_) => CssTokenKind::Ident,
        Token::AtKeyword(_) => CssTokenKind::AtKeyword,
        Token::Hash(_) => CssTokenKind::Hash,
        Token::IDHash(_) => CssTokenKind::IdHash,
        Token::QuotedString(_) => CssTokenKind::String,
        Token::UnquotedUrl(_) => CssTokenKind::Url,
        Token::Delim(_)
        | Token::IncludeMatch
        | Token::DashMatch
        | Token::PrefixMatch
        | Token::SuffixMatch
        | Token::SubstringMatch => CssTokenKind::Delim,
        Token::Number { .. } => CssTokenKind::Number,
        Token::Percentage { .. } => CssTokenKind::Percentage,
        Token::Dimension { .. } => CssTokenKind::Dimension,
        Token::WhiteSpace(_) => CssTokenKind::Whitespace,
        Token::Comment(_) => CssTokenKind::Comment,
        Token::Colon => CssTokenKind::Colon,
        Token::Semicolon => CssTokenKind::Semicolon,
        Token::Comma => CssTokenKind::Comma,
        Token::CDO => CssTokenKind::Cdo,
        Token::CDC => CssTokenKind::Cdc,
        Token::Function(_) => CssTokenKind::Function,
        Token::ParenthesisBlock => CssTokenKind::ParenthesisBlock,
        Token::SquareBracketBlock => CssTokenKind::SquareBracketBlock,
        Token::CurlyBracketBlock => CssTokenKind::CurlyBracketBlock,
        Token::BadUrl(_) => CssTokenKind::BadUrl,
        Token::BadString(_) => CssTokenKind::BadString,
        Token::CloseParenthesis => CssTokenKind::CloseParenthesis,
        Token::CloseSquareBracket => CssTokenKind::CloseSquareBracket,
        Token::CloseCurlyBracket => CssTokenKind::CloseCurlyBracket,
    }
}

fn production_for_at_rule(name: &str) -> CssProductionId {
    if name.eq_ignore_ascii_case("when") {
        return CssProductionId::new("ext.rule.when");
    }
    if name.eq_ignore_ascii_case("else") {
        return CssProductionId::new("ext.rule.else");
    }
    if name.eq_ignore_ascii_case("import") {
        CssProductionId::new("baseline.rule.import")
    } else if name.eq_ignore_ascii_case("layer") {
        CssProductionId::new("baseline.rule.layer-block")
    } else if name.eq_ignore_ascii_case("font-face") {
        CssProductionId::new("baseline.rule.font-face")
    } else if name.eq_ignore_ascii_case("font-palette-values") {
        CssProductionId::new("later.rule.font-palette-values")
    } else if name.eq_ignore_ascii_case("font-feature-values") {
        CssProductionId::new("later.rule.font-feature-values")
    } else if name.eq_ignore_ascii_case("keyframes") {
        CssProductionId::new("baseline.rule.keyframes")
    } else if name.eq_ignore_ascii_case("media") {
        CssProductionId::new("baseline.rule.media")
    } else if name.eq_ignore_ascii_case("container") {
        CssProductionId::new("baseline.rule.container")
    } else if name.eq_ignore_ascii_case("scope") {
        CssProductionId::new("baseline.rule.scope")
    } else {
        CssProductionId::new("css.at-rule")
    }
}

fn unsupported_at_rule_feature(name: &str) -> Option<CssFeatureId> {
    if name.eq_ignore_ascii_case("namespace") {
        Some(CssFeatureId::new("later.rule.namespace"))
    } else if name.eq_ignore_ascii_case("supports") {
        Some(CssFeatureId::new("later.rule.supports"))
    } else if name.eq_ignore_ascii_case("counter-style") {
        Some(CssFeatureId::new("later.rule.counter-style"))
    } else if name.eq_ignore_ascii_case("page") || is_page_margin_box_name(name) {
        Some(CssFeatureId::new("later.rule.page"))
    } else if name.eq_ignore_ascii_case("font-feature-values") {
        Some(CssFeatureId::new("later.rule.font-feature-values"))
    } else if name.eq_ignore_ascii_case("font-palette-values") {
        Some(CssFeatureId::new("later.rule.font-palette-values"))
    } else {
        None
    }
}

fn is_page_margin_box_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "top-left-corner"
            | "top-left"
            | "top-center"
            | "top-right"
            | "top-right-corner"
            | "bottom-left-corner"
            | "bottom-left"
            | "bottom-center"
            | "bottom-right"
            | "bottom-right-corner"
            | "left-top"
            | "left-middle"
            | "left-bottom"
            | "right-top"
            | "right-middle"
            | "right-bottom"
    )
}

fn at_rule_requires_block(name: &str) -> bool {
    name.eq_ignore_ascii_case("font-face")
        || name.eq_ignore_ascii_case("font-feature-values")
        || name.eq_ignore_ascii_case("font-palette-values")
        || name.eq_ignore_ascii_case("keyframes")
        || name.eq_ignore_ascii_case("media")
        || name.eq_ignore_ascii_case("container")
        || name.eq_ignore_ascii_case("scope")
        || name.eq_ignore_ascii_case("page")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn position() -> cssparser::SourceLocation {
        cssparser::SourceLocation { line: 0, column: 1 }
    }

    #[test]
    fn explicit_descriptor_operator_diagnostics_publish_the_first_original_delimiter() {
        for (operator, native) in [
            ("~=", Token::IncludeMatch),
            ("|=", Token::DashMatch),
            ("^=", Token::PrefixMatch),
            ("$=", Token::SuffixMatch),
            ("*=", Token::SubstringMatch),
        ] {
            let source = format!("x;\r\n😀{operator}");
            let error = from_parse_error(
                &source,
                invalid_descriptor_token_at(
                    cssparser::SourceLocation { line: 1, column: 3 },
                    "font-face",
                    "unicode-range",
                    &native,
                    operator,
                ),
            );
            let ErrorKind::InvalidDescriptorValue(detail) = error.kind() else {
                panic!("explicit descriptor error");
            };
            assert_eq!(detail.at_rule().as_str(), "font-face");
            assert_eq!(detail.descriptor().as_str(), "unicode-range");
            let token = detail.encountered().unwrap();
            assert_eq!(token.kind(), CssTokenKind::Delim);
            assert_eq!(token.authored(), &operator[..1]);
            assert_eq!(error.position().byte_offset().value(), 8);
            assert_eq!(error.position().line().value(), 1);
            assert_eq!(error.position().column().value(), 2);
        }
    }

    #[test]
    fn semantic_error_variants_report_their_stable_root_codes() {
        let token = CssTokenSummary {
            kind: CssTokenKind::Ident,
            authored: "x".to_owned(),
        };
        let at_rule = CssAtRuleName::new("x");
        let descriptor = CssDescriptorName::new("x");
        let roots = vec![
            (
                ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                    expectation: EXPECT_CSS_SYNTAX,
                }),
                CssErrorCode::UnexpectedEnd,
            ),
            (
                ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: token.clone(),
                }),
                CssErrorCode::UnexpectedToken,
            ),
            (
                ErrorKind::InvalidAtRulePlacement(CssAtRulePlacementError {
                    name: at_rule.clone(),
                    expected_context: EXPECT_CSS_SYNTAX,
                }),
                CssErrorCode::InvalidAtRulePlacement,
            ),
            (
                ErrorKind::InvalidAtRulePrelude(CssAtRuleSyntaxError {
                    name: at_rule.clone(),
                    production: QUALIFIED_RULE,
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: None,
                }),
                CssErrorCode::InvalidAtRulePrelude,
            ),
            (
                ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                    name: at_rule.clone(),
                    production: QUALIFIED_RULE,
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: None,
                }),
                CssErrorCode::InvalidAtRuleBody,
            ),
            (
                ErrorKind::UnknownAtRule(CssUnknownAtRuleError {
                    name: at_rule.clone(),
                }),
                CssErrorCode::UnknownAtRule,
            ),
            (
                ErrorKind::NamespaceRedeclaration(CssNamespaceRedeclarationError {
                    prefix: None,
                    previous_position: CssSourcePosition::from_byte_offset_in("@namespace '';", 0),
                }),
                CssErrorCode::NamespaceRedeclaration,
            ),
            (
                ErrorKind::UnsupportedAtRule(CssUnsupportedAtRuleError {
                    name: at_rule.clone(),
                    feature: CssFeatureId::new("x"),
                }),
                CssErrorCode::UnsupportedAtRule,
            ),
            (
                ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
                    production: QUALIFIED_RULE,
                    expectation: EXPECT_CSS_SYNTAX,
                    encountered: None,
                }),
                CssErrorCode::InvalidQualifiedRule,
            ),
            (
                ErrorKind::InvalidSelector(CssSelectorError {
                    production: None,
                    expectation: EXPECT_SELECTOR,
                    encountered: None,
                }),
                CssErrorCode::InvalidSelector,
            ),
            (
                ErrorKind::InvalidMediaQuery(CssMediaQueryError {
                    committed_context: false,
                    feature: None,
                    expectation: EXPECT_MEDIA_QUERY,
                    encountered: None,
                }),
                CssErrorCode::InvalidMediaQuery,
            ),
            (
                ErrorKind::UnknownProperty(CssUnknownPropertyError {
                    name: CssPropertyName::new("x"),
                }),
                CssErrorCode::UnknownProperty,
            ),
            (
                ErrorKind::UnsupportedProperty(CssUnsupportedPropertyError {
                    name: CssPropertyName::new("x"),
                    feature: CssFeatureId::new("x"),
                }),
                CssErrorCode::UnsupportedProperty,
            ),
            (
                ErrorKind::InvalidPropertyValue(PropertyValueRejection {
                    property: CssKnownProperty::Width,
                    expectation: EXPECT_PROPERTY_VALUE,
                    encountered: None,
                }),
                CssErrorCode::InvalidPropertyValue,
            ),
            (
                ErrorKind::InvalidDeclarationAnnotation(CssDeclarationAnnotationError {
                    context: CssDeclarationContext::OrdinaryKnown(CssKnownProperty::Width),
                    encountered: token.clone(),
                }),
                CssErrorCode::InvalidDeclarationAnnotation,
            ),
            (
                ErrorKind::UnknownDescriptor(CssUnknownDescriptorError {
                    at_rule: at_rule.clone(),
                    descriptor: descriptor.clone(),
                }),
                CssErrorCode::UnknownDescriptor,
            ),
            (
                ErrorKind::UnsupportedDescriptor(CssUnsupportedDescriptorError {
                    at_rule: at_rule.clone(),
                    descriptor: descriptor.clone(),
                    feature: CssFeatureId::new("x"),
                }),
                CssErrorCode::UnsupportedDescriptor,
            ),
            (
                ErrorKind::InvalidDescriptorValue(CssDescriptorValueError {
                    origin: DiagnosticOrigin::Inferred,
                    at_rule: at_rule.clone(),
                    descriptor: Box::new(descriptor.clone()),
                    expectation: EXPECT_DESCRIPTOR_VALUE,
                    encountered: None,
                }),
                CssErrorCode::InvalidDescriptorValue,
            ),
            (
                ErrorKind::InvalidDescriptorCombination(CssDescriptorCombinationError {
                    at_rule: at_rule.clone(),
                    responsible: descriptor.clone(),
                    conflicting: vec![descriptor],
                }),
                CssErrorCode::InvalidDescriptorCombination,
            ),
            (
                ErrorKind::InvalidColorSyntax(CssColorSyntaxError {
                    component: None,
                    expectation: EXPECT_COLOR,
                    encountered: None,
                }),
                CssErrorCode::InvalidColorSyntax,
            ),
            (
                ErrorKind::NestingLimit(CssNestingLimitError {
                    limit: 256,
                    enclosing_production: QUALIFIED_RULE,
                }),
                CssErrorCode::NestingLimit,
            ),
            (
                ErrorKind::ParserUnavailable(CssNestingLimitError {
                    limit: 128,
                    enclosing_production: QUALIFIED_RULE,
                }),
                CssErrorCode::ParserUnavailable,
            ),
        ];

        for (kind, expected) in roots {
            assert_eq!(CssError::at(position(), kind).code(), expected);
        }
    }

    #[test]
    fn semantic_error_details_expose_typed_fields() {
        let token = CssTokenSummary {
            kind: CssTokenKind::Delim,
            authored: "!".to_owned(),
        };
        let unexpected_end = CssUnexpectedEndError {
            expectation: EXPECT_CSS_SYNTAX,
        };
        assert_eq!(unexpected_end.expectation().as_str(), "valid CSS syntax");
        let unexpected_token = CssUnexpectedTokenError {
            expectation: EXPECT_CSS_SYNTAX,
            encountered: token.clone(),
        };
        assert_eq!(unexpected_token.expectation().as_str(), "valid CSS syntax");
        assert_eq!(unexpected_token.encountered().kind(), CssTokenKind::Delim);
        let unsupported_property = CssUnsupportedPropertyError {
            name: CssPropertyName::new("future-property"),
            feature: CssFeatureId::new("later.property.future-property"),
        };
        assert_eq!(unsupported_property.name().as_str(), "future-property");
        assert_eq!(
            unsupported_property.feature().as_str(),
            "later.property.future-property"
        );

        let unsupported_descriptor = CssUnsupportedDescriptorError {
            at_rule: CssAtRuleName::new("font-face"),
            descriptor: CssDescriptorName::new("future-descriptor"),
            feature: CssFeatureId::new("later.descriptor.future-descriptor"),
        };
        assert_eq!(unsupported_descriptor.at_rule().as_str(), "font-face");
        assert_eq!(
            unsupported_descriptor.descriptor().as_str(),
            "future-descriptor"
        );
        assert_eq!(
            unsupported_descriptor.feature().as_str(),
            "later.descriptor.future-descriptor"
        );

        let annotation = CssDeclarationAnnotationError {
            context: CssDeclarationContext::Descriptor {
                at_rule: CssAtRuleName::new("font-face"),
                descriptor: CssDescriptorName::new("src"),
            },
            encountered: token.clone(),
        };
        match annotation.context() {
            CssDeclarationContextRef::Descriptor {
                at_rule,
                descriptor,
            } => {
                assert_eq!(at_rule.as_str(), "font-face");
                assert_eq!(descriptor.as_str(), "src");
            }
            _ => panic!("wrong declaration context"),
        }
        assert_eq!(annotation.encountered().authored(), "!");

        let custom = CssDeclarationAnnotationError {
            context: CssDeclarationContext::OrdinaryCustom(
                CssCustomPropertyName::try_new("--theme").unwrap(),
            ),
            encountered: token.clone(),
        };
        match custom.context() {
            CssDeclarationContextRef::CustomProperty(name) => {
                assert_eq!(name.as_str(), "--theme");
            }
            _ => panic!("wrong custom-property context"),
        }

        let keyframe = CssDeclarationAnnotationError {
            context: CssDeclarationContext::Keyframe(CssKnownProperty::Opacity),
            encountered: token,
        };
        match keyframe.context() {
            CssDeclarationContextRef::Keyframe(property) => {
                assert_eq!(property, CssKnownProperty::Opacity);
            }
            _ => panic!("wrong keyframe context"),
        }

        let nesting = CssNestingLimitError {
            limit: 256,
            enclosing_production: QUALIFIED_RULE,
        };
        assert_eq!(nesting.limit(), 256);
        assert_eq!(
            nesting.enclosing_production().as_str(),
            "css.qualified-rule"
        );
    }
}
