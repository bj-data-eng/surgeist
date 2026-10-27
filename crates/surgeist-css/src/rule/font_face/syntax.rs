use crate::component::CssComponentValues;
use crate::rule::font_face::{CssFontMetricOverride, CssFontNamedInstance};
use crate::source::CssSourcePosition;
use crate::values::common::CssUrl;
use crate::values::font::{
    CssAuthoredFontFeatureSettings, CssFontFaceStyle, CssFontFaceWeight, CssFontFaceWidth,
    CssFontLanguageOverride, CssFontVariationSettings,
};

pub(crate) mod pending;

/// An authored `@font-face` rule: its descriptor occurrences in authored order.
///
/// The rule performs no font matching, loading or usability checks.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceRule {
    descriptors: CssFontFaceDescriptors,
    position: Option<CssSourcePosition>,
}

impl CssFontFaceRule {
    /// Assembles ordered checked descriptors without inventing parsed provenance.
    #[must_use]
    pub const fn new(descriptors: CssFontFaceDescriptors) -> Self {
        Self {
            descriptors,
            position: None,
        }
    }

    pub(crate) const fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the descriptor occurrences of the rule.
    #[must_use]
    pub const fn descriptors(&self) -> &CssFontFaceDescriptors {
        &self.descriptors
    }

    /// Returns the source position of the rule, or `None` when it was assembled programmatically.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// Selects an authored `@font-face` descriptor-value grammar.
///
/// This context has no source position: its name is not part of a raw value input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorKind {
    /// The `font-family` descriptor, which names the family the face belongs to.
    FontFamily,
    /// The `src` descriptor, which lists where the font data comes from.
    Src,
    /// The `font-weight` descriptor, which gives the weight or weight range the face covers.
    FontWeight,
    /// The `font-style` descriptor, which gives the style the face covers.
    FontStyle,
    /// The `font-width` descriptor, which gives the width or width range the face covers.
    FontWidth,
    /// The `font-display` descriptor, which chooses the loading display policy.
    FontDisplay,
    /// The `unicode-range` descriptor, which gives the code points the face may support, as a hint for deciding whether to download it.
    UnicodeRange,
    /// The `font-feature-settings` descriptor, which gives initial feature settings applied when the face is rendered.
    FontFeatureSettings,
    /// The `font-variation-settings` descriptor, which gives initial variation axis settings applied when the face is rendered.
    FontVariationSettings,
    /// The `font-named-instance` descriptor, which selects a named instance of a variable font.
    FontNamedInstance,
    /// The `font-language-override` descriptor, which gives an initial language override applied when the face is rendered.
    FontLanguageOverride,
    /// The `ascent-override` descriptor, which overrides the font's ascent metric.
    AscentOverride,
    /// The `descent-override` descriptor, which overrides the font's descent metric.
    DescentOverride,
    /// The `line-gap-override` descriptor, which overrides the font's line gap metric.
    LineGapOverride,
}

impl CssFontFaceDescriptorKind {
    /// Returns the canonical CSS descriptor name.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FontFamily => "font-family",
            Self::Src => "src",
            Self::FontWeight => "font-weight",
            Self::FontStyle => "font-style",
            Self::FontWidth => "font-width",
            Self::FontDisplay => "font-display",
            Self::UnicodeRange => "unicode-range",
            Self::FontFeatureSettings => "font-feature-settings",
            Self::FontVariationSettings => "font-variation-settings",
            Self::FontNamedInstance => "font-named-instance",
            Self::FontLanguageOverride => "font-language-override",
            Self::AscentOverride => "ascent-override",
            Self::DescentOverride => "descent-override",
            Self::LineGapOverride => "line-gap-override",
        }
    }

    /// Looks up an exact decoded descriptor name, ignoring ASCII case.
    ///
    /// `font-stretch` is the existing CSS alias for `font-width`. Unknown names,
    /// whitespace, escapes and punctuation return `None`; this does not tokenize
    /// CSS or manufacture a named descriptor occurrence.
    #[must_use]
    pub fn from_css_name(name: &str) -> Option<Self> {
        [
            Self::FontFamily,
            Self::Src,
            Self::FontWeight,
            Self::FontStyle,
            Self::FontWidth,
            Self::FontDisplay,
            Self::UnicodeRange,
            Self::FontFeatureSettings,
            Self::FontVariationSettings,
            Self::FontNamedInstance,
            Self::FontLanguageOverride,
            Self::AscentOverride,
            Self::DescentOverride,
            Self::LineGapOverride,
        ]
        .into_iter()
        .find(|kind| {
            kind.as_str().eq_ignore_ascii_case(name)
                || (*kind == Self::FontWidth && name.eq_ignore_ascii_case("font-stretch"))
        })
    }
}

