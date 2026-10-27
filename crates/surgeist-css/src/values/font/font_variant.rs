//! Authored font-variant family and intrinsic checked values.

use crate::rule::font_feature_values::CssFontFeatureValueName;

/// The CSS 2.1 `font-variant` keyword that the `font` shorthand can set: `normal` or `small-caps`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariant {
    /// `normal`: selects a font that is not a small-caps font.
    Normal,
    /// `small-caps`: selects a small-caps font, in which lowercase letters look like uppercase ones in a smaller size.
    SmallCaps,
}

/// The current authored `font-variant-caps` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantCaps {
    /// `normal`: none of the capitalization features are enabled.
    Normal,
    /// `small-caps`: enables display of small capitals (OpenType feature `smcp`).
    SmallCaps,
    /// `all-small-caps`: enables display of small capitals for both upper and lowercase letters (OpenType features `c2sc`, `smcp`).
    AllSmallCaps,
    /// `petite-caps`: enables display of petite capitals (OpenType feature `pcap`).
    PetiteCaps,
    /// `all-petite-caps`: enables display of petite capitals for both upper and lowercase letters (OpenType features `c2pc`, `pcap`).
    AllPetiteCaps,
    /// `unicase`: enables display of a mixture of small capitals for uppercase letters with normal lowercase letters (OpenType feature `unic`).
    Unicase,
    /// `titling-caps`: enables display of titling capitals (OpenType feature `titl`).
    TitlingCaps,
}

/// Whether one authored ligature group is enabled or disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatureState {
    /// The ligature group is enabled, as with `common-ligatures`.
    Enabled,
    /// The ligature group is disabled, as with `no-common-ligatures`.
    Disabled,
}

/// A nonempty checked set of authored ligature-group choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantLigatureValues {
    common: Option<CssFontVariantLigatureState>,
    discretionary: Option<CssFontVariantLigatureState>,
    historical: Option<CssFontVariantLigatureState>,
    contextual: Option<CssFontVariantLigatureState>,
}

impl CssFontVariantLigatureValues {
    /// Checks a set of ligature-group choices.
    ///
    /// Returns `None` when no group was chosen.
    #[must_use]
    pub const fn try_new(
        common: Option<CssFontVariantLigatureState>,
        discretionary: Option<CssFontVariantLigatureState>,
        historical: Option<CssFontVariantLigatureState>,
        contextual: Option<CssFontVariantLigatureState>,
    ) -> Option<Self> {
        if common.is_some()
            || discretionary.is_some()
            || historical.is_some()
            || contextual.is_some()
        {
            Some(Self {
                common,
                discretionary,
                historical,
                contextual,
            })
        } else {
            None
        }
    }

    /// Returns the choice for the common ligatures group, or `None` when it was not written.
    #[must_use]
    pub const fn common(self) -> Option<CssFontVariantLigatureState> {
        self.common
    }

    /// Returns the choice for the discretionary ligatures group, or `None` when it was not written.
    #[must_use]
    pub const fn discretionary(self) -> Option<CssFontVariantLigatureState> {
        self.discretionary
    }

    /// Returns the choice for the historical ligatures group, or `None` when it was not written.
    #[must_use]
    pub const fn historical(self) -> Option<CssFontVariantLigatureState> {
        self.historical
    }

    /// Returns the choice for the contextual alternates group, or `None` when it was not written.
    #[must_use]
    pub const fn contextual(self) -> Option<CssFontVariantLigatureState> {
        self.contextual
    }
}

/// The current authored `font-variant-ligatures` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatures {
    /// `normal`: ligatures and contextual forms are applied by the font's default behavior.
    Normal,
    /// `none`: all optional ligatures and contextual forms are disabled.
    None,
    /// One or more explicit ligature-group choices.
    Values(CssFontVariantLigatureValues),
}

/// The figure form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFigure {
    /// `lining-nums`: enables display of lining numerals (OpenType feature `lnum`).
    LiningNums,
    /// `oldstyle-nums`: enables display of old-style numerals (OpenType feature `onum`).
    OldstyleNums,
}

