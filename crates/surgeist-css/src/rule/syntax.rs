//! Authored CSS syntax values produced by this crate's parser.
//!
//! Property-coupled declarations represent CSS-owned authored syntax without a
//! broad property/value cross product. Property-specific parsers decide which
//! value forms are accepted. CSS owns intrinsic declaration expansion; downstream
//! crates own substitution environments, cascade, and contextual resolution.
//!
//! Parsed declarations retain authored source locations, while checked Rust
//! construction preserves supplied token origins without inventing coordinates.
//! Downstream adapters can inspect either without depending on parser internals.

use crate::component::CssValueOrigin;
use crate::declaration::CssDeclarationList;
use crate::query::CssCondition;
use crate::query::container::CssContainerPrelude;
use crate::query::media::CssMediaQueryList;
use crate::query::supports::CssSupportsLeaf;
use crate::rule::color_profile::CssColorProfileRule;
use crate::rule::font_feature_values::CssFontFeatureValuesRule;
use crate::rule::font_palette_values::CssFontPaletteValuesRule;
use crate::rule::keyframes::CssKeyframesRule;
use crate::selector::{CssScopeSelectorList, CssStyleSelectorList};
use crate::values::common::CssIdent;
use std::sync::Arc;

use crate::component::CssComponentValues;
use crate::source::CssParsedOrigin;

use crate::source::CssSourcePosition;

/// A parser-produced authored stylesheet.
///
/// The private fields guarantee that every retained rule is valid authored CSS
/// syntax. Discarded source remains in the parse report diagnostics; the sheet
/// does not apply cascade, substitution, selector matching, or resource loading.
///
/// # Specified serialization
///
/// Serializes retained authored rules; discarded source and its diagnostics remain on the original
/// parse report. Top-level rules are separated by one newline.
#[derive(Clone, Debug, Default)]
pub struct CssSheet {
    location: Option<crate::source::CssStylesheetLocation>,
    rules: Vec<CssRule>,
    parse_source: Option<crate::source::CssParseSource>,
}

impl PartialEq for CssSheet {
    fn eq(&self, other: &Self) -> bool {
        self.location == other.location && self.rules == other.rules
    }
}

impl CssSheet {
    /// Creates an empty sheet with no rules, no stylesheet location and no parse source.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            location: None,
            rules: Vec::new(),
            parse_source: None,
        }
    }

    pub(crate) fn push_rule(&mut self, rule: CssRule) {
        self.rules.push(rule);
    }

    /// Complete root source and actual grammar context; absent for construction.
    #[must_use]
    pub const fn parse_source(&self) -> Option<&crate::source::CssParseSource> {
        self.parse_source.as_ref()
    }

    pub(crate) fn bind_parse_source(&mut self, source: crate::source::CssParseSource) {
        self.parse_source = Some(source);
    }

    /// Returns the host-supplied symbolic stylesheet location, or null.
    /// Relative URLs are not resolved against this value by the CSS crate.
    #[must_use]
    pub const fn location(&self) -> Option<&crate::source::CssStylesheetLocation> {
        self.location.as_ref()
    }

    /// Attaches the host-supplied symbolic stylesheet location, or null when absent.
    ///
    /// The location is opaque host context. It never affects parsing, and it takes
    /// part in equality.
    #[must_use]
    pub fn with_location(mut self, location: Option<crate::source::CssStylesheetLocation>) -> Self {
        self.location = location;
        self
    }

    pub(crate) fn set_location(&mut self, location: Option<crate::source::CssStylesheetLocation>) {
        self.location = location;
    }

    /// Returns the valid authored rules retained in source order.
    ///
    /// Recovery diagnostics remain on the parse report. Reading the rules does
    /// not imply that the source was clean or perform downstream CSS behavior.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }
}

