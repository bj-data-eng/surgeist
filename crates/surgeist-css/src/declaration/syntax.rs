use crate::component::CssComponentValues;
use crate::property::CssKnownDeclaration;
use crate::property::{CssCustomPropertyMetadata, CssPropertyNameRef};
use crate::source::CssParsedOrigin;
use crate::source::CssSourcePosition;
use std::sync::Arc;

/// An ordered collection of parsed or checked ordinary and Page authored declarations.
///
/// Members retain their immutable occurrences, importance, component origins and
/// individual parser contexts. Checked assembly requires complete specified output
/// under its operation's limits; parser production retains its own recovery policy
/// and does not guarantee that later serialization succeeds. The collection is
/// read-only and performs no cascade, substitution or contextual resolution.
///
/// ```compile_fail
/// use surgeist_css::declaration::CssDeclarationList;
/// let _ = CssDeclarationList { declarations: Vec::new() };
/// ```
///
/// # Specified serialization
///
/// Emits every authored occurrence as canonical specified attribute text. Empty lists emit an empty
/// string. Each declaration has its canonical grammar name, `: `, owning specified value, optional
/// ` !important` and `;`; adjacent occurrences have one space. Duplicates, repeated handles and
/// priority remain in authored order. Custom names retain case and symbolic token values keep their
/// retained spelling, comments and internal trivia under the existing value writer. No braces,
/// enclosing rule, cascade selection or substitution is emitted. For terminal winner output, use
/// [`Self::serialize_cssom`] instead.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDeclarationList {
    declarations: Vec<CssDeclaration>,
}

impl CssDeclarationList {
    pub(crate) const fn new(declarations: Vec<CssDeclaration>) -> Self {
        Self { declarations }
    }

    /// Assembles ordered declarations after complete specified-output admission.
    ///
    /// Uses the existing default serialization limits. Empty membership is valid;
    /// duplicates, repeated handles and symbolic values remain ordered occurrences.
    /// The supplied vector is consumed on success or failure. Retain independent
    /// declaration clones before calling if a failed construction must be retried.
    /// No grammar is reparsed and no aggregate source or parser context is invented.
    ///
    /// Admits the complete list through its actual occurrence writer under limits.
    ///
    /// One fresh cumulative input, projection and UTF-8 byte budget covers the
    /// aggregate, every occurrence, semantic provider and output punctuation.
    /// Repeated handles are charged again. The full bounded temporary output is
    /// discarded after admission; neither text nor limits are stored in the list.
    /// A later output request applies its own policy and can therefore fail.
    ///
    /// Failure returns the existing provider/resource error and no partial list.
    /// The moved vector is consumed even on failure; independently retained clones
    /// remain usable with the same occurrences, components, origins and contexts.
    pub fn try_new(
        declarations: Vec<CssDeclaration>,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<Self, crate::serialization::CssSerializationError> {
        let list = Self::new(declarations);
        list.serialize_specified(limits)?;
        Ok(list)
    }

    /// Returns the declarations in authored order.
    #[must_use]
    pub fn as_slice(&self) -> &[CssDeclaration] {
        &self.declarations
    }

    /// Iterates over declarations in authored order.
    pub fn iter(&self) -> std::slice::Iter<'_, CssDeclaration> {
        self.declarations.iter()
    }

    /// Returns the number of declarations.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Returns whether no declarations were retained.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Normalizes terminal winners and emits their CSSOM specified declaration block.
    /// Authored occurrences and the compact occurrence writer remain independent.
    ///
    /// Builds and emits atomically under one cumulative input, projection and byte budget.
    pub fn serialize_cssom(
        &self,
        limits: crate::serialization::CssSerializationLimits,
    ) -> Result<String, crate::declaration::CssDeclarationBlockError> {
        let mut writer =
            crate::serialization::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        writer.append_cssom_declaration_list(self)?;
        Ok(writer.css)
    }
}

impl std::ops::Deref for CssDeclarationList {
    type Target = [CssDeclaration];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

/// The complete importance state of an ordinary authored declaration.
///
/// Importance is recognized at a parsed declaration boundary or supplied separately to
/// [`crate::CssParser::parse_property_value`]; it is not part of the property value. CSS Cascade defines exactly
/// two importance states, normal and `!important`, so this enum is exhaustive on purpose,
/// unlike the other value enums: a cascade consumer matches both states to order precedence
/// and needs no wildcard arm.
/// Downstream cascade policy may consume it, but this crate does not apply cascade.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum CssImportance {
    /// Normal importance; parsed declarations have no terminal importance annotation.
    #[default]
    Normal,
    /// Important precedence; parsed declarations have one valid terminal `!important` annotation.
    Important,
}