/// The figure spacing selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericSpacing {
    /// `proportional-nums`: enables display of proportional numerals (OpenType feature `pnum`).
    ProportionalNums,
    /// `tabular-nums`: enables display of tabular numerals (OpenType feature `tnum`).
    TabularNums,
}

/// The fraction form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFraction {
    /// `diagonal-fractions`: enables display of lining diagonal fractions (OpenType feature `frac`).
    DiagonalFractions,
    /// `stacked-fractions`: enables display of lining stacked fractions (OpenType feature `afrc`).
    StackedFractions,
}

/// A nonempty checked set of authored numeric-variant choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantNumericValues {
    figure: Option<CssFontVariantNumericFigure>,
    spacing: Option<CssFontVariantNumericSpacing>,
    fraction: Option<CssFontVariantNumericFraction>,
    ordinal: bool,
    slashed_zero: bool,
}

impl CssFontVariantNumericValues {
    /// Checks a set of numeric-variant choices.
    ///
    /// Returns `None` when no figure, spacing or fraction choice is given and neither `ordinal` nor `slashed-zero` is set.
    #[must_use]
    pub const fn try_new(
        figure: Option<CssFontVariantNumericFigure>,
        spacing: Option<CssFontVariantNumericSpacing>,
        fraction: Option<CssFontVariantNumericFraction>,
        ordinal: bool,
        slashed_zero: bool,
    ) -> Option<Self> {
        if figure.is_some() || spacing.is_some() || fraction.is_some() || ordinal || slashed_zero {
            Some(Self {
                figure,
                spacing,
                fraction,
                ordinal,
                slashed_zero,
            })
        } else {
            None
        }
    }

    /// Returns the figure form, or `None` when `lining-nums` and `oldstyle-nums` were both omitted.
    #[must_use]
    pub const fn figure(self) -> Option<CssFontVariantNumericFigure> {
        self.figure
    }

    /// Returns the figure spacing, or `None` when `proportional-nums` and `tabular-nums` were both omitted.
    #[must_use]
    pub const fn spacing(self) -> Option<CssFontVariantNumericSpacing> {
        self.spacing
    }

    /// Returns the fraction form, or `None` when neither fraction keyword was written.
    #[must_use]
    pub const fn fraction(self) -> Option<CssFontVariantNumericFraction> {
        self.fraction
    }

    /// Returns whether `ordinal` was written, enabling letter forms used with ordinal numbers (OpenType feature `ordn`).
    #[must_use]
    pub const fn ordinal(self) -> bool {
        self.ordinal
    }

    /// Returns whether `slashed-zero` was written, enabling slashed zeros (OpenType feature `zero`).
    #[must_use]
    pub const fn slashed_zero(self) -> bool {
        self.slashed_zero
    }
}

/// The current authored `font-variant-numeric` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumeric {
    /// `normal`: none of the numeric features are enabled.
    Normal,
    /// One or more explicit numeric-variant choices.
    Values(CssFontVariantNumericValues),
}

/// The regional glyph form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianVariant {
    /// `jis78`: JIS78 forms (OpenType feature `jp78`).
    Jis78,
    /// `jis83`: JIS83 forms (OpenType feature `jp83`).
    Jis83,
    /// `jis90`: JIS90 forms (OpenType feature `jp90`).
    Jis90,
    /// `jis04`: JIS2004 forms (OpenType feature `jp04`).
    Jis04,
    /// `simplified`: simplified forms (OpenType feature `smpl`).
    Simplified,
    /// `traditional`: traditional forms (OpenType feature `trad`).
    Traditional,
}

/// The glyph-width form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianWidth {
    /// `full-width`: full-width variants (OpenType feature `fwid`).
    FullWidth,
    /// `proportional-width`: proportionally spaced variants (OpenType feature `pwid`).
    ProportionalWidth,
}

