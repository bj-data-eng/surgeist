mod source_resolution;

use std::fmt;

use cssparser::{
    BasicParseError, BasicParseErrorKind, ParseError, ParseErrorKind, Parser, ToCss, Token,
};

use crate::properties::CssKnownProperty;
use crate::source::CssSourcePosition;
use crate::syntax::CssCustomPropertyName;
use crate::validation::{PropertyNameStatus, classify_property_name, property_for_supported_name};

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
        pub struct $name(String);

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

owned_name!(
    CssAtRuleName,
    "A diagnostic-phase identity preserving a decoded authored CSS at-rule name; it does not parse the rule or select recovery."
);
owned_name!(
    CssPropertyName,
    "A diagnostic-phase identity preserving a decoded authored CSS property name; it does not parse, resolve, or apply the property."
);
owned_name!(
    CssDescriptorName,
    "A diagnostic-phase identity preserving a decoded authored CSS descriptor name; it does not parse, resolve, or apply the descriptor."
);
owned_name!(
    CssMediaFeatureName,
    "A diagnostic-phase identity preserving a decoded authored media-feature name; it does not evaluate the media query."
);
owned_name!(
    CssColorComponentName,
    "A diagnostic-phase identity naming the authored color component responsible for a failure; it does not evaluate or convert the color."
);

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
    kind: CssTokenKind,
    authored: String,
}

impl CssTokenSummary {
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

