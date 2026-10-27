use super::{CssColorNumberLiteral, CssColorPercentageLiteral};
use crate::declaration::CssAuthoredDeclarationValue;
use crate::numeric::expression::CssCalculationExpression;
use crate::numeric::{
    CssAngle, CssCalculation, CssCalculationExpressionRef, CssCalculationType, CssHintedNumber,
    CssNumber, CssNumericLiteral, CssPercentage,
};
use surgeist_color::{ColorSpace, HueInterpolation, PredefinedColorSpace};

pub(crate) mod inverse;
pub(crate) mod operands;
pub(crate) mod recognition;
pub(crate) mod serialization;

/// An authored color retaining its specified syntax and symbolic dependencies.
///
/// # Specified serialization
///
/// Produces canonical specified color text without resolving external color context.
#[derive(Clone)]
pub struct CssColor {
    representation: CssColorRepresentation,
    original_closed: bool,
}

impl std::fmt::Debug for CssColor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CssColor")
            .field("representation", &self.representation)
            .finish()
    }
}

// Closure provenance does not redefine the existing semantic color equality.
impl PartialEq for CssColor {
    fn eq(&self, other: &Self) -> bool {
        self.representation == other.representation
    }
}

#[derive(Clone, Debug, PartialEq)]
enum CssColorRepresentation {
    CurrentColor,
    Transparent,
    Hex(CssHexColor),
    Named(CssNamedColor),
    System(CssSystemColor),
    Rgb(CssRgbColor),
    Hsl(CssHslColor),
    Hwb(CssHwbColor),
    Lab(CssLabColor),
    Lch(CssLchColor),
    Oklab(CssLabColor),
    Oklch(CssLchColor),
    Predefined(CssPredefinedColor),
    Custom(CssCustomColor),
    // Keep retained profile-expression metadata out of every inline color value.
    RelativeCustom(Box<CssRelativeCustomColor>),
    Alpha(CssAlphaColor),
    Relative(Box<CssRelativeColor>),
    ColorMix(CssColorMix),
    LightDark(Box<CssLightDarkColor>),
    ContrastColor(Box<CssContrastColor>),
    DeviceCmyk(Box<CssDeviceCmykColor>),
}

/// An uncalibrated authored device-CMYK color, without device/profile resolution.
///
/// Four ink channels remain unbounded. Legacy syntax admits only numbers and
/// number calculations, with no alpha; modern syntax also admits percentages
/// and missing components, and an optional alpha. Declared serialization retains
/// the device input rather than computing an RGB or Lab equivalent.
#[derive(Clone, Debug, PartialEq)]
pub struct CssDeviceCmykColor {
    syntax: CssColorSyntax,
    channels: [CssColorComponent; 4],
    alpha: Option<CssColorComponent>,
}

impl CssDeviceCmykColor {
    /// Checks syntax and complete component depth, retaining every supplied value.
    pub fn try_new(
        syntax: CssColorSyntax,
        channels: [CssColorComponent; 4],
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if syntax == CssColorSyntax::Legacy
            && (alpha.is_some()
                || channels.iter().any(|value| {
                    !matches!(
                        value,
                        CssColorComponent::Number(_) | CssColorComponent::NumberCalculation(_)
                    )
                }))
        {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::InvalidSyntax,
            ));
        }
        check_components_depth(channels.iter().chain(alpha.iter()))?;
        Ok(Self {
            syntax,
            channels,
            alpha,
        })
    }

    /// Returns the retained authored punctuation form.
    pub const fn syntax(&self) -> CssColorSyntax {
        self.syntax
    }
    /// Borrows cyan, magenta, yellow and black in that order.
    pub const fn channels(&self) -> &[CssColorComponent; 4] {
        &self.channels
    }
    /// Borrows explicit alpha; omission remains distinct from `none`.
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }
}

/// A checked symbolic input awaiting downstream contrast-policy evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssContrastColor {
    color: CssColor,
    nesting_depth: u32,
}

impl CssContrastColor {
    /// Checks one wrapper plus the complete input subtree against the structural ceiling.
    pub fn try_new(color: CssColor) -> Result<Self, CssColorConstructionError> {
        let nesting_depth = composed_color_depth(authored_color_depth(&color)?)?;
        Ok(Self {
            color,
            nesting_depth,
        })
    }

    /// Borrows the complete authored input without evaluating contrast.
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
}

/// Two checked authored colors awaiting downstream used-scheme selection.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLightDarkColor {
    light: CssColor,
    dark: CssColor,
    nesting_depth: u32,
}

impl CssLightDarkColor {
    /// Checks the complete composed color subtree against the structural ceiling.
    pub fn try_new(light: CssColor, dark: CssColor) -> Result<Self, CssColorConstructionError> {
        let nesting_depth = authored_color_depth(&light)?
            .max(authored_color_depth(&dark)?)
            .checked_add(1)
            .ok_or(CssColorConstructionError::new(
                ColorConstructionKind::CapacityOverflow,
            ))?;
        if nesting_depth > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::NestingLimit,
            ));
        }
        Ok(Self {
            light,
            dark,
            nesting_depth,
        })
    }

    /// Borrows the complete authored light-scheme branch.
    pub const fn light(&self) -> &CssColor {
        &self.light
    }
    /// Borrows the complete authored dark-scheme branch.
    pub const fn dark(&self) -> &CssColor {
        &self.dark
    }
}

// Closure admission is separate from browser recovery and canonical projection.
// The checked color graph has a bounded structural depth; no source is reserialized.
impl CssColor {
    pub(crate) fn retain_original_closure(&mut self, closed: bool) {
        self.original_closed = closed;
    }

    pub(crate) fn is_closed(&self) -> bool {
        if !self.original_closed {
            return false;
        }
        use CssColorRepresentation as R;
        let channel = |v: &CssColorComponent| match v {
            CssColorComponent::NumberCalculation(v) => {
                v.components().first_implicit_origin().is_none()
            }
            CssColorComponent::HintedNumberCalculation(v) => {
                v.components().first_implicit_origin().is_none()
            }
            CssColorComponent::PercentageCalculation(v) => {
                v.components().first_implicit_origin().is_none()
            }
            _ => true,
        };
        let hue = |v: &CssColorHue| match v {
            CssColorHue::NumberCalculation(v) => v.components().first_implicit_origin().is_none(),
            CssColorHue::AngleCalculation(v) => v.components().first_implicit_origin().is_none(),
            _ => true,
        };
        let relative = |v: &CssRelativeColorExpression| match &v.value {
            CssRelativeColorExpressionValue::Calculation(v) => v.data.expression.is_closed(),
            _ => true,
        };
        match &self.representation {
            R::CurrentColor | R::Transparent | R::Hex(_) | R::Named(_) | R::System(_) => true,
            R::Rgb(v) => v.channels.iter().chain(v.alpha.iter()).all(channel),
            R::Hsl(v) => {
                hue(&v.hue)
                    && [&v.saturation, &v.lightness]
                        .into_iter()
                        .chain(v.alpha.iter())
                        .all(channel)
            }
            R::Hwb(v) => {
                hue(&v.hue)
                    && [&v.whiteness, &v.blackness]
                        .into_iter()
                        .chain(v.alpha.iter())
                        .all(channel)
            }
            R::Lab(v) | R::Oklab(v) => [&v.lightness, &v.a, &v.b]
                .into_iter()
                .chain(v.alpha.iter())
                .all(channel),
            R::Lch(v) | R::Oklch(v) => {
                hue(&v.hue)
                    && [&v.lightness, &v.chroma]
                        .into_iter()
                        .chain(v.alpha.iter())
                        .all(channel)
            }
            R::Predefined(v) => v.channels.iter().chain(v.alpha.iter()).all(channel),
            R::DeviceCmyk(v) => v.channels.iter().chain(v.alpha.iter()).all(channel),
            R::Custom(v) => v.channels.iter().chain(v.alpha.iter()).all(channel),
            R::Relative(v) => {
                v.source.is_closed() && v.channels.iter().chain(v.alpha.iter()).all(relative)
            }
            R::RelativeCustom(v) => {
                v.source.is_closed()
                    && v.channels
                        .iter()
                        .chain(v.alpha.iter())
                        .all(|v| v.components().first_implicit_origin().is_none())
            }
            R::Alpha(v) => v.source.is_closed() && v.alpha.iter().all(relative),
            R::LightDark(v) => v.light.is_closed() && v.dark.is_closed(),
            R::ContrastColor(v) => v.color.is_closed(),
            R::ColorMix(v) => v.components.iter().all(|v| {
                v.color.is_closed()
                    && v.weight.as_ref().is_none_or(|v| {
                        v.calculation()
                            .is_none_or(|v| v.components().first_implicit_origin().is_none())
                    })
            }),
        }
    }
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssColor {
    /// Emits canonical specified colour text without resolving external colour context.
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> Result<(), crate::serialization::CssSerializationError> {
        self.append_specified(&mut writer.context, &mut writer.css)
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssColor);

impl CssColor {
    /// Retains an uncalibrated device input without conversion or ink clamping.
    pub fn from_device_cmyk(value: CssDeviceCmykColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::DeviceCmyk(Box::new(value)),
        }
    }

    /// Borrows the checked authored CMYK payload, when present.
    pub const fn device_cmyk_value(&self) -> Option<&CssDeviceCmykColor> {
        match &self.representation {
            CssColorRepresentation::DeviceCmyk(value) => Some(value),
            _ => None,
        }
    }

    /// Retains a checked symbolic input without choosing white or black.
    pub fn from_contrast_color(value: CssContrastColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::ContrastColor(Box::new(value)),
        }
    }

