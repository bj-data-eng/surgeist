use super::{
    CssAngleCalculation, CssAngleLiteral, CssAuthoredDeclarationValue, CssCalculationExpression,
    CssCalculationExpressionRef, CssCalculationType, CssColorAngleLiteral, CssColorNumberLiteral,
    CssColorPercentageLiteral, CssColorScalarError, CssFiniteNumber, CssNumberCalculation,
    CssPercentageCalculation, CssVariableReference,
};

mod serialization;

/// An authored color retaining its specified syntax and symbolic dependencies.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColor {
    representation: CssAuthoredColorRepresentation,
}

#[derive(Clone, Debug, PartialEq)]
enum CssAuthoredColorRepresentation {
    CurrentColor,
    Transparent,
    Hex(CssHexColor),
    Named(CssNamedColor),
    System(CssAuthoredSystemColor),
    Rgb(CssAuthoredRgbColor),
    Hsl(CssAuthoredHslColor),
    Hwb(CssAuthoredHwbColor),
    Lab(CssAuthoredLabColor),
    Lch(CssAuthoredLchColor),
    Oklab(CssAuthoredLabColor),
    Oklch(CssAuthoredLchColor),
    Predefined(CssAuthoredPredefinedColor),
    Custom(CssAuthoredCustomColor),
    // Keep retained profile-expression metadata out of every inline color value.
    RelativeCustom(Box<CssAuthoredRelativeCustomColor>),
    Alpha(CssAuthoredAlphaColor),
    Relative(CssAuthoredRelativeColor),
    ColorMix(CssAuthoredColorMix),
    PreservedI01(CssColor),
}

