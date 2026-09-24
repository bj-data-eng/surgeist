//! Canonical specified-color serialization over checked authored graphs.

use super::*;
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
    Color(&'a CssAuthoredColor, Mode),
    Frozen(&'a CssColor, Mode),
    Text(&'static str),
    Owned(String),
}

pub(super) fn serialize(value: &CssAuthoredColor, limits: Limits) -> Result<String> {
    let mut context = SpecifiedSerializationContext::new(limits);
    let mut output = String::new();
    value.append_specified(&mut context, &mut output)?;
    Ok(output)
}

impl CssAuthoredColor {
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
                Work::Frozen(color, mode) => {
                    context.charge_input(1)?;
                    context.charge_projection(1)?;
                    schedule_frozen(color, mode, context, &mut work)?;
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
    color: &'a CssAuthoredColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
    work: &mut Vec<Work<'a>>,
) -> Result<()> {
    reserve_work(work, 1)?;
    use CssAuthoredColorRepresentation as R;
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
        R::PreservedI01(value) => work.push(Work::Frozen(value, mode)),
    }
    Ok(())
}

fn schedule_frozen<'a>(
    color: &'a CssColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
    work: &mut Vec<Work<'a>>,
) -> Result<()> {
    reserve_work(work, 1)?;
    match color {
        CssColor::CurrentColor => work.push(Work::Text("currentcolor")),
        CssColor::Rgba(value) => {
            work.push(Work::Owned(serialize_frozen_rgba(value, mode, context)?));
        }
        CssColor::Hsl(value) => work.push(Work::Owned(serialize_frozen_hsl(value, mode, context)?)),
        CssColor::Hwb(value) => work.push(Work::Owned(serialize_frozen_hwb(value, mode, context)?)),
        CssColor::Lab(value) => {
            work.push(Work::Owned(serialize_frozen_lab("lab", value, context)?));
        }
        CssColor::Lch(value) => {
            work.push(Work::Owned(serialize_frozen_lch("lch", value, context)?));
        }
        CssColor::Oklab(value) => {
            work.push(Work::Owned(serialize_frozen_lab("oklab", value, context)?));
        }
        CssColor::Oklch(value) => {
            work.push(Work::Owned(serialize_frozen_lch("oklch", value, context)?));
        }
        CssColor::ColorFunction(value) => {
            work.push(Work::Owned(serialize_frozen_predefined(value, context)?));
        }
        CssColor::System(value) => work.push(Work::Text(system_name(*value))),
        CssColor::ColorMix(value) => schedule_frozen_mix(value, work, context)?,
        CssColor::Relative(value) => schedule_frozen_relative(value, work, context)?,
    }
    Ok(())
}

fn schedule_alpha<'a>(
    value: &'a CssAuthoredAlphaColor,
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
    value: &'a CssAuthoredRelativeCustomColor,
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
    value: &'a CssAuthoredRelativeColor,
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
    value: &'a CssAuthoredColorMix,
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

fn is_default_mix(value: &CssAuthoredColorInterpolation) -> bool {
    value.predefined().is_some_and(|value| {
        value.space() == CssColorInterpolationSpace::Oklab && value.hue().is_none()
    })
}

fn is_default_interpolation_method(value: &CssColorInterpolationMethod) -> bool {
    value.space() == CssColorInterpolationSpace::Oklab && value.hue().is_none()
}

fn schedule_frozen_mix<'a>(
    value: &'a CssColorMix,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    reserve_work(work, 12)?;
    context.charge_input(1)?;
    context.charge_projection(1)?;
    let weights = frozen_mix_weight_texts(value, context)?;
    work.push(Work::Text(")"));
    if let Some(weight) = weights[1].as_ref() {
        work.push(Work::Owned(weight.clone()));
        work.push(Work::Text(" "));
    }
    work.push(Work::Frozen(value.right().color(), Mode::Mix));
    work.push(Work::Text(", "));
    if let Some(weight) = weights[0].as_ref() {
        work.push(Work::Owned(weight.clone()));
        work.push(Work::Text(" "));
    }
    work.push(Work::Frozen(value.left().color(), Mode::Mix));
    if !is_default_interpolation_method(value.interpolation()) {
        work.push(Work::Text(", "));
        work.push(Work::Owned(serialize_interpolation_method(
            value.interpolation(),
        )));
        work.push(Work::Text("in "));
    }
    work.push(Work::Text("color-mix("));
    Ok(())
}