/// One checked rule in the authored stylesheet phase.
///
/// The non-exhaustive union contains only syntax that passed its rule grammar;
/// discarded source units are represented by report diagnostics instead. A rule
/// does not perform cascade, substitution, selector matching, query evaluation,
/// resource loading, or contextual resolution.
///
/// # Specified serialization
///
/// Serializes the authored rule graph, preserving declaration and child order.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssRule {
    /// A `@custom-media` rule.
    CustomMedia(crate::rule::CssCustomMediaRule),
    /// An `@import` rule.
    Import(CssImportRule),
    /// A `@namespace` rule.
    Namespace(CssNamespaceRule),
    /// A `@counter-style` rule.
    CounterStyle(CssCounterStyleRule),
    /// A `@page` rule.
    Page(CssPageRule),
    /// A `@layer` statement rule that only declares layer names.
    LayerStatement(CssLayerStatementRule),
    /// A `@layer` block rule that holds child rules.
    LayerBlock(CssLayerBlockRule),
    /// A `@font-face` rule.
    FontFace(CssFontFaceRule),
    /// A `@font-feature-values` rule.
    FontFeatureValues(CssFontFeatureValuesRule),
    /// A `@font-palette-values` rule.
    FontPaletteValues(CssFontPaletteValuesRule),
    /// A `@color-profile` rule.
    ColorProfile(CssColorProfileRule),
    /// A `@keyframes` rule.
    Keyframes(CssKeyframesRule),
    /// A style rule: a selector list with its declarations and nested rules.
    Style(CssStyleRule),
    /// A run of declarations nested directly in a style rule or conditional group.
    NestedDeclarations(CssNestedDeclarationsRule),
    /// A `@media` rule.
    Media(CssMediaRule),
    /// A `@supports` rule.
    Supports(CssSupportsRule),
    /// A `@supports-condition` rule that defines a named supports condition.
    SupportsCondition(crate::rule::CssSupportsConditionRule),
    /// A `@container` rule.
    Container(CssContainerRule),
    /// A `@when` rule.
    When(crate::rule::CssWhenRule),
    /// An `@else` rule.
    Else(crate::rule::CssElseRule),
    /// A `@scope` rule.
    Scope(CssScopeRule),
}

use crate::rule::page::syntax::*;

use crate::rule::counter_style::CssCounterStyleRule;

/// One intrinsically valid authored Namespaces 3 declaration.
///
/// The declaration retains its optional decoded prefix, literal namespace name,
/// and optional parser-produced source position. Placement and redeclaration
/// conformance belong to the declaring stylesheet. It does not resolve namespace
/// names, load resources, or perform selector matching.
///
/// # Specified serialization
///
/// Serializes the declaration using CSSOM's escaped prefix and quoted URL form. CSSOM string
/// serialization replaces programmatic U+0000 with U+FFFD.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamespaceRule {
    prefix: Option<CssNamespacePrefix>,
    name: CssNamespaceName,
    position: Option<CssSourcePosition>,
}

impl CssNamespaceRule {
    /// Constructs a declaration from a checked decoded prefix and a literal name.
    /// An omitted prefix declares the default. Empty names denote the null namespace.
    /// Programmatic construction has no authored source position.
    #[must_use]
    pub const fn new(prefix: Option<CssNamespacePrefix>, name: CssNamespaceName) -> Self {
        Self {
            prefix,
            name,
            position: None,
        }
    }

    pub(crate) const fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the optional exact, case-sensitive decoded namespace prefix.
    #[must_use]
    pub const fn prefix(&self) -> Option<&CssNamespacePrefix> {
        self.prefix.as_ref()
    }

    /// Returns the literal string carried by the authored string or `url()` token.
    #[must_use]
    pub const fn name(&self) -> &CssNamespaceName {
        &self.name
    }

    /// Returns the rule's authored at-keyword position, or `None` for Rust construction.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// One checked, decoded Namespaces 3 prefix.
///
/// Construction accepts exactly one decoded CSS identifier. Prefix equality is
/// exact and case-sensitive.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssNamespacePrefix {
    value: String,
}

impl CssNamespacePrefix {
    pub(crate) fn diagnostic_backing_bytes(&self) -> usize {
        self.value.capacity()
    }

    /// Constructs a prefix from a decoded CSS identifier, which may require escapes
    /// in its CSS spelling. Empty values and NUL cannot retain identifier identity.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        crate::component::CssComponentValue::try_ident(value.clone()).ok()?;
        Some(Self { value })
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        debug_assert!(crate::component::CssComponentValue::try_ident(value.clone()).is_ok());
        Self { value }
    }

    /// Returns the exact, case-sensitive decoded identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The literal namespace name from a Namespaces 3 declaration.
