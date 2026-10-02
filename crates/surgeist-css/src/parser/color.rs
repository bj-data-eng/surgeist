use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use super::values::{calculation_error, parse_numeric_function};
use crate::error::{Error, basic, invalid_color, unsupported_value_at, with_color_context};
use crate::numeric::{CalculationRoot, NumericInputContext};
use crate::syntax::*;

pub(super) fn parse_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    if next_is_authored_relative_color(input) {
        return parse_authored_relative_color(input, numeric)
            .map_err(|error| with_color_context(error, None));
    }
    if next_is_color_mix(input) {
        return parse_authored_color_mix(input, numeric)
            .map_err(|error| with_color_context(error, None));
    }
    parse_selected_authored_color(input, numeric).map_err(|error| with_color_context(error, None))
}

fn next_is_color_mix<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_color_mix = matches!(
        input.next(),
        Ok(Token::Function(name)) if name.eq_ignore_ascii_case("color-mix")
    );
    input.reset(&state);
    is_color_mix
}

fn next_is_authored_relative_color<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_relative = match input.next() {
        Ok(Token::Function(name)) if relative_color_function_from_name(name).is_some() => input
            .parse_nested_block(|input| {
                input.expect_ident_matching("from").map_err(basic)?;
                while input.next_including_whitespace().is_ok() {}
                Ok(())
            })
            .is_ok(),
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    is_relative
}

fn parse_selected_authored_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let start = input.position().byte_index();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("contrast-color") => input
            .parse_nested_block(|input| {
                let color = parse_color(input, numeric)?;
                input.expect_exhausted().map_err(basic)?;
                CssContrastColor::try_new(color)
                    .map(CssColor::from_contrast_color)
                    .map_err(|error| {
                        let kind = match error {
                            CssColorConstructionError::CapacityOverflow => {
                                crate::CssComponentValueErrorKind::CapacityOverflow
                            }
                            _ => crate::CssComponentValueErrorKind::NestingLimit,
                        };
                        crate::error::invalid_component_value(
                            location,
                            crate::CssComponentValueError::new(
                                kind,
                                numeric.origin_at(start).expect("color function origin"),
                            ),
                        )
                    })
            }),
        Token::Function(name) if name.eq_ignore_ascii_case("light-dark") => input
            .parse_nested_block(|input| {
                let light = parse_color(input, numeric)?;
                input.expect_comma().map_err(basic)?;
                let dark = parse_color(input, numeric)?;
                input.expect_exhausted().map_err(basic)?;
                CssLightDarkColor::try_new(light, dark)
                    .map(CssColor::from_light_dark)
                    .map_err(|error| {
                        let kind = match error {
                            CssColorConstructionError::CapacityOverflow => {
                                crate::CssComponentValueErrorKind::CapacityOverflow
                            }
                            _ => crate::CssComponentValueErrorKind::NestingLimit,
                        };
                        crate::error::invalid_component_value(
                            location,
                            crate::CssComponentValueError::new(
                                kind,
                                numeric.origin_at(start).expect("color function origin"),
                            ),
                        )
                    })
            }),
        Token::Ident(ident) if ident.eq_ignore_ascii_case("currentcolor") => {
            Ok(CssColor::current_color())
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("transparent") => {
            Ok(CssColor::transparent())
        }
        Token::Ident(ident) => {
            if let Some(system) = authored_system_color(&ident) {
                return Ok(CssColor::from_system(system));
            }
            if let Some(name) = CssNamedColor::try_new(ident.as_ref()) {
                return Ok(CssColor::from_named(name));
            }
            Err(invalid_color(location, None))
        }
        Token::Hash(digits) | Token::IDHash(digits)
            if matches!(digits.len(), 3 | 4 | 6 | 8)
                && digits.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            Ok(CssColor::from_hex(
                CssHexColor::try_new(digits.as_ref()).expect("checked hex token"),
            ))
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") =>
        {
            input
                .parse_nested_block(|input| parse_authored_rgb(input, numeric))
                .map(CssColor::from_rgb)
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") =>
        {
            input
                .parse_nested_block(|input| parse_authored_hsl(input, numeric))
                .map(CssColor::from_hsl)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("hwb") => input
            .parse_nested_block(|input| parse_authored_hwb(input, numeric))
            .map(CssColor::from_hwb),
        Token::Function(name) if name.eq_ignore_ascii_case("lab") => input
            .parse_nested_block(|input| parse_authored_lab(input, numeric))
            .map(CssColor::from_lab),
        Token::Function(name) if name.eq_ignore_ascii_case("lch") => input
            .parse_nested_block(|input| parse_authored_lch(input, numeric))
            .map(CssColor::from_lch),
        Token::Function(name) if name.eq_ignore_ascii_case("oklab") => input
            .parse_nested_block(|input| parse_authored_lab(input, numeric))
            .map(CssColor::from_oklab),
        Token::Function(name) if name.eq_ignore_ascii_case("oklch") => input
            .parse_nested_block(|input| parse_authored_lch(input, numeric))
            .map(CssColor::from_oklch),
        Token::Function(name) if name.eq_ignore_ascii_case("color") => {
            input.parse_nested_block(|input| {
                if let Ok(profile) = input.try_parse(|input| {
                    let name = input.expect_ident().map_err(basic)?;
                    CssColorProfileName::try_new(name.as_ref())
                        .ok_or_else(|| invalid_color(location, None))
                }) {
                    parse_authored_custom_arguments(input, numeric, profile)
                } else {
                    parse_authored_predefined_color(input, numeric).map(CssColor::from_predefined)
                }
            })
        }
        Token::Function(name) if name.eq_ignore_ascii_case("alpha") => {
            input.parse_nested_block(|input| {
                input.expect_ident_matching("from").map_err(basic)?;
                let source = parse_color(input, numeric)?;
                let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    Some(parse_typed_relative_color_expression(
                        input,
                        numeric,
                        CssRelativeColorEnvironment::Alpha,
                        CssRelativeColorResultDomain::Alpha,
                    )?)
                } else {
                    None
                };
                input.expect_exhausted().map_err(basic)?;
                CssAlphaColor::try_new(source, alpha)
                    .map(CssColor::from_alpha)
                    .map_err(|_| invalid_color(location, None))
            })
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            None,
        )),
    }
}