/// A checked declaration occurrence in the authored CSS syntax phase.
///
/// Its private fields couple a known property to its schema-selected value type, or a custom name
/// to its authored custom value. Parsed declarations retain original name and value origins;
/// [`CssDeclaration::try_from_components`] constructs declarations from checked components without
/// manufacturing a property-name position. Callers cannot create a property/value mismatch.
///
/// ```compile_fail,E0308
/// use surgeist_css::property::{CssKnownPropertyValueRef, CssPropertyValue};
/// use surgeist_css::values::box_model::CssSizeValue;
/// use surgeist_css::CssParser;
///
/// let width_report = CssParser::default().parse_block_contents("width: 1px");
/// let opacity_report = CssParser::default().parse_block_contents("opacity: .5");
/// let Some(CssKnownPropertyValueRef::Width(width)) =
///     width_report.syntax()[0].known().unwrap().property_value()
/// else { panic!("expected width") };
/// let Some(CssKnownPropertyValueRef::Opacity(opacity)) =
///     opacity_report.syntax()[0].known().unwrap().property_value()
/// else { panic!("expected opacity") };
/// fn require_width(_: &CssPropertyValue<CssSizeValue>) {}
/// require_width(width);
/// require_width(opacity);
/// ```
///
/// ```compile_fail
/// use surgeist_css::declaration::CssDeclaration;
/// let _ = CssDeclaration { body: todo!(), importance: todo!(), position: todo!() };
/// ```
///
/// This authored node records importance but does not apply cascade, substitution, or contextual
/// resolution.
/// Clones refer to the same immutable occurrence. Structural equality compares the body,
/// importance, semantic declaration context, and optional name position;
/// it does not compare token provenance or occurrence
/// identity. Use [`Self::same_occurrence`] when that identity matters.
///
/// # Specified serialization
///
/// Serializes one authored declaration using its semantic value provider. Emits a canonical grammar
/// name, `: `, the specified value, an optional ` !important`, and `;`. Custom names retain case
/// and are escaped as identifiers. Custom and substitution-dependent token text remains symbolic,
/// preserving its retained spelling, comments and internal whitespace. Only top-level edge
/// whitespace is omitted, as in CSS Syntax declaration consumption; the complete original component
/// region remains retained. This does not select declarations, substitute variables or apply
/// cascade.
#[derive(Clone, Debug)]
pub struct CssDeclaration {
    occurrence: Arc<DeclarationOccurrence>,
}

#[derive(Debug)]
struct DeclarationOccurrence {
    payload: Arc<DeclarationPayload>,
    importance: CssImportance,
}

#[derive(Clone, Debug)]
struct DeclarationPayload {
    parser_context: crate::CssParserContext,
    declaration_context: DeclarationContext,
    body: CssDeclarationBody,
    components: CssComponentValues,
    provenance: DeclarationProvenance,
}

/// The semantic declaration owner, independent of document parser mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DeclarationContext {
    Ordinary,
    Page,
}

#[derive(Clone, Debug)]
enum DeclarationProvenance {
    Parsed {
        name: CssParsedOrigin,
        value: CssParsedOrigin,
    },
    ParsedValue {
        value: CssParsedOrigin,
    },
    Constructed,
}

impl PartialEq for CssDeclaration {
    fn eq(&self, other: &Self) -> bool {
        self.parser_context() == other.parser_context()
            && self.declaration_context() == other.declaration_context()
            && self.body() == other.body()
            && self.importance() == other.importance()
            && self.position() == other.position()
    }
}