fn frozen_mix_weight_texts(
    value: &CssColorMix,
    context: &mut SpecifiedSerializationContext,
) -> Result<[Option<String>; 2]> {
    let percentages = [value.left().percentage(), value.right().percentage()];
    if percentages.iter().all(Option::is_none)
        || percentages.iter().flatten().all(|value| *value == 50.0)
    {
        return Ok([None, None]);
    }

    let explicit_index = percentages
        .iter()
        .position(Option::is_some)
        .expect("at least one frozen mix weight is explicit");
    let mut result: [Option<String>; 2] = [None, None];
    for (index, percentage) in percentages.into_iter().enumerate() {
        context.charge_input(usize::from(percentage.is_some()))?;
        let exact = if let Some(percentage) = percentage {
            crate::opacity_scalar::ExactRational::from_binary32_factor(
                percentage,
                exact_factor(Factor::ONE),
                context,
            )?
        } else {
            let explicit = percentages[explicit_index].expect("selected weight is explicit");
            crate::opacity_scalar::ExactRational::from_binary32_factor(
                explicit,
                exact_factor(Factor::ONE),
                context,
            )?
            .min_integer(100, context)?
            .subtract_from_integer(100, context)?
        };
        let text = exact.format_exact(context.remaining_bytes(), context)?;
        result[index] = Some(suffix_text(text, "%", context)?);
    }
    Ok(result)
}

fn schedule_frozen_relative<'a>(
    value: &'a CssRelativeColor,
    work: &mut Vec<Work<'a>>,
    context: &mut SpecifiedSerializationContext,
) -> Result<()> {
    reserve_work(work, work_slots(5, 2, value.components().len())?)?;
    let (name, space) = relative_function(value.function());
    let (environment, domains) = legacy_relative_signature(value.function());
    work.push(Work::Text(")"));
    if let Some(alpha) = value.alpha() {
        work.push(Work::Owned(serialize_legacy_expression(
            alpha.authored().as_css(),
            environment,
            CssRelativeColorResultDomain::NumberPercentage,
            true,
            context,
        )?));
        work.push(Work::Text(" / "));
    }
    for (index, channel) in value.components().iter().enumerate().rev() {
        work.push(Work::Owned(serialize_legacy_expression(
            channel.authored().as_css(),
            environment,
            domains[index],
            false,
            context,
        )?));
        work.push(Work::Text(" "));
    }
    if let Some(space) = space {
        work.push(Work::Text(space));
        work.push(Work::Text(" "));
    }
    work.push(Work::Frozen(value.source(), Mode::Origin));
    work.push(Work::Owned(format!("{name}(from ")));
    Ok(())
}

fn serialize_legacy_expression(
    css: &str,
    environment: CssRelativeColorEnvironment,
    domain: CssRelativeColorResultDomain,
    alpha: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    use crate::{
        CssComponentValueErrorKind as ComponentError,
        CssSpecifiedValueSerializationErrorKind as SerializationError,
    };

    let limits = crate::CssComponentValueLimits::try_new(
        crate::STRUCTURAL_NESTING_LIMIT,
        context.remaining_input_nodes(),
        context.remaining_bytes(),
    )
    .expect("repository structural ceiling is a valid component limit");
    let values = crate::parse_component_values_with_limits(css, limits).map_err(|error| {
        Error::new(match error.kind() {
            ComponentError::ComponentLimit => SerializationError::InputNodeLimit,
            ComponentError::ByteLimit => SerializationError::ByteLimit,
            ComponentError::CapacityOverflow => SerializationError::CapacityOverflow,
            _ => unreachable!("checked frozen relative expression"),
        })
    })?;
    let serialized = values
        .serialize_with_limit(context.remaining_bytes())
        .map_err(|error| {
            Error::new(match error.kind() {
                ComponentError::ByteLimit => SerializationError::ByteLimit,
                ComponentError::CapacityOverflow => SerializationError::CapacityOverflow,
                _ => unreachable!("checked frozen relative expression serialization"),
            })
        })?;
    if let Some(expression) =
        crate::parser::adapt_legacy_relative_expression(&values, &serialized, environment, domain)
    {
        return serialize_relative_expression(&expression, alpha, context);
    }
    context.charge_input(values.component_count())?;
    context.charge_projection(values.component_count())?;
    Ok(serialized.as_css().to_owned())
}