    fn from_token(token: &Token<'_>) -> Self {
        Self {
            kind: token_kind(token),
            authored: crate::tokenization::combined_operator_prefix(token)
                .map_or_else(|| token.to_css_string(), |first| first.to_string()),
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

const EXPECT_CSS_SYNTAX: CssGrammarExpectation = CssGrammarExpectation::new("valid CSS syntax");
const EXPECT_DECLARATION_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a declaration value");
const EXPECT_PROPERTY_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a value accepted by the property's grammar");
const EXPECT_DESCRIPTOR_VALUE: CssGrammarExpectation =
    CssGrammarExpectation::new("a value accepted by the descriptor grammar");
const EXPECT_SELECTOR: CssGrammarExpectation = CssGrammarExpectation::new("a supported selector");
const EXPECT_MEDIA_QUERY: CssGrammarExpectation =
    CssGrammarExpectation::new("a supported media query");
const EXPECT_COLOR: CssGrammarExpectation = CssGrammarExpectation::new("valid color syntax");

const QUALIFIED_RULE: CssProductionId = CssProductionId::new("css.qualified-rule");
const SELECTOR_LIST: CssProductionId = CssProductionId::new("baseline.selector.complex");

/// Diagnostic detail for strict parsing that reached EOF before an expected production completed.
///
/// The stored expectation identifies the violated grammar invariant; this value does not insert
/// missing syntax or perform browser-style recovery.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUnexpectedEndError {
    expectation: CssGrammarExpectation,
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
pub struct CssUnexpectedTokenError {
    expectation: CssGrammarExpectation,
    encountered: CssTokenSummary,
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
pub struct CssAtRulePlacementError {
    name: CssAtRuleName,
    expected_context: CssGrammarExpectation,
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
pub struct CssAtRuleSyntaxError {
    name: CssAtRuleName,
    production: CssProductionId,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
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
pub struct CssNamespaceRedeclarationError {
    prefix: Option<crate::CssNamespacePrefix>,
    previous_position: CssSourcePosition,
}

impl CssNamespaceRedeclarationError {
    /// Returns the exact decoded prefix, or `None` for a repeated default binding.
    #[must_use]
    pub const fn prefix(&self) -> Option<&crate::CssNamespacePrefix> {
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
pub struct CssLegacyNavigationTargetError {
    target: String,
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
pub struct CssUnknownAtRuleError {
    name: CssAtRuleName,
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
pub struct CssUnsupportedAtRuleError {
    name: CssAtRuleName,
    feature: CssFeatureId,
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
pub struct CssQualifiedRuleError {
    production: CssProductionId,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
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
pub struct CssSelectorError {
    production: Option<CssProductionId>,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
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
pub struct CssMediaQueryError {
    committed_context: bool,
    feature: Option<CssMediaFeatureName>,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
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
pub struct CssUnknownPropertyError {
    name: CssPropertyName,
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
pub struct CssUnsupportedPropertyError {
    name: CssPropertyName,
    feature: CssFeatureId,
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
pub struct CssPropertyValueError {
    property: CssKnownProperty,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
}

impl CssPropertyValueError {
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
enum CssDeclarationContext {
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
pub struct CssDeclarationAnnotationError {
    context: CssDeclarationContext,
    encountered: CssTokenSummary,
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
/// The owning rule and descriptor names preserve authored identity; this value does not reinterpret,
/// apply, or recover the descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUnknownDescriptorError {
    at_rule: CssAtRuleName,
    descriptor: CssDescriptorName,
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
pub struct CssUnsupportedDescriptorError {
    at_rule: CssAtRuleName,
    descriptor: CssDescriptorName,
    feature: CssFeatureId,
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
pub struct CssDescriptorValueError {
    origin: DiagnosticOrigin,
    at_rule: CssAtRuleName,
    // Keep the largest error payload compact when retaining explicit origins.
    descriptor: Box<CssDescriptorName>,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DiagnosticOrigin {
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
pub struct CssDescriptorCombinationError {
    at_rule: CssAtRuleName,
    responsible: CssDescriptorName,
    conflicting: Vec<CssDescriptorName>,
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
pub struct CssColorSyntaxError {
    component: Option<CssColorComponentName>,
    expectation: CssGrammarExpectation,
    encountered: Option<CssTokenSummary>,
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
pub struct CssNestingLimitError {
    limit: u32,
    enclosing_production: CssProductionId,
}

impl CssNestingLimitError {
    #[must_use]
    /// Returns the maximum nesting depth accepted by the strict parser.
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
pub enum CssEscapeError {
    /// Backslash-newline is not a valid escape; recovery returns a backslash delimiter.
    Newline,
    /// An escape reached EOF; recovery returns the replacement character U+FFFD.
    EndOfInput,
}

/// The structured diagnostic-phase detail for one CSS grammar or conformance violation.
///
/// Every variant carries the payload required by its stable [`CssErrorCode`] and preserves authored
/// provenance without a free-form catch-all. Matching a variant diagnoses authored syntax; it does
/// not perform browser-style recovery, cascade, substitution, matching, or contextual resolution.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    /// Tokenizer escape processing required lexical recovery.
    EscapeParseError(CssEscapeError),
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
    InvalidPropertyValue(CssPropertyValueError),
    /// A retained declaration value could not form a checked component sequence.
    InvalidComponentValue(Box<crate::CssComponentValueError>),
    /// An authored declaration annotation was malformed or misplaced.
    InvalidDeclarationAnnotation(CssDeclarationAnnotationError),
    /// An authored descriptor name was not recognized for its at-rule.
    UnknownDescriptor(CssUnknownDescriptorError),
    /// An authored descriptor was recognized but unsupported.
    UnsupportedDescriptor(CssUnsupportedDescriptorError),
    /// An authored descriptor value was invalid.
    InvalidDescriptorValue(CssDescriptorValueError),
    /// Authored descriptors formed an invalid combination.
    InvalidDescriptorCombination(CssDescriptorCombinationError),
    /// Authored color syntax was invalid or outside the supported grammar.
    InvalidColorSyntax(CssColorSyntaxError),
    /// Authored nesting exceeded the parser's configured structural limit.
    NestingLimit(CssNestingLimitError),
}

/// A structured grammar or conformance diagnostic at one semantic authored-source position.
///
/// The kind and position jointly identify the violated authored invariant. This error reports the
/// failure but does not perform browser-style recovery, cascade, substitution, selector matching,
/// contextual resolution, or resource loading.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Error {
    kind: ErrorKind,
    position: CssSourcePosition,
}

impl Error {
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

    #[must_use]
    /// Returns the structured diagnostic detail for the parse failure.
    pub const fn kind(&self) -> &ErrorKind {
        &self.kind
    }

    #[must_use]
    /// Returns the stable machine-readable root category corresponding to [`Self::kind`].
    pub const fn code(&self) -> CssErrorCode {
        match self.kind {
            ErrorKind::EscapeParseError(_) => CssErrorCode::EscapeParseError,
            ErrorKind::UnexpectedEnd(_) => CssErrorCode::UnexpectedEnd,
            ErrorKind::UnexpectedToken(_) => CssErrorCode::UnexpectedToken,
            ErrorKind::InvalidAtRulePlacement(_) => CssErrorCode::InvalidAtRulePlacement,
            ErrorKind::InvalidAtRulePrelude(_) => CssErrorCode::InvalidAtRulePrelude,
            ErrorKind::InvalidAtRuleBody(_) => CssErrorCode::InvalidAtRuleBody,
            ErrorKind::NamespaceRedeclaration(_) => CssErrorCode::NamespaceRedeclaration,
            ErrorKind::LegacyNavigationTarget(_) => CssErrorCode::LegacyNavigationTarget,
            ErrorKind::UnknownAtRule(_) => CssErrorCode::UnknownAtRule,
            ErrorKind::UnsupportedAtRule(_) => CssErrorCode::UnsupportedAtRule,
            ErrorKind::InvalidQualifiedRule(_) => CssErrorCode::InvalidQualifiedRule,
            ErrorKind::InvalidSelector(_) => CssErrorCode::InvalidSelector,
            ErrorKind::InvalidMediaQuery(_) => CssErrorCode::InvalidMediaQuery,
            ErrorKind::UnknownProperty(_) => CssErrorCode::UnknownProperty,
            ErrorKind::UnsupportedProperty(_) => CssErrorCode::UnsupportedProperty,
            ErrorKind::InvalidPropertyValue(_) => CssErrorCode::InvalidPropertyValue,
            ErrorKind::InvalidComponentValue(_) => CssErrorCode::InvalidComponentValue,
            ErrorKind::InvalidDeclarationAnnotation(_) => {
                CssErrorCode::InvalidDeclarationAnnotation
            }
            ErrorKind::UnknownDescriptor(_) => CssErrorCode::UnknownDescriptor,
            ErrorKind::UnsupportedDescriptor(_) => CssErrorCode::UnsupportedDescriptor,
            ErrorKind::InvalidDescriptorValue(_) => CssErrorCode::InvalidDescriptorValue,
            ErrorKind::InvalidDescriptorCombination(_) => {
                CssErrorCode::InvalidDescriptorCombination
            }
            ErrorKind::InvalidColorSyntax(_) => CssErrorCode::InvalidColorSyntax,
            ErrorKind::NestingLimit(_) => CssErrorCode::NestingLimit,
        }
    }

    #[must_use]
    /// Returns the semantic authored-source position of the responsible token or missing token.
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

impl fmt::Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CSS parse error {:?} at {}",
            self.code(),
            self.position
        )
    }
}

impl std::error::Error for Error {}

pub(crate) fn from_parse_error(source: &str, error: ParseError<'_, Error>) -> Error {
    match error.kind {
        ParseErrorKind::Custom(error) => error,
        ParseErrorKind::Basic(kind) => basic_error(error.location, kind),
    }
    .resolve_source(source)
}

pub(crate) fn is_nesting_limit_error(error: &ParseError<'_, Error>) -> bool {
    matches!(
        error.kind,
        ParseErrorKind::Custom(Error {
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
) -> ParseError<'i, Error> {
    let position = CssSourcePosition::from_byte_offset_in(source, byte_offset);
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value().saturating_add(1),
    };
    ParseError {
        kind: ParseErrorKind::Custom(Error {
            kind: ErrorKind::NestingLimit(CssNestingLimitError {
                limit,
                enclosing_production: CssProductionId::new(enclosing_production),
            }),
            position,
        }),
        location,
    }
}

pub(crate) fn legacy_navigation_target(target: &str, position: CssSourcePosition) -> Error {
    Error {
        kind: ErrorKind::LegacyNavigationTarget(CssLegacyNavigationTargetError {
            target: target.to_owned(),
        }),
        position,
    }
}

pub(crate) fn namespace_redeclaration(
    prefix: Option<crate::CssNamespacePrefix>,
    position: CssSourcePosition,
    previous_position: CssSourcePosition,
) -> Error {
    Error {
        kind: ErrorKind::NamespaceRedeclaration(CssNamespaceRedeclarationError {
            prefix,
            previous_position,
        }),
        position,
    }
}

pub(crate) fn implicit_eof(source: &str) -> Error {
    Error {
        kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_CSS_SYNTAX,
        }),
        position: CssSourcePosition::from_byte_offset_in(source, source.len()),
    }
}

pub(crate) fn unterminated_at_rule(source: &str, byte_offset: usize) -> Error {
    Error {
        kind: ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: CssGrammarExpectation::new("a semicolon or block terminating an at-rule"),
        }),
        position: CssSourcePosition::from_byte_offset_in(source, byte_offset),
    }
}

pub(crate) fn escape_parse_error(
    source: &str,
    byte_offset: usize,
    detail: CssEscapeError,
) -> Error {
    Error {
        kind: ErrorKind::EscapeParseError(detail),
        position: CssSourcePosition::from_byte_offset_in(source, byte_offset),
    }
}

pub(crate) fn unexpected_token_at(source: &str, byte_offset: usize, token: &Token<'_>) -> Error {
    Error {
        kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
            expectation: EXPECT_CSS_SYNTAX,
            encountered: CssTokenSummary::from_token(token),
        }),
        position: CssSourcePosition::from_byte_offset_in(source, byte_offset),
    }
}

/// A name-free grammar failure from an actual retained source token or opener.
pub(crate) fn unexpected_component_value<'i>(
    component: &crate::CssComponentValue,
    expectation: &'static str,
) -> ParseError<'i, Error> {
    use crate::{
        CssBlockKind as Block, CssComponentValueRef as Component, CssValueTokenRef as Value,
    };
    let kind = match component.view() {
        Component::Token(token) => match token {
            Value::Ident(_) => CssTokenKind::Ident,
            Value::AtKeyword(_) => CssTokenKind::AtKeyword,
            Value::Hash {
                flag: crate::CssHashFlag::Id,
                ..
            } => CssTokenKind::IdHash,
            Value::Hash {
                flag: crate::CssHashFlag::Unrestricted,
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
    };
    let (authored, origin) = component
        .structural_lexeme(false)
        .expect("a source component has its original leaf or opening lexeme");
    let crate::CssValueOrigin::Parsed(origin) = origin else {
        unreachable!("source value grammar owns parsed component origins")
    };
    let position = origin.span().start();
    ParseError {
        location: cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value() + 1,
        },
        kind: ParseErrorKind::Custom(Error {
            position,
            kind: ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                expectation: CssGrammarExpectation::new(expectation),
                encountered: CssTokenSummary {
                    kind,
                    authored: authored.to_owned(),
                },
            }),
        }),
    }
}

pub(crate) fn unexpected_end_at<'i>(
    location: cssparser::SourceLocation,
    expectation: &'static str,
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: CssGrammarExpectation::new(expectation),
        }),
    )
}

pub(crate) fn from_rule_parse_error(
    source: &str,
    failed_unit: &str,
    error: ParseError<'_, Error>,
) -> Error {
    let mut error = from_parse_error(source, error);
    let unit = failed_unit.trim_start();
    // The adopted conditional grammar owns its prelude and terminal component
    // failures. Only a valid prelude's basic missing-body failure is contextualized.
    // Use decoded token identity so escaped at-keywords keep the same ownership.
    if let Some((_, _, Token::AtKeyword(name))) = crate::tokenization::next_source_token(unit, 0)
        && (name.eq_ignore_ascii_case("when") || name.eq_ignore_ascii_case("else"))
    {
        if matches!(
            error.kind,
            ErrorKind::UnexpectedEnd(_)
                | ErrorKind::UnexpectedToken(_)
                | ErrorKind::InvalidAtRuleBody(_)
        ) {
            error.kind = ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name.as_ref()),
                production: production_for_at_rule(&name),
                expectation: CssGrammarExpectation::new("a block body for this at-rule"),
                encountered: None,
            });
        }
        return error;
    }
    if let Some(after_at) = unit.strip_prefix('@') {
        let name_end = after_at
            .find(|character: char| !character.is_alphanumeric() && character != '-')
            .unwrap_or(after_at.len());
        let name = &after_at[..name_end];
        if unit.trim_end().ends_with(';') && at_rule_requires_block(name) {
            error.kind = ErrorKind::InvalidAtRuleBody(CssAtRuleSyntaxError {
                name: CssAtRuleName::new(name),
                production: production_for_at_rule(name),
                expectation: CssGrammarExpectation::new("a block body for this at-rule"),
                encountered: None,
            });
        }
    } else if !unit.contains('{')
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

fn basic_error(location: cssparser::SourceLocation, kind: BasicParseErrorKind<'_>) -> Error {
    match kind {
        BasicParseErrorKind::EndOfInput => Error::at(
            location,
            ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
                expectation: EXPECT_CSS_SYNTAX,
            }),
        ),
        BasicParseErrorKind::UnexpectedToken(token) => Error::at(
            location,
            ErrorKind::UnexpectedToken(CssUnexpectedTokenError {
                expectation: EXPECT_CSS_SYNTAX,
                encountered: CssTokenSummary::from_token(&token),
            }),
        ),
        BasicParseErrorKind::AtRuleInvalid(name) => {
            let name = name.to_string();
            if let Some(feature) = unsupported_at_rule_feature(&name) {
                Error::at(
                    location,
                    ErrorKind::UnsupportedAtRule(CssUnsupportedAtRuleError {
                        name: CssAtRuleName::new(name),
                        feature,
                    }),
                )
            } else {
                Error::at(
                    location,
                    ErrorKind::UnknownAtRule(CssUnknownAtRuleError {
                        name: CssAtRuleName::new(name),
                    }),
                )
            }
        }
        BasicParseErrorKind::QualifiedRuleInvalid => Error::at(
            location,
            ErrorKind::InvalidQualifiedRule(CssQualifiedRuleError {
                production: QUALIFIED_RULE,
                expectation: EXPECT_CSS_SYNTAX,
                encountered: None,
            }),
        ),
        BasicParseErrorKind::AtRuleBodyInvalid => Error::at(
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

pub(crate) fn basic<'i>(error: BasicParseError<'i>) -> ParseError<'i, Error> {
    error.into()
}

pub(crate) fn selector_basic<'i>(error: BasicParseError<'i>) -> ParseError<'i, Error> {
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
    detail: crate::CssComponentValueError,
) -> ParseError<'i, Error> {
    let crate::CssValueOrigin::Parsed(origin) = detail.origin() else {
        return invalid_component_value(fallback, detail);
    };
    let authored = &origin.source().as_str()
        [origin.span().start().byte_offset().value()..origin.span().end().byte_offset().value()];
    let kind = match detail.kind() {
        crate::CssComponentValueErrorKind::BadString => CssTokenKind::BadString,
        crate::CssComponentValueErrorKind::BadUrl => CssTokenKind::BadUrl,
        crate::CssComponentValueErrorKind::UnmatchedClosingDelimiter => match authored {
            ")" => CssTokenKind::CloseParenthesis,
            "]" => CssTokenKind::CloseSquareBracket,
            "}" => CssTokenKind::CloseCurlyBracket,
            _ => return invalid_component_value(fallback, detail),
        },
        _ => return invalid_component_value(fallback, detail),
    };
    let position = origin.span().start();
    ParseError {
        location: cssparser::SourceLocation {
            line: position.line().value(),
            column: position.column().value().saturating_add(1),
        },
        kind: ParseErrorKind::Custom(Error {
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
    detail: crate::CssComponentValueError,
) -> ParseError<'i, Error> {
    let position = match detail.origin() {
        crate::CssValueOrigin::Parsed(origin) => origin.span().start(),
        crate::CssValueOrigin::ImplicitClosure { opening, .. } => opening.span().start(),
        crate::CssValueOrigin::Programmatic | crate::CssValueOrigin::UnretainedInput { .. } => {
            CssSourcePosition::from_source_location(location)
        }
    };
    ParseError {
        location,
        kind: ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidComponentValue(Box::new(detail)),
            position,
        }),
    }
}

pub(crate) fn invalid_syntax<'i>(
    location: cssparser::SourceLocation,
    _reason: impl Into<String>,
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
    let location = cssparser::SourceLocation {
        line: position.line().value(),
        column: position.column().value() + 1,
    };
    ParseError {
        location,
        kind: ParseErrorKind::Custom(Error {
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
pub(crate) fn invalid_free_else(source: &str, position: CssSourcePosition) -> Error {
    let name = crate::tokenization::next_source_token(source, position.byte_offset().value())
        .and_then(|(_, _, token)| match token {
            Token::AtKeyword(name) => Some(name),
            _ => None,
        })
        .expect("parsed Else starts at its original at-keyword");
    Error {
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
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::InvalidAtRulePlacement(CssAtRulePlacementError {
            name: CssAtRuleName::new(name),
            expected_context: CssGrammarExpectation::new(expected_context),
        }),
    )
}

pub(crate) fn with_at_rule_prelude_context<'i>(
    mut error: ParseError<'i, Error>,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, Error> {
    let encountered = take_encountered(&mut error.kind);
    if !matches!(
        error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidMediaQuery(_),
            ..
        })
    ) {
        error.kind = ParseErrorKind::Custom(Error::at(
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
    let position =
        CssSourcePosition::from_cssparser(input.position(), input.current_source_location())
            .previous_ascii_byte();
    let mut location = input.current_source_location();
    location.column = location.column.saturating_sub(1).max(1);
    invalid_at_rule_body_at(position, location, name, production, expectation)
}

fn invalid_at_rule_body_at<'i>(
    position: CssSourcePosition,
    location: cssparser::SourceLocation,
    name: &str,
    production: &'static str,
    expectation: &'static str,
) -> ParseError<'i, Error> {
    ParseError {
        kind: ParseErrorKind::Custom(Error::at_exact_nonzero_byte_offset(
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

pub(crate) fn invalid_selector<'i, 't>(
    input: &Parser<'i, 't>,
    reason: impl Into<String>,
) -> ParseError<'i, Error> {
    invalid_selector_at(input.current_source_location(), reason)
}

pub(crate) fn invalid_selector_at<'i>(
    location: cssparser::SourceLocation,
    _reason: impl Into<String>,
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::InvalidSelector(CssSelectorError {
            production: Some(SELECTOR_LIST),
            expectation: EXPECT_SELECTOR,
            encountered: None,
        }),
    )
}

pub(crate) fn unsupported_property<'i>(
    location: cssparser::SourceLocation,
    name: impl Into<String>,
) -> ParseError<'i, Error> {
    let name = name.into();
    error_at(
        location,
        ErrorKind::UnsupportedProperty(CssUnsupportedPropertyError {
            feature: property_feature_id(&name),
            name: CssPropertyName::new(name),
        }),
    )
}

pub(crate) fn property_name_error<'i>(
    location: cssparser::SourceLocation,
    name: &str,
) -> ParseError<'i, Error> {
    match classify_property_name(name) {
        PropertyNameStatus::Supported | PropertyNameStatus::KnownUnsupported => {
            unsupported_property(location, name)
        }
        PropertyNameStatus::Unknown => error_at(
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
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnknownDescriptor(CssUnknownDescriptorError {
            at_rule: CssAtRuleName::new(at_rule),
            descriptor: CssDescriptorName::new(name),
        }),
    )
}

pub(crate) fn invalid_descriptor_combination<'i, 't>(
    input: &Parser<'i, 't>,
    position: Option<CssSourcePosition>,
    at_rule: &str,
    responsible: &str,
    conflicting: &[&str],
) -> ParseError<'i, Error> {
    let kind = ErrorKind::InvalidDescriptorCombination(CssDescriptorCombinationError {
        at_rule: CssAtRuleName::new(at_rule),
        responsible: CssDescriptorName::new(responsible),
        conflicting: conflicting
            .iter()
            .map(|name| CssDescriptorName::new(*name))
            .collect(),
    });
    let error =
        if let Some(position) = position.filter(|position| position.byte_offset().value() != 0) {
            Error::at_exact_nonzero_byte_offset(position, kind)
        } else {
            Error::at(input.current_source_location(), kind)
        };
    ParseError {
        kind: ParseErrorKind::Custom(error),
        location: input.current_source_location(),
    }
}

pub(crate) fn unsupported_value<'i, 't>(
    input: &Parser<'i, 't>,
    _property: Option<&str>,
    _reason: impl Into<String>,
) -> ParseError<'i, Error> {
    unexpected_at(input.current_source_location())
}

