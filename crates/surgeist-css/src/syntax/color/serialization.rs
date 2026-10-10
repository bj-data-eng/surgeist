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

/// The specified format family's intrinsic gradient default. This is not a
/// used color or a result of interpolation, profile resolution, or evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GradientColorDefault {
    Srgb,
    Oklab,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GradientColorDefaults {
    pub(crate) original: GradientColorDefault,
    pub(crate) emitted: GradientColorDefault,
}

impl CssColor {
    pub(crate) fn gradient_color_defaults(&self) -> GradientColorDefaults {
        use CssColorRepresentation as R;
        use GradientColorDefault as D;
        // Color4 §13.2 qualifies Images4's default by format family, including
        // ordinary RGB/HSL/HWB with missing or mathematical channels. Keyword
        // colors use the selected frozen keyword/SRGB default. Relative legacy
        // functions use the separately qualified frozen nonkeyword/Oklab
        // fallback; Color5's comma-free syntax alone does not prove that rule.
        // The declared writer preserves every other new outer function family,
        // outside Color4's normative legacy list. No child or numeric result is
        // inspected, and no used-color or relative origin is evaluated here.
        let original = match &self.representation {
            R::CurrentColor
            | R::Transparent
            | R::Hex(_)
            | R::Named(_)
            | R::System(_)
            | R::Rgb(_)
            | R::Hsl(_)
            | R::Hwb(_) => D::Srgb,
            R::Lab(_)
            | R::Lch(_)
            | R::Oklab(_)
            | R::Oklch(_)
            | R::Predefined(_)
            | R::Custom(_)
            | R::RelativeCustom(_)
            | R::Alpha(_)
            | R::Relative(_)
            | R::ColorMix(_)
            | R::LightDark(_)
            | R::ContrastColor(_)
            | R::DeviceCmyk(_) => D::Oklab,
        };
        // Color4 §16.2.2 mandates nonlegacy color(srgb) output for missing RGB.
        // This typed branch is the only outer-family change in this owner.
        let missing_rgb = matches!(&self.representation, R::Rgb(value)
            if value.channels().iter().any(CssColorComponent::is_none)
                || value.alpha().is_some_and(CssColorComponent::is_none));
        GradientColorDefaults {
            original,
            emitted: if missing_rgb { D::Oklab } else { original },
        }
    }
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
        R::Named(value) => {
            if !context.output_suppressed() {
                work.push(Work::Owned(value.name().to_ascii_lowercase()));
            }
        }
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
        R::DeviceCmyk(value) => {
            work.push(Work::Owned(serialize_device_cmyk(value, mode, context)?))
        }
        R::RelativeCustom(value) => {
            schedule_relative_custom(value, Mode::DeclaredRelative, work, context)?;
        }
        R::Alpha(value) => schedule_alpha(value, Mode::DeclaredRelative, work, context)?,
        R::Relative(value) => schedule_relative(value, Mode::DeclaredRelative, work, context)?,
        R::ColorMix(value) => schedule_mix(value, work, context)?,
        R::LightDark(value) => {
            reserve_work(work, 5)?;
            work.push(Work::Text(")"));
            work.push(Work::Color(value.dark(), Mode::Standalone));
            work.push(Work::Text(", "));
            work.push(Work::Color(value.light(), Mode::Standalone));
            work.push(Work::Text("light-dark("));
        }
        R::ContrastColor(value) => {
            reserve_work(work, 3)?;
            work.push(Work::Text(")"));
            work.push(Work::Color(value.color(), Mode::Standalone));
            work.push(Work::Text("contrast-color("));
        }
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
            alpha,
            true,
            Factor::ONE,
            context,
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
            alpha,
            true,
            Factor::ONE,
            context,
        )?));
        work.push(Work::Text(" / "));
    }
    for (index, channel) in value.channels().iter().enumerate().rev() {
        work.push(Work::Owned(serialize_relative_expression(
            channel,
            false,
            relative_percentage_factor(value.function(), index),
            context,
        )?));
        work.push(Work::Text(" "));
    }
    if let Some(space) = space {
        work.push(Work::Text(space));
        work.push(Work::Text(" "));
    }
    work.push(Work::Color(value.source(), Mode::Origin));
    if !context.output_suppressed() {
        work.push(Work::Owned(format!("{name}(from ")));
    }
    Ok(())
}

fn schedule_mix<'a>(
    value: &'a CssColorMix,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    reserve_work(work, work_slots(6, 3, value.components().len())?)?;
    let weights = mix_weight_texts(
        value.components().iter().map(CssColorMixComponent::weight),
        context,
    )?;
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

pub(crate) fn is_default_mix(value: &CssColorInterpolation) -> bool {
    value.predefined().is_some_and(|value| {
        value.space() == CssColorInterpolationSpace::Oklab && value.hue().is_none()
    })
}

fn escaped_identifier(value: &str, context: &SpecifiedSerializationContext) -> Result<String> {
    if context.output_suppressed() {
        return Ok(String::new());
    }
    crate::serialization_escaping::capture_identifier(value, context)
}

fn ordinary_number(value: f64, context: &SpecifiedSerializationContext) -> Result<String> {
    if context.output_suppressed() {
        return Ok(String::new());
    }
    crate::numeric_formatting::format_ordinary_color_number(value, context.remaining_bytes())
}

fn output_text(value: &str, context: &SpecifiedSerializationContext) -> String {
    if context.output_suppressed() {
        String::new()
    } else {
        value.into()
    }
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
    suppressed: bool,
}

impl LocalCss {
    fn new(context: &SpecifiedSerializationContext) -> Self {
        Self {
            text: String::new(),
            limit: context.remaining_bytes(),
            suppressed: context.output_suppressed(),
        }
    }

