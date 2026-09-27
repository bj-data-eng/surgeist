//! Authored font-variant family and intrinsic checked values.

use crate::CssFontFeatureValueName;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariant {
    Normal,
    SmallCaps,
}

/// The current authored `font-variant-caps` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantCaps {
    Normal,
    SmallCaps,
    AllSmallCaps,
    PetiteCaps,
    AllPetiteCaps,
    Unicase,
    TitlingCaps,
}

/// Whether one authored ligature group is enabled or disabled.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatureState {
    Enabled,
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

    #[must_use]
    pub const fn common(self) -> Option<CssFontVariantLigatureState> {
        self.common
    }

    #[must_use]
    pub const fn discretionary(self) -> Option<CssFontVariantLigatureState> {
        self.discretionary
    }

    #[must_use]
    pub const fn historical(self) -> Option<CssFontVariantLigatureState> {
        self.historical
    }

    #[must_use]
    pub const fn contextual(self) -> Option<CssFontVariantLigatureState> {
        self.contextual
    }
}

/// The current authored `font-variant-ligatures` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantLigatures {
    Normal,
    None,
    Values(CssFontVariantLigatureValues),
}

/// The figure form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFigure {
    LiningNums,
    OldstyleNums,
}

/// The figure spacing selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericSpacing {
    ProportionalNums,
    TabularNums,
}

/// The fraction form selected by `font-variant-numeric`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumericFraction {
    DiagonalFractions,
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

    #[must_use]
    pub const fn figure(self) -> Option<CssFontVariantNumericFigure> {
        self.figure
    }

    #[must_use]
    pub const fn spacing(self) -> Option<CssFontVariantNumericSpacing> {
        self.spacing
    }

    #[must_use]
    pub const fn fraction(self) -> Option<CssFontVariantNumericFraction> {
        self.fraction
    }

    #[must_use]
    pub const fn ordinal(self) -> bool {
        self.ordinal
    }

    #[must_use]
    pub const fn slashed_zero(self) -> bool {
        self.slashed_zero
    }
}

/// The current authored `font-variant-numeric` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantNumeric {
    Normal,
    Values(CssFontVariantNumericValues),
}

/// The regional glyph form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianVariant {
    Jis78,
    Jis83,
    Jis90,
    Jis04,
    Simplified,
    Traditional,
}

/// The glyph-width form selected by `font-variant-east-asian`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsianWidth {
    FullWidth,
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

    #[must_use]
    pub const fn variant(self) -> Option<CssFontVariantEastAsianVariant> {
        self.variant
    }

    #[must_use]
    pub const fn width(self) -> Option<CssFontVariantEastAsianWidth> {
        self.width
    }

    #[must_use]
    pub const fn ruby(self) -> bool {
        self.ruby
    }
}

/// The current authored `font-variant-east-asian` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEastAsian {
    Normal,
    Values(CssFontVariantEastAsianValues),
}

/// The current authored `font-variant-position` value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantPosition {
    Normal,
    Sub,
    Super,
}

/// A checked nonempty ordered list of decoded feature-value names.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssFontVariantAlternateNames {
    names: Vec<CssFontFeatureValueName>,
}

impl CssFontVariantAlternateNames {
    #[must_use]
    pub fn try_new(names: Vec<CssFontFeatureValueName>) -> Option<Self> {
        (!names.is_empty()).then_some(Self { names })
    }

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

    #[must_use]
    pub fn stylistic(&self) -> Option<&CssFontFeatureValueName> {
        self.stylistic.as_ref()
    }
    #[must_use]
    pub fn historical_forms(&self) -> bool {
        self.historical_forms
    }
    #[must_use]
    pub fn styleset(&self) -> Option<&CssFontVariantAlternateNames> {
        self.styleset.as_ref()
    }
    #[must_use]
    pub fn character_variant(&self) -> Option<&CssFontVariantAlternateNames> {
        self.character_variant.as_ref()
    }
    #[must_use]
    pub fn swash(&self) -> Option<&CssFontFeatureValueName> {
        self.swash.as_ref()
    }
    #[must_use]
    pub fn ornaments(&self) -> Option<&CssFontFeatureValueName> {
        self.ornaments.as_ref()
    }
    #[must_use]
    pub fn annotation(&self) -> Option<&CssFontFeatureValueName> {
        self.annotation.as_ref()
    }
}

/// The authored `font-variant-alternates` value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantAlternates {
    Normal,
    Values(CssFontVariantAlternateValues),
}

/// The authored `font-variant-emoji` keyword.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantEmoji {
    Normal,
    Text,
    Emoji,
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

    #[must_use]
    pub const fn ligatures(&self) -> Option<&CssFontVariantLigatureValues> {
        self.ligatures.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<CssFontVariantPosition> {
        self.position
    }

    #[must_use]
    pub const fn caps(&self) -> Option<CssFontVariantCaps> {
        self.caps
    }

    #[must_use]
    pub const fn numeric(&self) -> Option<&CssFontVariantNumericValues> {
        self.numeric.as_ref()
    }

    #[must_use]
    pub const fn east_asian(&self) -> Option<&CssFontVariantEastAsianValues> {
        self.east_asian.as_ref()
    }

    #[must_use]
    pub const fn alternates(&self) -> Option<&CssFontVariantAlternateValues> {
        self.alternates.as_ref()
    }
    #[must_use]
    pub const fn emoji(&self) -> Option<CssFontVariantEmoji> {
        self.emoji
    }
}

/// The checked current authored `font-variant` shorthand value.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFontVariantValue {
    Normal,
    None,
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