///
/// Empty values and strings that are not valid URIs remain valid authored
/// syntax. This value is not normalized, resolved, or loaded.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssNamespaceName {
    value: String,
}

impl CssNamespaceName {
    /// Constructs an authored literal namespace name without URI validation.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the literal string carried by the authored string or `url()` token.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// A checked `@import` rule: its target, and optional layer, supports condition and media query list.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImportRule {
    target: CssImportTarget,
    layer: Option<CssImportLayer>,
    supports: Option<CssImportSupports>,
    media: Option<CssMediaQueryList>,
    pub(crate) syntax: Box<crate::rule::imports::ImportSyntax>,
}

impl CssImportRule {
    #[must_use]
    pub(crate) fn new(
        target: CssImportTarget,
        layer: Option<CssImportLayer>,
        supports: Option<CssImportSupports>,
        media: Option<CssMediaQueryList>,
        syntax: crate::rule::imports::ImportSyntax,
    ) -> Self {
        Self {
            target,
            layer,
            supports,
            media,
            syntax: Box::new(syntax),
        }
    }

    /// Returns the imported resource, either a `url()` or a string.
    #[must_use]
    pub const fn target(&self) -> &CssImportTarget {
        &self.target
    }

    /// Returns the `layer` clause, or `None` when the import has no layer clause.
    #[must_use]
    pub const fn layer(&self) -> Option<&CssImportLayer> {
        self.layer.as_ref()
    }

    /// Returns the optional authored supports condition that follows the import
    /// target and layer clause.
    #[must_use]
    pub const fn supports(&self) -> Option<&CssImportSupports> {
        self.supports.as_ref()
    }

    /// Returns the media query list that follows the import conditions, or `None` when the rule has none.
    #[must_use]
    pub const fn media(&self) -> Option<&CssMediaQueryList> {
        self.media.as_ref()
    }

    /// Returns the origin of the `@import` at-keyword.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        self.syntax.at_keyword.origin()
    }

    /// Returns the source position of the `@import` at-keyword, or `None` when its origin was not parsed from source.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
}

/// The resource an `@import` rule names.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportTarget {
    /// A `url()` target, as written.
    Url(CssImportUrl),
    /// A string literal, as written.
    String(CssImportString),
}

/// An `@import` target written as a URL function, retaining the shared authored URL.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssImportUrl {
    url: CssUrl,
}

impl CssImportUrl {
    /// Preserves a shared authored URL, including its function identity and modifiers.
    #[must_use]
    pub fn new(url: CssUrl) -> Self {
        Self { url }
    }

    /// Returns the shared authored URL without losing its function or modifiers.
    #[must_use]
    pub const fn url(&self) -> &CssUrl {
        &self.url
    }

    /// Returns the URL text without its function wrapper.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.url.as_str()
    }
}

/// An `@import` target written as a string, holding its decoded value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssImportString {
    value: String,
}

impl CssImportString {
    /// Preserves a decoded authored string, including empty or whitespace values.
    /// Resource resolution and usability belong to the importing layer.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        let value = value.into();
        Self { value }
    }

    /// Returns the decoded string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

/// The `layer` clause of an `@import` rule.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssImportLayer {
    /// `layer`: the import is placed in an anonymous cascade layer.
    Anonymous,
    /// `layer(<layer-name>)`: the import is placed in the named cascade layer.
    Named(CssLayerName),
}

/// One parser-produced Cascade 4 supports condition on an import rule.
///
/// The condition preserves authored syntax only. It is not evaluated and does
/// not load the imported resource.
#[derive(Clone, Debug, PartialEq)]
pub struct CssImportSupports {
    condition: CssCondition<CssSupportsLeaf>,
}

impl CssImportSupports {
    #[must_use]
    pub(crate) const fn new(condition: CssCondition<CssSupportsLeaf>) -> Self {
        Self { condition }
    }