    fn push(&mut self, value: &str) -> Result<()> {
        if self.suppressed {
            return Ok(());
        }
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
    let mut output = LocalCss::new(context);
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
    // Original post-scale outcome, before capture text or ScaledNumber loses
    // precision. None belongs to direct exact literals or contextual captures.
    scalar_value: Option<f64>,
    number: Option<ScaledNumber>,
    exact: Option<crate::exact_decimal::ExactRational>,
    contextual: bool,
    missing: bool,
    percentage: bool,
    calculation: bool,
    // A direct exact slot can defer its logical materialization until a
    // contextual HSL/HWB branch selects it. Emitted text is not phase state.
    direct_text_pending: bool,
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

/// Generic text belongs only to declared literal serialization. Identity and
/// decimal shifts borrow the source; rational conversion owns its bounded work.
fn generic_literal_text(
    representation: &str,
    factor: Factor,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let shift = match (factor.numerator, factor.denominator) {
        (1, 1) => Some(0),
        (1, 100) => Some(-2),
        _ => None,
    };
    if let Some(shift) = shift {
        context.charge_projection(1)?;
        if context.output_suppressed() {
            return Ok(String::new());
        }
        crate::numeric_formatting::format_css_number(
            representation,
            shift,
            context.remaining_bytes(),
        )
    } else {
        crate::exact_decimal::ExactRational::format_generic_number(
            representation,
            exact_factor(factor),
            context.remaining_bytes(),
            context,
        )
    }
}

/// The destination function and checked slot determine direct percentage units.
/// This policy never reaches references or calculation trees.
fn relative_percentage_factor(function: &CssRelativeColorFunction, index: usize) -> Factor {
    use CssRelativeColorFunction as F;
    match function {
        F::Rgb => Factor {
            numerator: 255,
            denominator: 100,
        },
        F::Hsl | F::Hwb => Factor::ONE,
        F::Lab if index != 0 => Factor {
            numerator: 5,
            denominator: 4,
        },
        F::Lch if index == 1 => Factor {
            numerator: 3,
            denominator: 2,
        },
        F::Lab | F::Lch => Factor::ONE,
        F::Oklab if index != 0 => Factor {
            numerator: 1,
            denominator: 250,
        },
        F::Oklch if index == 1 => Factor {
            numerator: 1,
            denominator: 250,
        },
        F::Oklab | F::Oklch | F::Color(_) => Factor {
            numerator: 1,
            denominator: 100,
        },
    }
}

fn declared_component_projection(
    value: &CssColorComponent,
    number_factor: Factor,
    percentage_factor: Factor,
    target: ComponentTarget,
    origin: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    if origin {
        let literal = match value {
            CssColorComponent::Number(value) => Some((value.numeric().representation(), false)),
            CssColorComponent::Percentage(value) => Some((value.numeric().representation(), true)),
            _ => None,
        };
        if let Some((source, percentage)) = literal {
            context.charge_input(1)?;
            return Ok(ProjectedScalar {
                scalar_value: None,
                text: generic_literal_text(source, Factor::ONE, context)?,
                number: None,
                exact: None,
                contextual: false,
                missing: false,
                percentage,
                calculation: false,
                direct_text_pending: false,
            });
        }
        let calculation = match value {
            CssColorComponent::NumberCalculation(value) => {
                Some(crate::numeric::ColorCalculationRef::Number(value))
            }
            CssColorComponent::HintedNumberCalculation(value) => {
                Some(crate::numeric::ColorCalculationRef::HintedNumber(value))
            }
            CssColorComponent::PercentageCalculation(value) => {
                Some(crate::numeric::ColorCalculationRef::Percentage(value))
            }
            _ => None,
        };
        if let Some(calculation) = calculation {
            let (text, outcome) = crate::numeric::capture_retained_color_calculation(
                calculation,
                crate::numeric::NumericProjectionScale::Identity,
                context,
            )?;
            return Ok(ProjectedScalar {
                scalar_value: outcome.scalar_value,
                text,
                number: outcome.scalar_value.map(ScaledNumber::from_binary64),
                exact: None,
                contextual: outcome.context_dependent,
                missing: false,
                percentage: false,
                calculation: true,
                direct_text_pending: false,
            });
        }
    }
    component_projection(value, number_factor, percentage_factor, target, context)
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

#[derive(Clone, Copy)]
enum LabComponentRange {
    LabLightness,
    OklabLightness,
    Chroma,
}

/// Parsed-domain bounds apply to standalone and ordinary Mix direct L/C
/// literals. Origin roles and calculation trees retain their projection and scratch.
fn lab_component_projection(
    value: &CssColorComponent,
    percentage_factor: Factor,
    range: LabComponentRange,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    let origin = mode == Mode::Origin;
    if !matches!(mode, Mode::Standalone | Mode::Mix) {
        return declared_component_projection(
            value,
            Factor::ONE,
            percentage_factor,
            if origin {
                ComponentTarget::Preserve
            } else {
                ComponentTarget::Number
            },
            origin,
            context,
        );
    }
    let (representation, factor, percentage) = match value {
        CssColorComponent::Number(value) => (value.numeric().representation(), Factor::ONE, false),
        CssColorComponent::Percentage(value) => {
            (value.numeric().representation(), percentage_factor, true)
        }
        _ => {
            return component_projection(
                value,
                Factor::ONE,
                percentage_factor,
                ComponentTarget::Number,
                context,
            );
        }
    };
    context.charge_input(1)?;
    // One selected direct-slot classification; its borrowed decimal comparison
    // allocates no rational or expanded text and does not charge per digit.
    context.charge_projection(1)?;
    let lexical = crate::exact_decimal::LexicalDecimal::new(representation);
    let endpoint = if lexical.len == 0 || lexical.negative {
        Some("0")
    } else {
        let upper = match range {
            LabComponentRange::LabLightness => Some((2, "100")),
            LabComponentRange::OklabLightness => Some((if percentage { 2 } else { 0 }, "1")),
            LabComponentRange::Chroma => None,
        };
        upper.and_then(|(exponent, text)| {
            // At this normalized one-digit scale, 1 equals the upper bound
            // and 2..=9 exceed it. Select both without a duplicate digit walk;
            // all other coefficients use the existing inclusive comparison.
            let at_boundary_scale = lexical.len == 1 && lexical.exponent == Some(exponent);
            (at_boundary_scale || !lexical.absolute_at_most("1", exponent)).then_some(text)
        })
    };
    if let Some(endpoint) = endpoint {
        let mut text = LocalCss::new(context);
        text.push(endpoint)?;
        return Ok(ProjectedScalar {
            text: text.finish(),
            scalar_value: None,
            number: None,
            exact: None,
            contextual: false,
            missing: false,
            percentage: false,
            calculation: false,
            direct_text_pending: false,
        });
    }
    direct_component_projection(representation, factor, false, true, context)
}

/// The caller owns the one direct-slot input visit. In-range and unbounded
/// literals retain the existing exact arithmetic, work, and bounded text.
fn direct_component_projection(
    representation: &str,
    factor: Factor,
    percentage: bool,
    materialize_direct: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    let exact = crate::exact_decimal::ExactRational::from_lexical_factor(
        representation,
        exact_factor(factor),
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
        scalar_value: None,
        number: Some(number),
        exact: Some(exact),
        text,
        contextual: false,
        missing: false,
        percentage,
        calculation: false,
        direct_text_pending: !materialize_direct,
    })
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
                scalar_value: None,
                text: output_text("none", context),
                number: None,
                exact: None,
                contextual: false,
                missing: true,
                percentage: target == ComponentTarget::Percentage,
                calculation: false,
                direct_text_pending: false,
            })
        }
        C::Number(value) => {
            context.charge_input(1)?;
            direct_component_projection(
                value.numeric().representation(),
                number_factor,
                target == ComponentTarget::Percentage,
                materialize_direct,
                context,
            )
        }
        C::Percentage(value) => {
            context.charge_input(1)?;
            direct_component_projection(
                value.numeric().representation(),
                percentage_factor,
                target != ComponentTarget::Number,
                materialize_direct,
                context,
            )
        }
        C::NumberCalculation(value) => number_calculation_projection(
            crate::numeric::ColorCalculationRef::Number(value),
            number_factor,
            target,
            context,
        ),
        C::HintedNumberCalculation(value) => number_calculation_projection(
            crate::numeric::ColorCalculationRef::HintedNumber(value),
            number_factor,
            target,
            context,
        ),
        C::PercentageCalculation(value) => {
            let scale = if target == ComponentTarget::Number {
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: percentage_factor.numerator,
                    denominator: percentage_factor.denominator,
                }
            } else {
                crate::numeric::NumericProjectionScale::Identity
            };
            let (text, outcome) = crate::numeric::capture_color_component_calculation(
                crate::numeric::ColorCalculationRef::Percentage(value),
                scale,
                context,
            )?;
            Ok(ProjectedScalar {
                scalar_value: outcome.scalar_value,
                text,
                number: outcome.scalar_value.map(ScaledNumber::from_binary64),
                exact: None,
                contextual: outcome.context_dependent,
                missing: false,
                percentage: false,
                calculation: true,
                direct_text_pending: false,
            })
        }
    }
}

