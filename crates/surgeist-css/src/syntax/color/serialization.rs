//! Canonical specified-color serialization over checked authored graphs.

use super::*;
use crate::CssAngleUnit;
use crate::{
    CssSpecifiedValueSerializationError as Error, CssSpecifiedValueSerializationLimits as Limits,
    specified_serialization::SpecifiedSerializationContext,
};

type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Mode {
    Standalone,
    Origin,
    DeclaredRelative,
    Mix,
}

enum Work<'a> {
    Color(&'a CssColor, Mode),
    Text(&'static str),
    Owned(String),
}

pub(super) fn serialize(value: &CssColor, limits: Limits) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut output = String::new();
    value.append_specified(&mut context, &mut output)?;
    Ok(output)
}

impl CssColor {
    /// Appends one checked color to a caller's cumulative specified-CSS budget.
    /// The caller owns the output and context for its entire composed value.
    pub(crate) fn append_specified(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        let mut work = Vec::new();
        reserve_work(&mut work, 1)?;
        work.push(Work::Color(self, Mode::Standalone));

        while let Some(item) = work.pop() {
            match item {
                Work::Text(text) => context.append(output, text)?,
                Work::Owned(text) => context.append(output, &text)?,
                Work::Color(color, mode) => {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    schedule_authored(color, mode, context, &mut work)?;
                }
            }
        }
        Ok(())
    }
}

fn reserve_work(work: &mut Vec<Work<'_>>, additional: usize) -> Result<()> {
    work.try_reserve(additional)
        .map_err(|_| Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow))
}

fn work_slots(base: usize, per_item: usize, items: usize) -> Result<usize> {
    per_item
        .checked_mul(items)
        .and_then(|count| base.checked_add(count))
        .ok_or_else(|| Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow))
}

fn schedule_authored<'a>(
    color: &'a CssColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
    work: &mut Vec<Work<'a>>,
) -> Result<()> {
    reserve_work(work, 1)?;
    use CssColorRepresentation as R;
    match &color.representation {
        R::CurrentColor => work.push(Work::Text("currentcolor")),
        R::Transparent => work.push(Work::Text("transparent")),
        R::Hex(value) => work.push(Work::Owned(serialize_hex(value, mode, context)?)),
        R::Named(value) => work.push(Work::Owned(value.name().to_ascii_lowercase())),
        R::System(value) => work.push(Work::Text(authored_system_name(*value))),
        R::Rgb(value) => work.push(Work::Owned(serialize_rgb(value, mode, context)?)),
        R::Hsl(value) => work.push(Work::Owned(serialize_hsl(value, mode, context)?)),
        R::Hwb(value) => work.push(Work::Owned(serialize_hwb(value, mode, context)?)),
        R::Lab(value) => work.push(Work::Owned(serialize_lab("lab", value, mode, context)?)),
        R::Lch(value) => work.push(Work::Owned(serialize_lch("lch", value, mode, context)?)),
        R::Oklab(value) => {
            work.push(Work::Owned(serialize_lab("oklab", value, mode, context)?));
        }
        R::Oklch(value) => {
            work.push(Work::Owned(serialize_lch("oklch", value, mode, context)?));
        }
        R::Predefined(value) => {
            work.push(Work::Owned(serialize_predefined(value, mode, context)?));
        }
        R::Custom(value) => work.push(Work::Owned(serialize_custom(value, mode, context)?)),
        R::RelativeCustom(value) => {
            schedule_relative_custom(value, Mode::DeclaredRelative, work, context)?;
        }
        R::Alpha(value) => schedule_alpha(value, Mode::DeclaredRelative, work, context)?,
        R::Relative(value) => schedule_relative(value, Mode::DeclaredRelative, work, context)?,
        R::ColorMix(value) => schedule_mix(value, work, context)?,
    }
    Ok(())
}

fn schedule_alpha<'a>(
    value: &'a CssAlphaColor,
    mode: Mode,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    debug_assert_eq!(mode, Mode::DeclaredRelative);
    reserve_work(work, 4)?;
    work.push(Work::Text(")"));
    if let Some(alpha) = value.alpha() {
        work.push(Work::Owned(serialize_relative_expression(
            alpha, true, context,
        )?));
        work.push(Work::Text(" / "));
    }
    work.push(Work::Color(value.source(), Mode::Origin));
    work.push(Work::Text("alpha(from "));
    Ok(())
}

fn schedule_relative_custom<'a>(
    value: &'a CssRelativeCustomColor,
    mode: Mode,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    debug_assert_eq!(mode, Mode::DeclaredRelative);
    reserve_work(work, work_slots(5, 2, value.channels().len())?)?;
    work.push(Work::Text(")"));
    if let Some(alpha) = value.alpha() {
        work.push(Work::Owned(serialize_profile_expression(
            alpha, true, context,
        )?));
        work.push(Work::Text(" / "));
    }
    for channel in value.channels().iter().rev() {
        work.push(Work::Owned(serialize_profile_expression(
            channel, false, context,
        )?));
        work.push(Work::Text(" "));
    }
    work.push(Work::Owned(escaped_identifier(
        value.profile().as_str(),
        context,
    )?));
    work.push(Work::Text(" "));
    work.push(Work::Color(value.source(), Mode::Origin));
    work.push(Work::Text("color(from "));
    Ok(())
}

fn schedule_relative<'a>(
    value: &'a CssRelativeColor,
    mode: Mode,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    debug_assert_eq!(mode, Mode::DeclaredRelative);
    reserve_work(work, work_slots(5, 2, value.channels().len())?)?;
    let (name, space) = relative_function(value.function());
    work.push(Work::Text(")"));
    if let Some(alpha) = value.alpha() {
        work.push(Work::Owned(serialize_relative_expression(
            alpha, true, context,
        )?));
        work.push(Work::Text(" / "));
    }
    for channel in value.channels().iter().rev() {
        work.push(Work::Owned(serialize_relative_expression(
            channel, false, context,
        )?));
        work.push(Work::Text(" "));
    }
    if let Some(space) = space {
        work.push(Work::Text(space));
        work.push(Work::Text(" "));
    }
    work.push(Work::Color(value.source(), Mode::Origin));
    work.push(Work::Owned(format!("{name}(from ")));
    Ok(())
}

fn schedule_mix<'a>(
    value: &'a CssColorMix,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    reserve_work(work, work_slots(6, 3, value.components().len())?)?;
    let weights = mix_weight_texts(value.components(), context)?;
    work.push(Work::Text(")"));
    for (index, (component, weight)) in value.components().iter().zip(weights).enumerate().rev() {
        if let Some(weight) = weight {
            work.push(Work::Owned(weight));
            work.push(Work::Text(" "));
        }
        work.push(Work::Color(component.color(), Mode::Mix));
        if index != 0 {
            work.push(Work::Text(", "));
        }
    }
    if let Some(interpolation) = value.interpolation() {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        if !is_default_mix(interpolation) {
            work.push(Work::Text(", "));
            work.push(Work::Owned(serialize_interpolation(
                interpolation,
                context,
            )?));
            work.push(Work::Text("in "));
        }
    }
    work.push(Work::Text("color-mix("));
    Ok(())
}

fn is_default_mix(value: &CssColorInterpolation) -> bool {
    value.predefined().is_some_and(|value| {
        value.space() == CssColorInterpolationSpace::Oklab && value.hue().is_none()
    })
}

fn escaped_identifier(value: &str, context: &SpecifiedSerializationContext) -> Result<String> {
    crate::numeric::capture_identifier(value, context)
}