fn parse_authored_rgb<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssRgbColor, ParseError<'i, Error>> {
    let first = parse_authored_color_component(input, numeric, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if first.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("red")));
        }
        let domain = first.domain();
        let second_location = input.current_source_location();
        let second = parse_authored_color_component(input, numeric, false)?;
        if second.domain() != domain {
            return Err(invalid_color(second_location, Some("component")));
        }
        input.expect_comma().map_err(basic)?;
        let third_location = input.current_source_location();
        let third = parse_authored_color_component(input, numeric, false)?;
        if third.domain() != domain {
            return Err(invalid_color(third_location, Some("component")));
        }
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, numeric, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        CssRgbColor::try_new(CssColorSyntax::Legacy, [first, second, third], alpha)
            .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
    } else {
        let second = parse_authored_color_component(input, numeric, true)?;
        let third = parse_authored_color_component(input, numeric, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, numeric, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        CssRgbColor::try_new(CssColorSyntax::Modern, [first, second, third], alpha)
            .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
    }
}

fn parse_authored_hsl<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssHslColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, numeric, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if hue.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("hue")));
        }
        let saturation = parse_authored_percentage_component(input, numeric, false)?;
        input.expect_comma().map_err(basic)?;
        let lightness = parse_authored_percentage_component(input, numeric, false)?;
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, numeric, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        CssHslColor::try_new(CssColorSyntax::Legacy, hue, saturation, lightness, alpha)
            .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
    } else {
        let saturation = parse_authored_color_component(input, numeric, true)?;
        let lightness = parse_authored_color_component(input, numeric, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, numeric, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        CssHslColor::try_new(CssColorSyntax::Modern, hue, saturation, lightness, alpha)
            .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
    }
}

fn parse_authored_hwb<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssHwbColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, numeric, true)?;
    let whiteness = parse_authored_color_component(input, numeric, true)?;
    let blackness = parse_authored_color_component(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    CssHwbColor::try_new(hue, whiteness, blackness, alpha)
        .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
}

fn parse_authored_lab<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLabColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, numeric, true)?;
    let a = parse_authored_color_component(input, numeric, true)?;
    let b = parse_authored_color_component(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    CssLabColor::try_new(lightness, a, b, alpha)
        .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
}