/// An owned validated descriptor value without parsed occurrence provenance.
///
/// These values contain no descriptor-name position. Raw parsing reports source
/// positions separately in diagnostics; stylesheet occurrences retain their real
/// authored name positions in [`CssFontFaceDescriptor`]. Values remain authored:
/// they do not perform font matching, loading, or contextual usability checks.
///
/// # Specified serialization
///
/// Canonical specified descriptor value, without a synthetic rule or resource loading.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorValue {
    /// A `font-family` value: one literal family name.
    FontFamily(CssFontFaceFamily),
    /// A `src` value: the ordered source list.
    Src(CssFontFaceSourceList),
    /// A `font-weight` value: `auto` or an authored one- or two-value weight range.
    FontWeight(CssFontFaceWeight),
    /// A `font-style` value: `auto`, a style keyword, or an authored oblique angle range.
    FontStyle(CssFontFaceStyle),
    /// A `font-width` value: `auto` or an authored one- or two-value width range.
    FontWidth(CssFontFaceWidth),
    /// A `font-display` value.
    FontDisplay(CssFontDisplay),
    /// A `unicode-range` value: the ordered range list.
    UnicodeRange(CssUnicodeRangeList),
    /// A `font-feature-settings` value, with feature tags as authored.
    FontFeatureSettings(CssAuthoredFontFeatureSettings),
    /// A `font-variation-settings` value.
    FontVariationSettings(CssFontVariationSettings),
    /// A `font-named-instance` value: `auto` or a string.
    FontNamedInstance(CssFontNamedInstance),
    /// A `font-language-override` value.
    FontLanguageOverride(CssFontLanguageOverride),
    /// An `ascent-override` value.
    AscentOverride(CssFontMetricOverride),
    /// A `descent-override` value.
    DescentOverride(CssFontMetricOverride),
    /// A `line-gap-override` value.
    LineGapOverride(CssFontMetricOverride),
}

impl CssFontFaceDescriptorValue {
    /// Returns the descriptor grammar represented by this typed value.
    #[must_use]
    pub const fn kind(&self) -> CssFontFaceDescriptorKind {
        match self {
            Self::FontFamily(_) => CssFontFaceDescriptorKind::FontFamily,
            Self::Src(_) => CssFontFaceDescriptorKind::Src,
            Self::FontWeight(_) => CssFontFaceDescriptorKind::FontWeight,
            Self::FontStyle(_) => CssFontFaceDescriptorKind::FontStyle,
            Self::FontWidth(_) => CssFontFaceDescriptorKind::FontWidth,
            Self::FontDisplay(_) => CssFontFaceDescriptorKind::FontDisplay,
            Self::UnicodeRange(_) => CssFontFaceDescriptorKind::UnicodeRange,
            Self::FontFeatureSettings(_) => CssFontFaceDescriptorKind::FontFeatureSettings,
            Self::FontVariationSettings(_) => CssFontFaceDescriptorKind::FontVariationSettings,
            Self::FontNamedInstance(_) => CssFontFaceDescriptorKind::FontNamedInstance,
            Self::FontLanguageOverride(_) => CssFontFaceDescriptorKind::FontLanguageOverride,
            Self::AscentOverride(_) => CssFontFaceDescriptorKind::AscentOverride,
            Self::DescentOverride(_) => CssFontFaceDescriptorKind::DescentOverride,
            Self::LineGapOverride(_) => CssFontFaceDescriptorKind::LineGapOverride,
        }
    }
}

/// An authored font-face value: checked ordinary grammar or deferred whole-descriptor grammar.
///
/// # Specified serialization
///
/// Reads a checked ordinary or pending descriptor through its owning writer. No descriptor name or
/// containing rule is generated.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFaceDescriptorValue {
    /// A checked value of the descriptor's own grammar.
    Ordinary(CssFontFaceDescriptorValue),
    /// A complete descriptor token stream awaiting environment substitution.
    Pending(CssPendingFontFaceDescriptorValue),
}