    /// Returns the authored declaration test or full supports condition.
    #[must_use]
    pub const fn condition(&self) -> &CssCondition<CssSupportsLeaf> {
        &self.condition
    }
}
use crate::rule::font_face::CssFontFaceRule;

/// A checked authored descriptor value and its semantic name position.
///
/// The private fields preserve the coupling between a validated descriptor value and the source
/// position of its descriptor-name start. Construction is parser-owned, so callers cannot forge
/// provenance. Parsed counter descriptors also retain their original name/value regions and
/// complete component trees. Semantic construction and descriptor parsers without retained
/// lexical metadata do not invent parsed origins. This occurrence does not apply descriptor
/// matching or load resources.
///
/// ```compile_fail
/// use surgeist_css::rule::CssDescriptorOccurrence;
/// use surgeist_css::rule::font_face::CssFontDisplay;
/// let _ = CssDescriptorOccurrence { value: CssFontDisplay::Swap, position: todo!() };
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssDescriptorOccurrence<T> {
    value: T,
    position: CssSourcePosition,
    parsed: Option<Arc<CssParsedDescriptor>>,
}

#[derive(Debug, PartialEq)]
struct CssParsedDescriptor {
    name: CssParsedOrigin,
    value: CssParsedOrigin,
    components: CssComponentValues,
}

impl<T> CssDescriptorOccurrence<T> {
    #[cfg(test)]
    pub(crate) const fn new(value: T, position: CssSourcePosition) -> Self {
        Self {
            value,
            position,
            parsed: None,
        }
    }

    pub(crate) fn from_parsed(
        value: T,
        name: CssParsedOrigin,
        value_origin: CssParsedOrigin,
        components: CssComponentValues,
    ) -> Self {
        Self {
            value,
            position: name.span().start(),
            parsed: Some(Arc::new(CssParsedDescriptor {
                name,
                value: value_origin,
                components,
            })),
        }
    }

    /// Reports identity of the genuine parsed occurrence, including across projection clones.
    #[must_use]
    pub fn same_occurrence(&self, other: &Self) -> bool {
        match (&self.parsed, &other.parsed) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            _ => std::ptr::eq(self, other),
        }
    }

    /// Borrows the original descriptor-name token when retained by its parser.
    #[must_use]
    pub fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        self.parsed.as_ref().map(|parsed| &parsed.name)
    }

    /// Borrows the original descriptor-value region when retained by its parser.
    #[must_use]
    pub fn parsed_value(&self) -> Option<&CssParsedOrigin> {
        self.parsed.as_ref().map(|parsed| &parsed.value)
    }

    /// Borrows every original value component, including trivia and nested delimiters.
    /// The retained source stays distinct from specified-text projection.
    #[must_use]
    pub fn value_components(&self) -> Option<&CssComponentValues> {
        self.parsed.as_ref().map(|parsed| &parsed.components)
    }

    /// Returns the typed authored descriptor value.
    #[must_use]
    pub const fn value(&self) -> &T {
        &self.value
    }

    /// Returns the semantic source position at the descriptor-name start.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

impl<T> std::ops::Deref for CssDescriptorOccurrence<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value()
    }
}

/// # Specified serialization
///
/// Formats this checked name through its native aggregate/component writer. Each identifier is
/// escaped independently; dots are grammar punctuation. One name aggregate and each component share
/// input/projection/output allowances.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssLayerName {
    components: Vec<String>,
}

impl CssLayerName {
    /// Constructs a name from decoded identifier components, which are the dot-separated parts of the `<layer-name>`.
    ///
    /// Returns `None` when there are no components, when a component is not a valid identifier, or when a component is a CSS-wide keyword (case-insensitively).
    #[must_use]
    pub fn try_new(components: impl IntoIterator<Item = impl Into<String>>) -> Option<Self> {
        let components = components.into_iter().map(Into::into).collect::<Vec<_>>();
        if components.is_empty()
            || components
                .iter()
                .any(|component| !is_valid_layer_name_component(component))
        {
            None
        } else {
            Some(Self::new(components))
        }
    }