fn parse_authored_lch<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLchColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, numeric, true)?;
    let chroma = parse_authored_color_component(input, numeric, true)?;
    let hue = parse_authored_hue(input, numeric, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    CssLchColor::try_new(lightness, chroma, hue, alpha)
        .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
}

fn parse_authored_custom_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    profile: CssColorProfileName,
) -> Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let mut channels = Vec::new();
    let mut alpha = None;
    while !input.is_exhausted() {
        if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            alpha = Some(parse_authored_color_component(input, numeric, true)?);
            break;
        }
        channels.push(parse_authored_color_component(input, numeric, true)?);
    }
    input.expect_exhausted().map_err(basic)?;
    CssCustomColor::try_new(profile, channels, alpha)
        .map(CssColor::from_custom)
        .map_err(|_| invalid_color(location, None))
}
fn parse_profile_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssProfileColorExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let offset = input.position().byte_index();
    let location = input.current_source_location();
    let component = numeric
        .collect(input)
        .map_err(|error| calculation_error(numeric.error_location(&error, location, offset)))?;
    let values = crate::CssComponentValues::try_new(vec![component])
        .map_err(|_| calculation_error(location))?;
    crate::CssProfileColorExpression::from_parser_components(values, numeric)
        .map_err(|error| calculation_error(numeric.error_location(&error, location, offset)))
}

fn parse_authored_predefined_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssPredefinedColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color_space = match_ignore_ascii_case! { &ident,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
        "display-p3-linear" => CssPredefinedColorSpace::DisplayP3Linear,
        "a98-rgb" => CssPredefinedColorSpace::A98Rgb,
        "prophoto-rgb" => CssPredefinedColorSpace::ProphotoRgb,
        "rec2020" => CssPredefinedColorSpace::Rec2020,
        "xyz" | "xyz-d65" => CssPredefinedColorSpace::XyzD65,
        "xyz-d50" => CssPredefinedColorSpace::XyzD50,
        _ => {
            return Err(with_color_context(
                location.new_unexpected_token_error::<Error>(Token::Ident(ident)),
                Some("color space"),
            ));
        }
    };
    let channels = [
        parse_authored_color_component(input, numeric, true)?,
        parse_authored_color_component(input, numeric, true)?,
        parse_authored_color_component(input, numeric, true)?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, numeric, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    CssPredefinedColor::try_new(color_space, channels, alpha)
        .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
}

fn parse_authored_color_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssColorComponent, ParseError<'i, Error>> {
    let before_opener = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssColorComponent::None)
        }
        Token::Number { .. } | Token::Percentage { .. } => {
            input.reset(&before_opener);
            let component = collect_color_scalar(input, numeric)?;
            crate::color_scalar::component(component)
                .map_err(|error| crate::error::invalid_component_value(location, error))
        }
        Token::Function(name) if is_math_function(&name) => {
            parse_authored_number_or_percentage_calculation(
                input,
                numeric,
                &before_opener,
                location,
            )
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("component"),
        )),
    }
}

fn parse_authored_percentage_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssColorComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_authored_color_component(input, numeric, allow_none)?;
    if matches!(
        value,
        CssColorComponent::None
            | CssColorComponent::Percentage(_)
            | CssColorComponent::PercentageCalculation(_)
    ) {
        Ok(value)
    } else {
        Err(invalid_color(location, Some("percentage")))
    }
}

fn parse_authored_alpha<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssColorComponent, ParseError<'i, Error>> {
    parse_authored_color_component(input, numeric, allow_none)
}

fn parse_authored_number_or_percentage_calculation<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    before_opener: &cssparser::ParserState,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssColorComponent, ParseError<'i, Error>> {
    let expression = parse_numeric_function(
        input,
        before_opener,
        numeric,
        CalculationRoot::NumberPercentage,
    )?;
    match expression.result_type() {
        CssCalculationType::Number => Ok(CssColorComponent::NumberCalculation(
            CssNumberCalculation::from_expression(expression),
        )),
        CssCalculationType::Percentage => Ok(CssColorComponent::PercentageCalculation(
            CssPercentageCalculation::from_expression(expression),
        )),
        _ => Err(invalid_color(location, Some("component"))),
    }
}

