//! Canonical specified-color serialization over checked authored graphs.

use crate::color::syntax::CssColorRepresentation;
use crate::color::{
    CssAlphaColor, CssColor, CssColorComponent, CssColorHue, CssColorInterpolation,
    CssColorInterpolationMethod, CssColorMix, CssColorMixComponent, CssColorMixPercentage,
    CssColorMixWeight, CssCustomColor, CssDeviceCmykColor, CssHexColor, CssHslColor, CssHwbColor,
    CssLabColor, CssLchColor, CssPredefinedColor, CssRelativeColor, CssRelativeColorChannel,
    CssRelativeColorExpression, CssRelativeColorExpressionValue, CssRelativeColorFunction,
    CssRelativeCustomColor, CssRgbColor, CssSystemColor,
};
use crate::color::{channels, keywords::ColorFunction};
use crate::numeric::CssAngleUnit;
use crate::numeric::CssCalculationType;
use crate::serialization::CssSerializationError as Error;
use crate::serialization::specified_serialization::SpecifiedSerializationContext;
use surgeist_color::{ColorSpace, HueInterpolation, PredefinedColorSpace};

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
                    context.charge_nodes(1)?;
                    schedule_authored(color, mode, context, &mut work)?;
                }
            }
        }
        Ok(())
    }
}

fn reserve_work(work: &mut Vec<Work<'_>>, additional: usize) -> Result<()> {
    work.try_reserve(additional)
        .map_err(|_| Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow))
}

fn work_slots(base: usize, per_item: usize, items: usize) -> Result<usize> {
    per_item
        .checked_mul(items)
        .and_then(|count| base.checked_add(count))
        .ok_or_else(|| Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow))
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
    let (name, space) = relative_function(value.function())?;
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
        context.charge_nodes(1)?;
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
    value
        .predefined()
        .is_some_and(|value| value.space() == ColorSpace::Oklab && value.hue().is_none())
}

fn escaped_identifier(value: &str, context: &SpecifiedSerializationContext) -> Result<String> {
    if context.output_suppressed() {
        return Ok(String::new());
    }
    crate::serialization::serialization_escaping::capture_identifier(value, context)
}

fn ordinary_number(value: f64, context: &SpecifiedSerializationContext) -> Result<String> {
    if context.output_suppressed() {
        return Ok(String::new());
    }
    crate::serialization::numeric_formatting::format_ordinary_color_number(
        value,
        context.remaining_bytes(),
    )
}

fn output_text(value: &str, context: &SpecifiedSerializationContext) -> String {
    if context.output_suppressed() {
        String::new()
    } else {
        value.into()
    }
}

fn relative_function(
    value: &CssRelativeColorFunction,
) -> Result<(&'static str, Option<&'static str>)> {
    Ok(match value {
        CssRelativeColorFunction::Rgb => ("rgb", None),
        CssRelativeColorFunction::Hsl => ("hsl", None),
        CssRelativeColorFunction::Hwb => ("hwb", None),
        CssRelativeColorFunction::Lab => ("lab", None),
        CssRelativeColorFunction::Lch => ("lch", None),
        CssRelativeColorFunction::Oklab => ("oklab", None),
        CssRelativeColorFunction::Oklch => ("oklch", None),
        CssRelativeColorFunction::Color(space) => ("color", Some(predefined_name(*space)?)),
    })
}

/// A space surgeist-color adds later has no CSS name yet, so it is
/// unrepresentable rather than a bug.
fn unknown_color_edition() -> Error {
    Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::UnrepresentableValue)
}

fn predefined_name(value: PredefinedColorSpace) -> Result<&'static str> {
    Ok(match value {
        PredefinedColorSpace::Srgb => "srgb",
        PredefinedColorSpace::SrgbLinear => "srgb-linear",
        PredefinedColorSpace::DisplayP3 => "display-p3",
        PredefinedColorSpace::DisplayP3Linear => "display-p3-linear",
        PredefinedColorSpace::A98Rgb => "a98-rgb",
        PredefinedColorSpace::ProphotoRgb => "prophoto-rgb",
        PredefinedColorSpace::Rec2020 => "rec2020",
        PredefinedColorSpace::XyzD50 => "xyz-d50",
        PredefinedColorSpace::XyzD65 => "xyz-d65",
        _ => return Err(unknown_color_edition()),
    })
}