impl CssDeclaration {
    pub(crate) const fn constructed_allocation_bytes() -> usize {
        std::mem::size_of::<DeclarationOccurrence>() + std::mem::size_of::<DeclarationPayload>()
    }
    /// Borrows the independent SVG authored body; known/custom accessors return None for it.
    #[must_use]
    pub fn svg_glyph_orientation_vertical(
        &self,
    ) -> Option<&crate::property::CssSvgGlyphOrientationVerticalDeclaration> {
        match self.body() {
            CssDeclarationBody::SvgGlyphOrientationVertical(value) => Some(value),
            _ => None,
        }
    }
    #[must_use]
    pub(crate) fn new_parsed(
        parser_context: crate::CssParserContext,
        declaration_context: DeclarationContext,
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
        name: CssParsedOrigin,
        value: CssParsedOrigin,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                importance,
                payload: Arc::new(DeclarationPayload {
                    parser_context,
                    declaration_context,
                    body,
                    components,
                    provenance: DeclarationProvenance::Parsed { name, value },
                }),
            }),
        }
    }

    pub(crate) fn new_parsed_value(
        parser_context: crate::CssParserContext,
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
        value: CssParsedOrigin,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                importance,
                payload: Arc::new(DeclarationPayload {
                    parser_context,
                    declaration_context: DeclarationContext::Ordinary,
                    body,
                    components,
                    provenance: DeclarationProvenance::ParsedValue { value },
                }),
            }),
        }
    }

    pub(crate) fn new_constructed(
        parser_context: crate::CssParserContext,
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
    ) -> Self {
        Self::new_constructed_in_context(
            parser_context,
            DeclarationContext::Ordinary,
            body,
            importance,
            components,
        )
    }

    pub(crate) fn new_constructed_in_context(
        parser_context: crate::CssParserContext,
        declaration_context: DeclarationContext,
        body: CssDeclarationBody,
        importance: CssImportance,
        components: CssComponentValues,
    ) -> Self {
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                importance,
                payload: Arc::new(DeclarationPayload {
                    parser_context,
                    declaration_context,
                    body,
                    components,
                    provenance: DeclarationProvenance::Constructed,
                }),
            }),
        }
    }

    /// Returns the authored parsing context retained for strict substitution reentry.
    #[must_use]
    pub fn parser_context(&self) -> crate::CssParserContext {
        self.occurrence.payload.parser_context
    }

    /// Changes only the checked domain, retaining genuine original token provenance.
    pub(crate) fn into_page_context(self) -> Self {
        if self.declaration_context() == DeclarationContext::Page {
            return self;
        }
        let mut payload = (*self.occurrence.payload).clone();
        payload.declaration_context = DeclarationContext::Page;
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                payload: Arc::new(payload),
                importance: self.importance(),
            }),
        }
    }
    pub(crate) fn declaration_context(&self) -> DeclarationContext {
        self.occurrence.payload.declaration_context
    }

    /// Returns the property-coupled authored body.
    #[must_use]
    pub fn body(&self) -> &CssDeclarationBody {
        &self.occurrence.payload.body
    }

    /// Returns the known-property declaration, or `None` for custom and independent SVG bodies.
    #[must_use]
    pub fn known(&self) -> Option<&CssKnownDeclaration> {
        match self.body() {
            CssDeclarationBody::Known(known) => Some(known),
            CssDeclarationBody::Custom(_) => None,
            CssDeclarationBody::SvgGlyphOrientationVertical(_) => None,
        }
    }

    /// Returns the custom declaration, or `None` for known and independent SVG bodies.
    #[must_use]
    pub fn custom(&self) -> Option<&CssCustomDeclaration> {
        match self.body() {
            CssDeclarationBody::Known(_) => None,
            CssDeclarationBody::SvgGlyphOrientationVertical(_) => None,
            CssDeclarationBody::Custom(custom) => Some(custom),
        }
    }

    /// Returns a borrowed semantic property-name view derived from the active body.
    #[must_use]
    pub fn property_name(&self) -> CssPropertyNameRef<'_> {
        match self.body() {
            CssDeclarationBody::Known(known) => CssPropertyNameRef::Known(known.property()),
            CssDeclarationBody::Custom(custom) => CssPropertyNameRef::Custom(custom.name()),
            CssDeclarationBody::SvgGlyphOrientationVertical(_) => {
                CssPropertyNameRef::SvgGlyphOrientationVertical
            }
        }
    }

    /// Known names are ASCII case-insensitive; custom names preserve exact code points.
    #[must_use]
    pub fn is_name_case_sensitive(&self) -> bool {
        self.custom().is_some()
    }

    /// Returns an immutable occurrence with the selected priority.
    /// An unchanged priority shares occurrence identity; a changed priority shares
    /// the complete authored payload but has a distinct occurrence identity.
    #[must_use]
    pub fn with_importance(&self, importance: CssImportance) -> Self {
        if self.importance() == importance {
            return self.clone();
        }
        Self {
            occurrence: Arc::new(DeclarationOccurrence {
                payload: Arc::clone(&self.occurrence.payload),
                importance,
            }),
        }
    }

    /// Returns importance recognized from source or supplied by the caller.
    #[must_use]
    pub fn importance(&self) -> CssImportance {
        self.occurrence.importance
    }

    /// Returns the original property-name start, or `None` when the name was supplied.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        self.parsed_name().map(|origin| origin.span().start())
    }

    /// Returns the consumed property-name token's original source span.
    /// Raw value parsing and checked construction have no parsed property name.
    #[must_use]
    pub fn parsed_name(&self) -> Option<&CssParsedOrigin> {
        match &self.occurrence.payload.provenance {
            DeclarationProvenance::Parsed { name, .. } => Some(name),
            DeclarationProvenance::ParsedValue { .. } | DeclarationProvenance::Constructed => None,
        }
    }

    /// Returns the parsed value region, excluding its annotation and declaration delimiter.
    ///
    /// Raw value parsing retains its original value source without a fabricated name.
    /// Checked construction has no single declaration-source region, even when individual
    /// supplied tokens have parsed origins. Empty parsed custom values retain a zero-width span.
    #[must_use]
    pub fn parsed_value(&self) -> Option<&CssParsedOrigin> {
        match &self.occurrence.payload.provenance {
            DeclarationProvenance::Parsed { value, .. }
            | DeclarationProvenance::ParsedValue { value } => Some(value),
            DeclarationProvenance::Constructed => None,
        }
    }

    /// Returns the owned value components with their original or programmatic provenance.
    #[must_use]
    pub fn value_components(&self) -> &CssComponentValues {
        &self.occurrence.payload.components
    }

    /// Reports whether both handles refer to the same immutable declaration occurrence.
    #[must_use]
    pub fn same_occurrence(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.occurrence, &other.occurrence)
    }
}

