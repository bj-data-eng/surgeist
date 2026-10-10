use crate::error::{Error, basic, unexpected_at};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::{
    CssColor, CssComponentValueRef, CssComponentValues, CssFlexCalculation, CssImage1D,
    CssImage1DConstructionError, CssLengthPercentageCalculation, CssSpecifiedFlex, CssStripe,
    CssStripeLengthPercentage, CssStripeThickness, CssStripes, CssValueTokenRef,
};
use cssparser::{ParseError, Parser, ParserInput, Token};

pub(super) static IMPLEMENTED_SHARED_VALUES: &[crate::CssFeatureId] = &[
    crate::CssFeatureId::new("ext.value.image-1d"),
    crate::CssFeatureId::new("ext.value.stripes"),
];

pub(crate) fn checked_color(
    values: &CssComponentValues,
) -> Result<CssColor, CssImage1DConstructionError> {
    let serialized = values
        .serialize()
        .map_err(CssImage1DConstructionError::Component)?;
    super::fragments::bounded_execution(serialized.as_css(), || {
        let numeric = NumericInputContext::components(values, &serialized);
        let mut input = ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        parser
            .parse_entirely(|input| super::color::parse_color(input, &numeric))
            .map_err(|error| {
                CssImage1DConstructionError::Grammar(
                    crate::CssPropertyValueParseError::from_grammar(
                        crate::error::from_parse_error(serialized.as_css(), error),
                        &serialized,
                    ),
                )
            })
    })
}
pub(crate) fn checked_image(
    values: &CssComponentValues,
) -> Result<CssImage1D, CssImage1DConstructionError> {
    let serialized = values
        .serialize()
        .map_err(CssImage1DConstructionError::Component)?;
    super::fragments::bounded_execution(serialized.as_css(), || {
        let numeric = NumericInputContext::components(values, &serialized);
        let mut input = ParserInput::new(serialized.as_css());
        let mut parser = Parser::new(&mut input);
        parser
            .parse_entirely(|input| parse_image_1d(input, &numeric))
            .map_err(|error| {
                CssImage1DConstructionError::Grammar(
                    crate::CssPropertyValueParseError::from_grammar(
                        crate::error::from_parse_error(serialized.as_css(), error),
                        &serialized,
                    ),
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
    let original = numeric
        .collect(input)
        .map_err(|error| numeric_error(numeric, &error, location, start.position().byte_index()))?;
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
                            let component = numeric.collect(input).map_err(|error| {
                                numeric_error(
                                    numeric,
                                    &error,
                                    location,
                                    state.position().byte_index(),
                                )
                            })?;
                            input.reset(&after);
                            color = Some(value);
                            color_component = Some(component);
                            continue;
                        }
                        Err(error) if crate::error::is_resource_parse_error(&error) => {
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
    location: cssparser::SourceLocation,
    offset: usize,
) -> ParseError<'i, Error> {
    let location = numeric.error_location(error, location, offset);
    if let Some(component) = error.component_error()
        && crate::error::is_component_resource_error(component)
    {
        return crate::error::invalid_component_value(location, component.clone());
    }
    unexpected_at(location)
}
fn parse_thickness<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssStripeThickness, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric
        .collect(input)
        .map_err(|error| numeric_error(numeric, &error, location, offset))?;
    if matches!(component.view(), CssComponentValueRef::Function(_)) {
        let values = CssComponentValues::try_new(vec![component])
            .map_err(|error| crate::error::invalid_component_value(location, error))?;
        let flex = numeric.admit(values.clone(), CalculationRoot::Flex);
        match flex {
            Ok(value) => {
                return CssSpecifiedFlex::from_parser_calculation(
                    CssFlexCalculation::from_expression(value),
                )
                .map(CssStripeThickness::Flex)
                .map_err(|error| numeric_error(numeric, &error, location, offset));
            }
            Err(error)
                if error
                    .component_error()
                    .is_some_and(crate::error::is_component_resource_error) =>
            {
                return Err(numeric_error(numeric, &error, location, offset));
            }
            Err(_) => {}
        }
        let expression = numeric
            .admit(values, CalculationRoot::LengthPercentage)
            .map_err(|error| numeric_error(numeric, &error, location, offset))?;
        return CssStripeLengthPercentage::from_parser_calculation(
            CssLengthPercentageCalculation::from_expression(expression),
        )
        .map(CssStripeThickness::LengthPercentage)
        .map_err(|error| numeric_error(numeric, &error, location, offset));
    }
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
