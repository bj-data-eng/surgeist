use crate::component::CssValueOrigin;
use crate::query::CssCondition;
use crate::query::CssGeneralEnclosed;
use crate::query::media::construction::*;
use crate::query::media::media_features::*;
use crate::source::CssSourcePosition;

#[derive(Clone, Debug)]
pub struct CssMediaQueryList {
    parse_source: Option<crate::source::CssParseSource>,
    queries: Vec<CssMediaQuery>,
    pub(crate) comma_origins: Vec<CssValueOrigin>,
}

impl PartialEq for CssMediaQueryList {
    fn eq(&self, other: &Self) -> bool {
        self.queries == other.queries && self.comma_origins == other.comma_origins
    }
}

impl CssMediaQueryList {
    /// Complete actual parse input/context, absent for constructed lists.
    #[must_use]
    pub const fn parse_source(&self) -> Option<&crate::source::CssParseSource> {
        self.parse_source.as_ref()
    }
    pub(crate) fn bind_parse_source(&mut self, source: crate::source::CssParseSource) {
        self.parse_source = Some(source);
    }

    /// Constructs a list, including the valid empty media-query list.
    ///
    /// Every supplied checked member is retained in order.
    #[must_use]
    pub fn new(queries: Vec<CssMediaQuery>) -> Self {
        Self {
            parse_source: None,
            queries,
            comma_origins: Vec::new(),
        }
    }

    pub(crate) fn with_comma_origins(
        queries: Vec<CssMediaQuery>,
        comma_origins: Vec<CssValueOrigin>,
    ) -> Self {
        Self {
            parse_source: None,
            queries,
            comma_origins,
        }
    }

    #[must_use]
    pub fn queries(&self) -> &[CssMediaQuery] {
        &self.queries
    }

    pub(crate) fn into_single_query(self) -> Option<CssMediaQuery> {
        if self.queries.len() == 1 {
            self.queries.into_iter().next()
        } else {
            None
        }
    }
}

/// One authored or parser-recovered media-query-list member.
///
/// The `Never` branch is parser-owned recovery syntax for a malformed authored member. It is not
/// publicly constructible and is the only branch for which [`Self::is_guaranteed_false`] is true.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssMediaQuery {
    Condition(CssCondition<CssMediaLeaf>),
    Typed(CssTypedMediaQuery),
    Never(CssNeverMediaQuery),
}

impl CssMediaQuery {
    /// Returns the first non-trivia position of this authored or recovered query member.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        match self {
            Self::Condition(v) => v.origin(),
            Self::Typed(v) => v.origin(),
            Self::Never(v) => v.origin(),
        }
    }

    /// Returns whether this member is the parser-owned guaranteed-false recovery sentinel.
    #[must_use]
    pub const fn is_guaranteed_false(&self) -> bool {
        matches!(self, Self::Never(_))
    }
}

/// A parser-owned guaranteed-false replacement for one malformed media-query-list member.
///
/// Its position is the member's first non-trivia position, or the member end when it contained no
/// non-trivia token. The complete malformed source unit remains on the paired recovery diagnostic.
/// Callers cannot construct this recovered state.
///
/// ```compile_fail
/// use surgeist_css::query::media::{CssMediaQuery, CssNeverMediaQuery};
/// use surgeist_css::source::CssSourcePosition;
///
/// fn forge(position: CssSourcePosition) -> CssMediaQuery {
///     CssMediaQuery::Never(CssNeverMediaQuery { position })
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNeverMediaQuery {
    origin: CssValueOrigin,
}
impl CssNeverMediaQuery {
    pub(crate) fn new(origin: crate::source::CssParsedOrigin) -> Self {
        Self {
            origin: CssValueOrigin::Parsed(origin),
        }
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    pub const fn position(&self) -> CssSourcePosition {
        match &self.origin {
            CssValueOrigin::Parsed(v) => v.span().start(),
            _ => panic!("parser-only recovery origin"),
        }
    }
}

/// A checked authored typed media query with parsed or programmatic provenance.
///
/// Callers can inspect authored semantics but cannot construct or forge parser provenance.
///
/// ```compile_fail
/// use surgeist_css::query::media::{CssMediaQueryModifier, CssMediaType, CssTypedMediaQuery};
/// use surgeist_css::source::CssSourcePosition;
///
/// fn forge(position: CssSourcePosition) -> CssTypedMediaQuery {
///     CssTypedMediaQuery {
///         modifier: Some(CssMediaQueryModifier::Only),
///         media_type: CssMediaType::Screen,
///         condition: None,
///         position,
///     }
/// }
/// ```
#[derive(Clone, Debug, PartialEq)]
pub struct CssTypedMediaQuery {
    modifier: Option<CssMediaQueryModifier>,
    media_type: CssMediaType,
    unknown_media_type: Option<CssUnknownMediaType>,
    condition: Option<Box<CssCondition<CssMediaLeaf>>>,
    origin: CssValueOrigin,
    pub(crate) syntax: Box<MediaTypedSyntax>,
}

impl CssTypedMediaQuery {
    #[must_use]
    pub(crate) fn new(
        modifier: Option<CssMediaQueryModifier>,
        media_type: CssMediaType,
        condition: Option<CssCondition<CssMediaLeaf>>,
        origin: CssValueOrigin,
        syntax: MediaTypedSyntax,
    ) -> Self {
        debug_assert_ne!(media_type, CssMediaType::Unknown);
        Self {
            modifier,
            media_type,
            unknown_media_type: None,
            condition: condition.map(Box::new),
            origin,
            syntax: Box::new(syntax),
        }
    }