pub(crate) fn unsupported_value_at<'i>(
    location: cssparser::SourceLocation,
    _property: Option<&str>,
    _reason: impl Into<String>,
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_DECLARATION_VALUE,
        }),
    )
}

pub(crate) fn invalid_color<'i>(
    location: cssparser::SourceLocation,
    component: Option<&str>,
) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::InvalidColorSyntax(CssColorSyntaxError {
            component: component.map(CssColorComponentName::new),
            expectation: EXPECT_COLOR,
            encountered: None,
        }),
    )
}

pub(crate) fn with_color_context<'i>(
    mut error: ParseError<'i, Error>,
    component: Option<&str>,
) -> ParseError<'i, Error> {
    if matches!(
        error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidColorSyntax(_),
            ..
        })
    ) {
        return error;
    }
    let encountered = take_encountered(&mut error.kind);
    error.kind = ParseErrorKind::Custom(Error::at(
        error.location,
        ErrorKind::InvalidColorSyntax(CssColorSyntaxError {
            component: component.map(CssColorComponentName::new),
            expectation: EXPECT_COLOR,
            encountered,
        }),
    ));
    error
}

pub(crate) fn with_media_query_context<'i>(
    mut error: ParseError<'i, Error>,
    feature: Option<&str>,
) -> ParseError<'i, Error> {
    if matches!(
        error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidMediaQuery(_),
            ..
        })
    ) {
        return error;
    }
    let encountered = take_encountered(&mut error.kind);
    error.kind = ParseErrorKind::Custom(Error::at(
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
) -> ParseError<'i, Error> {
    let mut error = with_media_query_context(
        invalid_syntax(location, "custom-media references require boolean context"),
        Some(name),
    );
    if let ParseErrorKind::Custom(Error {
        kind: ErrorKind::InvalidMediaQuery(detail),
        ..
    }) = &mut error.kind
    {
        detail.committed_context = true;
    }
    error
}

