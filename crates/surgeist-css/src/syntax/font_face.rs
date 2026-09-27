use super::{
    CssAuthoredFontFeatureSettings, CssDescriptorOccurrence, CssFiniteNumber, CssSourcePosition,
};
use crate::CssFontFaceWidth;

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
/// authored name positions in [`CssDescriptorOccurrence`]. Values remain authored:
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
        }
    }

    pub(crate) fn into_occurrence(self, position: CssSourcePosition) -> CssFontFaceDescriptor {
        match self {
            Self::FontFamily(value) => {
                CssFontFaceDescriptor::FontFamily(CssDescriptorOccurrence::new(value, position))
            }
            Self::Src(value) => {
                CssFontFaceDescriptor::Src(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontWeight(value) => {
                CssFontFaceDescriptor::FontWeight(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontStyle(value) => {
                CssFontFaceDescriptor::FontStyle(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontWidth(value) => {
                CssFontFaceDescriptor::FontWidth(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontDisplay(value) => {
                CssFontFaceDescriptor::FontDisplay(CssDescriptorOccurrence::new(value, position))
            }
            Self::UnicodeRange(value) => {
                CssFontFaceDescriptor::UnicodeRange(CssDescriptorOccurrence::new(value, position))
            }
            Self::FontFeatureSettings(value) => CssFontFaceDescriptor::FontFeatureSettings(
                CssDescriptorOccurrence::new(value, position),
            ),
        }
    }
}

/// The validated semantic aggregate of authored `@font-face` descriptor occurrences.
///
/// Every valid occurrence retains its authored order, typed value, and descriptor-name position.
/// Typed accessors expose the effective last valid occurrence of each descriptor. Construction is
/// crate-private, so callers cannot forge descriptor provenance. Every descriptor is optional
/// in authored syntax. Font matching later requires effective `font-family` and `src` values;
/// this aggregate does not match or load fonts.
#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceDescriptors {
    font_family: Option<CssDescriptorOccurrence<CssFontFaceFamily>>,
    src: Option<CssDescriptorOccurrence<CssFontFaceSourceList>>,
    font_weight: Option<CssDescriptorOccurrence<CssFontFaceWeight>>,
    font_style: Option<CssDescriptorOccurrence<CssFontFaceStyle>>,
    font_width: Option<CssDescriptorOccurrence<CssFontFaceWidth>>,
    font_display: Option<CssDescriptorOccurrence<CssFontDisplay>>,
    unicode_range: Option<CssDescriptorOccurrence<CssUnicodeRangeList>>,
    font_feature_settings: Option<CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>>,
    occurrences: Vec<CssFontFaceDescriptor>,
}

impl CssFontFaceDescriptors {
    #[must_use]
    #[cfg(test)]
    pub(crate) fn new(
        font_family: Option<CssDescriptorOccurrence<CssFontFaceFamily>>,
        src: Option<CssDescriptorOccurrence<CssFontFaceSourceList>>,
        font_weight: Option<CssDescriptorOccurrence<CssFontFaceWeight>>,
        font_style: Option<CssDescriptorOccurrence<CssFontFaceStyle>>,
        font_width: Option<CssDescriptorOccurrence<CssFontFaceWidth>>,
        font_display: Option<CssDescriptorOccurrence<CssFontDisplay>>,
        unicode_range: Option<CssDescriptorOccurrence<CssUnicodeRangeList>>,
    ) -> Self {
        let mut occurrences = Vec::new();
        if let Some(value) = font_family {
            occurrences.push(CssFontFaceDescriptor::FontFamily(value));
        }
        if let Some(value) = src {
            occurrences.push(CssFontFaceDescriptor::Src(value));
        }
        if let Some(value) = font_weight {
            occurrences.push(CssFontFaceDescriptor::FontWeight(value));
        }
        if let Some(value) = font_style {
            occurrences.push(CssFontFaceDescriptor::FontStyle(value));
        }
        if let Some(value) = font_width {
            occurrences.push(CssFontFaceDescriptor::FontWidth(value));
        }
        if let Some(value) = font_display {
            occurrences.push(CssFontFaceDescriptor::FontDisplay(value));
        }
        if let Some(value) = unicode_range {
            occurrences.push(CssFontFaceDescriptor::UnicodeRange(value));
        }
        Self::from_occurrences(occurrences)
    }

    #[must_use]
    pub(crate) fn from_occurrences(occurrences: Vec<CssFontFaceDescriptor>) -> Self {
        let mut font_family = None;
        let mut src = None;
        let mut font_weight = None;
        let mut font_style = None;
        let mut font_width = None;
        let mut font_display = None;
        let mut unicode_range = None;
        let mut font_feature_settings = None;

        for descriptor in &occurrences {
            match descriptor {
                CssFontFaceDescriptor::FontFamily(value) => font_family = Some(value.clone()),
                CssFontFaceDescriptor::Src(value) => src = Some(value.clone()),
                CssFontFaceDescriptor::FontWeight(value) => font_weight = Some(value.clone()),
                CssFontFaceDescriptor::FontStyle(value) => font_style = Some(value.clone()),
                CssFontFaceDescriptor::FontWidth(value) => font_width = Some(value.clone()),
                CssFontFaceDescriptor::FontDisplay(value) => font_display = Some(value.clone()),
                CssFontFaceDescriptor::UnicodeRange(value) => unicode_range = Some(value.clone()),
                CssFontFaceDescriptor::FontFeatureSettings(value) => {
                    font_feature_settings = Some(value.clone());
                }
            }
        }

        Self {
            font_family,
            src,
            font_weight,
            font_style,
            font_width,
            font_display,
            unicode_range,
            font_feature_settings,
            occurrences,
        }
    }

    #[must_use]
    /// Returns the effective last valid authored `font-family` occurrence, if present.
    pub const fn font_family(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceFamily>> {
        self.font_family.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `src` occurrence, if present.
    pub const fn src(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceSourceList>> {
        self.src.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-weight` occurrence.
    pub const fn font_weight(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceWeight>> {
        self.font_weight.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-style` occurrence.
    pub const fn font_style(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceStyle>> {
        self.font_style.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-width` occurrence.
    pub const fn font_width(&self) -> Option<&CssDescriptorOccurrence<CssFontFaceWidth>> {
        self.font_width.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-display` occurrence.
    pub const fn font_display(&self) -> Option<&CssDescriptorOccurrence<CssFontDisplay>> {
        self.font_display.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `unicode-range` occurrence.
    pub const fn unicode_range(&self) -> Option<&CssDescriptorOccurrence<CssUnicodeRangeList>> {
        self.unicode_range.as_ref()
    }

    #[must_use]
    /// Returns the effective last valid authored `font-feature-settings` occurrence.
    pub const fn font_feature_settings(
        &self,
    ) -> Option<&CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>> {
        self.font_feature_settings.as_ref()
    }

    /// Returns every valid authored descriptor occurrence in source order.
    pub fn occurrences(&self) -> impl ExactSizeIterator<Item = CssFontFaceDescriptorRef<'_>> {
        self.occurrences.iter().map(CssFontFaceDescriptor::as_ref)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum CssFontFaceDescriptor {
    FontFamily(CssDescriptorOccurrence<CssFontFaceFamily>),
    Src(CssDescriptorOccurrence<CssFontFaceSourceList>),
    FontWeight(CssDescriptorOccurrence<CssFontFaceWeight>),
    FontStyle(CssDescriptorOccurrence<CssFontFaceStyle>),
    FontWidth(CssDescriptorOccurrence<CssFontFaceWidth>),
    FontDisplay(CssDescriptorOccurrence<CssFontDisplay>),
    UnicodeRange(CssDescriptorOccurrence<CssUnicodeRangeList>),
    FontFeatureSettings(CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>),
}

impl CssFontFaceDescriptor {
    fn as_ref(&self) -> CssFontFaceDescriptorRef<'_> {
        match self {
            Self::FontFamily(value) => CssFontFaceDescriptorRef::FontFamily(value),
            Self::Src(value) => CssFontFaceDescriptorRef::Src(value),
            Self::FontWeight(value) => CssFontFaceDescriptorRef::FontWeight(value),
            Self::FontStyle(value) => CssFontFaceDescriptorRef::FontStyle(value),
            Self::FontWidth(value) => CssFontFaceDescriptorRef::FontWidth(value),
            Self::FontDisplay(value) => CssFontFaceDescriptorRef::FontDisplay(value),
            Self::UnicodeRange(value) => CssFontFaceDescriptorRef::UnicodeRange(value),
            Self::FontFeatureSettings(value) => {
                CssFontFaceDescriptorRef::FontFeatureSettings(value)
            }
        }
    }
}

/// A borrowed valid authored `@font-face` descriptor occurrence.
#[derive(Clone, Copy, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceDescriptorRef<'a> {
    FontFamily(&'a CssDescriptorOccurrence<CssFontFaceFamily>),
    Src(&'a CssDescriptorOccurrence<CssFontFaceSourceList>),
    FontWeight(&'a CssDescriptorOccurrence<CssFontFaceWeight>),
    FontStyle(&'a CssDescriptorOccurrence<CssFontFaceStyle>),
    FontWidth(&'a CssDescriptorOccurrence<CssFontFaceWidth>),
    FontDisplay(&'a CssDescriptorOccurrence<CssFontDisplay>),
    UnicodeRange(&'a CssDescriptorOccurrence<CssUnicodeRangeList>),
    FontFeatureSettings(&'a CssDescriptorOccurrence<CssAuthoredFontFeatureSettings>),
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceSource {
    Url(CssFontFaceUrlSource),
    Local(CssFontLocalName),
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssFontFaceUrlSource {
    url: String,
    format: Option<CssFontFormatHint>,
    formats: Option<CssFontFormatList>,
    tech: Vec<CssFontTechHint>,
}

impl CssFontFaceUrlSource {
    /// Preserves an authored URL string and its format and technology hints.
    ///
    /// Every URL string is accepted, including empty and whitespace-only strings.
    /// Resource resolution and loading belong to a later phase. The optional return
    /// retains the existing signature after removal of the nonempty restriction.
    #[must_use]
    pub fn try_new(
        url: impl Into<String>,
        format: Option<CssFontFormatHint>,
        tech: Vec<CssFontTechHint>,
    ) -> Option<Self> {
        let formats =
            format.map(|format| CssFontFormatList::new(CssFontFormatString::new(format.as_str())));
        Some(Self::new_with_formats(url, formats, tech))
    }

    /// Preserves an authored URL, a checked single format argument, and technologies.
    ///
    /// Construction is infallible because the format wrapper already enforces its
    /// cardinality. Empty URLs and empty or unrecognized format strings are valid
    /// authored values. Strings are decoded values, not CSS source to parse again.
    #[must_use]
    pub fn new_with_formats(
        url: impl Into<String>,
        formats: Option<CssFontFormatList>,
        tech: Vec<CssFontTechHint>,
    ) -> Self {
        let format = formats.as_ref().and_then(CssFontFormatList::recognized);
        Self {
            url: url.into(),
            format,
            formats,
            tech,
        }
    }

    #[must_use]
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Returns the recognized base format, including the four legacy variation strings.
    ///
    /// `None` can mean either an absent hint or an unrecognized string; use
    /// [`Self::formats`] to distinguish them. Format recognition does not determine
    /// resource support. TrueType and OpenType retain distinct authored identities;
    /// [`CssFontFormatHint::is_equivalent_to`] compares their compatibility meaning.
    #[must_use]
    pub const fn format(&self) -> Option<&CssFontFormatHint> {
        self.format.as_ref()
    }

    /// Returns the single authored `format()` argument, when present.
    ///
    /// The list-named wrapper preserves the original inspection API. A present
    /// hint always contains exactly one string, including empty or unrecognized
    /// strings; `None` means that the source has no `format()` hint.
    #[must_use]
    pub const fn formats(&self) -> Option<&CssFontFormatList> {
        self.formats.as_ref()
    }

    /// Returns technology hints in authored order, including repetitions.
    ///
    /// A legacy format string's implied technology is exposed separately by
    /// [`Self::required_technologies`].
    #[must_use]
    pub fn tech(&self) -> &[CssFontTechHint] {
        &self.tech
    }

    /// Returns the distinct technologies that the source requires together.
    ///
    /// Authored technologies retain first-occurrence order. The `variations`
    /// requirement implied by a legacy variation format string follows them when
    /// it was not already authored. This projection leaves [`Self::tech`] unchanged
    /// and does not decide whether a resource loader supports those technologies.
    pub fn required_technologies(&self) -> impl Iterator<Item = CssFontTechHint> + '_ {
        let implied = self.formats.as_ref().and_then(|formats| {
            formats.formats[0]
                .legacy_variation_format()
                .map(|_| CssFontTechHint::Variations)
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

/// One authored string from a font source `format()` hint.
///
/// Empty and unrecognized strings remain authored values. This type does not
/// determine whether a resource loader recognizes or supports the format.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFormatString {
    value: String,
}

impl CssFontFormatString {
    /// Preserves an authored string, including the empty string.
    ///
    /// Every string is accepted. The optional return retains this constructor's
    /// existing signature after removal of the former nonempty restriction.
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Some(Self::new(value))
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
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

/// Exactly one authored string from a font source `format()` hint.
///
/// The name and slice accessor originate in the earlier Fonts3 list model.
/// The selected Fonts4 grammar permits exactly one argument, so empty and
/// multiple-element inputs are rejected by [`Self::try_new`]. The one string
/// itself may be empty or unrecognized; resource support is a later concern.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontFormatList {
    formats: [CssFontFormatString; 1],
}

impl CssFontFormatList {
    #[must_use]
    pub fn try_new(formats: Vec<CssFontFormatString>) -> Option<Self> {
        let [format]: [CssFontFormatString; 1] = formats.try_into().ok()?;
        Some(Self::new(format))
    }

    #[must_use]
    pub(crate) fn new(format: CssFontFormatString) -> Self {
        Self { formats: [format] }
    }

    #[must_use]
    pub fn formats(&self) -> &[CssFontFormatString] {
        &self.formats
    }

    #[must_use]
    fn recognized(&self) -> Option<CssFontFormatHint> {
        self.formats[0].recognized()
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceWeight {
    start: CssFontFaceWeightValue,
    end: Option<CssFontFaceWeightValue>,
    keyword: Option<CssFontFaceWeightKeyword>,
}

impl CssFontFaceWeight {
    #[must_use]
    pub fn normal() -> Self {
        Self::from_keyword(CssFontFaceWeightKeyword::Normal)
    }

    #[must_use]
    pub fn bold() -> Self {
        Self::from_keyword(CssFontFaceWeightKeyword::Bold)
    }

    #[must_use]
    pub(crate) fn from_keyword(keyword: CssFontFaceWeightKeyword) -> Self {
        let value = match keyword {
            CssFontFaceWeightKeyword::Normal => 400.0,
            CssFontFaceWeightKeyword::Bold => 700.0,
        };
        Self {
            start: CssFontFaceWeightValue {
                value: CssFiniteNumber::new_unchecked(value),
            },
            end: None,
            keyword: Some(keyword),
        }
    }

    #[must_use]
    pub fn try_single(value: f32) -> Option<Self> {
        Some(Self {
            start: CssFontFaceWeightValue::try_new(value)?,
            end: None,
            keyword: None,
        })
    }

    #[must_use]
    pub fn try_range(start: f32, end: f32) -> Option<Self> {
        let start = CssFontFaceWeightValue::try_new(start)?;
        let end = CssFontFaceWeightValue::try_new(end)?;
        if start.value().value() <= end.value().value() {
            Some(Self {
                start,
                end: Some(end),
                keyword: None,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn start(self) -> CssFontFaceWeightValue {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> Option<CssFontFaceWeightValue> {
        self.end
    }

    #[must_use]
    pub const fn keyword(self) -> Option<CssFontFaceWeightKeyword> {
        self.keyword
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceWeightKeyword {
    Normal,
    Bold,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceWeightValue {
    value: CssFiniteNumber,
}

impl CssFontFaceWeightValue {
    #[must_use]
    pub fn try_new(value: f32) -> Option<Self> {
        if (1.0..=1000.0).contains(&value) {
            CssFiniteNumber::try_new(value).map(|value| Self { value })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn value(self) -> CssFiniteNumber {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssFontFaceStyle {
    Normal,
    Italic,
    Oblique(Option<CssFontFaceObliqueRange>),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssFontFaceObliqueRange {
    start_degrees: CssFiniteNumber,
    end_degrees: Option<CssFiniteNumber>,
}

impl CssFontFaceObliqueRange {
    #[must_use]
    pub fn try_new(start_degrees: f32, end_degrees: Option<f32>) -> Option<Self> {
        if !(-90.0..=90.0).contains(&start_degrees) {
            return None;
        }

        let start_degrees = CssFiniteNumber::try_new(start_degrees)?;
        let end_degrees = match end_degrees {
            Some(end_degrees)
                if (-90.0..=90.0).contains(&end_degrees)
                    && start_degrees.value() <= end_degrees =>
            {
                Some(CssFiniteNumber::try_new(end_degrees)?)
            }
            Some(_) => return None,
            None => None,
        };

        Some(Self {
            start_degrees,
            end_degrees,
        })
    }

    #[must_use]
    pub const fn start_degrees(self) -> CssFiniteNumber {
        self.start_degrees
    }

    #[must_use]
    pub const fn end_degrees(self) -> Option<CssFiniteNumber> {
        self.end_degrees
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

    #[must_use]
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Woff => "woff",
            Self::Woff2 => "woff2",
            Self::TrueType => "truetype",
            Self::OpenType => "opentype",
            Self::Collection => "collection",
            Self::EmbeddedOpenType => "embedded-opentype",
            Self::Svg => "svg",
        }
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
