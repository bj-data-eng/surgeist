use super::property_input::{Site, Stream};
use crate::color::CssColor;
use crate::component::{CssComponentValueRef, CssComponentValues, CssValueTokenRef};
use crate::diagnostic::error::property::SyntaxError;
use crate::diagnostic::error::property::{Error, basic, unexpected_at};
use crate::numeric::expression::{CalculationRoot, NumericInputContext};
use crate::numeric::{CssCalculation, CssFlex, CssLengthPercentage, CssNumeric};
use crate::values::image::{
    CssImage1D, CssImageConstructionError, CssStripe, CssStripeLengthPercentage,
    CssStripeThickness, CssStripes,
};
use cssparser::Token;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::diagnostic::CssFeatureId] = &[
    crate::diagnostic::CssFeatureId::new("ext.value.image-1d"),
    crate::diagnostic::CssFeatureId::new("ext.value.stripes"),
];

pub(crate) fn checked_color(
    values: &CssComponentValues,
) -> Result<CssColor, CssImageConstructionError> {
    checked(values, |input, numeric| {
        super::color::parse_color(input, numeric)
    })
}
pub(crate) fn checked_image(
    values: &CssComponentValues,
) -> Result<CssImage1D, CssImageConstructionError> {
    checked(values, parse_image_1d)
}
fn checked<T: Send>(
    values: &CssComponentValues,
    parse: impl for<'i> FnOnce(
        &mut Stream<'i>,
        &NumericInputContext<'_>,
    ) -> Result<T, SyntaxError<'i, Error>>
    + Send,
) -> Result<T, CssImageConstructionError> {
    let refused = CssImageConstructionError::from;
    super::bounded_execution::components(values.items(), values.nesting_depth(), refused, || {
        let document = crate::syntax_consumption::normalize(
            crate::syntax_consumption::SyntaxInput::CheckedComponents(values.items()),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(CssImageConstructionError::from)?;
        let mut input = Stream::new(&document);
        input
            .parse_complete(|input| parse(input, &NumericInputContext::Checked))
            .map_err(|error| {
                let (kind, origin) = error.into_parts();
                CssImageConstructionError::from(
                    crate::property::CssPropertyValueError::from_direct_grammar(kind, origin),
                )
            })
    })
}
pub(super) fn parse_image_1d<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssImage1D, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let start = input.mark();
    let location = input.site();
    let original = input
        .collect_numeric_component(numeric)
        .map_err(|error| numeric_error(numeric, &error, location, start.site()))?;
    input.restore(start);
    match input.consume_after_whitespace().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("stripes") => {}
        token => return Err(location.unexpected::<Error>(token.clone())),
    }
    let stripes = input.within_block(|input| {
        input.comma_segments(|input| {
            let mut color = None;
            let mut color_component = None;
            let mut thickness = None;
            while !input.only_whitespace_remains() {
                if color.is_none() {
                    let state = input.mark();
                    match input.attempt(|input| super::color::parse_color(input, numeric)) {
                        Ok(value) => {
                            let after = input.mark();
                            input.restore(state);
                            let component =
                                input.collect_numeric_component(numeric).map_err(|error| {
                                    numeric_error(numeric, &error, location, state.site())
                                })?;
                            input.restore(after);
                            color = Some(value);
                            color_component = Some(component);
                            continue;
                        }
                        Err(error)
                            if crate::diagnostic::error::property::is_resource_parse_error(
                                &error,
                            ) =>
                        {
                            return Err(error);
                        }
                        Err(_) => {}
                    }
                }
                if thickness.is_none() {
                    thickness = Some(parse_thickness(input, numeric)?);
                    continue;
                }
                return Err(unexpected_at(input.site()));
            }
            let (Some(color), Some(color_component)) = (color, color_component) else {
                return Err(unexpected_at(input.site()));
            };
            Ok(CssStripe::from_parser(color, color_component, thickness))
        })
    })?;
    if stripes.is_empty() {
        return Err(unexpected_at(location));
    }
    Ok(CssImage1D::Stripes(CssStripes::from_parser(
        stripes, original,
    )))
}
fn numeric_error<'i>(
    numeric: &NumericInputContext<'_>,
    error: &crate::numeric::CssNumericConstructionError,
    location: Site<'i>,
    offset: Site<'i>,
) -> SyntaxError<'i, Error> {
    let location = numeric.property_error_location(error, location, offset);
    if let Some(component) = error.component_error()
        && crate::diagnostic::error::is_component_resource_error(component)
    {
        return crate::diagnostic::error::property::invalid_component_value(
            location,
            component.clone(),
        );
    }
    unexpected_at(location)
}
fn parse_thickness<'i>(
    input: &mut Stream<'i>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssStripeThickness, SyntaxError<'i, Error>> {
    input.discard_whitespace();
    let location = input.site();
    let offset = input.site();
    let component = input
        .collect_component()
        .map_err(crate::numeric::CssNumericConstructionError::component)
        .map_err(|error| numeric_error(numeric, &error, location, offset))?;
    if matches!(component.view(), CssComponentValueRef::Function(_)) {
        let values = crate::numeric::expression::CalculationComponents::single(component).map_err(
            |error| crate::diagnostic::error::property::invalid_component_value(location, error),
        )?;
        let values = match numeric.admit_or_return_input(values, CalculationRoot::Flex) {
            Ok(value) => {
                return CssNumeric::<CssFlex>::from_parser_calculation(
                    CssCalculation::<CssFlex>::from_expression(value),
                )
                .map(CssStripeThickness::Flex)
                .map_err(|error| numeric_error(numeric, &error, location, offset));
            }
            Err(failure)
                if failure
                    .error
                    .component_error()
                    .is_some_and(crate::diagnostic::error::is_component_resource_error) =>
            {
                return Err(numeric_error(numeric, &failure.error, location, offset));
            }
            Err(failure) => failure.values,
        };
        let expression = numeric
            .admit(values, CalculationRoot::LengthPercentage)
            .map_err(|error| numeric_error(numeric, &error, location, offset))?;
        return CssStripeLengthPercentage::from_parser_calculation(CssCalculation::<
            CssLengthPercentage,
        >::from_expression(
            expression
        ))
        .map(CssStripeThickness::LengthPercentage)
        .map_err(|error| numeric_error(numeric, &error, location, offset));
    }
    let component = component.into_owned();
    if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("fr"))
    {
        CssNumeric::<CssFlex>::try_from_component(component)
            .map(CssStripeThickness::Flex)
            .map_err(|error| numeric_error(numeric, &error, location, offset))
    } else {
        CssStripeLengthPercentage::try_from_component(component)
            .map(CssStripeThickness::LengthPercentage)
            .map_err(|error| numeric_error(numeric, &error, location, offset))
    }
}
