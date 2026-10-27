use crate::component::CssValueOrigin;
use crate::query::CssCondition;
use crate::query::container::container_features::*;
use crate::query::container::{CssScrollStateLeaf, CssStyleQueryLeaf};
use crate::query::media::media_features::*;
use crate::source::CssSourcePosition;

fn is_exact_css_identifier(value: &str) -> bool {
    let mut input = cssparser::ParserInput::new(value);
    let mut parser = cssparser::Parser::new(&mut input);
    parser
        .expect_ident_cloned()
        .ok()
        .is_some_and(|parsed| parser.expect_exhausted().is_ok() && parsed.as_ref() == value)
}

/// A container name: a decoded `<custom-ident>` that is not a CSS-wide keyword or one of `none`, `and`, `or`, `not` and `default`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CssContainerName {
    name: String,
}

impl CssContainerName {
    /// Creates a name from text that is exactly one identifier token whose decoded text equals `name`.
    ///
    /// Returns `None` when `name` is not such an identifier or is a reserved word (compared case-insensitively).
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        if is_valid_container_name(&name) {
            Some(Self::new(name))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        debug_assert!(is_valid_container_name(&name));
        Self { name }
    }

    /// Constructs a decoded container name, including names requiring CSS escapes.
    /// Empty, NUL-containing and reserved names are rejected; spelling is not reparsed.
    #[must_use]
    pub fn try_from_decoded(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        Self::from_decoded(name)
    }

    /// Admits the decoded value of one identifier token without tokenizing again.
    pub(crate) fn from_decoded(name: String) -> Option<Self> {
        (!name.is_empty() && !name.contains('\0') && !is_parser_reserved_container_name(&name))
            .then_some(Self { name })
    }

    /// Returns the decoded name text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

fn is_valid_container_name(name: &str) -> bool {
    is_exact_css_identifier(name) && !is_parser_reserved_container_name(name)
}

fn is_parser_reserved_container_name(name: &str) -> bool {
    matches!(
        name.to_ascii_lowercase().as_str(),
        "inherit"
            | "initial"
            | "unset"
            | "revert"
            | "revert-layer"
            | "none"
            | "and"
            | "or"
            | "not"
            | "default"
    )
}

/// A nonempty ordered list of independent authored container-query entries.
/// Commas select alternative containers; they are not Boolean query operators.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerPrelude {
    entries: Vec<CssContainerQueryEntry>,
    lexical: crate::query::supports::construction::SupportsLexical,
    origin: CssValueOrigin,
}
impl crate::source::CssRetainedSources for CssContainerPrelude {
    fn visit_retained_sources(
        &self,
        sources: &mut crate::source::CssSourceRetention,
    ) -> Result<(), crate::source::CssSourceRetentionError> {
        sources.visit(&self.origin)?;
        sources.visit(&self.lexical)
    }
}

impl CssContainerPrelude {
    pub(crate) fn new(
        entries: Vec<CssContainerQueryEntry>,
        lexical: crate::query::supports::construction::SupportsLexical,
    ) -> Self {
        debug_assert!(!entries.is_empty());
        let origin = lexical.first_origin().clone();
        Self {
            entries,
            lexical,
            origin,
        }
    }
    /// Returns the comma-separated entries in authored order; the list is never empty.
    #[must_use]
    pub fn entries(&self) -> &[CssContainerQueryEntry] {
        &self.entries
    }
    /// Includes names, commas, comments and whitespace in their authored order.
    #[must_use]
    pub fn components(&self) -> &[crate::component::CssComponentValue] {
        self.lexical.items()
    }
    /// Returns the origin of the first component of the prelude.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    /// Returns the source position of the prelude, or `None` when its origin was not parsed from source.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.origin)
    }
}

