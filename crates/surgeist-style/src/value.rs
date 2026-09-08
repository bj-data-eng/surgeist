use std::collections::BTreeSet;

use crate::{
    AuthoredDeclarations, AuthoredTokens, CustomPropertyName, StyleAttributeName, VariableReference,
};

use super::{
    CalcLength, CalcOperator, Error, ErrorCode, Interpolation, Property, Result,
    error::{validate_finite, validate_non_negative},
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Color {
    pub const BLACK: Self = Self::raw_rgba(0.0, 0.0, 0.0, 1.0);
    pub const TRANSPARENT: Self = Self::raw_rgba(0.0, 0.0, 0.0, 0.0);

    #[must_use]
    pub(crate) const fn raw_rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }

    pub fn try_rgba(r: f32, g: f32, b: f32, a: f32) -> Result<Self> {
        let color = Self::raw_rgba(r, g, b, a);
        color.validate()?;
        Ok(color)
    }

    #[must_use]
    pub const fn r(self) -> f32 {
        self.r
    }

    #[must_use]
    pub const fn g(self) -> f32 {
        self.g
    }

    #[must_use]
    pub const fn b(self) -> f32 {
        self.b
    }

    #[must_use]
    pub const fn a(self) -> f32 {
        self.a
    }

    pub fn validate(self) -> Result<()> {
        validate_finite(self.r, "color r")?;
        validate_finite(self.g, "color g")?;
        validate_finite(self.b, "color b")?;
        validate_finite(self.a, "color a")
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
    }
}

