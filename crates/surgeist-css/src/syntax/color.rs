use super::{
    CssAngleCalculation, CssAngleLiteral, CssAuthoredDeclarationValue, CssCalculationExpression,
    CssCalculationExpressionRef, CssCalculationType, CssColorNumberLiteral,
    CssColorPercentageLiteral, CssColorScalarError, CssNumberCalculation, CssPercentageCalculation,
};

mod serialization;

/// An authored color retaining its specified syntax and symbolic dependencies.
#[derive(Clone, Debug, PartialEq)]
pub struct CssColor {
    representation: CssColorRepresentation,
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
                || channels
                    .iter()
                    .any(|value| value.domain() != Some(CssCalculationType::Number)))
        {
            return Err(CssColorConstructionError::InvalidSyntax);
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
            .ok_or(CssColorConstructionError::CapacityOverflow)?;
        if nesting_depth > crate::STRUCTURAL_NESTING_LIMIT {
            return Err(CssColorConstructionError::NestingLimit);
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

impl CssColor {
    /// Retains an uncalibrated device input without conversion or ink clamping.
    pub fn from_device_cmyk(value: CssDeviceCmykColor) -> Self {
        Self {
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

    pub const fn from_custom(value: CssCustomColor) -> Self {
        Self {
            representation: CssColorRepresentation::Custom(value),
        }
    }
    pub fn from_relative_custom(value: CssRelativeCustomColor) -> Self {
        Self {
            representation: CssColorRepresentation::RelativeCustom(Box::new(value)),
        }
    }
    pub const fn from_alpha(value: CssAlphaColor) -> Self {
        Self {
            representation: CssColorRepresentation::Alpha(value),
        }
    }
    pub const fn custom_value(&self) -> Option<&CssCustomColor> {
        match &self.representation {
            CssColorRepresentation::Custom(v) => Some(v),
            _ => None,
        }
    }
    pub const fn relative_custom_value(&self) -> Option<&CssRelativeCustomColor> {
        match &self.representation {
            CssColorRepresentation::RelativeCustom(v) => Some(v),
            _ => None,
        }
    }
    pub const fn alpha_value(&self) -> Option<&CssAlphaColor> {
        match &self.representation {
            CssColorRepresentation::Alpha(v) => Some(v),
            _ => None,
        }
    }

    pub const fn current_color() -> Self {
        Self {
            representation: CssColorRepresentation::CurrentColor,
        }
    }

    pub const fn transparent() -> Self {
        Self {
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
    /// use surgeist_css::{CssColor, CssHexColor, CssNamedColor};
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

    pub const fn from_hex(value: CssHexColor) -> Self {
        Self {
            representation: CssColorRepresentation::Hex(value),
        }
    }

    pub const fn from_named(value: CssNamedColor) -> Self {
        Self {
            representation: CssColorRepresentation::Named(value),
        }
    }

    pub const fn from_system(value: CssSystemColor) -> Self {
        Self {
            representation: CssColorRepresentation::System(value),
        }
    }

    pub const fn from_rgb(value: CssRgbColor) -> Self {
        Self {
            representation: CssColorRepresentation::Rgb(value),
        }
    }

    pub const fn from_hsl(value: CssHslColor) -> Self {
        Self {
            representation: CssColorRepresentation::Hsl(value),
        }
    }

    pub const fn from_hwb(value: CssHwbColor) -> Self {
        Self {
            representation: CssColorRepresentation::Hwb(value),
        }
    }

    pub const fn from_lab(value: CssLabColor) -> Self {
        Self {
            representation: CssColorRepresentation::Lab(value),
        }
    }

    pub const fn from_lch(value: CssLchColor) -> Self {
        Self {
            representation: CssColorRepresentation::Lch(value),
        }
    }

    pub const fn from_oklab(value: CssLabColor) -> Self {
        Self {
            representation: CssColorRepresentation::Oklab(value),
        }
    }

    pub const fn from_oklch(value: CssLchColor) -> Self {
        Self {
            representation: CssColorRepresentation::Oklch(value),
        }
    }

    pub const fn from_predefined(value: CssPredefinedColor) -> Self {
        Self {
            representation: CssColorRepresentation::Predefined(value),
        }
    }

    pub fn from_relative(value: CssRelativeColor) -> Self {
        Self {
            representation: CssColorRepresentation::Relative(Box::new(value)),
        }
    }

    pub const fn from_color_mix(value: CssColorMix) -> Self {
        Self {
            representation: CssColorRepresentation::ColorMix(value),
        }
    }

    #[must_use]
    pub const fn is_current_color(&self) -> bool {
        matches!(self.representation, CssColorRepresentation::CurrentColor)
    }

    #[must_use]
    pub const fn is_transparent(&self) -> bool {
        matches!(self.representation, CssColorRepresentation::Transparent)
    }

    #[must_use]
    pub const fn hex_value(&self) -> Option<&CssHexColor> {
        match &self.representation {
            CssColorRepresentation::Hex(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn named(&self) -> Option<&CssNamedColor> {
        match &self.representation {
            CssColorRepresentation::Named(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn system(&self) -> Option<CssSystemColor> {
        match self.representation {
            CssColorRepresentation::System(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn rgb_value(&self) -> Option<&CssRgbColor> {
        match &self.representation {
            CssColorRepresentation::Rgb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hsl_value(&self) -> Option<&CssHslColor> {
        match &self.representation {
            CssColorRepresentation::Hsl(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn hwb_value(&self) -> Option<&CssHwbColor> {
        match &self.representation {
            CssColorRepresentation::Hwb(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lab_value(&self) -> Option<&CssLabColor> {
        match &self.representation {
            CssColorRepresentation::Lab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn lch_value(&self) -> Option<&CssLchColor> {
        match &self.representation {
            CssColorRepresentation::Lch(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklab_value(&self) -> Option<&CssLabColor> {
        match &self.representation {
            CssColorRepresentation::Oklab(value) => Some(value),
            _ => None,
        }
    }

    #[must_use]
    pub const fn oklch_value(&self) -> Option<&CssLchColor> {
        match &self.representation {
            CssColorRepresentation::Oklch(value) => Some(value),
            _ => None,
        }
    }

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssHexColor {
    digits: String,
}

impl CssHexColor {
    pub fn try_new(digits: impl Into<String>) -> Option<Self> {
        let digits = digits.into();
        (matches!(digits.len(), 3 | 4 | 6 | 8)
            && digits.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .then_some(Self { digits })
    }

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
    pub fn try_new(name: impl Into<String>) -> Option<Self> {
        let name = name.into();
        let (red, green, blue) = cssparser::color::parse_named_color(&name).ok()?;
        Some(Self {
            name: name.to_ascii_lowercase(),
            rgb: [red, green, blue],
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
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
pub enum CssColorSyntax {
    Legacy,
    Modern,
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorComponent {
    None,
    Number(CssColorNumberLiteral),
    Percentage(CssColorPercentageLiteral),
    NumberCalculation(CssNumberCalculation),
    PercentageCalculation(CssPercentageCalculation),
}

impl CssColorComponent {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub(crate) const fn domain(&self) -> Option<CssCalculationType> {
        match self {
            Self::None => None,
            Self::Number(_) | Self::NumberCalculation(_) => Some(CssCalculationType::Number),
            Self::Percentage(_) | Self::PercentageCalculation(_) => {
                Some(CssCalculationType::Percentage)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssColorHue {
    None,
    Number(CssColorNumberLiteral),
    Angle(CssAngleLiteral),
    NumberCalculation(CssNumberCalculation),
    AngleCalculation(CssAngleCalculation),
}

impl CssColorHue {
    pub(crate) const fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssRgbColor {
    syntax: CssColorSyntax,
    channels: [CssColorComponent; 3],
    alpha: Option<CssColorComponent>,
}

impl CssRgbColor {
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
            return Err(CssColorConstructionError::InvalidSyntax);
        }
        check_components_depth(channels.iter().chain(alpha.iter()))?;
        Ok(Self {
            syntax,
            channels,
            alpha,
        })
    }

    #[must_use]
    pub const fn syntax(&self) -> CssColorSyntax {
        self.syntax
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssHslColor {
    syntax: CssColorSyntax,
    hue: CssColorHue,
    saturation: CssColorComponent,
    lightness: CssColorComponent,
    alpha: Option<CssColorComponent>,
}

impl CssHslColor {
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
            return Err(CssColorConstructionError::InvalidSyntax);
        }
        check_color_depth(
            color_hue_depth(&hue)
                .max(color_component_depth(&saturation))
                .max(color_component_depth(&lightness))
                .max(alpha.as_ref().map_or(0, color_component_depth)),
        )?;
        Ok(Self {
            syntax,
            hue,
            saturation,
            lightness,
            alpha,
        })
    }

    #[must_use]
    pub const fn syntax(&self) -> CssColorSyntax {
        self.syntax
    }

    #[must_use]
    pub const fn hue(&self) -> &CssColorHue {
        &self.hue
    }

    #[must_use]
    pub const fn saturation(&self) -> &CssColorComponent {
        &self.saturation
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CssHwbColor {
    hue: CssColorHue,
    whiteness: CssColorComponent,
    blackness: CssColorComponent,
    alpha: Option<CssColorComponent>,
}

impl CssHwbColor {
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
        Ok(Self {
            hue,
            whiteness,
            blackness,
            alpha,
        })
    }

    #[must_use]
    pub const fn hue(&self) -> &CssColorHue {
        &self.hue
    }

    #[must_use]
    pub const fn whiteness(&self) -> &CssColorComponent {
        &self.whiteness
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
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
        Ok(Self {
            lightness,
            a,
            b,
            alpha,
        })
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn a(&self) -> &CssColorComponent {
        &self.a
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
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
        Ok(Self {
            lightness,
            chroma,
            hue,
            alpha,
        })
    }

    #[must_use]
    pub const fn lightness(&self) -> &CssColorComponent {
        &self.lightness
    }

    #[must_use]
    pub const fn chroma(&self) -> &CssColorComponent {
        &self.chroma
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
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
pub enum CssColorConstructionError {
    EmptyComponents,
    InvalidSyntax,
    InvalidComponent,
    NestingLimit,
    CapacityOverflow,
    InvalidExpressionEnvironment,
}
impl std::fmt::Display for CssColorConstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::EmptyComponents => "color requires at least one component",
            Self::InvalidSyntax => "invalid color syntax and component combination",
            Self::InvalidComponent => "invalid color component domain",
            Self::NestingLimit => "color nesting limit exceeded",
            Self::CapacityOverflow => "color capacity overflow",
            Self::InvalidExpressionEnvironment => "color expression environment mismatch",
        })
    }
}
impl std::error::Error for CssColorConstructionError {}
impl From<ColorGraphDepthError> for CssColorConstructionError {
    fn from(value: ColorGraphDepthError) -> Self {
        match value {
            ColorGraphDepthError::NestingLimit => Self::NestingLimit,
            ColorGraphDepthError::CapacityOverflow => Self::CapacityOverflow,
        }
    }
}
fn check_color_depth(child_depth: u32) -> Result<(), CssColorConstructionError> {
    composed_color_depth(child_depth).map(|_| ())
}
fn check_components_depth<'a>(
    values: impl Iterator<Item = &'a CssColorComponent>,
) -> Result<(), CssColorConstructionError> {
    check_color_depth(values.map(color_component_depth).max().unwrap_or(0))
}
fn composed_color_depth(child_depth: u32) -> Result<u32, CssColorConstructionError> {
    let depth = child_depth
        .checked_add(1)
        .ok_or(CssColorConstructionError::CapacityOverflow)?;
    if depth > crate::STRUCTURAL_NESTING_LIMIT {
        return Err(CssColorConstructionError::NestingLimit);
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
    pub fn try_new(
        profile: CssColorProfileName,
        channels: Vec<CssColorComponent>,
        alpha: Option<CssColorComponent>,
    ) -> Result<Self, CssColorConstructionError> {
        if channels.is_empty() {
            return Err(CssColorConstructionError::EmptyComponents);
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
    pub fn channels(&self) -> &[CssColorComponent] {
        &self.channels
    }
    pub const fn alpha(&self) -> Option<&CssColorComponent> {
        self.alpha.as_ref()
    }
}
/// A custom relative color retaining unbound profile references in authored order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssRelativeCustomColor {
    source: Box<CssColor>,
    profile: CssColorProfileName,
    channels: Vec<crate::CssProfileColorExpression>,
    alpha: Option<crate::CssProfileColorExpression>,
    nesting_depth: u32,
}
impl CssRelativeCustomColor {
    pub fn try_new(
        source: CssColor,
        profile: CssColorProfileName,
        channels: Vec<crate::CssProfileColorExpression>,
        alpha: Option<crate::CssProfileColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        if channels.is_empty() {
            return Err(CssColorConstructionError::EmptyComponents);
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
    pub const fn source(&self) -> &CssColor {
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
pub struct CssAlphaColor {
    source: Box<CssColor>,
    alpha: Option<CssRelativeColorExpression>,
    nesting_depth: u32,
}
impl CssAlphaColor {
    pub fn try_new(
        source: CssColor,
        alpha: Option<CssRelativeColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        if alpha.as_ref().is_some_and(|v| {
            v.environment() != CssRelativeColorEnvironment::Alpha
                || v.result_domain() != CssRelativeColorResultDomain::Alpha
        }) {
            return Err(CssColorConstructionError::InvalidExpressionEnvironment);
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
    pub const fn source(&self) -> &CssColor {
        &self.source
    }
    pub const fn alpha(&self) -> Option<&CssRelativeColorExpression> {
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
impl CssColor {
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
    color_space: CssPredefinedColorSpace,
    channels: [CssColorComponent; 3],
    alpha: Option<CssColorComponent>,
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
impl CssRelativeColorEnvironment {
    pub(crate) const fn is_consistent(self) -> bool {
        !matches!(
            self,
            Self::PredefinedRgb(CssPredefinedColorSpace::XyzD50 | CssPredefinedColorSpace::XyzD65)
                | Self::Xyz(
                    CssPredefinedColorSpace::Srgb
                        | CssPredefinedColorSpace::SrgbLinear
                        | CssPredefinedColorSpace::DisplayP3
                        | CssPredefinedColorSpace::DisplayP3Linear
                        | CssPredefinedColorSpace::A98Rgb
                        | CssPredefinedColorSpace::ProphotoRgb
                        | CssPredefinedColorSpace::Rec2020
                )
        )
    }
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
    None,
    Number(CssColorNumberLiteral),
    Percentage(CssColorPercentageLiteral),
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
pub struct CssRelativeColorExpression {
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
    value: CssRelativeColorExpressionValue,
    origin: crate::CssValueOrigin,
}

impl CssRelativeColorExpression {
    pub(crate) const fn new(
        environment: CssRelativeColorEnvironment,
        result_domain: CssRelativeColorResultDomain,
        value: CssRelativeColorExpressionValue,
        origin: crate::CssValueOrigin,
    ) -> Self {
        Self {
            environment,
            result_domain,
            value,
            origin,
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
    #[must_use]
    pub const fn origin(&self) -> &crate::CssValueOrigin {
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
    pub fn try_new(
        function: CssRelativeColorFunction,
        source: CssColor,
        channels: [CssRelativeColorExpression; 3],
        alpha: Option<CssRelativeColorExpression>,
    ) -> Result<Self, CssColorConstructionError> {
        let (environment, domains) = function.signature();
        for (value, domain) in channels.iter().zip(domains) {
            if value.environment() != environment || value.result_domain() != domain {
                return Err(CssColorConstructionError::InvalidExpressionEnvironment);
            }
        }
        if alpha.as_ref().is_some_and(|value| {
            value.environment() != environment
                || value.result_domain() != CssRelativeColorResultDomain::Alpha
        }) {
            return Err(CssColorConstructionError::InvalidExpressionEnvironment);
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

    #[must_use]
    pub const fn function(&self) -> &CssRelativeColorFunction {
        &self.function
    }

    #[must_use]
    pub const fn environment(&self) -> CssRelativeColorEnvironment {
        self.environment
    }

    #[must_use]
    pub const fn source(&self) -> &CssColor {
        &self.source
    }

    #[must_use]
    pub const fn channels(&self) -> &[CssRelativeColorExpression; 3] {
        &self.channels
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&CssRelativeColorExpression> {
        self.alpha.as_ref()
    }
}

impl CssPredefinedColor {
    pub fn try_new(
        color_space: CssPredefinedColorSpace,
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

    #[must_use]
    pub const fn color_space(&self) -> CssPredefinedColorSpace {
        self.color_space
    }

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
    pub fn parsed_alpha(&self) -> crate::CssParsedColorAlphaRef<'_> {
        crate::color_alpha::parsed_alpha(self.alpha())
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
        component: crate::CssComponentValue,
    ) -> Result<Self, CssColorScalarError> {
        let literal = CssColorPercentageLiteral::try_from_component(component)?;
        if !crate::exact_decimal::LexicalDecimal::new(literal.numeric().representation())
            .in_percentage_range()
        {
            return Err(CssColorScalarError::out_of_range(literal.origin().clone()));
        }
        Ok(Self { literal })
    }
    #[must_use]
    pub const fn literal(&self) -> &CssColorPercentageLiteral {
        &self.literal
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
pub struct CssColorInterpolation(ColorInterpolation);
#[derive(Clone, Debug, PartialEq)]
enum ColorInterpolation {
    Predefined(CssColorInterpolationMethod),
    Custom(CssColorProfileName),
}
impl CssColorInterpolation {
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
pub struct CssColorMixWeight(ColorMixWeight);
#[derive(Clone, Debug, PartialEq)]
enum ColorMixWeight {
    Literal(CssColorMixPercentage),
    Calculation(CssPercentageCalculation),
}
impl CssColorMixWeight {
    #[must_use]
    pub fn literal(value: CssColorMixPercentage) -> Self {
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
    pub fn literal_value(&self) -> Option<&CssColorMixPercentage> {
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
pub struct CssColorMixComponent {
    color: Box<CssColor>,
    weight: Option<CssColorMixWeight>,
}
impl CssColorMixComponent {
    #[must_use]
    pub fn new(color: CssColor, weight: Option<CssColorMixWeight>) -> Self {
        Self {
            color: Box::new(color),
            weight,
        }
    }
    #[must_use]
    pub const fn color(&self) -> &CssColor {
        &self.color
    }
    #[must_use]
    pub const fn weight(&self) -> Option<&CssColorMixWeight> {
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
pub struct CssColorMix {
    interpolation: Option<CssColorInterpolation>,
    components: Vec<CssColorMixComponent>,
    nesting_depth: u32,
}
impl CssColorMix {
    pub fn try_new(
        interpolation: Option<CssColorInterpolation>,
        components: Vec<CssColorMixComponent>,
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
    pub const fn interpolation(&self) -> Option<&CssColorInterpolation> {
        self.interpolation.as_ref()
    }
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
impl From<ColorGraphDepthError> for CssColorMixConstructionError {
    fn from(value: ColorGraphDepthError) -> Self {
        match value {
            ColorGraphDepthError::NestingLimit => Self::NestingLimit,
            ColorGraphDepthError::CapacityOverflow => Self::CapacityOverflow,
        }
    }
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
            Self::Color(
                space @ (CssPredefinedColorSpace::XyzD50 | CssPredefinedColorSpace::XyzD65),
            ) => (E::Xyz(space), [N; 3]),
            Self::Color(space) => (E::PredefinedRgb(space), [N; 3]),
        }
    }
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