impl CssAuthoredFontFaceDescriptorValue {
    /// Returns the descriptor grammar of the ordinary or pending value.
    #[must_use]
    pub const fn kind(&self) -> CssFontFaceDescriptorKind {
        match self {
            Self::Ordinary(value) => value.kind(),
            Self::Pending(value) => value.kind(),
        }
    }

    pub(crate) fn pending(kind: CssFontFaceDescriptorKind, components: CssComponentValues) -> Self {
        Self::Pending(CssPendingFontFaceDescriptorValue { kind, components })
    }
}

impl From<CssFontFaceDescriptorValue> for CssAuthoredFontFaceDescriptorValue {
    fn from(value: CssFontFaceDescriptorValue) -> Self {
        Self::Ordinary(value)
    }
}

/// A complete descriptor token stream awaiting environment substitution.
///
/// # Specified serialization
///
/// Reads the supported pending whole-descriptor component stream without substitution, resolving a
/// font or changing its original token origins.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPendingFontFaceDescriptorValue {
    kind: CssFontFaceDescriptorKind,
    components: CssComponentValues,
}

impl CssPendingFontFaceDescriptorValue {
    /// Returns the descriptor grammar the pending value will be rechecked against.
    #[must_use]
    pub const fn kind(&self) -> CssFontFaceDescriptorKind {
        self.kind
    }

    /// Returns the complete pending component stream.
    #[must_use]
    pub const fn components(&self) -> &CssComponentValues {
        &self.components
    }
}

/// One ordered descriptor occurrence, retaining a parsed name position when present.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceDescriptor {
    value: CssAuthoredFontFaceDescriptorValue,
    position: Option<CssSourcePosition>,
}

impl CssFontFaceDescriptor {
    /// Constructs a descriptor without parsed source position.
    #[must_use]
    pub const fn new(value: CssAuthoredFontFaceDescriptorValue) -> Self {
        Self {
            value,
            position: None,
        }
    }

    pub(crate) const fn with_position(mut self, position: CssSourcePosition) -> Self {
        self.position = Some(position);
        self
    }

    /// Returns the authored ordinary or pending value.
    #[must_use]
    pub const fn value(&self) -> &CssAuthoredFontFaceDescriptorValue {
        &self.value
    }

    /// Returns the parsed descriptor name position, if this came from source.
    #[must_use]
    pub const fn position(&self) -> Option<CssSourcePosition> {
        self.position
    }
}

/// All admitted font-face descriptors, in authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceDescriptors {
    occurrences: Vec<CssFontFaceDescriptor>,
}

impl CssFontFaceDescriptors {
    /// Retains descriptor occurrences in authored order, including repetitions.
    #[must_use]
    pub fn new(occurrences: Vec<CssFontFaceDescriptor>) -> Self {
        Self { occurrences }
    }

    /// Iterates all admitted occurrences in authored order.
    pub fn occurrences(&self) -> impl ExactSizeIterator<Item = &CssFontFaceDescriptor> {
        self.occurrences.iter()
    }

    /// Finds the last occurrence of a kind; a pending value supersedes an earlier ordinary one.
    #[must_use]
    pub fn effective(&self, kind: CssFontFaceDescriptorKind) -> Option<&CssFontFaceDescriptor> {
        self.occurrences
            .iter()
            .rev()
            .find(|record| record.value.kind() == kind)
    }
}

/// # Specified serialization
///
/// Emits one retained local or URL source through its concrete provider.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceSource {
    /// A `url()` source with optional `format()` and `tech()` hints.
    Url(CssFontFaceUrlSource),
    /// A `local()` source naming an installed font.
    Local(CssFontLocalName),
}

/// # Specified serialization
///
/// Emits the authored URL followed by its optional format and ordered technologies. Keyword and
/// string format productions remain distinct, including empty strings.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceUrlSource {
    url: CssUrl,
    format: Option<CssFontFormat>,
    tech: Vec<CssFontTechHint>,
}