fn relative_function(value: &CssRelativeColorFunction) -> (&'static str, Option<&'static str>) {
    match value {
        CssRelativeColorFunction::Rgb => ("rgb", None),
        CssRelativeColorFunction::Hsl => ("hsl", None),
        CssRelativeColorFunction::Hwb => ("hwb", None),
        CssRelativeColorFunction::Lab => ("lab", None),
        CssRelativeColorFunction::Lch => ("lch", None),
        CssRelativeColorFunction::Oklab => ("oklab", None),
        CssRelativeColorFunction::Oklch => ("oklch", None),
        CssRelativeColorFunction::Color(space) => ("color", Some(predefined_name(*space))),
    }
}

fn predefined_name(value: CssPredefinedColorSpace) -> &'static str {
    match value {
        CssPredefinedColorSpace::Srgb => "srgb",
        CssPredefinedColorSpace::SrgbLinear => "srgb-linear",
        CssPredefinedColorSpace::DisplayP3 => "display-p3",
        CssPredefinedColorSpace::DisplayP3Linear => "display-p3-linear",
        CssPredefinedColorSpace::A98Rgb => "a98-rgb",
        CssPredefinedColorSpace::ProphotoRgb => "prophoto-rgb",
        CssPredefinedColorSpace::Rec2020 => "rec2020",
        CssPredefinedColorSpace::XyzD50 => "xyz-d50",
        CssPredefinedColorSpace::XyzD65 => "xyz-d65",
    }
}

fn authored_system_name(value: CssSystemColor) -> &'static str {
    match value {
        CssSystemColor::Canvas => "canvas",
        CssSystemColor::CanvasText => "canvastext",
        CssSystemColor::LinkText => "linktext",
        CssSystemColor::VisitedText => "visitedtext",
        CssSystemColor::ActiveText => "activetext",
        CssSystemColor::ButtonFace => "buttonface",
        CssSystemColor::ButtonText => "buttontext",
        CssSystemColor::ButtonBorder => "buttonborder",
        CssSystemColor::Field => "field",
        CssSystemColor::FieldText => "fieldtext",
        CssSystemColor::Highlight => "highlight",
        CssSystemColor::HighlightText => "highlighttext",
        CssSystemColor::Mark => "mark",
        CssSystemColor::MarkText => "marktext",
        CssSystemColor::GrayText => "graytext",
        CssSystemColor::SelectedItem => "selecteditem",
        CssSystemColor::SelectedItemText => "selecteditemtext",
        CssSystemColor::AccentColor => "accentcolor",
        CssSystemColor::AccentColorText => "accentcolortext",
        CssSystemColor::ActiveBorder => "activeborder",
        CssSystemColor::ActiveCaption => "activecaption",
        CssSystemColor::AppWorkspace => "appworkspace",
        CssSystemColor::Background => "background",
        CssSystemColor::ButtonHighlight => "buttonhighlight",
        CssSystemColor::ButtonShadow => "buttonshadow",
        CssSystemColor::CaptionText => "captiontext",
        CssSystemColor::InactiveBorder => "inactiveborder",
        CssSystemColor::InactiveCaption => "inactivecaption",
        CssSystemColor::InactiveCaptionText => "inactivecaptiontext",
        CssSystemColor::InfoBackground => "infobackground",
        CssSystemColor::InfoText => "infotext",
        CssSystemColor::Menu => "menu",
        CssSystemColor::MenuText => "menutext",
        CssSystemColor::Scrollbar => "scrollbar",
        CssSystemColor::ThreeDDarkShadow => "threeddarkshadow",
        CssSystemColor::ThreeDFace => "threedface",
        CssSystemColor::ThreeDHighlight => "threedhighlight",
        CssSystemColor::ThreeDLightShadow => "threedlightshadow",
        CssSystemColor::ThreeDShadow => "threedshadow",
        CssSystemColor::Window => "window",
        CssSystemColor::WindowFrame => "windowframe",
        CssSystemColor::WindowText => "windowtext",
    }
}

struct LocalCss {
    text: String,
    limit: usize,
}

impl LocalCss {
    fn new(limit: usize) -> Self {
        Self {
            text: String::new(),
            limit,
        }
    }

    fn push(&mut self, value: &str) -> Result<()> {
        use crate::CssSpecifiedValueSerializationErrorKind as K;
        let next = self
            .text
            .len()
            .checked_add(value.len())
            .ok_or_else(|| Error::new(K::CapacityOverflow))?;
        if next > self.limit {
            return Err(Error::new(K::ByteLimit));
        }
        self.text
            .try_reserve(value.len())
            .map_err(|_| Error::new(K::CapacityOverflow))?;
        self.text.push_str(value);
        Ok(())
    }

    fn finish(self) -> String {
        self.text
    }
}

fn suffix_text(
    text: String,
    suffix: &str,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    let mut output = LocalCss::new(context.remaining_bytes());
    output.push(&text)?;
    output.push(suffix)?;
    Ok(output.finish())
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct Factor {
    numerator: u64,
    denominator: u64,
}

impl Factor {
    const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };
}

struct ProjectedScalar {
    text: String,
    number: Option<ScaledNumber>,
    exact: Option<crate::exact_decimal::ExactRational>,
    contextual: bool,
    missing: bool,
    percentage: bool,
    calculation: bool,
}

#[derive(Clone, Copy, Debug)]
struct ScaledNumber {
    coefficient: f64,
    exponent: i128,
}

impl ScaledNumber {
    const ZERO: Self = Self {
        coefficient: 0.0,
        exponent: 0,
    };

    fn new(coefficient: f64, exponent: i128) -> Self {
        if coefficient == 0.0 {
            return Self::ZERO;
        }
        if !coefficient.is_finite() {
            return Self {
                coefficient,
                exponent: 0,
            };
        }
        let normalization = coefficient.abs().log10().floor() as i128;
        Self {
            coefficient: coefficient / 10f64.powi(normalization as i32),
            exponent: exponent.saturating_add(normalization),
        }
    }

    fn from_binary64(value: f64) -> Self {
        Self::new(value, 0)
    }

    fn from_exact(value: &crate::exact_decimal::ExactRational) -> Result<Self> {
        let (coefficient, exponent) = value.scaled_binary64()?;
        Ok(Self::new(coefficient, exponent))
    }

    fn is_negative(self) -> bool {
        self.coefficient.is_sign_negative()
    }

    fn compare(self, other: Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        if self.coefficient.is_nan() || other.coefficient.is_nan() {
            return Ordering::Equal;
        }
        if self.coefficient.is_infinite() || other.coefficient.is_infinite() {
            return self
                .coefficient
                .partial_cmp(&other.coefficient)
                .expect("non-NaN scaled color values");
        }
        if self.coefficient == 0.0 || other.coefficient == 0.0 {
            return self
                .coefficient
                .partial_cmp(&other.coefficient)
                .expect("finite scaled color values");
        }
        if self.is_negative() != other.is_negative() {
            return if self.is_negative() {
                Ordering::Less
            } else {
                Ordering::Greater
            };
        }
        let magnitude = self.exponent.cmp(&other.exponent).then_with(|| {
            self.coefficient
                .abs()
                .partial_cmp(&other.coefficient.abs())
                .expect("finite scaled color values")
        });
        if self.is_negative() {
            magnitude.reverse()
        } else {
            magnitude
        }
    }