/// The authored body of a declaration, with known, custom and independent SVG invariants.
///
/// Known values are coupled to their schema identity; custom values remain attached to their
/// case-sensitive custom name. SVG values retain their actual independent admission.
/// Parsing and [`crate::CssParser::parse_property_value`] construct these
/// checked bodies without performing cascade or substitution.
#[non_exhaustive]
#[expect(
    clippy::large_enum_variant,
    reason = "authored declaration variants retain their checked values inline"
)]
#[derive(Clone, Debug, PartialEq)]
pub enum CssDeclarationBody {
    /// The independent SVG authored definition, retaining CSS versus attribute admission.
    SvgGlyphOrientationVertical(crate::property::CssSvgGlyphOrientationVerticalDeclaration),
    /// A schema-recognized property carrying only its property-specific declared value.
    Known(CssKnownDeclaration),
    /// A case-sensitive custom property carrying its current authored value representation.
    Custom(CssCustomDeclaration),
}

/// A checked authored custom-property declaration.
///
/// Its private fields prevent attaching the custom value to a known property name. Parsing and
/// [`crate::CssParser::parse_property_value`] preserve custom syntax without substituting references,
/// computing cascade, or validating a post-substitution value.
///
/// ```compile_fail
/// use surgeist_css::declaration::CssCustomDeclaration;
/// let _ = CssCustomDeclaration { name: todo!(), value: todo!() };
/// ```
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomDeclaration {
    name: CssCustomPropertyName,
    value: CssCustomPropertyDeclaredValue,
}

impl CssCustomDeclaration {
    pub(crate) const fn new(
        name: CssCustomPropertyName,
        value: CssCustomPropertyDeclaredValue,
    ) -> Self {
        Self { name, value }
    }

    /// Returns the case-sensitive validated custom property name.
    #[must_use]
    pub const fn name(&self) -> &CssCustomPropertyName {
        &self.name
    }

    /// Returns the preserved authored custom-property value.
    #[must_use]
    pub const fn value(&self) -> &CssCustomPropertyDeclaredValue {
        &self.value
    }
}

static CUSTOM_PROPERTY_METADATA: CssCustomPropertyMetadata = CssCustomPropertyMetadata {
    source: crate::conformance::catalog::O_VARIABLES1,
};

/// A case-sensitive semantic custom-property name.
///
/// [`Self::try_new`] accepts one complete authored CSS identifier token whose decoded name begins
/// with `--`, including non-ASCII characters and escapes. [`Self::as_str`] returns its decoded
/// semantic
/// identity, matching names produced by the stylesheet parser; it does not retain the name's
/// source escape spelling.
/// [`Self::try_from_decoded`] instead preserves an already semantic string exactly,
/// using the selected CSSOM prefix/nonempty check without tokenizing its spelling.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssCustomPropertyName {
    name: String,
}