    #[must_use]
    pub(crate) fn new(components: Vec<String>) -> Self {
        debug_assert!(!components.is_empty());
        debug_assert!(
            components
                .iter()
                .all(|component| is_valid_layer_name_component(component))
        );
        Self { components }
    }

    /// Returns the decoded identifier components in order.
    #[must_use]
    pub fn components(&self) -> &[String] {
        &self.components
    }
}

/// A nonempty ordered list of cascade layer names, as in a `@layer a, b;` statement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssLayerNameList {
    names: Vec<CssLayerName>,
}

impl CssLayerNameList {
    /// Constructs a list from `names`, or returns `None` when `names` is empty.
    #[must_use]
    pub fn try_new(names: Vec<CssLayerName>) -> Option<Self> {
        if names.is_empty() {
            None
        } else {
            Some(Self::new(names))
        }
    }

    #[must_use]
    pub(crate) fn new(names: Vec<CssLayerName>) -> Self {
        debug_assert!(!names.is_empty());
        Self { names }
    }

    /// Returns the names in authored order; the list is never empty.
    #[must_use]
    pub fn names(&self) -> &[CssLayerName] {
        &self.names
    }
}

/// A `@layer` statement rule that declares one or more layer names without a block.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLayerStatementRule {
    names: CssLayerNameList,
    position: CssSourcePosition,
}

impl CssLayerStatementRule {
    #[must_use]
    pub(crate) const fn new(names: CssLayerNameList, position: CssSourcePosition) -> Self {
        Self { names, position }
    }

    /// Returns the names the statement declares.
    #[must_use]
    pub const fn names(&self) -> &CssLayerNameList {
        &self.names
    }

    /// Returns the source position of the rule.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// A `@layer` block rule: an optional layer name and the child rules in that layer.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLayerBlockRule {
    name: Option<CssLayerName>,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssLayerBlockRule {
    #[must_use]
    pub(crate) fn new(
        name: Option<CssLayerName>,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            name,
            rules,
            position: Some(position),
        }
    }

    /// Returns the layer name, or `None` for an anonymous layer block.
    #[must_use]
    pub const fn name(&self) -> Option<&CssLayerName> {
        self.name.as_ref()
    }

    /// Returns the child rules in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

fn is_valid_layer_name_component(component: &str) -> bool {
    // Components are decoded identifier values, not authored token spellings.
    !is_parser_reserved_layer_name(component) && CssIdent::try_new(component).is_ok()
}

fn is_parser_reserved_layer_name(component: &str) -> bool {
    matches!(
        component.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

/// A checked `@media` rule: a media query list and its child rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMediaRule {
    query: CssMediaQueryList,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssMediaRule {
    #[must_use]
    pub(crate) fn new(
        query: CssMediaQueryList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            query,
            rules,
            position: Some(position),
        }
    }

    /// Returns the media query list that gates the child rules.
    #[must_use]
    pub const fn query(&self) -> &CssMediaQueryList {
        &self.query
    }

    /// Returns the child rules in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// A parser-produced Conditional Rules 3 `@supports` group rule.
///
/// The condition and children preserve authored syntax only. This value does
/// not evaluate support, match selectors, apply cascade, or resolve values.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSupportsRule {
    condition: CssCondition<CssSupportsLeaf>,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssSupportsRule {
    #[must_use]
    pub(crate) fn new(
        condition: CssCondition<CssSupportsLeaf>,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            condition,
            rules,
            position: Some(position),
        }
    }

    /// Returns the authored `@supports` condition.
    #[must_use]
    pub const fn condition(&self) -> &CssCondition<CssSupportsLeaf> {
        &self.condition
    }

    /// Returns the child rules in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// A checked `@container` rule: its container prelude and child rules.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerRule {
    prelude: CssContainerPrelude,
    rules: Vec<CssRule>,
    position: Option<CssSourcePosition>,
}

impl CssContainerRule {
    #[must_use]
    pub(crate) fn new(
        prelude: CssContainerPrelude,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            prelude,
            rules,
            position: Some(position),
        }
    }

    /// Returns every authored entry and its shared lexical prelude.
    #[must_use]
    pub const fn prelude(&self) -> &CssContainerPrelude {
        &self.prelude
    }

    /// Returns the child rules in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.rules
    }