/// A nonempty checked set of authored East Asian variant choices.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssFontVariantEastAsianValues {
    variant: Option<CssFontVariantEastAsianVariant>,
    width: Option<CssFontVariantEastAsianWidth>,
    ruby: bool,
}

impl CssFontVariantEastAsianValues {
    /// Checks a set of East Asian variant choices.
    ///
    /// Returns `None` when no variant or width was given and `ruby` is not set.
    #[must_use]
    pub const fn try_new(
        variant: Option<CssFontVariantEastAsianVariant>,
        width: Option<CssFontVariantEastAsianWidth>,
        ruby: bool,
    ) -> Option<Self> {
        if variant.is_some() || width.is_some() || ruby {
            Some(Self {
                variant,
                width,
                ruby,
            })
        } else {
            None
        }
    }

    /// Returns the regional glyph form, or `None` when it was omitted.
    #[must_use]
    pub const fn variant(self) -> Option<CssFontVariantEastAsianVariant> {
        self.variant
    }

    /// Returns the glyph width form, or `None` when it was omitted.
    #[must_use]
    pub const fn width(self) -> Option<CssFontVariantEastAsianWidth> {
        self.width
    }

    /// Returns whether `ruby` was written, selecting ruby variant glyphs.
    #[must_use]
    pub const fn ruby(self) -> bool {
        self.ruby
    }
}

/// The current authored `font-variant-east-asian` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsian {
    /// `normal`: none of the East Asian features are enabled.
    Normal,
    /// One or more explicit East Asian variant choices.
    Values(CssFontVariantEastAsianValues),
}

/// The current authored `font-variant-position` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantPosition {
    /// `normal`: none of the subscript and superscript features are enabled.
    Normal,
    /// `sub`: enables display of subscript variants (OpenType feature `subs`).
    Sub,
    /// `super`: enables display of superscript variants (OpenType feature `sups`).
    Super,
}

/// A checked nonempty ordered list of decoded feature-value names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariantAlternateNames {
    names: Vec<CssFontFeatureValueName>,
}

impl CssFontVariantAlternateNames {
    /// Checks a list of feature-value names.
    ///
    /// Returns `None` when `names` is empty.
    #[must_use]
    pub fn try_new(names: Vec<CssFontFeatureValueName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }

    /// Returns the names in authored order; the slice is never empty.
    #[must_use]
    pub fn names(&self) -> &[CssFontFeatureValueName] {
        &self.names
    }
}

/// A checked nonempty union of the seven alternate-feature choices.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariantAlternateValues {
    stylistic: Option<CssFontFeatureValueName>,
    historical_forms: bool,
    styleset: Option<CssFontVariantAlternateNames>,
    character_variant: Option<CssFontVariantAlternateNames>,
    swash: Option<CssFontFeatureValueName>,
    ornaments: Option<CssFontFeatureValueName>,
    annotation: Option<CssFontFeatureValueName>,
}

impl CssFontVariantAlternateValues {
    /// Checks a set of alternate-feature choices.
    ///
    /// Returns `None` when no choice is given and `historical_forms` is `false`.
    #[must_use]
    pub fn try_new(
        stylistic: Option<CssFontFeatureValueName>,
        historical_forms: bool,
        styleset: Option<CssFontVariantAlternateNames>,
        character_variant: Option<CssFontVariantAlternateNames>,
        swash: Option<CssFontFeatureValueName>,
        ornaments: Option<CssFontFeatureValueName>,
        annotation: Option<CssFontFeatureValueName>,
    ) -> Option<Self> {
        if stylistic.is_none()
            && !historical_forms
            && styleset.is_none()
            && character_variant.is_none()
            && swash.is_none()
            && ornaments.is_none()
            && annotation.is_none()
        {
            return None;
        }
        Some(Self {
            stylistic,
            historical_forms,
            styleset,
            character_variant,
            swash,
            ornaments,
            annotation,
        })
    }