impl From<Color> for peniko::Color {
    fn from(color: Color) -> Self {
        Self::new([color.r(), color.g(), color.b(), color.a()])
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Alpha(f32);

impl Alpha {
    pub fn new(value: f32) -> Result<Self> {
        validate_finite(value, "alpha")?;
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "alpha must be between 0 and 1",
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorComponent(Option<f32>);

impl ColorComponent {
    pub fn new(value: Option<f32>) -> Result<Self> {
        if value.is_some_and(|value| !value.is_finite()) {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "color component must be finite",
            ))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub const fn get(self) -> Option<f32> {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SystemColor {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PredefinedColorSpace {
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

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ColorInterpolationSpace {
    Predefined(PredefinedColorSpace),
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HueInterpolationMethod {
    Shorter,
    Longer,
    Increasing,
    Decreasing,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ColorInterpolationMethod {
    space: ColorInterpolationSpace,
    hue: Option<HueInterpolationMethod>,
}

impl ColorInterpolationMethod {
    #[must_use]
    pub const fn new(space: ColorInterpolationSpace, hue: Option<HueInterpolationMethod>) -> Self {
        Self { space, hue }
    }

    #[must_use]
    pub const fn space(self) -> ColorInterpolationSpace {
        self.space
    }

    #[must_use]
    pub const fn hue(self) -> Option<HueInterpolationMethod> {
        self.hue
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StyleColor {
    CurrentColor,
    Rgba(Color),
    System(SystemColor),
    Hsl {
        hue: ColorComponent,
        saturation: ColorComponent,
        lightness: ColorComponent,
        alpha: Option<Alpha>,
    },
    Hwb {
        hue: ColorComponent,
        whiteness: ColorComponent,
        blackness: ColorComponent,
        alpha: Option<Alpha>,
    },
    Lab {
        lightness: ColorComponent,
        a: ColorComponent,
        b: ColorComponent,
        alpha: Option<Alpha>,
    },
    Lch {
        lightness: ColorComponent,
        chroma: ColorComponent,
        hue: ColorComponent,
        alpha: Option<Alpha>,
    },
    Oklab {
        lightness: ColorComponent,
        a: ColorComponent,
        b: ColorComponent,
        alpha: Option<Alpha>,
    },
    Oklch {
        lightness: ColorComponent,
        chroma: ColorComponent,
        hue: ColorComponent,
        alpha: Option<Alpha>,
    },
    ColorFunction(ColorFunction),
    ColorMix(Box<ColorMix>),
    Relative(Box<RelativeColor>),
}

impl StyleColor {
    #[must_use]
    pub const fn current_color() -> Self {
        Self::CurrentColor
    }

    #[must_use]
    pub const fn rgba(color: Color) -> Self {
        Self::Rgba(color)
    }

    #[must_use]
    pub const fn system(color: SystemColor) -> Self {
        Self::System(color)
    }

    #[must_use]
    pub fn color_mix(value: ColorMix) -> Self {
        Self::ColorMix(Box::new(value))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::CurrentColor | Self::System(_) => Ok(()),
            Self::Rgba(color) => color.validate(),
            Self::ColorFunction(value) => value.validate(),
            Self::ColorMix(value) => value.validate(),
            Self::Relative(value) => value.validate(),
            Self::Hsl {
                hue,
                saturation,
                lightness,
                alpha,
            }
            | Self::Hwb {
                hue,
                whiteness: saturation,
                blackness: lightness,
                alpha,
            }
            | Self::Lab {
                lightness: hue,
                a: saturation,
                b: lightness,
                alpha,
            }
            | Self::Lch {
                lightness: hue,
                chroma: saturation,
                hue: lightness,
                alpha,
            }
            | Self::Oklab {
                lightness: hue,
                a: saturation,
                b: lightness,
                alpha,
            }
            | Self::Oklch {
                lightness: hue,
                chroma: saturation,
                hue: lightness,
                alpha,
            } => validate_color_components([*hue, *saturation, *lightness], *alpha),
        }
    }
}

impl From<Color> for StyleColor {
    fn from(value: Color) -> Self {
        Self::rgba(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorFunction {
    space: PredefinedColorSpace,
    components: [ColorComponent; 3],
    alpha: Option<Alpha>,
}

impl ColorFunction {
    #[must_use]
    pub const fn new(
        space: PredefinedColorSpace,
        components: [ColorComponent; 3],
        alpha: Option<Alpha>,
    ) -> Self {
        Self {
            space,
            components,
            alpha,
        }
    }

    #[must_use]
    pub const fn space(&self) -> PredefinedColorSpace {
        self.space
    }

    #[must_use]
    pub const fn components(&self) -> &[ColorComponent; 3] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<Alpha> {
        self.alpha
    }

    pub fn validate(&self) -> Result<()> {
        validate_color_components(self.components, self.alpha)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorMixComponent {
    color: StyleColor,
    percentage: Option<f32>,
}

impl ColorMixComponent {
    pub fn try_new(color: StyleColor, percentage: Option<f32>) -> Result<Self> {
        if percentage.is_some_and(|value| !value.is_finite() || !(0.0..=100.0).contains(&value)) {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "color mix percentage must be between 0 and 100",
            ));
        }
        color.validate()?;
        Ok(Self { color, percentage })
    }

    #[must_use]
    pub const fn color(&self) -> &StyleColor {
        &self.color
    }

    #[must_use]
    pub const fn percentage(&self) -> Option<f32> {
        self.percentage
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ColorMix {
    interpolation: ColorInterpolationMethod,
    left: ColorMixComponent,
    right: ColorMixComponent,
}

impl ColorMix {
    #[must_use]
    pub const fn new(
        interpolation: ColorInterpolationMethod,
        left: ColorMixComponent,
        right: ColorMixComponent,
    ) -> Self {
        Self {
            interpolation,
            left,
            right,
        }
    }

    #[must_use]
    pub const fn interpolation(&self) -> ColorInterpolationMethod {
        self.interpolation
    }

    #[must_use]
    pub const fn left(&self) -> &ColorMixComponent {
        &self.left
    }

    #[must_use]
    pub const fn right(&self) -> &ColorMixComponent {
        &self.right
    }

    pub fn validate(&self) -> Result<()> {
        self.left.color.validate()?;
        self.right.color.validate()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SymbolicComponentExpression {
    authored: AuthoredTokens,
    references: Vec<VariableReference>,
}

impl SymbolicComponentExpression {
    pub fn new(
        authored: AuthoredTokens,
        references: impl IntoIterator<Item = VariableReference>,
    ) -> Result<Self> {
        if authored.as_css().trim().is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "symbolic component expression cannot be empty",
            ))
        } else {
            Ok(Self {
                authored,
                references: references.into_iter().collect(),
            })
        }
    }

    #[must_use]
    pub const fn authored(&self) -> &AuthoredTokens {
        &self.authored
    }

    #[must_use]
    pub fn references(&self) -> &[VariableReference] {
        &self.references
    }

    #[must_use]
    pub fn dependencies(&self) -> Vec<CustomPropertyName> {
        let mut dependencies = BTreeSet::new();
        for reference in &self.references {
            dependencies.insert(reference.name().clone());
            if let Some(fallback) = reference.fallback() {
                dependencies.extend(fallback.expression().dependencies());
            }
        }
        dependencies.into_iter().collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color(PredefinedColorSpace),
}

impl RelativeColorFunction {
    #[must_use]
    pub const fn component_count(self) -> usize {
        3
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RelativeColor {
    function: RelativeColorFunction,
    source: StyleColor,
    components: Vec<SymbolicComponentExpression>,
    alpha: Option<SymbolicComponentExpression>,
}

impl RelativeColor {
    pub fn try_new(
        function: RelativeColorFunction,
        source: StyleColor,
        components: Vec<SymbolicComponentExpression>,
        alpha: Option<SymbolicComponentExpression>,
    ) -> Result<Self> {
        if components.len() != function.component_count() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "relative color component count does not match function",
            ));
        }
        source.validate()?;
        Ok(Self {
            function,
            source,
            components,
            alpha,
        })
    }

    #[must_use]
    pub const fn function(&self) -> RelativeColorFunction {
        self.function
    }

    #[must_use]
    pub const fn source(&self) -> &StyleColor {
        &self.source
    }

    #[must_use]
    pub fn components(&self) -> &[SymbolicComponentExpression] {
        &self.components
    }

    #[must_use]
    pub const fn alpha(&self) -> Option<&SymbolicComponentExpression> {
        self.alpha.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if self.components.len() == self.function.component_count() {
            self.source.validate()
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "relative color component count does not match function",
            ))
        }
    }
}

fn validate_color_components(components: [ColorComponent; 3], alpha: Option<Alpha>) -> Result<()> {
    for component in components {
        component.validate()?;
    }
    if let Some(alpha) = alpha {
        Alpha::new(alpha.get())?;
    }
    Ok(())
}

impl ColorComponent {
    fn validate(self) -> Result<()> {
        Self::new(self.0).map(|_| ())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CssPx(f32);

impl CssPx {
    pub fn new(value: f32) -> Result<Self> {
        validate_finite(value, "css px")?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DimensionLength(Length);

impl DimensionLength {
    pub fn px(value: CssPx) -> Result<Self> {
        validate_non_negative(value.get(), "dimension length px")?;
        Ok(Self(Length::Px(value.get())))
    }

    pub(crate) fn into_length(self) -> Length {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opacity(f32);

impl Opacity {
    pub fn new(value: f32) -> Result<Self> {
        validate_finite(value, "opacity")?;
        if (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "opacity must be between 0 and 1",
            ))
        }
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DurationSeconds(f32);

impl DurationSeconds {
    pub fn new(value: f32) -> Result<Self> {
        validate_non_negative(value, "duration seconds")?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TimeList {
    values: Vec<DurationSeconds>,
}

impl TimeList {
    pub fn try_new(values: impl IntoIterator<Item = DurationSeconds>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "time list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn seconds(&self) -> &[DurationSeconds] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "time list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_zero() -> Self {
        Self {
            values: vec![DurationSeconds::new(0.0).expect("zero seconds is valid")],
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EasingArguments {
    authored: String,
}

impl EasingArguments {
    pub fn try_new(authored: impl Into<String>) -> Result<Self> {
        let authored = authored.into();
        if authored.trim().is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "easing arguments cannot be empty",
            ));
        }
        if authored.contains('\0') {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "easing arguments cannot contain U+0000",
            ));
        }
        Ok(Self { authored })
    }

    #[must_use]
    pub fn as_css(&self) -> &str {
        &self.authored
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum EasingFunction {
    Ease,
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    StepStart,
    StepEnd,
    CubicBezier(EasingArguments),
    Steps(EasingArguments),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct EasingList {
    values: Vec<EasingFunction>,
}

impl EasingList {
    pub fn try_new(values: impl IntoIterator<Item = EasingFunction>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "easing list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[EasingFunction] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "easing list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_ease() -> Self {
        Self {
            values: vec![EasingFunction::Ease],
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TransitionPropertyName {
    value: String,
}

impl TransitionPropertyName {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        let value = validate_timing_ident(value.as_ref(), "transition property name")?;
        if value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("all") {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "transition property name cannot be `none` or `all`",
            ));
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum TransitionPropertyTarget {
    All,
    None,
    Property(Property),
    Custom(TransitionPropertyName),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TransitionPropertyList {
    values: Vec<TransitionPropertyTarget>,
}

impl TransitionPropertyList {
    pub fn try_new(values: impl IntoIterator<Item = TransitionPropertyTarget>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "transition property list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[TransitionPropertyTarget] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "transition property list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_all() -> Self {
        Self {
            values: vec![TransitionPropertyTarget::All],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionItem {
    property: Option<TransitionPropertyTarget>,
    duration: Option<DurationSeconds>,
    delay: Option<DurationSeconds>,
    timing_function: Option<EasingFunction>,
}

impl TransitionItem {
    pub fn try_new(
        property: Option<TransitionPropertyTarget>,
        duration: Option<DurationSeconds>,
        delay: Option<DurationSeconds>,
        timing_function: Option<EasingFunction>,
    ) -> Result<Self> {
        if property.is_none() && duration.is_none() && delay.is_none() && timing_function.is_none()
        {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "transition item cannot be empty",
            ));
        }
        Ok(Self {
            property,
            duration,
            delay,
            timing_function,
        })
    }

    #[must_use]
    pub const fn property(&self) -> Option<&TransitionPropertyTarget> {
        self.property.as_ref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<DurationSeconds> {
        self.duration
    }

    #[must_use]
    pub const fn delay(&self) -> Option<DurationSeconds> {
        self.delay
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&EasingFunction> {
        self.timing_function.as_ref()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionList {
    items: Vec<TransitionItem>,
}

impl TransitionList {
    pub fn try_new(items: impl IntoIterator<Item = TransitionItem>) -> Result<Self> {
        let items = items.into_iter().collect::<Vec<_>>();
        if items.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "transition list cannot be empty",
            ));
        }
        Ok(Self { items })
    }

    #[must_use]
    pub fn items(&self) -> &[TransitionItem] {
        &self.items
    }

    pub fn validate(&self) -> Result<()> {
        if self.items.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "transition list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ContentVisibility {
    #[default]
    Visible,
    Hidden,
    Auto,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ScrollbarWidth {
    #[default]
    Auto,
    Thin,
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ZIndex {
    #[default]
    Auto,
    Integer(i32),
}

impl ZIndex {
    #[must_use]
    pub const fn integer(value: i32) -> Self {
        Self::Integer(value)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Order(i32);

impl Order {
    #[must_use]
    pub const fn new(value: i32) -> Self {
        Self(value)
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FlexFactor(f32);

impl FlexFactor {
    pub fn new(value: f32) -> Result<Self> {
        validate_non_negative(value, "flex factor")?;
        Ok(Self(value))
    }

    #[must_use]
    pub const fn zero() -> Self {
        Self(0.0)
    }

    #[must_use]
    pub const fn one() -> Self {
        Self(1.0)
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }

    pub fn validate(self) -> Result<()> {
        validate_non_negative(self.0, "flex factor")
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Flex {
    None,
    Auto,
    Components {
        grow: FlexFactor,
        shrink: Option<FlexFactor>,
        basis: Option<Length>,
    },
}

impl Flex {
    #[must_use]
    pub const fn none() -> Self {
        Self::None
    }

    #[must_use]
    pub const fn auto() -> Self {
        Self::Auto
    }

    #[must_use]
    pub const fn components(
        grow: FlexFactor,
        shrink: Option<FlexFactor>,
        basis: Option<Length>,
    ) -> Self {
        Self::Components {
            grow,
            shrink,
            basis,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::None | Self::Auto => Ok(()),
            Self::Components {
                grow,
                shrink,
                basis,
            } => {
                grow.validate()?;
                if let Some(shrink) = shrink {
                    shrink.validate()?;
                }
                if let Some(basis) = basis {
                    basis.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AspectRatio(AspectRatioKind);

#[derive(Clone, Copy, Debug, PartialEq)]
enum AspectRatioKind {
    Auto,
    Ratio(f32),
}

impl AspectRatio {
    pub const AUTO: Self = Self(AspectRatioKind::Auto);

    pub fn ratio(value: f32) -> Result<Self> {
        validate_finite(value, "aspect ratio")?;
        if value <= 0.0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "aspect ratio must be positive",
            ));
        }
        Ok(Self(AspectRatioKind::Ratio(value)))
    }

    #[must_use]
    pub const fn as_ratio(self) -> Option<f32> {
        match self.0 {
            AspectRatioKind::Auto => None,
            AspectRatioKind::Ratio(value) => Some(value),
        }
    }

    pub fn validate(self) -> Result<()> {
        match self.as_ratio() {
            Some(value) => Self::ratio(value).map(|_| ()),
            None => Ok(()),
        }
    }
}

impl Default for AspectRatio {
    fn default() -> Self {
        Self::AUTO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Length {
    Normal,
    Px(f32),
    Percent(f32),
    Calc(CalcLength),
    Fill,
    Fit,
    MinContent,
    MaxContent,
    Auto,
}

impl Length {
    pub const NORMAL: Self = Self::Normal;
    pub const ZERO: Self = Self::Px(0.0);

    #[must_use]
    pub const fn px(value: f32) -> Self {
        Self::Px(value)
    }

    pub fn try_px(value: f32) -> Result<Self> {
        let length = Self::px(value);
        length.validate()?;
        Ok(length)
    }

    #[must_use]
    pub const fn percent(value: f32) -> Self {
        Self::Percent(value)
    }

    pub fn try_percent(value: f32) -> Result<Self> {
        let length = Self::percent(value);
        length.validate()?;
        Ok(length)
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Normal => Ok(()),
            Self::Px(value) => validate_finite(*value, "length px"),
            Self::Percent(value) => validate_finite(*value, "length percent"),
            Self::Calc(value) => value.validate(),
            Self::Fill | Self::Fit | Self::MinContent | Self::MaxContent | Self::Auto => Ok(()),
        }
    }
}

impl Default for Length {
    fn default() -> Self {
        Self::ZERO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Edges {
    pub top: Length,
    pub right: Length,
    pub bottom: Length,
    pub left: Length,
}

impl Edges {
    #[must_use]
    pub fn all(value: Length) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }

    #[must_use]
    pub const fn new(top: Length, right: Length, bottom: Length, left: Length) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.top.validate()?;
        self.right.validate()?;
        self.bottom.validate()?;
        self.left.validate()
    }
}

impl Default for Edges {
    fn default() -> Self {
        Self::all(Length::ZERO)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Corners {
    pub top_left: Length,
    pub top_right: Length,
    pub bottom_right: Length,
    pub bottom_left: Length,
}

impl Corners {
    #[must_use]
    pub fn all(value: Length) -> Self {
        Self {
            top_left: value.clone(),
            top_right: value.clone(),
            bottom_right: value.clone(),
            bottom_left: value,
        }
    }

    #[must_use]
    pub const fn new(
        top_left: Length,
        top_right: Length,
        bottom_right: Length,
        bottom_left: Length,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.top_left.validate()?;
        self.top_right.validate()?;
        self.bottom_right.validate()?;
        self.bottom_left.validate()
    }
}

impl Default for Corners {
    fn default() -> Self {
        Self::all(Length::ZERO)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BorderLineStyle {
    None,
    Hidden,
    Dotted,
    Dashed,
    Solid,
    Double,
    Groove,
    Ridge,
    Inset,
    Outset,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BorderSide {
    Top,
    Right,
    Bottom,
    Left,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BorderStyles {
    top: BorderLineStyle,
    right: BorderLineStyle,
    bottom: BorderLineStyle,
    left: BorderLineStyle,
}

impl BorderStyles {
    #[must_use]
    pub const fn new(
        top: BorderLineStyle,
        right: BorderLineStyle,
        bottom: BorderLineStyle,
        left: BorderLineStyle,
    ) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    #[must_use]
    pub const fn top(self) -> BorderLineStyle {
        self.top
    }

    #[must_use]
    pub const fn right(self) -> BorderLineStyle {
        self.right
    }

    #[must_use]
    pub const fn bottom(self) -> BorderLineStyle {
        self.bottom
    }

    #[must_use]
    pub const fn left(self) -> BorderLineStyle {
        self.left
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Border {
    width: Option<Length>,
    style: Option<BorderLineStyle>,
    color: Option<StyleColor>,
}

impl Border {
    pub fn try_new(
        width: Option<Length>,
        style: Option<BorderLineStyle>,
        color: Option<StyleColor>,
    ) -> Result<Self> {
        if width.is_none() && style.is_none() && color.is_none() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "border shorthand requires at least one component",
            ));
        }
        if let Some(width) = &width {
            validate_border_width_length(width)?;
        }
        if let Some(color) = &color {
            color.validate()?;
        }
        Ok(Self {
            width,
            style,
            color,
        })
    }

    #[must_use]
    pub const fn width(&self) -> Option<&Length> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<BorderLineStyle> {
        self.style
    }

    #[must_use]
    pub const fn color(&self) -> Option<&StyleColor> {
        self.color.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(width) = &self.width {
            validate_border_width_length(width)?;
        }
        if let Some(color) = &self.color {
            color.validate()?;
        }
        if self.width.is_some() || self.style.is_some() || self.color.is_some() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "border shorthand requires at least one component",
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CornerRadius {
    horizontal: Length,
    vertical: Length,
}

impl CornerRadius {
    pub fn new(horizontal: Length, vertical: Length) -> Result<Self> {
        validate_radius_length(&horizontal)?;
        validate_radius_length(&vertical)?;
        Ok(Self {
            horizontal,
            vertical,
        })
    }

    #[must_use]
    pub const fn horizontal(&self) -> &Length {
        &self.horizontal
    }

    #[must_use]
    pub const fn vertical(&self) -> &Length {
        &self.vertical
    }

    pub fn validate(&self) -> Result<()> {
        validate_radius_length(&self.horizontal)?;
        validate_radius_length(&self.vertical)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BorderRadii {
    top_left: CornerRadius,
    top_right: CornerRadius,
    bottom_right: CornerRadius,
    bottom_left: CornerRadius,
}

impl BorderRadii {
    #[must_use]
    pub fn all(radius: CornerRadius) -> Self {
        Self {
            top_left: radius.clone(),
            top_right: radius.clone(),
            bottom_right: radius.clone(),
            bottom_left: radius,
        }
    }

    #[must_use]
    pub const fn new(
        top_left: CornerRadius,
        top_right: CornerRadius,
        bottom_right: CornerRadius,
        bottom_left: CornerRadius,
    ) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }

    #[must_use]
    pub const fn top_left(&self) -> &CornerRadius {
        &self.top_left
    }

    #[must_use]
    pub const fn top_right(&self) -> &CornerRadius {
        &self.top_right
    }

    #[must_use]
    pub const fn bottom_right(&self) -> &CornerRadius {
        &self.bottom_right
    }

    #[must_use]
    pub const fn bottom_left(&self) -> &CornerRadius {
        &self.bottom_left
    }

    pub fn validate(&self) -> Result<()> {
        self.top_left.validate()?;
        self.top_right.validate()?;
        self.bottom_right.validate()?;
        self.bottom_left.validate()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OutlineStyle {
    Auto,
    Border(BorderLineStyle),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OutlineWidth {
    Thin,
    Medium,
    Thick,
    Length(OutlineWidthLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct OutlineWidthLength(Length);

impl OutlineWidthLength {
    pub fn new(length: Length) -> Result<Self> {
        validate_border_width_length(&length)?;
        Ok(Self(length))
    }

    #[must_use]
    pub const fn length(&self) -> &Length {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Outline {
    width: Option<OutlineWidth>,
    style: Option<OutlineStyle>,
    color: Option<StyleColor>,
}

impl Outline {
    pub fn try_new(
        width: Option<OutlineWidth>,
        style: Option<OutlineStyle>,
        color: Option<StyleColor>,
    ) -> Result<Self> {
        if width.is_none() && style.is_none() && color.is_none() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "outline shorthand requires at least one component",
            ));
        }
        if let Some(color) = &color {
            color.validate()?;
        }
        Ok(Self {
            width,
            style,
            color,
        })
    }

    #[must_use]
    pub const fn width(&self) -> Option<&OutlineWidth> {
        self.width.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<OutlineStyle> {
        self.style
    }

    #[must_use]
    pub const fn color(&self) -> Option<&StyleColor> {
        self.color.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(width) = &self.width {
            match width {
                OutlineWidth::Thin | OutlineWidth::Medium | OutlineWidth::Thick => {}
                OutlineWidth::Length(length) => {
                    validate_border_width_length(length.length())?;
                }
            }
        }
        if let Some(color) = &self.color {
            color.validate()?;
        }
        if self.width.is_some() || self.style.is_some() || self.color.is_some() {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "outline shorthand requires at least one component",
            ))
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
struct StyleStringList(Vec<String>);

impl StyleStringList {
    #[must_use]
    const fn empty() -> Self {
        Self(Vec::new())
    }

    fn new(
        values: impl IntoIterator<Item = impl Into<String>>,
        item_field: &'static str,
    ) -> Result<Self> {
        let list = Self(values.into_iter().map(Into::into).collect());
        list.validate(item_field)?;
        Ok(list)
    }

    fn validate(&self, item_field: &str) -> Result<()> {
        self.0
            .iter()
            .try_for_each(|value| validate_style_string(value, item_field))
    }

    #[must_use]
    fn as_slice(&self) -> &[String] {
        &self.0
    }

    #[must_use]
    fn into_vec(self) -> Vec<String> {
        self.0
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct FontFamilyList(StyleStringList);

impl FontFamilyList {
    #[must_use]
    pub const fn empty() -> Self {
        Self(StyleStringList::empty())
    }

    pub fn new(values: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        Ok(Self(StyleStringList::new(values, "font family")?))
    }

    pub fn validate(&self) -> Result<()> {
        self.0.validate("font family")
    }

    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        self.0.as_slice()
    }

    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.0.as_slice().iter().map(String::as_str)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<String> {
        self.0.into_vec()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct FontWeightNumber(i32);

impl FontWeightNumber {
    pub fn new(value: i32) -> Result<Self> {
        if (1..=1000).contains(&value) {
            Ok(Self(value))
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "font weight number must be between 1 and 1000",
            ))
        }
    }

    #[must_use]
    pub const fn normal() -> Self {
        Self(400)
    }

    #[must_use]
    pub const fn bold() -> Self {
        Self(700)
    }

    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}

impl Default for FontWeightNumber {
    fn default() -> Self {
        Self::normal()
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
    Bolder,
    Lighter,
    Number(FontWeightNumber),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontStretch {
    #[default]
    Normal,
    UltraCondensed,
    ExtraCondensed,
    Condensed,
    SemiCondensed,
    SemiExpanded,
    Expanded,
    ExtraExpanded,
    UltraExpanded,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FontVariant {
    #[default]
    Normal,
    SmallCaps,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FontFeatureTag(String);

impl FontFeatureTag {
    pub fn new(tag: impl Into<String>) -> Result<Self> {
        let tag = tag.into();
        if tag.chars().count() == 4 {
            Ok(Self(tag))
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "font feature tag must contain exactly four characters",
            ))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum FontFeatureValue {
    On,
    Off,
    Integer(i32),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FontFeature {
    tag: FontFeatureTag,
    value: Option<FontFeatureValue>,
}

impl FontFeature {
    #[must_use]
    pub const fn new(tag: FontFeatureTag, value: Option<FontFeatureValue>) -> Self {
        Self { tag, value }
    }

    #[must_use]
    pub const fn tag(&self) -> &FontFeatureTag {
        &self.tag
    }

    #[must_use]
    pub const fn value(&self) -> Option<FontFeatureValue> {
        self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FontFeatureSettings {
    kind: FontFeatureSettingsKind,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum FontFeatureSettingsKind {
    Normal,
    Features(Vec<FontFeature>),
}

impl Default for FontFeatureSettings {
    fn default() -> Self {
        Self::NORMAL
    }
}

impl FontFeatureSettings {
    pub const NORMAL: Self = Self {
        kind: FontFeatureSettingsKind::Normal,
    };

    pub fn features(features: impl IntoIterator<Item = FontFeature>) -> Result<Self> {
        let features = features.into_iter().collect::<Vec<_>>();
        if features.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "font feature settings must contain at least one feature",
            ))
        } else {
            Ok(Self {
                kind: FontFeatureSettingsKind::Features(features),
            })
        }
    }

    #[must_use]
    pub fn as_slice(&self) -> &[FontFeature] {
        match &self.kind {
            FontFeatureSettingsKind::Normal => &[],
            FontFeatureSettingsKind::Features(features) => features.as_slice(),
        }
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Font {
    style: Option<TextSlant>,
    variant: Option<FontVariant>,
    weight: Option<FontWeight>,
    stretch: Option<FontStretch>,
    size: Length,
    line_height: Option<Length>,
    family: FontFamilyList,
}

impl Font {
    pub fn try_new(
        style: Option<TextSlant>,
        variant: Option<FontVariant>,
        weight: Option<FontWeight>,
        stretch: Option<FontStretch>,
        size: Length,
        line_height: Option<Length>,
        family: FontFamilyList,
    ) -> Result<Self> {
        validate_font_size_length(&size)?;
        if let Some(line_height) = &line_height {
            validate_line_height_length(line_height)?;
        }
        family.validate()?;
        if family.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "font shorthand requires at least one font family",
            ));
        }
        let font = Self {
            style,
            variant,
            weight,
            stretch,
            size,
            line_height,
            family,
        };
        font.validate()?;
        Ok(font)
    }

    #[must_use]
    pub const fn style(&self) -> Option<TextSlant> {
        self.style
    }

    #[must_use]
    pub const fn variant(&self) -> Option<FontVariant> {
        self.variant
    }

    #[must_use]
    pub const fn weight(&self) -> Option<FontWeight> {
        self.weight
    }

    #[must_use]
    pub const fn stretch(&self) -> Option<FontStretch> {
        self.stretch
    }

    #[must_use]
    pub const fn size(&self) -> &Length {
        &self.size
    }

    #[must_use]
    pub const fn line_height(&self) -> Option<&Length> {
        self.line_height.as_ref()
    }

    #[must_use]
    pub const fn family(&self) -> &FontFamilyList {
        &self.family
    }

    pub fn validate(&self) -> Result<()> {
        validate_font_size_length(&self.size)?;
        if let Some(style) = self.style {
            validate_slant(style)?;
        }
        if let Some(line_height) = &self.line_height {
            validate_line_height_length(line_height)?;
        }
        self.family.validate()
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct KeyframesIdent {
    value: String,
}

impl KeyframesIdent {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        let value = validate_timing_ident(value.as_ref(), "keyframes name")?;
        if value.eq_ignore_ascii_case("none") {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "keyframes name cannot be `none`",
            ));
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KeyframesString {
    value: String,
}

impl KeyframesString {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "keyframes string name cannot be empty",
            ));
        }
        if value.contains('\0') {
            return Err(Error::new(
                ErrorCode::InvalidString,
                "keyframes string name cannot contain U+0000",
            ));
        }
        Ok(Self { value })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum KeyframesName {
    Ident(KeyframesIdent),
    String(KeyframesString),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct KeyframeOffset {
    percent: f32,
}

impl KeyframeOffset {
    pub fn try_new(percent: f32) -> Result<Self> {
        validate_finite(percent, "keyframe offset")?;
        if !(0.0..=100.0).contains(&percent) {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "keyframe offset must be between 0 and 100 percent",
            ));
        }
        Ok(Self { percent })
    }

    #[must_use]
    pub fn from() -> Self {
        Self::try_new(0.0).expect("from keyframe offset is valid")
    }

    #[must_use]
    pub fn to() -> Self {
        Self::try_new(100.0).expect("to keyframe offset is valid")
    }

    #[must_use]
    pub const fn percent(self) -> f32 {
        self.percent
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeSelectorList {
    offsets: Vec<KeyframeOffset>,
}

impl KeyframeSelectorList {
    pub fn try_new(offsets: impl IntoIterator<Item = KeyframeOffset>) -> Result<Self> {
        let offsets = offsets.into_iter().collect::<Vec<_>>();
        if offsets.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "keyframe selector list cannot be empty",
            ));
        }

        let mut seen = Vec::new();
        for offset in &offsets {
            let percent = offset.percent();
            if seen.contains(&percent) {
                return Err(Error::new(
                    ErrorCode::InvalidValue,
                    "keyframe selector list cannot contain duplicate offsets",
                ));
            }
            seen.push(percent);
        }

        Ok(Self { offsets })
    }

    #[must_use]
    pub fn offsets(&self) -> &[KeyframeOffset] {
        &self.offsets
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframeBlock {
    selectors: KeyframeSelectorList,
    declarations: AuthoredDeclarations,
}

impl KeyframeBlock {
    pub fn try_new(
        selectors: KeyframeSelectorList,
        declarations: AuthoredDeclarations,
    ) -> Result<Self> {
        if declarations.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "keyframe block declarations cannot be empty",
            ));
        }
        Ok(Self {
            selectors,
            declarations,
        })
    }

    #[must_use]
    pub const fn selectors(&self) -> &KeyframeSelectorList {
        &self.selectors
    }

    #[must_use]
    pub const fn declarations(&self) -> &AuthoredDeclarations {
        &self.declarations
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KeyframesRule {
    name: KeyframesName,
    blocks: Vec<KeyframeBlock>,
}

impl KeyframesRule {
    pub fn try_new(
        name: KeyframesName,
        blocks: impl IntoIterator<Item = KeyframeBlock>,
    ) -> Result<Self> {
        let blocks = blocks.into_iter().collect::<Vec<_>>();
        if blocks.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "keyframes rule must contain at least one block",
            ));
        }

        let mut seen = Vec::new();
        for block in &blocks {
            for offset in block.selectors().offsets() {
                let percent = offset.percent();
                if seen.contains(&percent) {
                    return Err(Error::new(
                        ErrorCode::InvalidValue,
                        "keyframes rule cannot contain duplicate offsets",
                    ));
                }
                seen.push(percent);
            }
        }

        Ok(Self { name, blocks })
    }

    #[must_use]
    pub const fn name(&self) -> &KeyframesName {
        &self.name
    }

    #[must_use]
    pub fn blocks(&self) -> &[KeyframeBlock] {
        &self.blocks
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum AnimationName {
    None,
    Keyframes(KeyframesName),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AnimationNameList {
    names: Vec<AnimationName>,
}

impl AnimationNameList {
    pub fn try_new(names: impl IntoIterator<Item = AnimationName>) -> Result<Self> {
        let names = names.into_iter().collect::<Vec<_>>();
        if names.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation name list cannot be empty",
            ));
        }
        Ok(Self { names })
    }

    pub fn validate(&self) -> Result<()> {
        if self.names.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation name list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn names(&self) -> &[AnimationName] {
        &self.names
    }

    #[must_use]
    pub fn single_none() -> Self {
        Self {
            names: vec![AnimationName::None],
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AnimationIterationCount {
    Infinite,
    Number(AnimationIterationNumber),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnimationIterationNumber {
    value: f32,
}

impl AnimationIterationNumber {
    pub fn try_new(value: f32) -> Result<Self> {
        validate_non_negative(value, "animation iteration count")?;
        Ok(Self { value })
    }

    #[must_use]
    pub const fn get(self) -> f32 {
        self.value
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationIterationCountList {
    values: Vec<AnimationIterationCount>,
}

impl AnimationIterationCountList {
    pub fn try_new(values: impl IntoIterator<Item = AnimationIterationCount>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation iteration count list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[AnimationIterationCount] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation iteration count list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_one() -> Self {
        Self {
            values: vec![AnimationIterationCount::Number(
                AnimationIterationNumber::try_new(1.0).unwrap(),
            )],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AnimationDirection {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AnimationFillMode {
    #[default]
    None,
    Forwards,
    Backwards,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AnimationPlayState {
    #[default]
    Running,
    Paused,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AnimationDirectionList {
    values: Vec<AnimationDirection>,
}

impl AnimationDirectionList {
    pub fn try_new(values: impl IntoIterator<Item = AnimationDirection>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation direction list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[AnimationDirection] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation direction list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_normal() -> Self {
        Self {
            values: vec![AnimationDirection::Normal],
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AnimationFillModeList {
    values: Vec<AnimationFillMode>,
}

impl AnimationFillModeList {
    pub fn try_new(values: impl IntoIterator<Item = AnimationFillMode>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation fill mode list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[AnimationFillMode] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation fill mode list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_none() -> Self {
        Self {
            values: vec![AnimationFillMode::None],
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct AnimationPlayStateList {
    values: Vec<AnimationPlayState>,
}

impl AnimationPlayStateList {
    pub fn try_new(values: impl IntoIterator<Item = AnimationPlayState>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation play state list cannot be empty",
            ));
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[AnimationPlayState] {
        &self.values
    }

    pub fn validate(&self) -> Result<()> {
        if self.values.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation play state list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_running() -> Self {
        Self {
            values: vec![AnimationPlayState::Running],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationItem {
    name: Option<AnimationName>,
    duration: Option<DurationSeconds>,
    delay: Option<DurationSeconds>,
    timing_function: Option<EasingFunction>,
    iteration_count: Option<AnimationIterationCount>,
    direction: Option<AnimationDirection>,
    fill_mode: Option<AnimationFillMode>,
    play_state: Option<AnimationPlayState>,
}

impl AnimationItem {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        name: Option<AnimationName>,
        duration: Option<DurationSeconds>,
        delay: Option<DurationSeconds>,
        timing_function: Option<EasingFunction>,
        iteration_count: Option<AnimationIterationCount>,
        direction: Option<AnimationDirection>,
        fill_mode: Option<AnimationFillMode>,
        play_state: Option<AnimationPlayState>,
    ) -> Result<Self> {
        if name.is_none()
            && duration.is_none()
            && delay.is_none()
            && timing_function.is_none()
            && iteration_count.is_none()
            && direction.is_none()
            && fill_mode.is_none()
            && play_state.is_none()
        {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation item cannot be empty",
            ));
        }
        Ok(Self {
            name,
            duration,
            delay,
            timing_function,
            iteration_count,
            direction,
            fill_mode,
            play_state,
        })
    }

    #[must_use]
    pub const fn name(&self) -> Option<&AnimationName> {
        self.name.as_ref()
    }

    #[must_use]
    pub const fn duration(&self) -> Option<DurationSeconds> {
        self.duration
    }

    #[must_use]
    pub const fn delay(&self) -> Option<DurationSeconds> {
        self.delay
    }

    #[must_use]
    pub const fn timing_function(&self) -> Option<&EasingFunction> {
        self.timing_function.as_ref()
    }

    #[must_use]
    pub const fn iteration_count(&self) -> Option<AnimationIterationCount> {
        self.iteration_count
    }

    #[must_use]
    pub const fn direction(&self) -> Option<AnimationDirection> {
        self.direction
    }

    #[must_use]
    pub const fn fill_mode(&self) -> Option<AnimationFillMode> {
        self.fill_mode
    }

    #[must_use]
    pub const fn play_state(&self) -> Option<AnimationPlayState> {
        self.play_state
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnimationList {
    items: Vec<AnimationItem>,
}

impl AnimationList {
    pub fn try_new(items: impl IntoIterator<Item = AnimationItem>) -> Result<Self> {
        let items = items.into_iter().collect::<Vec<_>>();
        if items.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "animation list cannot be empty",
            ));
        }
        Ok(Self { items })
    }

    #[must_use]
    pub fn items(&self) -> &[AnimationItem] {
        &self.items
    }

    pub fn validate(&self) -> Result<()> {
        if self.items.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "animation list cannot be empty",
            ))
        } else {
            Ok(())
        }
    }

    #[must_use]
    pub fn single_initial() -> Self {
        Self {
            items: vec![
                AnimationItem::try_new(
                    Some(AnimationName::None),
                    Some(DurationSeconds::new(0.0).unwrap()),
                    Some(DurationSeconds::new(0.0).unwrap()),
                    Some(EasingFunction::Ease),
                    Some(AnimationIterationCount::Number(
                        AnimationIterationNumber::try_new(1.0).unwrap(),
                    )),
                    Some(AnimationDirection::Normal),
                    Some(AnimationFillMode::None),
                    Some(AnimationPlayState::Running),
                )
                .unwrap(),
            ],
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StyleUrl(String);

impl StyleUrl {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "style URL cannot be empty",
            ))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ContentString {
    value: String,
}

impl ContentString {
    pub fn try_new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.contains('\0') {
            Err(Error::new(
                ErrorCode::InvalidString,
                "content string cannot contain U+0000",
            ))
        } else {
            Ok(Self { value })
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CounterName {
    value: String,
}

impl CounterName {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            value: validate_counter_identifier(value.as_ref(), "counter name")?,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CounterStyleName {
    value: String,
}

impl CounterStyleName {
    pub fn try_new(value: impl AsRef<str>) -> Result<Self> {
        Ok(Self {
            value: validate_counter_identifier(value.as_ref(), "counter style name")?,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BuiltInCounterStyle {
    Disc,
    Circle,
    Square,
    Decimal,
    DecimalLeadingZero,
    LowerAlpha,
    UpperAlpha,
    LowerLatin,
    UpperLatin,
    LowerRoman,
    UpperRoman,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum CounterStyle {
    BuiltIn(BuiltInCounterStyle),
    Named(CounterStyleName),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CounterFunction {
    name: CounterName,
    style: Option<CounterStyle>,
}

impl CounterFunction {
    #[must_use]
    pub const fn new(name: CounterName, style: Option<CounterStyle>) -> Self {
        Self { name, style }
    }

    #[must_use]
    pub const fn name(&self) -> &CounterName {
        &self.name
    }

    #[must_use]
    pub const fn style(&self) -> Option<&CounterStyle> {
        self.style.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CountersFunction {
    name: CounterName,
    separator: ContentString,
    style: Option<CounterStyle>,
}

impl CountersFunction {
    #[must_use]
    pub const fn new(
        name: CounterName,
        separator: ContentString,
        style: Option<CounterStyle>,
    ) -> Self {
        Self {
            name,
            separator,
            style,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &CounterName {
        &self.name
    }

    #[must_use]
    pub const fn separator(&self) -> &ContentString {
        &self.separator
    }

    #[must_use]
    pub const fn style(&self) -> Option<&CounterStyle> {
        self.style.as_ref()
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ContentItem {
    String(ContentString),
    Url(StyleUrl),
    Counter(CounterFunction),
    Counters(CountersFunction),
    Attr(StyleAttributeName),
    OpenQuote,
    CloseQuote,
    NoOpenQuote,
    NoCloseQuote,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ContentItemList {
    items: Vec<ContentItem>,
}

impl ContentItemList {
    pub fn try_new(items: impl IntoIterator<Item = ContentItem>) -> Result<Self> {
        let items = items.into_iter().collect::<Vec<_>>();
        if items.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "content item list cannot be empty",
            ))
        } else {
            Ok(Self { items })
        }
    }

    #[must_use]
    pub fn items(&self) -> &[ContentItem] {
        &self.items
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum Content {
    #[default]
    Normal,
    None,
    Items(ContentItemList),
}

impl Content {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Normal | Self::None => Ok(()),
            Self::Items(items) => {
                if items.items().is_empty() {
                    Err(Error::new(
                        ErrorCode::InvalidValue,
                        "content item list cannot be empty",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ListStyleType {
    None,
    CounterStyle(CounterStyle),
    String(ContentString),
}

impl Default for ListStyleType {
    fn default() -> Self {
        Self::CounterStyle(CounterStyle::BuiltIn(BuiltInCounterStyle::Disc))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum ListStylePosition {
    Inside,
    #[default]
    Outside,
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum ListStyleImage {
    #[default]
    None,
    Url(StyleUrl),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ListStyle {
    style_type: Option<ListStyleType>,
    position: Option<ListStylePosition>,
    image: Option<ListStyleImage>,
}

impl ListStyle {
    pub fn try_new(
        style_type: Option<ListStyleType>,
        position: Option<ListStylePosition>,
        image: Option<ListStyleImage>,
    ) -> Result<Self> {
        if style_type.is_none() && position.is_none() && image.is_none() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "list style shorthand requires at least one component",
            ))
        } else {
            Ok(Self {
                style_type,
                position,
                image,
            })
        }
    }

    #[must_use]
    pub const fn style_type(&self) -> Option<&ListStyleType> {
        self.style_type.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<ListStylePosition> {
        self.position
    }

    #[must_use]
    pub const fn image(&self) -> Option<&ListStyleImage> {
        self.image.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if self.style_type.is_none() && self.position.is_none() && self.image.is_none() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "list style shorthand requires at least one component",
            ))
        } else {
            Ok(())
        }
    }
}

/// A normalized counter mutation with an explicit integer value.
///
/// CSS omitted counter integers must be lowered before values reach this style
/// model, so `value` is always the concrete mutation amount to apply.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CounterChange {
    name: CounterName,
    value: i32,
}

impl CounterChange {
    #[must_use]
    pub const fn new(name: CounterName, value: i32) -> Self {
        Self { name, value }
    }

    #[must_use]
    pub const fn name(&self) -> &CounterName {
        &self.name
    }

    #[must_use]
    pub const fn value(&self) -> i32 {
        self.value
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CounterChangeList {
    changes: Vec<CounterChange>,
}

impl CounterChangeList {
    pub fn try_new(changes: impl IntoIterator<Item = CounterChange>) -> Result<Self> {
        let changes = changes.into_iter().collect::<Vec<_>>();
        if changes.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "counter change list cannot be empty",
            ))
        } else {
            Ok(Self { changes })
        }
    }

    #[must_use]
    pub fn changes(&self) -> &[CounterChange] {
        &self.changes
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub enum CounterChanges {
    #[default]
    None,
    Changes(CounterChangeList),
}

impl CounterChanges {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Changes(changes) => {
                if changes.changes().is_empty() {
                    Err(Error::new(
                        ErrorCode::InvalidValue,
                        "counter change list cannot be empty",
                    ))
                } else {
                    Ok(())
                }
            }
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SymbolicFunctionValue(String);

impl SymbolicFunctionValue {
    pub fn new(value: impl Into<String>) -> Result<Self> {
        let value = value.into();
        if value.trim().is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "symbolic function value cannot be empty",
            ))
        } else {
            Ok(Self(value))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BoxDecorationBreak {
    Slice,
    Clone,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum FilterFunction {
    Blur(SymbolicFunctionValue),
    Brightness(SymbolicFunctionValue),
    Contrast(SymbolicFunctionValue),
    DropShadow(SymbolicFunctionValue),
    Grayscale(SymbolicFunctionValue),
    HueRotate(SymbolicFunctionValue),
    Invert(SymbolicFunctionValue),
    Opacity(SymbolicFunctionValue),
    Saturate(SymbolicFunctionValue),
    Sepia(SymbolicFunctionValue),
    Url(StyleUrl),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct FilterFunctionList {
    functions: Vec<FilterFunction>,
}

impl FilterFunctionList {
    pub fn try_new(functions: impl IntoIterator<Item = FilterFunction>) -> Result<Self> {
        let functions = functions.into_iter().collect::<Vec<_>>();
        if functions.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "filter function list cannot be empty",
            ))
        } else {
            Ok(Self { functions })
        }
    }

    #[must_use]
    pub fn functions(&self) -> &[FilterFunction] {
        &self.functions
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Filter {
    None,
    Functions(FilterFunctionList),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum BasicShape {
    Inset(SymbolicFunctionValue),
    Circle(SymbolicFunctionValue),
    Ellipse(SymbolicFunctionValue),
    Polygon(SymbolicFunctionValue),
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ClipPath {
    None,
    Url(StyleUrl),
    BasicShape(BasicShape),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Translate {
    None,
    Values(TranslateValues),
}

impl Translate {
    pub fn try_values(values: impl IntoIterator<Item = Length>) -> Result<Self> {
        Ok(Self::Values(TranslateValues::try_new(values)?))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranslateValues {
    values: Vec<Length>,
}

impl TranslateValues {
    pub fn try_new(values: impl IntoIterator<Item = Length>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() || values.len() > 3 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "translate requires one to three values",
            ));
        }
        for value in &values {
            value.validate()?;
        }
        Ok(Self { values })
    }

    #[must_use]
    pub fn values(&self) -> &[Length] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Rotate {
    None,
    Value(SymbolicFunctionValue),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Scale {
    None,
    Values(ScaleValues),
}

impl Scale {
    pub fn try_values(values: impl IntoIterator<Item = f32>) -> Result<Self> {
        Ok(Self::Values(ScaleValues::try_new(values)?))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScaleValues {
    values: Vec<f32>,
}

impl ScaleValues {
    pub fn try_new(values: impl IntoIterator<Item = f32>) -> Result<Self> {
        let values = values.into_iter().collect::<Vec<_>>();
        if values.is_empty() || values.len() > 3 || values.iter().any(|value| !value.is_finite()) {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "scale requires one to three finite values",
            ))
        } else {
            Ok(Self { values })
        }
    }

    #[must_use]
    pub fn values(&self) -> &[f32] {
        &self.values
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ImageLayer {
    None,
    Url(StyleUrl),
}

impl ImageLayer {
    #[must_use]
    pub const fn url(url: StyleUrl) -> Self {
        Self::Url(url)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ImageLayerList {
    layers: Vec<ImageLayer>,
}

impl ImageLayerList {
    pub fn try_new(layers: impl IntoIterator<Item = ImageLayer>) -> Result<Self> {
        let layers = layers.into_iter().collect::<Vec<_>>();
        if layers.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "image layer list cannot be empty",
            ))
        } else {
            Ok(Self { layers })
        }
    }

    #[must_use]
    pub fn layers(&self) -> &[ImageLayer] {
        &self.layers
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum HorizontalPositionKeyword {
    Left,
    Center,
    Right,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VerticalPositionKeyword {
    Top,
    Center,
    Bottom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum PositionComponent {
    Horizontal(HorizontalPositionKeyword),
    Vertical(VerticalPositionKeyword),
    Length(Length),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Position {
    components: Vec<PositionComponent>,
}

impl Position {
    pub fn try_new(components: impl IntoIterator<Item = PositionComponent>) -> Result<Self> {
        let components = components.into_iter().collect::<Vec<_>>();
        validate_position_components(&components)?;
        Ok(Self { components })
    }

    #[must_use]
    pub fn origin() -> Self {
        Self {
            components: vec![
                PositionComponent::Length(Length::Percent(0.0)),
                PositionComponent::Length(Length::Percent(0.0)),
            ],
        }
    }

    #[must_use]
    pub fn components(&self) -> &[PositionComponent] {
        &self.components
    }

    pub fn validate(&self) -> Result<()> {
        validate_position_components(&self.components)
    }
}

fn validate_position_components(components: &[PositionComponent]) -> Result<()> {
    if components.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "position cannot be empty",
        ));
    }
    if components.len() > 4 {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "position cannot contain more than four components",
        ));
    }

    let mut horizontal_sides = 0;
    let mut vertical_sides = 0;
    for component in components {
        match component {
            PositionComponent::Horizontal(
                HorizontalPositionKeyword::Left | HorizontalPositionKeyword::Right,
            ) => horizontal_sides += 1,
            PositionComponent::Vertical(
                VerticalPositionKeyword::Top | VerticalPositionKeyword::Bottom,
            ) => vertical_sides += 1,
            PositionComponent::Length(length) => length.validate()?,
            PositionComponent::Horizontal(HorizontalPositionKeyword::Center)
            | PositionComponent::Vertical(VerticalPositionKeyword::Center) => {}
        }
    }
    if horizontal_sides > 1 {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "position cannot contain duplicate horizontal side keywords",
        ));
    }
    if vertical_sides > 1 {
        return Err(Error::new(
            ErrorCode::InvalidValue,
            "position cannot contain duplicate vertical side keywords",
        ));
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq)]
pub struct PositionList {
    positions: Vec<Position>,
}

impl PositionList {
    pub fn try_new(positions: impl IntoIterator<Item = Position>) -> Result<Self> {
        let positions = positions.into_iter().collect::<Vec<_>>();
        if positions.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "position list cannot be empty",
            ));
        }
        for position in &positions {
            position.validate()?;
        }
        Ok(Self { positions })
    }

    #[must_use]
    pub fn positions(&self) -> &[Position] {
        &self.positions
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundSizeComponent {
    Auto,
    Length(Length),
}

impl BackgroundSizeComponent {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Auto => Ok(()),
            Self::Length(length) => length.validate(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum BackgroundSize {
    Cover,
    Contain,
    Explicit {
        width: BackgroundSizeComponent,
        height: Option<BackgroundSizeComponent>,
    },
}

impl BackgroundSize {
    #[must_use]
    pub const fn auto() -> Self {
        Self::Explicit {
            width: BackgroundSizeComponent::Auto,
            height: None,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Cover | Self::Contain => Ok(()),
            Self::Explicit { width, height } => {
                width.validate()?;
                if let Some(height) = height {
                    height.validate()?;
                }
                Ok(())
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BackgroundSizeList {
    sizes: Vec<BackgroundSize>,
}

impl BackgroundSizeList {
    pub fn try_new(sizes: impl IntoIterator<Item = BackgroundSize>) -> Result<Self> {
        let sizes = sizes.into_iter().collect::<Vec<_>>();
        if sizes.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "background size list cannot be empty",
            ));
        }
        for size in &sizes {
            size.validate()?;
        }
        Ok(Self { sizes })
    }

    #[must_use]
    pub fn sizes(&self) -> &[BackgroundSize] {
        &self.sizes
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BackgroundRepeatStyle {
    Repeat,
    Space,
    Round,
    NoRepeat,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BackgroundRepeat {
    Axes {
        x: BackgroundRepeatStyle,
        y: BackgroundRepeatStyle,
    },
    RepeatX,
    RepeatY,
}

impl BackgroundRepeat {
    #[must_use]
    pub const fn repeat() -> Self {
        Self::Axes {
            x: BackgroundRepeatStyle::Repeat,
            y: BackgroundRepeatStyle::Repeat,
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BackgroundRepeatList {
    repeats: Vec<BackgroundRepeat>,
}

impl BackgroundRepeatList {
    pub fn try_new(repeats: impl IntoIterator<Item = BackgroundRepeat>) -> Result<Self> {
        let repeats = repeats.into_iter().collect::<Vec<_>>();
        if repeats.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "background repeat list cannot be empty",
            ))
        } else {
            Ok(Self { repeats })
        }
    }

    #[must_use]
    pub fn repeats(&self) -> &[BackgroundRepeat] {
        &self.repeats
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BackgroundBox {
    BorderBox,
    PaddingBox,
    ContentBox,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BackgroundAttachment {
    Scroll,
    Fixed,
    Local,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct BackgroundAttachmentList {
    attachments: Vec<BackgroundAttachment>,
}

impl BackgroundAttachmentList {
    pub fn try_new(attachments: impl IntoIterator<Item = BackgroundAttachment>) -> Result<Self> {
        let attachments = attachments.into_iter().collect::<Vec<_>>();
        if attachments.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "background attachment list cannot be empty",
            ))
        } else {
            Ok(Self { attachments })
        }
    }

    #[must_use]
    pub fn attachments(&self) -> &[BackgroundAttachment] {
        &self.attachments
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayer {
    image: Option<ImageLayer>,
    position: Option<Position>,
    size: Option<BackgroundSize>,
    repeat: Option<BackgroundRepeat>,
}

impl MaskLayer {
    pub fn try_new(
        image: Option<ImageLayer>,
        position: Option<Position>,
        size: Option<BackgroundSize>,
        repeat: Option<BackgroundRepeat>,
    ) -> Result<Self> {
        if image.is_none() && position.is_none() && size.is_none() && repeat.is_none() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "mask layer requires at least one component",
            ));
        }
        if let Some(position) = &position {
            position.validate()?;
        }
        if let Some(size) = &size {
            size.validate()?;
        }
        Ok(Self {
            image,
            position,
            size,
            repeat,
        })
    }

    #[must_use]
    pub const fn image(&self) -> Option<&ImageLayer> {
        self.image.as_ref()
    }

    #[must_use]
    pub const fn position(&self) -> Option<&Position> {
        self.position.as_ref()
    }

    #[must_use]
    pub const fn size(&self) -> Option<&BackgroundSize> {
        self.size.as_ref()
    }

    #[must_use]
    pub const fn repeat(&self) -> Option<BackgroundRepeat> {
        self.repeat
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(position) = &self.position {
            position.validate()?;
        }
        if let Some(size) = &self.size {
            size.validate()?;
        }
        if self.image.is_some()
            || self.position.is_some()
            || self.size.is_some()
            || self.repeat.is_some()
        {
            Ok(())
        } else {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "mask layer requires at least one component",
            ))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MaskLayerList {
    layers: Vec<MaskLayer>,
}

impl MaskLayerList {
    pub fn try_new(layers: impl IntoIterator<Item = MaskLayer>) -> Result<Self> {
        let layers = layers.into_iter().collect::<Vec<_>>();
        if layers.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "mask layer list cannot be empty",
            ));
        }
        for layer in &layers {
            layer.validate()?;
        }
        Ok(Self { layers })
    }

    #[must_use]
    pub fn layers(&self) -> &[MaskLayer] {
        &self.layers
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Keyword(Keyword),
    Display(Display),
    BoxSizing(BoxSizing),
    Position(LayoutPosition),
    ZIndex(ZIndex),
    ScrollbarWidth(ScrollbarWidth),
    ContentVisibility(ContentVisibility),
    Content(Content),
    ListStyleType(ListStyleType),
    ListStylePosition(ListStylePosition),
    ListStyleImage(ListStyleImage),
    ListStyle(ListStyle),
    CounterChanges(CounterChanges),
    Order(Order),
    FlexFactor(FlexFactor),
    Flex(Flex),
    AspectRatio(AspectRatio),
    Direction(Direction),
    Overflow(Overflow),
    OverflowAxes(OverflowAxes),
    Float(Float),
    Clear(Clear),
    TextAlign(StyleTextAlign),
    TextAlignLast(TextAlignLast),
    TextIndent(TextIndent),
    VerticalAlign(VerticalAlign),
    LetterSpacing(LetterSpacing),
    TextTransform(TextTransform),
    TextDecoration(TextDecoration),
    TextDecorationLine(TextDecorationLine),
    TextDecorationStyle(TextDecorationStyle),
    TextDecorationThickness(TextDecorationThickness),
    TextWrap(TextWrap),
    WhiteSpace(WhiteSpace),
    WordBreak(WordBreak),
    OverflowWrap(OverflowWrap),
    TextOverflow(TextOverflow),
    WritingMode(WritingMode),
    FlexDirection(FlexDirection),
    FlexWrap(FlexWrap),
    AlignItems(AlignItems),
    AlignContent(AlignContent),
    PlaceContentAlignment(PlaceContentAlignment),
    PlaceItemsAlignment(PlaceItemsAlignment),
    Number(f32),
    Length(Length),
    Size(Size),
    Edges(Edges),
    GridTrackList(GridTrackList),
    GridTemplateAreas(GridTemplateAreas),
    GridTemplate(GridTemplate),
    GridDefinition(GridDefinition),
    GridLine(GridLine),
    GridPlacement(GridPlacement),
    GridAreaPlacement(GridAreaPlacement),
    GridAutoFlow(GridAutoFlow),
    GridFlowTolerance(GridFlowTolerance),
    StyleColor(StyleColor),
    Color(Color),
    Corners(Corners),
    Border(Border),
    BorderStyles(BorderStyles),
    BorderLineStyle(BorderLineStyle),
    CornerRadius(CornerRadius),
    BorderRadii(BorderRadii),
    Outline(Outline),
    OutlineStyle(OutlineStyle),
    OutlineWidth(OutlineWidth),
    FontFamilyList(FontFamilyList),
    FontWeight(FontWeight),
    TextSlant(TextSlant),
    FontStretch(FontStretch),
    FontVariant(FontVariant),
    FontFeatureSettings(FontFeatureSettings),
    Font(Font),
    AnimationNameList(AnimationNameList),
    AnimationIterationCountList(AnimationIterationCountList),
    AnimationDirectionList(AnimationDirectionList),
    AnimationFillModeList(AnimationFillModeList),
    AnimationPlayStateList(AnimationPlayStateList),
    AnimationList(AnimationList),
    ImageLayerList(ImageLayerList),
    PositionList(PositionList),
    BackgroundSizeList(BackgroundSizeList),
    BackgroundRepeatList(BackgroundRepeatList),
    BackgroundBox(BackgroundBox),
    BackgroundAttachmentList(BackgroundAttachmentList),
    MaskLayerList(MaskLayerList),
    BoxDecorationBreak(BoxDecorationBreak),
    Filter(Filter),
    ClipPath(ClipPath),
    TransitionPropertyList(TransitionPropertyList),
    TimeList(TimeList),
    EasingList(EasingList),
    TransitionList(TransitionList),
    ShadowList(Vec<Shadow>),
    Stroke(Stroke),
    Text(TextValue),
    Transform(Transform),
    Translate(Translate),
    Rotate(Rotate),
    Scale(Scale),
    Cursor(Cursor),
    PointerEvents(PointerEvents),
    UserSelect(UserSelect),
    Visibility(Visibility),
}

impl Value {
    #[must_use]
    pub const fn interpolation(&self) -> Interpolation {
        match self {
            Self::Number(_) => Interpolation::Number,
            Self::Length(_) => Interpolation::Length,
            Self::Size(_) => Interpolation::Length,
            Self::Edges(_) => Interpolation::Edges,
            Self::StyleColor(_) => Interpolation::Color,
            Self::Color(_) => Interpolation::Color,
            Self::Corners(_) => Interpolation::Corners,
            Self::CornerRadius(_) => Interpolation::Corners,
            Self::BorderRadii(_) => Interpolation::Corners,
            Self::OutlineWidth(_) => Interpolation::Length,
            Self::ShadowList(_) => Interpolation::ShadowList,
            Self::Stroke(_) => Interpolation::Stroke,
            Self::Transform(_) => Interpolation::Transform,
            Self::Translate(_) | Self::Rotate(_) | Self::Scale(_) => Interpolation::Transform,
            Self::FlexFactor(_) | Self::AspectRatio(_) | Self::FontWeight(_) => {
                Interpolation::Number
            }
            Self::TextIndent(_) | Self::VerticalAlign(_) | Self::LetterSpacing(_) => {
                Interpolation::Length
            }
            Self::TextDecorationThickness(_) => Interpolation::Length,
            Self::ImageLayerList(_)
            | Self::PositionList(_)
            | Self::BackgroundSizeList(_)
            | Self::BackgroundRepeatList(_)
            | Self::BackgroundBox(_)
            | Self::BackgroundAttachmentList(_)
            | Self::MaskLayerList(_)
            | Self::Content(_)
            | Self::ListStyleType(_)
            | Self::ListStylePosition(_)
            | Self::ListStyleImage(_)
            | Self::ListStyle(_)
            | Self::CounterChanges(_)
            | Self::BoxDecorationBreak(_)
            | Self::Filter(_)
            | Self::ClipPath(_) => Interpolation::Discrete,
            Self::Keyword(_)
            | Self::Display(_)
            | Self::BoxSizing(_)
            | Self::Position(_)
            | Self::ZIndex(_)
            | Self::ScrollbarWidth(_)
            | Self::ContentVisibility(_)
            | Self::Order(_)
            | Self::Flex(_)
            | Self::Direction(_)
            | Self::Overflow(_)
            | Self::OverflowAxes(_)
            | Self::Float(_)
            | Self::Clear(_)
            | Self::TextAlign(_)
            | Self::TextAlignLast(_)
            | Self::TextTransform(_)
            | Self::TextDecoration(_)
            | Self::TextDecorationLine(_)
            | Self::TextDecorationStyle(_)
            | Self::TextWrap(_)
            | Self::WhiteSpace(_)
            | Self::WordBreak(_)
            | Self::OverflowWrap(_)
            | Self::TextOverflow(_)
            | Self::WritingMode(_)
            | Self::FlexDirection(_)
            | Self::FlexWrap(_)
            | Self::AlignItems(_)
            | Self::AlignContent(_)
            | Self::PlaceContentAlignment(_)
            | Self::PlaceItemsAlignment(_)
            | Self::GridTrackList(_)
            | Self::GridTemplateAreas(_)
            | Self::GridTemplate(_)
            | Self::GridDefinition(_)
            | Self::GridLine(_)
            | Self::GridPlacement(_)
            | Self::GridAreaPlacement(_)
            | Self::GridAutoFlow(_)
            | Self::GridFlowTolerance(_)
            | Self::Border(_)
            | Self::BorderStyles(_)
            | Self::BorderLineStyle(_)
            | Self::Outline(_)
            | Self::OutlineStyle(_)
            | Self::FontFamilyList(_)
            | Self::TextSlant(_)
            | Self::FontStretch(_)
            | Self::FontVariant(_)
            | Self::FontFeatureSettings(_)
            | Self::Font(_)
            | Self::AnimationNameList(_)
            | Self::AnimationIterationCountList(_)
            | Self::AnimationDirectionList(_)
            | Self::AnimationFillModeList(_)
            | Self::AnimationPlayStateList(_)
            | Self::AnimationList(_)
            | Self::TransitionPropertyList(_)
            | Self::TimeList(_)
            | Self::EasingList(_)
            | Self::TransitionList(_)
            | Self::Text(_)
            | Self::Cursor(_)
            | Self::PointerEvents(_)
            | Self::UserSelect(_)
            | Self::Visibility(_) => Interpolation::Discrete,
        }
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Keyword(_) => Ok(()),
            Self::Display(_) => Ok(()),
            Self::BoxSizing(_)
            | Self::Position(_)
            | Self::ZIndex(_)
            | Self::ScrollbarWidth(_)
            | Self::ContentVisibility(_)
            | Self::ListStyleType(_)
            | Self::ListStylePosition(_)
            | Self::ListStyleImage(_)
            | Self::Order(_)
            | Self::Direction(_)
            | Self::Overflow(_)
            | Self::OverflowAxes(_)
            | Self::Float(_)
            | Self::Clear(_)
            | Self::TextAlign(_)
            | Self::TextAlignLast(_)
            | Self::TextTransform(_)
            | Self::WritingMode(_)
            | Self::FlexDirection(_)
            | Self::FlexWrap(_)
            | Self::AlignItems(_)
            | Self::AlignContent(_)
            | Self::PlaceContentAlignment(_)
            | Self::PlaceItemsAlignment(_) => Ok(()),
            Self::Number(value) => validate_finite(*value, "number"),
            Self::Content(value) => value.validate(),
            Self::ListStyle(value) => value.validate(),
            Self::CounterChanges(value) => value.validate(),
            Self::FlexFactor(value) => value.validate(),
            Self::Flex(value) => value.validate(),
            Self::AspectRatio(value) => value.validate(),
            Self::TextIndent(value) => validate_text_length(value.length(), "text-indent"),
            Self::VerticalAlign(value) => value.validate(),
            Self::LetterSpacing(value) => value.validate(),
            Self::TextDecoration(value) => value.validate(),
            Self::TextDecorationLine(value) => value.validate(),
            Self::TextDecorationStyle(_) => Ok(()),
            Self::TextDecorationThickness(value) => value.validate(),
            Self::TextWrap(_)
            | Self::WhiteSpace(_)
            | Self::WordBreak(_)
            | Self::OverflowWrap(_)
            | Self::TextOverflow(_) => Ok(()),
            Self::Length(value) => value.validate(),
            Self::Size(value) => value.validate(),
            Self::Edges(value) => value.validate(),
            Self::GridTrackList(value) => value.validate(),
            Self::GridTemplateAreas(value) => value.validate(),
            Self::GridTemplate(value) => value.validate(),
            Self::GridDefinition(value) => value.validate(),
            Self::GridLine(value) => value.validate(),
            Self::GridPlacement(value) => value.validate(),
            Self::GridAreaPlacement(value) => value.validate(),
            Self::GridAutoFlow(_) => Ok(()),
            Self::GridFlowTolerance(value) => value.validate(),
            Self::StyleColor(value) => value.validate(),
            Self::Color(value) => value.validate(),
            Self::Corners(value) => value.validate(),
            Self::Border(value) => value.validate(),
            Self::BorderStyles(_) | Self::BorderLineStyle(_) => Ok(()),
            Self::CornerRadius(value) => value.validate(),
            Self::BorderRadii(value) => value.validate(),
            Self::Outline(value) => value.validate(),
            Self::OutlineStyle(_) | Self::OutlineWidth(_) => Ok(()),
            Self::FontFamilyList(values) => values.validate(),
            Self::FontWeight(_)
            | Self::FontStretch(_)
            | Self::FontVariant(_)
            | Self::FontFeatureSettings(_) => Ok(()),
            Self::TextSlant(value) => validate_slant(*value),
            Self::Font(value) => value.validate(),
            Self::AnimationNameList(values) => values.validate(),
            Self::AnimationIterationCountList(values) => values.validate(),
            Self::AnimationDirectionList(values) => values.validate(),
            Self::AnimationFillModeList(values) => values.validate(),
            Self::AnimationPlayStateList(values) => values.validate(),
            Self::AnimationList(values) => values.validate(),
            Self::ImageLayerList(value) => {
                if value.layers().is_empty() {
                    Err(Error::new(
                        ErrorCode::InvalidValue,
                        "image layer list cannot be empty",
                    ))
                } else {
                    Ok(())
                }
            }
            Self::PositionList(value) => {
                if value.positions().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidValue,
                        "position list cannot be empty",
                    ));
                }
                value.positions().iter().try_for_each(Position::validate)
            }
            Self::BackgroundSizeList(value) => {
                if value.sizes().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidValue,
                        "background size list cannot be empty",
                    ));
                }
                value.sizes().iter().try_for_each(BackgroundSize::validate)
            }
            Self::BackgroundRepeatList(value) => {
                if value.repeats().is_empty() {
                    Err(Error::new(
                        ErrorCode::InvalidValue,
                        "background repeat list cannot be empty",
                    ))
                } else {
                    Ok(())
                }
            }
            Self::BackgroundBox(_) => Ok(()),
            Self::BackgroundAttachmentList(value) => {
                if value.attachments().is_empty() {
                    Err(Error::new(
                        ErrorCode::InvalidValue,
                        "background attachment list cannot be empty",
                    ))
                } else {
                    Ok(())
                }
            }
            Self::MaskLayerList(value) => {
                if value.layers().is_empty() {
                    return Err(Error::new(
                        ErrorCode::InvalidValue,
                        "mask layer list cannot be empty",
                    ));
                }
                value.layers().iter().try_for_each(MaskLayer::validate)
            }
            Self::ShadowList(shadows) => shadows.iter().try_for_each(|shadow| shadow.validate()),
            Self::Stroke(stroke) => stroke.validate(),
            Self::Text(text) => text.validate(),
            Self::Transform(transform) => transform.validate(),
            Self::BoxDecorationBreak(_) => Ok(()),
            Self::Filter(filter) => filter.validate(),
            Self::ClipPath(clip_path) => clip_path.validate(),
            Self::TransitionPropertyList(value) => value.validate(),
            Self::TimeList(value) => value.validate(),
            Self::EasingList(value) => value.validate(),
            Self::TransitionList(value) => value.validate(),
            Self::Translate(translate) => translate.validate(),
            Self::Rotate(rotate) => rotate.validate(),
            Self::Scale(scale) => scale.validate(),
            Self::Cursor(_) | Self::PointerEvents(_) | Self::UserSelect(_) => Ok(()),
            Self::Visibility(_) => Ok(()),
        }
    }
}

impl SymbolicFunctionValue {
    fn validate(&self) -> Result<()> {
        if self.0.trim().is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "symbolic function value cannot be empty",
            ))
        } else {
            Ok(())
        }
    }
}

impl FilterFunction {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Blur(value)
            | Self::Brightness(value)
            | Self::Contrast(value)
            | Self::DropShadow(value)
            | Self::Grayscale(value)
            | Self::HueRotate(value)
            | Self::Invert(value)
            | Self::Opacity(value)
            | Self::Saturate(value)
            | Self::Sepia(value) => value.validate(),
            Self::Url(url) => StyleUrl::new(url.as_str()).map(|_| ()),
        }
    }
}

impl FilterFunctionList {
    fn validate(&self) -> Result<()> {
        if self.functions.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "filter function list cannot be empty",
            ));
        }
        self.functions.iter().try_for_each(FilterFunction::validate)
    }
}

impl Filter {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Functions(functions) => functions.validate(),
        }
    }
}

impl BasicShape {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Inset(value)
            | Self::Circle(value)
            | Self::Ellipse(value)
            | Self::Polygon(value) => value.validate(),
        }
    }
}

impl ClipPath {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Url(url) => StyleUrl::new(url.as_str()).map(|_| ()),
            Self::BasicShape(shape) => shape.validate(),
        }
    }
}

impl Translate {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Values(values) => values.validate(),
        }
    }
}

impl TranslateValues {
    fn validate(&self) -> Result<()> {
        if self.values.is_empty() || self.values.len() > 3 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "translate requires one to three values",
            ));
        }
        self.values.iter().try_for_each(Length::validate)
    }
}

impl Rotate {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Value(value) => value.validate(),
        }
    }
}

impl Scale {
    fn validate(&self) -> Result<()> {
        match self {
            Self::None => Ok(()),
            Self::Values(values) => values.validate(),
        }
    }
}

impl ScaleValues {
    fn validate(&self) -> Result<()> {
        if self.values.is_empty()
            || self.values.len() > 3
            || self.values.iter().any(|value| !value.is_finite())
        {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "scale requires one to three finite values",
            ))
        } else {
            Ok(())
        }
    }
}

fn validate_text_length(length: &Length, property_name: &str) -> Result<()> {
    match length {
        Length::Px(_) | Length::Percent(_) | Length::Calc(_) => length.validate(),
        Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{property_name} accepts only text length values"),
        )),
    }
}

fn validate_letter_spacing_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(_) => length.validate(),
        Length::Calc(calc) if !calc.uses_percentage() => length.validate(),
        Length::Percent(_)
        | Length::Calc(_)
        | Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            "letter-spacing accepts only non-percentage length values",
        )),
    }
}

fn validate_text_decoration_thickness_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(value) | Length::Percent(value) if *value >= 0.0 => length.validate(),
        Length::Calc(calc) if !calc_is_definitely_negative(calc) => length.validate(),
        Length::Px(_)
        | Length::Percent(_)
        | Length::Calc(_)
        | Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            "text-decoration-thickness accepts only non-negative thickness lengths",
        )),
    }
}

pub(crate) fn validate_border_width_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(_) | Length::Calc(_) => {
            length.validate()?;
            validate_non_negative_style_length(length, "border width")
        }
        Length::Percent(_)
        | Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            "border width accepts only non-negative non-percentage lengths",
        )),
    }
}

pub(crate) fn validate_radius_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(_) | Length::Percent(_) | Length::Calc(_) => {
            length.validate()?;
            validate_non_negative_style_length(length, "corner radius")
        }
        Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            "corner radius accepts only non-negative length or percentage values",
        )),
    }
}

fn validate_non_negative_style_length(length: &Length, field: &'static str) -> Result<()> {
    match length {
        Length::Px(value) | Length::Percent(value) if *value < 0.0 => Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{field} must be non-negative"),
        )),
        Length::Calc(calc) if calc_is_definitely_negative(calc) => Err(Error::new(
            ErrorCode::InvalidValue,
            format!("{field} must be non-negative"),
        )),
        _ => Ok(()),
    }
}

fn calc_is_definitely_negative(calc: &CalcLength) -> bool {
    calc_coefficients(calc, 1.0).is_some_and(|coefficients| {
        coefficients.px < 0.0 && coefficients.percent <= 0.0
            || coefficients.px <= 0.0 && coefficients.percent < 0.0
    })
}

#[derive(Clone, Copy, Debug, Default)]
struct CalcCoefficients {
    px: f32,
    percent: f32,
}

fn calc_coefficients(calc: &CalcLength, sign: f32) -> Option<CalcCoefficients> {
    match calc {
        CalcLength::Px(value) => Some(CalcCoefficients {
            px: sign * *value,
            percent: 0.0,
        }),
        CalcLength::Percent(value) => Some(CalcCoefficients {
            px: 0.0,
            percent: sign * *value,
        }),
        CalcLength::Sum(terms) => {
            let mut total = CalcCoefficients::default();
            for term in terms {
                let term_sign = match term.operator() {
                    CalcOperator::Add => sign,
                    CalcOperator::Sub => -sign,
                };
                let coefficients = calc_coefficients(term.value(), term_sign)?;
                total.px += coefficients.px;
                total.percent += coefficients.percent;
            }
            Some(total)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Keyword {
    Inherit,
    Initial,
    Unset,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Display {
    Block,
    #[default]
    Flex,
    Grid,
    InlineBlock,
    InlineGrid,
    GridLanes,
    InlineGridLanes,
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum BoxSizing {
    ContentBox,
    #[default]
    BorderBox,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum LayoutPosition {
    Static,
    #[default]
    Relative,
    Absolute,
    Fixed,
    Sticky,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Direction {
    #[default]
    Ltr,
    Rtl,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Clip,
    Hidden,
    Scroll,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct OverflowAxes {
    pub x: Overflow,
    pub y: Overflow,
}

impl OverflowAxes {
    #[must_use]
    pub const fn new(x: Overflow, y: Overflow) -> Self {
        Self { x, y }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Float {
    #[default]
    None,
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Clear {
    #[default]
    None,
    Left,
    Right,
    Both,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum StyleTextAlign {
    #[default]
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
    LegacyLeft,
    LegacyRight,
    LegacyCenter,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextAlignLast {
    #[default]
    Auto,
    Start,
    End,
    Left,
    Right,
    Center,
    Justify,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextIndent {
    length: Length,
    hanging: bool,
    each_line: bool,
}

impl TextIndent {
    pub fn new(length: Length, hanging: bool, each_line: bool) -> Result<Self> {
        validate_text_length(&length, "text-indent")?;
        Ok(Self {
            length,
            hanging,
            each_line,
        })
    }

    #[must_use]
    pub const fn length(&self) -> &Length {
        &self.length
    }

    #[must_use]
    pub const fn hanging(&self) -> bool {
        self.hanging
    }

    #[must_use]
    pub const fn each_line(&self) -> bool {
        self.each_line
    }
}

impl Default for TextIndent {
    fn default() -> Self {
        Self {
            length: Length::ZERO,
            hanging: false,
            each_line: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum VerticalAlign {
    #[default]
    Baseline,
    Sub,
    Super,
    TextTop,
    TextBottom,
    Middle,
    Top,
    Bottom,
    Length(VerticalAlignLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct VerticalAlignLength(Length);

impl VerticalAlignLength {
    pub fn new(length: Length) -> Result<Self> {
        validate_text_length(&length, "vertical-align")?;
        Ok(Self(length))
    }

    #[must_use]
    pub const fn length(&self) -> &Length {
        &self.0
    }
}

impl VerticalAlign {
    pub fn try_length(length: Length) -> Result<Self> {
        Ok(Self::Length(VerticalAlignLength::new(length)?))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Length(length) => validate_text_length(length.length(), "vertical-align"),
            Self::Baseline
            | Self::Sub
            | Self::Super
            | Self::TextTop
            | Self::TextBottom
            | Self::Middle
            | Self::Top
            | Self::Bottom => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum LetterSpacing {
    #[default]
    Normal,
    Length(LetterSpacingLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetterSpacingLength(Length);

impl LetterSpacingLength {
    pub fn new(length: Length) -> Result<Self> {
        validate_letter_spacing_length(&length)?;
        Ok(Self(length))
    }

    #[must_use]
    pub const fn length(&self) -> &Length {
        &self.0
    }
}

impl LetterSpacing {
    pub fn try_length(length: Length) -> Result<Self> {
        Ok(Self::Length(LetterSpacingLength::new(length)?))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Normal => Ok(()),
            Self::Length(length) => validate_letter_spacing_length(length.length()),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextTransform {
    #[default]
    None,
    Capitalize,
    Uppercase,
    Lowercase,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TextDecorationLineComponent {
    Underline,
    Overline,
    LineThrough,
    Blink,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct TextDecorationLine {
    components: Vec<TextDecorationLineComponent>,
    none: bool,
}

impl TextDecorationLine {
    pub fn try_new(
        components: impl IntoIterator<Item = TextDecorationLineComponent>,
    ) -> Result<Self> {
        let components = components.into_iter().collect::<Vec<_>>();
        if components.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "text-decoration-line requires at least one component",
            ));
        }
        if has_duplicate_decoration_line_components(&components) {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "text-decoration-line components must not repeat",
            ));
        }
        Ok(Self {
            components,
            none: false,
        })
    }

    #[must_use]
    pub const fn none() -> Self {
        Self {
            components: Vec::new(),
            none: true,
        }
    }

    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.none
    }

    #[must_use]
    pub fn components(&self) -> &[TextDecorationLineComponent] {
        &self.components
    }

    pub fn validate(&self) -> Result<()> {
        if self.none {
            if self.components.is_empty() {
                Ok(())
            } else {
                Err(Error::new(
                    ErrorCode::InvalidValue,
                    "text-decoration-line none cannot include components",
                ))
            }
        } else {
            Self::try_new(self.components.iter().copied()).map(|_| ())
        }
    }
}

impl Default for TextDecorationLine {
    fn default() -> Self {
        Self::none()
    }
}

fn has_duplicate_decoration_line_components(components: &[TextDecorationLineComponent]) -> bool {
    components.iter().enumerate().any(|(index, component)| {
        components
            .iter()
            .skip(index + 1)
            .any(|candidate| candidate == component)
    })
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextDecorationStyle {
    #[default]
    Solid,
    Double,
    Dotted,
    Dashed,
    Wavy,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum TextDecorationThickness {
    #[default]
    Auto,
    FromFont,
    Length(TextDecorationThicknessLength),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextDecorationThicknessLength(Length);

impl TextDecorationThicknessLength {
    pub fn new(length: Length) -> Result<Self> {
        validate_text_decoration_thickness_length(&length)?;
        Ok(Self(length))
    }

    #[must_use]
    pub const fn length(&self) -> &Length {
        &self.0
    }
}

impl TextDecorationThickness {
    pub fn try_length(length: Length) -> Result<Self> {
        Ok(Self::Length(TextDecorationThicknessLength::new(length)?))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Auto | Self::FromFont => Ok(()),
            Self::Length(length) => validate_text_decoration_thickness_length(length.length()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextDecoration {
    line: Option<TextDecorationLine>,
    color: Option<StyleColor>,
    style: Option<TextDecorationStyle>,
    thickness: Option<TextDecorationThickness>,
}

impl TextDecoration {
    pub fn try_new(
        line: Option<TextDecorationLine>,
        color: Option<StyleColor>,
        style: Option<TextDecorationStyle>,
        thickness: Option<TextDecorationThickness>,
    ) -> Result<Self> {
        if line.is_none() && color.is_none() && style.is_none() && thickness.is_none() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "text-decoration shorthand requires at least one component",
            ));
        }
        let value = Self {
            line,
            color,
            style,
            thickness,
        };
        value.validate()?;
        Ok(value)
    }

    #[must_use]
    pub const fn line(&self) -> Option<&TextDecorationLine> {
        self.line.as_ref()
    }

    #[must_use]
    pub const fn color(&self) -> Option<&StyleColor> {
        self.color.as_ref()
    }

    #[must_use]
    pub const fn style(&self) -> Option<TextDecorationStyle> {
        self.style
    }

    #[must_use]
    pub const fn thickness(&self) -> Option<&TextDecorationThickness> {
        self.thickness.as_ref()
    }

    pub fn validate(&self) -> Result<()> {
        if self.line.is_none()
            && self.color.is_none()
            && self.style.is_none()
            && self.thickness.is_none()
        {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "text-decoration shorthand requires at least one component",
            ));
        }
        if let Some(line) = &self.line {
            line.validate()?;
        }
        if let Some(color) = &self.color {
            color.validate()?;
        }
        if let Some(thickness) = &self.thickness {
            thickness.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WritingMode {
    #[default]
    HorizontalTb,
    VerticalLr,
    VerticalRl,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlexDirection {
    #[default]
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    Wrap,
    WrapReverse,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AlignItems {
    #[default]
    Auto,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Baseline,
    LastBaseline,
    Stretch,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum AlignContent {
    #[default]
    Auto,
    Start,
    End,
    FlexStart,
    FlexEnd,
    Center,
    SafeEnd,
    SafeFlexEnd,
    SafeCenter,
    Stretch,
    SpaceBetween,
    SpaceEvenly,
    SpaceAround,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlaceContentAlignment {
    first: AlignContent,
    second: AlignContent,
}

impl PlaceContentAlignment {
    #[must_use]
    pub const fn new(first: AlignContent, second: AlignContent) -> Self {
        Self { first, second }
    }

    #[must_use]
    pub const fn all(value: AlignContent) -> Self {
        Self::new(value, value)
    }

    #[must_use]
    pub const fn first(self) -> AlignContent {
        self.first
    }

    #[must_use]
    pub const fn second(self) -> AlignContent {
        self.second
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PlaceItemsAlignment {
    first: AlignItems,
    second: AlignItems,
}

impl PlaceItemsAlignment {
    #[must_use]
    pub const fn new(first: AlignItems, second: AlignItems) -> Self {
        Self { first, second }
    }

    #[must_use]
    pub const fn all(value: AlignItems) -> Self {
        Self::new(value, value)
    }

    #[must_use]
    pub const fn first(self) -> AlignItems {
        self.first
    }

    #[must_use]
    pub const fn second(self) -> AlignItems {
        self.second
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridTrackList {
    pub components: Vec<GridTrackComponent>,
}

impl GridTrackList {
    #[must_use]
    pub fn new(components: Vec<GridTrackComponent>) -> Self {
        Self { components }
    }

    pub fn validate(&self) -> Result<()> {
        for component in &self.components {
            component.validate()?;
        }
        Ok(())
    }

    #[must_use]
    pub fn contains_subgrid(&self) -> bool {
        self.components
            .iter()
            .any(GridTrackComponent::contains_subgrid)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct GridLineName(String);

impl GridLineName {
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        validate_grid_line_name(&name)?;
        Ok(Self(name))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct GridLineNameSet(Vec<GridLineName>);

impl GridLineNameSet {
    pub fn new(names: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        let names = names
            .into_iter()
            .map(|name| GridLineName::new(name.into()))
            .collect::<Result<Vec<_>>>()?;
        Ok(Self(names))
    }

    #[must_use]
    pub fn as_slice(&self) -> &[GridLineName] {
        &self.0
    }

    #[must_use]
    pub fn to_strings(&self) -> Vec<String> {
        self.0.iter().map(|name| name.as_str().to_owned()).collect()
    }

    pub fn validate(&self) -> Result<()> {
        for name in &self.0 {
            validate_grid_line_name(name.as_str())?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SubgridLineNameSets(Vec<GridLineNameSet>);

impl SubgridLineNameSets {
    pub fn new(
        line_name_sets: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<String>>>,
    ) -> Result<Self> {
        let line_name_sets = line_name_sets
            .into_iter()
            .map(GridLineNameSet::new)
            .collect::<Result<Vec<_>>>()?;
        if line_name_sets.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "subgrid line-name repeat must contain at least one line-name set",
            ));
        }
        Ok(Self(line_name_sets))
    }

    #[must_use]
    pub fn as_slice(&self) -> &[GridLineNameSet] {
        &self.0
    }

    #[must_use]
    pub fn to_string_sets(&self) -> Vec<Vec<String>> {
        self.0.iter().map(GridLineNameSet::to_strings).collect()
    }

    pub fn validate(&self) -> Result<()> {
        if self.0.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "subgrid line-name repeat must contain at least one line-name set",
            ));
        }
        for names in &self.0 {
            names.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridTrackComponent {
    Track(TrackSizing),
    Repeat(TrackRepeat),
    LineNames(GridLineNameSet),
    Subgrid(SubgridTrack),
}

impl GridTrackComponent {
    pub fn line_names(names: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        Ok(Self::LineNames(GridLineNameSet::new(names)?))
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Track(track) => track.validate(),
            Self::Repeat(repeat) => repeat.validate(),
            Self::LineNames(names) => names.validate(),
            Self::Subgrid(subgrid) => subgrid.validate(),
        }
    }

    #[must_use]
    pub fn contains_subgrid(&self) -> bool {
        match self {
            Self::Subgrid(_) => true,
            Self::Repeat(repeat) => repeat.components.iter().any(Self::contains_subgrid),
            Self::Track(_) | Self::LineNames(_) => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackRepeat {
    pub(crate) count: TrackRepeatCount,
    pub(crate) components: Vec<GridTrackComponent>,
}

impl TrackRepeat {
    pub fn count(count: u16, components: Vec<GridTrackComponent>) -> Result<Self> {
        Self::new(TrackRepeatCount::count(count)?, components)
    }

    pub fn auto_fill(components: Vec<GridTrackComponent>) -> Result<Self> {
        Self::new(TrackRepeatCount::AutoFill, components)
    }

    pub fn auto_fit(components: Vec<GridTrackComponent>) -> Result<Self> {
        Self::new(TrackRepeatCount::AutoFit, components)
    }

    pub fn new(count: TrackRepeatCount, components: Vec<GridTrackComponent>) -> Result<Self> {
        if components.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "grid track repeat must contain at least one component",
            ));
        }
        let repeat = Self { count, components };
        repeat.validate()?;
        Ok(repeat)
    }

    #[must_use]
    pub const fn count_value(&self) -> TrackRepeatCount {
        self.count
    }

    #[must_use]
    pub fn components(&self) -> &[GridTrackComponent] {
        &self.components
    }

    pub fn validate(&self) -> Result<()> {
        self.count.validate()?;
        if self.components.is_empty() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "grid track repeat must contain at least one component",
            ));
        }
        for component in &self.components {
            component.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum TrackRepeatCount {
    Count(TrackRepeatCountValue),
    AutoFill,
    AutoFit,
}

impl TrackRepeatCount {
    pub fn count(count: u16) -> Result<Self> {
        Ok(Self::Count(TrackRepeatCountValue::new(count)?))
    }

    pub fn validate(self) -> Result<()> {
        match self {
            Self::Count(count) => count.validate(),
            Self::AutoFill | Self::AutoFit => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct TrackRepeatCountValue(u16);

impl TrackRepeatCountValue {
    pub fn new(count: u16) -> Result<Self> {
        let value = Self(count);
        value.validate()?;
        Ok(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    pub fn validate(self) -> Result<()> {
        if self.0 == 0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "grid track repeat count must be greater than zero",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubgridTrack {
    name_components: Vec<SubgridLineNameComponent>,
}

impl SubgridTrack {
    pub fn new(line_names: Vec<Vec<String>>) -> Result<Self> {
        let name_components = line_names
            .into_iter()
            .map(GridLineNameSet::new)
            .collect::<Result<Vec<_>>>()?
            .into_iter()
            .map(SubgridLineNameComponent::LineNames)
            .collect();
        Self::from_components(name_components)
    }

    pub fn from_components(name_components: Vec<SubgridLineNameComponent>) -> Result<Self> {
        let subgrid = Self { name_components };
        subgrid.validate()?;
        Ok(subgrid)
    }

    #[must_use]
    pub fn name_components(&self) -> &[SubgridLineNameComponent] {
        &self.name_components
    }

    #[must_use]
    pub fn line_names(&self) -> Vec<Vec<String>> {
        let mut line_names = Vec::new();
        for component in &self.name_components {
            match component {
                SubgridLineNameComponent::LineNames(names) => {
                    line_names.push(names.to_strings());
                }
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::Count(count),
                    line_name_sets,
                } => {
                    for _ in 0..count.get() {
                        line_names.extend(line_name_sets.to_string_sets());
                    }
                }
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::AutoFill,
                    line_name_sets,
                } => line_names.extend(line_name_sets.to_string_sets()),
            }
        }
        line_names
    }

    pub fn validate(&self) -> Result<()> {
        let mut auto_fill_count = 0usize;
        for component in &self.name_components {
            component.validate()?;
            if matches!(
                component,
                SubgridLineNameComponent::Repeat {
                    count: SubgridLineNameRepeatCount::AutoFill,
                    ..
                }
            ) {
                auto_fill_count += 1;
            }
        }
        if auto_fill_count > 1 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "subgrid line names cannot contain multiple auto-fill repeats",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum SubgridLineNameComponent {
    LineNames(GridLineNameSet),
    Repeat {
        count: SubgridLineNameRepeatCount,
        line_name_sets: SubgridLineNameSets,
    },
}

impl SubgridLineNameComponent {
    pub fn line_names(names: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        Ok(Self::LineNames(GridLineNameSet::new(names)?))
    }

    pub fn repeat(
        count: SubgridLineNameRepeatCount,
        line_name_sets: impl IntoIterator<Item = impl IntoIterator<Item = impl Into<String>>>,
    ) -> Result<Self> {
        Ok(Self::Repeat {
            count,
            line_name_sets: SubgridLineNameSets::new(line_name_sets)?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::LineNames(names) => names.validate(),
            Self::Repeat {
                count,
                line_name_sets,
            } => {
                count.validate()?;
                line_name_sets.validate()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SubgridLineNameRepeatCount {
    Count(SubgridLineNameRepeatCountValue),
    AutoFill,
}

impl SubgridLineNameRepeatCount {
    pub fn count(count: usize) -> Result<Self> {
        Ok(Self::Count(SubgridLineNameRepeatCountValue::new(count)?))
    }

    pub fn validate(self) -> Result<()> {
        match self {
            Self::Count(count) => count.validate(),
            Self::AutoFill => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SubgridLineNameRepeatCountValue(usize);

impl SubgridLineNameRepeatCountValue {
    pub fn new(count: usize) -> Result<Self> {
        let value = Self(count);
        value.validate()?;
        Ok(value)
    }

    #[must_use]
    pub const fn get(self) -> usize {
        self.0
    }

    pub fn validate(self) -> Result<()> {
        if self.0 == 0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "subgrid line-name repeat count must be greater than zero",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TrackSizing {
    pub min: MinTrackSizing,
    pub max: MaxTrackSizing,
}

impl TrackSizing {
    pub const AUTO: Self = Self {
        min: MinTrackSizing::Auto,
        max: MaxTrackSizing::Auto,
    };

    #[must_use]
    pub const fn px(value: f32) -> Self {
        Self {
            min: MinTrackSizing::Length(Length::Px(value)),
            max: MaxTrackSizing::Length(Length::Px(value)),
        }
    }

    #[must_use]
    pub const fn percent(value: f32) -> Self {
        Self {
            min: MinTrackSizing::Length(Length::Percent(value)),
            max: MaxTrackSizing::Length(Length::Percent(value)),
        }
    }

    #[must_use]
    pub const fn fr(value: f32) -> Self {
        Self {
            min: MinTrackSizing::Auto,
            max: MaxTrackSizing::Flex(value),
        }
    }

    #[must_use]
    pub const fn minmax(min: MinTrackSizing, max: MaxTrackSizing) -> Self {
        Self { min, max }
    }

    fn validate(&self) -> Result<()> {
        self.min.validate()?;
        self.max.validate()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MinTrackSizing {
    Length(Length),
    Auto,
    MinContent,
    MaxContent,
}

impl MinTrackSizing {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Length(length) => length.validate(),
            Self::Auto | Self::MinContent | Self::MaxContent => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum MaxTrackSizing {
    Length(Length),
    Flex(f32),
    Auto,
    MinContent,
    MaxContent,
    FitContent(Length),
}

impl MaxTrackSizing {
    #[must_use]
    pub const fn fr(value: f32) -> Self {
        Self::Flex(value)
    }

    fn validate(&self) -> Result<()> {
        match self {
            Self::Length(length) | Self::FitContent(length) => length.validate(),
            Self::Flex(value) => validate_non_negative(*value, "grid track flex"),
            Self::Auto | Self::MinContent | Self::MaxContent => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridTemplateAreas {
    pub rows: Vec<GridTemplateAreaRow>,
}

impl GridTemplateAreas {
    #[must_use]
    pub fn new(rows: impl IntoIterator<Item = GridTemplateAreaRow>) -> Self {
        Self {
            rows: rows.into_iter().collect(),
        }
    }

    pub fn validate(&self) -> Result<()> {
        let mut width = None;
        for row in &self.rows {
            row.validate()?;
            let row_width = row.cells.len();
            if row_width == 0 {
                return Err(Error::new(
                    ErrorCode::InvalidValue,
                    "grid template area rows cannot be empty",
                ));
            }
            if let Some(width) = width {
                if width != row_width {
                    return Err(Error::new(
                        ErrorCode::InvalidValue,
                        "grid template area rows must have equal widths",
                    ));
                }
            } else {
                width = Some(row_width);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridTemplateAreaRow {
    pub cells: Vec<Option<String>>,
}

impl GridTemplateAreaRow {
    #[must_use]
    pub fn new(cells: impl IntoIterator<Item = Option<impl Into<String>>>) -> Self {
        Self {
            cells: cells.into_iter().map(|cell| cell.map(Into::into)).collect(),
        }
    }

    #[must_use]
    pub fn named(names: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::new(names.into_iter().map(|name| Some(name.into())))
    }

    fn validate(&self) -> Result<()> {
        for name in self.cells.iter().flatten() {
            validate_grid_area_name(name)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridTemplate {
    pub rows: GridTrackList,
    pub columns: GridTrackList,
    pub areas: GridTemplateAreas,
}

impl GridTemplate {
    #[must_use]
    pub fn new(rows: GridTrackList, columns: GridTrackList, areas: GridTemplateAreas) -> Self {
        Self {
            rows,
            columns,
            areas,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.rows.validate()?;
        self.columns.validate()?;
        self.areas.validate()
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GridDefinition {
    pub template: GridTemplate,
    pub auto_rows: GridTrackList,
    pub auto_columns: GridTrackList,
    pub auto_flow: GridAutoFlow,
}

impl GridDefinition {
    #[must_use]
    pub fn new(template: GridTemplate) -> Self {
        Self {
            template,
            ..Self::default()
        }
    }

    #[must_use]
    pub fn auto_rows(mut self, tracks: GridTrackList) -> Self {
        self.auto_rows = tracks;
        self
    }

    #[must_use]
    pub fn auto_columns(mut self, tracks: GridTrackList) -> Self {
        self.auto_columns = tracks;
        self
    }

    #[must_use]
    pub const fn auto_flow(mut self, flow: GridAutoFlow) -> Self {
        self.auto_flow = flow;
        self
    }

    pub fn validate(&self) -> Result<()> {
        self.template.validate()?;
        self.auto_rows.validate()?;
        self.auto_columns.validate()?;
        if self.auto_rows.contains_subgrid() || self.auto_columns.contains_subgrid() {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "grid auto tracks cannot contain subgrid tracks",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridPlacement {
    pub start: GridLine,
    pub end: GridLine,
}

impl GridPlacement {
    pub const AUTO: Self = Self {
        start: GridLine::Auto,
        end: GridLine::Auto,
    };

    #[must_use]
    pub const fn new(start: GridLine, end: GridLine) -> Self {
        Self { start, end }
    }

    pub fn line(line: i16) -> Result<Self> {
        Ok(Self::new(GridLine::line(line)?, GridLine::Auto))
    }

    pub fn span_line(span: u16, line: i16) -> Result<Self> {
        Ok(Self::new(GridLine::span(span)?, GridLine::line(line)?))
    }

    pub fn validate(&self) -> Result<()> {
        self.start.validate()?;
        self.end.validate()
    }
}

impl Default for GridPlacement {
    fn default() -> Self {
        Self::AUTO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct GridAreaPlacement {
    pub row_start: GridLine,
    pub column_start: GridLine,
    pub row_end: GridLine,
    pub column_end: GridLine,
}

impl GridAreaPlacement {
    pub const AUTO: Self = Self {
        row_start: GridLine::Auto,
        column_start: GridLine::Auto,
        row_end: GridLine::Auto,
        column_end: GridLine::Auto,
    };

    #[must_use]
    pub const fn new(
        row_start: GridLine,
        column_start: GridLine,
        row_end: GridLine,
        column_end: GridLine,
    ) -> Self {
        Self {
            row_start,
            column_start,
            row_end,
            column_end,
        }
    }

    pub fn validate(&self) -> Result<()> {
        self.row_start.validate()?;
        self.column_start.validate()?;
        self.row_end.validate()?;
        self.column_end.validate()
    }
}

impl Default for GridAreaPlacement {
    fn default() -> Self {
        Self::AUTO
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum GridLine {
    Auto,
    Line(GridLineIndex),
    Span(GridSpanCount),
    BareIdent(GridLineName),
    NamedLine {
        name: GridLineName,
        index: GridLineIndex,
    },
    NamedSpan {
        name: GridLineName,
        index: GridSpanCount,
    },
}

impl GridLine {
    pub fn line(line: i16) -> Result<Self> {
        Ok(Self::Line(GridLineIndex::new(line)?))
    }

    pub fn span(span: u16) -> Result<Self> {
        Ok(Self::Span(GridSpanCount::new(span)?))
    }

    pub fn bare_ident(name: impl Into<String>) -> Result<Self> {
        Ok(Self::BareIdent(GridLineName::new(name)?))
    }

    pub fn named_line(name: impl Into<String>, index: i16) -> Result<Self> {
        Ok(Self::NamedLine {
            name: GridLineName::new(name)?,
            index: GridLineIndex::new(index)?,
        })
    }

    pub fn named_span(name: impl Into<String>, index: u16) -> Result<Self> {
        Ok(Self::NamedSpan {
            name: GridLineName::new(name)?,
            index: GridSpanCount::new(index)?,
        })
    }

    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Line(line) => line.validate("grid line index"),
            Self::Span(span) => span.validate("grid span count"),
            Self::BareIdent(name) => name.validate(),
            Self::NamedLine { name, index } => {
                name.validate()?;
                index.validate("named grid line index")
            }
            Self::NamedSpan { name, index } => {
                name.validate()?;
                index.validate("named grid span count")
            }
            Self::Auto => Ok(()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GridLineIndex(i16);

impl GridLineIndex {
    pub fn new(index: i16) -> Result<Self> {
        let value = Self(index);
        value.validate("grid line index")?;
        Ok(value)
    }

    #[must_use]
    pub const fn get(self) -> i16 {
        self.0
    }

    pub fn validate(self, label: &str) -> Result<()> {
        if self.0 == 0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                format!("{label} cannot be zero"),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct GridSpanCount(u16);

impl GridSpanCount {
    pub fn new(span: u16) -> Result<Self> {
        let value = Self(span);
        value.validate("grid span count")?;
        Ok(value)
    }

    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }

    pub fn validate(self, label: &str) -> Result<()> {
        if self.0 == 0 {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                format!("{label} cannot be zero"),
            ));
        }
        Ok(())
    }
}

impl GridLineName {
    pub fn validate(&self) -> Result<()> {
        validate_grid_line_name(self.as_str())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum GridAutoFlow {
    #[default]
    Row,
    Column,
    RowDense,
    ColumnDense,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub enum GridFlowTolerance {
    #[default]
    Normal,
    Length(Length),
    Percent(f32),
    Infinite,
}

impl GridFlowTolerance {
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Length(length) => length.validate(),
            Self::Percent(value) => validate_finite(*value, "grid flow tolerance percent"),
            Self::Normal | Self::Infinite => Ok(()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

impl Size {
    #[must_use]
    pub const fn new(width: Length, height: Length) -> Self {
        Self { width, height }
    }

    pub fn validate(&self) -> Result<()> {
        self.width.validate()?;
        self.height.validate()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Shadow {
    pub x: Length,
    pub y: Length,
    pub blur: Length,
    pub spread: Length,
    pub color: Color,
    pub inset: bool,
}

impl Shadow {
    #[must_use]
    pub const fn new(x: Length, y: Length, blur: Length, spread: Length, color: Color) -> Self {
        Self {
            x,
            y,
            blur,
            spread,
            color,
            inset: false,
        }
    }

    #[must_use]
    pub const fn soft(alpha: f32) -> Self {
        Self::new(
            Length::Px(0.0),
            Length::Px(8.0),
            Length::Px(24.0),
            Length::Px(0.0),
            Color::raw_rgba(0.0, 0.0, 0.0, alpha),
        )
    }

    #[must_use]
    pub const fn inset(mut self, inset: bool) -> Self {
        self.inset = inset;
        self
    }

    pub fn validate(&self) -> Result<()> {
        self.x.validate()?;
        self.y.validate()?;
        self.blur.validate()?;
        self.spread.validate()?;
        self.color.validate()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Stroke {
    pub width: Length,
    pub color: Color,
    pub style: LineStyle,
    pub sides: SideSet,
    pub dash: Option<Dash>,
    pub align: StrokeAlign,
}

impl Stroke {
    #[must_use]
    pub const fn new(width: Length, color: Color) -> Self {
        Self {
            width,
            color,
            style: LineStyle::Solid,
            sides: SideSet::all(),
            dash: None,
            align: StrokeAlign::Center,
        }
    }

    #[must_use]
    pub const fn style(mut self, style: LineStyle) -> Self {
        self.style = style;
        self
    }

    #[must_use]
    pub const fn sides(mut self, sides: SideSet) -> Self {
        self.sides = sides;
        self
    }

    #[must_use]
    pub const fn dash(mut self, dash: Dash) -> Self {
        self.dash = Some(dash);
        self
    }

    #[must_use]
    pub const fn align(mut self, align: StrokeAlign) -> Self {
        self.align = align;
        self
    }

    pub fn validate(&self) -> Result<()> {
        self.width.validate()?;
        self.color.validate()?;
        self.sides.validate()?;
        if let Some(dash) = self.dash {
            dash.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum LineStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SideSet {
    pub top: bool,
    pub right: bool,
    pub bottom: bool,
    pub left: bool,
}

impl SideSet {
    #[must_use]
    pub const fn all() -> Self {
        Self {
            top: true,
            right: true,
            bottom: true,
            left: true,
        }
    }

    #[must_use]
    pub const fn horizontal() -> Self {
        Self {
            top: true,
            right: false,
            bottom: true,
            left: false,
        }
    }

    #[must_use]
    pub const fn vertical() -> Self {
        Self {
            top: false,
            right: true,
            bottom: false,
            left: true,
        }
    }

    #[must_use]
    pub const fn empty() -> Self {
        Self {
            top: false,
            right: false,
            bottom: false,
            left: false,
        }
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        !self.top && !self.right && !self.bottom && !self.left
    }

    pub fn validate(self) -> Result<()> {
        if self.is_empty() {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "stroke side set cannot be empty",
            ))
        } else {
            Ok(())
        }
    }
}

impl Default for SideSet {
    fn default() -> Self {
        Self::all()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    pub density: f32,
    pub phase: f32,
    pub rounded: bool,
    pub circular: bool,
}

impl Dash {
    #[must_use]
    pub const fn new(density: f32) -> Self {
        Self {
            density,
            phase: 0.0,
            rounded: false,
            circular: false,
        }
    }

    #[must_use]
    pub const fn dashed() -> Self {
        Self::new(1.0)
    }

    #[must_use]
    pub const fn dotted() -> Self {
        Self {
            density: 1.0,
            phase: 0.0,
            rounded: true,
            circular: true,
        }
    }

    #[must_use]
    pub const fn density(mut self, density: f32) -> Self {
        self.density = density;
        self
    }

    #[must_use]
    pub const fn phase(mut self, phase: f32) -> Self {
        self.phase = phase;
        self
    }

    #[must_use]
    pub const fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }

    #[must_use]
    pub const fn circular(mut self) -> Self {
        self.circular = true;
        self.rounded = true;
        self
    }

    pub fn validate(self) -> Result<()> {
        validate_non_negative(self.density, "dash density")?;
        validate_finite(self.phase, "dash phase")?;
        if self.density <= f32::EPSILON {
            return Err(Error::new(
                ErrorCode::InvalidValue,
                "dash density must be greater than zero",
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum StrokeAlign {
    #[default]
    Center,
    Inside,
    Outside,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum TextSlant {
    #[default]
    Normal,
    Italic,
    Oblique(Option<f32>),
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextWrap {
    #[default]
    Wrap,
    NoWrap,
    Balance,
    Pretty,
    Stable,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WhiteSpace {
    #[default]
    Normal,
    NoWrap,
    Pre,
    PreWrap,
    PreLine,
    BreakSpaces,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum WordBreak {
    #[default]
    Normal,
    BreakAll,
    KeepAll,
    BreakWord,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum OverflowWrap {
    #[default]
    Normal,
    BreakWord,
    Anywhere,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum TextOverflow {
    #[default]
    Clip,
    Ellipsis,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Decoration {
    enabled: bool,
    offset: Option<f32>,
    size: Option<f32>,
    brush: Option<Color>,
}

impl Decoration {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            enabled: false,
            offset: None,
            size: None,
            brush: None,
        }
    }

    pub fn solid(brush: Option<Color>) -> Result<Self> {
        let decoration = Self {
            enabled: true,
            offset: None,
            size: None,
            brush,
        };
        decoration.validate()?;
        Ok(decoration)
    }

    #[must_use]
    pub const fn enabled(self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn offset(self) -> Option<f32> {
        self.offset
    }

    #[must_use]
    pub const fn size(self) -> Option<f32> {
        self.size
    }

    #[must_use]
    pub const fn brush(self) -> Option<Color> {
        self.brush
    }

    pub fn with_offset(mut self, offset: f32) -> Result<Self> {
        validate_finite(offset, "text decoration offset")?;
        self.offset = Some(offset);
        Ok(self)
    }

    #[must_use]
    pub const fn without_offset(mut self) -> Self {
        self.offset = None;
        self
    }

    pub fn with_size(mut self, size: f32) -> Result<Self> {
        validate_non_negative(size, "text decoration size")?;
        self.size = Some(size);
        Ok(self)
    }

    #[must_use]
    pub const fn without_size(mut self) -> Self {
        self.size = None;
        self
    }

    pub fn with_brush(mut self, brush: Color) -> Result<Self> {
        validate_decoration_brush(brush)?;
        self.brush = Some(brush);
        Ok(self)
    }

    #[must_use]
    pub const fn without_brush(mut self) -> Self {
        self.brush = None;
        self
    }

    pub fn validate(self) -> Result<()> {
        if let Some(offset) = self.offset {
            validate_finite(offset, "text decoration offset")?;
        }
        if let Some(size) = self.size {
            validate_non_negative(size, "text decoration size")?;
        }
        if let Some(brush) = self.brush {
            validate_decoration_brush(brush)?;
        }
        Ok(())
    }
}

impl Default for Decoration {
    fn default() -> Self {
        Self::none()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextValue {
    pub font_family: Vec<String>,
    pub font_size: Length,
    pub font_weight: TextWeight,
    pub font_style: TextSlant,
    pub line_height: Length,
    pub color: Color,
    pub alignment: StyleTextAlign,
    pub wrap: TextWrap,
    pub white_space: WhiteSpace,
    pub word_break: WordBreak,
    pub overflow_wrap: OverflowWrap,
    pub underline: Decoration,
    pub strikethrough: Decoration,
    pub selection_color: Color,
}

impl TextValue {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn family(mut self, family: impl Into<String>) -> Self {
        self.font_family.push(family.into());
        self
    }

    #[must_use]
    pub fn size(mut self, size: Length) -> Self {
        self.font_size = size;
        self
    }

    #[must_use]
    pub const fn weight(mut self, weight: TextWeight) -> Self {
        self.font_weight = weight;
        self
    }

    #[must_use]
    pub const fn style(mut self, style: TextSlant) -> Self {
        self.font_style = style;
        self
    }

    #[must_use]
    pub fn line_height(mut self, line_height: Length) -> Self {
        self.line_height = line_height;
        self
    }

    #[must_use]
    pub const fn color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    #[must_use]
    pub const fn align(mut self, alignment: StyleTextAlign) -> Self {
        self.alignment = alignment;
        self
    }

    #[must_use]
    pub const fn wrap(mut self, wrap: TextWrap) -> Self {
        self.wrap = wrap;
        self
    }

    #[must_use]
    pub const fn white_space(mut self, white_space: WhiteSpace) -> Self {
        self.white_space = white_space;
        self
    }

    #[must_use]
    pub const fn word_break(mut self, word_break: WordBreak) -> Self {
        self.word_break = word_break;
        self
    }

    #[must_use]
    pub const fn overflow_wrap(mut self, overflow_wrap: OverflowWrap) -> Self {
        self.overflow_wrap = overflow_wrap;
        self
    }

    #[must_use]
    pub const fn underline(mut self, underline: Decoration) -> Self {
        self.underline = underline;
        self
    }

    #[must_use]
    pub const fn strikethrough(mut self, strikethrough: Decoration) -> Self {
        self.strikethrough = strikethrough;
        self
    }

    #[must_use]
    pub const fn selection_color(mut self, selection_color: Color) -> Self {
        self.selection_color = selection_color;
        self
    }

    pub fn validate(&self) -> Result<()> {
        for family in &self.font_family {
            validate_style_string(family, "font family")?;
        }
        self.font_size.validate()?;
        validate_slant(self.font_style)?;
        self.line_height.validate()?;
        self.color.validate()?;
        validate_decoration(self.underline)?;
        validate_decoration(self.strikethrough)?;
        self.selection_color.validate()
    }
}

impl Default for TextValue {
    fn default() -> Self {
        Self {
            font_family: Vec::new(),
            font_size: Length::Px(16.0),
            font_weight: TextWeight::Normal,
            font_style: TextSlant::Normal,
            line_height: Length::Percent(100.0),
            color: Color::BLACK,
            alignment: StyleTextAlign::Start,
            wrap: TextWrap::Wrap,
            white_space: WhiteSpace::Normal,
            word_break: WordBreak::Normal,
            overflow_wrap: OverflowWrap::Normal,
            underline: Decoration::none(),
            strikethrough: Decoration::none(),
            selection_color: Color::TRANSPARENT,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Transform {
    pub operations: Vec<TransformOp>,
}

impl Transform {
    pub fn validate(&self) -> Result<()> {
        for operation in &self.operations {
            operation.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransformOp {
    Translate { x: Length, y: Length },
    Scale { x: f32, y: f32 },
    Rotate { radians: f32 },
}

impl TransformOp {
    fn validate(&self) -> Result<()> {
        match self {
            Self::Translate { x, y } => {
                x.validate()?;
                y.validate()
            }
            Self::Scale { x, y } => {
                validate_finite(*x, "scale x")?;
                validate_finite(*y, "scale y")
            }
            Self::Rotate { radians } => validate_finite(*radians, "rotate radians"),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Visibility {
    #[default]
    Visible,
    Hidden,
    Collapsed,
    RetainedOnly,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum Cursor {
    #[default]
    Default,
    Pointer,
    Text,
    Grab,
    Grabbing,
    Crosshair,
    NotAllowed,
    ResizeHorizontal,
    ResizeVertical,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum PointerEvents {
    #[default]
    Auto,
    None,
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum UserSelect {
    #[default]
    Auto,
    Text,
    None,
    All,
    Contain,
}

fn validate_style_string(value: &str, field: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} cannot be empty"),
        ));
    }
    if value
        .chars()
        .any(|ch| ch == '\0' || (ch.is_control() && !matches!(ch, '\n' | '\r' | '\t')))
    {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} contains unsupported control character"),
        ));
    }
    Ok(())
}

fn validate_counter_identifier(value: &str, field: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} cannot be empty"),
        ));
    }
    if trimmed == "-" {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} cannot be a lone hyphen"),
        ));
    }
    if matches!(
        trimmed.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "none"
    ) {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} cannot use reserved identifier `{trimmed}`"),
        ));
    }

    let mut chars = trimmed.chars();
    let first = chars.next().expect("trimmed is not empty");
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} must start with `_` or an ASCII letter"),
        ));
    }
    if chars.any(|ch| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == '-')) {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{field} must contain only ASCII letters, digits, `_`, or `-`"),
        ));
    }

    Ok(trimmed.to_string())
}

fn validate_timing_ident(value: &str, label: &str) -> Result<String> {
    if value.is_empty() {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{label} cannot be empty"),
        ));
    }
    if value.contains('\0') {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{label} cannot contain U+0000"),
        ));
    }
    if is_css_wide_keyword(value) || matches!(value.to_ascii_lowercase().as_str(), "span" | "auto")
    {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("{label} uses a reserved CSS identifier"),
        ));
    }
    Ok(value.to_owned())
}

fn is_css_wide_keyword(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer"
    )
}

fn validate_grid_line_name(name: &str) -> Result<()> {
    validate_style_string(name, "grid line name")?;
    if matches!(name, "auto" | "span") {
        return Err(Error::new(
            ErrorCode::InvalidString,
            format!("grid line name cannot use reserved identifier `{name}`"),
        ));
    }
    Ok(())
}

fn validate_grid_area_name(name: &str) -> Result<()> {
    validate_style_string(name, "grid area name")
}

fn validate_slant(value: TextSlant) -> Result<()> {
    match value {
        TextSlant::Oblique(Some(angle)) => validate_finite(angle, "font oblique angle"),
        TextSlant::Normal | TextSlant::Italic | TextSlant::Oblique(None) => Ok(()),
    }
}

pub(crate) fn validate_font_size_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(_) | Length::Percent(_) | Length::Calc(_) => length.validate(),
        Length::Auto
        | Length::Normal
        | Length::Fill
        | Length::Fit
        | Length::MinContent
        | Length::MaxContent => Err(Error::new(
            ErrorCode::InvalidValue,
            "font-size accepts only font-size length values",
        )),
    }
}

pub(crate) fn validate_line_height_length(length: &Length) -> Result<()> {
    match length {
        Length::Px(_) | Length::Percent(_) | Length::Normal | Length::Calc(_) => length.validate(),
        Length::Auto | Length::Fill | Length::Fit | Length::MinContent | Length::MaxContent => {
            Err(Error::new(
                ErrorCode::InvalidValue,
                "line-height accepts only line-height length values",
            ))
        }
    }
}

fn validate_decoration(value: Decoration) -> Result<()> {
    value.validate()
}

fn validate_decoration_brush(brush: Color) -> Result<()> {
    validate_finite(brush.r, "text decoration brush r")?;
    validate_finite(brush.g, "text decoration brush g")?;
    validate_finite(brush.b, "text decoration brush b")?;
    validate_finite(brush.a, "text decoration brush a")
}

#[cfg(test)]
mod tests {
    use super::{
        AnimationDirection, AnimationDirectionList, AnimationFillMode, AnimationFillModeList,
        AnimationIterationCount, AnimationIterationCountList, AnimationIterationNumber,
        AnimationName, AnimationNameList, AnimationPlayState, AnimationPlayStateList,
        BuiltInCounterStyle, Color, Content, ContentItem, ContentItemList, ContentString,
        CounterChange, CounterChangeList, CounterChanges, CounterFunction, CounterName,
        CounterStyle, CounterStyleName, CountersFunction, CssPx, Decoration, DimensionLength,
        DurationSeconds, EasingArguments, EasingFunction, EasingList, ErrorCode, FontFamilyList,
        KeyframeBlock, KeyframeOffset, KeyframeSelectorList, KeyframesIdent, KeyframesName,
        KeyframesRule, Length, ListStyle, ListStyleImage, ListStylePosition, ListStyleType,
        OverflowWrap, StyleTextAlign, StyleUrl, TextSlant, TextValue, TextWeight, TextWrap,
        TimeList, TransitionItem, TransitionList, TransitionPropertyList, TransitionPropertyName,
        TransitionPropertyTarget, Value, WhiteSpace, WordBreak,
    };
    use crate::{
        AuthoredDeclaration, AuthoredDeclarations, AuthoredProperty, AuthoredValue, Property,
        StyleAttributeName,
    };

    #[test]
    fn dimension_length_px_rejects_negative_css_px() {
        let err = DimensionLength::px(CssPx::new(-1.0).expect("finite css px"))
            .expect_err("negative dimensions are invalid");

        assert_eq!(err.code(), ErrorCode::InvalidValue);
        assert_eq!(err.message(), "dimension length px must be non-negative");
    }

    #[test]
    fn string_list_wrappers_preserve_empty_defaults() {
        let font_families = FontFamilyList::empty();
        let animation_names = AnimationNameList::single_none();

        assert!(font_families.is_empty());
        assert_eq!(animation_names.names(), &[AnimationName::None]);
        Value::FontFamilyList(font_families).validate().unwrap();
        Value::AnimationNameList(animation_names)
            .validate()
            .unwrap();
    }

    #[test]
    fn string_list_wrappers_reject_empty_items_at_construction() {
        let font_error = FontFamilyList::new([""]).unwrap_err();
        let animation_error = AnimationNameList::try_new([]).unwrap_err();

        assert_eq!(font_error.code(), ErrorCode::InvalidString);
        assert_eq!(animation_error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn timing_lists_and_easing_values_preserve_symbolic_payloads() {
        let times = TimeList::try_new([
            DurationSeconds::new(0.2).unwrap(),
            DurationSeconds::new(1.0).unwrap(),
        ])
        .unwrap();
        assert_eq!(times.seconds().len(), 2);
        assert!(TimeList::try_new([]).is_err());

        let easing =
            EasingFunction::CubicBezier(EasingArguments::try_new("0.4, 0, 0.2, 1").unwrap());
        let list = EasingList::try_new([easing.clone()]).unwrap();
        assert_eq!(list.values(), &[easing]);
        assert_eq!(
            EasingArguments::try_new("").unwrap_err().code(),
            ErrorCode::InvalidString
        );
    }

    #[test]
    fn transition_models_require_non_empty_lists_and_items() {
        let property =
            TransitionPropertyTarget::Custom(TransitionPropertyName::try_new("opacity").unwrap());
        let item = TransitionItem::try_new(
            Some(property.clone()),
            Some(DurationSeconds::new(0.15).unwrap()),
            None,
            Some(EasingFunction::EaseOut),
        )
        .unwrap();
        let list = TransitionList::try_new([item]).unwrap();
        assert!(matches!(list.items()[0].property(), Some(value) if value == &property));
        assert!(TransitionItem::try_new(None, None, None, None).is_err());
        assert!(TransitionList::try_new([]).is_err());
        assert!(TransitionPropertyList::try_new([]).is_err());
        assert!(TransitionPropertyName::try_new("none").is_err());
        assert!(TransitionPropertyName::try_new("-webkit-transform").is_ok());
        assert!(TransitionPropertyName::try_new("フェード").is_ok());
        assert!(TransitionPropertyName::try_new("auto").is_err());
    }

    #[test]
    fn animation_longhand_lists_preserve_css_timing_shape() {
        let names = AnimationNameList::try_new([
            AnimationName::None,
            AnimationName::Keyframes(KeyframesName::Ident(
                KeyframesIdent::try_new("fade-in").unwrap(),
            )),
        ])
        .unwrap();
        assert_eq!(names.names().len(), 2);
        assert!(KeyframesIdent::try_new("none").is_err());

        let iterations = AnimationIterationCountList::try_new([
            AnimationIterationCount::Number(AnimationIterationNumber::try_new(2.5).unwrap()),
            AnimationIterationCount::Infinite,
        ])
        .unwrap();
        assert_eq!(iterations.values().len(), 2);
        assert!(AnimationIterationNumber::try_new(-1.0).is_err());
        assert!(AnimationIterationNumber::try_new(f32::INFINITY).is_err());

        let directions = AnimationDirectionList::try_new([
            AnimationDirection::Normal,
            AnimationDirection::Reverse,
        ])
        .unwrap();
        assert_eq!(directions.values().len(), 2);
        assert!(AnimationDirectionList::try_new([]).is_err());

        let fill_modes =
            AnimationFillModeList::try_new([AnimationFillMode::None, AnimationFillMode::Both])
                .unwrap();
        assert_eq!(fill_modes.values().len(), 2);
        assert!(AnimationFillModeList::try_new([]).is_err());

        let play_states = AnimationPlayStateList::try_new([
            AnimationPlayState::Running,
            AnimationPlayState::Paused,
        ])
        .unwrap();
        assert_eq!(play_states.values().len(), 2);
        assert!(AnimationPlayStateList::try_new([]).is_err());
    }

    #[test]
    fn keyframes_require_non_empty_unique_offsets_and_declarations() {
        let mut declarations = AuthoredDeclarations::new();
        declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Opacity),
                    AuthoredValue::Value(Value::Number(1.0)),
                )
                .unwrap(),
            )
            .unwrap();

        let selectors =
            KeyframeSelectorList::try_new([KeyframeOffset::from(), KeyframeOffset::to()]).unwrap();
        let block = KeyframeBlock::try_new(selectors, declarations).unwrap();
        let rule = KeyframesRule::try_new(
            KeyframesName::Ident(KeyframesIdent::try_new("fade-in").unwrap()),
            [block],
        )
        .unwrap();

        assert_eq!(rule.blocks().len(), 1);
        assert_eq!(KeyframeOffset::from().percent(), 0.0);
        assert_eq!(KeyframeOffset::to().percent(), 100.0);
        assert!(KeyframeOffset::try_new(f32::NAN).is_err());
        assert!(KeyframeOffset::try_new(-1.0).is_err());
        assert!(KeyframeOffset::try_new(101.0).is_err());
        assert!(KeyframeSelectorList::try_new([]).is_err());
        assert!(
            KeyframeSelectorList::try_new([KeyframeOffset::from(), KeyframeOffset::from()])
                .is_err()
        );
        assert!(
            KeyframeBlock::try_new(
                KeyframeSelectorList::try_new([KeyframeOffset::to()]).unwrap(),
                AuthoredDeclarations::new(),
            )
            .is_err()
        );
        assert!(
            KeyframesRule::try_new(
                KeyframesName::Ident(KeyframesIdent::try_new("fade-in").unwrap()),
                []
            )
            .is_err()
        );

        let mut duplicate_declarations = AuthoredDeclarations::new();
        duplicate_declarations
            .try_push(
                AuthoredDeclaration::try_new(
                    AuthoredProperty::Property(Property::Opacity),
                    AuthoredValue::Value(Value::Number(0.5)),
                )
                .unwrap(),
            )
            .unwrap();
        let duplicate_block_a = KeyframeBlock::try_new(
            KeyframeSelectorList::try_new([KeyframeOffset::from()]).unwrap(),
            duplicate_declarations.clone(),
        )
        .unwrap();
        let duplicate_block_b = KeyframeBlock::try_new(
            KeyframeSelectorList::try_new([KeyframeOffset::from()]).unwrap(),
            duplicate_declarations,
        )
        .unwrap();
        assert!(
            KeyframesRule::try_new(
                KeyframesName::Ident(KeyframesIdent::try_new("fade-in").unwrap()),
                [duplicate_block_a, duplicate_block_b],
            )
            .is_err()
        );
    }

    #[test]
    fn generated_content_values_preserve_symbolic_payloads() {
        let string = ContentString::try_new("Chapter ").unwrap();
        let url = StyleUrl::new("chapter.svg").unwrap();
        let counter_name = CounterName::try_new("section").unwrap();
        let named_style = CounterStyle::Named(CounterStyleName::try_new("legal").unwrap());
        let counter = CounterFunction::new(counter_name.clone(), Some(named_style.clone()));
        let counters = CountersFunction::new(
            counter_name.clone(),
            ContentString::try_new(".").unwrap(),
            Some(CounterStyle::BuiltIn(
                BuiltInCounterStyle::DecimalLeadingZero,
            )),
        );
        let attr_name = StyleAttributeName::new("data-label").unwrap();

        assert_eq!(string.as_str(), "Chapter ");
        assert_eq!(counter.name().as_str(), "section");
        assert_eq!(counter.style(), Some(&named_style));
        assert_eq!(counters.name().as_str(), "section");
        assert_eq!(counters.separator().as_str(), ".");
        assert_eq!(
            counters.style(),
            Some(&CounterStyle::BuiltIn(
                BuiltInCounterStyle::DecimalLeadingZero
            ))
        );

        let items = ContentItemList::try_new([
            ContentItem::String(string),
            ContentItem::Url(url),
            ContentItem::Counter(counter),
            ContentItem::Counters(counters),
            ContentItem::Attr(attr_name),
            ContentItem::OpenQuote,
            ContentItem::CloseQuote,
            ContentItem::NoOpenQuote,
            ContentItem::NoCloseQuote,
        ])
        .unwrap();
        assert_eq!(items.items().len(), 9);
        assert!(matches!(Content::default(), Content::Normal));
        assert!(matches!(Content::Items(items), Content::Items(_)));
        assert!(matches!(Content::None, Content::None));
    }

    #[test]
    fn generated_content_names_and_lists_validate_invariants() {
        let trimmed = CounterName::try_new("  section-1  ").unwrap();
        let style = CounterStyleName::try_new("_legal").unwrap();

        assert_eq!(trimmed.as_str(), "section-1");
        assert_eq!(style.as_str(), "_legal");

        for invalid in [
            "",
            " ",
            "-",
            "two words",
            "inherit",
            "initial",
            "unset",
            "revert",
            "revert-layer",
            "none",
            "1bad",
        ] {
            assert!(
                CounterName::try_new(invalid).is_err(),
                "counter name should reject {invalid:?}"
            );
            assert!(
                CounterStyleName::try_new(invalid).is_err(),
                "counter style name should reject {invalid:?}"
            );
        }

        let content_string_error = ContentString::try_new("bad\0string").unwrap_err();
        let empty_list_error = ContentItemList::try_new([]).unwrap_err();

        assert_eq!(content_string_error.code(), ErrorCode::InvalidString);
        assert_eq!(empty_list_error.code(), ErrorCode::InvalidValue);
    }

    #[test]
    fn list_style_models_preserve_marker_policy() {
        assert_eq!(
            ListStyleType::default(),
            ListStyleType::CounterStyle(CounterStyle::BuiltIn(BuiltInCounterStyle::Disc))
        );
        assert_eq!(ListStylePosition::default(), ListStylePosition::Outside);
        assert_eq!(ListStyleImage::default(), ListStyleImage::None);

        let style = ListStyle::try_new(
            Some(ListStyleType::String(
                ContentString::try_new("Section ").unwrap(),
            )),
            Some(ListStylePosition::Inside),
            Some(ListStyleImage::Url(StyleUrl::new("marker.svg").unwrap())),
        )
        .unwrap();

        assert!(matches!(
            style.style_type(),
            Some(ListStyleType::String(string)) if string.as_str() == "Section "
        ));
        assert_eq!(style.position(), Some(ListStylePosition::Inside));
        assert!(
            matches!(style.image(), Some(ListStyleImage::Url(url)) if url.as_str() == "marker.svg")
        );
        assert_eq!(
            ListStyle::try_new(None, None, None).unwrap_err().code(),
            ErrorCode::InvalidValue
        );
    }

    #[test]
    fn counter_changes_require_explicit_values_and_non_empty_lists() {
        let change = CounterChange::new(CounterName::try_new("section").unwrap(), -2);
        assert_eq!(change.name().as_str(), "section");
        assert_eq!(change.value(), -2);

        let changes = CounterChangeList::try_new([change.clone()]).unwrap();
        assert_eq!(changes.changes(), &[change]);
        assert!(matches!(CounterChanges::default(), CounterChanges::None));
        assert!(matches!(
            CounterChanges::Changes(changes),
            CounterChanges::Changes(_)
        ));
        assert_eq!(
            CounterChangeList::try_new([]).unwrap_err().code(),
            ErrorCode::InvalidValue
        );
    }

    #[test]
    fn text_value_defaults_preserve_style_text_contract() {
        let text = TextValue::default();

        assert!(text.font_family.is_empty());
        assert_eq!(text.font_size, Length::Px(16.0));
        assert_eq!(text.font_weight, TextWeight::Normal);
        assert_eq!(text.font_style, TextSlant::Normal);
        assert_eq!(text.line_height, Length::Percent(100.0));
        assert_eq!(text.color, Color::BLACK);
        assert_eq!(text.alignment, StyleTextAlign::Start);
        assert_eq!(text.wrap, TextWrap::Wrap);
        assert_eq!(text.white_space, WhiteSpace::Normal);
        assert_eq!(text.word_break, WordBreak::Normal);
        assert_eq!(text.overflow_wrap, OverflowWrap::Normal);
        assert_eq!(text.underline, Decoration::none());
        assert_eq!(text.strikethrough, Decoration::none());
        assert_eq!(text.selection_color, Color::TRANSPARENT);
        text.validate().unwrap();
    }

    #[test]
    fn text_slant_oblique_rejects_non_finite_angle() {
        let error = TextValue::new()
            .style(TextSlant::Oblique(Some(f32::NAN)))
            .validate()
            .unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidValue);
        assert_eq!(error.message(), "font oblique angle must be finite");
    }

    #[test]
    fn decoration_rejects_non_finite_offset() {
        let error = Decoration::none().with_offset(f32::INFINITY).unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidValue);
        assert_eq!(error.message(), "text decoration offset must be finite");
    }

    #[test]
    fn decoration_rejects_negative_and_non_finite_size() {
        let negative = Decoration::none().with_size(-1.0).unwrap_err();
        let non_finite = Decoration::none().with_size(f32::NAN).unwrap_err();

        assert_eq!(negative.code(), ErrorCode::InvalidValue);
        assert_eq!(
            negative.message(),
            "text decoration size must be non-negative"
        );
        assert_eq!(non_finite.code(), ErrorCode::InvalidValue);
        assert_eq!(non_finite.message(), "text decoration size must be finite");
    }

    #[test]
    fn decoration_rejects_non_finite_brush_channels() {
        let error = Decoration::solid(Some(Color::raw_rgba(0.0, f32::NAN, 0.0, 1.0))).unwrap_err();

        assert_eq!(error.code(), ErrorCode::InvalidValue);
        assert_eq!(error.message(), "text decoration brush g must be finite");
    }
}