    /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// One authored `@scope` rule: its optional root and limit boundaries and its body.
///
/// The body is an ordinary [`CssRuleList`](crate::rule::CssRuleList) of [`CssRule`]s. Placement inside a scope
/// (no `@import`, `@namespace` or `@page`, declaration runs only under a style
/// ancestor) and the no-pseudo-element boundary invariant are checks of the
/// construction and assembly context, not separate types.
#[derive(Clone, Debug, PartialEq)]
pub struct CssScopeRule {
    root: Option<CssScopeSelectorList>,
    limit: Option<CssScopeSelectorList>,
    rules: crate::rule::CssRuleList,
    position: Option<CssSourcePosition>,
}

impl CssScopeRule {
    #[must_use]
    pub(crate) fn new(
        root: Option<CssScopeSelectorList>,
        limit: Option<CssScopeSelectorList>,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            root,
            limit,
            rules: crate::rule::CssRuleList::from_parsed(rules),
            position: Some(position),
        }
    }

    /// Returns the scoping root selector list, or `None` when the rule has no root prelude.
    #[must_use]
    pub const fn root(&self) -> Option<&CssScopeSelectorList> {
        self.root.as_ref()
    }

    /// Returns the scoping limit selector list, or `None` when the rule has no `to` limit.
    #[must_use]
    pub const fn limit(&self) -> Option<&CssScopeSelectorList> {
        self.limit.as_ref()
    }

    /// Returns the child rules of the scope.
    #[must_use]
    pub const fn rules(&self) -> &crate::rule::CssRuleList {
        &self.rules
    }

    /// Returns the source position of the rule, or `None` when it was constructed rather than parsed.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// Validation context for checked scope construction, not a complete ancestry
/// record. Final stylesheet or group assembly rechecks actual ancestry. The
/// nearest enclosing style or scope binds implicit relative roots; explicit
/// root `&` and declaration runs use any enclosing style ancestor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScopeNestingContext {
    /// No enclosing style or scope. Relative roots are invalid.
    None,
    /// A style ancestor supplies explicit root `&` and declaration runs, including
    /// when another scope intervenes. Implicit relatives retain no synthetic anchor.
    Style,
    /// An enclosing scope supplies the root's scope context, without a style ancestor.
    Scope,
}

// Shared parser-produced style-body payload. Syntax owns the declaration-run
// invariant; both selector-bearing rules and raw blocks retain this same shape.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StyleContents {
    pub(crate) declarations: CssDeclarationList,
    pub(crate) rules: Vec<CssRule>,
}

impl StyleContents {
    pub(crate) fn into_nested_rules(self) -> Vec<CssRule> {
        let mut rules = Vec::new();
        if !self.declarations.is_empty() {
            rules.push(CssRule::NestedDeclarations(CssNestedDeclarationsRule::new(
                self.declarations,
            )));
        }
        rules.extend(self.rules);
        rules
    }
}

/// One parser-produced style block with its genuine authored brace region.
///
/// Leading declarations precede the first retained child rule. Later declaration
/// runs appear as [`CssRule::NestedDeclarations`] among [`Self::rules`]. Child
/// selectors remain symbolic; no enclosing selector or parent list is fabricated.
/// Equality compares content and origin source text/span, not snapshot identity.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleBlock {
    contents: StyleContents,
    origin: CssParsedOrigin,
}

impl CssStyleBlock {
    pub(crate) fn new(contents: StyleContents, origin: CssParsedOrigin) -> Self {
        Self { contents, origin }
    }

    /// Returns leading declarations when the first retained body item is a declaration run.
    /// A rejected rule can end this run without producing a retained child.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.contents.declarations
    }

    /// Returns child rules and subsequent declaration runs in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.contents.rules
    }

    /// Returns the opening brace through the explicit closing brace or actual EOF.
    /// Surrounding trivia is excluded; the snapshot retains the entire original input.
    #[must_use]
    pub const fn origin(&self) -> &CssParsedOrigin {
        &self.origin
    }
}