impl CssFontFaceUrlSource {
    /// Preserves a shared authored URL, optional format, and ordered technologies.
    /// Resource resolution and support remain downstream.
    #[must_use]
    pub fn new(url: CssUrl, format: Option<CssFontFormat>, tech: Vec<CssFontTechHint>) -> Self {
        Self { url, format, tech }
    }

    /// Returns the authored URL.
    #[must_use]
    pub const fn url(&self) -> &CssUrl {
        &self.url
    }

    /// Returns the single authored format argument; absence differs from an empty string.
    #[must_use]
    pub const fn format(&self) -> Option<&CssFontFormat> {
        self.format.as_ref()
    }

    /// Returns technology hints in authored order, including repetitions.
    #[must_use]
    pub fn tech(&self) -> &[CssFontTechHint] {
        &self.tech
    }

    /// Yields distinct required technologies in first-occurrence order, followed by
    /// the legacy string's implied variations requirement when not already authored.
    pub fn required_technologies(&self) -> impl Iterator<Item = CssFontTechHint> + '_ {
        let implied = self.format.as_ref().and_then(|format| match format {
            CssFontFormat::String(value) => value
                .legacy_variation_format()
                .map(|_| CssFontTechHint::Variations),
            CssFontFormat::Keyword(_) => None,
        });
        let mut seen = Vec::new();
        self.tech
            .iter()
            .copied()
            .chain(implied)
            .filter(move |hint| {
                if seen.contains(hint) {
                    false
                } else {
                    seen.push(*hint);
                    true
                }
            })
    }
}

/// The distinct authored keyword and string productions of a font format.
/// Used by font source hints and `font-format()` supports predicates.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFormat {
    /// A `format()` keyword such as `woff2`.
    Keyword(CssFontFormatHint),
    /// A `format()` string, which may be empty or unrecognized.
    String(CssFontFormatString),
}

impl CssFontFormat {
    /// Recognizes format meaning without deciding resource support or changing authored identity.
    #[must_use]
    pub fn recognized_format(&self) -> Option<CssFontFormatHint> {
        match self {
            Self::Keyword(format) => Some(*format),
            Self::String(value) => value.recognized(),
        }
    }
}

/// One authored string from a font source hint or a `font-format()` predicate.
///
/// Empty and unrecognized strings remain authored values. This type does not
/// determine whether a resource loader recognizes or a processor supports the
/// format. Support predicates retain strings separately from keyword arguments.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFormatString {
    value: String,
}

impl CssFontFormatString {
    /// Preserves decoded authored text, including empty and unknown format strings.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the decoded format text, which may be empty.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }

    fn recognized(&self) -> Option<CssFontFormatHint> {
        CssFontFormatHint::from_ascii_name(self.value.as_bytes())
            .or_else(|| self.legacy_variation_format())
    }

    fn legacy_variation_format(&self) -> Option<CssFontFormatHint> {
        // Fonts4 section 4.3.1 defines these string equivalents. Keep this table
        // separate from the keyword grammar: bare legacy names are not keywords.
        if self.value.eq_ignore_ascii_case("woff2-variations") {
            Some(CssFontFormatHint::Woff2)
        } else if self.value.eq_ignore_ascii_case("woff-variations") {
            Some(CssFontFormatHint::Woff)
        } else if self.value.eq_ignore_ascii_case("truetype-variations") {
            Some(CssFontFormatHint::TrueType)
        } else if self.value.eq_ignore_ascii_case("opentype-variations") {
            Some(CssFontFormatHint::OpenType)
        } else {
            None
        }
    }
}

/// # Specified serialization
///
/// Emits a decoded literal family name, quoting names that cannot be bare identifier sequences.
/// This model has no generic or authored-quoting branch.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFaceFamily {
    name: String,
}

impl CssFontFaceFamily {
    /// Preserves a decoded literal name, including empty and whitespace-only names.
    /// Reserved spellings remain literal names and can be serialized quoted.
    /// NUL is rejected because CSS replaces it with a different character.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        (!name.contains('\0')).then_some(Self { name })
    }

    /// Returns the decoded family name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// # Specified serialization
///
/// Emits `local()` containing the CSSOM string serialization of the decoded name.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontLocalName {
    name: String,
}

impl CssFontLocalName {
    /// Preserves a decoded literal name, including empty and whitespace-only names.
    /// Reserved spellings remain literal names and can be serialized quoted.
    /// NUL is rejected because CSS replaces it with a different character.
    #[must_use]
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        (!name.contains('\0')).then_some(Self { name })
    }

    /// Returns the decoded local font name.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// # Specified serialization