    fn add(self, other: Self) -> Self {
        if self.coefficient == 0.0 {
            return other;
        }
        if other.coefficient == 0.0 {
            return self;
        }
        let exponent = self.exponent.max(other.exponent);
        let left_shift = self.exponent.saturating_sub(exponent);
        let right_shift = other.exponent.saturating_sub(exponent);
        let left = if left_shift < -324 {
            0.0
        } else {
            self.coefficient * 10f64.powi(left_shift as i32)
        };
        let right = if right_shift < -324 {
            0.0
        } else {
            other.coefficient * 10f64.powi(right_shift as i32)
        };
        Self::new(left + right, exponent)
    }

    fn subtract(self, other: Self) -> Self {
        self.add(Self {
            coefficient: -other.coefficient,
            exponent: other.exponent,
        })
    }

    fn multiply(self, other: Self) -> Self {
        Self::new(
            self.coefficient * other.coefficient,
            self.exponent.saturating_add(other.exponent),
        )
    }

    fn divide(self, other: Self) -> Self {
        debug_assert_ne!(other.coefficient, 0.0);
        Self::new(
            self.coefficient / other.coefficient,
            self.exponent.saturating_sub(other.exponent),
        )
    }

    fn scale(self, factor: f64) -> Self {
        Self::new(self.coefficient * factor, self.exponent)
    }

    fn max_zero(self) -> Self {
        if self.is_negative() { Self::ZERO } else { self }
    }

    fn clamp(self, lower: f64, upper: f64) -> Self {
        let lower = Self::from_binary64(lower);
        let upper = Self::from_binary64(upper);
        if self.coefficient.is_nan() || self.compare(lower).is_lt() {
            lower
        } else if self.compare(upper).is_gt() {
            upper
        } else {
            self
        }
    }

    fn binary64(self) -> f64 {
        if !self.coefficient.is_finite() {
            return self.coefficient;
        }
        if self.coefficient == 0.0 || self.exponent < -324 {
            return 0.0;
        }
        debug_assert!(self.exponent <= 308);
        self.coefficient * 10f64.powi(self.exponent as i32)
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ComponentTarget {
    Preserve,
    Number,
    Percentage,
}

fn exact_factor(value: Factor) -> crate::exact_decimal::ExactFactor {
    crate::exact_decimal::ExactFactor {
        numerator: value.numerator,
        denominator: value.denominator,
    }
}

fn exact_text(
    representation: &str,
    factor: Factor,
    rounded_places: Option<usize>,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let value = crate::exact_decimal::ExactRational::from_lexical_factor(
        representation,
        exact_factor(factor),
        context,
    )?;
    match rounded_places {
        Some(places) => value.format_rounded(places, context.remaining_bytes(), context),
        None => value.format_exact(context.remaining_bytes(), context),
    }
}

fn component_projection(
    value: &CssColorComponent,
    number_factor: Factor,
    percentage_factor: Factor,
    target: ComponentTarget,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    component_projection_with_text(
        value,
        number_factor,
        percentage_factor,
        target,
        true,
        context,
    )
}

fn component_projection_with_text(
    value: &CssColorComponent,
    number_factor: Factor,
    percentage_factor: Factor,
    target: ComponentTarget,
    materialize_direct: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    use CssColorComponent as C;
    match value {
        C::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(ProjectedScalar {
                text: "none".into(),
                number: None,
                exact: None,
                contextual: false,
                missing: true,
                percentage: target == ComponentTarget::Percentage,
                calculation: false,
            })
        }
        C::Number(value) => {
            context.charge_input(1)?;
            let representation = value.numeric().representation();
            let exact = crate::exact_decimal::ExactRational::from_lexical_factor(
                representation,
                exact_factor(number_factor),
                context,
            )?;
            let number = ScaledNumber::from_exact(&exact)?;
            let text = if materialize_direct {
                exact
                    .clone_with_budget(context)?
                    .format_exact(context.remaining_bytes(), context)?
            } else {
                String::new()
            };
            Ok(ProjectedScalar {
                number: Some(number),
                exact: Some(exact),
                text,
                contextual: false,
                missing: false,
                percentage: target == ComponentTarget::Percentage,
                calculation: false,
            })
        }
        C::Percentage(value) => {
            context.charge_input(1)?;
            let representation = value.numeric().representation();
            let exact = crate::exact_decimal::ExactRational::from_lexical_factor(
                representation,
                exact_factor(percentage_factor),
                context,
            )?;
            let number = ScaledNumber::from_exact(&exact)?;
            let text = if materialize_direct {
                exact
                    .clone_with_budget(context)?
                    .format_exact(context.remaining_bytes(), context)?
            } else {
                String::new()
            };
            Ok(ProjectedScalar {
                number: Some(number),
                exact: Some(exact),
                text,
                contextual: false,
                missing: false,
                percentage: target != ComponentTarget::Number,
                calculation: false,
            })
        }
        C::NumberCalculation(value) => {
            let scale = match target {
                ComponentTarget::Preserve => crate::numeric::NumericProjectionScale::Identity,
                ComponentTarget::Number => crate::numeric::NumericProjectionScale::Number {
                    numerator: number_factor.numerator,
                    denominator: number_factor.denominator,
                },
                ComponentTarget::Percentage => {
                    crate::numeric::NumericProjectionScale::NumberToPercentage {
                        numerator: number_factor.numerator,
                        denominator: number_factor.denominator,
                    }
                }
            };
            let (text, outcome) = crate::numeric::capture_color_calculation_scaled(
                crate::numeric::ColorCalculationRef::Number(value),
                scale,
                context,
            )?;
            Ok(ProjectedScalar {
                text,
                number: outcome.scalar_value.map(ScaledNumber::from_binary64),
                exact: None,
                contextual: outcome.context_dependent,
                missing: false,
                percentage: false,
                calculation: true,
            })
        }
        C::PercentageCalculation(value) => {
            let scale = if target == ComponentTarget::Number {
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: percentage_factor.numerator,
                    denominator: percentage_factor.denominator,
                }
            } else {
                crate::numeric::NumericProjectionScale::Identity
            };
            let (text, outcome) = crate::numeric::capture_color_calculation_scaled(
                crate::numeric::ColorCalculationRef::Percentage(value),
                scale,
                context,
            )?;
            Ok(ProjectedScalar {
                text,
                number: outcome.scalar_value.map(ScaledNumber::from_binary64),
                exact: None,
                contextual: outcome.context_dependent,
                missing: false,
                percentage: false,
                calculation: true,
            })
        }
    }
}

fn serialize_alpha(
    value: Option<&CssColorComponent>,
    origin: bool,
    retain_explicit: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if origin {
        let projected = component_projection(
            value,
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            context,
        )?;
        let suffix = if projected.percentage && !projected.missing {
            "%"
        } else {
            ""
        };
        return Ok(Some(suffix_text(projected.text, suffix, context)?));
    }
    if matches!(value, CssColorComponent::None) {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        return Ok(Some("none".into()));
    }
    match value {
        CssColorComponent::NumberCalculation(calculation) => {
            let (text, _) = crate::numeric::capture_color_calculation(
                crate::numeric::ColorCalculationRef::Number(calculation),
                context,
            )?;
            Ok(Some(text))
        }
        CssColorComponent::PercentageCalculation(calculation) => {
            let (text, _) = crate::numeric::capture_color_calculation_scaled(
                crate::numeric::ColorCalculationRef::Percentage(calculation),
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                },
                context,
            )?;
            Ok(Some(text))
        }
        CssColorComponent::Number(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                false,
                retain_explicit,
                context,
            )
        }
        CssColorComponent::Percentage(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                true,
                retain_explicit,
                context,
            )
        }
        CssColorComponent::None => unreachable!("handled above"),
    }
}