    /// Returns the `stylistic()` feature-value name, or `None` when it was not written.
    #[must_use]
    pub fn stylistic(&self) -> Option<&CssFontFeatureValueName> {
        self.stylistic.as_ref()
    }
    /// Returns whether `historical-forms` was written.
    #[must_use]
    pub fn historical_forms(&self) -> bool {
        self.historical_forms
    }
    /// Returns the `styleset()` names, or `None` when it was not written.
    #[must_use]
    pub fn styleset(&self) -> Option<&CssFontVariantAlternateNames> {
        self.styleset.as_ref()
    }
    /// Returns the `character-variant()` names, or `None` when it was not written.
    #[must_use]
    pub fn character_variant(&self) -> Option<&CssFontVariantAlternateNames> {
        self.character_variant.as_ref()
    }
    /// Returns the `swash()` feature-value name, or `None` when it was not written.
    #[must_use]
    pub fn swash(&self) -> Option<&CssFontFeatureValueName> {
        self.swash.as_ref()
    }
    /// Returns the `ornaments()` feature-value name, or `None` when it was not written.
    #[must_use]
    pub fn ornaments(&self) -> Option<&CssFontFeatureValueName> {
        self.ornaments.as_ref()
    }
    /// Returns the `annotation()` feature-value name, or `None` when it was not written.
    #[must_use]
    pub fn annotation(&self) -> Option<&CssFontFeatureValueName> {
        self.annotation.as_ref()
    }
}

/// The authored `font-variant-alternates` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantAlternates {
    /// `normal`: none of the alternate features are enabled.
    Normal,
    /// One or more explicit alternate-feature choices.
    Values(CssFontVariantAlternateValues),
}

/// The authored `font-variant-emoji` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEmoji {
    /// `normal`: the user agent can choose to draw an emoji presentation participating code point in either emoji or text style, typically following platform conventions.
    Normal,
    /// `text`: code points are rendered as if U+FE0E VARIATION SELECTOR-15 was appended to every emoji presentation participating code point.
    Text,
    /// `emoji`: code points are rendered as if U+FE0F VARIATION SELECTOR-16 was appended to every emoji presentation participating code point.
    Emoji,
    /// `unicode`: code points are rendered as UTS #51 specifies, as emoji-default, text-default or text-only, depending on their Emoji and Emoji_Presentation properties.
    Unicode,
}

/// A nonempty checked compatible union of `font-variant` component groups.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariantValues {
    ligatures: Option<CssFontVariantLigatureValues>,
    position: Option<CssFontVariantPosition>,
    caps: Option<CssFontVariantCaps>,
    numeric: Option<CssFontVariantNumericValues>,
    east_asian: Option<CssFontVariantEastAsianValues>,
    alternates: Option<CssFontVariantAlternateValues>,
    emoji: Option<CssFontVariantEmoji>,
}

impl CssFontVariantValues {
    /// Checks the combination of `font-variant` component groups.
    ///
    /// Returns `None` when every group is omitted, or when `position`, `caps` or `emoji` is the `normal` keyword, which the shorthand grammar admits only as the whole-value `normal`.
    #[must_use]
    pub fn try_new(
        ligatures: Option<CssFontVariantLigatureValues>,
        position: Option<CssFontVariantPosition>,
        caps: Option<CssFontVariantCaps>,
        numeric: Option<CssFontVariantNumericValues>,
        east_asian: Option<CssFontVariantEastAsianValues>,
        alternates: Option<CssFontVariantAlternateValues>,
        emoji: Option<CssFontVariantEmoji>,
    ) -> Option<Self> {
        if matches!(position, Some(CssFontVariantPosition::Normal))
            || matches!(caps, Some(CssFontVariantCaps::Normal))
            || matches!(emoji, Some(CssFontVariantEmoji::Normal))
            || (ligatures.is_none()
                && position.is_none()
                && caps.is_none()
                && numeric.is_none()
                && east_asian.is_none()
                && alternates.is_none()
                && emoji.is_none())
        {
            None
        } else {
            Some(Self {
                ligatures,
                position,
                caps,
                numeric,
                east_asian,
                alternates,
                emoji,
            })
        }
    }

