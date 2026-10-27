use super::property_input::{Parser, Site};
use crate::color::CssColor;
use crate::component::{CssComponentValueRef, CssComponentValues, CssValueTokenRef};
use crate::diagnostic::error::property::ParseError;
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
    parse: impl for<'i, 't> FnOnce(
        &mut Parser<'i, 't>,
        &NumericInputContext<'_>,
    ) -> Result<T, ParseError<'i, Error>>
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
        let mut input = Parser::new(&document);
        input
            .parse_entirely(|input| parse(input, &NumericInputContext::Checked))
            .map_err(|error| {
                let (kind, origin) = error.into_parts();
                CssImageConstructionError::from(
                    crate::property::CssPropertyValueError::from_direct_grammar(kind, origin),
                )
            })
    })
}
pub(super) fn parse_image_1d<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssImage1D, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.state();
    let location = input.current_source_location();
    let original = input
        .collect_numeric_component(numeric)
        .map_err(|error| numeric_error(numeric, &error, location, start.position()))?;
    input.reset(&start);
    match input.next().map_err(basic)? {
        Token::Function(name) if name.eq_ignore_ascii_case("stripes") => {}
        token => return Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
    let stripes = input.parse_nested_block(|input| {
        input.parse_comma_separated(|input| {
            let mut color = None;
            let mut color_component = None;
            let mut thickness = None;
            while !input.is_exhausted() {
                if color.is_none() {
                    let state = input.state();
                    match input.try_parse(|input| super::color::parse_color(input, numeric)) {
                        Ok(value) => {
                            let after = input.state();
                            input.reset(&state);
                            let component =
                                input.collect_numeric_component(numeric).map_err(|error| {
                                    numeric_error(numeric, &error, location, state.position())
                                })?;
                            input.reset(&after);
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
                return Err(unexpected_at(input.current_source_location()));
            }
            let (Some(color), Some(color_component)) = (color, color_component) else {
                return Err(unexpected_at(input.current_source_location()));
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
) -> ParseError<'i, Error> {
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
fn parse_thickness<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssStripeThickness, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position();
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