    /// Borrows the checked payload, when this is a contrast color.
    pub const fn contrast_color_value(&self) -> Option<&CssContrastColor> {
        match &self.representation {
            CssColorRepresentation::ContrastColor(value) => Some(value),
            _ => None,
        }
    }

    /// Retains a checked pair without selecting a scheme or resolving its colors.
    pub fn from_light_dark(value: CssLightDarkColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::LightDark(Box::new(value)),
        }
    }

    /// Borrows the checked LightDark payload, when this is a color pair.
    pub const fn light_dark_value(&self) -> Option<&CssLightDarkColor> {
        match &self.representation {
            CssColorRepresentation::LightDark(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn nesting_depth(&self) -> Result<u32, CssColorConstructionError> {
        authored_color_depth(self).map_err(Into::into)
    }
    /// Retains a checked custom-profile `color()` value.
    pub const fn from_custom(value: CssCustomColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Custom(value),
        }
    }
    /// Retains a checked custom-profile relative color.
    pub fn from_relative_custom(value: CssRelativeCustomColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::RelativeCustom(Box::new(value)),
        }
    }
    /// Retains a checked `alpha(from ...)` value.
    pub const fn from_alpha(value: CssAlphaColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Alpha(value),
        }
    }
    /// Borrows the checked payload, when this is a custom-profile `color()`.
    pub const fn custom_value(&self) -> Option<&CssCustomColor> {
        match &self.representation {
            CssColorRepresentation::Custom(v) => Some(v),
            _ => None,
        }
    }
    /// Borrows the checked payload, when this is a custom-profile relative color.
    pub const fn relative_custom_value(&self) -> Option<&CssRelativeCustomColor> {
        match &self.representation {
            CssColorRepresentation::RelativeCustom(v) => Some(v),
            _ => None,
        }
    }
    /// Borrows the checked payload, when this is an `alpha(from ...)` color.
    pub const fn alpha_value(&self) -> Option<&CssAlphaColor> {
        match &self.representation {
            CssColorRepresentation::Alpha(v) => Some(v),
            _ => None,
        }
    }

    /// Creates the `currentcolor` keyword.
    pub const fn current_color() -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::CurrentColor,
        }
    }

    /// Creates the `transparent` keyword.
    pub const fn transparent() -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Transparent,
        }
    }

    /// Returns the fixed keyword's encoded sRGB bytes in straight-alpha R, G, B, A order.
    ///
    /// The 148 opaque named colors from Color 4 CRD 2026-09-08 §6.1 return
    /// their specified RGB bytes with alpha 255. The distinct `transparent`
    /// branch returns transparent black `[0, 0, 0, 0]`, following §6.3.
    /// The declared keyword and alias identity remain unchanged.
    ///
    /// `None` means this authored branch is outside the fixed keyword projection,
    /// not that it is invalid or unresolvable. Hex colors and color functions
    /// return `None`, as do `currentcolor`, system colors, and contextual or
    /// compound colors. This does not evaluate authored expressions, select
    /// palettes or profiles, perform gamut mapping, or serialize computed CSS.
    ///
    /// ```
    /// use surgeist_css::color::{CssColor, CssHexColor, CssNamedColor};
    /// let cyan = CssColor::from_named(CssNamedColor::try_new("CyAn").unwrap());
    /// assert_eq!(cyan.keyword_srgba8(), Some([0, 255, 255, 255]));
    /// assert_eq!(cyan.named().unwrap().name(), "cyan");
    /// assert_eq!(CssColor::transparent().keyword_srgba8(), Some([0, 0, 0, 0]));
    /// let hex = CssColor::from_hex(CssHexColor::try_new("00ffff").unwrap());
    /// assert_eq!(hex.keyword_srgba8(), None);
    /// ```
    #[must_use]
    pub const fn keyword_srgba8(&self) -> Option<[u8; 4]> {
        match &self.representation {
            CssColorRepresentation::Named(value) => {
                let [red, green, blue] = value.rgb;
                Some([red, green, blue, 255])
            }
            CssColorRepresentation::Transparent => Some([0, 0, 0, 0]),
            _ => None,
        }
    }

    /// Retains a checked hex color.
    pub const fn from_hex(value: CssHexColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Hex(value),
        }
    }

    /// Retains a checked named color keyword.
    pub const fn from_named(value: CssNamedColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Named(value),
        }
    }

    /// Retains a system color keyword.
    pub const fn from_system(value: CssSystemColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::System(value),
        }
    }

    /// Retains a checked `rgb()` or `rgba()` color.
    pub const fn from_rgb(value: CssRgbColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Rgb(value),
        }
    }

    /// Retains a checked `hsl()` or `hsla()` color.
    pub const fn from_hsl(value: CssHslColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Hsl(value),
        }
    }

    /// Retains a checked `hwb()` color.
    pub const fn from_hwb(value: CssHwbColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Hwb(value),
        }
    }

    /// Retains a checked `lab()` color.
    pub const fn from_lab(value: CssLabColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Lab(value),
        }
    }

    /// Retains a checked `lch()` color.
    pub const fn from_lch(value: CssLchColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Lch(value),
        }
    }

    /// Retains a checked `oklab()` color; it shares the payload type of `lab()`.
    pub const fn from_oklab(value: CssLabColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Oklab(value),
        }
    }

    /// Retains a checked `oklch()` color; it shares the payload type of `lch()`.
    pub const fn from_oklch(value: CssLchColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Oklch(value),
        }
    }

    /// Retains a checked `color()` value in a predefined color space.
    pub const fn from_predefined(value: CssPredefinedColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Predefined(value),
        }
    }

    /// Retains a checked relative color.
    pub fn from_relative(value: CssRelativeColor) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::Relative(Box::new(value)),
        }
    }

    /// Retains a checked `color-mix()` color.
    pub const fn from_color_mix(value: CssColorMix) -> Self {
        Self {
            original_closed: true,
            representation: CssColorRepresentation::ColorMix(value),
        }
    }

    /// Returns whether this is the `currentcolor` keyword.
    #[must_use]
    pub const fn is_current_color(&self) -> bool {
        matches!(self.representation, CssColorRepresentation::CurrentColor)
    }

    /// Returns whether this is the `transparent` keyword.
    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        matches!(self.representation, CssColorRepresentation::Transparent)
    }

    /// Borrows the checked payload, when this is a hex color.
    #[must_use]
    pub const fn hex_value(&self) -> Option<&CssHexColor> {
        match &self.representation {
            CssColorRepresentation::Hex(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is a named color.
    #[must_use]
    pub const fn named(&self) -> Option<&CssNamedColor> {
        match &self.representation {
            CssColorRepresentation::Named(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the system color keyword, when this is a system color.
    #[must_use]
    pub const fn system(&self) -> Option<CssSystemColor> {
        match self.representation {
            CssColorRepresentation::System(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `rgb()` or `rgba()` color.
    #[must_use]
    pub const fn rgb_value(&self) -> Option<&CssRgbColor> {
        match &self.representation {
            CssColorRepresentation::Rgb(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `hsl()` or `hsla()` color.
    #[must_use]
    pub const fn hsl_value(&self) -> Option<&CssHslColor> {
        match &self.representation {
            CssColorRepresentation::Hsl(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `hwb()` color.
    #[must_use]
    pub const fn hwb_value(&self) -> Option<&CssHwbColor> {
        match &self.representation {
            CssColorRepresentation::Hwb(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is a `lab()` color.
    #[must_use]
    pub const fn lab_value(&self) -> Option<&CssLabColor> {
        match &self.representation {
            CssColorRepresentation::Lab(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `lch()` color.
    #[must_use]
    pub const fn lch_value(&self) -> Option<&CssLchColor> {
        match &self.representation {
            CssColorRepresentation::Lch(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `oklab()` color.
    #[must_use]
    pub const fn oklab_value(&self) -> Option<&CssLabColor> {
        match &self.representation {
            CssColorRepresentation::Oklab(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is an `oklch()` color.
    #[must_use]
    pub const fn oklch_value(&self) -> Option<&CssLchColor> {
        match &self.representation {
            CssColorRepresentation::Oklch(value) => Some(value),
            _ => None,
        }
    }

    /// Borrows the checked payload, when this is a `color()` value in a predefined color space.
    #[must_use]
    pub const fn predefined_value(&self) -> Option<&CssPredefinedColor> {
        match &self.representation {
            CssColorRepresentation::Predefined(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the typed preserved Color 5 relative-color branch, when present.
    #[must_use]
    pub const fn relative_value(&self) -> Option<&CssRelativeColor> {
        match &self.representation {
            CssColorRepresentation::Relative(value) => Some(value),
            _ => None,
        }
    }

    /// Returns the checked preserved Color 5 `color-mix()` branch, when present.
    #[must_use]
    pub const fn color_mix_value(&self) -> Option<&CssColorMix> {
        match &self.representation {
            CssColorRepresentation::ColorMix(value) => Some(value),
            _ => None,
        }
    }

    /// Returns a short lowercase label of the authored branch, such as `hex`, `named`, `rgb`, `color-mix` or `relative`.
    #[must_use]
    pub const fn kind_name(&self) -> &'static str {
        match &self.representation {
            CssColorRepresentation::CurrentColor => "currentcolor",
            CssColorRepresentation::Transparent => "transparent",
            CssColorRepresentation::Hex(_) => "hex",
            CssColorRepresentation::Named(_) => "named",
            CssColorRepresentation::System(_) => "system",
            CssColorRepresentation::Rgb(_) => "rgb",
            CssColorRepresentation::Hsl(_) => "hsl",
            CssColorRepresentation::Hwb(_) => "hwb",
            CssColorRepresentation::Lab(_) => "lab",
            CssColorRepresentation::Lch(_) => "lch",
            CssColorRepresentation::Oklab(_) => "oklab",
            CssColorRepresentation::Oklch(_) => "oklch",
            CssColorRepresentation::Custom(_) => "color",
            CssColorRepresentation::RelativeCustom(_) => "relative",
            CssColorRepresentation::Alpha(_) => "alpha",
            CssColorRepresentation::Predefined(_) => "color",
            CssColorRepresentation::Relative(_) => "relative",
            CssColorRepresentation::ColorMix(_) => "color-mix",
            CssColorRepresentation::LightDark(_) => "light-dark",
            CssColorRepresentation::ContrastColor(_) => "contrast-color",
            CssColorRepresentation::DeviceCmyk(_) => "device-cmyk",
        }
    }
}

/// A checked hex color retaining its digits as authored, without the leading `#`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssHexColor {
    digits: String,
}

impl CssHexColor {
    /// Checks that `digits` is 3, 4, 6 or 8 ASCII hexadecimal digits, without a leading `#`.
    ///
    /// Returns `None` for any other length or character.
    pub fn try_new(digits: impl Into<String>) -> Option<Self> {
        let digits = digits.into();
        (matches!(digits.len(), 3 | 4 | 6 | 8)
            && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then_some(Self { digits })
    }

    /// Returns the retained hexadecimal digits, without a leading `#`.
    #[must_use]
    pub fn digits(&self) -> &str {
        &self.digits
    }
}

/// A checked opaque named color retaining its canonical lowercase keyword.
///
/// Aliases with the same fixed sRGB value retain distinct names and identity.
/// The separate [`CssColor::transparent`] branch represents transparent black.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssNamedColor {
    name: String,
    rgb: [u8; 3],
}

impl CssNamedColor {
    pub(crate) const fn checked_rgb(&self) -> [u8; 3] {
        self.rgb
    }
    /// Checks `name` against the named color keywords, ASCII case-insensitively, and retains it lowercased.
    ///
    /// Returns `None` when `name` is not a named color keyword.
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        let rgb = surgeist_color::named::lookup(&name)?;
        Some(Self {
            name: name.to_ascii_lowercase(),
            rgb,
        })
    }

    /// Returns the canonical lowercase keyword.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// A `<system-color>` keyword, or a deprecated system color of Color 4 Appendix A, as authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssSystemColor {
    /// `Canvas`: background of application content or documents.
    Canvas,
    /// `CanvasText`: text in application content or documents.
    CanvasText,
    /// `LinkText`: text in non-active, non-visited links.
    LinkText,
    /// `VisitedText`: text in visited links.
    VisitedText,
    /// `ActiveText`: text in active links.
    ActiveText,
    /// `ButtonFace`: face background of push buttons.
    ButtonFace,
    /// `ButtonText`: text on push buttons.
    ButtonText,
    /// `ButtonBorder`: base border color of push buttons.
    ButtonBorder,
    /// `Field`: background of input fields.
    Field,
    /// `FieldText`: text in input fields.
    FieldText,
    /// `Highlight`: background of selected text.
    Highlight,
    /// `HighlightText`: text of selected text.
    HighlightText,
    /// `Mark`: background of specially marked text.
    Mark,
    /// `MarkText`: specially marked text.
    MarkText,
    /// `GrayText`: disabled text.
    GrayText,
    /// `SelectedItem`: background of selected items.
    SelectedItem,
    /// `SelectedItemText`: text of selected items.
    SelectedItemText,
    /// `AccentColor`: background of accented user interface controls.
    AccentColor,
    /// `AccentColorText`: text of accented user interface controls.
    AccentColorText,
    /// `ActiveBorder`: deprecated active window border; it computes as `ButtonBorder`.
    ActiveBorder,
    /// `ActiveCaption`: deprecated active window caption; it computes as `Canvas`.
    ActiveCaption,
    /// `AppWorkspace`: deprecated multiple-document-interface background; it computes as `Canvas`.
    AppWorkspace,
    /// `Background`: deprecated desktop background; it computes as `Canvas`.
    Background,
    /// `ButtonHighlight`: deprecated border facing the light on one-layer 3-D elements; it computes as `ButtonFace`.
    ButtonHighlight,
    /// `ButtonShadow`: deprecated border away from the light on one-layer 3-D elements; it computes as `ButtonFace`.
    ButtonShadow,
    /// `CaptionText`: deprecated text in captions, size boxes and scrollbar arrow boxes; it computes as `CanvasText`.
    CaptionText,
    /// `InactiveBorder`: deprecated inactive window border; it computes as `ButtonBorder`.
    InactiveBorder,
    /// `InactiveCaption`: deprecated inactive window caption; it computes as `Canvas`.
    InactiveCaption,
    /// `InactiveCaptionText`: deprecated text in an inactive caption; it computes as `GrayText`.
    InactiveCaptionText,
    /// `InfoBackground`: deprecated tooltip background; it computes as `Canvas`.
    InfoBackground,
    /// `InfoText`: deprecated tooltip text; it computes as `CanvasText`.
    InfoText,
    /// `Menu`: deprecated menu background; it computes as `Canvas`.
    Menu,
    /// `MenuText`: deprecated text in menus; it computes as `CanvasText`.
    MenuText,
    /// `Scrollbar`: deprecated scroll bar gray area; it computes as `Canvas`.
    Scrollbar,
    /// `ThreeDDarkShadow`: deprecated darker outer border away from the light on two-layer 3-D elements; it computes as `ButtonBorder`.
    ThreeDDarkShadow,
    /// `ThreeDFace`: deprecated face background of two-layer 3-D elements; it computes as `ButtonFace`.
    ThreeDFace,
    /// `ThreeDHighlight`: deprecated lighter outer border facing the light on two-layer 3-D elements; it computes as `ButtonBorder`.
    ThreeDHighlight,
    /// `ThreeDLightShadow`: deprecated darker inner border facing the light on two-layer 3-D elements; it computes as `ButtonBorder`.
    ThreeDLightShadow,
    /// `ThreeDShadow`: deprecated lighter inner border away from the light on two-layer 3-D elements; it computes as `ButtonBorder`.
    ThreeDShadow,
    /// `Window`: deprecated window background; it computes as `Canvas`.
    Window,
    /// `WindowFrame`: deprecated window frame; it computes as `ButtonBorder`.
    WindowFrame,
    /// `WindowText`: deprecated text in windows; it computes as `CanvasText`.
    WindowText,
}

/// The punctuation form in which a color function was authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssColorSyntax {
    /// The comma-separated legacy form, such as `rgb(0, 0, 0)`; each color constructor rejects the component combinations this form does not admit.
    Legacy,
    /// The space-separated modern form, with `/` before alpha, such as `rgb(0 0 0 / 50%)`.
    Modern,
}

/// One authored color component: a scalar or a calculation, retained without evaluation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorComponent {
    /// The missing component `none`.
    None,
    /// A `<number>` literal.
    Number(CssColorNumberLiteral),
    /// A `<percentage>` literal.
    Percentage(CssColorPercentageLiteral),
    /// A math function resolving to a `<number>`.
    NumberCalculation(CssCalculation<CssNumber>),
    /// A math function resolving to a `<number>` whose percentage basis remains contextual.
    HintedNumberCalculation(CssCalculation<CssHintedNumber>),
    /// A math function resolving to a `<percentage>`.
    PercentageCalculation(CssCalculation<CssPercentage>),
}

impl CssColorComponent {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) const fn domain(&self) -> Option<CssCalculationType> {
        match self {
            Self::None => None,
            Self::Number(_) | Self::NumberCalculation(_) | Self::HintedNumberCalculation(_) => {
                Some(CssCalculationType::Number)
            }
            Self::Percentage(_) | Self::PercentageCalculation(_) => {
                Some(CssCalculationType::Percentage)
            }
        }
    }
}

/// One authored hue: a scalar or a calculation, retained without evaluation.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorHue {
    /// The missing component `none`.
    None,
    /// A unitless `<number>` hue, in degrees.
    Number(CssColorNumberLiteral),
    /// An `<angle>` literal.
    Angle(CssNumericLiteral<CssAngle>),
    /// A math function resolving to a `<number>`.
    NumberCalculation(CssCalculation<CssNumber>),
    /// A math function resolving to an `<angle>`.
    AngleCalculation(CssCalculation<CssAngle>),
}

impl CssColorHue {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

/// A checked authored `rgb()` or `rgba()` color with three channels, retaining its punctuation form.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRgbColor {
    syntax: CssColorSyntax,
    channels: [CssColorComponent; 3],
    alpha: Option<CssColorComponent>,
}

impl CssRgbColor {
    /// Checks the syntax and component combination, retaining every supplied component.
    ///
    /// # Errors
    ///
    /// Returns an error when the legacy syntax is combined with `none` or with channels of different kinds
    /// (number or percentage), when a component expression is not valid in a pure context, or when the
    /// composed color exceeds the structural nesting limit.
    pub fn try_new(
        syntax: CssColorSyntax,
        channels: [CssColorComponent; 3],
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if syntax == CssColorSyntax::Legacy
            && (channels.iter().any(CssColorComponent::is_none)
                || alpha.as_ref().is_some_and(CssColorComponent::is_none)
                || channels.iter().any(|v| v.domain() != channels[0].domain()))
        {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::InvalidSyntax,
            ));
        }
        check_components_depth(channels.iter().chain(alpha.iter()))?;
        Ok(Self {
            syntax,
            channels,
            alpha,
        })
    }

    /// Returns the retained authored punctuation form.
    #[must_use]
    pub const fn syntax(&self) -> CssColorSyntax {
        self.syntax
    }

    /// Borrows the red, green and blue channels, in that order.
    #[must_use]
    pub const fn channels(&self) -> &[CssColorComponent; 3] {
        &self.channels
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// A checked authored `hsl()` or `hsla()` color, retaining its punctuation form.
#[derive(Clone, Debug, PartialEq)]
pub struct CssHslColor {
    syntax: CssColorSyntax,
    hue: CssColorHue,
    saturation: CssColorComponent,
    lightness: CssColorComponent,
    alpha: Option<CssColorComponent>,
}

impl CssHslColor {
    /// Checks the syntax and component combination, retaining every supplied component.
    ///
    /// # Errors
    ///
    /// Returns an error when the legacy syntax is combined with a `none` hue or alpha, or with a saturation
    /// or lightness that is not a percentage, when a component expression is not valid in a pure context, or
    /// when the composed color exceeds the structural nesting limit.
    pub fn try_new(
        syntax: CssColorSyntax,
        hue: CssColorHue,
        saturation: CssColorComponent,
        lightness: CssColorComponent,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if syntax == CssColorSyntax::Legacy
            && (hue.is_none()
                || !matches!(saturation.domain(), Some(CssCalculationType::Percentage))
                || !matches!(lightness.domain(), Some(CssCalculationType::Percentage))
                || alpha.as_ref().is_some_and(CssColorComponent::is_none))
        {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::InvalidSyntax,
            ));
        }
        check_color_depth(
            color_hue_depth(&hue)
                .max(color_component_depth(&saturation))
                .max(color_component_depth(&lightness))
                .max(alpha.as_ref().map_or(0, color_component_depth)),
        )?;
        check_color_context(
            color_hue_admits_pure_context(&hue)
                && [&saturation, &lightness]
                    .into_iter()
                    .chain(alpha.iter())
                    .all(color_component_admits_pure_context),
        )?;
        Ok(Self {
            syntax,
            hue,
            saturation,
            lightness,
            alpha,
        })
    }

    /// Returns the retained authored punctuation form.
    #[must_use]
    pub const fn syntax(&self) -> CssColorSyntax {
        self.syntax
    }

    /// Borrows the authored hue.
    #[must_use]
    pub const fn hue(&self) -> &CssColorHue {
        &self.hue
    }

    /// Borrows the authored saturation.
    #[must_use]
    pub const fn saturation(&self) -> &CssColorComponent {
        &self.saturation
    }

    /// Borrows the authored lightness.
    #[must_use]
    pub const fn lightness(&self) -> &CssColorComponent {
        &self.lightness
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// A checked authored `hwb()` color.
#[derive(Clone, Debug, PartialEq)]
pub struct CssHwbColor {
    hue: CssColorHue,
    whiteness: CssColorComponent,
    blackness: CssColorComponent,
    alpha: Option<CssColorComponent>,
}

impl CssHwbColor {
    /// Retains every supplied component after checking the composed color.
    ///
    /// # Errors
    ///
    /// Returns an error when a component expression is not valid in a pure context or when the composed
    /// color exceeds the structural nesting limit.
    pub fn try_new(
        hue: CssColorHue,
        whiteness: CssColorComponent,
        blackness: CssColorComponent,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        check_color_depth(
            color_hue_depth(&hue)
                .max(color_component_depth(&whiteness))
                .max(color_component_depth(&blackness))
                .max(alpha.as_ref().map_or(0, color_component_depth)),
        )?;
        check_color_context(
            color_hue_admits_pure_context(&hue)
                && [&whiteness, &blackness]
                    .into_iter()
                    .chain(alpha.iter())
                    .all(color_component_admits_pure_context),
        )?;
        Ok(Self {
            hue,
            whiteness,
            blackness,
            alpha,
        })
    }

    /// Borrows the authored hue.
    #[must_use]
    pub const fn hue(&self) -> &CssColorHue {
        &self.hue
    }

    /// Borrows the authored whiteness.
    #[must_use]
    pub const fn whiteness(&self) -> &CssColorComponent {
        &self.whiteness
    }

    /// Borrows the authored blackness.
    #[must_use]
    pub const fn blackness(&self) -> &CssColorComponent {
        &self.blackness
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// A checked authored Lab-family color with exact channel kinds.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLabColor {
    lightness: CssColorComponent,
    a: CssColorComponent,
    b: CssColorComponent,
    alpha: Option<CssColorComponent>,
}

impl CssLabColor {
    /// Retains every supplied component after checking the composed color.
    ///
    /// # Errors
    ///
    /// Returns an error when a component expression is not valid in a pure context or when the composed
    /// color exceeds the structural nesting limit.
    pub fn try_new(
        lightness: CssColorComponent,
        a: CssColorComponent,
        b: CssColorComponent,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        check_color_depth(
            color_component_depth(&lightness)
                .max(color_component_depth(&a))
                .max(color_component_depth(&b))
                .max(alpha.as_ref().map_or(0, color_component_depth)),
        )?;
        check_color_context(
            [&lightness, &a, &b]
                .into_iter()
                .chain(alpha.iter())
                .all(color_component_admits_pure_context),
        )?;
        Ok(Self {
            lightness,
            a,
            b,
            alpha,
        })
    }

    /// Borrows the authored lightness.
    #[must_use]
    pub const fn lightness(&self) -> &CssColorComponent {
        &self.lightness
    }

    /// Borrows the authored `a` axis component.
    #[must_use]
    pub const fn a(&self) -> &CssColorComponent {
        &self.a
    }

    /// Borrows the authored `b` axis component.
    #[must_use]
    pub const fn b(&self) -> &CssColorComponent {
        &self.b
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// A checked authored LCH-family color with an angle-capable hue.
#[derive(Clone, Debug, PartialEq)]
pub struct CssLchColor {
    lightness: CssColorComponent,
    chroma: CssColorComponent,
    hue: CssColorHue,
    alpha: Option<CssColorComponent>,
}

impl CssLchColor {
    /// Retains every supplied component after checking the composed color.
    ///
    /// # Errors
    ///
    /// Returns an error when a component expression is not valid in a pure context or when the composed
    /// color exceeds the structural nesting limit.
    pub fn try_new(
        lightness: CssColorComponent,
        chroma: CssColorComponent,
        hue: CssColorHue,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        check_color_depth(
            color_component_depth(&lightness)
                .max(color_component_depth(&chroma))
                .max(color_hue_depth(&hue))
                .max(alpha.as_ref().map_or(0, color_component_depth)),
        )?;
        check_color_context(
            color_hue_admits_pure_context(&hue)
                && [&lightness, &chroma]
                    .into_iter()
                    .chain(alpha.iter())
                    .all(color_component_admits_pure_context),
        )?;
        Ok(Self {
            lightness,
            chroma,
            hue,
            alpha,
        })
    }

    /// Borrows the authored lightness.
    #[must_use]
    pub const fn lightness(&self) -> &CssColorComponent {
        &self.lightness
    }

    /// Borrows the authored chroma.
    #[must_use]
    pub const fn chroma(&self) -> &CssColorComponent {
        &self.chroma
    }

    /// Borrows the authored hue.
    #[must_use]
    pub const fn hue(&self) -> &CssColorHue {
        &self.hue
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// Decoded custom-profile component name. Binding belongs to profile resolution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileComponentName(String);
impl CssColorProfileComponentName {
    /// Checks `decoded` as a custom-profile component name and retains it unchanged.
    ///
    /// Returns `None` when `decoded` is `none` (ASCII case-insensitively) or is not a valid identifier.
    pub fn try_new(decoded: impl Into<String>) -> Option<Self> {
        let decoded = decoded.into();
        if decoded.eq_ignore_ascii_case("none") {
            return None;
        }
        crate::component::CssComponentValue::try_ident(decoded.clone()).ok()?;
        Some(Self(decoded))
    }
    /// Returns the decoded component name.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
/// Intrinsic failure while composing an authored color graph, a `color-mix()`,
/// an interpolation method or a checked colour scalar.
///
/// The violated invariant is read through the `is_*` predicates; a scalar
/// failure also exposes its [`Self::origin`] and, when a component was
/// malformed, its [`Self::component_error`], which is the error's source.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorConstructionError {
    kind: ColorConstructionKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ColorConstructionKind {
    EmptyComponents,
    EmptyMixComponents,
    InvalidSyntax,
    NestingLimit,
    CapacityOverflow,
    InvalidExpressionEnvironment,
    HueRequiresPolarSpace,
    Scalar(Box<ColorScalarFailure>),
}
/// A checked colour scalar outside its range, or a malformed scalar component.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ColorScalarFailure {
    origin: crate::component::CssValueOrigin,
    component: Option<crate::component::CssComponentValueError>,
}
impl CssColorConstructionError {
    pub(crate) const fn new(kind: ColorConstructionKind) -> Self {
        Self { kind }
    }
    pub(crate) fn scalar_out_of_range(origin: crate::component::CssValueOrigin) -> Self {
        Self::new(ColorConstructionKind::Scalar(Box::new(
            ColorScalarFailure {
                origin,
                component: None,
            },
        )))
    }
    crate::diagnostic::error::kind_predicates! { kind;
        /// Returns whether a color or `color-mix()` was given no components.
        is_empty_components => ColorConstructionKind::EmptyComponents
            | ColorConstructionKind::EmptyMixComponents,
        /// Returns whether the syntax and component combination is invalid.
        is_invalid_syntax => ColorConstructionKind::InvalidSyntax,
        /// Returns whether the composed color exceeds the structural nesting limit.
        is_nesting_limit => ColorConstructionKind::NestingLimit,
        /// Returns whether a structural size calculation overflowed.
        is_capacity_overflow => ColorConstructionKind::CapacityOverflow,
        /// Returns whether an expression's environment does not match its color.
        is_invalid_expression_environment => ColorConstructionKind::InvalidExpressionEnvironment,
        /// Returns whether an explicit hue strategy was given a non-polar color space.
        is_hue_requires_polar_space => ColorConstructionKind::HueRequiresPolarSpace,
    }
    /// Returns whether a checked colour scalar lies outside its range.
    #[must_use]
    pub fn is_out_of_range(&self) -> bool {
        matches!(&self.kind, ColorConstructionKind::Scalar(failure) if failure.component.is_none())
    }
    /// Returns the supplied origin of a rejected colour scalar.
    #[must_use]
    pub fn origin(&self) -> Option<&crate::component::CssValueOrigin> {
        match &self.kind {
            ColorConstructionKind::Scalar(failure) => Some(&failure.origin),
            _ => None,
        }
    }
    /// Returns the component failure of a malformed colour scalar.
    #[must_use]
    pub fn component_error(&self) -> Option<&crate::component::CssComponentValueError> {
        match &self.kind {
            ColorConstructionKind::Scalar(failure) => failure.component.as_ref(),
            _ => None,
        }
    }
}
impl From<crate::component::CssComponentValueError> for CssColorConstructionError {
    fn from(source: crate::component::CssComponentValueError) -> Self {
        Self::new(ColorConstructionKind::Scalar(Box::new(
            ColorScalarFailure {
                origin: source.origin().clone(),
                component: Some(source),
            },
        )))
    }
}
impl std::fmt::Display for CssColorConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match &self.kind {
            ColorConstructionKind::EmptyComponents => "color requires at least one component",
            ColorConstructionKind::EmptyMixComponents => {
                "color-mix requires at least one component"
            }
            ColorConstructionKind::InvalidSyntax => {
                "invalid color syntax and component combination"
            }
            ColorConstructionKind::NestingLimit => "color nesting limit exceeded",
            ColorConstructionKind::CapacityOverflow => "color capacity overflow",
            ColorConstructionKind::InvalidExpressionEnvironment => {
                "color expression environment mismatch"
            }
            ColorConstructionKind::HueRequiresPolarSpace => {
                "hue interpolation requires a polar color space"
            }
            ColorConstructionKind::Scalar(failure) if failure.component.is_some() => {
                "invalid color scalar: the component is not a color scalar"
            }
            ColorConstructionKind::Scalar(_) => "invalid color scalar: the value is out of range",
        })
    }
}
impl std::error::Error for CssColorConstructionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.component_error()
            .map(|error| error as &(dyn std::error::Error + 'static))
    }
}
impl From<ColorGraphDepthError> for CssColorConstructionError {
    fn from(value: ColorGraphDepthError) -> Self {
        Self::new(match value {
            ColorGraphDepthError::NestingLimit => ColorConstructionKind::NestingLimit,
            ColorGraphDepthError::CapacityOverflow => ColorConstructionKind::CapacityOverflow,
        })
    }
}
fn check_color_depth(child_depth: u32) -> Result<(), CssColorConstructionError> {
    composed_color_depth(child_depth).map(|_| ())
}
fn check_components_depth<'a>(
    values: impl Iterator<Item = &'a CssColorComponent>,
) -> Result<u32, CssColorConstructionError> {
    let (child_depth, pure) = values.fold((0, true), |(depth, pure), value| {
        (
            depth.max(color_component_depth(value)),
            pure && color_component_admits_pure_context(value),
        )
    });
    let depth = composed_color_depth(child_depth)?;
    check_color_context(pure)?;
    Ok(depth)
}
fn check_color_context(admits_pure_context: bool) -> Result<(), CssColorConstructionError> {
    if admits_pure_context {
        Ok(())
    } else {
        Err(CssColorConstructionError::new(
            ColorConstructionKind::InvalidExpressionEnvironment,
        ))
    }
}
fn color_component_admits_pure_context(value: &CssColorComponent) -> bool {
    match value {
        CssColorComponent::NumberCalculation(value) => value.expression.admits_pure_context(),
        CssColorComponent::HintedNumberCalculation(value) => value.expression.admits_pure_context(),
        CssColorComponent::PercentageCalculation(value) => value.expression.admits_pure_context(),
        _ => true,
    }
}
fn color_hue_admits_pure_context(value: &CssColorHue) -> bool {
    match value {
        CssColorHue::NumberCalculation(value) => value.expression.admits_pure_context(),
        CssColorHue::AngleCalculation(value) => value.expression.admits_pure_context(),
        _ => true,
    }
}
fn composed_color_depth(child_depth: u32) -> Result<u32, CssColorConstructionError> {
    let depth = child_depth
        .checked_add(1)
        .ok_or(CssColorConstructionError::new(
            ColorConstructionKind::CapacityOverflow,
        ))?;
    if depth > crate::STRUCTURAL_NESTING_LIMIT {
        return Err(CssColorConstructionError::new(
            ColorConstructionKind::NestingLimit,
        ));
    }
    Ok(depth)
}
/// A nonempty authored custom `color()` without profile binding or evaluation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssCustomColor {
    profile: CssColorProfileName,
    channels: Vec<CssColorComponent>,
    alpha: Option<CssColorComponent>,
    nesting_depth: u32,
}
impl CssCustomColor {
    /// Checks that at least one channel is supplied and that the composed color is within the structural limits.
    ///
    /// # Errors
    ///
    /// Returns an error when `channels` is empty, when a component expression is not valid in a pure
    /// context, or when the composed color exceeds the structural nesting limit.
    pub fn try_new(
        profile: CssColorProfileName,
        channels: Vec<CssColorComponent>,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if channels.is_empty() {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::EmptyComponents,
            ));
        }
        let nesting_depth = check_components_depth(channels.iter().chain(alpha.iter()))?;
        Ok(Self {
            profile,
            channels,
            alpha,
            nesting_depth,
        })
    }
    /// Borrows the custom color profile name.
    pub const fn profile(&self) -> &CssColorProfileName {
        &self.profile
    }
    /// Borrows the authored channels, in authored order.
    pub fn channels(&self) -> &[CssColorComponent] {
        &self.channels
    }
    /// Borrows the authored alpha, when present; omission remains distinct from `none`.
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }
}
/// A custom relative color retaining unbound profile references in authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeCustomColor {
    source: Box<CssColor>,
    profile: CssColorProfileName,
    channels: Vec<crate::color::CssProfileColorExpression>,
    alpha: Option<crate::color::CssProfileColorExpression>,
    nesting_depth: u32,
}
impl CssRelativeCustomColor {
    /// Checks that at least one channel expression is supplied and that the composed color is within the structural limits.
    ///
    /// # Errors
    ///
    /// Returns an error when `channels` is empty or when the composed color exceeds the structural nesting limit.
    pub fn try_new(
        source: CssColor,
        profile: CssColorProfileName,
        channels: Vec<crate::color::CssProfileColorExpression>,
        alpha: Option<crate::color::CssProfileColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        if channels.is_empty() {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::EmptyComponents,
            ));
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
    /// Borrows the source (origin) color.
    pub const fn source(&self) -> &CssColor {
        &self.source
    }
    /// Borrows the custom color profile name.
    pub const fn profile(&self) -> &CssColorProfileName {
        &self.profile
    }
    /// Borrows the authored channel expressions, in authored order.
    pub fn channels(&self) -> &[crate::color::CssProfileColorExpression] {
        &self.channels
    }
    /// Borrows the authored alpha expression, when present; omission remains distinct from `none`.
    pub const fn alpha(&self) -> Option<&crate::color::CssProfileColorExpression> {
        self.alpha.as_ref()
    }
}
/// Authored `alpha(from ...)`, preserving omission independently from `none`.
#[derive(Clone, Debug, PartialEq)]
pub struct CssAlphaColor {
    source: Box<CssColor>,
    alpha: Option<CssRelativeColorExpression>,
    nesting_depth: u32,
}
impl CssAlphaColor {
    /// Checks that `alpha` is an alpha-slot expression of the alpha environment and that the composed color is within the structural limits.
    ///
    /// # Errors
    ///
    /// Returns an error when `alpha` was authored for another environment or result domain, or when the
    /// composed color exceeds the structural nesting limit.
    pub fn try_new(
        source: CssColor,
        alpha: Option<CssRelativeColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        if alpha.as_ref().is_some_and(|v| {
            v.environment() != CssRelativeColorEnvironment::Alpha
                || v.result_domain() != CssRelativeColorResultDomain::Alpha
        }) {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::InvalidExpressionEnvironment,
            ));
        }
        let expression_depth = match alpha.as_ref().map(CssRelativeColorExpression::value) {
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
    /// Borrows the source (origin) color.
    pub const fn source(&self) -> &CssColor {
        &self.source
    }
    /// Borrows the authored alpha expression, when present; omission remains distinct from `none`.
    pub const fn alpha(&self) -> Option<&CssRelativeColorExpression> {
        self.alpha.as_ref()
    }
}
/// Authored eligibility only; this does not promise a usable computed color.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteColorEligibility {
    /// The color needs no external context and no custom color profile.
    Eligible,
    /// The color depends on a custom color profile.
    ProfileDependent,
    /// The color depends on external context named by the exclusion.
    Contextual(CssAbsoluteColorExclusion),
}
/// The external context that excludes a color from absolute eligibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssAbsoluteColorExclusion {
    /// The color contains `currentcolor`.
    CurrentColor,
    /// The color contains a system color.
    SystemColor,
    /// The color contains `light-dark()`.
    LightDark,
    /// The color contains `contrast-color()`.
    ContrastColor,
    /// The color contains `device-cmyk()`.
    DeviceCmyk,
}
impl CssColor {
    /// Classifies the whole authored color graph.
    ///
    /// It is contextual when the graph contains `currentcolor`, a system color, `light-dark()`,
    /// `contrast-color()` or `device-cmyk()`; otherwise profile-dependent when it uses a custom
    /// color profile; otherwise eligible.
    pub fn absolute_eligibility(&self) -> CssAbsoluteColorEligibility {
        use CssAbsoluteColorEligibility as E;
        use CssAbsoluteColorExclusion as X;
        use CssColorRepresentation as R;
        let mut pending = vec![self];
        let mut profile = false;
        while let Some(color) = pending.pop() {
            match &color.representation {
                R::LightDark(_) => return E::Contextual(X::LightDark),
                R::ContrastColor(_) => return E::Contextual(X::ContrastColor),
                R::DeviceCmyk(_) => return E::Contextual(X::DeviceCmyk),
                R::CurrentColor => return E::Contextual(X::CurrentColor),
                R::System(_) => return E::Contextual(X::SystemColor),
                R::Custom(_) => profile = true,
                R::RelativeCustom(v) => {
                    profile = true;
                    pending.push(v.source());
                }
                R::Alpha(v) => pending.push(v.source()),
                R::Relative(v) => pending.push(v.source()),
                R::ColorMix(v) => {
                    profile |= v
                        .interpolation()
                        .is_some_and(|v| v.custom_profile().is_some());
                    pending.extend(v.components().iter().rev().map(|v| v.color()));
                }
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
            }
        }
        if profile {
            E::ProfileDependent
        } else {
            E::Eligible
        }
    }
}

/// A checked absolute `color()` value in a predefined Color 4 space.
#[derive(Clone, Debug, PartialEq)]
pub struct CssPredefinedColor {
    color_space: PredefinedColorSpace,
    channels: [CssColorComponent; 3],
    alpha: Option<CssColorComponent>,
}

/// The closed origin-channel environment for a preserved relative-color family.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorEnvironment {
    /// The `alpha(from ...)` environment, which exposes only `alpha`.
    Alpha,
    /// The `rgb()` environment, which exposes `r`, `g`, `b` and `alpha`.
    Rgb,
    /// The `hsl()` environment, which exposes `h`, `s`, `l` and `alpha`.
    Hsl,
    /// The `hwb()` environment, which exposes `h`, `w`, `b` and `alpha`.
    Hwb,
    /// The `lab()` environment, which exposes `l`, `a`, `b` and `alpha`.
    Lab,
    /// The `lch()` environment, which exposes `l`, `c`, `h` and `alpha`.
    Lch,
    /// The `oklab()` environment, which exposes `l`, `a`, `b` and `alpha`.
    Oklab,
    /// The `oklch()` environment, which exposes `l`, `c`, `h` and `alpha`.
    Oklch,
    /// The `color()` environment of an RGB-like predefined space, which exposes `r`, `g`, `b` and `alpha`.
    PredefinedRgb(PredefinedColorSpace),
    /// The `color()` environment of an XYZ predefined space, which exposes `x`, `y`, `z` and `alpha`.
    Xyz(PredefinedColorSpace),
}
impl CssRelativeColorEnvironment {
    pub(crate) const fn is_consistent(self) -> bool {
        !matches!(
            self,
            Self::PredefinedRgb(PredefinedColorSpace::XyzD50 | PredefinedColorSpace::XyzD65)
                | Self::Xyz(
                    PredefinedColorSpace::Srgb
                        | PredefinedColorSpace::SrgbLinear
                        | PredefinedColorSpace::DisplayP3
                        | PredefinedColorSpace::DisplayP3Linear
                        | PredefinedColorSpace::A98Rgb
                        | PredefinedColorSpace::ProphotoRgb
                        | PredefinedColorSpace::Rec2020
                )
        )
    }
}

/// The semantic result slot in which a relative-color expression is authored.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorResultDomain {
    /// A slot that takes a number or a percentage.
    NumberPercentage,
    /// A hue slot, which takes a number or an angle.
    Hue,
    /// The alpha slot.
    Alpha,
}

/// A channel name made available by one relative-color origin environment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorChannel {
    /// `r`: the red channel.
    R,
    /// `g`: the green channel.
    G,
    /// `b`: the blue channel, the blackness of `hwb()`, or the `b` axis of the Lab family, according to the environment.
    B,
    /// `h`: the hue.
    H,
    /// `s`: the saturation.
    S,
    /// `l`: the lightness.
    L,
    /// `w`: the whiteness.
    W,
    /// `a`: the `a` axis of the Lab family.
    A,
    /// `c`: the chroma.
    C,
    /// `x`: the X coordinate of an XYZ space.
    X,
    /// `y`: the Y coordinate of an XYZ space.
    Y,
    /// `z`: the Z coordinate of an XYZ space.
    Z,
    /// `alpha`: the alpha channel.
    Alpha,
}

/// The authored kind of one validated relative-color result expression.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorExpressionValue {
    /// The missing component `none`.
    None,
    /// A `<number>` literal.
    Number(CssColorNumberLiteral),
    /// A `<percentage>` literal.
    Percentage(CssColorPercentageLiteral),
    /// An `<angle>` literal.
    Angle(CssNumericLiteral<CssAngle>),
    /// A bare channel keyword of the environment.
    Channel(CssRelativeColorChannel),
    /// A calculation retained symbolically.
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

    /// Returns the authored declaration value of the calculation, with its original components.
    #[must_use]
    pub const fn authored(&self) -> &CssAuthoredDeclarationValue {
        &self.data.authored
    }

    /// Returns the result type of the calculation.
    #[must_use]
    pub const fn result_type(&self) -> CssCalculationType {
        self.data.result_type
    }

    /// Returns the relative color channels the calculation references.
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
pub struct CssRelativeColorExpression {
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
    value: CssRelativeColorExpressionValue,
    origin: crate::component::CssValueOrigin,
}

impl CssRelativeColorExpression {
    pub(crate) const fn new(
        environment: CssRelativeColorEnvironment,
        result_domain: CssRelativeColorResultDomain,
        value: CssRelativeColorExpressionValue,
        origin: crate::component::CssValueOrigin,
    ) -> Self {
        Self {
            environment,
            result_domain,
            value,
            origin,
        }
    }

    /// Returns the origin-channel environment the expression was checked against.
    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    /// Returns the result slot the expression was checked for.
    #[must_use]
    pub const fn result_domain(&self) -> CssRelativeColorResultDomain {
        self.result_domain
    }

    /// Returns the checked expression value.
    #[must_use]
    pub const fn value(&self) -> &CssRelativeColorExpressionValue {
        &self.value
    }
    /// Returns the origin of the authored expression.
    #[must_use]
    pub const fn origin(&self) -> &crate::component::CssValueOrigin {
        &self.origin
    }
}

/// A checked relative color with an exact three-channel typed result environment.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeColor {
    function: CssRelativeColorFunction,
    environment: CssRelativeColorEnvironment,
    source: Box<CssColor>,
    channels: [CssRelativeColorExpression; 3],
    alpha: Option<CssRelativeColorExpression>,
}

impl CssRelativeColor {
    /// Checks that every channel expression and the alpha expression match the environment and slot domains of `function`.
    ///
    /// # Errors
    ///
    /// Returns an error when a channel or alpha expression was authored for another environment or result
    /// domain, or when the composed color exceeds the structural nesting limit.
    pub fn try_new(
        function: CssRelativeColorFunction,
        source: CssColor,
        channels: [CssRelativeColorExpression; 3],
        alpha: Option<CssRelativeColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        let (environment, domains) = function.signature();
        for (value, domain) in channels.iter().zip(domains) {
            if value.environment() != environment || value.result_domain() != domain {
                return Err(CssColorConstructionError::new(
                    ColorConstructionKind::InvalidExpressionEnvironment,
                ));
            }
        }
        if alpha.as_ref().is_some_and(|value| {
            value.environment() != environment
                || value.result_domain() != CssRelativeColorResultDomain::Alpha
        }) {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::InvalidExpressionEnvironment,
            ));
        }
        let expression_depth = channels
            .iter()
            .chain(alpha.iter())
            .map(|value| match value.value() {
                CssRelativeColorExpressionValue::Calculation(v) => {
                    v.data.expression.component_nesting_depth()
                }
                _ => 0,
            })
            .max()
            .unwrap_or(0);
        composed_color_depth(expression_depth.max(authored_color_depth(&source)?))?;
        Ok(Self {
            function,
            environment,
            source: Box::new(source),
            channels,
            alpha,
        })
    }

    /// Returns the relative color function.
    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    /// Returns the origin-channel environment of the function.
    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    /// Borrows the source (origin) color.
    #[must_use]
    pub const fn source(&self) -> &CssColor {
        &self.source
    }

    /// Borrows the three channel result expressions, in the function's channel order.
    #[must_use]
    pub const fn channels(&self) -> &[CssRelativeColorExpression; 3] {
        &self.channels
    }

    /// Borrows the alpha result expression, when present; omission remains distinct from `none`.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssRelativeColorExpression> {
        self.alpha.as_ref()
    }
}

impl CssPredefinedColor {
    /// Checks the channel and alpha components and retains them with the predefined color space.
    ///
    /// # Errors
    ///
    /// Returns an error when a component expression is not valid in a pure context or when the composed
    /// color exceeds the structural nesting limit.
    pub fn try_new(
        color_space: PredefinedColorSpace,
        channels: [CssColorComponent; 3],
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        check_components_depth(channels.iter().chain(alpha.iter()))?;
        Ok(Self {
            color_space,
            channels,
            alpha,
        })
    }

    /// Returns the predefined color space of the `color()` value.
    #[must_use]
    pub const fn color_space(&self) -> PredefinedColorSpace {
        self.color_space
    }

    /// Borrows the three authored channels, in the order of the color space.
    #[must_use]
    pub const fn channels(&self) -> &[CssColorComponent; 3] {
        &self.channels
    }

    /// Borrows the authored alpha without applying parsed-value clamping.
    ///
    /// Omission, missing components, original coefficients and calculations
    /// retain their identities. Use [`Self::parsed_alpha`] for the intrinsic
    /// parsed-alpha view while keeping this authored representation intact.
    #[must_use]
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }

    /// Borrows the intrinsic parsed-alpha view without allocating or evaluating math.
    ///
    /// Omitted alpha defaults to opaque; `none` remains explicitly missing.
    /// Direct numbers and percentages are classified exactly after scaling and
    /// clamping to [0, 1]. Calculations remain borrowed and unresolved for their
    /// applicable phase. The authored component and its provenance do not change.
    #[must_use]
    pub fn parsed_alpha(&self) -> crate::color::CssParsedColorAlphaRef<'_> {
        crate::color::color_alpha::parsed_alpha(self.alpha())
    }
}

/// A checked literal percentage for one `color-mix()` component.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMixPercentage {
    literal: CssColorPercentageLiteral,
}
impl CssColorMixPercentage {
    /// Checks the exact literal coefficient against the inclusive 0..100 range.
    pub fn try_from_component(
        component: crate::component::CssComponentValue,
    ) -> Result<Self, CssColorConstructionError> {
        let literal = CssColorPercentageLiteral::try_from_component(component)?;
        if !crate::serialization::exact_decimal::LexicalDecimal::new(
            literal.numeric().representation(),
        )
        .in_percentage_range()
        {
            return Err(CssColorConstructionError::scalar_out_of_range(
                literal.origin().clone(),
            ));
        }
        Ok(Self { literal })
    }
    /// Borrows the exact checked percentage literal.
    #[must_use]
    pub const fn literal(&self) -> &CssColorPercentageLiteral {
        &self.literal
    }
}

/// A case-sensitive decoded custom color-profile identifier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssColorProfileName(String);

impl CssColorProfileName {
    /// Checks `decoded` as a custom color profile name, case-sensitively.
    ///
    /// Returns `None` unless `decoded` starts with `--` and is a valid identifier.
    pub fn try_new(decoded: impl Into<String>) -> Option<Self> {
        let decoded = decoded.into();
        if !decoded.starts_with("--") {
            return None;
        }
        crate::component::CssComponentValue::try_ident(decoded.clone()).ok()?;
        Some(Self(decoded))
    }
    /// Returns the decoded name, including its leading `--`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A checked predefined interpolation method or symbolic custom profile.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorInterpolation(ColorInterpolation);
#[derive(Clone, Debug, PartialEq)]
enum ColorInterpolation {
    Predefined(CssColorInterpolationMethod),
    Custom(CssColorProfileName),
}
impl CssColorInterpolation {
    /// Wraps an intrinsically checked predefined interpolation method.
    #[must_use]
    pub const fn from_predefined(method: CssColorInterpolationMethod) -> Self {
        Self(ColorInterpolation::Predefined(method))
    }
    /// Wraps a symbolic custom color profile name as the interpolation space.
    #[must_use]
    pub fn custom(name: CssColorProfileName) -> Self {
        Self(ColorInterpolation::Custom(name))
    }
    /// Returns the predefined interpolation method, or `None` for a custom profile.
    #[must_use]
    pub fn predefined(&self) -> Option<CssColorInterpolationMethod> {
        match &self.0 {
            ColorInterpolation::Predefined(value) => Some(*value),
            ColorInterpolation::Custom(_) => None,
        }
    }
    /// Returns the custom color profile name, or `None` for a predefined method.
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
pub struct CssColorMixWeight(ColorMixWeight);
#[derive(Clone, Debug, PartialEq)]
enum ColorMixWeight {
    Literal(CssColorMixPercentage),
    Calculation(CssCalculation<CssPercentage>),
}
impl CssColorMixWeight {
    /// Wraps a checked literal percentage as a weight.
    #[must_use]
    pub fn literal(value: CssColorMixPercentage) -> Self {
        Self(ColorMixWeight::Literal(value))
    }
    /// Wraps a percentage calculation as a weight.
    ///
    /// # Errors
    ///
    /// Returns an error when `value` is not exactly one math function, ignoring whitespace and comments.
    pub fn try_calculation(
        value: CssCalculation<CssPercentage>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        use crate::component::CssComponentValueRef as V;
        use crate::component::CssValueTokenRef as T;
        let mut significant = value
            .components()
            .items()
            .iter()
            .filter(|v| !matches!(v.view(), V::Comment(_) | V::Token(T::Whitespace(_))));
        let first = significant.next();
        if !first.is_some_and(|v| matches!(v.view(), V::Function(_)))
            || significant.next().is_some()
        {
            return Err(crate::component::CssComponentValueError::new(
                crate::component::values::CssComponentValueErrorKind::InvalidToken,
                first.map_or_else(|| value.origin().clone(), |v| v.origin().clone()),
            ));
        }
        Ok(Self(ColorMixWeight::Calculation(value)))
    }
    /// Returns the literal weight, or `None` for a calculated weight.
    #[must_use]
    pub fn literal_value(&self) -> Option<&CssColorMixPercentage> {
        match &self.0 {
            ColorMixWeight::Literal(v) => Some(v),
            ColorMixWeight::Calculation(_) => None,
        }
    }
    /// Returns the calculated weight, or `None` for a literal weight.
    #[must_use]
    pub fn calculation(&self) -> Option<&CssCalculation<CssPercentage>> {
        match &self.0 {
            ColorMixWeight::Calculation(v) => Some(v),
            ColorMixWeight::Literal(_) => None,
        }
    }
}

/// One authored color and its optional literal or calculated weight.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMixComponent {
    color: Box<CssColor>,
    weight: Option<CssColorMixWeight>,
}
impl CssColorMixComponent {
    /// Pairs a color with its optional weight; omission remains distinct from an explicit weight.
    #[must_use]
    pub fn new(color: CssColor, weight: Option<CssColorMixWeight>) -> Self {
        Self {
            color: Box::new(color),
            weight,
        }
    }
    /// Borrows the authored color.
    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
    /// Borrows the authored weight, when present.
    #[must_use]
    pub const fn weight(&self) -> Option<&CssColorMixWeight> {
        self.weight.as_ref()
    }
}

/// A nonempty ordered authored mix with optional explicit interpolation.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColorMix {
    interpolation: Option<CssColorInterpolation>,
    components: Vec<CssColorMixComponent>,
    nesting_depth: u32,
}
impl CssColorMix {
    /// Checks that at least one component is supplied and that the composed color is within the structural limits.
    ///
    /// # Errors
    ///
    /// Returns an error when `components` is empty or when the composed color exceeds the structural
    /// nesting limit.
    pub fn try_new(
        interpolation: Option<CssColorInterpolation>,
        components: Vec<CssColorMixComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if components.is_empty() {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::EmptyMixComponents,
            ));
        }
        let nesting_depth = color_mix_depth(&components)?;
        Ok(Self {
            interpolation,
            components,
            nesting_depth,
        })
    }
    /// Returns the explicit interpolation (`in <space>`), or `None` when it was omitted.
    #[must_use]
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }
    /// Borrows the authored components, in authored order.
    #[must_use]
    pub fn components(&self) -> &[CssColorMixComponent] {
        &self.components
    }
}

#[derive(Clone, Copy, Debug)]
enum ColorGraphDepthError {
    NestingLimit,
    CapacityOverflow,
}

fn color_mix_depth(components: &[CssColorMixComponent]) -> Result<u32, ColorGraphDepthError> {
    let mut depth = 1;
    for component in components {
        depth = depth.max(
            1u32.checked_add(authored_color_depth(component.color())?)
                .ok_or(ColorGraphDepthError::CapacityOverflow)?,
        );
        if let Some(value) = component.weight().and_then(CssColorMixWeight::calculation) {
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

fn color_component_depth(value: &CssColorComponent) -> u32 {
    match value {
        CssColorComponent::NumberCalculation(v) => v.components().nesting_depth(),
        CssColorComponent::HintedNumberCalculation(v) => v.components().nesting_depth(),
        CssColorComponent::PercentageCalculation(v) => v.components().nesting_depth(),
        _ => 0,
    }
}
fn color_hue_depth(value: &CssColorHue) -> u32 {
    match value {
        CssColorHue::NumberCalculation(v) => v.components().nesting_depth(),
        CssColorHue::AngleCalculation(v) => v.components().nesting_depth(),
        _ => 0,
    }
}
fn authored_color_depth(mut color: &CssColor) -> Result<u32, ColorGraphDepthError> {
    use CssColorRepresentation as R;
    let mut ancestors = 0u32;
    let mut maximum = 0u32;
    loop {
        let alpha = |v: &Option<CssColorComponent>| v.as_ref().map_or(0, color_component_depth);
        let channels =
            |v: &[CssColorComponent]| v.iter().map(color_component_depth).max().unwrap_or(0);
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
            R::DeviceCmyk(v) => 1 + channels(&v.channels).max(alpha(&v.alpha)),
            // Every mix constructor caches the complete checked subtree depth.
            R::ColorMix(v) => v.nesting_depth,
            R::LightDark(v) => v.nesting_depth,
            R::ContrastColor(v) => v.nesting_depth,
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

/// A checked authored predefined interpolation method.
/// Explicit hue strategies are valid only for polar color spaces.
///
/// ```compile_fail,E0451
/// use surgeist_color::ColorSpace;
/// use surgeist_css::color::CssColorInterpolationMethod;
/// let unchecked = CssColorInterpolationMethod {
///     space: ColorSpace::Hsl,
///     hue: None,
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssColorInterpolationMethod {
    space: ColorSpace,
    hue: Option<HueInterpolation>,
}

impl CssColorInterpolationMethod {
    /// Checks the polar-only hue grammar without resolving or interpolating colors.
    pub const fn try_new(
        space: ColorSpace,
        hue: Option<HueInterpolation>,
    ) -> Result<Self, CssColorConstructionError> {
        if hue.is_some() && !space.is_polar() {
            return Err(CssColorConstructionError::new(
                ColorConstructionKind::HueRequiresPolarSpace,
            ));
        }
        Ok(Self { space, hue })
    }

    /// Returns the interpolation color space.
    #[must_use]
    pub const fn space(&self) -> ColorSpace {
        self.space
    }

    /// Returns the authored hue strategy, or `None` when it was omitted.
    #[must_use]
    pub const fn hue(&self) -> Option<HueInterpolation> {
        self.hue
    }

    /// Returns the Color4 baseline hue strategy when the host supplies no override.
    /// An omitted polar strategy uses `Shorter`; rectangular spaces have no hue
    /// strategy. The authored omission remains available through [`Self::hue`].
    /// This does not select a host's policy or execute color interpolation.
    #[must_use]
    pub const fn effective_hue(&self) -> Option<HueInterpolation> {
        if self.space.is_polar() {
            match self.hue {
                Some(hue) => Some(hue),
                None => Some(HueInterpolation::Shorter),
            }
        } else {
            None
        }
    }
}

/// The color function of a relative color, which fixes its origin environment and channel domains.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssRelativeColorFunction {
    /// `rgb()`.
    Rgb,
    /// `hsl()`.
    Hsl,
    /// `hwb()`.
    Hwb,
    /// `lab()`.
    Lab,
    /// `lch()`.
    Lch,
    /// `oklab()`.
    Oklab,
    /// `oklch()`.
    Oklch,
    /// `color()` in the given predefined color space.
    Color(PredefinedColorSpace),
}

impl CssRelativeColorFunction {
    /// Returns the origin-channel environment and the result domain of each of the three channels.
    pub const fn signature(
        self,
    ) -> (
        CssRelativeColorEnvironment,
        [CssRelativeColorResultDomain; 3],
    ) {
        use CssRelativeColorEnvironment as E;
        use CssRelativeColorResultDomain::{Hue, NumberPercentage as N};
        match self {
            Self::Rgb => (E::Rgb, [N; 3]),
            Self::Hsl => (E::Hsl, [Hue, N, N]),
            Self::Hwb => (E::Hwb, [Hue, N, N]),
            Self::Lab => (E::Lab, [N; 3]),
            Self::Lch => (E::Lch, [N, N, Hue]),
            Self::Oklab => (E::Oklab, [N; 3]),
            Self::Oklch => (E::Oklch, [N, N, Hue]),
            Self::Color(space @ (PredefinedColorSpace::XyzD50 | PredefinedColorSpace::XyzD65)) => {
                (E::Xyz(space), [N; 3])
            }
            Self::Color(space) => (E::PredefinedRgb(space), [N; 3]),
        }
    }
    /// Returns the number of non-alpha channels, which is three for every function.
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