fn parse_authored_hue<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    allow_none: bool,
) -> std::result::Result<CssColorHue, ParseError<'i, Error>> {
    let before_opener = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssColorHue::None)
        }
        token @ (Token::Number { .. } | Token::Dimension { .. }) => {
            input.reset(&before_opener);
            let component = collect_color_scalar(input, numeric)?;
            crate::color_scalar::hue(component).map_err(|_| {
                with_color_context(
                    location.new_unexpected_token_error::<Error>(token),
                    Some("hue"),
                )
            })
        }
        Token::Function(name) if is_math_function(&name) => {
            if let Ok(expression) = input.try_parse(|input| {
                parse_numeric_function(input, &before_opener, numeric, CalculationRoot::Number)
            }) {
                return Ok(CssColorHue::NumberCalculation(
                    CssNumberCalculation::from_expression(expression),
                ));
            }
            parse_numeric_function(input, &before_opener, numeric, CalculationRoot::Angle)
                .map(CssAngleCalculation::from_expression)
                .map(CssColorHue::AngleCalculation)
                .map_err(|_| invalid_color(location, Some("hue")))
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("hue"),
        )),
    }
}

fn authored_system_color(ident: &str) -> Option<CssSystemColor> {
    let value = match_ignore_ascii_case! { ident,
        "canvas" => CssSystemColor::Canvas,
        "canvastext" => CssSystemColor::CanvasText,
        "linktext" => CssSystemColor::LinkText,
        "visitedtext" => CssSystemColor::VisitedText,
        "activetext" => CssSystemColor::ActiveText,
        "buttonface" => CssSystemColor::ButtonFace,
        "buttontext" => CssSystemColor::ButtonText,
        "buttonborder" => CssSystemColor::ButtonBorder,
        "field" => CssSystemColor::Field,
        "fieldtext" => CssSystemColor::FieldText,
        "highlight" => CssSystemColor::Highlight,
        "highlighttext" => CssSystemColor::HighlightText,
        "mark" => CssSystemColor::Mark,
        "marktext" => CssSystemColor::MarkText,
        "graytext" => CssSystemColor::GrayText,
        "selecteditem" => CssSystemColor::SelectedItem,
        "selecteditemtext" => CssSystemColor::SelectedItemText,
        "accentcolor" => CssSystemColor::AccentColor,
        "accentcolortext" => CssSystemColor::AccentColorText,
        "activeborder" => CssSystemColor::ActiveBorder,
        "activecaption" => CssSystemColor::ActiveCaption,
        "appworkspace" => CssSystemColor::AppWorkspace,
        "background" => CssSystemColor::Background,
        "buttonhighlight" => CssSystemColor::ButtonHighlight,
        "buttonshadow" => CssSystemColor::ButtonShadow,
        "captiontext" => CssSystemColor::CaptionText,
        "inactiveborder" => CssSystemColor::InactiveBorder,
        "inactivecaption" => CssSystemColor::InactiveCaption,
        "inactivecaptiontext" => CssSystemColor::InactiveCaptionText,
        "infobackground" => CssSystemColor::InfoBackground,
        "infotext" => CssSystemColor::InfoText,
        "menu" => CssSystemColor::Menu,
        "menutext" => CssSystemColor::MenuText,
        "scrollbar" => CssSystemColor::Scrollbar,
        "threeddarkshadow" => CssSystemColor::ThreeDDarkShadow,
        "threedface" => CssSystemColor::ThreeDFace,
        "threedhighlight" => CssSystemColor::ThreeDHighlight,
        "threedlightshadow" => CssSystemColor::ThreeDLightShadow,
        "threedshadow" => CssSystemColor::ThreeDShadow,
        "window" => CssSystemColor::Window,
        "windowframe" => CssSystemColor::WindowFrame,
        "windowtext" => CssSystemColor::WindowText,
        _ => return None,
    };
    Some(value)
}

fn parse_authored_relative_color<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let Token::Function(name) = token else {
        return Err(location.new_unexpected_token_error(token));
    };
    let Some(function) = relative_color_function_from_name(&name) else {
        return Err(location.new_unexpected_token_error(Token::Function(name)));
    };
    input.parse_nested_block(|input| {
        parse_authored_relative_color_arguments(input, numeric, function)
    })
}