/// One authored style rule with a complete selector list and ordered children.
///
/// Leading declarations belong to this rule. Declaration runs after its first retained child
/// appear as [`CssRule::NestedDeclarations`] in [`Self::rules`]. Nested selectors retain symbolic
/// parent references; this tree does not expand selectors or select cascade winners.
#[derive(Clone, Debug, PartialEq)]
pub struct CssStyleRule {
    selectors: CssStyleSelectorList,
    contents: StyleContents,
    position: CssSourcePosition,
}

impl CssStyleRule {
    pub(crate) fn new(
        selectors: CssStyleSelectorList,
        declarations: CssDeclarationList,
        rules: Vec<CssRule>,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            selectors,
            contents: StyleContents {
                declarations,
                rules,
            },
            position,
        }
    }

    /// Returns the complete authored selector list, without parent-selector expansion.
    #[must_use]
    pub const fn selectors(&self) -> &CssStyleSelectorList {
        &self.selectors
    }

    /// Returns leading declarations when the first retained body item is a declaration run.
    /// A rejected rule can end this run without producing a retained child.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.contents.declarations
    }

    /// Returns nested rules and subsequent declaration runs in authored order.
    #[must_use]
    pub fn rules(&self) -> &[CssRule] {
        &self.contents.rules
    }

    /// Returns the source position of the authored selector-list start.
    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// A nonempty declaration run in a style rule's child list or a nested conditional group.
///
/// It inherits the enclosing style rule's selector context, including pseudo-elements, rather
/// than synthesizing an `&` selector. Declaration positions preserve its original provenance.
#[derive(Clone, Debug, PartialEq)]
pub struct CssNestedDeclarationsRule {
    declarations: CssDeclarationList,
}

impl CssNestedDeclarationsRule {
    pub(crate) fn new(declarations: CssDeclarationList) -> Self {
        debug_assert!(!declarations.is_empty());
        Self { declarations }
    }

    /// Returns the declarations of the run in authored order; the list is never empty.
    #[must_use]
    pub const fn declarations(&self) -> &CssDeclarationList {
        &self.declarations
    }

    /// Returns the position of the first declaration in this nonempty run.
    #[must_use]
    pub fn position(&self) -> CssSourcePosition {
        self.declarations[0]
            .position()
            .expect("a parsed nested declaration run contains parsed declarations")
    }
}
use crate::values::common::CssUrl;

impl CssSheet {
    /// Checks ordered authored rules, deriving bindings from leading namespace declarations.
    /// No supplied location or enclosing parsed position is invented.
    pub fn try_from_rules(
        rules: Vec<CssRule>,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::sheet(&rules)?;
        Ok(Self {
            location: None,
            rules,
            parse_source: None,
        })
    }
}

impl CssMediaRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing
    /// provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing
    /// assembly.
    pub fn try_new(
        query: CssMediaQueryList,
        rules: Vec<CssRule>,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            query,
            rules,
            position: None,
        })
    }
}

impl CssSupportsRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing
    /// provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing
    /// assembly.
    pub fn try_new(
        condition: CssCondition<CssSupportsLeaf>,
        rules: Vec<CssRule>,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::supports(&condition, context)?;
        crate::rule::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            condition,
            rules,
            position: None,
        })
    }
}

impl CssContainerRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing
    /// provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing
    /// assembly.
    pub fn try_new(
        prelude: CssContainerPrelude,
        rules: Vec<CssRule>,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            prelude,
            rules,
            position: None,
        })
    }
}

impl CssLayerBlockRule {
    /// Assembles an ordinary group with explicit namespace bindings and absent enclosing
    /// provenance.
    /// Parsed descendants retain their own positions; nested placement is rechecked by enclosing
    /// assembly.
    pub fn try_new(
        name: Option<CssLayerName>,
        rules: Vec<CssRule>,
        context: &crate::selector::CssNamespaceContext,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::ordinary_group(&rules, context)?;
        Ok(Self {
            name,
            rules,
            position: None,
        })
    }
}