///
/// Emits the checked nonempty source list in retained order, separated by `, `.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceSourceList {
    sources: Vec<CssFontFaceSource>,
}

impl CssFontFaceSourceList {
    /// Creates a source list, or returns `None` when `sources` is empty.
    #[must_use]
    pub fn try_new(sources: Vec<CssFontFaceSource>) -> Option<Self> {
        if sources.is_empty() {
            None
        } else {
            Some(Self::new(sources))
        }
    }

    #[must_use]
    pub(crate) fn new(sources: Vec<CssFontFaceSource>) -> Self {
        debug_assert!(!sources.is_empty());
        Self { sources }
    }

    /// Returns the sources in authored order; the slice is never empty.
    #[must_use]
    pub fn sources(&self) -> &[CssFontFaceSource] {
        &self.sources
    }
}

/// The `font-display` descriptor keyword, which chooses how a font face is shown while it loads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontDisplay {
    /// `auto`: the font display policy is user-agent-defined.
    Auto,
    /// `block`: a short block period and an infinite swap period.
    Block,
    /// `swap`: an extremely small block period and an infinite swap period.
    Swap,
    /// `fallback`: an extremely small block period and a short swap period.
    Fallback,
    /// `optional`: the font is used if it loads immediately; otherwise the block and swap periods
    /// are treated as expired before it finished loading.
    Optional,
}

/// A non-empty list of `<urange>` values of a `unicode-range` descriptor, in authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssUnicodeRangeList {
    ranges: Vec<CssUnicodeRange>,
}

impl CssUnicodeRangeList {
    /// Creates a range list, or returns `None` when `ranges` is empty.
    #[must_use]
    pub fn try_new(ranges: Vec<CssUnicodeRange>) -> Option<Self> {
        if ranges.is_empty() {
            None
        } else {
            Some(Self::new(ranges))
        }
    }

    #[must_use]
    pub(crate) fn new(ranges: Vec<CssUnicodeRange>) -> Self {
        debug_assert!(!ranges.is_empty());
        Self { ranges }
    }

    /// Returns the ranges in authored order; the slice is never empty.
    #[must_use]
    pub fn ranges(&self) -> &[CssUnicodeRange] {
        &self.ranges
    }
}

/// One inclusive range of Unicode code points of a `unicode-range` descriptor, as scalar bounds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUnicodeRange {
    start: u32,
    end: u32,
}

impl CssUnicodeRange {
    /// Creates a range from inclusive bounds, or returns `None` unless `start <= end` and `end` is at most `0x10FFFF`.
    #[must_use]
    pub const fn try_new(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= 0x10ffff {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Returns the first code point of the range.
    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    /// Returns the last code point of the range, which is included.
    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }
}

/// The finite keyword vocabulary shared by font hints and support predicates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFormatHint {
    /// `woff`: a WOFF 1.0 font.
    Woff,
    /// `woff2`: a WOFF 2.0 font.
    Woff2,
    /// `truetype`: a TrueType font; it is equivalent to `opentype` for compatibility checks.
    TrueType,
    /// `opentype`: an OpenType font; it is equivalent to `truetype` for compatibility checks.
    OpenType,
    /// `collection`: an OpenType collection.
    Collection,
    /// `embedded-opentype`: an Embedded OpenType font.
    EmbeddedOpenType,
    /// `svg`: an SVG font, which the specification deprecates.
    Svg,
}

impl CssFontFormatHint {
    /// Compares format compatibility without changing authored equality.
    ///
    /// Fonts4 section 11.2 makes TrueType and OpenType synonymous. Other formats
    /// are equivalent only to themselves. This does not compare technology hints
    /// or determine resource support.
    #[must_use]
    pub fn is_equivalent_to(self, other: Self) -> bool {
        self == other
            || matches!(
                (self, other),
                (Self::TrueType, Self::OpenType) | (Self::OpenType, Self::TrueType)
            )
    }