impl CssCustomPropertyName {
    pub(crate) fn diagnostic_backing_bytes(&self) -> usize {
        self.name.capacity()
    }

    /// Tokenizes and validates one complete authored custom-property name.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let authored = name.into();
        let mut lexer = surgeist_css_parser::lexer::Lexer::new(
            &authored,
            &surgeist_css_parser::lexer::Options::default(),
        );
        let lexeme = lexer.next_lexeme()?;
        let surgeist_css_parser::Token::Ident(decoded) = lexeme.token() else {
            return None;
        };
        // The identifier is the whole text: no whitespace or comment follows it.
        if lexeme.range().end.value() != authored.len() {
            return None;
        }
        Self::from_ident_token(decoded)
    }

    /// Admits an already decoded or supplied semantic custom-property name.
    ///
    /// The string must start with `--` and have a nonempty suffix. Every supplied
    /// character, including whitespace, punctuation, backslashes and NUL, is
    /// preserved exactly. No escapes, case or Unicode sequences are normalized.
    /// This creates no parsed-source provenance. CSS serialization independently
    /// applies its identifier output rules, including NUL replacement.
    ///
    /// ```
    /// use surgeist_css::declaration::CssCustomPropertyName;
    /// let name = CssCustomPropertyName::try_from_decoded("--bad name").unwrap();
    /// assert_eq!(name.as_str(), "--bad name");
    /// assert!(CssCustomPropertyName::try_new("--bad name").is_none());
    /// assert!(CssCustomPropertyName::try_from_decoded("--").is_none());
    /// ```
    #[must_use]
    pub fn try_from_decoded(name: impl AsRef<str>) -> Option<Self> {
        Self::from_ident_token(name.as_ref())
    }

    #[must_use]
    pub(crate) fn from_ident_token(name: &str) -> Option<Self> {
        Self::accepts_ident_token(name).then(|| Self {
            name: name.to_owned(),
        })
    }

    pub(crate) fn accepts_ident_token(name: &str) -> bool {
        name.strip_prefix("--")
            .is_some_and(|suffix| !suffix.is_empty())
    }

    /// Returns the decoded, case-sensitive semantic custom-property name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }

    /// Returns the shared intrinsic definition for every validated custom name.
    #[must_use]
    pub fn metadata(&self) -> &'static CssCustomPropertyMetadata {
        &CUSTOM_PROPERTY_METADATA
    }
}

/// A non-blank authored declaration value, retained as the exact CSS text before any
/// substitution or grammar interpretation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredDeclarationValue {
    css: String,
}

impl CssAuthoredDeclarationValue {
    pub(crate) fn retained_backing_bytes(&self) -> usize {
        self.css.capacity()
    }
    /// Returns `None` when `css` is empty or only whitespace.
    #[must_use]
    pub fn try_new(css: impl Into<String>) -> Option<Self> {
        let css = css.into();
        if css.trim().is_empty() {
            None
        } else {
            Some(Self::new(css))
        }
    }

    #[must_use]
    pub(crate) fn new(css: impl Into<String>) -> Self {
        Self { css: css.into() }
    }

    /// Returns the retained authored text exactly as written.
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }
}

/// An exact retained custom-property token stream in the authored CSS syntax phase.
///
/// The value can be empty and preserves interior UTF-8 source spelling after parser-owned boundary
/// trivia removal. It does not substitute variables, expose dependency tokens, compute cascade, or
/// validate a computed value.
///
/// ```compile_fail
/// use surgeist_css::declaration::CssCustomPropertyValue;
/// fn dependency_tokens(value: &CssCustomPropertyValue) {
///     let _ = value.references();
/// }
/// ```
///
/// ```compile_fail
/// use surgeist_css::declaration::CssCustomPropertyValue;
/// let _ = CssCustomPropertyValue { authored: todo!() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomPropertyValue {
    authored: CssAuthoredDeclarationValue,
}

impl CssCustomPropertyValue {
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredDeclarationValue) -> Self {
        Self { authored }
    }

    /// Returns the exact retained UTF-8 source after parser-owned boundary trivia removal.
    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }

    /// Returns whether the retained authored token stream is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.authored.as_css().is_empty()
    }
}