pub(crate) fn with_property_context<'i>(
    mut error: ParseError<'i, Error>,
    property: &str,
) -> ParseError<'i, Error> {
    let Some(property) = property_for_supported_name(property) else {
        return error;
    };
    if matches!(
        &error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidColorSyntax(_),
            ..
        })
    ) || matches!(
        &error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidComponentValue(detail),
            ..
        }) if is_component_resource_error(detail)
    ) {
        return error;
    }
    let encountered = take_encountered(&mut error.kind);
    error.kind = ParseErrorKind::Custom(Error::at(
        error.location,
        ErrorKind::InvalidPropertyValue(CssPropertyValueError {
            property,
            expectation: EXPECT_PROPERTY_VALUE,
            encountered,
        }),
    ));
    error
}

/// Resource failures are component invariants even when found during a
/// property-specific preflight. Keep their typed kind and original token origin.
pub(crate) fn is_component_resource_error(detail: &crate::CssComponentValueError) -> bool {
    matches!(
        detail.kind(),
        crate::CssComponentValueErrorKind::NestingLimit
            | crate::CssComponentValueErrorKind::ComponentLimit
            | crate::CssComponentValueErrorKind::ByteLimit
            | crate::CssComponentValueErrorKind::CapacityOverflow
    )
}

/// A speculative grammar alternative must not erase resource failures.
pub(crate) fn is_resource_parse_error(error: &ParseError<'_, Error>) -> bool {
    is_nesting_limit_error(error)
        || matches!(
            &error.kind,
            ParseErrorKind::Custom(Error { kind: ErrorKind::InvalidComponentValue(detail), .. })
                if is_component_resource_error(detail)
        )
}