pub(crate) fn authored_system_name(value: CssSystemColor) -> &'static str {
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
        use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as K;
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
    // Original post-scale outcome, before capture text loses precision.
    // None belongs to direct exact literals or contextual captures.
    scalar_value: Option<f64>,
    exact: Option<crate::serialization::exact_decimal::ExactRational>,
    contextual: bool,
    missing: bool,
    percentage: bool,
    calculation: bool,
    // A direct exact slot can defer its logical materialization until a
    // contextual HSL/HWB branch selects it. Emitted text is not phase state.
    direct_text_pending: bool,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ComponentTarget {
    Preserve,
    Number,
    Percentage,
}

fn exact_factor(value: Factor) -> crate::serialization::exact_decimal::ExactFactor {
    crate::serialization::exact_decimal::ExactFactor {
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
        crate::serialization::numeric_formatting::format_css_number(
            representation,
            shift,
            context.remaining_bytes(),
        )
    } else {
        crate::serialization::exact_decimal::ExactRational::format_generic_number(
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
    use CssRelativeColorFunction as R;
    percent_factor(
        match function {
            R::Rgb => ColorFunction::Rgb,
            R::Hsl => ColorFunction::Hsl,
            R::Hwb => ColorFunction::Hwb,
            R::Lab => ColorFunction::Lab,
            R::Lch => ColorFunction::Lch,
            R::Oklab => ColorFunction::Oklab,
            R::Oklch => ColorFunction::Oklch,
            R::Color(_) => ColorFunction::Color,
        },
        index,
    )
}

/// The declared value of 1% in one channel; a hue slot has no percentage.
fn percent_factor(function: ColorFunction, index: usize) -> Factor {
    match channels::slot(function, index) {
        channels::Slot::Scalar { percent, .. } => Factor {
            numerator: percent.numerator,
            denominator: percent.denominator,
        },
        channels::Slot::Hue => Factor::ONE,
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
                exact: None,
                contextual: false,
                missing: false,
                percentage,
                calculation: false,
                direct_text_pending: false,
            });
        }
        let calculation = match value {
            CssColorComponent::NumberCalculation(value) => Some(
                crate::numeric::expression::ColorCalculationRef::Number(value),
            ),
            CssColorComponent::HintedNumberCalculation(value) => {
                Some(crate::numeric::expression::ColorCalculationRef::HintedNumber(value))
            }
            CssColorComponent::PercentageCalculation(value) => Some(
                crate::numeric::expression::ColorCalculationRef::Percentage(value),
            ),
            _ => None,
        };
        if let Some(calculation) = calculation {
            let (text, outcome) = crate::numeric::expression::capture_retained_color_calculation(
                calculation,
                crate::numeric::projection::NumericProjectionScale::Identity,
                context,
            )?;
            return Ok(ProjectedScalar {
                scalar_value: outcome.scalar_value,
                text,
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

/// Parsed-domain bounds apply to standalone and ordinary Mix direct L/C
/// literals. Origin roles and calculation trees retain their projection and scratch.
/// The declared slot supplies the clamp: lightness to `[0, 100%]`, chroma to
/// nonnegative values.
fn lab_component_projection(
    value: &CssColorComponent,
    percentage_factor: Factor,
    slot: channels::Slot,
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
    let lexical = crate::serialization::exact_decimal::LexicalDecimal::new(representation);
    let endpoint = if lexical.is_zero() || lexical.negative {
        Some("0")
    } else {
        let upper = match slot {
            channels::Slot::Scalar {
                percent,
                clamp: channels::Clamp::Reference,
            } => {
                // 100% denotes the reference value 10^exponent.
                let exponent = percent
                    .reference_power_of_ten()
                    .expect("Lab-family lightness reference is a power of ten");
                let text = ["1", "10", "100"][exponent as usize];
                Some((if percentage { 2 } else { exponent }, text))
            }
            _ => None,
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
    let exact = crate::serialization::exact_decimal::ExactRational::from_lexical_factor(
        representation,
        exact_factor(factor),
        context,
    )?;
    let text = if materialize_direct {
        exact
            .clone_with_budget(context)?
            .format_exact(context.remaining_bytes(), context)?
    } else {
        String::new()
    };
    Ok(ProjectedScalar {
        scalar_value: None,
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
            context.charge_nodes(1)?;
            Ok(ProjectedScalar {
                scalar_value: None,
                text: output_text("none", context),
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
            crate::numeric::expression::ColorCalculationRef::Number(value),
            number_factor,
            target,
            context,
        ),
        C::HintedNumberCalculation(value) => number_calculation_projection(
            crate::numeric::expression::ColorCalculationRef::HintedNumber(value),
            number_factor,
            target,
            context,
        ),
        C::PercentageCalculation(value) => {
            let scale = if target == ComponentTarget::Number {
                crate::numeric::projection::NumericProjectionScale::PercentageToNumber {
                    numerator: percentage_factor.numerator,
                    denominator: percentage_factor.denominator,
                }
            } else {
                crate::numeric::projection::NumericProjectionScale::Identity
            };
            let (text, outcome) = crate::numeric::expression::capture_color_component_calculation(
                crate::numeric::expression::ColorCalculationRef::Percentage(value),
                scale,
                context,
            )?;
            Ok(ProjectedScalar {
                scalar_value: outcome.scalar_value,
                text,
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
    calculation: crate::numeric::expression::ColorCalculationRef<'_>,
    factor: Factor,
    target: ComponentTarget,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    let scale = match target {
        ComponentTarget::Number
            if matches!(
                calculation,
                crate::numeric::expression::ColorCalculationRef::HintedNumber(_)
            ) && factor.numerator == factor.denominator =>
        {
            crate::numeric::projection::NumericProjectionScale::Identity
        }
        ComponentTarget::Preserve => crate::numeric::projection::NumericProjectionScale::Identity,
        ComponentTarget::Number => crate::numeric::projection::NumericProjectionScale::Number {
            numerator: factor.numerator,
            denominator: factor.denominator,
        },
        ComponentTarget::Percentage => {
            crate::numeric::projection::NumericProjectionScale::NumberToPercentage {
                numerator: factor.numerator,
                denominator: factor.denominator,
            }
        }
    };
    let (text, outcome) = crate::numeric::expression::capture_color_component_calculation(
        calculation,
        scale,
        context,
    )?;
    Ok(ProjectedScalar {
        scalar_value: outcome.scalar_value,
        text,
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
        context.charge_nodes(1)?;
        return Ok(Some(output_text("none", context)));
    }
    match value {
        CssColorComponent::NumberCalculation(_)
        | CssColorComponent::HintedNumberCalculation(_)
        | CssColorComponent::PercentageCalculation(_) => {
            let (calculation, scale) = match value {
                CssColorComponent::NumberCalculation(value) => (
                    crate::numeric::expression::ColorCalculationRef::Number(value),
                    crate::numeric::projection::NumericProjectionScale::Identity,
                ),
                CssColorComponent::HintedNumberCalculation(value) => (
                    crate::numeric::expression::ColorCalculationRef::HintedNumber(value),
                    crate::numeric::projection::NumericProjectionScale::Identity,
                ),
                CssColorComponent::PercentageCalculation(value) => (
                    crate::numeric::expression::ColorCalculationRef::Percentage(value),
                    crate::numeric::projection::NumericProjectionScale::PercentageToNumber {
                        numerator: 1,
                        denominator: 100,
                    },
                ),
                _ => unreachable!("calculated alpha"),
            };
            // Capture owns cumulative graph and scratch checks even when the
            // ordinary scalar will be omitted. Its outcome precedes rounding.
            let (text, outcome) = crate::numeric::expression::capture_retained_color_calculation(
                calculation,
                scale,
                context,
            )?;
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
    let value =
        crate::serialization::exact_decimal::ExactRational::from_lexical_factor_clamped_unit(
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
    let expanded = crate::color::hex::expand(value.digits()).expect("checked hex color");
    let alpha = Some(expanded[3]).filter(|alpha| *alpha != 255);
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
        out.push(
            &crate::serialization::specified_serialization::format_binary64(
                surgeist_color::byte_alpha_for_serialization(alpha),
            ),
        )?;
    }
    out.push(")")?;
    Ok(out.finish())
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
    // rgb() declares 100% = 255; color(srgb) declares 100% = 1.
    let rgb_percent = percent_factor(ColorFunction::Rgb, 0);
    let rgb_reference = rgb_percent.numerator * 100 / rgb_percent.denominator;
    if missing {
        let mut channels = value
            .channels()
            .iter()
            .enumerate()
            .map(|(index, channel)| {
                component_projection_with_text(
                    channel,
                    Factor {
                        numerator: 1,
                        denominator: rgb_reference,
                    },
                    percent_factor(ColorFunction::Color, index),
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
        .enumerate()
        .map(|(index, channel)| {
            component_projection_with_text(
                channel,
                Factor::ONE,
                percent_factor(ColorFunction::Rgb, index),
                ComponentTarget::Number,
                false,
                context,
            )
        })
        .collect::<Result<Vec<_>>>()?;
    for channel in &mut channels {
        finalize_rgb_channel(channel, rgb_reference, context)?;
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

fn finalize_hue(
    value: &mut ProjectedScalar,
    context: &SpecifiedSerializationContext,
) -> Result<()> {
    if let Some(number) = value.scalar_value {
        value.text = ordinary_number(crate::color::channels::normalize_hue(number), context)?;
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
    // Conversion reads scalar_value's original projection bits; only the
    // emitted natural-form text uses the normalized operand.
    let number = if number.is_nan() { 0.0 } else { number };
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

/// Binary64 channel inputs exactly as used-value resolution reads them: a
/// literal is its correctly rounded decimal value, and a calculation is its
/// projected scalar. A finite literal beyond binary64 is the closest finite
/// value (Values 4 §10.12), as in used literal conversion.
fn used_scalar(value: &CssColorComponent, projected: &ProjectedScalar) -> channels::Component {
    use CssColorComponent as C;
    let (domain, number) = match value {
        C::Number(value) => (
            channels::Domain::Number,
            literal_binary64(value.numeric().representation()),
        ),
        C::Percentage(value) => (
            channels::Domain::Percentage,
            literal_binary64(value.numeric().representation()),
        ),
        C::PercentageCalculation(_) => (
            channels::Domain::Percentage,
            projected.scalar_value.expect("noncontextual calculation"),
        ),
        C::NumberCalculation(_) | C::HintedNumberCalculation(_) => (
            channels::Domain::Number,
            projected.scalar_value.expect("noncontextual calculation"),
        ),
        C::None => unreachable!("complete HSL/HWB channel"),
    };
    channels::Component {
        domain,
        value: saturated(number),
    }
}

fn used_hue(
    value: &CssColorHue,
    projected: &ProjectedScalar,
    context: &mut SpecifiedSerializationContext,
) -> Result<channels::Component> {
    use CssColorHue as H;
    Ok(match value {
        H::Number(value) => channels::Component {
            domain: channels::Domain::Number,
            value: literal_hue_degrees(value.numeric(), None, context)?,
        },
        H::Angle(value) => channels::Component {
            domain: channels::Domain::Degrees,
            value: literal_hue_degrees(
                value.numeric(),
                Some(crate::numeric::CssNumericUnit::Angle(value.unit())),
                context,
            )?,
        },
        H::NumberCalculation(_) => channels::Component {
            domain: channels::Domain::Number,
            value: projected.scalar_value.expect("noncontextual calculation"),
        },
        H::AngleCalculation(_) => channels::Component {
            domain: channels::Domain::Degrees,
            value: projected.scalar_value.expect("noncontextual calculation"),
        },
        H::None => unreachable!("complete HSL/HWB hue"),
    })
}

/// A literal hue in degrees exactly as used resolution reads it. Below
/// 1e305 no canonical angle can exceed binary64 (360 is the largest unit
/// factor), so binary64 scaling of the correctly rounded literal is exact
/// policy; larger literals run the shared used literal conversion, charged per
/// action, for its scaled gradians and its 360deg-multiple overflow endpoint.
fn literal_hue_degrees(
    numeric: crate::component::CssNumericTokenRef<'_>,
    unit: Option<crate::numeric::CssNumericUnit>,
    context: &mut SpecifiedSerializationContext,
) -> Result<f64> {
    let representation = numeric.representation();
    if crate::serialization::exact_decimal::LexicalDecimal::new(representation)
        .compare_absolute("1", 305)
        .is_lt()
    {
        let value = literal_binary64(representation);
        return Ok(unit
            .and_then(crate::numeric::CssNumericUnit::intrinsic_projection)
            .map_or(value, |scale| scale.apply(value)));
    }
    crate::color::authored_resolution::spelling::authored_hue_degrees(numeric, unit, || {
        context.charge_projection(1)
    })
}

/// Whether a literal hue overflows binary64, so its degrees are the nearest
/// supported multiple of 360deg, which normalizes to 0deg.
fn literal_hue_overflows(
    numeric: crate::component::CssNumericTokenRef<'_>,
    unit: Option<crate::numeric::CssNumericUnit>,
    context: &mut SpecifiedSerializationContext,
) -> Result<bool> {
    if crate::serialization::exact_decimal::LexicalDecimal::new(numeric.representation())
        .compare_absolute("1", 305)
        .is_lt()
    {
        return Ok(false);
    }
    let degrees =
        crate::color::authored_resolution::spelling::authored_hue_degrees(numeric, unit, || {
            context.charge_projection(1)
        })?;
    Ok(degrees.abs()
        == crate::numeric::projection::closest_supported_value(f64::INFINITY, true, true))
}

type HueParts = (
    String,
    Option<f64>,
    Option<crate::serialization::exact_decimal::ExactRational>,
    bool,
    bool,
    bool,
);

/// Values 4 §10.12: a literal hue beyond binary64 is the nearest supported
/// multiple of 360deg, which normalizes to 0deg, as used values read it.
fn overflowed_hue(context: &mut SpecifiedSerializationContext) -> Result<HueParts> {
    let zero = crate::serialization::exact_decimal::ExactRational::integer(0, context)?;
    Ok((
        output_text("0", context),
        None,
        Some(zero),
        false,
        false,
        false,
    ))
}

fn literal_binary64(representation: &str) -> f64 {
    representation
        .parse()
        .expect("checked CSS number is a decimal literal")
}

fn saturated(value: f64) -> f64 {
    if value.is_infinite() {
        f64::MAX.copysign(value)
    } else {
        value
    }
}

/// Color 4 §§7.1, 8.1 and 16.2.2: complete specified hsl()/hwb() serialize
/// as the corresponding sRGB channels in legacy rgb() form. The numerical
/// conversion is surgeist-color's, through the same channel resolution as
/// used values, so specified and used serialization share one algorithm.
fn converted_rgb_text(
    function: ColorFunction,
    components: [channels::Component; 3],
    context: &SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    let unrepresentable = || {
        Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::UnrepresentableValue)
    };
    let color = channels::absolute_coordinates(
        function,
        None,
        crate::color::CssColorSyntax::Modern,
        components,
        None,
    )
    .map_err(|_| unrepresentable())?
    .converted(ColorSpace::Predefined(PredefinedColorSpace::Srgb));
    let mut text = [String::new(), String::new(), String::new()];
    for (target, channel) in text.iter_mut().zip(color.channels()) {
        let channel = channel.ok_or_else(unrepresentable)?;
        *target = ordinary_number(channel.clamp(0.0, 1.0) * 255.0, context)?;
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
    let channels = converted_rgb_text(
        ColorFunction::Hsl,
        [
            used_hue(value.hue(), &hue, context)?,
            used_scalar(value.saturation(), &saturation),
            used_scalar(value.lightness(), &lightness),
        ],
        context,
    )?;
    legacy_rgb(&channels, alpha.as_deref(), context)
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
    let channels = converted_rgb_text(
        ColorFunction::Hwb,
        [
            used_hue(value.hue(), &hue, context)?,
            used_scalar(value.whiteness(), &white),
            used_scalar(value.blackness(), &black),
        ],
        context,
    )?;
    legacy_rgb(&channels, alpha.as_deref(), context)
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
                exact: None,
                contextual: false,
                missing: false,
                percentage: false,
                calculation: false,
                direct_text_pending: false,
            });
        }
    }

    let (text, scalar_value, exact, contextual, missing, calculation) = match value {
        H::None => {
            context.charge_nodes(1)?;
            (output_text("none", context), None, None, false, true, false)
        }
        H::Number(value) if literal_hue_overflows(value.numeric(), None, context)? => {
            context.charge_input(1)?;
            overflowed_hue(context)?
        }
        H::Angle(value)
            if literal_hue_overflows(
                value.numeric(),
                Some(crate::numeric::CssNumericUnit::Angle(value.unit())),
                context,
            )? =>
        {
            context.charge_input(1)?;
            overflowed_hue(context)?
        }
        H::Number(value) => {
            context.charge_input(1)?;
            let source = value.numeric().representation();
            let exact =
                crate::serialization::exact_decimal::ExactRational::from_lexical_factor_modulo(
                    source,
                    exact_factor(Factor::ONE),
                    360,
                    context,
                )?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, None, Some(exact), false, false, false)
        }
        H::Angle(value) => {
            context.charge_input(1)?;
            let factor = angle_factor(value.unit());
            let source = value.numeric().representation();
            let exact =
                crate::serialization::exact_decimal::ExactRational::from_lexical_factor_modulo(
                    source,
                    exact_factor(factor),
                    360,
                    context,
                )?;
            let degrees = exact.clone_with_budget(context)?;
            // Color 4 §16 serializes the hue as a canonicalized <number> of
            // degrees, which CSSOM rounds to at most six decimals. Degree and
            // turn literals are exact decimal multiples and keep their digits.
            let text = if matches!(value.unit(), CssAngleUnit::Radians | CssAngleUnit::Gradians) {
                let text = degrees.format_rounded(6, context.remaining_bytes(), context)?;
                // A normalized hue just below 360 can round up to the
                // excluded endpoint; it denotes the same direction as 0.
                if text == "360" {
                    output_text("0", context)
                } else {
                    text
                }
            } else {
                degrees.format_exact(context.remaining_bytes(), context)?
            };
            (text, None, Some(exact), false, false, false)
        }
        H::NumberCalculation(value) => {
            let calculation = crate::numeric::expression::ColorCalculationRef::Number(value);
            let (text, outcome) = if origin {
                crate::numeric::expression::capture_retained_color_calculation(
                    calculation,
                    crate::numeric::projection::NumericProjectionScale::Identity,
                    context,
                )?
            } else {
                crate::numeric::expression::capture_color_component_calculation(
                    calculation,
                    crate::numeric::projection::NumericProjectionScale::Identity,
                    context,
                )?
            };
            (
                text,
                outcome.scalar_value,
                None,
                outcome.context_dependent,
                false,
                true,
            )
        }
        H::AngleCalculation(value) => {
            let calculation = crate::numeric::expression::ColorCalculationRef::Angle(value);
            let (text, outcome) = if origin {
                crate::numeric::expression::capture_retained_color_calculation(
                    calculation,
                    crate::numeric::projection::NumericProjectionScale::Identity,
                    context,
                )?
            } else {
                crate::numeric::expression::capture_color_component_calculation(
                    calculation,
                    crate::numeric::projection::NumericProjectionScale::Identity,
                    context,
                )?
            };
            (
                text,
                outcome.scalar_value,
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
        exact,
        contextual,
        missing,
        percentage: false,
        calculation,
        direct_text_pending: false,
    })
}

fn angle_factor(unit: CssAngleUnit) -> Factor {
    use crate::numeric::units::{self, UnitRatio};
    let factor = match units::ratio(crate::numeric::CssNumericUnit::Angle(unit)) {
        UnitRatio::Exact(factor) => factor,
        UnitRatio::Radian => units::RADIAN_BINARY64,
        UnitRatio::Contextual => unreachable!("angles convert intrinsically"),
    };
    Factor {
        numerator: factor.numerator,
        denominator: factor.denominator,
    }
}

fn serialize_lab(
    name: &str,
    value: &CssLabColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let origin = mode == Mode::Origin;
    let function = if name == "lab" {
        ColorFunction::Lab
    } else {
        ColorFunction::Oklab
    };
    let (light_percentage, axis_percentage) = if origin {
        (Factor::ONE, Factor::ONE)
    } else {
        (percent_factor(function, 0), percent_factor(function, 1))
    };
    let target = if origin {
        ComponentTarget::Preserve
    } else {
        ComponentTarget::Number
    };
    let lightness = lab_component_projection(
        value.lightness(),
        light_percentage,
        channels::slot(function, 0),
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
    let function = if name == "lch" {
        ColorFunction::Lch
    } else {
        ColorFunction::Oklch
    };
    let (light_percentage, chroma_percentage) = if origin {
        (Factor::ONE, Factor::ONE)
    } else {
        (percent_factor(function, 0), percent_factor(function, 1))
    };
    let lightness = lab_component_projection(
        value.lightness(),
        light_percentage,
        channels::slot(function, 0),
        mode,
        context,
    )?;
    let channels = [
        lightness,
        lab_component_projection(
            value.chroma(),
            chroma_percentage,
            channels::slot(function, 1),
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
        .enumerate()
        .map(|(index, channel)| {
            declared_component_projection(
                channel,
                Factor::ONE,
                if origin {
                    Factor::ONE
                } else {
                    percent_factor(ColorFunction::Color, index)
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
    let name = format!("color({}", predefined_name(value.color_space())?);
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
        .enumerate()
        .map(|(index, channel)| {
            declared_component_projection(
                channel,
                Factor::ONE,
                if origin {
                    Factor::ONE
                } else {
                    percent_factor(ColorFunction::DeviceCmyk, index)
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
        .enumerate()
        .map(|(index, channel)| {
            declared_component_projection(
                channel,
                Factor::ONE,
                if origin {
                    Factor::ONE
                } else {
                    percent_factor(ColorFunction::Color, index)
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
            context.charge_nodes(1)?;
            Ok(output_text("none", context))
        }
        V::Channel(channel) => {
            context.charge_nodes(1)?;
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
                crate::numeric::projection::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                }
            } else {
                crate::numeric::projection::NumericProjectionScale::Identity
            };
            let (text, _) = crate::numeric::projection::capture_specified_scaled(
                &value.data.expression,
                scale,
                context,
            )?;
            Ok(text)
        }
    }
}

fn serialize_profile_expression(
    value: &crate::color::CssProfileColorExpression,
    alpha: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    match value.view() {
        crate::color::CssProfileColorExpressionRef::Literal(value) => {
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
        crate::color::CssProfileColorExpressionRef::Reference(value) => {
            context.charge_nodes(1)?;
            escaped_identifier(value.as_str(), context)
        }
        crate::color::CssProfileColorExpressionRef::Calculation(value) => {
            let scale = if value.result_type() == CssCalculationType::Percentage {
                crate::numeric::projection::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                }
            } else {
                crate::numeric::projection::NumericProjectionScale::Identity
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
        serialize_interpolation_method(&value)
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

fn serialize_interpolation_method(value: &CssColorInterpolationMethod) -> Result<String> {
    let mut result = match value.space() {
        ColorSpace::Predefined(value) => predefined_name(value)?.to_owned(),
        ColorSpace::Hsl => "hsl".into(),
        ColorSpace::Hwb => "hwb".into(),
        ColorSpace::Lab => "lab".into(),
        ColorSpace::Lch => "lch".into(),
        ColorSpace::Oklab => "oklab".into(),
        ColorSpace::Oklch => "oklch".into(),
        _ => return Err(unknown_color_edition()),
    };
    if let Some(hue) = value.hue().filter(|hue| *hue != HueInterpolation::Shorter) {
        result.push(' ');
        result.push_str(match hue {
            HueInterpolation::Shorter => "shorter hue",
            HueInterpolation::Longer => "longer hue",
            HueInterpolation::Increasing => "increasing hue",
            HueInterpolation::Decreasing => "decreasing hue",
            _ => return Err(unknown_color_edition()),
        });
    }
    Ok(result)
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
            Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
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
                    let (text, _) = crate::numeric::expression::capture_retained_color_calculation(
                        crate::numeric::expression::ColorCalculationRef::Percentage(calculation),
                        crate::numeric::projection::NumericProjectionScale::Identity,
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
            Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
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
    exact: Vec<Option<crate::serialization::exact_decimal::ExactRational>>,
    generated: Option<crate::serialization::exact_decimal::ExactRational>,
}
impl EffectiveMixWeights {
    pub(super) fn value(
        &self,
        index: usize,
    ) -> &crate::serialization::exact_decimal::ExactRational {
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
        Error::new(crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::CapacityOverflow)
    })?;
    let mut sum: Option<crate::serialization::exact_decimal::ExactRational> = None;
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
    let sum = sum.unwrap_or(
        crate::serialization::exact_decimal::ExactRational::from_lexical_factor(
            "0",
            exact_factor(Factor::ONE),
            context,
        )?,
    );
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
) -> Result<crate::serialization::exact_decimal::ExactRational> {
    context.charge_input(1)?;
    crate::serialization::exact_decimal::ExactRational::from_lexical_factor(
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
    use crate::serialization::CssSerializationLimits as Limits;

    use crate::property::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};
    use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;

    fn assert_suppressed_color(source: &str) {
        let declaration = crate::declaration::CssDeclaration::try_from_components(
            &crate::CssParser::default(),
            crate::property::CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::Color,
            )),
            crate::component::values::parse_component_values(source).unwrap(),
            crate::declaration::CssImportance::Normal,
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
    use crate::property::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};
    use crate::serialization::CssSerializationLimits as Limits;
    use crate::serialization::specified_rule_serialization::SpecifiedRuleWriter;
    use crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind as K;

    fn color(source: &str) -> CssColor {
        let declaration = crate::declaration::CssDeclaration::try_from_components(
            &crate::CssParser::default(),
            crate::property::CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::Color,
            )),
            crate::component::values::parse_component_values(source).unwrap(),
            crate::declaration::CssImportance::Important,
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
    fn hsl_vertices_never_materialize_discarded_channel_text() {
        // Color 4 §7.1: full saturation at half lightness reaches the sRGB
        // cube vertices at multiples of 60 degrees.
        for (angle, expected) in [
            (0.0, ["255", "0", "0"]),
            (60.0, ["255", "255", "0"]),
            (120.0, ["0", "255", "0"]),
            (180.0, ["0", "255", "255"]),
            (240.0, ["0", "0", "255"]),
            (300.0, ["255", "0", "255"]),
        ] {
            for suppressed in [false, true] {
                let mut context = SpecifiedSerializationContext::new(Limits::new(
                    0,
                    1_000,
                    if suppressed { 0 } else { 100 },
                ));
                context.replace_output_suppression(suppressed);
                let percent = |value| channels::Component {
                    domain: channels::Domain::Percentage,
                    value,
                };
                let channels = converted_rgb_text(
                    ColorFunction::Hsl,
                    [
                        channels::Component {
                            domain: channels::Domain::Degrees,
                            value: angle,
                        },
                        percent(100.0),
                        percent(50.0),
                    ],
                    &context,
                )
                .unwrap();
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
        let value = color("rgb(12345678901234567891e170141183460469231731687303715884105727 0 0)");
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
    use crate::property::{CssKnownProperty, CssKnownPropertyValueRef, CssPropertyNameRef};
    use crate::serialization::CssSerializationLimits as Limits;

    fn checked_color(source: &str) -> CssColor {
        let declaration = crate::declaration::CssDeclaration::try_from_components(
            &crate::CssParser::default(),
            crate::property::CssPropertyTarget::Property(CssPropertyNameRef::Known(
                CssKnownProperty::Color,
            )),
            crate::component::values::parse_component_values(source).unwrap(),
            crate::declaration::CssImportance::Normal,
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
            value
                .serialize_specified(Limits::new(usize::MAX, usize::MAX, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified(Limits::new(usize::MAX, usize::MAX, expected.len() - 1))
                .unwrap_err()
                .kind(),
            crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::ByteLimit
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
                value
                    .serialize_specified(Limits::new(5, projections, expected.len()))
                    .unwrap(),
                expected
            );
            assert_eq!(
                value
                    .serialize_specified(Limits::new(5, projections - 1, expected.len()))
                    .unwrap_err()
                    .kind(),
                crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
            );
            assert_eq!(
                value
                    .serialize_specified(Limits::new(4, projections, expected.len()))
                    .unwrap_err()
                    .kind(),
                crate::serialization::specified_serialization::CssSpecifiedValueSerializationErrorKind::InputNodeLimit
            );
            assert_eq!(value, original);
        }
    }
}