fn parse_authored_relative_color_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    function: RelativeColorFunction,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    input.expect_ident_matching("from").map_err(basic)?;
    let source = parse_color(input, numeric)?;
    if matches!(function, RelativeColorFunction::Color) {
        let location = input.current_source_location();
        if let Ok(profile) = input.try_parse(|input| {
            let name = input.expect_ident().map_err(basic)?;
            CssColorProfileName::try_new(name.as_ref()).ok_or_else(|| invalid_color(location, None))
        }) {
            let mut channels = Vec::new();
            let mut alpha = None;
            while !input.is_exhausted() {
                if input.try_parse(|input| input.expect_delim('/')).is_ok() {
                    alpha = Some(parse_profile_expression(input, numeric)?);
                    break;
                }
                channels.push(parse_profile_expression(input, numeric)?);
            }
            input.expect_exhausted().map_err(basic)?;
            return CssRelativeCustomColor::try_new(source, profile, channels, alpha)
                .map(CssColor::from_relative_custom)
                .map_err(|_| invalid_color(location, None));
        }
    }
    let (function, environment, domains) = relative_color_signature(input, function)?;
    let channels = [
        parse_typed_relative_color_expression(input, numeric, environment, domains[0])?,
        parse_typed_relative_color_expression(input, numeric, environment, domains[1])?,
        parse_typed_relative_color_expression(input, numeric, environment, domains[2])?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_typed_relative_color_expression(
            input,
            numeric,
            environment,
            CssRelativeColorResultDomain::Alpha,
        )?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    CssRelativeColor::try_new(function, source, channels, alpha)
        .map(CssColor::from_relative)
        .map_err(|_| invalid_color(input.current_source_location(), Some("component")))
}

fn relative_color_signature<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: RelativeColorFunction,
) -> std::result::Result<
    (
        CssRelativeColorFunction,
        CssRelativeColorEnvironment,
        [CssRelativeColorResultDomain; 3],
    ),
    ParseError<'i, Error>,
> {
    let function = match function {
        RelativeColorFunction::Rgb => CssRelativeColorFunction::Rgb,
        RelativeColorFunction::Hsl => CssRelativeColorFunction::Hsl,
        RelativeColorFunction::Hwb => CssRelativeColorFunction::Hwb,
        RelativeColorFunction::Lab => CssRelativeColorFunction::Lab,
        RelativeColorFunction::Lch => CssRelativeColorFunction::Lch,
        RelativeColorFunction::Oklab => CssRelativeColorFunction::Oklab,
        RelativeColorFunction::Oklch => CssRelativeColorFunction::Oklch,
        RelativeColorFunction::Color => {
            CssRelativeColorFunction::Color(parse_relative_predefined_color_space(input)?)
        }
    };
    let (environment, domains) = function.signature();
    Ok((function, environment, domains))
}

fn parse_typed_relative_color_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
) -> std::result::Result<CssRelativeColorExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let offset = input.position().byte_index();
    let location = input.current_source_location();
    let component = numeric.collect(input).map_err(|error| {
        let at = numeric.error_location(&error, location, offset);
        calculation_error(at)
    })?;
    let values = crate::CssComponentValues::try_new(vec![component])
        .map_err(|error| crate::error::invalid_component_value(location, error))?;
    CssRelativeColorExpression::from_parser_components(values, numeric, environment, result_domain)
        .map_err(|error| {
            let at = numeric.error_location(&error, location, offset);
            if let Some(component) = error.component_error() {
                crate::error::invalid_component_value(at, component.clone())
            } else {
                invalid_color(at, Some("relative channel"))
            }
        })
}

pub(crate) fn numeric_relative_channel(
    environment: CssRelativeColorEnvironment,
    name: &str,
) -> Option<(CssRelativeColorChannel, CssCalculationType)> {
    let channel = relative_color_channel(environment, name)?;
    Some((channel, relative_channel_type(environment, channel)))
}