fn legacy_relative_signature(
    function: &CssRelativeColorFunction,
) -> (
    CssRelativeColorEnvironment,
    [CssRelativeColorResultDomain; 3],
) {
    use CssRelativeColorEnvironment as E;
    use CssRelativeColorResultDomain::{Hue, NumberPercentage};
    match *function {
        CssRelativeColorFunction::Rgb => (E::Rgb, [NumberPercentage; 3]),
        CssRelativeColorFunction::Hsl => (E::Hsl, [Hue, NumberPercentage, NumberPercentage]),
        CssRelativeColorFunction::Hwb => (E::Hwb, [Hue, NumberPercentage, NumberPercentage]),
        CssRelativeColorFunction::Lab => (E::Lab, [NumberPercentage; 3]),
        CssRelativeColorFunction::Lch => (E::Lch, [NumberPercentage, NumberPercentage, Hue]),
        CssRelativeColorFunction::Oklab => (E::Oklab, [NumberPercentage; 3]),
        CssRelativeColorFunction::Oklch => (E::Oklch, [NumberPercentage, NumberPercentage, Hue]),
        CssRelativeColorFunction::Color(space) => (
            match space {
                CssPredefinedColorSpace::XyzD50 | CssPredefinedColorSpace::XyzD65 => E::Xyz(space),
                _ => E::PredefinedRgb(space),
            },
            [NumberPercentage; 3],
        ),
    }
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

fn authored_system_name(value: CssAuthoredSystemColor) -> &'static str {
    match value {
        CssAuthoredSystemColor::Canvas => "canvas",
        CssAuthoredSystemColor::CanvasText => "canvastext",
        CssAuthoredSystemColor::LinkText => "linktext",
        CssAuthoredSystemColor::VisitedText => "visitedtext",
        CssAuthoredSystemColor::ActiveText => "activetext",
        CssAuthoredSystemColor::ButtonFace => "buttonface",
        CssAuthoredSystemColor::ButtonText => "buttontext",
        CssAuthoredSystemColor::ButtonBorder => "buttonborder",
        CssAuthoredSystemColor::Field => "field",
        CssAuthoredSystemColor::FieldText => "fieldtext",
        CssAuthoredSystemColor::Highlight => "highlight",
        CssAuthoredSystemColor::HighlightText => "highlighttext",
        CssAuthoredSystemColor::Mark => "mark",
        CssAuthoredSystemColor::MarkText => "marktext",
        CssAuthoredSystemColor::GrayText => "graytext",
        CssAuthoredSystemColor::SelectedItem => "selecteditem",
        CssAuthoredSystemColor::SelectedItemText => "selecteditemtext",
        CssAuthoredSystemColor::AccentColor => "accentcolor",
        CssAuthoredSystemColor::AccentColorText => "accentcolortext",
        CssAuthoredSystemColor::ActiveBorder => "activeborder",
        CssAuthoredSystemColor::ActiveCaption => "activecaption",
        CssAuthoredSystemColor::AppWorkspace => "appworkspace",
        CssAuthoredSystemColor::Background => "background",
        CssAuthoredSystemColor::ButtonHighlight => "buttonhighlight",
        CssAuthoredSystemColor::ButtonShadow => "buttonshadow",
        CssAuthoredSystemColor::CaptionText => "captiontext",
        CssAuthoredSystemColor::InactiveBorder => "inactiveborder",
        CssAuthoredSystemColor::InactiveCaption => "inactivecaption",
        CssAuthoredSystemColor::InactiveCaptionText => "inactivecaptiontext",
        CssAuthoredSystemColor::InfoBackground => "infobackground",
        CssAuthoredSystemColor::InfoText => "infotext",
        CssAuthoredSystemColor::Menu => "menu",
        CssAuthoredSystemColor::MenuText => "menutext",
        CssAuthoredSystemColor::Scrollbar => "scrollbar",
        CssAuthoredSystemColor::ThreeDDarkShadow => "threeddarkshadow",
        CssAuthoredSystemColor::ThreeDFace => "threedface",
        CssAuthoredSystemColor::ThreeDHighlight => "threedhighlight",
        CssAuthoredSystemColor::ThreeDLightShadow => "threedlightshadow",
        CssAuthoredSystemColor::ThreeDShadow => "threedshadow",
        CssAuthoredSystemColor::Window => "window",
        CssAuthoredSystemColor::WindowFrame => "windowframe",
        CssAuthoredSystemColor::WindowText => "windowtext",
    }
}