/// One checked entry, with a name, a query, or both, retaining its lexical region.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerQueryEntry {
    kind: CssContainerQueryEntryKind,
    lexical: crate::query::supports::construction::SupportsLexical,
    origin: CssValueOrigin,
}
#[derive(Clone, Debug, PartialEq)]
enum CssContainerQueryEntryKind {
    NameOnly(CssContainerName),
    Query {
        name: Option<CssContainerName>,
        query: CssCondition<CssContainerLeaf>,
    },
}
impl CssContainerQueryEntry {
    pub(crate) fn name_only(
        name: CssContainerName,
        lexical: crate::query::supports::construction::SupportsLexical,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: CssContainerQueryEntryKind::NameOnly(name),
            lexical,
            origin,
        }
    }
    pub(crate) fn with_query(
        name: Option<CssContainerName>,
        query: CssCondition<CssContainerLeaf>,
        lexical: crate::query::supports::construction::SupportsLexical,
    ) -> Self {
        let origin = lexical.first_origin().clone();
        Self {
            kind: CssContainerQueryEntryKind::Query { name, query },
            lexical,
            origin,
        }
    }
    /// Returns the container name, or `None` when the entry is a query without a name.
    #[must_use]
    pub const fn name(&self) -> Option<&CssContainerName> {
        match &self.kind {
            CssContainerQueryEntryKind::NameOnly(name) => Some(name),
            CssContainerQueryEntryKind::Query { name, .. } => name.as_ref(),
        }
    }
    /// Returns the container condition, or `None` when the entry is only a name.
    #[must_use]
    pub const fn query(&self) -> Option<&CssCondition<CssContainerLeaf>> {
        match &self.kind {
            CssContainerQueryEntryKind::NameOnly(_) => None,
            CssContainerQueryEntryKind::Query { query, .. } => Some(query),
        }
    }
    /// Excludes the separating comma and other entries; includes entry trivia.
    #[must_use]
    pub fn components(&self) -> &[crate::component::CssComponentValue] {
        self.lexical.items()
    }
    /// Returns the origin of the first component of the entry.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    /// Returns the source position of the entry, or `None` when its origin was not parsed from source.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.origin)
    }
}

/// A leaf of a container condition: a size feature, a `style()` or `scroll-state()`
/// query, or any other enclosure.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssContainerLeaf {
    /// A `<general-enclosed>` production that is not a recognized container feature, `style()` or `scroll-state()`.
    GeneralEnclosed(CssContainerGeneralEnclosed),
    /// A size feature test such as `(width > 400px)` or `(orientation: portrait)`.
    Feature(CssContainerFeatureQuery),
    /// A `style()` query over style features.
    Style(CssCondition<CssStyleQueryLeaf>),
    /// A `scroll-state()` query over scroll-state features.
    ScrollState(CssCondition<CssScrollStateLeaf>),
}

/// A container operand classified as general-enclosed by the container grammar.
/// Its private lexical region prevents bypassing recognized feature/style admission.
///
/// ```compile_fail,E0616
/// use surgeist_css::query::container::CssContainerGeneralEnclosed;
/// fn replace_lexical(mut value: CssContainerGeneralEnclosed, other: CssContainerGeneralEnclosed) {
///     value.lexical = other.lexical;
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssContainerGeneralEnclosed {
    lexical: crate::query::supports::construction::SupportsLexical,
}
impl Eq for CssContainerGeneralEnclosed {}

impl CssContainerGeneralEnclosed {
    pub(crate) fn new(lexical: crate::query::supports::construction::SupportsLexical) -> Self {
        Self { lexical }
    }
    /// Borrows the single admitted enclosure from the shared lexical root.
    #[must_use]
    pub fn component(&self) -> &crate::component::CssComponentValue {
        self.lexical
            .items()
            .iter()
            .find(|value| !crate::query::supports::construction::trivia(value))
            .expect("one admitted enclosure")
    }
    /// Returns the origin of the enclosure component.
    #[must_use]
    pub fn origin(&self) -> &CssValueOrigin {
        self.component().origin()
    }
    /// Returns the source position of the enclosure, or `None` when its origin was not parsed from source.
    #[must_use]
    pub fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
    /// Returns the enclosure's original source text, or `None` when it was not parsed from source.
    #[must_use]
    pub fn authored(&self) -> Option<&str> {
        let origin = self.component().parsed_origin()?;
        Some(
            &origin.source().as_str()[origin.span().start().byte_offset().value()
                ..origin.span().end().byte_offset().value()],
        )
    }
    /// Serializes the original enclosure, including EOF-implied delimiters.
    ///
    /// `limits.max_css_bytes()` caps the output, including inserted syntax; the
    /// other component limits do not apply.
    ///
    /// # Errors
    ///
    /// Returns [`crate::component::CssComponentValueError`] when the byte limit is exhausted.
    pub fn serialize_tokens(
        &self,
        limits: crate::component::CssComponentValueLimits,
    ) -> Result<crate::component::CssSerializedValue, crate::component::CssComponentValueError>
    {
        let mut out =
            crate::component::values::serialize::CssCanonicalBuilder::new(limits.max_css_bytes());
        out.push_components(std::slice::from_ref(self.component()))?;
        out.finish()
    }
}