    pub(crate) fn from_ascii_name(value: &[u8]) -> Option<Self> {
        if value.eq_ignore_ascii_case(b"woff") {
            Some(Self::Woff)
        } else if value.eq_ignore_ascii_case(b"woff2") {
            Some(Self::Woff2)
        } else if value.eq_ignore_ascii_case(b"truetype") {
            Some(Self::TrueType)
        } else if value.eq_ignore_ascii_case(b"opentype") {
            Some(Self::OpenType)
        } else if value.eq_ignore_ascii_case(b"collection") {
            Some(Self::Collection)
        } else if value.eq_ignore_ascii_case(b"embedded-opentype") {
            Some(Self::EmbeddedOpenType)
        } else if value.eq_ignore_ascii_case(b"svg") {
            Some(Self::Svg)
        } else {
            None
        }
    }
}

/// The finite technology vocabulary shared by font hints and support predicates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontTechHint {
    /// `variations`: support for font variations.
    Variations,
    /// `palettes`: support for font color palettes.
    Palettes,
    /// `color-COLRv0`: support for the color font technology represented by the `COLR` table.
    ColorCOLRv0,
    /// `color-COLRv1`: support for the color font technology represented by the `COLR` table.
    ColorCOLRv1,
    /// `color-SVG`: support for the `SVG` color table.
    ColorSVG,
    /// `color-sbix`: support for the `sbix` color table.
    ColorSbix,
    /// `color-CBDT`: support for the `CBDT` color table.
    ColorCBDT,
    /// `features-opentype`: support for OpenType layout features.
    FeaturesOpenType,
    /// `features-aat`: support for AAT font features.
    FeaturesAAT,
    /// `features-graphite`: support for Graphite font features.
    FeaturesGraphite,
    /// `incremental`: support for incremental font transfer.
    Incremental,
}
impl CssFontTechHint {
    pub(crate) fn from_ascii_name(value: &[u8]) -> Option<Self> {
        if value.eq_ignore_ascii_case(b"variations") {
            Some(Self::Variations)
        } else if value.eq_ignore_ascii_case(b"palettes") {
            Some(Self::Palettes)
        } else if value.eq_ignore_ascii_case(b"color-colrv0") {
            Some(Self::ColorCOLRv0)
        } else if value.eq_ignore_ascii_case(b"color-colrv1") {
            Some(Self::ColorCOLRv1)
        } else if value.eq_ignore_ascii_case(b"color-svg") {
            Some(Self::ColorSVG)
        } else if value.eq_ignore_ascii_case(b"color-sbix") {
            Some(Self::ColorSbix)
        } else if value.eq_ignore_ascii_case(b"color-cbdt") {
            Some(Self::ColorCBDT)
        } else if value.eq_ignore_ascii_case(b"features-opentype") {
            Some(Self::FeaturesOpenType)
        } else if value.eq_ignore_ascii_case(b"features-aat") {
            Some(Self::FeaturesAAT)
        } else if value.eq_ignore_ascii_case(b"features-graphite") {
            Some(Self::FeaturesGraphite)
        } else if value.eq_ignore_ascii_case(b"incremental") {
            Some(Self::Incremental)
        } else {
            None
        }
    }
}
impl CssAuthoredFontFaceDescriptorValue {
    /// Parses one complete raw `@font-face` descriptor value in the selected grammar.
    ///
    /// The source contains only the value, without a descriptor name, annotation or
    /// declaration delimiter. The returned typed value has no fabricated name position.
    /// Diagnostics use the original source's UTF-8 bytes, zero-based UTF-16 columns and
    /// actual EOF. Invalid outer values return `None` with `RejectInput`; resource limits
    /// retain `StopAtNestingLimit`. A retained `src` list can report discarded members.
    /// Implicit closures are reported only for retained components. This does not require
    /// surrounding `font-family` or `src` descriptors, match fonts, or load resources.
    ///
    /// The parser is not read: neither its document mode nor its component limits apply to this grammar, and only the fixed nesting ceiling bounds the input.
    #[expect(
        unused_variables,
        reason = "this grammar does not read the parser configuration"
    )]
    #[must_use]
    pub fn parse(
        parser: &crate::CssParser,
        source: impl AsRef<str>,
        kind: CssFontFaceDescriptorKind,
    ) -> crate::CssParseReport<Option<CssAuthoredFontFaceDescriptorValue>> {
        crate::parser::fragments::parse_font_face_descriptor_value(source.as_ref(), kind)
    }
}