impl CssScopeRule {
    /// Assembles boundaries and a direct scope body in the supplied authored
    /// nesting context, checking selectors and the stricter page placement.
    pub fn try_new(
        root: Option<CssScopeSelectorList>,
        limit: Option<CssScopeSelectorList>,
        rules: Vec<CssRule>,
        context: &crate::selector::CssNamespaceContext,
        nesting: CssScopeNestingContext,
    ) -> Result<Self, crate::rule::CssRuleConstructionError> {
        crate::rule::rule_construction::scope(
            root.as_ref(),
            limit.as_ref(),
            &rules,
            context,
            nesting,
        )?;
        Ok(Self {
            root,
            limit,
            rules: crate::rule::CssRuleList::from_parsed(rules),
            position: None,
        })
    }
}

// Parser-only sibling mutation, after complete structural reconstruction.
impl CssSheet {
    pub(crate) fn rules_mut(&mut self) -> &mut Vec<CssRule> {
        &mut self.rules
    }
}
impl CssStyleBlock {
    pub(crate) fn rules_mut(&mut self) -> &mut Vec<CssRule> {
        &mut self.contents.rules
    }
}
impl CssRule {
    pub(crate) fn parser_children_mut(&mut self) -> Option<&mut Vec<CssRule>> {
        match self {
            Self::Style(rule) => Some(&mut rule.contents.rules),
            Self::Media(rule) => Some(&mut rule.rules),
            Self::Supports(rule) => Some(&mut rule.rules),
            Self::Container(rule) => Some(&mut rule.rules),
            Self::LayerBlock(rule) => Some(&mut rule.rules),
            Self::Scope(rule) => Some(rule.rules.rules_mut()),
            Self::When(rule) => Some(rule.rules_mut()),
            Self::Else(rule) => Some(rule.rules_mut()),
            _ => None,
        }
    }
}
impl CssRule {
    /// Parses exactly one complete ordinary rule using supplied namespace bindings.
    ///
    /// The original source must contain one supported style rule or at-rule, with
    /// optional surrounding whitespace/comments. Multiple rules, trailing nontrivia
    /// and rejected outer grammar return `None` with `RejectInput`. Encoding directives
    /// are stylesheet metadata and cannot produce a rule; no BOM or CDO/CDC is stripped.
    /// Imports and namespaces use isolated top-level grammar, without insertion-order
    /// validation against an existing sheet. The supplied namespace context is immutable.
    ///
    /// A retained rule preserves its inner declaration, selector, query and child-rule
    /// recovery diagnostics. Implicit EOF closures are reported only when the outer
    /// rule survives; resource diagnostics preserve `StopAtNestingLimit`. Positions
    /// refer to the original UTF-8 source and zero-based UTF-16 coordinates. Nested
    /// contents remain inside their owning rule. This performs no matching, cascade,
    /// CSSOM insertion or contextual resolution.
    ///
    /// The parser supplies the document mode.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        namespaces: &crate::selector::CssNamespaceContext,
    ) -> crate::CssParseReport<Option<CssRule>> {
        crate::parser::fragments::parse_rule_with_context(
            source.as_ref(),
            namespaces,
            parser.context(),
        )
    }
}
impl CssStyleBlock {
    /// Parses exactly one real-brace style block with supplied namespace bindings.
    ///
    /// The source contains the braces and optional surrounding whitespace/comments,
    /// without a selector. Empty and recovered-empty blocks are retained. A second
    /// block, missing opening brace or trailing nontrivia rejects the complete input
    /// with `RejectInput`; resource failures preserve `StopAtNestingLimit`.
    ///
    /// Inner declarations, nested rules and subsequent declaration runs use ordinary
    /// style-body grammar and preserve their recovery diagnostics. Relative child
    /// selectors and explicit anchors remain symbolic. No parent selector is invented.
    /// The block origin includes its actual braces or ends at implicit EOF, excludes
    /// surrounding trivia, and shares the original source snapshot with declarations.
    /// Implicit closure diagnostics are published only after the outer block survives.
    ///
    /// The parser supplies the document mode.
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        namespaces: &crate::selector::CssNamespaceContext,
    ) -> crate::CssParseReport<Option<CssStyleBlock>> {
        crate::parser::fragments::parse_style_block_with_context(
            source.as_ref(),
            namespaces,
            parser.context(),
        )
    }
}