fn relative_color_channel(
    environment: CssRelativeColorEnvironment,
    ident: &str,
) -> Option<CssRelativeColorChannel> {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    let channel = match environment {
        CssRelativeColorEnvironment::Alpha => {
            if ident.eq_ignore_ascii_case("alpha") {
                Alpha
            } else {
                return None;
            }
        }
        CssRelativeColorEnvironment::Rgb | CssRelativeColorEnvironment::PredefinedRgb(_) => {
            match_ignore_ascii_case! { ident,
                "r" => R,
                "g" => G,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Hsl => match_ignore_ascii_case! { ident,
            "h" => H,
            "s" => S,
            "l" => L,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Hwb => match_ignore_ascii_case! { ident,
            "h" => H,
            "w" => W,
            "b" => B,
            "alpha" => Alpha,
            _ => return None,
        },
        CssRelativeColorEnvironment::Lab | CssRelativeColorEnvironment::Oklab => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "a" => A,
                "b" => B,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Lch | CssRelativeColorEnvironment::Oklch => {
            match_ignore_ascii_case! { ident,
                "l" => L,
                "c" => C,
                "h" => H,
                "alpha" => Alpha,
                _ => return None,
            }
        }
        CssRelativeColorEnvironment::Xyz(_) => match_ignore_ascii_case! { ident,
            "x" => X,
            "y" => Y,
            "z" => Z,
            "alpha" => Alpha,
            _ => return None,
        },
    };
    Some(channel)
}

fn relative_channel_type(
    _environment: CssRelativeColorEnvironment,
    _channel: CssRelativeColorChannel,
) -> CssCalculationType {
    CssCalculationType::Number
}

#[derive(Clone, Copy)]
enum RelativeColorFunction {
    Rgb,
    Hsl,
    Hwb,
    Lab,
    Lch,
    Oklab,
    Oklch,
    Color,
}

fn relative_color_function_from_name(name: &str) -> Option<RelativeColorFunction> {
    let function = match_ignore_ascii_case! { name,
        "rgb" | "rgba" => RelativeColorFunction::Rgb,
        "hsl" | "hsla" => RelativeColorFunction::Hsl,
        "hwb" => RelativeColorFunction::Hwb,
        "lab" => RelativeColorFunction::Lab,
        "lch" => RelativeColorFunction::Lch,
        "oklab" => RelativeColorFunction::Oklab,
        "oklch" => RelativeColorFunction::Oklch,
        "color" => RelativeColorFunction::Color,
        _ => return None,
    };
    Some(function)
}

fn parse_relative_predefined_color_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssPredefinedColorSpace, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color_space = match_ignore_ascii_case! { &ident,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
        "display-p3-linear" => CssPredefinedColorSpace::DisplayP3Linear,
        "a98-rgb" => CssPredefinedColorSpace::A98Rgb,
        "prophoto-rgb" => CssPredefinedColorSpace::ProphotoRgb,
        "rec2020" => CssPredefinedColorSpace::Rec2020,
        "xyz" => CssPredefinedColorSpace::XyzD65,
        "xyz-d50" => CssPredefinedColorSpace::XyzD50,
        "xyz-d65" => CssPredefinedColorSpace::XyzD65,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color space `{ident}`"),
        )),
    };
    Ok(color_space)
}

fn parse_authored_color_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("color-mix") => input
            .parse_nested_block(|input| parse_authored_color_mix_arguments(input, numeric))
            .map(CssColor::from_color_mix),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_authored_color_mix_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColorMix, ParseError<'i, Error>> {
    let start = input.position().byte_index();
    let location = input.current_source_location();
    let interpolation = if input
        .try_parse(|input| input.expect_ident_matching("in"))
        .is_ok()
    {
        let value = parse_authored_color_mix_interpolation_method(input)?;
        input.expect_comma().map_err(basic)?;
        Some(value)
    } else {
        None
    };
    let mut components = vec![parse_authored_color_mix_component(input, numeric)?];
    while !input.is_exhausted() {
        input.expect_comma().map_err(basic)?;
        components.push(parse_authored_color_mix_component(input, numeric)?);
    }
    CssColorMix::try_new(interpolation, components).map_err(|error| match error {
        CssColorMixConstructionError::EmptyComponents => {
            invalid_color(location, Some("color-mix component"))
        }
        CssColorMixConstructionError::NestingLimit
        | CssColorMixConstructionError::CapacityOverflow => {
            let kind = if error == CssColorMixConstructionError::NestingLimit {
                crate::CssComponentValueErrorKind::NestingLimit
            } else {
                crate::CssComponentValueErrorKind::CapacityOverflow
            };
            crate::error::invalid_component_value(
                location,
                crate::CssComponentValueError::new(
                    kind,
                    numeric.origin_at(start).expect("mix argument token origin"),
                ),
            )
        }
    })
}