/// A custom property's declared value in the authored CSS syntax phase.
///
/// The branches preserve the strict parser's existing distinction between authored token text
/// and a whole-value CSS-wide keyword. This value remains attached to a validated custom name and
/// does not perform substitution, cascade, or post-substitution validation.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CssCustomPropertyDeclaredValue {
    /// Preserved authored custom-property token text.
    Value(CssCustomPropertyValue),
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
}

impl CssCustomPropertyDeclaredValue {
    /// Returns the preserved authored custom-property value when present.
    #[must_use]
    pub const fn value(&self) -> Option<&CssCustomPropertyValue> {
        match self {
            Self::Value(value) => Some(value),
            Self::Global(_) => None,
        }
    }

    /// Returns the symbolic CSS-wide keyword when present.
    #[must_use]
    pub const fn global(&self) -> Option<CssGlobalKeyword> {
        match self {
            Self::Value(_) => None,
            Self::Global(keyword) => Some(*keyword),
        }
    }
}

/// A known property's authored value whose grammar depends on later CSS substitution.
///
/// The complete authored value remains symbolic. This value exposes only its retained CSS text: it
/// does not promise post-substitution grammar validity, resolve variables, or expose/build a
/// dependency graph.
///
/// ```compile_fail
/// use surgeist_css::declaration::CssSubstitutionDependentValue;
/// fn dependency_graph(value: &CssSubstitutionDependentValue) {
///     let _ = value.references();
/// }
/// ```
///
/// ```compile_fail
/// use surgeist_css::declaration::CssSubstitutionDependentValue;
/// let _ = CssSubstitutionDependentValue { authored: todo!() };
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssSubstitutionDependentValue {
    authored: CssAuthoredDeclarationValue,
}

impl CssSubstitutionDependentValue {
    pub(crate) fn backing_capacity(&self) -> usize {
        self.authored.retained_backing_bytes()
    }
    #[must_use]
    pub(crate) const fn new(authored: CssAuthoredDeclarationValue) -> Self {
        Self { authored }
    }

    /// Returns the complete retained authored declaration value.
    #[must_use]
    pub fn as_css(&self) -> &str {
        self.authored.as_css()
    }
}

/// A CSS-wide keyword retained in the authored declaration phase.
///
/// It remains symbolic and does not apply inheritance, cascade, or revert behavior.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CssGlobalKeyword {
    /// The authored `inherit` keyword.
    Inherit,
    /// The authored `initial` keyword.
    Initial,
    /// The authored `unset` keyword.
    Unset,
    /// The authored `revert` keyword.
    Revert,
    /// The authored `revert-layer` keyword.
    RevertLayer,
}

/// A property-specific declared value in the authored CSS syntax phase.
///
/// `Value` contains only the type selected by the active known-declaration variant. Global and
/// substitution-dependent syntax remain symbolic. These views do not apply cascade, perform
/// substitution, or contextually resolve the value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssDeclaredValue<T> {
    /// A value accepted by the property's typed authored grammar.
    Value(T),
    /// A whole-value CSS-wide keyword.
    Global(CssGlobalKeyword),
    /// Authored syntax whose grammar depends on later substitution.
    SubstitutionDependent(CssSubstitutionDependentValue),
}

/// The authored text of a functional notation's arguments, serialized from the retained
/// tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssAuthoredFunctionArguments {
    css: String,
}

impl CssAuthoredFunctionArguments {
    #[must_use]
    pub(crate) fn new(css: impl Into<String>) -> Self {
        Self { css: css.into() }
    }

    /// Returns the argument text.
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.css
    }
}
impl CssDeclaration {
    /// Parses exactly one complete, grammar-valid ordinary declaration from raw source.
    ///
    /// Surrounding whitespace and comments are accepted. The source must contain a
    /// recognized property or a valid custom property, its colon, and its value;
    /// top-level semicolons and stray closing delimiters reject the entire input.
    /// Use `CssParser::parse_block_contents` for a declaration list with separators.
    /// Name and value origins share the original source snapshot, and importance is
    /// recognized by the ordinary declaration grammar. No annotation span is added.
    /// Rejection returns `None` with `RejectInput`, except resource exhaustion retains
    /// `StopAtNestingLimit`. Implicit EOF closures are reported only after the complete
    /// declaration survives. This validates property grammar beyond CSS Syntax's
    /// generic consume-declaration algorithm and performs no contextual resolution.
    ///
    /// The parser supplies the document mode.
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
    ) -> crate::CssParseReport<Option<CssDeclaration>> {
        crate::parser::fragments::parse_declaration_with_context(source.as_ref(), parser.context())
    }
}