impl CssAuthoredColor {
    /// Produces canonical specified color text without resolving external color context.
    pub fn to_specified_css(&self) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        self.to_specified_css_with_limits(crate::CssSpecifiedValueSerializationLimits::default())
    }

    /// Produces canonical specified color text under cumulative resource limits.
    pub fn to_specified_css_with_limits(
        &self,
        limits: crate::CssSpecifiedValueSerializationLimits,
    ) -> Result<String, crate::CssSpecifiedValueSerializationError> {
        serialization::serialize(self, limits)
    }

    pub const fn from_custom(value: CssAuthoredCustomColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Custom(value),
        }
    }
    pub fn from_relative_custom(value: CssAuthoredRelativeCustomColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::RelativeCustom(Box::new(value)),
        }
    }
    pub const fn from_alpha(value: CssAuthoredAlphaColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Alpha(value),
        }
    }
    pub const fn custom_value(&self) -> Option<&CssAuthoredCustomColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Custom(v) => Some(v),
            _ => None,
        }
    }
    pub const fn relative_custom_value(&self) -> Option<&CssAuthoredRelativeCustomColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::RelativeCustom(v) => Some(v),
            _ => None,
        }
    }
    pub const fn alpha_value(&self) -> Option<&CssAuthoredAlphaColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Alpha(v) => Some(v),
            _ => None,
        }
    }

    pub(crate) const fn current_color() -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::CurrentColor,
        }
    }

    pub(crate) const fn transparent() -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Transparent,
        }
    }

    pub(crate) const fn hex(value: CssHexColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hex(value),
        }
    }

    pub(crate) const fn from_named(value: CssNamedColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Named(value),
        }
    }

    pub(crate) const fn from_system(value: CssAuthoredSystemColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::System(value),
        }
    }

    pub(crate) const fn rgb(value: CssAuthoredRgbColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Rgb(value),
        }
    }

    pub(crate) const fn hsl(value: CssAuthoredHslColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hsl(value),
        }
    }

    pub(crate) const fn hwb(value: CssAuthoredHwbColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Hwb(value),
        }
    }

    pub(crate) const fn lab(value: CssAuthoredLabColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Lab(value),
        }
    }

    pub(crate) const fn lch(value: CssAuthoredLchColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Lch(value),
        }
    }

    pub(crate) const fn oklab(value: CssAuthoredLabColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Oklab(value),
        }
    }

    pub(crate) const fn oklch(value: CssAuthoredLchColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Oklch(value),
        }
    }

    pub(crate) const fn predefined(value: CssAuthoredPredefinedColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Predefined(value),
        }
    }

    pub(crate) const fn relative(value: CssAuthoredRelativeColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::Relative(value),
        }
    }

    pub(crate) const fn color_mix(value: CssAuthoredColorMix) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::ColorMix(value),
        }
    }

    pub(crate) const fn preserved_i01(value: CssColor) -> Self {
        Self {
            representation: CssAuthoredColorRepresentation::PreservedI01(value),
        }
    }

    #[must_use]
    pub const fn is_current_color(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredColorRepresentation::CurrentColor
        )
    }

    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        matches!(
            self.representation,
            CssAuthoredColorRepresentation::Transparent
        )
    }

    #[must_use]
    pub const fn hex_value(&self) -> Option<&CssHexColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hex(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn named(&self) -> Option<&CssNamedColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Named(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn system(&self) -> Option<CssAuthoredSystemColor> {
        match self.representation {
            CssAuthoredColorRepresentation::System(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn rgb_value(&self) -> Option<&CssAuthoredRgbColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Rgb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hsl_value(&self) -> Option<&CssAuthoredHslColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hsl(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hwb_value(&self) -> Option<&CssAuthoredHwbColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Hwb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lab_value(&self) -> Option<&CssAuthoredLabColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Lab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lch_value(&self) -> Option<&CssAuthoredLchColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Lch(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklab_value(&self) -> Option<&CssAuthoredLabColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Oklab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklch_value(&self) -> Option<&CssAuthoredLchColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Oklch(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn predefined_value(&self) -> Option<&CssAuthoredPredefinedColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Predefined(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the typed preserved Color 5 relative-color branch, when present.
    #[must_use]
    pub const fn relative_value(&self) -> Option<&CssAuthoredRelativeColor> {
        match &self.representation {
            CssAuthoredColorRepresentation::Relative(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the checked preserved Color 5 `color-mix()` branch, when present.
    #[must_use]
    pub const fn color_mix_value(&self) -> Option<&CssAuthoredColorMix> {
        match &self.representation {
            CssAuthoredColorRepresentation::ColorMix(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match &self.representation {
            CssAuthoredColorRepresentation::CurrentColor => "currentcolor",
            CssAuthoredColorRepresentation::Transparent => "transparent",
            CssAuthoredColorRepresentation::Hex(_) => "hex",
            CssAuthoredColorRepresentation::Named(_) => "named",
            CssAuthoredColorRepresentation::System(_) => "system",
            CssAuthoredColorRepresentation::Rgb(_) => "rgb",
            CssAuthoredColorRepresentation::Hsl(_) => "hsl",
            CssAuthoredColorRepresentation::Hwb(_) => "hwb",
            CssAuthoredColorRepresentation::Lab(_) => "lab",
            CssAuthoredColorRepresentation::Lch(_) => "lch",
            CssAuthoredColorRepresentation::Oklab(_) => "oklab",
            CssAuthoredColorRepresentation::Oklch(_) => "oklch",
            CssAuthoredColorRepresentation::Custom(_) => "color",
            CssAuthoredColorRepresentation::RelativeCustom(_) => "relative",
            CssAuthoredColorRepresentation::Alpha(_) => "alpha",
            CssAuthoredColorRepresentation::Predefined(_) => "color",
            CssAuthoredColorRepresentation::Relative(_) => "relative",
            CssAuthoredColorRepresentation::ColorMix(_) => "color-mix",
            CssAuthoredColorRepresentation::PreservedI01(value) => value.kind_name(),
        }
    }

    /// Proves the actual candidate payload, including its scale, before pairing.
    pub(crate) fn matches_i01(&self, candidate: &CssColor) -> bool {
        use crate::color_scalar::{alpha_matches, channel_matches, hue_matches, relative_matches};
        let mut pending = vec![(self, candidate)];
        while let Some((current, candidate)) = pending.pop() {
            let valid = match (&current.representation, candidate) {
                (CssAuthoredColorRepresentation::PreservedI01(value), candidate) => {
                    value == candidate
                }
                (CssAuthoredColorRepresentation::CurrentColor, CssColor::CurrentColor) => true,
                (CssAuthoredColorRepresentation::Transparent, candidate) => {
                    *candidate == CssColor::TRANSPARENT
                }
                (
                    CssAuthoredColorRepresentation::Hex(_)
                    | CssAuthoredColorRepresentation::Named(_),
                    CssColor::Rgba(_),
                ) => true,
                (CssAuthoredColorRepresentation::System(_), CssColor::System(_)) => true,
                (CssAuthoredColorRepresentation::Rgb(value), CssColor::Rgba(candidate)) => {
                    value
                        .channels()
                        .iter()
                        .zip([candidate.red(), candidate.green(), candidate.blue()])
                        .all(|(channel, byte)| {
                            channel_matches(channel, Some(f32::from(byte)), 255, 100)
                        })
                        && alpha_matches(value.alpha(), Some(candidate.alpha()))
                }
                (CssAuthoredColorRepresentation::Hsl(value), CssColor::Hsl(candidate)) => {
                    hue_matches(value.hue(), candidate.hue())
                        && channel_matches(value.saturation(), candidate.saturation(), 1, 100)
                        && channel_matches(value.lightness(), candidate.lightness(), 1, 100)
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (CssAuthoredColorRepresentation::Hwb(value), CssColor::Hwb(candidate)) => {
                    hue_matches(value.hue(), candidate.hue())
                        && channel_matches(value.whiteness(), candidate.whiteness(), 1, 100)
                        && channel_matches(value.blackness(), candidate.blackness(), 1, 100)
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (CssAuthoredColorRepresentation::Lab(value), CssColor::Lab(candidate)) => {
                    channel_matches(value.lightness(), candidate.lightness(), 1, 1)
                        && channel_matches(value.a(), candidate.a(), 5, 4)
                        && channel_matches(value.b(), candidate.b(), 5, 4)
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (CssAuthoredColorRepresentation::Oklab(value), CssColor::Oklab(candidate)) => {
                    channel_matches(value.lightness(), candidate.lightness(), 1, 100)
                        && channel_matches(value.a(), candidate.a(), 1, 250)
                        && channel_matches(value.b(), candidate.b(), 1, 250)
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (CssAuthoredColorRepresentation::Lch(value), CssColor::Lch(candidate)) => {
                    channel_matches(value.lightness(), candidate.lightness(), 1, 1)
                        && channel_matches(value.chroma(), candidate.chroma(), 3, 2)
                        && hue_matches(value.hue(), candidate.hue())
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (CssAuthoredColorRepresentation::Oklch(value), CssColor::Oklch(candidate)) => {
                    channel_matches(value.lightness(), candidate.lightness(), 1, 100)
                        && channel_matches(value.chroma(), candidate.chroma(), 1, 250)
                        && hue_matches(value.hue(), candidate.hue())
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (
                    CssAuthoredColorRepresentation::Predefined(value),
                    CssColor::ColorFunction(candidate),
                ) => {
                    value.color_space() == candidate.color_space()
                        && value
                            .channels()
                            .iter()
                            .zip(candidate.components())
                            .all(|(value, candidate)| channel_matches(value, *candidate, 1, 100))
                        && alpha_matches(value.alpha(), candidate.alpha())
                }
                (
                    CssAuthoredColorRepresentation::ColorMix(value),
                    CssColor::ColorMix(candidate),
                ) => {
                    let weights_match =
                        |current: &CssAuthoredColorMixComponent,
                         candidate: &CssColorMixComponent| {
                            match (current.weight(), candidate.percentage()) {
                                (None, None) => true,
                                (Some(current), Some(candidate)) => current
                                    .literal_value()
                                    .is_some_and(|value| value.value() == Some(candidate)),
                                _ => false,
                            }
                        };
                    let [left, right] = value.components() else {
                        return false;
                    };
                    pending.push((left.color(), candidate.left().color()));
                    pending.push((right.color(), candidate.right().color()));
                    value
                        .interpolation()
                        .and_then(CssAuthoredColorInterpolation::predefined)
                        .as_ref()
                        == Some(candidate.interpolation())
                        && weights_match(left, candidate.left())
                        && weights_match(right, candidate.right())
                }
                (
                    CssAuthoredColorRepresentation::Relative(value),
                    CssColor::Relative(candidate),
                ) => {
                    pending.push((value.source(), candidate.source()));
                    value.function() == candidate.function()
                        && value.channels().len() == candidate.components().len()
                        && value
                            .channels()
                            .iter()
                            .zip(candidate.components())
                            .all(|(value, candidate)| relative_matches(value, candidate))
                        && match (value.alpha(), candidate.alpha()) {
                            (None, None) => true,
                            (Some(value), Some(candidate)) => relative_matches(value, candidate),
                            _ => false,
                        }
                }
                _ => false,
            };
            if !valid {
                return false;
            }
        }
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssHexColor {
    digits: String,
}

impl CssHexColor {
    pub(crate) fn new(digits: impl Into<String>) -> Self {
        Self {
            digits: digits.into(),
        }
    }

    #[must_use]
    pub fn digits(&self) -> &str {
        &self.digits
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamedColor {
    name: String,
}

impl CssNamedColor {
    pub(crate) fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredSystemColor {
    Canvas,
    CanvasText,
    LinkText,
    VisitedText,
    ActiveText,
    ButtonFace,
    ButtonText,
    ButtonBorder,
    Field,
    FieldText,
    Highlight,
    HighlightText,
    Mark,
    MarkText,
    GrayText,
    SelectedItem,
    SelectedItemText,
    AccentColor,
    AccentColorText,
    ActiveBorder,
    ActiveCaption,
    AppWorkspace,
    Background,
    ButtonHighlight,
    ButtonShadow,
    CaptionText,
    InactiveBorder,
    InactiveCaption,
    InactiveCaptionText,
    InfoBackground,
    InfoText,
    Menu,
    MenuText,
    Scrollbar,
    ThreeDDarkShadow,
    ThreeDFace,
    ThreeDHighlight,
    ThreeDLightShadow,
    ThreeDShadow,
    Window,
    WindowFrame,
    WindowText,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredColorSyntax {
    Legacy,
    Modern,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredColorComponent {
    ExactNumber(CssColorNumberLiteral),
    ExactPercentage(CssColorPercentageLiteral),
    None,
    Number(CssFiniteNumber),
    Percentage(CssFiniteNumber),
    NumberCalculation(CssNumberCalculation),
    PercentageCalculation(CssPercentageCalculation),
}

impl CssAuthoredColorComponent {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) const fn domain(&self) -> Option<CssCalculationType> {
        match self {
            Self::None => None,
            Self::Number(_) | Self::ExactNumber(_) | Self::NumberCalculation(_) => {
                Some(CssCalculationType::Number)
            }
            Self::Percentage(_) | Self::ExactPercentage(_) | Self::PercentageCalculation(_) => {
                Some(CssCalculationType::Percentage)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredHue {
    ExactNumber(CssColorNumberLiteral),
    ExactAngle(CssColorAngleLiteral),
    None,
    Number(CssFiniteNumber),
    Angle(CssAngleLiteral),
    NumberCalculation(CssNumberCalculation),
    AngleCalculation(CssAngleCalculation),
}

impl CssAuthoredHue {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredRgbColor {
    syntax: CssAuthoredColorSyntax,
    channels: [CssAuthoredColorComponent; 3],
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredRgbColor {
    pub(crate) const fn new(
        syntax: CssAuthoredColorSyntax,
        channels: [CssAuthoredColorComponent; 3],
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            syntax,
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn syntax(&self) -> CssAuthoredColorSyntax {
        self.syntax
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssAuthoredColorComponent; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredHslColor {
    syntax: CssAuthoredColorSyntax,
    hue: CssAuthoredHue,
    saturation: CssAuthoredColorComponent,
    lightness: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredHslColor {
    pub(crate) const fn new(
        syntax: CssAuthoredColorSyntax,
        hue: CssAuthoredHue,
        saturation: CssAuthoredColorComponent,
        lightness: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            syntax,
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    #[must_use]
    pub const fn syntax(&self) -> CssAuthoredColorSyntax {
        self.syntax
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn saturation(&self) -> &CssAuthoredColorComponent {
        &self.saturation
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredHwbColor {
    hue: CssAuthoredHue,
    whiteness: CssAuthoredColorComponent,
    blackness: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredHwbColor {
    pub(crate) const fn new(
        hue: CssAuthoredHue,
        whiteness: CssAuthoredColorComponent,
        blackness: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            hue,
            whiteness,
            blackness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn whiteness(&self) -> &CssAuthoredColorComponent {
        &self.whiteness
    }

    #[must_use]
    pub const fn blackness(&self) -> &CssAuthoredColorComponent {
        &self.blackness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A parser-owned authored Lab-family color with exact channel kinds.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredLabColor {
    lightness: CssAuthoredColorComponent,
    a: CssAuthoredColorComponent,
    b: CssAuthoredColorComponent,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredLabColor {
    pub(crate) const fn new(
        lightness: CssAuthoredColorComponent,
        a: CssAuthoredColorComponent,
        b: CssAuthoredColorComponent,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn a(&self) -> &CssAuthoredColorComponent {
        &self.a
    }

    #[must_use]
    pub const fn b(&self) -> &CssAuthoredColorComponent {
        &self.b
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A parser-owned authored LCH-family color with an angle-capable hue.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredLchColor {
    lightness: CssAuthoredColorComponent,
    chroma: CssAuthoredColorComponent,
    hue: CssAuthoredHue,
    alpha: Option<CssAuthoredColorComponent>,
}

impl CssAuthoredLchColor {
    pub(crate) const fn new(
        lightness: CssAuthoredColorComponent,
        chroma: CssAuthoredColorComponent,
        hue: CssAuthoredHue,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssAuthoredColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn chroma(&self) -> &CssAuthoredColorComponent {
        &self.chroma
    }

    #[must_use]
    pub const fn hue(&self) -> &CssAuthoredHue {
        &self.hue
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// Decoded custom-profile component name. Binding belongs to profile resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileComponentName(String);
impl CssColorProfileComponentName {
    pub fn try_new(decoded: impl Into<String>) -> Option<Self> {
        let decoded = decoded.into();
        if decoded.eq_ignore_ascii_case("none") {
            return None;
        }
        crate::CssComponentValue::try_ident(decoded.clone()).ok()?;
        Some(Self(decoded))
    }
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// Intrinsic failures while composing an authored color graph.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAuthoredColorConstructionError {
    EmptyComponents,
    NestingLimit,
    CapacityOverflow,
    InvalidExpressionEnvironment,
}
impl std::fmt::Display for CssAuthoredColorConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyComponents => "color requires at least one component",
            Self::NestingLimit => "color nesting limit exceeded",
            Self::CapacityOverflow => "color capacity overflow",
            Self::InvalidExpressionEnvironment => "color expression environment mismatch",
        })
    }
}
impl std::error::Error for CssAuthoredColorConstructionError {}
impl From<ColorGraphDepthError> for CssAuthoredColorConstructionError {
    fn from(value: ColorGraphDepthError) -> Self {
        match value {
            ColorGraphDepthError::NestingLimit => Self::NestingLimit,
            ColorGraphDepthError::CapacityOverflow => Self::CapacityOverflow,
        }
    }
}
fn composed_color_depth(child_depth: u32) -> Result<u32, CssAuthoredColorConstructionError> {
    let depth = child_depth
        .checked_add(1)
        .ok_or(CssAuthoredColorConstructionError::CapacityOverflow)?;
    if depth > crate::STRUCTURAL_NESTING_LIMIT {
        return Err(CssAuthoredColorConstructionError::NestingLimit);
    }
    Ok(depth)
}
/// A nonempty authored custom `color()` without profile binding or evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredCustomColor {
    profile: CssColorProfileName,
    channels: Vec<CssAuthoredColorComponent>,
    alpha: Option<CssAuthoredColorComponent>,
    nesting_depth: u32,
}
impl CssAuthoredCustomColor {
    pub fn try_new(
        profile: CssColorProfileName,
        channels: Vec<CssAuthoredColorComponent>,
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Result<Self, CssAuthoredColorConstructionError> {
        if channels.is_empty() {
            return Err(CssAuthoredColorConstructionError::EmptyComponents);
        }
        let nesting_depth = composed_color_depth(
            channels
                .iter()
                .chain(alpha.iter())
                .map(color_component_depth)
                .max()
                .unwrap_or(0),
        )?;
        Ok(Self {
            profile,
            channels,
            alpha,
            nesting_depth,
        })
    }
    pub const fn profile(&self) -> &CssColorProfileName {
        &self.profile
    }
    pub fn channels(&self) -> &[CssAuthoredColorComponent] {
        &self.channels
    }
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}
/// A custom relative color retaining unbound profile references in authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredRelativeCustomColor {
    source: Box<CssAuthoredColor>,
    profile: CssColorProfileName,
    channels: Vec<crate::CssProfileColorExpression>,
    alpha: Option<crate::CssProfileColorExpression>,
    nesting_depth: u32,
}
impl CssAuthoredRelativeCustomColor {
    pub fn try_new(
        source: CssAuthoredColor,
        profile: CssColorProfileName,
        channels: Vec<crate::CssProfileColorExpression>,
        alpha: Option<crate::CssProfileColorExpression>,
    ) -> Result<Self, CssAuthoredColorConstructionError> {
        if channels.is_empty() {
            return Err(CssAuthoredColorConstructionError::EmptyComponents);
        }
        let expressions = channels
            .iter()
            .chain(alpha.iter())
            .map(|v| v.components().nesting_depth())
            .max()
            .unwrap_or(0);
        let nesting_depth = composed_color_depth(expressions.max(authored_color_depth(&source)?))?;
        Ok(Self {
            source: Box::new(source),
            profile,
            channels,
            alpha,
            nesting_depth,
        })
    }
    pub const fn source(&self) -> &CssAuthoredColor {
        &self.source
    }
    pub const fn profile(&self) -> &CssColorProfileName {
        &self.profile
    }
    pub fn channels(&self) -> &[crate::CssProfileColorExpression] {
        &self.channels
    }
    pub const fn alpha(&self) -> Option<&crate::CssProfileColorExpression> {
        self.alpha.as_ref()
    }
}
/// Authored `alpha(from ...)`, preserving omission independently from `none`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredAlphaColor {
    source: Box<CssAuthoredColor>,
    alpha: Option<CssTypedRelativeColorExpression>,
    nesting_depth: u32,
}
impl CssAuthoredAlphaColor {
    pub fn try_new(
        source: CssAuthoredColor,
        alpha: Option<CssTypedRelativeColorExpression>,
    ) -> Result<Self, CssAuthoredColorConstructionError> {
        if alpha.as_ref().is_some_and(|v| {
            v.environment() != CssRelativeColorEnvironment::Alpha
                || v.result_domain() != CssRelativeColorResultDomain::Alpha
        }) {
            return Err(CssAuthoredColorConstructionError::InvalidExpressionEnvironment);
        }
        let expression_depth = match alpha.as_ref().map(CssTypedRelativeColorExpression::value) {
            Some(CssRelativeColorExpressionValue::Calculation(v)) => {
                v.data.expression.component_nesting_depth()
            }
            _ => 0,
        };
        let nesting_depth =
            composed_color_depth(expression_depth.max(authored_color_depth(&source)?))?;
        Ok(Self {
            source: Box::new(source),
            alpha,
            nesting_depth,
        })
    }
    pub const fn source(&self) -> &CssAuthoredColor {
        &self.source
    }
    pub const fn alpha(&self) -> Option<&CssTypedRelativeColorExpression> {
        self.alpha.as_ref()
    }
}
/// Authored eligibility only; this does not promise a usable computed color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteColorEligibility {
    Eligible,
    ProfileDependent,
    Contextual(CssAbsoluteColorExclusion),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteColorExclusion {
    CurrentColor,
    SystemColor,
    LightDark,
    ContrastColor,
    DeviceCmyk,
}
impl CssAuthoredColor {
    pub fn absolute_eligibility(&self) -> CssAbsoluteColorEligibility {
        enum Pending<'a> {
            Current(&'a CssAuthoredColor),
            Frozen(&'a CssColor),
        }
        use CssAbsoluteColorEligibility as E;
        use CssAbsoluteColorExclusion as X;
        use CssAuthoredColorRepresentation as R;
        let mut pending = vec![Pending::Current(self)];
        let mut profile = false;
        while let Some(next) = pending.pop() {
            match next {
                Pending::Current(color) => match &color.representation {
                    R::CurrentColor => return E::Contextual(X::CurrentColor),
                    R::System(_) => return E::Contextual(X::SystemColor),
                    R::Custom(_) => profile = true,
                    R::RelativeCustom(v) => {
                        profile = true;
                        pending.push(Pending::Current(v.source()));
                    }
                    R::Alpha(v) => pending.push(Pending::Current(v.source())),
                    R::Relative(v) => pending.push(Pending::Current(v.source())),
                    R::ColorMix(v) => {
                        profile |= v
                            .interpolation()
                            .is_some_and(|v| v.custom_profile().is_some());
                        pending.extend(
                            v.components()
                                .iter()
                                .rev()
                                .map(|v| Pending::Current(v.color())),
                        );
                    }
                    R::PreservedI01(v) => pending.push(Pending::Frozen(v)),
                    R::Transparent
                    | R::Hex(_)
                    | R::Named(_)
                    | R::Rgb(_)
                    | R::Hsl(_)
                    | R::Hwb(_)
                    | R::Lab(_)
                    | R::Lch(_)
                    | R::Oklab(_)
                    | R::Oklch(_)
                    | R::Predefined(_) => {}
                },
                Pending::Frozen(color) => match color {
                    CssColor::CurrentColor => return E::Contextual(X::CurrentColor),
                    CssColor::System(_) => return E::Contextual(X::SystemColor),
                    CssColor::Relative(v) => pending.push(Pending::Frozen(v.source())),
                    CssColor::ColorMix(v) => {
                        pending.push(Pending::Frozen(v.right().color()));
                        pending.push(Pending::Frozen(v.left().color()));
                    }
                    CssColor::Rgba(_)
                    | CssColor::Hsl(_)
                    | CssColor::Hwb(_)
                    | CssColor::Lab(_)
                    | CssColor::Lch(_)
                    | CssColor::Oklab(_)
                    | CssColor::Oklch(_)
                    | CssColor::ColorFunction(_) => {}
                },
            }
        }
        if profile {
            E::ProfileDependent
        } else {
            E::Eligible
        }
    }
}

/// A parser-owned absolute `color()` value in a predefined Color 4 space.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredPredefinedColor {
    color_space: CssPredefinedColorSpace,
    channels: [CssAuthoredColorComponent; 3],
    alpha: Option<CssAuthoredColorComponent>,
}

/// The closed origin-channel environment for a preserved relative-color family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorEnvironment {
    Alpha,
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    PredefinedRgb(CssPredefinedColorSpace),
    Xyz(CssPredefinedColorSpace),
}

/// The semantic result slot in which a relative-color expression is authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorResultDomain {
    NumberPercentage,
    Hue,
    Alpha,
}

/// A channel name made available by one relative-color origin environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorChannel {
    R,
    G,
    B,
    H,
    S,
    L,
    W,
    A,
    C,
    X,
    Y,
    Z,
    Alpha,
}

/// The authored kind of one validated relative-color result expression.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorExpressionValue {
    ExactNumber(CssColorNumberLiteral),
    ExactPercentage(CssColorPercentageLiteral),
    ExactAngle(CssColorAngleLiteral),
    None,
    Number(CssFiniteNumber),
    Percentage(CssFiniteNumber),
    Angle(CssAngleLiteral),
    Channel(CssRelativeColorChannel),
    Calculation(CssRelativeColorCalculation),
}

/// A validated relative-color calculation retained symbolically without evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssRelativeColorCalculation {
    data: Box<CssRelativeColorCalculationData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CssRelativeColorCalculationData {
    authored: CssAuthoredDeclarationValue,
    result_type: CssCalculationType,
    references: Vec<CssRelativeColorChannel>,
    expression: CssCalculationExpression,
}

impl CssRelativeColorCalculation {
    pub(crate) fn from_expression(
        authored: CssAuthoredDeclarationValue,
        expression: CssCalculationExpression,
    ) -> Self {
        Self {
            data: Box::new(CssRelativeColorCalculationData {
                authored,
                result_type: expression.result_type(),
                references: expression.references(),
                expression,
            }),
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &CssAuthoredDeclarationValue {
        &self.data.authored
    }

    #[must_use]
    pub const fn result_type(&self) -> CssCalculationType {
        self.data.result_type
    }

    #[must_use]
    pub fn references(&self) -> &[CssRelativeColorChannel] {
        &self.data.references
    }

    /// Returns the shared exact numeric expression, including original leaf origins.
    pub fn expression(&self) -> CssCalculationExpressionRef<'_> {
        self.data.expression.as_ref()
    }
}

/// One result expression checked against a closed relative-color environment and slot domain.
#[derive(Clone, Debug, PartialEq)]
pub struct CssTypedRelativeColorExpression {
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
    value: CssRelativeColorExpressionValue,
}

impl CssTypedRelativeColorExpression {
    pub(crate) const fn new(
        environment: CssRelativeColorEnvironment,
        result_domain: CssRelativeColorResultDomain,
        value: CssRelativeColorExpressionValue,
    ) -> Self {
        Self {
            environment,
            result_domain,
            value,
        }
    }

    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    #[must_use]
    pub const fn result_domain(&self) -> CssRelativeColorResultDomain {
        self.result_domain
    }

    #[must_use]
    pub const fn value(&self) -> &CssRelativeColorExpressionValue {
        &self.value
    }
}

/// A parser-owned relative color with an exact three-channel typed result environment.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredRelativeColor {
    function: CssRelativeColorFunction,
    environment: CssRelativeColorEnvironment,
    source: Box<CssAuthoredColor>,
    channels: [CssTypedRelativeColorExpression; 3],
    alpha: Option<CssTypedRelativeColorExpression>,
}

impl CssAuthoredRelativeColor {
    pub(crate) fn new(
        function: CssRelativeColorFunction,
        environment: CssRelativeColorEnvironment,
        source: CssAuthoredColor,
        channels: [CssTypedRelativeColorExpression; 3],
        alpha: Option<CssTypedRelativeColorExpression>,
    ) -> Self {
        Self {
            function,
            environment,
            source: Box::new(source),
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    #[must_use]
    pub const fn source(&self) -> &CssAuthoredColor {
        &self.source
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssTypedRelativeColorExpression; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssTypedRelativeColorExpression> {
        self.alpha.as_ref()
    }
}

impl CssAuthoredPredefinedColor {
    pub(crate) const fn new(
        color_space: CssPredefinedColorSpace,
        channels: [CssAuthoredColorComponent; 3],
        alpha: Option<CssAuthoredColorComponent>,
    ) -> Self {
        Self {
            color_space,
            channels,
            alpha,
        }
    }

    #[must_use]
    pub const fn color_space(&self) -> CssPredefinedColorSpace {
        self.color_space
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssAuthoredColorComponent; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssAuthoredColorComponent> {
        self.alpha.as_ref()
    }
}

/// A checked authored percentage trailing one preserved `color-mix()` component.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMixPercentage {
    value: ColorMixPercentageValue,
}
#[derive(Clone, Debug, PartialEq)]
enum ColorMixPercentageValue {
    Finite(CssFiniteNumber),
    Exact(CssColorPercentageLiteral),
}
impl CssAuthoredColorMixPercentage {
    #[must_use]
    pub const fn try_new(value: f32) -> Option<Self> {
        if value >= 0.0 && value <= 100.0 {
            match CssFiniteNumber::try_new(value) {
                Some(value) => Some(Self {
                    value: ColorMixPercentageValue::Finite(value),
                }),
                None => None,
            }
        } else {
            None
        }
    }
    /// Checks the exact literal coefficient against the inclusive 0..100 range.
    pub fn try_from_component(
        component: crate::CssComponentValue,
    ) -> Result<Self, CssColorScalarError> {
        let literal = CssColorPercentageLiteral::try_from_component(component)?;
        if !crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation())
            .in_percentage_range()
        {
            return Err(CssColorScalarError::out_of_range(literal.origin().clone()));
        }
        Ok(Self {
            value: match crate::exact_decimal::exact_legacy_value(
                literal.numeric().representation(),
            ) {
                Some(value) => ColorMixPercentageValue::Finite(
                    CssFiniteNumber::try_new(value).expect("proved finite"),
                ),
                None => ColorMixPercentageValue::Exact(literal),
            },
        })
    }
    /// Returns the exact binary32 subset, never a rounded approximation.
    #[must_use]
    pub const fn value(&self) -> Option<f32> {
        match &self.value {
            ColorMixPercentageValue::Finite(v) => Some(v.value()),
            ColorMixPercentageValue::Exact(_) => None,
        }
    }
    #[must_use]
    pub const fn exact_literal(&self) -> Option<&CssColorPercentageLiteral> {
        match &self.value {
            ColorMixPercentageValue::Exact(v) => Some(v),
            ColorMixPercentageValue::Finite(_) => None,
        }
    }
}

/// A case-sensitive decoded custom color-profile identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileName(String);

impl CssColorProfileName {
    pub fn try_new(decoded: impl Into<String>) -> Option<Self> {
        let decoded = decoded.into();
        if !decoded.starts_with("--") {
            return None;
        }
        crate::CssComponentValue::try_ident(decoded.clone()).ok()?;
        Some(Self(decoded))
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A checked predefined interpolation method or symbolic custom profile.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorInterpolation(ColorInterpolation);
#[derive(Clone, Debug, PartialEq)]
enum ColorInterpolation {
    Predefined(CssColorInterpolationMethod),
    Custom(CssColorProfileName),
}
impl CssAuthoredColorInterpolation {
    pub fn try_predefined(method: CssColorInterpolationMethod) -> Option<Self> {
        if method.hue().is_some() && !method.space().is_polar() {
            return None;
        }
        Some(Self(ColorInterpolation::Predefined(method)))
    }
    #[must_use]
    pub fn custom(name: CssColorProfileName) -> Self {
        Self(ColorInterpolation::Custom(name))
    }
    #[must_use]
    pub fn predefined(&self) -> Option<CssColorInterpolationMethod> {
        match &self.0 {
            ColorInterpolation::Predefined(value) => Some(*value),
            ColorInterpolation::Custom(_) => None,
        }
    }
    #[must_use]
    pub fn custom_profile(&self) -> Option<&CssColorProfileName> {
        match &self.0 {
            ColorInterpolation::Custom(value) => Some(value),
            ColorInterpolation::Predefined(_) => None,
        }
    }
}

/// A literal weight in range or a symbolic percentage math function.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMixWeight(ColorMixWeight);
#[derive(Clone, Debug, PartialEq)]
enum ColorMixWeight {
    Literal(CssAuthoredColorMixPercentage),
    Calculation(CssPercentageCalculation),
}
impl CssAuthoredColorMixWeight {
    #[must_use]
    pub fn literal(value: CssAuthoredColorMixPercentage) -> Self {
        Self(ColorMixWeight::Literal(value))
    }
    pub fn try_calculation(
        value: CssPercentageCalculation,
    ) -> Result<Self, crate::CssComponentValueError> {
        use crate::{CssComponentValueRef as V, CssValueTokenRef as T};
        let mut significant = value
            .components()
            .items()
            .iter()
            .filter(|v| !matches!(v.view(), V::Comment(_) | V::Token(T::Whitespace(_))));
        let first = significant.next();
        if !first.is_some_and(|v| matches!(v.view(), V::Function(_)))
            || significant.next().is_some()
        {
            return Err(crate::CssComponentValueError::new(
                crate::CssComponentValueErrorKind::InvalidToken,
                first.map_or_else(|| value.origin().clone(), |v| v.origin().clone()),
            ));
        }
        Ok(Self(ColorMixWeight::Calculation(value)))
    }
    #[must_use]
    pub fn literal_value(&self) -> Option<&CssAuthoredColorMixPercentage> {
        match &self.0 {
            ColorMixWeight::Literal(v) => Some(v),
            ColorMixWeight::Calculation(_) => None,
        }
    }
    #[must_use]
    pub fn calculation(&self) -> Option<&CssPercentageCalculation> {
        match &self.0 {
            ColorMixWeight::Calculation(v) => Some(v),
            ColorMixWeight::Literal(_) => None,
        }
    }
}

/// One authored color and its optional literal or calculated weight.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMixComponent {
    color: Box<CssAuthoredColor>,
    weight: Option<CssAuthoredColorMixWeight>,
}
impl CssAuthoredColorMixComponent {
    #[must_use]
    pub fn new(color: CssAuthoredColor, percentage: Option<CssAuthoredColorMixPercentage>) -> Self {
        Self::with_weight(color, percentage.map(CssAuthoredColorMixWeight::literal))
    }
    #[must_use]
    pub fn with_weight(color: CssAuthoredColor, weight: Option<CssAuthoredColorMixWeight>) -> Self {
        Self {
            color: Box::new(color),
            weight,
        }
    }
    #[must_use]
    pub const fn color(&self) -> &CssAuthoredColor {
        &self.color
    }
    #[must_use]
    pub const fn weight(&self) -> Option<&CssAuthoredColorMixWeight> {
        self.weight.as_ref()
    }
}

/// A rejected authored mix graph, without invented source coordinates.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorMixConstructionError {
    EmptyComponents,
    NestingLimit,
    CapacityOverflow,
}
impl std::fmt::Display for CssColorMixConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyComponents => "color-mix requires at least one component",
            Self::NestingLimit => "color-mix exceeds the structural nesting limit",
            Self::CapacityOverflow => "color-mix structural capacity overflow",
        })
    }
}
impl std::error::Error for CssColorMixConstructionError {}

/// A nonempty ordered authored mix with optional explicit interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAuthoredColorMix {
    interpolation: Option<CssAuthoredColorInterpolation>,
    components: Vec<CssAuthoredColorMixComponent>,
    nesting_depth: u32,
}
impl CssAuthoredColorMix {
    pub fn try_from_components(
        interpolation: Option<CssAuthoredColorInterpolation>,
        components: Vec<CssAuthoredColorMixComponent>,
    ) -> Result<Self, CssColorMixConstructionError> {
        if components.is_empty() {
            return Err(CssColorMixConstructionError::EmptyComponents);
        }
        let nesting_depth = color_mix_depth(&components)?;
        Ok(Self {
            interpolation,
            components,
            nesting_depth,
        })
    }
    #[must_use]
    pub fn try_new(
        interpolation: CssColorInterpolationMethod,
        left: CssAuthoredColorMixComponent,
        right: CssAuthoredColorMixComponent,
    ) -> Option<Self> {
        Self::try_from_components(
            Some(CssAuthoredColorInterpolation::try_predefined(
                interpolation,
            )?),
            vec![left, right],
        )
        .ok()
    }
    #[must_use]
    pub const fn interpolation(&self) -> Option<&CssAuthoredColorInterpolation> {
        self.interpolation.as_ref()
    }
    #[must_use]
    pub fn components(&self) -> &[CssAuthoredColorMixComponent] {
        &self.components
    }
}

#[derive(Clone, Copy, Debug)]
enum ColorGraphDepthError {
    NestingLimit,
    CapacityOverflow,
}
impl From<ColorGraphDepthError> for CssColorMixConstructionError {
    fn from(value: ColorGraphDepthError) -> Self {
        match value {
            ColorGraphDepthError::NestingLimit => Self::NestingLimit,
            ColorGraphDepthError::CapacityOverflow => Self::CapacityOverflow,
        }
    }
}

fn color_mix_depth(
    components: &[CssAuthoredColorMixComponent],
) -> Result<u32, ColorGraphDepthError> {
    let mut depth = 1;
    for component in components {
        depth = depth.max(
            1u32.checked_add(authored_color_depth(component.color())?)
                .ok_or(ColorGraphDepthError::CapacityOverflow)?,
        );
        if let Some(value) = component
            .weight()
            .and_then(CssAuthoredColorMixWeight::calculation)
        {
            depth = depth.max(
                1u32.checked_add(value.components().nesting_depth())
                    .ok_or(ColorGraphDepthError::CapacityOverflow)?,
            );
        }
        if depth > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(ColorGraphDepthError::NestingLimit);
        }
    }
    Ok(depth)
}

fn color_component_depth(value: &CssAuthoredColorComponent) -> u32 {
    match value {
        CssAuthoredColorComponent::NumberCalculation(v) => v.components().nesting_depth(),
        CssAuthoredColorComponent::PercentageCalculation(v) => v.components().nesting_depth(),
        _ => 0,
    }
}
fn color_hue_depth(value: &CssAuthoredHue) -> u32 {
    match value {
        CssAuthoredHue::NumberCalculation(v) => v.components().nesting_depth(),
        CssAuthoredHue::AngleCalculation(v) => v.components().nesting_depth(),
        _ => 0,
    }
}
fn authored_color_depth(mut color: &CssAuthoredColor) -> Result<u32, ColorGraphDepthError> {
    use CssAuthoredColorRepresentation as R;
    let mut ancestors = 0u32;
    let mut maximum = 0u32;
    loop {
        let alpha =
            |v: &Option<CssAuthoredColorComponent>| v.as_ref().map_or(0, color_component_depth);
        let channels = |v: &[CssAuthoredColorComponent; 3]| {
            v.iter().map(color_component_depth).max().unwrap_or(0)
        };
        let depth = match &color.representation {
            R::CurrentColor | R::Transparent | R::Hex(_) | R::Named(_) | R::System(_) => 0,
            R::Rgb(v) => 1 + channels(&v.channels).max(alpha(&v.alpha)),
            R::Hsl(v) => {
                1 + color_hue_depth(&v.hue)
                    .max(color_component_depth(&v.saturation))
                    .max(color_component_depth(&v.lightness))
                    .max(alpha(&v.alpha))
            }
            R::Hwb(v) => {
                1 + color_hue_depth(&v.hue)
                    .max(color_component_depth(&v.whiteness))
                    .max(color_component_depth(&v.blackness))
                    .max(alpha(&v.alpha))
            }
            R::Lab(v) | R::Oklab(v) => {
                1 + color_component_depth(&v.lightness)
                    .max(color_component_depth(&v.a))
                    .max(color_component_depth(&v.b))
                    .max(alpha(&v.alpha))
            }
            R::Lch(v) | R::Oklch(v) => {
                1 + color_component_depth(&v.lightness)
                    .max(color_component_depth(&v.chroma))
                    .max(color_hue_depth(&v.hue))
                    .max(alpha(&v.alpha))
            }
            R::Predefined(v) => 1 + channels(&v.channels).max(alpha(&v.alpha)),
            // Every mix constructor caches the complete checked subtree depth.
            R::ColorMix(v) => v.nesting_depth,
            R::Custom(v) => v.nesting_depth,
            R::RelativeCustom(v) => v.nesting_depth,
            R::Alpha(v) => v.nesting_depth,
            R::Relative(v) => {
                ancestors = ancestors
                    .checked_add(1)
                    .ok_or(ColorGraphDepthError::CapacityOverflow)?;
                for expression in v.channels.iter().chain(v.alpha.iter()) {
                    let depth = match &expression.value {
                        CssRelativeColorExpressionValue::Calculation(calc) => {
                            calc.data.expression.component_nesting_depth()
                        }
                        _ => 0,
                    };
                    maximum = maximum.max(
                        ancestors
                            .checked_add(depth)
                            .ok_or(ColorGraphDepthError::CapacityOverflow)?,
                    );
                }
                if ancestors > crate::STRUCTURAL_NESTING_LIMIT
                    || maximum > crate::STRUCTURAL_NESTING_LIMIT
                {
                    return Err(ColorGraphDepthError::NestingLimit);
                }
                color = &v.source;
                continue;
            }
            R::PreservedI01(v) => legacy_color_depth(v)?,
        };
        maximum = maximum.max(
            ancestors
                .checked_add(depth)
                .ok_or(ColorGraphDepthError::CapacityOverflow)?,
        );
        return if maximum > crate::STRUCTURAL_NESTING_LIMIT {
            Err(ColorGraphDepthError::NestingLimit)
        } else {
            Ok(maximum)
        };
    }
}

fn legacy_color_depth(color: &CssColor) -> Result<u32, ColorGraphDepthError> {
    let mut pending = vec![(color, 0u32)];
    let mut maximum = 0;
    while let Some((color, enclosing)) = pending.pop() {
        let depth = enclosing
            .checked_add(match color {
                CssColor::CurrentColor | CssColor::System(_) | CssColor::Rgba(_) => 0,
                _ => 1,
            })
            .ok_or(ColorGraphDepthError::CapacityOverflow)?;
        maximum = maximum.max(depth);
        if maximum > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(ColorGraphDepthError::NestingLimit);
        }
        match color {
            CssColor::ColorMix(v) => {
                pending.push((v.left().color(), depth));
                pending.push((v.right().color(), depth));
            }
            CssColor::Relative(v) => {
                pending.push((v.source(), depth));
                // Frozen expressions retain only authored text; use the shared
                // component owner, never a second numeric grammar or evaluator.
                for expression in v.components().iter().chain(v.alpha()) {
                    let values = crate::parse_component_values(expression.authored().as_css())
                        .map_err(|_| ColorGraphDepthError::NestingLimit)?;
                    maximum = maximum.max(
                        depth
                            .checked_add(values.nesting_depth())
                            .ok_or(ColorGraphDepthError::CapacityOverflow)?,
                    );
                }
            }
            _ => {}
        }
    }
    if maximum > crate::STRUCTURAL_NESTING_LIMIT {
        Err(ColorGraphDepthError::NestingLimit)
    } else {
        Ok(maximum)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CssParsedColor {
    current: CssAuthoredColor,
    i01_subset: Option<CssColor>,
}

impl CssParsedColor {
    pub(crate) const fn new(current: CssAuthoredColor, i01_subset: Option<CssColor>) -> Self {
        Self {
            current,
            i01_subset,
        }
    }

    pub(crate) fn from_i01(value: CssColor) -> Self {
        Self::new(CssAuthoredColor::preserved_i01(value.clone()), Some(value))
    }

    pub(crate) fn into_parts(self) -> (CssAuthoredColor, Option<CssColor>) {
        (self.current, self.i01_subset)
    }

    pub(crate) const fn current(&self) -> &CssAuthoredColor {
        &self.current
    }

    pub(crate) const fn i01_subset(&self) -> Option<&CssColor> {
        self.i01_subset.as_ref()
    }
}

pub(super) fn parsed_color_options_equal(
    left: Option<&CssParsedColor>,
    right: Option<&CssParsedColor>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(left), Some(right)) => match (left.i01_subset(), right.i01_subset()) {
            (Some(left), Some(right)) => left == right,
            (None, None) => left.current() == right.current(),
            (Some(_), None) | (None, Some(_)) => false,
        },
        (None, Some(_)) | (Some(_), None) => false,
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColor {
    CurrentColor,
    Rgba(CssRgbaColor),
    Hsl(CssHslColor),
    Hwb(CssHwbColor),
    Lab(CssLabColor),
    Lch(CssLchColor),
    Oklab(CssLabColor),
    Oklch(CssLchColor),
    ColorFunction(CssColorFunction),
    System(CssSystemColor),
    ColorMix(CssColorMix),
    Relative(CssRelativeColor),
}

impl CssColor {
    pub const TRANSPARENT: Self = Self::Rgba(CssRgbaColor {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 0.0,
    });
    pub const BLACK: Self = Self::Rgba(CssRgbaColor {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 1.0,
    });
    pub const WHITE: Self = Self::Rgba(CssRgbaColor {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 1.0,
    });

    #[must_use]
    pub const fn as_rgba(&self) -> Option<&CssRgbaColor> {
        match self {
            Self::Rgba(color) => Some(color),
            Self::CurrentColor
            | Self::Hsl(_)
            | Self::Hwb(_)
            | Self::Lab(_)
            | Self::Lch(_)
            | Self::Oklab(_)
            | Self::Oklch(_)
            | Self::ColorFunction(_)
            | Self::System(_)
            | Self::ColorMix(_)
            | Self::Relative(_) => None,
        }
    }

    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match self {
            Self::CurrentColor => "currentcolor",
            Self::Rgba(_) => "rgba",
            Self::Hsl(_) => "hsl",
            Self::Hwb(_) => "hwb",
            Self::Lab(_) => "lab",
            Self::Lch(_) => "lch",
            Self::Oklab(_) => "oklab",
            Self::Oklch(_) => "oklch",
            Self::ColorFunction(_) => "color",
            Self::System(_) => "system",
            Self::ColorMix(_) => "color-mix",
            Self::Relative(_) => "relative",
        }
    }

    #[must_use]
    pub fn try_rgba(r: f32, g: f32, b: f32, a: f32) -> Option<Self> {
        if [r, g, b, a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
        {
            Some(Self::rgba_unchecked(r, g, b, a))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn rgba_unchecked(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self::Rgba(CssRgbaColor {
            red: normalized_color_channel_to_byte(r),
            green: normalized_color_channel_to_byte(g),
            blue: normalized_color_channel_to_byte(b),
            alpha: a,
        })
    }
}

fn normalized_color_channel_to_byte(channel: f32) -> u8 {
    (channel * 255.0).round() as u8
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssRgbaColor {
    red: u8,
    green: u8,
    blue: u8,
    alpha: f32,
}

impl CssRgbaColor {
    #[must_use]
    pub fn try_new(red: u8, green: u8, blue: u8, alpha: f32) -> Option<Self> {
        if alpha.is_finite() && (0.0..=1.0).contains(&alpha) {
            Some(Self {
                red,
                green,
                blue,
                alpha,
            })
        } else {
            None
        }
    }

    #[must_use]
    pub const fn red(&self) -> u8 {
        self.red
    }

    #[must_use]
    pub const fn green(&self) -> u8 {
        self.green
    }

    #[must_use]
    pub const fn blue(&self) -> u8 {
        self.blue
    }

    #[must_use]
    pub const fn alpha(&self) -> f32 {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssHslColor {
    hue: Option<f32>,
    saturation: Option<f32>,
    lightness: Option<f32>,
    alpha: Option<f32>,
}

impl CssHslColor {
    #[must_use]
    pub fn try_new(
        hue: Option<f32>,
        saturation: Option<f32>,
        lightness: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([hue, saturation, lightness]) && color_alpha_is_valid(alpha)
        {
            Some(Self::new(hue, saturation, lightness, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        hue: Option<f32>,
        saturation: Option<f32>,
        lightness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            hue,
            saturation,
            lightness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn saturation(&self) -> Option<f32> {
        self.saturation
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssHwbColor {
    hue: Option<f32>,
    whiteness: Option<f32>,
    blackness: Option<f32>,
    alpha: Option<f32>,
}

impl CssHwbColor {
    #[must_use]
    pub fn try_new(
        hue: Option<f32>,
        whiteness: Option<f32>,
        blackness: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([hue, whiteness, blackness]) && color_alpha_is_valid(alpha) {
            Some(Self::new(hue, whiteness, blackness, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        hue: Option<f32>,
        whiteness: Option<f32>,
        blackness: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            hue,
            whiteness,
            blackness,
            alpha,
        }
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn whiteness(&self) -> Option<f32> {
        self.whiteness
    }

    #[must_use]
    pub const fn blackness(&self) -> Option<f32> {
        self.blackness
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssLabColor {
    lightness: Option<f32>,
    a: Option<f32>,
    b: Option<f32>,
    alpha: Option<f32>,
}

impl CssLabColor {
    #[must_use]
    pub fn try_new(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([lightness, a, b]) && color_alpha_is_valid(alpha) {
            Some(Self::new(lightness, a, b, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        lightness: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            lightness,
            a,
            b,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn a(&self) -> Option<f32> {
        self.a
    }

    #[must_use]
    pub const fn b(&self) -> Option<f32> {
        self.b
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssLchColor {
    lightness: Option<f32>,
    chroma: Option<f32>,
    hue: Option<f32>,
    alpha: Option<f32>,
}

impl CssLchColor {
    #[must_use]
    pub fn try_new(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite([lightness, chroma, hue]) && color_alpha_is_valid(alpha) {
            Some(Self::new(lightness, chroma, hue, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        lightness: Option<f32>,
        chroma: Option<f32>,
        hue: Option<f32>,
        alpha: Option<f32>,
    ) -> Self {
        Self {
            lightness,
            chroma,
            hue,
            alpha,
        }
    }

    #[must_use]
    pub const fn lightness(&self) -> Option<f32> {
        self.lightness
    }

    #[must_use]
    pub const fn chroma(&self) -> Option<f32> {
        self.chroma
    }

    #[must_use]
    pub const fn hue(&self) -> Option<f32> {
        self.hue
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssPredefinedColorSpace {
    Srgb,
    SrgbLinear,
    DisplayP3,
    DisplayP3Linear,
    A98Rgb,
    ProphotoRgb,
    Rec2020,
    XyzD50,
    XyzD65,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssColorFunction {
    color_space: CssPredefinedColorSpace,
    components: [Option<f32>; 3],
    alpha: Option<f32>,
}

impl CssColorFunction {
    #[must_use]
    pub fn try_new(
        color_space: CssPredefinedColorSpace,
        components: [Option<f32>; 3],
        alpha: Option<f32>,
    ) -> Option<Self> {
        if color_components_are_finite(components) && color_alpha_is_valid(alpha) {
            Some(Self::new(color_space, components, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) const fn new(
        color_space: CssPredefinedColorSpace,
        components: [Option<f32>; 3],
        alpha: Option<f32>,
    ) -> Self {
        Self {
            color_space,
            components,
            alpha,
        }
    }

    #[must_use]
    pub const fn color_space(&self) -> CssPredefinedColorSpace {
        self.color_space
    }

    #[must_use]
    pub const fn components(&self) -> &[Option<f32>; 3] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<f32> {
        self.alpha
    }
}

fn color_components_are_finite(components: [Option<f32>; 3]) -> bool {
    components
        .into_iter()
        .all(|component| component.is_none_or(f32::is_finite))
}

fn color_alpha_is_valid(alpha: Option<f32>) -> bool {
    alpha.is_none_or(|alpha| alpha.is_finite() && (0.0..=1.0).contains(&alpha))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSystemColor {
    Canvas,
    CanvasText,
    LinkText,
    VisitedText,
    ActiveText,
    ButtonFace,
    ButtonText,
    ButtonBorder,
    Field,
    FieldText,
    Highlight,
    HighlightText,
    Mark,
    MarkText,
    GrayText,
    SelectedItem,
    SelectedItemText,
    AccentColor,
    AccentColorText,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMix {
    interpolation: CssColorInterpolationMethod,
    left: CssColorMixComponent,
    right: CssColorMixComponent,
}

impl CssColorMix {
    #[must_use]
    pub const fn new(
        interpolation: CssColorInterpolationMethod,
        left: CssColorMixComponent,
        right: CssColorMixComponent,
    ) -> Self {
        Self {
            interpolation,
            left,
            right,
        }
    }

    #[must_use]
    pub const fn interpolation(&self) -> &CssColorInterpolationMethod {
        &self.interpolation
    }

    #[must_use]
    pub const fn left(&self) -> &CssColorMixComponent {
        &self.left
    }

    #[must_use]
    pub const fn right(&self) -> &CssColorMixComponent {
        &self.right
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMixComponent {
    color: Box<CssColor>,
    percentage: Option<f32>,
}

impl CssColorMixComponent {
    #[must_use]
    pub fn try_new(color: CssColor, percentage: Option<f32>) -> Option<Self> {
        if color_percentage_is_valid(percentage) {
            Some(Self::new(color, percentage))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(color: CssColor, percentage: Option<f32>) -> Self {
        Self {
            color: Box::new(color),
            percentage,
        }
    }

    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }

    #[must_use]
    pub const fn percentage(&self) -> Option<f32> {
        self.percentage
    }
}

fn color_percentage_is_valid(percentage: Option<f32>) -> bool {
    percentage
        .is_none_or(|percentage| percentage.is_finite() && (0.0..=100.0).contains(&percentage))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInterpolationMethod {
    space: CssColorInterpolationSpace,
    hue: Option<CssHueInterpolationMethod>,
}

impl CssColorInterpolationMethod {
    #[must_use]
    pub const fn new(
        space: CssColorInterpolationSpace,
        hue: Option<CssHueInterpolationMethod>,
    ) -> Self {
        Self { space, hue }
    }

    #[must_use]
    pub const fn space(&self) -> CssColorInterpolationSpace {
        self.space
    }

    #[must_use]
    pub const fn hue(&self) -> Option<CssHueInterpolationMethod> {
        self.hue
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorInterpolationSpace {
    Predefined(CssPredefinedColorSpace),
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
}

impl CssColorInterpolationSpace {
    /// Reports whether the interpolation space has a polar hue component.
    #[must_use]
    pub const fn is_polar(self) -> bool {
        matches!(self, Self::Hsl | Self::Hwb | Self::Lch | Self::Oklch)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssHueInterpolationMethod {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeColor {
    function: CssRelativeColorFunction,
    source: Box<CssColor>,
    components: Vec<CssColorComponentExpression>,
    alpha: Option<CssColorComponentExpression>,
}

impl CssRelativeColor {
    #[must_use]
    pub fn try_new(
        function: CssRelativeColorFunction,
        source: CssColor,
        components: Vec<CssColorComponentExpression>,
        alpha: Option<CssColorComponentExpression>,
    ) -> Option<Self> {
        if components.len() == function.component_count() {
            Some(Self::new(function, source, components, alpha))
        } else {
            None
        }
    }

    #[must_use]
    pub(crate) fn new(
        function: CssRelativeColorFunction,
        source: CssColor,
        components: Vec<CssColorComponentExpression>,
        alpha: Option<CssColorComponentExpression>,
    ) -> Self {
        Self {
            function,
            source: Box::new(source),
            components,
            alpha,
        }
    }

    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    #[must_use]
    pub const fn source(&self) -> &CssColor {
        &self.source
    }

    #[must_use]
    pub fn components(&self) -> &[CssColorComponentExpression] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponentExpression> {
        self.alpha.as_ref()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color(CssPredefinedColorSpace),
}

impl CssRelativeColorFunction {
    #[must_use]
    pub const fn component_count(self) -> usize {
        match self {
            Self::Rgb
            | Self::Hsl
            | Self::Hwb
            | Self::Lab
            | Self::Lch
            | Self::Oklab
            | Self::Oklch
            | Self::Color(_) => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorComponentExpression {
    authored: CssAuthoredDeclarationValue,
    references: Vec<CssVariableReference>,
}

impl CssColorComponentExpression {
    #[must_use]
    pub fn new(
        authored: CssAuthoredDeclarationValue,
        references: Vec<CssVariableReference>,
    ) -> Self {
        Self {
            authored,
            references,
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &CssAuthoredDeclarationValue {
        &self.authored
    }

    #[must_use]
    pub fn references(&self) -> &[CssVariableReference] {
        &self.references
    }
}