    /// Returns the ligature choices, or `None` when they were omitted.
    #[must_use]
    pub const fn ligatures(&self) -> Option<&CssFontVariantLigatureValues> {
        self.ligatures.as_ref()
    }

    /// Returns the position keyword, or `None` when it was omitted.
    #[must_use]
    pub const fn position(&self) -> Option<CssFontVariantPosition> {
        self.position
    }

    /// Returns the capitalization keyword, or `None` when it was omitted.
    #[must_use]
    pub const fn caps(&self) -> Option<CssFontVariantCaps> {
        self.caps
    }

    /// Returns the numeric-variant choices, or `None` when they were omitted.
    #[must_use]
    pub const fn numeric(&self) -> Option<&CssFontVariantNumericValues> {
        self.numeric.as_ref()
    }

    /// Returns the East Asian variant choices, or `None` when they were omitted.
    #[must_use]
    pub const fn east_asian(&self) -> Option<&CssFontVariantEastAsianValues> {
        self.east_asian.as_ref()
    }

    /// Returns the alternate-feature choices, or `None` when they were omitted.
    #[must_use]
    pub const fn alternates(&self) -> Option<&CssFontVariantAlternateValues> {
        self.alternates.as_ref()
    }
    /// Returns the emoji keyword, or `None` when it was omitted.
    #[must_use]
    pub const fn emoji(&self) -> Option<CssFontVariantEmoji> {
        self.emoji
    }
}

/// The checked current authored `font-variant` shorthand value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantValue {
    /// `normal`: every `font-variant` longhand takes its initial value.
    Normal,
    /// `none`: ligatures are disabled and the other `font-variant` longhands take their initial values.
    None,
    /// One or more explicit component groups.
    Values(CssFontVariantValues),
}

impl CssFontVariantValue {
    pub(crate) fn expanded_ligatures(&self) -> CssFontVariantLigatures {
        match self {
            Self::None => CssFontVariantLigatures::None,
            Self::Values(value) => value.ligatures.map_or(
                CssFontVariantLigatures::Normal,
                CssFontVariantLigatures::Values,
            ),
            Self::Normal => CssFontVariantLigatures::Normal,
        }
    }
    pub(crate) fn expanded_caps(&self) -> CssFontVariantCaps {
        match self {
            Self::Values(value) => value.caps.unwrap_or(CssFontVariantCaps::Normal),
            _ => CssFontVariantCaps::Normal,
        }
    }
    pub(crate) fn expanded_alternates(&self) -> CssFontVariantAlternates {
        match self {
            Self::Values(value) => value.alternates.clone().map_or(
                CssFontVariantAlternates::Normal,
                CssFontVariantAlternates::Values,
            ),
            _ => CssFontVariantAlternates::Normal,
        }
    }
    pub(crate) fn expanded_numeric(&self) -> CssFontVariantNumeric {
        match self {
            Self::Values(value) => value
                .numeric
                .map_or(CssFontVariantNumeric::Normal, CssFontVariantNumeric::Values),
            _ => CssFontVariantNumeric::Normal,
        }
    }
    pub(crate) fn expanded_east_asian(&self) -> CssFontVariantEastAsian {
        match self {
            Self::Values(value) => value.east_asian.map_or(
                CssFontVariantEastAsian::Normal,
                CssFontVariantEastAsian::Values,
            ),
            _ => CssFontVariantEastAsian::Normal,
        }
    }
    pub(crate) fn expanded_position(&self) -> CssFontVariantPosition {
        match self {
            Self::Values(value) => value.position.unwrap_or(CssFontVariantPosition::Normal),
            _ => CssFontVariantPosition::Normal,
        }
    }
    pub(crate) fn expanded_emoji(&self) -> CssFontVariantEmoji {
        match self {
            Self::Values(value) => value.emoji.unwrap_or(CssFontVariantEmoji::Normal),
            _ => CssFontVariantEmoji::Normal,
        }
    }
}