    #[must_use]
    pub(crate) fn new_unknown(
        modifier: Option<CssMediaQueryModifier>,
        media_type: CssUnknownMediaType,
        condition: Option<CssCondition<CssMediaLeaf>>,
        origin: CssValueOrigin,
        syntax: MediaTypedSyntax,
    ) -> Self {
        Self {
            modifier,
            media_type: CssMediaType::Unknown,
            unknown_media_type: Some(media_type),
            condition: condition.map(Box::new),
            origin,
            syntax: Box::new(syntax),
        }
    }

    #[must_use]
    pub const fn modifier(&self) -> Option<CssMediaQueryModifier> {
        self.modifier
    }

    #[must_use]
    pub const fn media_type(&self) -> CssMediaType {
        self.media_type
    }

    /// Returns the exact authored spelling and token position for an unknown media type.
    ///
    /// This is present exactly when [`Self::media_type`] returns [`CssMediaType::Unknown`].
    #[must_use]
    pub const fn unknown_media_type(&self) -> Option<&CssUnknownMediaType> {
        self.unknown_media_type.as_ref()
    }

    #[must_use]
    pub const fn condition(&self) -> Option<&CssCondition<CssMediaLeaf>> {
        match &self.condition {
            Some(condition) => Some(condition),
            None => None,
        }
    }

    /// Returns the first non-trivia position of the authored typed media query.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.origin)
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaQueryModifier {
    Not,
    Only,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaType {
    All,
    Screen,
    Print,
    Aural,
    Braille,
    Embossed,
    Handheld,
    Projection,
    Speech,
    Tty,
    Tv,
    /// A syntactically valid unknown MQ3 media type, which is defined to be false.
    Unknown,
}

impl CssMediaType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Screen => "screen",
            Self::Print => "print",
            Self::Aural => "aural",
            Self::Braille => "braille",
            Self::Embossed => "embossed",
            Self::Handheld => "handheld",
            Self::Projection => "projection",
            Self::Speech => "speech",
            Self::Tty => "tty",
            Self::Tv => "tv",
            Self::Unknown => "unknown",
        }
    }
    /// Whether this type is defined not to match in the selected MQ5 profile.
    pub const fn is_defined_nonmatching(self) -> bool {
        !matches!(self, Self::All | Self::Screen | Self::Print)
    }
}

/// A checked unknown media type with authored spelling and parsed or programmatic provenance.
///
/// Unknown types are valid authored syntax with defined-false semantics. Private fields prevent
/// callers from forging parser provenance or pairing this model with a known media type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssUnknownMediaType {
    spelling: String,
    origin: CssValueOrigin,
}

impl CssUnknownMediaType {
    #[must_use]
    pub(crate) fn new(spelling: impl Into<String>, origin: CssValueOrigin) -> Self {
        let spelling = spelling.into();
        debug_assert!(!spelling.is_empty());
        Self { spelling, origin }
    }

    /// Returns the exact authored media-type token spelling, including escapes and casing.
    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.spelling
    }

    /// Returns the defined-false classification for this unknown media type.
    #[must_use]
    pub const fn reason(&self) -> CssDefinedFalseMediaReason {
        CssDefinedFalseMediaReason::UnknownType
    }

    /// Returns the position of the authored unknown media-type token.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(&self.origin)
    }
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

/// A leaf of a media condition: one `<media-in-parens>` operand that is not itself a
/// negation, junction or explicit grouping.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssMediaLeaf {
    CustomMediaReference(crate::query::media::CssCustomMediaReference),
    Feature(CssMediaFeatureQuery),
    UnknownFeature(CssUnknownMediaFeature),
    GeneralEnclosed(CssGeneralEnclosed),
}

/// Why an unknown media type is defined to be nonmatching.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssDefinedFalseMediaReason {
    UnknownType,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOrientation {
    Portrait,
    Landscape,
}

/// The authored scanning mode of the Media Queries Level 3 `scan` feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScanMode {
    Progressive,
    Interlace,
}

/// The authored binary state of the Media Queries Level 3 `grid` feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridMode {
    Bitmap,
    Grid,
}

impl CssGridMode {
    /// Returns the authored integer meaning of this grid mode.
    #[must_use]
    pub const fn value(self) -> u8 {
        match self {
            Self::Bitmap => 0,
            Self::Grid => 1,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSchemePreference {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedMotionPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedTransparencyPreference {
    Reduce,
    NoPreference,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContrastPreference {
    NoPreference,
    More,
    Less,
    Custom,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssForcedColorsMode {
    None,
    Active,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHoverCapability {
    None,
    Hover,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerCapability {
    None,
    Coarse,
    Fine,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDisplayMode {
    Fullscreen,
    Standalone,
    MinimalUi,
    Browser,
    PictureInPicture,
}