fn number_calculation_projection(
    calculation: crate::numeric::ColorCalculationRef<'_>,
    factor: Factor,
    target: ComponentTarget,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    let scale = match target {
        ComponentTarget::Number
            if matches!(
                calculation,
                crate::numeric::ColorCalculationRef::HintedNumber(_)
            ) && factor.numerator == factor.denominator =>
        {
            crate::numeric::NumericProjectionScale::Identity
        }
        ComponentTarget::Preserve => crate::numeric::NumericProjectionScale::Identity,
        ComponentTarget::Number => crate::numeric::NumericProjectionScale::Number {
            numerator: factor.numerator,
            denominator: factor.denominator,
        },
        ComponentTarget::Percentage => crate::numeric::NumericProjectionScale::NumberToPercentage {
            numerator: factor.numerator,
            denominator: factor.denominator,
        },
    };
    let (text, outcome) =
        crate::numeric::capture_color_component_calculation(calculation, scale, context)?;
    Ok(ProjectedScalar {
        scalar_value: outcome.scalar_value,
        text,
        number: outcome.scalar_value.map(ScaledNumber::from_binary64),
        exact: None,
        contextual: outcome.context_dependent,
        missing: false,
        percentage: false,
        calculation: true,
        direct_text_pending: false,
    })
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum AlphaRole {
    Origin,
    OrdinarySrgb,
    Retained,
    ExplicitRelativeOverride,
}

fn serialize_alpha(
    value: Option<&CssColorComponent>,
    role: AlphaRole,
    context: &mut SpecifiedSerializationContext,
) -> Result<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if role == AlphaRole::Origin {
        let projected = declared_component_projection(
            value,
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            true,
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
        return Ok(Some(output_text("none", context)));
    }
    match value {
        CssColorComponent::NumberCalculation(_)
        | CssColorComponent::HintedNumberCalculation(_)
        | CssColorComponent::PercentageCalculation(_) => {
            let (calculation, scale) = match value {
                CssColorComponent::NumberCalculation(value) => (
                    crate::numeric::ColorCalculationRef::Number(value),
                    crate::numeric::NumericProjectionScale::Identity,
                ),
                CssColorComponent::HintedNumberCalculation(value) => (
                    crate::numeric::ColorCalculationRef::HintedNumber(value),
                    crate::numeric::NumericProjectionScale::Identity,
                ),
                CssColorComponent::PercentageCalculation(value) => (
                    crate::numeric::ColorCalculationRef::Percentage(value),
                    crate::numeric::NumericProjectionScale::PercentageToNumber {
                        numerator: 1,
                        denominator: 100,
                    },
                ),
                _ => unreachable!("calculated alpha"),
            };
            // Capture owns cumulative graph and scratch checks even when the
            // ordinary scalar will be omitted. Its outcome precedes rounding.
            let (text, outcome) =
                crate::numeric::capture_retained_color_calculation(calculation, scale, context)?;
            if role == AlphaRole::OrdinarySrgb
                && !outcome.context_dependent
                && let Some(scalar) = outcome.scalar_value
            {
                let clamped = if scalar.is_nan() {
                    0.0
                } else {
                    scalar.clamp(0.0, 1.0)
                };
                if clamped == 1.0 {
                    return Ok(None);
                }
                return Ok(Some(ordinary_number(clamped, context)?));
            }
            Ok(Some(text))
        }
        CssColorComponent::Number(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                false,
                role == AlphaRole::ExplicitRelativeOverride,
                context,
            )
        }
        CssColorComponent::Percentage(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                true,
                role == AlphaRole::ExplicitRelativeOverride,
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
    // A checked hex leaf has no children or fallible projection to visit.
    if context.output_suppressed() {
        return Ok(String::new());
    }
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
    let alpha = expanded.get(3).copied().filter(|alpha| *alpha != 255);
    let mut out = LocalCss::new(context);
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
    if let Some(alpha) = alpha {
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
                declared_component_projection(
                    channel,
                    Factor::ONE,
                    Factor::ONE,
                    ComponentTarget::Preserve,
                    true,
                    context,
                )
            })
            .collect::<Result<Vec<_>>>()?;
        let alpha = serialize_alpha(value.alpha(), AlphaRole::Origin, context)?;
        return modern_function("rgb", &channels, alpha.as_deref(), context);
    }

    let alpha = serialize_alpha(value.alpha(), AlphaRole::OrdinarySrgb, context)?;
    let missing = value.channels().iter().any(CssColorComponent::is_none)
        || value.alpha().is_some_and(CssColorComponent::is_none);
    if missing {
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
            finalize_rgb_channel(channel, 1, context)?;
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
    for channel in &mut channels {
        finalize_rgb_channel(channel, 255, context)?;
    }
    if channels.iter().all(|channel| !channel.contextual) {
        let text = channels
            .iter()
            .map(|channel| channel.text.clone())
            .collect::<Vec<_>>();
        return legacy_rgb(&text, alpha.as_deref(), context);
    }
    modern_function("rgb", &channels, alpha.as_deref(), context)
}

/// Capture and its cumulative work must succeed before final slot text is
/// selected. RGB's chosen form owns the reference range; contextual captures
/// retain the producer's dimensions and generic coefficient policy.
fn finalize_rgb_channel(
    value: &mut ProjectedScalar,
    upper: u64,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    if let Some(number) = value.scalar_value {
        let clipped = if number.is_nan() {
            0.0
        } else {
            number.clamp(0.0, upper as f64)
        };
        value.text = ordinary_number(clipped, context)?;
    } else if let Some(exact) = &value.exact {
        value.text = if exact.compare_integer(0, context)?.is_le() {
            output_text("0", context)
        } else if exact.compare_integer(upper, context)?.is_ge() {
            if context.output_suppressed() {
                String::new()
            } else {
                upper.to_string()
            }
        } else if upper == 1 {
            exact.clone_with_budget(context)?.format_exact_or_rounded(
                6,
                context.remaining_bytes(),
                context,
            )?
        } else {
            exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?
        };
    }
    Ok(())
}

fn materialize_direct(
    value: &mut ProjectedScalar,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    if value.direct_text_pending {
        value.text = value
            .exact
            .as_ref()
            .expect("pending direct exact scalar")
            .clone_with_budget(context)?
            .format_exact(context.remaining_bytes(), context)?;
        value.direct_text_pending = false;
    }
    Ok(())
}

fn modern_function(
    name: &str,
    channels: &[ProjectedScalar],
    alpha: Option<&str>,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    let mut out = LocalCss::new(context);
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
    let mut out = LocalCss::new(context);
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

fn normalized_numeric_hue(value: f64) -> f64 {
    if !value.is_finite() {
        return 0.0;
    }
    let hue = value.rem_euclid(360.0);
    // Tiny negative remainders can round to 360 in binary64.
    if hue >= 360.0 { 0.0 } else { hue }
}

fn finalize_hue(
    value: &mut ProjectedScalar,
    context: &SpecifiedSerializationContext,
) -> Result<()> {
    if let Some(number) = value.scalar_value {
        value.text = ordinary_number(normalized_numeric_hue(number), context)?;
        if value.text == "360" {
            value.text = output_text("0", context);
        }
    }
    Ok(())
}

fn percentage_destination(value: &CssColorComponent, missing: bool) -> bool {
    missing
        || matches!(
            value,
            CssColorComponent::Percentage(_) | CssColorComponent::PercentageCalculation(_)
        )
}

fn finalize_hsl_saturation(
    value: &mut ProjectedScalar,
    percentage: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    if let Some(exact) = &value.exact
        && exact.compare_integer(0, context)?.is_lt()
    {
        value.number = Some(ScaledNumber::ZERO);
        value.text = output_text("0", context);
        value.direct_text_pending = false;
    }
    let number = value.scalar_value.map(|number| {
        if number.is_nan() || number < 0.0 {
            0.0
        } else {
            number
        }
    });
    if let Some(number) = number {
        value.number = Some(ScaledNumber::from_binary64(number));
        finalize_hsl_hwb_number(value, number, percentage, context)?;
    }
    Ok(())
}

fn finalize_hsl_hwb_component(
    value: &mut ProjectedScalar,
    percentage: bool,
    context: &SpecifiedSerializationContext,
) -> Result<()> {
    if let Some(number) = value.scalar_value {
        finalize_hsl_hwb_number(value, number, percentage, context)?;
    }
    Ok(())
}

fn finalize_hsl_hwb_number(
    value: &mut ProjectedScalar,
    number: f64,
    percentage: bool,
    context: &SpecifiedSerializationContext,
) -> Result<()> {
    let number = if number.is_nan() {
        // Conversion uses the normalized operand; preserve scalar_value's
        // original projection bits for the independently owned final scalar.
        value.number = Some(ScaledNumber::ZERO);
        0.0
    } else {
        number
    };
    if number.is_finite() {
        value.text = ordinary_number(number, context)?;
        value.percentage = percentage;
    } else {
        let text = match (number.is_sign_negative(), percentage) {
            (false, false) => "calc(infinity)",
            (true, false) => "calc(-infinity)",
            (false, true) => "calc(infinity * 1%)",
            (true, true) => "calc(-infinity * 1%)",
        };
        let mut output = LocalCss::new(context);
        output.push(text)?;
        value.text = output.finish();
        // Exceptional dimensional calculations already contain their unit.
        value.percentage = false;
    }
    Ok(())
}

fn numeric_hue(value: &ProjectedScalar) -> f64 {
    value
        .scalar_value
        .map(normalized_numeric_hue)
        .unwrap_or_else(|| value.number.expect("numeric hue").binary64())
}

fn converted_rgb_text(
    channels: [ScaledNumber; 3],
    context: &SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    let mut text = [String::new(), String::new(), String::new()];
    for (target, channel) in text.iter_mut().zip(channels) {
        *target = ordinary_number(channel.clamp(0.0, 1.0).binary64() * 255.0, context)?;
    }
    Ok(text)
}

fn serialize_hsl(
    value: &CssHslColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    if mode == Mode::Origin {
        let hue = hue_projection(value.hue(), true, context)?;
        let first = declared_component_projection(
            value.saturation(),
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            true,
            context,
        )?;
        let second = declared_component_projection(
            value.lightness(),
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            true,
            context,
        )?;
        let alpha = serialize_alpha(value.alpha(), AlphaRole::Origin, context)?;
        return hsl_like("hsl", &hue, &first, &second, alpha.as_deref(), context);
    }
    let missing = matches!(value.hue(), CssColorHue::None)
        || value.saturation().is_none()
        || value.lightness().is_none()
        || value.alpha().is_some_and(CssColorComponent::is_none);
    let target = if missing {
        ComponentTarget::Percentage
    } else {
        ComponentTarget::Preserve
    };
    let mut hue = hue_projection(value.hue(), false, context)?;
    let materialize = missing;
    let mut saturation = component_projection_with_text(
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
    let alpha = serialize_alpha(value.alpha(), AlphaRole::OrdinarySrgb, context)?;
    finalize_hue(&mut hue, context)?;
    finalize_hsl_saturation(
        &mut saturation,
        percentage_destination(value.saturation(), missing),
        context,
    )?;
    finalize_hsl_hwb_component(
        &mut lightness,
        percentage_destination(value.lightness(), missing),
        context,
    )?;
    if missing {
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
    let hue = numeric_hue(&hue);
    let saturation = saturation
        .number
        .expect("numeric saturation")
        .max_zero()
        .scale(0.01);
    let lightness = lightness.number.expect("numeric lightness").scale(0.01);
    let rgb = hsl_to_rgb_scaled(hue, saturation, lightness);
    let channels = converted_rgb_text(rgb, context)?;
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
                return Ok(channels.map(|channel| output_text(channel, context)));
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
    if mode == Mode::Origin {
        let hue = hue_projection(value.hue(), true, context)?;
        let first = declared_component_projection(
            value.whiteness(),
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            true,
            context,
        )?;
        let second = declared_component_projection(
            value.blackness(),
            Factor::ONE,
            Factor::ONE,
            ComponentTarget::Preserve,
            true,
            context,
        )?;
        let alpha = serialize_alpha(value.alpha(), AlphaRole::Origin, context)?;
        return hsl_like("hwb", &hue, &first, &second, alpha.as_deref(), context);
    }
    let missing = matches!(value.hue(), CssColorHue::None)
        || value.whiteness().is_none()
        || value.blackness().is_none()
        || value.alpha().is_some_and(CssColorComponent::is_none);
    let target = if missing {
        ComponentTarget::Percentage
    } else {
        ComponentTarget::Preserve
    };
    let mut hue = hue_projection(value.hue(), false, context)?;
    let materialize = missing;
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
    let alpha = serialize_alpha(value.alpha(), AlphaRole::OrdinarySrgb, context)?;
    finalize_hue(&mut hue, context)?;
    finalize_hsl_hwb_component(
        &mut white,
        percentage_destination(value.whiteness(), missing),
        context,
    )?;
    finalize_hsl_hwb_component(
        &mut black,
        percentage_destination(value.blackness(), missing),
        context,
    )?;
    if missing {
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
    let hue = numeric_hue(&hue);
    let white = white.number.expect("numeric whiteness").scale(0.01);
    let black = black.number.expect("numeric blackness").scale(0.01);
    let rgb = hwb_to_rgb_scaled(hue, white, black);
    let channels = converted_rgb_text(rgb, context)?;
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
            return Ok(std::array::from_fn(|_| output_text("0", context)));
        }
        if white.compare(&sum, context)?.is_ge() {
            return Ok(std::array::from_fn(|_| output_text("255", context)));
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
        scalar_value: value.scalar_value,
        text: value.text.clone(),
        number: value.number,
        exact: None,
        contextual: value.contextual,
        missing: value.missing,
        percentage: value.percentage,
        calculation: value.calculation,
        direct_text_pending: value.direct_text_pending,
    }
}

fn hue_projection(
    value: &CssColorHue,
    origin: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    use CssColorHue as H;
    if origin {
        let literal = match value {
            H::Number(value) => Some((value.numeric().representation(), Factor::ONE, "")),
            H::Angle(value) => Some((
                value.numeric().representation(),
                angle_factor(value.unit()),
                "deg",
            )),
            _ => None,
        };
        if let Some((source, factor, suffix)) = literal {
            context.charge_input(1)?;
            let text = generic_literal_text(source, factor, context)?;
            return Ok(ProjectedScalar {
                scalar_value: None,
                text: suffix_text(text, suffix, context)?,
                number: None,
                exact: None,
                contextual: false,
                missing: false,
                percentage: false,
                calculation: false,
                direct_text_pending: false,
            });
        }
    }

    let (text, scalar_value, number, exact, contextual, missing, calculation) = match value {
        H::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            (
                output_text("none", context),
                None,
                None,
                None,
                false,
                true,
                false,
            )
        }
        H::Number(value) => {
            context.charge_input(1)?;
            let source = value.numeric().representation();
            let exact = crate::exact_decimal::ExactRational::from_lexical_factor_modulo(
                source,
                exact_factor(Factor::ONE),
                360,
                context,
            )?;
            let number = ScaledNumber::from_exact(&exact)?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, None, Some(number), Some(exact), false, false, false)
        }
        H::Angle(value) => {
            context.charge_input(1)?;
            let factor = angle_factor(value.unit());
            let source = value.numeric().representation();
            let exact = crate::exact_decimal::ExactRational::from_lexical_factor_modulo(
                source,
                exact_factor(factor),
                360,
                context,
            )?;
            let number = ScaledNumber::from_exact(&exact)?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, None, Some(number), Some(exact), false, false, false)
        }
        H::NumberCalculation(value) => {
            let calculation = crate::numeric::ColorCalculationRef::Number(value);
            let (text, outcome) = if origin {
                crate::numeric::capture_retained_color_calculation(
                    calculation,
                    crate::numeric::NumericProjectionScale::Identity,
                    context,
                )?
            } else {
                crate::numeric::capture_color_component_calculation(
                    calculation,
                    crate::numeric::NumericProjectionScale::Identity,
                    context,
                )?
            };
            (
                text,
                outcome.scalar_value,
                outcome.scalar_value.map(ScaledNumber::from_binary64),
                None,
                outcome.context_dependent,
                false,
                true,
            )
        }
        H::AngleCalculation(value) => {
            let calculation = crate::numeric::ColorCalculationRef::Angle(value);
            let (text, outcome) = if origin {
                crate::numeric::capture_retained_color_calculation(
                    calculation,
                    crate::numeric::NumericProjectionScale::Identity,
                    context,
                )?
            } else {
                crate::numeric::capture_color_component_calculation(
                    calculation,
                    crate::numeric::NumericProjectionScale::Identity,
                    context,
                )?
            };
            (
                text,
                outcome.scalar_value,
                outcome.scalar_value.map(ScaledNumber::from_binary64),
                None,
                outcome.context_dependent,
                false,
                true,
            )
        }
    };
    Ok(ProjectedScalar {
        scalar_value,
        text,
        number,
        exact,
        contextual,
        missing,
        percentage: false,
        calculation,
        direct_text_pending: false,
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
    let lightness = lab_component_projection(
        value.lightness(),
        light_percentage,
        if name == "lab" {
            LabComponentRange::LabLightness
        } else {
            LabComponentRange::OklabLightness
        },
        mode,
        context,
    )?;
    let channels = [
        lightness,
        declared_component_projection(
            value.a(),
            Factor::ONE,
            axis_percentage,
            target,
            origin,
            context,
        )?,
        declared_component_projection(
            value.b(),
            Factor::ONE,
            axis_percentage,
            target,
            origin,
            context,
        )?,
    ];
    let alpha = serialize_alpha(
        value.alpha(),
        if origin {
            AlphaRole::Origin
        } else {
            AlphaRole::Retained
        },
        context,
    )?;
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
    let lightness = lab_component_projection(
        value.lightness(),
        light_percentage,
        if name == "lch" {
            LabComponentRange::LabLightness
        } else {
            LabComponentRange::OklabLightness
        },
        mode,
        context,
    )?;
    let channels = [
        lightness,
        lab_component_projection(
            value.chroma(),
            chroma_percentage,
            LabComponentRange::Chroma,
            mode,
            context,
        )?,
        hue_projection(value.hue(), origin, context)?,
    ];
    let alpha = serialize_alpha(
        value.alpha(),
        if origin {
            AlphaRole::Origin
        } else {
            AlphaRole::Retained
        },
        context,
    )?;
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
            declared_component_projection(
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
                origin,
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let alpha = serialize_alpha(
        value.alpha(),
        if origin {
            AlphaRole::Origin
        } else {
            AlphaRole::Retained
        },
        context,
    )?;
    if context.output_suppressed() {
        return Ok(String::new());
    }
    let name = format!("color({}", predefined_name(value.color_space()));
    modern_function(&name, &channels, alpha.as_deref(), context)
}

fn serialize_device_cmyk(
    value: &CssDeviceCmykColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let channels = value
        .channels()
        .iter()
        .map(|channel| {
            declared_component_projection(
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
                origin,
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let alpha = serialize_alpha(
        value.alpha(),
        if origin {
            AlphaRole::Origin
        } else {
            AlphaRole::Retained
        },
        context,
    )?;
    modern_function("device-cmyk", &channels, alpha.as_deref(), context)
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
            declared_component_projection(
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
                origin,
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    let alpha = serialize_alpha(
        value.alpha(),
        if origin {
            AlphaRole::Origin
        } else {
            AlphaRole::Retained
        },
        context,
    )?;
    let profile = escaped_identifier(value.profile().as_str(), context)?;
    let mut name = LocalCss::new(context);
    name.push("color(")?;
    name.push(&profile)?;
    let name = name.finish();
    modern_function(&name, &channels, alpha.as_deref(), context)
}

fn serialize_relative_expression(
    value: &CssRelativeColorExpression,
    alpha: bool,
    percentage_factor: Factor,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    use CssRelativeColorExpressionValue as V;
    match value.value() {
        V::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(output_text("none", context))
        }
        V::Channel(channel) => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(output_text(relative_channel(*channel), context))
        }
        V::Number(value) => {
            context.charge_input(1)?;
            if alpha {
                Ok(
                    alpha_literal(value.numeric().representation(), false, true, context)?
                        .expect("explicit relative alpha retained"),
                )
            } else {
                generic_literal_text(value.numeric().representation(), Factor::ONE, context)
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
                generic_literal_text(value.numeric().representation(), percentage_factor, context)
            }
        }
        V::Angle(value) => {
            context.charge_input(1)?;
            let text = generic_literal_text(
                value.numeric().representation(),
                angle_factor(value.unit()),
                context,
            )?;
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
            let (text, _) =
                crate::numeric::capture_specified_scaled(&value.data.expression, scale, context)?;
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
                serialize_alpha(Some(value), AlphaRole::ExplicitRelativeOverride, context)
                    .map(|value| value.expect("explicit custom relative alpha retained"))
            } else {
                match value {
                    CssColorComponent::Number(value) => {
                        context.charge_input(1)?;
                        generic_literal_text(value.numeric().representation(), Factor::ONE, context)
                    }
                    CssColorComponent::Percentage(value) => {
                        context.charge_input(1)?;
                        generic_literal_text(
                            value.numeric().representation(),
                            Factor {
                                numerator: 1,
                                denominator: 100,
                            },
                            context,
                        )
                    }
                    _ => component_projection(
                        value,
                        Factor::ONE,
                        Factor {
                            numerator: 1,
                            denominator: 100,
                        },
                        ComponentTarget::Number,
                        context,
                    )
                    .map(|value| value.text),
                }
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
            let (text, _) = value.capture_specified(scale, context)?;
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

pub(crate) fn serialize_interpolation(
    value: &CssColorInterpolation,
    context: &SpecifiedSerializationContext,
) -> Result<String> {
    // The owning mix has charged this checked interpolation node already.
    if context.output_suppressed() {
        return Ok(String::new());
    }
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
    if let Some(hue) = value
        .hue()
        .filter(|hue| *hue != CssHueInterpolationMethod::Shorter)
    {
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

pub(crate) fn mix_weight_texts<'a>(
    weights: impl ExactSizeIterator<Item = Option<&'a CssColorMixWeight>> + Clone,
    context: &mut SpecifiedSerializationContext,
) -> Result<Vec<Option<String>>> {
    let count = weights.len();
    if weights
        .clone()
        .any(|weight| weight.is_some_and(|weight| weight.calculation().is_some()))
    {
        let mut output = Vec::new();
        output.try_reserve(count).map_err(|_| {
            Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
        })?;
        for weight in weights {
            output.push(match weight {
                None => None,
                Some(weight) if weight.literal_value().is_some() => {
                    let text = literal_weight(weight.literal_value().unwrap(), context)?;
                    Some(suffix_text(text, "%", context)?)
                }
                Some(weight) => {
                    let calculation = weight.calculation().expect("checked weight variant");
                    let (text, _) = crate::numeric::capture_retained_color_calculation(
                        crate::numeric::ColorCalculationRef::Percentage(calculation),
                        crate::numeric::NumericProjectionScale::Identity,
                        context,
                    )?;
                    Some(text)
                }
            });
        }
        return Ok(output);
    }

    let EffectiveMixWeights { exact, generated } = effective_mix_weights(weights, context)?;
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
        output.try_reserve(count).map_err(|_| {
            Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
        })?;
        output.resize_with(count, || None);
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

/// Exact declared shares before serialization rounding. Explicit shares remain
/// unchanged; only omitted shares use the capped literal sum. The caller must
/// select this path only when no weight is a retained calculation.
pub(super) struct EffectiveMixWeights {
    exact: Vec<Option<crate::exact_decimal::ExactRational>>,
    generated: Option<crate::exact_decimal::ExactRational>,
}
impl EffectiveMixWeights {
    pub(super) fn value(&self, index: usize) -> &crate::exact_decimal::ExactRational {
        self.exact[index]
            .as_ref()
            .or(self.generated.as_ref())
            .expect("every mix slot has an effective weight")
    }
}
pub(super) fn effective_mix_weights<'a>(
    weights: impl ExactSizeIterator<Item = Option<&'a CssColorMixWeight>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<EffectiveMixWeights> {
    let count = weights.len();
    let mut exact = Vec::new();
    exact.try_reserve(count).map_err(|_| {
        Error::new(crate::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    })?;
    let mut sum: Option<crate::exact_decimal::ExactRational> = None;
    let mut omitted = 0usize;
    for weight in weights {
        let value = weight
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
    Ok(EffectiveMixWeights { exact, generated })
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

#[cfg(test)]
mod suppressed_color_output_tests {
    use super::*;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};

    fn assert_suppressed_color(source: &str) {
        let declaration = crate::parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            crate::parse_component_values(source).unwrap(),
            crate::CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("checked color property");
        };
        let value = value.value();
        let original = value.clone();
        let mut writer = SpecifiedRuleWriter::new(Limits::new(1_000_000, 1_000_000, 7));
        writer.append("prefix:").unwrap();
        assert_eq!(writer.context.remaining_bytes(), 0);
        let result = writer
            .without_output(|writer| value.append_specified(&mut writer.context, &mut writer.css));
        assert_eq!(writer.css, "prefix:");
        assert!(!writer.context.output_suppressed());
        assert_eq!(value, &original);
        result.unwrap();
    }

    #[test]
    fn suppressed_hex_does_not_format_output_with_an_exhausted_byte_budget() {
        assert_suppressed_color("#123456");
    }

    #[test]
    fn suppressed_rgb_does_not_format_output_with_an_exhausted_byte_budget() {
        assert_suppressed_color("rgb(20 40 60 / .5)");
    }

    #[test]
    fn suppressed_unequal_mix_weights_do_not_format_output_with_an_exhausted_byte_budget() {
        assert_suppressed_color("color-mix(in oklab, red 20%, blue 80%)");
    }

    #[test]
    fn suppressed_custom_mix_profile_does_not_escape_output_with_an_exhausted_byte_budget() {
        assert_suppressed_color("color-mix(in --Profile, red, blue)");
    }

    #[test]
    fn suppressed_custom_profile_calculation_needs_no_identifier_output_bytes() {
        assert_suppressed_color("color(from red --P calc(Cyan / 3))");
    }

    #[test]
    fn suppressed_keyword_color_preserves_the_enclosing_buffer() {
        assert_suppressed_color("currentcolor");
    }

    #[test]
    fn suppressed_equal_weight_mix_preserves_the_enclosing_buffer() {
        assert_suppressed_color("color-mix(in oklab, red, blue)");
    }
}

#[cfg(test)]
mod suppressed_color_semantics_tests {
    use super::*;
    use crate::CssSpecifiedValueSerializationErrorKind as K;
    use crate::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};

    fn color(source: &str) -> CssColor {
        let declaration = crate::parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            crate::parse_component_values(source).unwrap(),
            crate::CssImportance::Important,
        )
        .unwrap_or_else(|error| panic!("{source}: {error:?}"));
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("checked color");
        };
        value.value().clone()
    }

    fn append(value: &CssColor, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        value.append_specified(&mut writer.context, &mut writer.css)
    }

    #[test]
    fn every_authored_color_family_visits_without_output_bytes() {
        for source in [
            "currentcolor",
            "transparent",
            "Purple",
            "CanvasText",
            "#1234",
            "rgb(20 40 60 / .5)",
            "rgb(none calc(50%) 0)",
            "hsl(1 50% 50%)",
            "hsl(none 50% 50%)",
            "hwb(0 20% 80%)",
            "hwb(0 calc(1em / 1px) 20%)",
            "hsl(calc(1em / 1px) 50% 50%)",
            "hsl(calc(1em / 1px) -20% 50%)",
            "lab(100% 100% -100% / .5)",
            "lch(50% 30% 1rad)",
            "oklab(100% 100% -100%)",
            "oklch(100% 100% 0)",
            "color(display-p3 calc(50%) 0 .2)",
            "color(--P 0% 70% 20% 0%)",
            "device-cmyk(0 .2 .4 .6 / .5)",
            "alpha(from hsl(1turn 50% 50%) / calc(50%))",
            "rgb(from rgb(1 2 3 / .2) calc(r / 3) g b / 1)",
            "color(from red --P calc(Cyan / 3) / alpha)",
            "color-mix(in --Profile, red 20%, blue 80%)",
            "color-mix(red calc(20%), blue)",
            "light-dark(alpha(from red), contrast-color(blue))",
        ] {
            let value = color(source);
            let original = value.clone();
            for prefix in ["", "prefix:"] {
                let mut writer =
                    SpecifiedRuleWriter::new(Limits::new(100_000, 100_000, prefix.len()));
                writer.append(prefix).unwrap();
                writer
                    .without_output(|writer| append(&value, writer))
                    .unwrap_or_else(|error| panic!("{source}: {error:?}"));
                assert_eq!(writer.css, prefix, "{source}");
                assert_eq!(writer.context.remaining_bytes(), 0);
                assert!(!writer.context.output_suppressed());
                assert_eq!(value, original, "{source}");
            }
        }
    }

    #[test]
    fn suppressed_children_and_emitted_siblings_share_exact_work_limits() {
        let nested = color("light-dark(currentcolor, transparent)");
        let sibling = color("currentcolor");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(4, 4, 13));
        writer.append("x").unwrap();
        writer
            .without_output(|writer| append(&nested, writer))
            .unwrap();
        assert_eq!(writer.context.remaining_bytes(), 12);
        append(&sibling, &mut writer).unwrap();
        assert_eq!(writer.css, "xcurrentcolor");
        assert_eq!(writer.context.remaining_bytes(), 0);
        assert_eq!(
            writer.context.charge_input(1).unwrap_err().kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            writer.context.charge_projection(1).unwrap_err().kind(),
            K::ProjectionNodeLimit
        );
    }

    #[test]
    fn contextual_slots_have_identical_semantic_budget_boundaries_in_both_modes() {
        for source in [
            "hsl(0 100% 50%)",
            "hsl(60 100% 50%)",
            "hsl(120 100% 50%)",
            "hsl(180 100% 50%)",
            "hsl(240 100% 50%)",
            "hsl(300 100% 50%)",
            "hwb(0 calc(1em / 1px) 20%)",
            "hsl(calc(1em / 1px) -20% 50%)",
            "hsl(calc(1em / 1px) 50% 50%)",
            "color(from red --P calc(Cyan / 3))",
            "color-mix(in --Profile, red 20%, blue 80%)",
        ] {
            let value = color(source);
            for input_is_bounded in [true, false] {
                let mut reached_success = false;
                for limit in 0..400 {
                    let (input, projection) = if input_is_bounded {
                        (limit, 10_000)
                    } else {
                        (10_000, limit)
                    };
                    let mut emitted =
                        SpecifiedRuleWriter::new(Limits::new(input, projection, 1_000));
                    let ordinary = append(&value, &mut emitted).map_err(|error| error.kind());
                    let mut suppressed =
                        SpecifiedRuleWriter::new(Limits::new(input, projection, 0));
                    let discarded = suppressed
                        .without_output(|writer| append(&value, writer))
                        .map_err(|error| error.kind());
                    assert_eq!(
                        ordinary, discarded,
                        "{source}, input bounded {input_is_bounded}, limit {limit}"
                    );
                    assert!(suppressed.css.is_empty());
                    assert!(!suppressed.context.output_suppressed());
                    if ordinary.is_ok() {
                        // The shared threshold must include all successful
                        // visits. A following node cannot consume a refund.
                        let ordinary_next = if input_is_bounded {
                            emitted.context.charge_input(1)
                        } else {
                            emitted.context.charge_projection(1)
                        }
                        .map_err(|error| error.kind());
                        let discarded_next = if input_is_bounded {
                            suppressed.context.charge_input(1)
                        } else {
                            suppressed.context.charge_projection(1)
                        }
                        .map_err(|error| error.kind());
                        assert_eq!(ordinary_next, discarded_next);
                        assert!(ordinary_next.is_err(), "first successful limit is exact");
                        reached_success = true;
                        break;
                    }
                }
                assert!(reached_success, "{source}: bounded semantic witness");
            }
        }
    }

    #[test]
    fn exact_hsl_vertices_never_materialize_discarded_channel_text() {
        use crate::exact_decimal::ExactRational;

        for (angle, expected) in [
            ("0", ["255", "0", "0"]),
            ("60", ["255", "255", "0"]),
            ("120", ["0", "255", "0"]),
            ("180", ["0", "255", "255"]),
            ("240", ["0", "0", "255"]),
            ("300", ["255", "0", "255"]),
        ] {
            for suppressed in [false, true] {
                let mut context = SpecifiedSerializationContext::new(Limits::new(
                    0,
                    1_000,
                    if suppressed { 0 } else { 100 },
                ));
                context.replace_output_suppression(suppressed);
                let hue = ExactRational::from_lexical_factor(
                    angle,
                    exact_factor(Factor::ONE),
                    &mut context,
                )
                .unwrap();
                let saturation = ExactRational::from_lexical_factor(
                    "100",
                    exact_factor(Factor::ONE),
                    &mut context,
                )
                .unwrap();
                let lightness = ExactRational::from_lexical_factor(
                    "50",
                    exact_factor(Factor::ONE),
                    &mut context,
                )
                .unwrap();
                let channels = exact_hsl_text(&hue, &saturation, &lightness, &mut context).unwrap();
                if suppressed {
                    assert_eq!(channels, [String::new(), String::new(), String::new()]);
                } else {
                    assert_eq!(channels, expected);
                }
                assert_eq!(context.output_suppressed(), suppressed);
            }
        }
    }

    #[test]
    fn nested_suppression_restores_both_modes_after_semantic_failure() {
        let value = color("light-dark(currentcolor, transparent)");
        for (input, projection, expected) in [
            (2, 100, K::InputNodeLimit),
            (100, 2, K::ProjectionNodeLimit),
        ] {
            let mut writer = SpecifiedRuleWriter::new(Limits::new(input, projection, 2));
            writer.append("x").unwrap();
            let error = writer
                .without_output(|writer| {
                    let error = writer
                        .without_output(|writer| append(&value, writer))
                        .unwrap_err();
                    assert!(writer.context.output_suppressed());
                    Err::<(), _>(error)
                })
                .unwrap_err();
            assert_eq!(error.kind(), expected);
            assert!(!writer.context.output_suppressed());
            writer.append("y").unwrap();
            assert_eq!(writer.css, "xy");
        }
    }

    #[test]
    fn suppression_preserves_checked_exact_arithmetic_errors() {
        let value =
            color("hsl(0 12345678901234567891e170141183460469231731687303715884105727% 50%)");
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100_000, 100_000, 0));
        let error = writer
            .without_output(|writer| append(&value, writer))
            .unwrap_err();
        assert_eq!(error.kind(), K::CapacityOverflow);
        assert!(writer.css.is_empty());
        assert!(!writer.context.output_suppressed());
    }

    #[test]
    fn explicit_alpha_and_profile_metadata_survive_discarded_output() {
        let source = "/* origin */color(from rgb(1 2 3 / .2) --P Cyan / alpha)";
        let value = color(source);
        let original = value.clone();
        let expected = "color(from rgb(1 2 3 / 0.2) --P Cyan / alpha)";
        let mut writer = SpecifiedRuleWriter::new(Limits::new(100_000, 100_000, expected.len()));
        writer
            .without_output(|writer| append(&value, writer))
            .unwrap();
        assert_eq!(value, original);
        append(&value, &mut writer).unwrap();
        assert_eq!(writer.css, expected);
    }
}

#[cfg(test)]
mod declared_literal_tests {
    use super::*;
    use crate::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};

    fn checked_color(source: &str) -> CssColor {
        let declaration = crate::parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            crate::parse_component_values(source).unwrap(),
            crate::CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::Color(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("checked color property");
        };
        value.value().clone()
    }

    fn assert_exact_bytes(source: &str, expected: &str) {
        let value = checked_color(source);
        let original = value.clone();
        assert_eq!(
            serialize(&value, Limits::new(usize::MAX, usize::MAX, expected.len())).unwrap(),
            expected
        );
        assert_eq!(
            serialize(
                &value,
                Limits::new(usize::MAX, usize::MAX, expected.len() - 1)
            )
            .unwrap_err()
            .kind(),
            crate::CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(value, original);
    }

    #[test]
    fn converted_angle_guard_handles_tail_carry_in_origins_and_relative_hues() {
        for (source, expected) in [
            (
                "alpha(from hsl(1.00000199grad 50% 50%))",
                "alpha(from hsl(0.900002deg 50% 50%))",
            ),
            (
                "alpha(from hsl(-1.00000199grad 50% 50%))",
                "alpha(from hsl(-0.900002deg 50% 50%))",
            ),
            (
                "hsl(from red 1.00000199rad s l)",
                "hsl(from red 57.295894deg s l)",
            ),
            (
                "hsl(from red -1.00000199rad s l)",
                "hsl(from red -57.295894deg s l)",
            ),
        ] {
            assert_exact_bytes(source, expected);
        }
    }

    #[test]
    fn rational_relative_midpoint_keeps_long_authored_tails_and_signs() {
        for (coefficient, expected) in [
            (
                "0.00000019607843137254901960784313725490196078431372549019",
                "0",
            ),
            (
                "0.00000019607843137254901960784313725490196078431372549020",
                "0.000001",
            ),
            (
                "-0.00000019607843137254901960784313725490196078431372549019",
                "0",
            ),
            (
                "-0.00000019607843137254901960784313725490196078431372549020",
                "-0.000001",
            ),
        ] {
            assert_exact_bytes(
                &format!("rgb(from red {coefficient}% g b)"),
                &format!("rgb(from red {expected} g b)"),
            );
        }
        let coefficient = format!("0.1234567{}", "1".repeat(100_000));
        assert_exact_bytes(
            &format!("rgb(from red {coefficient}% g b)"),
            "rgb(from red 0.314815 g b)",
        );
    }

    #[test]
    fn literal_conversion_work_shares_the_public_cumulative_projection_budget() {
        // Two roots and g/b references add four projections. Identity uses one
        // lexical emission; 20% uses scalar + one prefix chunk + division;
        // .000000199% additionally compares two borrowed tail digits.
        for (source, expected, projections) in [
            (
                "rgb(from red .1234567 g b)",
                "rgb(from red 0.123457 g b)",
                5,
            ),
            ("rgb(from red 20% g b)", "rgb(from red 51 g b)", 7),
            (
                "rgb(from red .000000199% g b)",
                "rgb(from red 0.000001 g b)",
                9,
            ),
        ] {
            let value = checked_color(source);
            let original = value.clone();
            assert_eq!(
                serialize(&value, Limits::new(5, projections, expected.len())).unwrap(),
                expected
            );
            assert_eq!(
                serialize(&value, Limits::new(5, projections - 1, expected.len()))
                    .unwrap_err()
                    .kind(),
                crate::CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            assert_eq!(
                serialize(&value, Limits::new(4, projections, expected.len()))
                    .unwrap_err()
                    .kind(),
                crate::CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            );
            assert_eq!(value, original);
        }
    }
}