fn parse_authored_color_mix_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolation, ParseError<'i, Error>> {
    let state = input.state();
    if let Ok(name) = input.expect_ident_cloned()
        && let Some(profile) = CssColorProfileName::try_new(name.as_ref())
    {
        return Ok(CssColorInterpolation::custom(profile));
    }
    input.reset(&state);
    let space = parse_color_mix_interpolation_space(input)?;
    let hue_location = input.current_source_location();
    let hue = input.try_parse(parse_color_mix_hue_interpolation).ok();
    CssColorInterpolation::try_predefined(CssColorInterpolationMethod::new(space, hue))
        .ok_or_else(|| invalid_color(hue_location, Some("hue interpolation")))
}

fn parse_authored_color_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColorMixComponent, ParseError<'i, Error>> {
    let leading = if next_is_mix_weight(input) {
        Some(parse_authored_color_mix_weight(input, numeric)?)
    } else {
        None
    };
    let color = parse_color(input, numeric)?;
    let weight = if leading.is_none() && next_is_mix_weight(input) {
        Some(parse_authored_color_mix_weight(input, numeric)?)
    } else {
        leading
    };
    Ok(CssColorMixComponent::new(color, weight))
}

fn next_is_mix_weight(input: &mut Parser<'_, '_>) -> bool {
    let state = input.state();
    let result = match input.next() {
        Ok(Token::Percentage { .. }) => true,
        Ok(Token::Function(name)) => is_math_function(name),
        _ => false,
    };
    input.reset(&state);
    result
}

fn parse_authored_color_mix_weight<'i>(
    input: &mut Parser<'i, '_>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssColorMixWeight, ParseError<'i, Error>> {
    input.skip_whitespace();
    let state = input.state();
    let location = input.current_source_location();
    if matches!(input.next(), Ok(Token::Function(_))) {
        let expression =
            parse_numeric_function(input, &state, numeric, CalculationRoot::Percentage)?;
        return CssColorMixWeight::try_calculation(CssPercentageCalculation::from_expression(
            expression,
        ))
        .map_err(|error| crate::error::invalid_component_value(location, error));
    }
    input.reset(&state);
    parse_authored_color_mix_percentage(input, numeric).map(CssColorMixWeight::literal)
}

fn parse_authored_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssColorMixPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let component = collect_color_scalar(input, numeric)?;
    CssColorMixPercentage::try_from_component(component)
        .map_err(|_| invalid_color(location, Some("color-mix component percentage")))
}

fn collect_color_scalar<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<crate::CssComponentValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let offset = input.position().byte_index();
    let location = input.current_source_location();
    numeric.collect(input).map_err(|error| {
        let location = numeric.error_location(&error, location, offset);
        if let Some(component) = error.component_error() {
            crate::error::invalid_component_value(location, component.clone())
        } else {
            invalid_color(location, Some("component"))
        }
    })
}

fn parse_color_mix_interpolation_space<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationSpace, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let space = match_ignore_ascii_case! { &ident,
        "srgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Srgb),
        "srgb-linear" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::SrgbLinear),
        "display-p3" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3),
        "display-p3-linear" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::DisplayP3Linear),
        "a98-rgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::A98Rgb),
        "prophoto-rgb" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::ProphotoRgb),
        "rec2020" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::Rec2020),
        "xyz" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "xyz-d50" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD50),
        "xyz-d65" => CssColorInterpolationSpace::Predefined(CssPredefinedColorSpace::XyzD65),
        "hsl" => CssColorInterpolationSpace::Hsl,
        "hwb" => CssColorInterpolationSpace::Hwb,
        "lab" => CssColorInterpolationSpace::Lab,
        "lch" => CssColorInterpolationSpace::Lch,
        "oklab" => CssColorInterpolationSpace::Oklab,
        "oklch" => CssColorInterpolationSpace::Oklch,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported color-mix interpolation space `{ident}`"),
        )),
    };
    Ok(space)
}

fn parse_color_mix_hue_interpolation<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssHueInterpolationMethod, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let hue = match_ignore_ascii_case! { &ident,
        "shorter" => CssHueInterpolationMethod::Shorter,
        "longer" => CssHueInterpolationMethod::Longer,
        "increasing" => CssHueInterpolationMethod::Increasing,
        "decreasing" => CssHueInterpolationMethod::Decreasing,
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported color-mix hue interpolation method `{ident}`"),
        )),
    };
    input.expect_ident_matching("hue").map_err(basic)?;
    Ok(hue)
}