fn alpha_literal(
    representation: &str,
    percentage: bool,
    retain_explicit: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<Option<String>> {
    let factor = if percentage {
        Factor {
            numerator: 1,
            denominator: 100,
        }
    } else {
        Factor::ONE
    };
    let value = crate::exact_decimal::ExactRational::from_lexical_factor_clamped_unit(
        representation,
        exact_factor(factor),
        context,
    )?;
    if value.equals_ratio(1, 1, context)? && !retain_explicit {
        return Ok(None);
    }
    Ok(Some(value.format_rounded(
        6,
        context.remaining_bytes(),
        context,
    )?))
}

fn serialize_hex(
    value: &CssHexColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let digits = value.digits();
    let expanded: Vec<u8> = match digits.len() {
        3 | 4 => digits
            .bytes()
            .map(|byte| (hex_digit(byte) << 4) | hex_digit(byte))
            .collect(),
        6 | 8 => digits
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
            .collect(),
        _ => unreachable!("checked hex color"),
    };
    let alpha = expanded.get(3).copied();
    let mut out = LocalCss::new(context.remaining_bytes());
    let modern = mode == Mode::Origin;
    out.push(if alpha.is_some() && !modern {
        "rgba("
    } else {
        "rgb("
    })?;
    for (index, channel) in expanded[..3].iter().enumerate() {
        if index != 0 {
            out.push(if modern { " " } else { ", " })?;
        }
        out.push(&channel.to_string())?;
    }
    if let Some(alpha) = alpha.filter(|alpha| *alpha != 255) {
        out.push(if modern { " / " } else { ", " })?;
        out.push(&legacy_byte_alpha(alpha))?;
    }
    out.push(")")?;
    Ok(out.finish())
}

fn hex_digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        b'A'..=b'F' => value - b'A' + 10,
        _ => unreachable!("checked hex digit"),
    }
}

fn legacy_byte_alpha(value: u8) -> String {
    for places in [2i32, 3] {
        let factor = 10f64.powi(places);
        let rounded = (f64::from(value) / 255.0 * factor).round() / factor;
        if (rounded * 255.0).round() as u8 == value {
            return crate::specified_serialization::format_binary64(rounded);
        }
    }
    unreachable!("three decimals recover every alpha byte")
}