fn system_name(value: CssSystemColor) -> &'static str {
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
    exact: Option<crate::opacity_scalar::ExactRational>,
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

    fn from_exact(value: &crate::opacity_scalar::ExactRational) -> Result<Self> {
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

fn exact_factor(value: Factor) -> crate::opacity_scalar::ExactFactor {
    crate::opacity_scalar::ExactFactor {
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
    let value = crate::opacity_scalar::ExactRational::from_lexical_factor(
        representation,
        exact_factor(factor),
        context,
    )?;
    match rounded_places {
        Some(places) => value.format_rounded(places, context.remaining_bytes(), context),
        None => value.format_exact(context.remaining_bytes(), context),
    }
}

fn finite_text(
    value: f32,
    factor: Factor,
    rounded_places: Option<usize>,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
        value,
        exact_factor(factor),
        context,
    )?;
    match rounded_places {
        Some(places) => exact.format_rounded(places, context.remaining_bytes(), context),
        None => exact.format_exact(context.remaining_bytes(), context),
    }
}

fn format_finite(value: f32, context: &mut SpecifiedSerializationContext) -> Result<String> {
    finite_text(value, Factor::ONE, None, context)
}

fn component_projection(
    value: &CssAuthoredColorComponent,
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
    value: &CssAuthoredColorComponent,
    number_factor: Factor,
    percentage_factor: Factor,
    target: ComponentTarget,
    materialize_direct: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    use CssAuthoredColorComponent as C;
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
        C::ExactNumber(value) => {
            context.charge_input(1)?;
            let representation = value.numeric().representation();
            let exact = crate::opacity_scalar::ExactRational::from_lexical_factor(
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
        C::ExactPercentage(value) => {
            context.charge_input(1)?;
            let representation = value.numeric().representation();
            let exact = crate::opacity_scalar::ExactRational::from_lexical_factor(
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
        C::Number(value) => {
            context.charge_input(1)?;
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value.value(),
                exact_factor(number_factor),
                context,
            )?;
            Ok(ProjectedScalar {
                text: if materialize_direct {
                    exact
                        .clone_with_budget(context)?
                        .format_exact(context.remaining_bytes(), context)?
                } else {
                    String::new()
                },
                number: Some(ScaledNumber::from_exact(&exact)?),
                exact: Some(exact),
                contextual: false,
                missing: false,
                percentage: target == ComponentTarget::Percentage,
                calculation: false,
            })
        }
        C::Percentage(value) => {
            context.charge_input(1)?;
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value.value(),
                exact_factor(percentage_factor),
                context,
            )?;
            Ok(ProjectedScalar {
                text: if materialize_direct {
                    exact
                        .clone_with_budget(context)?
                        .format_exact(context.remaining_bytes(), context)?
                } else {
                    String::new()
                },
                number: Some(ScaledNumber::from_exact(&exact)?),
                exact: Some(exact),
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
            let (text, outcome) = crate::numeric::capture_calculation_specified_scaled(
                crate::numeric::SpecifiedCalculationRef::Number(value),
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
            let (text, outcome) = crate::numeric::capture_calculation_specified_scaled(
                crate::numeric::SpecifiedCalculationRef::Percentage(value),
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
    value: Option<&CssAuthoredColorComponent>,
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
    if matches!(value, CssAuthoredColorComponent::None) {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        return Ok(Some("none".into()));
    }
    match value {
        CssAuthoredColorComponent::NumberCalculation(calculation) => {
            let (text, _) = crate::numeric::capture_calculation_specified(
                crate::numeric::SpecifiedCalculationRef::Number(calculation),
                context,
            )?;
            Ok(Some(text))
        }
        CssAuthoredColorComponent::PercentageCalculation(calculation) => {
            let (text, _) = crate::numeric::capture_calculation_specified_scaled(
                crate::numeric::SpecifiedCalculationRef::Percentage(calculation),
                crate::numeric::NumericProjectionScale::PercentageToNumber {
                    numerator: 1,
                    denominator: 100,
                },
                context,
            )?;
            Ok(Some(text))
        }
        CssAuthoredColorComponent::ExactNumber(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                false,
                retain_explicit,
                context,
            )
        }
        CssAuthoredColorComponent::ExactPercentage(value) => {
            context.charge_input(1)?;
            alpha_literal(
                value.numeric().representation(),
                true,
                retain_explicit,
                context,
            )
        }
        CssAuthoredColorComponent::Number(value) => {
            context.charge_input(1)?;
            alpha_finite(value.value(), false, retain_explicit, context)
        }
        CssAuthoredColorComponent::Percentage(value) => {
            context.charge_input(1)?;
            alpha_finite(value.value(), true, retain_explicit, context)
        }
        CssAuthoredColorComponent::None => unreachable!("handled above"),
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
    let value = crate::opacity_scalar::ExactRational::from_lexical_factor_clamped_unit(
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

fn alpha_finite(
    value: f32,
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
    let value = crate::opacity_scalar::ExactRational::from_binary32_factor(
        value,
        exact_factor(factor),
        context,
    )?
    .clamp_unit(context)?;
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
    value: &CssAuthoredRgbColor,
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
    if value
        .channels()
        .iter()
        .any(CssAuthoredColorComponent::is_none)
    {
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
    value: &CssAuthoredHslColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let missing = matches!(value.hue(), CssAuthoredHue::None)
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
    hue: &crate::opacity_scalar::ExactRational,
    saturation: &crate::opacity_scalar::ExactRational,
    lightness: &crate::opacity_scalar::ExactRational,
    context: &mut SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    use crate::opacity_scalar::ExactRational as Exact;

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
    m1: &crate::opacity_scalar::ExactRational,
    m2: &crate::opacity_scalar::ExactRational,
    hue: &crate::opacity_scalar::ExactRational,
    offset: i64,
    context: &mut SpecifiedSerializationContext,
) -> Result<crate::opacity_scalar::ExactRational> {
    use crate::opacity_scalar::ExactRational as Exact;

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
    value: &CssAuthoredHwbColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let missing = matches!(value.hue(), CssAuthoredHue::None)
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
    hue: &crate::opacity_scalar::ExactRational,
    white: &crate::opacity_scalar::ExactRational,
    black: &crate::opacity_scalar::ExactRational,
    context: &mut SpecifiedSerializationContext,
) -> Result<[String; 3]> {
    use crate::opacity_scalar::ExactRational as Exact;

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
    value: &CssAuthoredHue,
    origin: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    use CssAuthoredHue as H;
    let (text, number, exact, contextual, missing, calculation) = match value {
        H::None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            ("none".into(), None, None, false, true, false)
        }
        H::ExactNumber(value) => {
            context.charge_input(1)?;
            let source = value.numeric().representation();
            let exact = if origin {
                crate::opacity_scalar::ExactRational::from_lexical_factor(
                    source,
                    exact_factor(Factor::ONE),
                    context,
                )?
            } else {
                crate::opacity_scalar::ExactRational::from_lexical_factor_modulo(
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
        H::ExactAngle(value) => {
            context.charge_input(1)?;
            let factor = angle_factor(value.unit());
            let source = value.numeric().representation();
            let exact = if origin {
                crate::opacity_scalar::ExactRational::from_lexical_factor(
                    source,
                    exact_factor(factor),
                    context,
                )?
            } else {
                crate::opacity_scalar::ExactRational::from_lexical_factor_modulo(
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
        H::Number(value) => {
            context.charge_input(1)?;
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value.value(),
                exact_factor(Factor::ONE),
                context,
            )?;
            let exact = if origin {
                exact
            } else {
                exact.modulo(360, context)?
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
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value.value(),
                exact_factor(factor),
                context,
            )?;
            let exact = if origin {
                exact
            } else {
                exact.modulo(360, context)?
            };
            let number = ScaledNumber::from_exact(&exact)?;
            let text = exact
                .clone_with_budget(context)?
                .format_exact(context.remaining_bytes(), context)?;
            (text, Some(number), Some(exact), false, false, false)
        }
        H::NumberCalculation(value) => {
            let (text, outcome) = crate::numeric::capture_calculation_specified(
                crate::numeric::SpecifiedCalculationRef::Number(value),
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
            let (text, outcome) = crate::numeric::capture_calculation_specified(
                crate::numeric::SpecifiedCalculationRef::Angle(value),
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

fn hwb_to_rgb(hue: f64, white: f64, black: f64) -> [f64; 3] {
    if white + black >= 1.0 {
        let gray = white / (white + black);
        return [gray; 3];
    }
    let mut rgb = hsl_to_rgb(hue, 1.0, 0.5);
    let factor = 1.0 - white - black;
    for channel in &mut rgb {
        *channel = *channel * factor + white;
    }
    rgb
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
    value: &CssAuthoredLabColor,
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
    value: &CssAuthoredLchColor,
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
    value: &CssAuthoredPredefinedColor,
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
    value: &CssAuthoredCustomColor,
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
    value: &CssTypedRelativeColorExpression,
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
        V::ExactNumber(value) => {
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
        V::ExactPercentage(value) => {
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
        V::ExactAngle(value) => {
            context.charge_input(1)?;
            let exact = crate::opacity_scalar::ExactRational::from_lexical_factor(
                value.numeric().representation(),
                exact_factor(angle_factor(value.unit())),
                context,
            )?;
            let text = exact.format_exact(context.remaining_bytes(), context)?;
            suffix_text(text, "deg", context)
        }
        V::Number(value) => {
            context.charge_input(1)?;
            if alpha {
                Ok(alpha_finite(value.value(), false, true, context)?
                    .expect("explicit relative alpha retained"))
            } else {
                format_finite(value.value(), context)
            }
        }
        V::Percentage(value) => {
            context.charge_input(1)?;
            if alpha {
                Ok(alpha_finite(value.value(), true, true, context)?
                    .expect("explicit relative alpha retained"))
            } else {
                let text = format_finite(value.value(), context)?;
                suffix_text(text, "%", context)
            }
        }
        V::Angle(value) => {
            context.charge_input(1)?;
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value.value(),
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
            let (text, _) = crate::numeric::capture_calculation_specified_scaled(
                crate::numeric::SpecifiedCalculationRef::Profile(value),
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
    value: &CssAuthoredColorInterpolation,
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
    components: &[CssAuthoredColorMixComponent],
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
                    let (text, _) = crate::numeric::capture_calculation_specified(
                        crate::numeric::SpecifiedCalculationRef::Percentage(calculation),
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
    let mut sum: Option<crate::opacity_scalar::ExactRational> = None;
    let mut omitted = 0usize;
    for component in components {
        let value = component
            .weight()
            .and_then(CssAuthoredColorMixWeight::literal_value)
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
    let sum = sum.unwrap_or(crate::opacity_scalar::ExactRational::from_lexical_factor(
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
    value: &CssAuthoredColorMixPercentage,
    context: &mut SpecifiedSerializationContext,
) -> Result<crate::opacity_scalar::ExactRational> {
    context.charge_input(1)?;
    if let Some(value) = value.exact_literal() {
        crate::opacity_scalar::ExactRational::from_lexical_factor(
            value.numeric().representation(),
            exact_factor(Factor::ONE),
            context,
        )
    } else {
        crate::opacity_scalar::ExactRational::from_binary32_factor(
            value.value().expect("finite mix weight"),
            exact_factor(Factor::ONE),
            context,
        )
    }
}

fn literal_weight(
    value: &CssAuthoredColorMixPercentage,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    exact_weight(value, context)?.format_exact(context.remaining_bytes(), context)
}

fn serialize_frozen_rgba(
    value: &CssRgbaColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    context.charge_input(3)?;
    context.charge_projection(3)?;
    let channels = [
        value.red().to_string(),
        value.green().to_string(),
        value.blue().to_string(),
    ];
    let alpha = frozen_alpha(Some(value.alpha()), context)?;
    if mode == Mode::Origin {
        let channels = channels.map(|text| ProjectedScalar {
            number: text.parse().ok().map(ScaledNumber::from_binary64),
            exact: None,
            text,
            contextual: false,
            missing: false,
            percentage: false,
            calculation: false,
        });
        modern_function("rgb", &channels, alpha.as_deref(), context)
    } else {
        legacy_rgb(&channels, alpha.as_deref(), context)
    }
}

fn frozen_slot(
    value: Option<f32>,
    percentage: bool,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    frozen_slot_scaled(value, percentage, Factor::ONE, context)
}

fn frozen_slot_scaled(
    value: Option<f32>,
    percentage: bool,
    factor: Factor,
    context: &mut SpecifiedSerializationContext,
) -> Result<ProjectedScalar> {
    context.charge_input(1)?;
    context.charge_projection(1)?;
    match value {
        Some(value) => {
            let exact = crate::opacity_scalar::ExactRational::from_binary32_factor(
                value,
                exact_factor(factor),
                context,
            )?;
            let number = ScaledNumber::from_exact(&exact)?;
            Ok(ProjectedScalar {
                text: exact
                    .clone_with_budget(context)?
                    .format_exact(context.remaining_bytes(), context)?,
                number: Some(number),
                exact: Some(exact),
                contextual: false,
                missing: false,
                percentage,
                calculation: false,
            })
        }
        None => Ok(ProjectedScalar {
            text: "none".into(),
            number: None,
            exact: None,
            contextual: false,
            missing: true,
            percentage: false,
            calculation: false,
        }),
    }
}

fn frozen_alpha(
    value: Option<f32>,
    context: &mut SpecifiedSerializationContext,
) -> Result<Option<String>> {
    match value {
        Some(value) => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            alpha_finite(value, false, false, context)
        }
        None => {
            context.charge_input(1)?;
            context.charge_projection(1)?;
            Ok(Some("none".into()))
        }
    }
}

fn serialize_frozen_hsl(
    value: &CssHslColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let hue = frozen_slot(value.hue(), false, context)?;
    let saturation = frozen_slot_scaled(
        value.saturation(),
        true,
        Factor {
            numerator: 100,
            denominator: 1,
        },
        context,
    )?;
    let lightness = frozen_slot_scaled(
        value.lightness(),
        true,
        Factor {
            numerator: 100,
            denominator: 1,
        },
        context,
    )?;
    let alpha = frozen_alpha(value.alpha(), context)?;
    if mode == Mode::Origin || hue.missing || saturation.missing || lightness.missing {
        return hsl_like(
            "hsl",
            &hue,
            &saturation,
            &lightness,
            alpha.as_deref(),
            context,
        );
    }
    let rgb = hsl_to_rgb(
        hue.number.expect("frozen numeric hue").binary64(),
        saturation
            .number
            .expect("frozen numeric saturation")
            .binary64()
            / 100.0,
        lightness
            .number
            .expect("frozen numeric lightness")
            .binary64()
            / 100.0,
    );
    let channels = rgb.map(|channel| rounded_f64(channel.clamp(0.0, 1.0) * 255.0, 6));
    legacy_rgb(&channels, alpha.as_deref(), context)
}

fn serialize_frozen_hwb(
    value: &CssHwbColor,
    mode: Mode,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let hue = frozen_slot(value.hue(), false, context)?;
    let white = frozen_slot_scaled(
        value.whiteness(),
        true,
        Factor {
            numerator: 100,
            denominator: 1,
        },
        context,
    )?;
    let black = frozen_slot_scaled(
        value.blackness(),
        true,
        Factor {
            numerator: 100,
            denominator: 1,
        },
        context,
    )?;
    let alpha = frozen_alpha(value.alpha(), context)?;
    if mode == Mode::Origin || hue.missing || white.missing || black.missing {
        return hsl_like("hwb", &hue, &white, &black, alpha.as_deref(), context);
    }
    let rgb = hwb_to_rgb(
        hue.number.expect("frozen numeric hue").binary64(),
        white.number.expect("frozen numeric whiteness").binary64() / 100.0,
        black.number.expect("frozen numeric blackness").binary64() / 100.0,
    );
    let channels = rgb.map(|channel| rounded_f64(channel.clamp(0.0, 1.0) * 255.0, 6));
    legacy_rgb(&channels, alpha.as_deref(), context)
}

fn serialize_frozen_lab(
    name: &str,
    value: &CssLabColor,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let light_scale = if name == "lab" {
        Factor::ONE
    } else {
        Factor {
            numerator: 100,
            denominator: 1,
        }
    };
    let channels = [
        frozen_slot_scaled(value.lightness(), false, light_scale, context)?,
        frozen_slot(value.a(), false, context)?,
        frozen_slot(value.b(), false, context)?,
    ];
    let alpha = frozen_alpha(value.alpha(), context)?;
    modern_function(name, &channels, alpha.as_deref(), context)
}

fn serialize_frozen_lch(
    name: &str,
    value: &CssLchColor,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let light_scale = if name == "lch" {
        Factor::ONE
    } else {
        Factor {
            numerator: 100,
            denominator: 1,
        }
    };
    let channels = [
        frozen_slot_scaled(value.lightness(), false, light_scale, context)?,
        frozen_slot(value.chroma(), false, context)?,
        frozen_slot(value.hue(), false, context)?,
    ];
    let alpha = frozen_alpha(value.alpha(), context)?;
    modern_function(name, &channels, alpha.as_deref(), context)
}

fn serialize_frozen_predefined(
    value: &CssColorFunction,
    context: &mut SpecifiedSerializationContext,
) -> Result<String> {
    let channels = value
        .components()
        .iter()
        .map(|value| frozen_slot(*value, false, context))
        .collect::<Result<Vec<_>>>()?;
    let alpha = frozen_alpha(value.alpha(), context)?;
    let name = format!("color({}", predefined_name(value.color_space()));
    modern_function(&name, &channels, alpha.as_deref(), context)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn legacy_expression(css: &str) -> CssColorComponentExpression {
        CssColorComponentExpression::new(
            CssAuthoredDeclarationValue::try_new(css).expect("nonempty expression"),
            Vec::new(),
        )
    }

    #[test]
    fn frozen_leaves_use_origin_punctuation_and_the_bounded_numeric_adapter() {
        let rgba = CssRgbaColor::try_new(1, 2, 3, 0.5).unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        assert_eq!(
            serialize_frozen_rgba(&rgba, Mode::Origin, &mut context).unwrap(),
            "rgb(1 2 3 / 0.5)"
        );

        let relative = CssRelativeColor::try_new(
            CssRelativeColorFunction::Rgb,
            CssColor::CurrentColor,
            vec![
                legacy_expression("calc(1 + 2)"),
                legacy_expression("g"),
                legacy_expression("b"),
            ],
            Some(legacy_expression("50%")),
        )
        .unwrap();
        let authored = CssAuthoredColor::preserved_i01(CssColor::Relative(relative));
        assert_eq!(
            serialize(&authored, Limits::default()).unwrap(),
            "rgb(from currentcolor calc(3) g b / 0.5)"
        );
    }

    #[test]
    fn frozen_default_mix_and_equal_weights_are_omitted() {
        let component =
            || CssColorMixComponent::try_new(CssColor::CurrentColor, Some(50.0)).unwrap();
        let mix = CssColorMix::new(
            CssColorInterpolationMethod::new(CssColorInterpolationSpace::Oklab, None),
            component(),
            component(),
        );
        let authored = CssAuthoredColor::preserved_i01(CssColor::ColorMix(mix));
        assert_eq!(
            serialize(&authored, Limits::default()).unwrap(),
            "color-mix(currentcolor, currentcolor)"
        );
    }

    #[test]
    fn frozen_scalars_are_counted_and_scaled_before_binary32_rounding() {
        let rgba = CssAuthoredColor::preserved_i01(CssColor::Rgba(
            CssRgbaColor::try_new(1, 2, 3, 1.0).unwrap(),
        ));
        assert_eq!(
            serialize(&rgba, Limits::new(6, usize::MAX, usize::MAX),).unwrap(),
            "rgb(1, 2, 3)"
        );
        assert_eq!(
            serialize(&rgba, Limits::new(5, usize::MAX, usize::MAX),)
                .unwrap_err()
                .kind(),
            crate::CssSpecifiedValueSerializationErrorKind::InputNodeLimit
        );

        let hsl = CssHslColor::try_new(Some(0.0), Some(0.3), Some(0.5), Some(1.0)).unwrap();
        let mut context = SpecifiedSerializationContext::new(Limits::default());
        assert_eq!(
            serialize_frozen_hsl(&hsl, Mode::Origin, &mut context).unwrap(),
            "hsl(0 30.0000011920928955078125% 50%)"
        );
    }
}
