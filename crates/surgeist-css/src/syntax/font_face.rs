use super::CssSourcePosition;
use crate::{
    CssAuthoredFontFeatureSettings, CssComponentValues, CssFontFaceStyle, CssFontFaceWeight,
    CssFontFaceWidth, CssFontLanguageOverride, CssFontMetricOverride, CssFontNamedInstance,
    CssFontVariationSettings, CssUrl,
};

mod pending;
pub use pending::{CssFontFaceValueError, CssFontFaceValueErrorKind};

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceRule {
    descriptors: CssFontFaceDescriptors,
    position: CssSourcePosition,
}

impl CssFontFaceRule {
    #[must_use]
    pub(crate) const fn new(
        descriptors: CssFontFaceDescriptors,
        position: CssSourcePosition,
    ) -> Self {
        Self {
            descriptors,
            position,
        }
    }

    #[must_use]
    pub const fn descriptors(&self) -> &CssFontFaceDescriptors {
        &self.descriptors
    }

    #[must_use]
    pub const fn position(&self) -> CssSourcePosition {
        self.position
    }
}

/// Selects an authored `@font-face` descriptor-value grammar.
///
/// This context has no source position: its name is not part of a raw value input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorKind {
    FontFamily,
    Src,
    FontWeight,
    FontStyle,
    FontWidth,
    FontDisplay,
    UnicodeRange,
    FontFeatureSettings,
    FontVariationSettings,
    FontNamedInstance,
    FontLanguageOverride,
    AscentOverride,
    DescentOverride,
    LineGapOverride,
}

impl CssFontFaceDescriptorKind {
    /// Returns the canonical CSS descriptor name.
    #[must_use]
    pub const fn css_name(self) -> &'static str {
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

    pub(crate) fn from_css_name(name: &str) -> Option<Self> {
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
            kind.css_name().eq_ignore_ascii_case(name)
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorValue {
    FontFamily(CssFontFaceFamily),
    Src(CssFontFaceSourceList),
    FontWeight(CssFontFaceWeight),
    FontStyle(CssFontFaceStyle),
    FontWidth(CssFontFaceWidth),
    FontDisplay(CssFontDisplay),
    UnicodeRange(CssUnicodeRangeList),
    FontFeatureSettings(CssAuthoredFontFeatureSettings),
    FontVariationSettings(CssFontVariationSettings),
    FontNamedInstance(CssFontNamedInstance),
    FontLanguageOverride(CssFontLanguageOverride),
    AscentOverride(CssFontMetricOverride),
    DescentOverride(CssFontMetricOverride),
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
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredFontFaceDescriptorValue {
    Ordinary(CssFontFaceDescriptorValue),
    Pending(CssPendingFontFaceDescriptorValue),
}

impl CssAuthoredFontFaceDescriptorValue {
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
#[derive(Clone, Debug, PartialEq)]
pub struct CssPendingFontFaceDescriptorValue {
    kind: CssFontFaceDescriptorKind,
    components: CssComponentValues,
}

impl CssPendingFontFaceDescriptorValue {
    #[must_use]
    pub const fn kind(&self) -> CssFontFaceDescriptorKind {
        self.kind
    }

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

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceSource {
    Url(CssFontFaceUrlSource),
    Local(CssFontLocalName),
}

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

/// The distinct authored keyword and string productions of a font format hint.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFormat {
    Keyword(CssFontFormatHint),
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

/// One authored string from a font source `format()` hint.
///
/// Empty and unrecognized strings remain authored values. This type does not
/// determine whether a resource loader recognizes or supports the format.
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

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

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

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceSourceList {
    sources: Vec<CssFontFaceSource>,
}

impl CssFontFaceSourceList {
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

    #[must_use]
    pub fn sources(&self) -> &[CssFontFaceSource] {
        &self.sources
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontDisplay {
    Auto,
    Block,
    Swap,
    Fallback,
    Optional,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssUnicodeRangeList {
    ranges: Vec<CssUnicodeRange>,
}

impl CssUnicodeRangeList {
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

    #[must_use]
    pub fn ranges(&self) -> &[CssUnicodeRange] {
        &self.ranges
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssUnicodeRange {
    start: u32,
    end: u32,
}

impl CssUnicodeRange {
    #[must_use]
    pub const fn try_new(start: u32, end: u32) -> Option<Self> {
        if start <= end && end <= 0x10ffff {
            Some(Self { start, end })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFormatHint {
    Woff,
    Woff2,
    TrueType,
    OpenType,
    Collection,
    EmbeddedOpenType,
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontTechHint {
    Variations,
    Palettes,
    ColorCOLRv0,
    ColorCOLRv1,
    ColorSVG,
    ColorSbix,
    ColorCBDT,
    FeaturesOpenType,
    FeaturesAAT,
    FeaturesGraphite,
    Incremental,
}