fn serialize_rgb(
    value: &CssRgbColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    if mode == Mode::Origin {
        let channels = value
            .channels()
            .iter()
            .map(|channel| {
                component_projection(
                    channel,
                    Factor::ONE,
                    Factor::ONE,
                    ComponentTarget::Preserve,
                    context,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let alpha = serialize_alpha(value.alpha(), true, false, context)?;
        return modern_function("rgb", &channels, alpha.as_deref(), context);
    }

    let alpha = serialize_alpha(value.alpha(), false, false, context)?;
    if value.channels().iter().any(CssColorComponent::is_none) {
        let mut channels = value
            .channels()
            .iter()
            .map(|channel| {
                component_projection_with_text(
                    channel,
                    Factor {
                        numerator: 1,
                        denominator: 255,
                    },
                    Factor {
                        numerator: 1,
                        denominator: 100,
                    },
                    ComponentTarget::Number,
                    false,
                    context,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        for channel in &mut channels {
            if channel.text.is_empty() {
                channel.text = channel
                    .exact
                    .as_ref()
                    .expect("unmaterialized direct scalar")
                    .clone_with_budget(context)?
                    .format_exact_or_rounded(6, context.remaining_bytes(), context)?;
            }
        }
        return modern_function("color(srgb", &channels, alpha.as_deref(), context);
    }
    let mut channels = value
        .channels()
        .iter()
        .map(|channel| {
            component_projection_with_text(
                channel,
                Factor::ONE,
                Factor {
                    numerator: 255,
                    denominator: 100,
                },
                ComponentTarget::Number,
                false,
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    if channels.iter().all(|channel| !channel.contextual) {
        let text = channels
            .iter()
            .map(|channel| -> Result<String> {
                let number = channel.number.expect("noncontextual direct color scalar");
                if !channel.calculation {
                    let exact = channel.exact.as_ref().expect("direct exact channel");
                    if exact.compare_integer(0, context)?.is_le() {
                        Ok("0".into())
                    } else if exact.compare_integer(255, context)?.is_ge() {
                        Ok("255".into())
                    } else {
                        exact
                            .clone_with_budget(context)?
                            .format_exact(context.remaining_bytes(), context)
                    }
                } else if number.coefficient.is_nan() || number.compare(ScaledNumber::ZERO).is_le()
                {
                    Ok("0".into())
                } else if number.compare(ScaledNumber::from_binary64(255.0)).is_ge() {
                    Ok("255".into())
                } else if channel.calculation {
                    Ok(rounded_f64(number.binary64(), 6))
                } else {
                    unreachable!("direct channels handled exactly")
                }
            })
            .collect::<Result<Vec<_>>>()?;
        return legacy_rgb(&text, alpha.as_deref(), context);
    }
    for channel in &mut channels {
        materialize_direct(channel, context)?;
    }
    modern_function("rgb", &channels, alpha.as_deref(), context)
}

fn materialize_direct(
    value: &mut ProjectedScalar,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    if value.text.is_empty() {
        value.text = value
            .exact
            .as_ref()
            .expect("unmaterialized direct scalar")
            .clone_with_budget(context)?
            .format_exact(context.remaining_bytes(), context)?;
    }
    Ok(())
}

fn modern_function(
    name: &str,
    channels: &[ProjectedScalar],
    alpha: Option<&str>,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    let mut out = LocalCss::new(context.remaining_bytes());
    out.push(name)?;
    if name.starts_with("color(") {
        out.push(" ")?;
    } else {
        out.push("(")?;
    }
    for (index, channel) in channels.iter().enumerate() {
        if index != 0 {
            out.push(" ")?;
        }
        out.push(&channel.text)?;
        if channel.percentage && !channel.missing && !channel.text.ends_with('%') {
            out.push("%")?;
        }
    }
    if let Some(alpha) = alpha {
        out.push(" / ")?;
        out.push(alpha)?;
    }
    out.push(")")?;
    Ok(out.finish())
}

fn legacy_rgb(
    channels: &[String],
    alpha: Option<&str>,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    let mut out = LocalCss::new(context.remaining_bytes());
    out.push(if alpha.is_some() { "rgba(" } else { "rgb(" })?;
    for (index, channel) in channels.iter().enumerate() {
        if index != 0 {
            out.push(", ")?;
        }
        out.push(channel)?;
    }
    if let Some(alpha) = alpha {
        out.push(", ")?;
        out.push(alpha)?;
    }
    out.push(")")?;
    Ok(out.finish())
}

fn rounded_f64(value: f64, places: i32) -> String {
    let factor = 10f64.powi(places);
    let rounded = (value * factor).round() / factor;
    crate::specified_serialization::format_binary64(if rounded == 0.0 { 0.0 } else { rounded })
}

fn serialize_hsl(
    value: &CssHslColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let missing = matches!(value.hue(), CssColorHue::None)
        || value.saturation().is_none()
        || value.lightness().is_none();
    let target = if mode == Mode::Origin {
        ComponentTarget::Preserve
    } else if missing {
        ComponentTarget::Percentage
    } else {
        ComponentTarget::Preserve
    };
    let hue = hue_projection(value.hue(), mode == Mode::Origin, context)?;
    let materialize = mode == Mode::Origin || missing;
    let saturation = component_projection_with_text(
        value.saturation(),
        Factor::ONE,
        Factor::ONE,
        target,
        materialize,
        context,
    )?;
    let mut lightness = component_projection_with_text(
        value.lightness(),
        Factor::ONE,
        Factor::ONE,
        target,
        materialize,
        context,
    )?;
    let alpha = serialize_alpha(value.alpha(), mode == Mode::Origin, false, context)?;
    if mode == Mode::Origin {
        return hsl_like(
            "hsl",
            &hue,
            &saturation,
            &lightness,
            alpha.as_deref(),
            context,
        );
    }
    if hue.missing || saturation.missing || lightness.missing {
        return hsl_like(
            "hsl",
            &hue,
            &saturation,
            &lightness,
            alpha.as_deref(),
            context,
        );
    }
    if hue.contextual || saturation.contextual || lightness.contextual {
        let saturation = clamp_direct_negative(saturation);
        let mut saturation = saturation;
        materialize_direct(&mut saturation, context)?;
        materialize_direct(&mut lightness, context)?;
        return hsl_like(
            "hsl",
            &hue,
            &saturation,
            &lightness,
            alpha.as_deref(),
            context,
        );
    }
    if let (Some(hue), Some(saturation), Some(lightness)) = (
        hue.exact.as_ref(),
        saturation.exact.as_ref(),
        lightness.exact.as_ref(),
    ) {
        let channels = exact_hsl_text(hue, saturation, lightness, context)?;
        return legacy_rgb(&channels, alpha.as_deref(), context);
    }
    let hue = hue.number.expect("numeric hue").binary64();
    let saturation = saturation
        .number
        .expect("numeric saturation")
        .max_zero()
        .scale(0.01);
    let lightness = lightness.number.expect("numeric lightness").scale(0.01);
    let rgb = hsl_to_rgb_scaled(hue, saturation, lightness);
    let channels = rgb.map(|channel| rounded_f64(channel.clamp(0.0, 1.0).binary64() * 255.0, 6));
    legacy_rgb(&channels, alpha.as_deref(), context)
}

fn exact_hsl_text(
    hue: &crate::exact_decimal::ExactRational,
    saturation: &crate::exact_decimal::ExactRational,
    lightness: &crate::exact_decimal::ExactRational,
    context: &mut SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    use crate::exact_decimal::ExactRational as Exact;

    // At half lightness, hue vertices have channels (1 ± saturation) / 2.
    // Saturation at least one therefore clips each vertex to an exact endpoint,
    // without expanding a potentially enormous authored decimal exponent.
    if lightness.compare_integer(50, context)?.is_eq()
        && saturation.compare_integer(100, context)?.is_ge()
    {
        for (angle, channels) in [
            (0, ["255", "0", "0"]),
            (60, ["255", "255", "0"]),
            (120, ["0", "255", "0"]),
            (180, ["0", "255", "255"]),
            (240, ["0", "0", "255"]),
            (300, ["255", "0", "255"]),
        ] {
            if hue.compare_integer(angle, context)?.is_eq() {
                return Ok(channels.map(str::to_owned));
            }
        }
    }

    let saturation = saturation
        .clone_with_budget(context)?
        .scale_ratio(1, 100, context)?
        .max_zero(context)?;
    let lightness = lightness
        .clone_with_budget(context)?
        .scale_ratio(1, 100, context)?;
    let half = Exact::integer(1, context)?.scale_ratio(1, 2, context)?;
    let one = Exact::integer(1, context)?;
    let m2 = if lightness.compare(&half, context)?.is_le() {
        lightness.clone_with_budget(context)?.multiply_signed(
            saturation
                .clone_with_budget(context)?
                .add_signed(one, context)?,
            context,
        )?
    } else {
        let product = lightness
            .clone_with_budget(context)?
            .multiply_signed(saturation.clone_with_budget(context)?, context)?;
        lightness
            .clone_with_budget(context)?
            .add_signed(saturation, context)?
            .subtract_signed(product, context)?
    };
    let m1 = lightness
        .scale_ratio(2, 1, context)?
        .subtract_signed(m2.clone_with_budget(context)?, context)?;
    let mut channels = [
        exact_hue_channel(&m1, &m2, hue, 120, context)?,
        exact_hue_channel(&m1, &m2, hue, 0, context)?,
        exact_hue_channel(&m1, &m2, hue, -120, context)?,
    ];
    let mut output = [String::new(), String::new(), String::new()];
    for (target, channel) in output.iter_mut().zip(&mut channels) {
        let value = std::mem::replace(channel, Exact::integer(0, context)?)
            .clamp_unit(context)?
            .scale_ratio(255, 1, context)?;
        *target = value.format_rounded(6, context.remaining_bytes(), context)?;
    }
    Ok(output)
}

fn exact_hue_channel(
    m1: &crate::exact_decimal::ExactRational,
    m2: &crate::exact_decimal::ExactRational,
    hue: &crate::exact_decimal::ExactRational,
    offset: i64,
    context: &mut SpecifiedSerializationContext,
) -> Result<crate::exact_decimal::ExactRational> {
    use crate::exact_decimal::ExactRational as Exact;

    let offset_value = Exact::integer(offset.unsigned_abs(), context)?;
    let adjusted = if offset.is_negative() {
        hue.clone_with_budget(context)?
            .subtract_signed(offset_value, context)?
    } else {
        hue.clone_with_budget(context)?
            .add_signed(offset_value, context)?
    }
    .modulo(360, context)?;
    if adjusted.compare_integer(60, context)?.is_lt() {
        let ratio = adjusted.scale_ratio(1, 60, context)?;
        let delta = m2
            .clone_with_budget(context)?
            .subtract_signed(m1.clone_with_budget(context)?, context)?
            .multiply_signed(ratio, context)?;
        m1.clone_with_budget(context)?.add_signed(delta, context)
    } else if adjusted.compare_integer(180, context)?.is_lt() {
        m2.clone_with_budget(context)
    } else if adjusted.compare_integer(240, context)?.is_lt() {
        let ratio = Exact::integer(240, context)?
            .subtract_signed(adjusted, context)?
            .scale_ratio(1, 60, context)?;
        let delta = m2
            .clone_with_budget(context)?
            .subtract_signed(m1.clone_with_budget(context)?, context)?
            .multiply_signed(ratio, context)?;
        m1.clone_with_budget(context)?.add_signed(delta, context)
    } else {
        m1.clone_with_budget(context)
    }
}

fn serialize_hwb(
    value: &CssHwbColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let missing = matches!(value.hue(), CssColorHue::None)
        || value.whiteness().is_none()
        || value.blackness().is_none();
    let target = if mode == Mode::Origin {
        ComponentTarget::Preserve
    } else if missing {
        ComponentTarget::Percentage
    } else {
        ComponentTarget::Preserve
    };
    let hue = hue_projection(value.hue(), mode == Mode::Origin, context)?;
    let materialize = mode == Mode::Origin || missing;
    let mut white = component_projection_with_text(
        value.whiteness(),
        Factor::ONE,
        Factor::ONE,
        target,
        materialize,
        context,
    )?;
    let mut black = component_projection_with_text(
        value.blackness(),
        Factor::ONE,
        Factor::ONE,
        target,
        materialize,
        context,
    )?;
    let alpha = serialize_alpha(value.alpha(), mode == Mode::Origin, false, context)?;
    if mode == Mode::Origin || hue.missing || white.missing || black.missing {
        return hsl_like("hwb", &hue, &white, &black, alpha.as_deref(), context);
    }
    if hue.contextual || white.contextual || black.contextual {
        materialize_direct(&mut white, context)?;
        materialize_direct(&mut black, context)?;
        return hsl_like("hwb", &hue, &white, &black, alpha.as_deref(), context);
    }
    if let (Some(hue), Some(white), Some(black)) = (
        hue.exact.as_ref(),
        white.exact.as_ref(),
        black.exact.as_ref(),
    ) {
        let channels = exact_hwb_text(hue, white, black, context)?;
        return legacy_rgb(&channels, alpha.as_deref(), context);
    }
    let hue = hue.number.expect("numeric hue").binary64();
    let white = white.number.expect("numeric whiteness").scale(0.01);
    let black = black.number.expect("numeric blackness").scale(0.01);
    let rgb = hwb_to_rgb_scaled(hue, white, black);
    let channels = rgb.map(|channel| rounded_f64(channel.clamp(0.0, 1.0).binary64() * 255.0, 6));
    legacy_rgb(&channels, alpha.as_deref(), context)
}

fn exact_hwb_text(
    hue: &crate::exact_decimal::ExactRational,
    white: &crate::exact_decimal::ExactRational,
    black: &crate::exact_decimal::ExactRational,
    context: &mut SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    use crate::exact_decimal::ExactRational as Exact;

    let white = white
        .clone_with_budget(context)?
        .scale_ratio(1, 100, context)?;
    let black = black
        .clone_with_budget(context)?
        .scale_ratio(1, 100, context)?;
    let sum = white
        .clone_with_budget(context)?
        .add_signed(black, context)?;
    let one = Exact::integer(1, context)?;
    if sum.compare(&one, context)?.is_ge() {
        if white.compare_integer(0, context)?.is_le() {
            return Ok(["0".into(), "0".into(), "0".into()]);
        }
        if white.compare(&sum, context)?.is_ge() {
            return Ok(["255".into(), "255".into(), "255".into()]);
        }
        let rounded = white.rounded_positive_ratio(&sum, 255, 6, context)?;
        let text = Exact::integer(rounded, context)?
            .scale_ratio(1, 1_000_000, context)?
            .format_exact(context.remaining_bytes(), context)?;
        return Ok([text.clone(), text.clone(), text]);
    }

    let zero = Exact::integer(0, context)?;
    let channels = [
        exact_hue_channel(&zero, &one, hue, 120, context)?,
        exact_hue_channel(&zero, &one, hue, 0, context)?,
        exact_hue_channel(&zero, &one, hue, -120, context)?,
    ];
    let factor = one.subtract_signed(sum, context)?;
    let mut output = [String::new(), String::new(), String::new()];
    for (target, channel) in output.iter_mut().zip(channels) {
        let value = channel
            .multiply_signed(factor.clone_with_budget(context)?, context)?
            .add_signed(white.clone_with_budget(context)?, context)?
            .clamp_unit(context)?
            .scale_ratio(255, 1, context)?;
        *target = value.format_rounded(6, context.remaining_bytes(), context)?;
    }
    Ok(output)
}

fn clamp_direct_negative(mut value: ProjectedScalar) -> ProjectedScalar {
    if !value.calculation
        && value
            .number
            .is_some_and(|number| number.compare(ScaledNumber::ZERO).is_lt())
    {
        value.number = Some(ScaledNumber::ZERO);
        value.text = "0".into();
    }
    value
}

fn hsl_like(
    name: &str,
    hue: &ProjectedScalar,
    first: &ProjectedScalar,
    second: &ProjectedScalar,
    alpha: Option<&str>,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    modern_function(
        name,
        &[clone_scalar(hue), clone_scalar(first), clone_scalar(second)],
        alpha,
        context,
    )
}

fn clone_scalar(value: &ProjectedScalar) -> ProjectedScalar {
    ProjectedScalar {
        text: value.text.clone(),
        number: value.number,
        exact: None,
        contextual: value.contextual,
        missing: value.missing,
        percentage: value.percentage,
        calculation: value.calculation,
    }
}

fn hue_projection(
    value: &CssColorHue,
    origin: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    use CssColorHue as H;
    let (text, number, exact, contextual, missing, calculation) = match value {
        H::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            ("none".into(), None, None, false, true, false)
        }
        H::Number(value) => {
            context.charge_input(1)?;
            let source = value.numeric().representation();
            let exact = if origin {
                crate::exact_decimal::ExactRational::from_lexical_factor(
                    source,
                    exact_factor(Factor::ONE),
                    context,
                )?
            } else {
                crate::exact_decimal::ExactRational::from_lexical_factor_modulo(
                    source,
                    exact_factor(Factor::ONE),
                    360,
                    context,
                )?
            };
            let number = ScaledNumber::from_exact(&exact)?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, Some(number), Some(exact), false, false, false)
        }
        H::Angle(value) => {
            context.charge_input(1)?;
            let factor = angle_factor(value.unit());
            let source = value.numeric().representation();
            let exact = if origin {
                crate::exact_decimal::ExactRational::from_lexical_factor(
                    source,
                    exact_factor(factor),
                    context,
                )?
            } else {
                crate::exact_decimal::ExactRational::from_lexical_factor_modulo(
                    source,
                    exact_factor(factor),
                    360,
                    context,
                )?
            };
            let number = ScaledNumber::from_exact(&exact)?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, Some(number), Some(exact), false, false, false)
        }
        H::NumberCalculation(value) => {
            let (text, outcome) = crate::numeric::capture_color_calculation(
                crate::numeric::ColorCalculationRef::Number(value),
                context,
            )?;
            (
                text,
                outcome.scalar_value.map(ScaledNumber::from_binary64),
                None,
                outcome.context_dependent,
                false,
                true,
            )
        }
        H::AngleCalculation(value) => {
            let (text, outcome) = crate::numeric::capture_color_calculation(
                crate::numeric::ColorCalculationRef::Angle(value),
                context,
            )?;
            (
                text,
                outcome.scalar_value.map(ScaledNumber::from_binary64),
                None,
                outcome.context_dependent,
                false,
                true,
            )
        }
    };
    let text = if origin && !missing && !calculation {
        suffix_text(text, "deg", context)?
    } else {
        text
    };
    Ok(ProjectedScalar {
        text,
        number,
        exact,
        contextual,
        missing,
        percentage: false,
        calculation,
    })
}

fn angle_factor(unit: CssAngleUnit) -> Factor {
    match unit {
        CssAngleUnit::Degrees => Factor::ONE,
        CssAngleUnit::Gradians => Factor {
            numerator: 9,
            denominator: 10,
        },
        CssAngleUnit::Radians => Factor {
            numerator: 1_007_958_012_753_983,
            denominator: 17_592_186_044_416,
        },
        CssAngleUnit::Turns => Factor {
            numerator: 360,
            denominator: 1,
        },
    }
}

fn hsl_to_rgb(hue: f64, saturation: f64, lightness: f64) -> [f64; 3] {
    let hue = hue.rem_euclid(360.0) / 360.0;
    if saturation == 0.0 {
        return [lightness; 3];
    }
    let m2 = if lightness <= 0.5 {
        lightness * (saturation + 1.0)
    } else {
        lightness + saturation - lightness * saturation
    };
    let m1 = lightness * 2.0 - m2;
    fn hue_to_rgb(m1: f64, m2: f64, mut hue: f64) -> f64 {
        hue = hue.rem_euclid(1.0);
        if hue * 6.0 < 1.0 {
            m1 + (m2 - m1) * hue * 6.0
        } else if hue * 2.0 < 1.0 {
            m2
        } else if hue * 3.0 < 2.0 {
            m1 + (m2 - m1) * (2.0 / 3.0 - hue) * 6.0
        } else {
            m1
        }
    }
    [
        hue_to_rgb(m1, m2, hue + 1.0 / 3.0),
        hue_to_rgb(m1, m2, hue),
        hue_to_rgb(m1, m2, hue - 1.0 / 3.0),
    ]
}

fn hsl_to_rgb_scaled(
    hue: f64,
    saturation: ScaledNumber,
    lightness: ScaledNumber,
) -> [ScaledNumber; 3] {
    let hue = hue.rem_euclid(360.0) / 360.0;
    if saturation.coefficient == 0.0 {
        return [lightness; 3];
    }
    let one = ScaledNumber::from_binary64(1.0);
    let two = ScaledNumber::from_binary64(2.0);
    let half = ScaledNumber::from_binary64(0.5);
    let m2 = if lightness.compare(half).is_le() {
        lightness.multiply(saturation.add(one))
    } else {
        lightness
            .add(saturation)
            .subtract(lightness.multiply(saturation))
    };
    let m1 = lightness.multiply(two).subtract(m2);
    fn hue_to_rgb(m1: ScaledNumber, m2: ScaledNumber, mut hue: f64) -> ScaledNumber {
        hue = hue.rem_euclid(1.0);
        if hue * 6.0 < 1.0 {
            m1.add(m2.subtract(m1).scale(hue * 6.0))
        } else if hue * 2.0 < 1.0 {
            m2
        } else if hue * 3.0 < 2.0 {
            m1.add(m2.subtract(m1).scale((2.0 / 3.0 - hue) * 6.0))
        } else {
            m1
        }
    }
    [
        hue_to_rgb(m1, m2, hue + 1.0 / 3.0),
        hue_to_rgb(m1, m2, hue),
        hue_to_rgb(m1, m2, hue - 1.0 / 3.0),
    ]
}

fn hwb_to_rgb_scaled(hue: f64, white: ScaledNumber, black: ScaledNumber) -> [ScaledNumber; 3] {
    let sum = white.add(black);
    let one = ScaledNumber::from_binary64(1.0);
    if sum.compare(one).is_ge() {
        let gray = white.divide(sum);
        return [gray; 3];
    }
    let rgb = hsl_to_rgb(hue, 1.0, 0.5);
    let factor = one.subtract(sum);
    rgb.map(|channel| {
        ScaledNumber::from_binary64(channel)
            .multiply(factor)
            .add(white)
    })
}

fn serialize_lab(
    name: &str,
    value: &CssLabColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let (light_percentage, axis_percentage) = if origin {
        (Factor::ONE, Factor::ONE)
    } else if name == "lab" {
        (
            Factor::ONE,
            Factor {
                numerator: 5,
                denominator: 4,
            },
        )
    } else {
        (
            Factor {
                numerator: 1,
                denominator: 100,
            },
            Factor {
                numerator: 1,
                denominator: 250,
            },
        )
    };
    let target = if origin {
        ComponentTarget::Preserve
    } else {
        ComponentTarget::Number
    };
    let lightness = component_projection(
        value.lightness(),
        Factor::ONE,
        light_percentage,
        target,
        context,
    )?;
    let channels = [
        lightness,
        component_projection(value.a(), Factor::ONE, axis_percentage, target, context)?,
        component_projection(value.b(), Factor::ONE, axis_percentage, target, context)?,
    ];
    let alpha = serialize_alpha(value.alpha(), origin, false, context)?;
    modern_function(name, &channels, alpha.as_deref(), context)
}

fn serialize_lch(
    name: &str,
    value: &CssLchColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let (light_percentage, chroma_percentage) = if origin {
        (Factor::ONE, Factor::ONE)
    } else if name == "lch" {
        (
            Factor::ONE,
            Factor {
                numerator: 3,
                denominator: 2,
            },
        )
    } else {
        (
            Factor {
                numerator: 1,
                denominator: 100,
            },
            Factor {
                numerator: 1,
                denominator: 250,
            },
        )
    };
    let target = if origin {
        ComponentTarget::Preserve
    } else {
        ComponentTarget::Number
    };
    let lightness = component_projection(
        value.lightness(),
        Factor::ONE,
        light_percentage,
        target,
        context,
    )?;
    let channels = [
        lightness,
        component_projection(
            value.chroma(),
            Factor::ONE,
            chroma_percentage,
            target,
            context,
        )?,
        hue_projection(value.hue(), origin, context)?,
    ];
    let alpha = serialize_alpha(value.alpha(), origin, false, context)?;
    modern_function(name, &channels, alpha.as_deref(), context)
}

fn serialize_predefined(
    value: &CssPredefinedColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let channels = value
        .channels()
        .iter()
        .map(|channel| {
            component_projection(
                channel,
                Factor::ONE,
                if origin {
                    Factor::ONE
                } else {
                    Factor {
                        numerator: 1,
                        denominator: 100,
                    }
                },
                if origin {
                    ComponentTarget::Preserve
                } else {
                    ComponentTarget::Number
                },
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let alpha = serialize_alpha(value.alpha(), origin, false, context)?;
    let name = format!("color({}", predefined_name(value.color_space()));
    modern_function(&name, &channels, alpha.as_deref(), context)
}

fn serialize_custom(
    value: &CssCustomColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let channels = value
        .channels()
        .iter()
        .map(|channel| {
            component_projection(
                channel,
                Factor::ONE,
                if origin {
                    Factor::ONE
                } else {
                    Factor {
                        numerator: 1,
                        denominator: 100,
                    }
                },
                if origin {
                    ComponentTarget::Preserve
                } else {
                    ComponentTarget::Number
                },
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let alpha = serialize_alpha(value.alpha(), origin, false, context)?;
    let profile = escaped_identifier(value.profile().as_str(), context)?;
    let mut name = LocalCss::new(context.remaining_bytes());
    name.push("color(")?;
    name.push(&profile)?;
    let name = name.finish();
    modern_function(&name, &channels, alpha.as_deref(), context)
}

fn serialize_relative_expression(
    value: &CssRelativeColorExpression,
    alpha: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    use CssRelativeColorExpressionValue as V;
    match value.value() {
        V::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok("none".into())
        }
        V::Channel(channel) => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(relative_channel(*channel).into())
        }
        V::Number(value) => {
            context.charge_input(1)?;
            if alpha {
                Ok(
                    alpha_literal(value.numeric().representation(), false, true, context)?
                        .expect("explicit relative alpha retained"),
                )
            } else {
                exact_text(value.numeric().representation(), Factor::ONE, None, context)
            }
        }
        V::Percentage(value) => {
            context.charge_input(1)?;
            if alpha {
                Ok(
                    alpha_literal(value.numeric().representation(), true, true, context)?
                        .expect("explicit relative alpha retained"),
                )
            } else {
                let text =
                    exact_text(value.numeric().representation(), Factor::ONE, None, context)?;
                suffix_text(text, "%", context)
            }
        }
        V::Angle(value) => {
            context.charge_input(1)?;
            let exact = crate::exact_decimal::ExactRational::from_lexical_factor(
                value.numeric().representation(),
                exact_factor(angle_factor(value.unit())),
                context,
            )?;
            let text = exact.format_exact(context.remaining_bytes(), context)?;
            suffix_text(text, "deg", context)
        }
        V::Calculation(value) => {
            let scale = if alpha && value.result_type() == CssCalculationType::Percentage {
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                }
            } else {
                crate::numeric::NumericProjectionScale::Identity
            };
            let (text, _) = crate::numeric::capture_color_specified_scaled(
                &value.data.expression,
                scale,
                context,
            )?;
            Ok(text)
        }
    }
}

fn serialize_profile_expression(
    value: &crate::CssProfileColorExpression,
    alpha: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    match value.view() {
        crate::CssProfileColorExpressionRef::Literal(value) => {
            if alpha {
                serialize_alpha(Some(value), false, true, context)
                    .map(|value| value.expect("explicit custom relative alpha retained"))
            } else {
                let value = component_projection(
                    value,
                    Factor::ONE,
                    Factor {
                        numerator: 1,
                        denominator: 100,
                    },
                    ComponentTarget::Number,
                    context,
                )?;
                Ok(value.text)
            }
        }
        crate::CssProfileColorExpressionRef::Reference(value) => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            escaped_identifier(value.as_str(), context)
        }
        crate::CssProfileColorExpressionRef::Calculation(value) => {
            let scale = if value.result_type() == CssCalculationType::Percentage {
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                }
            } else {
                crate::numeric::NumericProjectionScale::Identity
            };
            let (text, _) = crate::numeric::capture_color_calculation_scaled(
                crate::numeric::ColorCalculationRef::Profile(value),
                scale,
                context,
            )?;
            Ok(text)
        }
    }
}

fn relative_channel(value: CssRelativeColorChannel) -> &'static str {
    match value {
        CssRelativeColorChannel::R => "r",
        CssRelativeColorChannel::G => "g",
        CssRelativeColorChannel::B => "b",
        CssRelativeColorChannel::H => "h",
        CssRelativeColorChannel::S => "s",
        CssRelativeColorChannel::L => "l",
        CssRelativeColorChannel::W => "w",
        CssRelativeColorChannel::A => "a",
        CssRelativeColorChannel::C => "c",
        CssRelativeColorChannel::X => "x",
        CssRelativeColorChannel::Y => "y",
        CssRelativeColorChannel::Z => "z",
        CssRelativeColorChannel::Alpha => "alpha",
    }
}

fn serialize_interpolation(
    value: &CssColorInterpolation,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    if let Some(value) = value.predefined() {
        Ok(serialize_interpolation_method(&value))
    } else {
        escaped_identifier(
            value
                .custom_profile()
                .expect("checked interpolation variant")
                .as_str(),
            context,
        )
    }
}

fn serialize_interpolation_method(value: &CssColorInterpolationMethod) -> String {
    let mut result = match value.space() {
        CssColorInterpolationSpace::Predefined(value) => predefined_name(value).to_owned(),
        CssColorInterpolationSpace::Hsl => "hsl".into(),
        CssColorInterpolationSpace::Hwb => "hwb".into(),
        CssColorInterpolationSpace::Lab => "lab".into(),
        CssColorInterpolationSpace::Lch => "lch".into(),
        CssColorInterpolationSpace::Oklab => "oklab".into(),
        CssColorInterpolationSpace::Oklch => "oklch".into(),
    };
    if let Some(hue) = value.hue() {
        result.push(' ');
        result.push_str(match hue {
            CssHueInterpolationMethod::Shorter => "shorter hue",
            CssHueInterpolationMethod::Longer => "longer hue",
            CssHueInterpolationMethod::Increasing => "increasing hue",
            CssHueInterpolationMethod::Decreasing => "decreasing hue",
        });
    }
    result
}

fn mix_weight_texts(
    components: &[CssColorMixComponent],
    context: &mut SpecifiedSerializationContext,
) -> Result<Vec<Option<String>>> {
    if components.iter().any(|component| {
        component
            .weight()
            .is_some_and(|weight| weight.calculation().is_some())
    }) {
        let mut output = Vec::new();
        output.try_reserve(components.len()).map_err(|_| {
            Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
        })?;
        for component in components {
            output.push(match component.weight() {
                None => None,
                Some(weight) if weight.literal_value().is_some() => {
                    let text = literal_weight(weight.literal_value().unwrap(), context)?;
                    Some(suffix_text(text, "%", context)?)
                }
                Some(weight) => {
                    let calculation = weight.calculation().expect("checked weight variant");
                    let (text, _) = crate::numeric::capture_color_calculation(
                        crate::numeric::ColorCalculationRef::Percentage(calculation),
                        context,
                    )?;
                    Some(text)
                }
            });
        }
        return Ok(output);
    }

    let mut exact = Vec::new();
    exact.try_reserve(components.len()).map_err(|_| {
        Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    })?;
    let mut sum: Option<crate::exact_decimal::ExactRational> = None;
    let mut omitted = 0usize;
    for component in components {
        let value = component
            .weight()
            .and_then(CssColorMixWeight::literal_value)
            .map(|value| exact_weight(value, context))
            .transpose()?;
        if let Some(value) = &value {
            sum = Some(match sum {
                Some(current) => current.add_nonnegative(value, context)?,
                None => value.clone_with_budget(context)?,
            });
        } else {
            omitted += 1;
        }
        exact.push(value);
    }
    let sum = sum.unwrap_or(crate::exact_decimal::ExactRational::from_lexical_factor(
        "0",
        exact_factor(Factor::ONE),
        context,
    )?);
    let generated = if omitted == 0 {
        None
    } else {
        Some(
            sum.min_integer(100, context)?
                .subtract_from_integer(100, context)?
                .divide_by(omitted)?,
        )
    };
    let count = components.len();
    let mut all_equal = true;
    for value in &exact {
        let value = value
            .as_ref()
            .or(generated.as_ref())
            .expect("every mix slot has an effective weight");
        if !value.equals_ratio(100, count as u64, context)? {
            all_equal = false;
            break;
        }
    }
    if all_equal {
        let mut output = Vec::new();
        output.try_reserve(components.len()).map_err(|_| {
            Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
        })?;
        output.resize_with(components.len(), || None);
        return Ok(output);
    }
    exact
        .into_iter()
        .map(|value| {
            if let Some(value) = value {
                let text = value.format_exact(context.remaining_bytes(), context)?;
                Ok(Some(suffix_text(text, "%", context)?))
            } else {
                let text = generated
                    .as_ref()
                    .expect("omitted slot has generated share")
                    .clone_with_budget(context)?
                    .format_rounded(6, context.remaining_bytes(), context)?;
                Ok(Some(suffix_text(text, "%", context)?))
            }
        })
        .collect()
}

fn exact_weight(
    value: &CssColorMixPercentage,
    context: &mut SpecifiedSerializationContext,
) -> Result<crate::exact_decimal::ExactRational> {
    context.charge_input(1)?;
    crate::exact_decimal::ExactRational::from_lexical_factor(
        value.literal().numeric().representation(),
        exact_factor(Factor::ONE),
        context,
    )
}

fn literal_weight(
    value: &CssColorMixPercentage,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    exact_weight(value, context)?.format_exact(context.remaining_bytes(), context)
}
