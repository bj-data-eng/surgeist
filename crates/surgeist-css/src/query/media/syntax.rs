use crate::component::CssValueOrigin;
use crate::query::CssCondition;
use crate::query::CssGeneralEnclosed;
use crate::query::media::construction::*;
use crate::query::media::media_features::*;
use crate::source::CssSourcePosition;

/// A `<media-query-list>`: the ordered, comma-separated members of a media query, possibly empty.
///
/// Parsed lists retain their comma origins and parse source; constructed lists have neither.
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

    /// Returns the members in authored order.
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
    /// A bare `<media-condition>` query, such as `(width > 600px)`.
    Condition(CssCondition<CssMediaLeaf>),
    /// A query with a media type and an optional `and` condition, such as `only screen and (color)`.
    Typed(CssTypedMediaQuery),
    /// A parser-recovery replacement for a malformed member, which is guaranteed false.
    Never(CssNeverMediaQuery),
}

impl CssMediaQuery {
    /// Returns the first non-trivia position of this authored or recovered query member.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        crate::query::media::construction::parsed_position(self.origin())
    }
    /// Returns the origin of the query member.
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
    /// Returns the origin of the recovered member, which is always parsed from source.
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
    /// Returns the first non-trivia position of the recovered member, or the member end when it had no non-trivia token.
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

    /// Returns the `not` or `only` modifier, or `None` when the query has none.
    #[must_use]
    pub const fn modifier(&self) -> Option<CssMediaQueryModifier> {
        self.modifier
    }

    /// Returns the media type, which is [`CssMediaType::Unknown`] when the authored type is not a known one.
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

    /// Returns the condition that follows `and`, or `None` when the query is only a media type.
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
    /// Returns the origin of the query.
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

/// The optional prefix of a typed media query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaQueryModifier {
    /// `not`: negates the whole query.
    Not,
    /// `only`: hides the query from legacy user agents; it does not change the result.
    Only,
}

/// A media type of a typed media query.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMediaType {
    /// `all`: matches every device.
    All,
    /// `screen`: matches all devices that are not matched by `print`.
    Screen,
    /// `print`: matches printers and devices intended to reproduce a printed display, such as a browser showing a print preview.
    Print,
    /// `aural`: a deprecated media type that is defined not to match.
    Aural,
    /// `braille`: a deprecated media type that is defined not to match.
    Braille,
    /// `embossed`: a deprecated media type that is defined not to match.
    Embossed,
    /// `handheld`: a deprecated media type that is defined not to match.
    Handheld,
    /// `projection`: a deprecated media type that is defined not to match.
    Projection,
    /// `speech`: a deprecated media type that is defined not to match.
    Speech,
    /// `tty`: a deprecated media type that is defined not to match.
    Tty,
    /// `tv`: a deprecated media type that is defined not to match.
    Tv,
    /// A syntactically valid media type that is not a known one; an unknown media type is treated as not matching.
    Unknown,
}

impl CssMediaType {
    /// Returns the lowercase media-type name; [`CssMediaType::Unknown`] returns `unknown`, not the authored spelling.
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
    /// Returns the origin of the unknown media-type token.
    pub const fn origin(&self) -> &CssValueOrigin {
        &self.origin
    }
}

/// A leaf of a media condition: one `<media-in-parens>` operand that is not itself a
/// negation, junction or explicit grouping.
#[non_exhaustive]
#[derive(Clone, Debug, PartialEq)]
pub enum CssMediaLeaf {
    /// A reference to a custom media query, such as `(--narrow)`.
    CustomMediaReference(crate::query::media::CssCustomMediaReference),
    /// A recognized media feature test, such as `(width > 600px)`.
    Feature(CssMediaFeatureQuery),
    /// A structurally valid feature whose name, value or operation has unknown truth.
    UnknownFeature(CssUnknownMediaFeature),
    /// A `<general-enclosed>` production that is not a media feature or custom media reference.
    GeneralEnclosed(CssGeneralEnclosed),
}

/// Why an unknown media type is defined to be nonmatching.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssDefinedFalseMediaReason {
    /// The media type is not a known one.
    UnknownType,
}

/// The `orientation` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOrientation {
    /// `portrait`: the value of the `height` feature is greater than or equal to the value of the `width` feature.
    Portrait,
    /// `landscape`: otherwise, so the value of the `width` feature is greater than the value of the `height` feature.
    Landscape,
}

/// The authored scanning mode of the `scan` media feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScanMode {
    /// `progressive`: the screen displays each frame in full.
    Progressive,
    /// `interlace`: the screen alternates between the even and odd lines of successive frames.
    Interlace,
}

/// The authored binary state of the `grid` media feature.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssGridMode {
    /// `0`: the output device is bitmap-based.
    Bitmap,
    /// `1`: the output device is grid-based, such as a `tty` terminal.
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

/// The `prefers-color-scheme` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSchemePreference {
    /// `light`: the user prefers a light theme or has expressed no preference.
    Light,
    /// `dark`: the user prefers a dark theme.
    Dark,
}

/// The `prefers-reduced-motion` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedMotionPreference {
    /// `reduce`: the user prefers an interface that removes or replaces motion-based animation.
    Reduce,
    /// `no-preference`: the user has made no preference known.
    NoPreference,
}

/// The `prefers-reduced-transparency` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssReducedTransparencyPreference {
    /// `reduce`: the user prefers an interface with fewer transparent or translucent effects.
    Reduce,
    /// `no-preference`: the user has made no preference known.
    NoPreference,
}

/// The `prefers-contrast` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssContrastPreference {
    /// `no-preference`: the user has made no preference known.
    NoPreference,
    /// `more`: the user prefers an interface with higher contrast.
    More,
    /// `less`: the user prefers an interface with lower contrast.
    Less,
    /// `custom`: the user wants a specific set of colors whose contrast matches neither `more` nor `less`.
    Custom,
}

/// The `forced-colors` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssForcedColorsMode {
    /// `none`: forced colors mode is not active.
    None,
    /// `active`: forced colors mode is active and the user agent enforces a limited color palette.
    Active,
}

/// The `hover` and `any-hover` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHoverCapability {
    /// `none`: the pointing device cannot hover, or there is none.
    None,
    /// `hover`: the pointing device can easily hover over parts of the page.
    Hover,
}

/// The `pointer` and `any-pointer` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPointerCapability {
    /// `none`: there is no pointing device.
    None,
    /// `coarse`: the pointing device has limited accuracy, such as a touchscreen.
    Coarse,
    /// `fine`: the pointing device is accurate, such as a mouse or stylus.
    Fine,
}

/// The `display-mode` media feature value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssDisplayMode {
    /// `fullscreen`: the context is displayed with browser interface elements hidden, using the entire available display area.
    Fullscreen,
    /// `standalone`: the standalone display mode is in use; only applicable in an application context.
    Standalone,
    /// `minimal-ui`: the minimal-ui display mode is in use; only applicable in an application context.
    MinimalUi,
    /// `browser`: the context is displayed using the platform-specific convention for opening hyperlinks in the user agent, such as a browser tab or window with controls.
    Browser,
    /// `picture-in-picture`: the context is displayed in a floating, always-on-top window.
    PictureInPicture,
}
