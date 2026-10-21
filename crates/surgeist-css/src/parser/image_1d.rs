use super::property_input::{Parser, Site};
use crate::error::property::ParseError;
use crate::error::property::{Error, basic, unexpected_at};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::{
    CssColor, CssComponentValueRef, CssComponentValues, CssFlexCalculation, CssImage1D,
    CssImage1DConstructionError, CssLengthPercentageCalculation, CssSpecifiedFlex, CssStripe,
    CssStripeLengthPercentage, CssStripeThickness, CssStripes, CssValueTokenRef,
};
use cssparser::Token;

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] = &[
    crate::CssFeatureId::new("ext.value.image-1d"),
    crate::CssFeatureId::new("ext.value.stripes"),
];

pub(crate) fn checked_color(
    values: &CssComponentValues,
) -> Result<CssColor, CssImage1DConstructionError> {
    checked(values, |input, numeric| {
        super::color::parse_color(input, numeric)
    })
}
pub(crate) fn checked_image(
    values: &CssComponentValues,
) -> Result<CssImage1D, CssImage1DConstructionError> {
    checked(values, parse_image_1d)
}
fn checked<T: Send>(
    values: &CssComponentValues,
    parse: impl for<'i, 't> FnOnce(
        &mut Parser<'i, 't>,
        &NumericInputContext<'_>,
    ) -> Result<T, ParseError<'i, Error>>
    + Send,
) -> Result<T, CssImage1DConstructionError> {
    let refused = CssImage1DConstructionError::Component;
    super::bounded_execution::components(values.items(), values.nesting_depth(), refused, || {
        let document = crate::syntax_consumption::normalize(
            crate::syntax_consumption::SyntaxInput::CheckedComponents(values.items()),
            crate::syntax_consumption::SyntaxInputLimits::default(),
            0,
        )
        .map_err(CssImage1DConstructionError::Component)?;
        let mut input = Parser::new(&document);
        input
            .parse_entirely(|input| parse(input, &NumericInputContext::Checked))
            .map_err(|error| {
                let (kind, origin) = error.into_parts();
                CssImage1DConstructionError::Grammar(
                    crate::CssPropertyValueParseError::from_direct_grammar(kind, origin),
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
                        Err(error) if crate::error::property::is_resource_parse_error(&error) => {
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
    error: &crate::CssNumericConstructionError,
    location: Site<'i>,
    offset: Site<'i>,
) -> ParseError<'i, Error> {
    let location = numeric.property_error_location(error, location, offset);
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::property::invalid_component_value(location, component.clone());
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
        .map_err(crate::CssNumericConstructionError::component)
        .map_err(|error| numeric_error(numeric, &error, location, offset))?;
    if matches!(component.view(), CssComponentValueRef::Function(_)) {
        let values = crate::numeric::CalculationComponents::single(component)
            .map_err(|error| crate::error::property::invalid_component_value(location, error))?;
        let values = match numeric.admit_or_return_input(values, CalculationRoot::Flex) {
            Ok(value) => {
                return CssSpecifiedFlex::from_parser_calculation(
                    CssFlexCalculation::from_expression(value),
                )
                .map(CssStripeThickness::Flex)
                .map_err(|error| numeric_error(numeric, &error, location, offset));
            }
            Err(failure)
                if failure
                    .error
                    .component_error()
                    .is_some_and(crate::error::is_component_resource_error) =>
            {
                return Err(numeric_error(numeric, &failure.error, location, offset));
            }
            Err(failure) => failure.values,
        };
        let expression = numeric
            .admit(values, CalculationRoot::LengthPercentage)
            .map_err(|error| numeric_error(numeric, &error, location, offset))?;
        return CssStripeLengthPercentage::from_parser_calculation(
            CssLengthPercentageCalculation::from_expression(expression),
        )
        .map(CssStripeThickness::LengthPercentage)
        .map_err(|error| numeric_error(numeric, &error, location, offset));
    }
    let component = component.into_owned();
    if matches!(component.view(), CssComponentValueRef::Token(CssValueTokenRef::Dimension { unit, .. }) if unit.eq_ignore_ascii_case("fr"))
    {
        CssSpecifiedFlex::try_from_component(component)
            .map(CssStripeThickness::Flex)
            .map_err(|error| numeric_error(numeric, &error, location, offset))
    } else {
        CssStripeLengthPercentage::try_from_component(component)
            .map(CssStripeThickness::LengthPercentage)
            .map_err(|error| numeric_error(numeric, &error, location, offset))
    }
}