pub(crate) fn with_descriptor_context<'i>(
    mut error: ParseError<'i, Error>,
    at_rule: &str,
    descriptor: &str,
) -> ParseError<'i, Error> {
    if matches!(
        error.kind,
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::UnknownDescriptor(_)
                | ErrorKind::UnsupportedDescriptor(_)
                | ErrorKind::InvalidDeclarationAnnotation(_),
            ..
        })
    ) {
        return error;
    }
    let origin = match &error.kind {
        ParseErrorKind::Custom(Error {
            kind: ErrorKind::InvalidDescriptorValue(detail),
            ..
        }) => detail.origin,
        _ => DiagnosticOrigin::Inferred,
    };
    let encountered = take_encountered(&mut error.kind);
    let contextual = Error::at(
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
    let error = Error::at(
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
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
) -> ParseError<'i, Error> {
    ParseError {
        kind: ParseErrorKind::Custom(Error::at(location, kind)),
        location,
    }
}

fn unexpected_at<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    error_at(
        location,
        ErrorKind::UnexpectedEnd(CssUnexpectedEndError {
            expectation: EXPECT_DECLARATION_VALUE,
        }),
    )
}

fn take_encountered(kind: &mut ParseErrorKind<'_, Error>) -> Option<CssTokenSummary> {
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
        || name.eq_ignore_ascii_case("font-palette-values")
        || name.eq_ignore_ascii_case("keyframes")
        || name.eq_ignore_ascii_case("media")
        || name.eq_ignore_ascii_case("container")
        || name.eq_ignore_ascii_case("scope")
        || name.eq_ignore_ascii_case("page")
}

fn property_feature_id(name: &str) -> CssFeatureId {
    let _ = name;
    CssFeatureId::new("baseline.property.recognized-unsupported")
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
                ErrorKind::InvalidPropertyValue(CssPropertyValueError {
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
        ];

        for (kind, expected) in roots {
            assert_eq!(Error::at(position(), kind).code(), expected);
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