/// A container query or prelude that failed checked construction through its grammar
/// or a resource.
///
/// The failure is read through the `is_*` predicates and located by [`Self::origin`]; a
/// rejected component is exposed by [`Self::component_error`] and is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainerConstructionError {
    kind: ContainerConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ContainerConstructionKind {
    InvalidConditionGrammar { origin: CssValueOrigin },
    InvalidPreludeGrammar { origin: CssValueOrigin },
    Component(crate::component::CssComponentValueError),
    WorkerUnavailable { origin: CssValueOrigin },
}
impl From<ContainerConstructionKind> for CssContainerConstructionError {
    fn from(kind: ContainerConstructionKind) -> Self {
        Self { kind }
    }
}
impl CssContainerConstructionError {
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether the components do not form a container condition.
        is_invalid_condition_grammar => ContainerConstructionKind::InvalidConditionGrammar { .. },
        /// Returns whether the components do not form a container prelude.
        is_invalid_prelude_grammar => ContainerConstructionKind::InvalidPreludeGrammar { .. },
        /// Returns whether the bounded deep-construction worker could not be started.
        is_worker_unavailable => ContainerConstructionKind::WorkerUnavailable { .. },
    }
    /// Returns the rejected component.
    #[must_use]
    pub const fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match &self.kind {
            ContainerConstructionKind::Component(error) => Some(error),
            _ => None,
        }
    }
    /// Returns the responsible original or programmatic origin.
    #[must_use]
    pub const fn origin(&self) -> &CssValueOrigin {
        match &self.kind {
            ContainerConstructionKind::InvalidConditionGrammar { origin }
            | ContainerConstructionKind::InvalidPreludeGrammar { origin }
            | ContainerConstructionKind::WorkerUnavailable { origin } => origin,
            ContainerConstructionKind::Component(error) => error.origin(),
        }
    }
}
impl std::fmt::Display for CssContainerConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid authored container construction: ")?;
        f.write_str(match &self.kind {
            ContainerConstructionKind::InvalidConditionGrammar { .. } => {
                "the components do not form a container condition"
            }
            ContainerConstructionKind::InvalidPreludeGrammar { .. } => {
                "the components do not form a container prelude"
            }
            ContainerConstructionKind::Component(_) => "a component is invalid",
            ContainerConstructionKind::WorkerUnavailable { .. } => {
                "the construction thread could not be started"
            }
        })
    }
}
impl std::error::Error for CssContainerConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<crate::component::CssComponentValueError> for CssContainerConstructionError {
    fn from(error: crate::component::CssComponentValueError) -> Self {
        Self {
            kind: ContainerConstructionKind::Component(error),
        }
    }
}

/// An authored container size feature test, without evaluation against a container.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssContainerFeatureQuery {
    /// A feature name alone, such as `(width)`.
    Boolean(CssContainerSizeFeatureKind),
    /// `width`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    Width(CssMediaRange<CssContainerLength>),
    /// `height`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    Height(CssMediaRange<CssContainerLength>),
    /// `inline-size`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    InlineSize(CssMediaRange<CssContainerLength>),
    /// `block-size`, as a plain, `min-`/`max-` prefixed, comparison or interval range.
    BlockSize(CssMediaRange<CssContainerLength>),
    /// `aspect-ratio`, as a plain, `min-`/`max-` prefixed, comparison or interval range of `<ratio>` values.
    AspectRatio(CssMediaRange<CssContainerRatio>),
    /// `orientation: <value>`, a discrete orientation keyword or pending operand.
    Orientation(CssContainerOrientation),
}
