use cssparser::{
    ParseError, Parser, ParserInput, ToCss, Token,
    color::PredefinedColorSpace as ParsedPredefinedColorSpace, match_ignore_ascii_case,
};
use cssparser_color::{Color as ParsedColor, DefaultColorParser, parse_color_with};

use crate::error::{
    CssFeatureId, Error, basic, invalid_color, unsupported_value_at, with_color_context,
};
use crate::syntax::*;
use crate::validation::{LengthUnitStatus, classify_length_unit, parse_global_keyword};

pub(crate) static IMPLEMENTED_SHARED_VALUES: &[CssFeatureId] = &[
    CssFeatureId::new("official.value.syntax-token-stream"),
    CssFeatureId::new("official.value.component-value"),
    CssFeatureId::new("official.value.simple-block"),
    CssFeatureId::new("official.value.function"),
    CssFeatureId::new("official.value.declaration-value"),
    CssFeatureId::new("official.value.any-value"),
    CssFeatureId::new("official.value.an-plus-b"),
    CssFeatureId::new("official.value.unicode-range"),
    CssFeatureId::new("official.value.css-wide-keyword"),
    CssFeatureId::new("official.value.custom-ident"),
    CssFeatureId::new("official.value.ident"),
    CssFeatureId::new("official.value.string"),
    CssFeatureId::new("official.value.url"),
    CssFeatureId::new("official.value.url-modifier"),
    CssFeatureId::new("official.value.integer"),
    CssFeatureId::new("official.value.number"),
    CssFeatureId::new("official.value.dimension"),
    CssFeatureId::new("official.value.percentage"),
    CssFeatureId::new("official.value.length"),
    CssFeatureId::new("official.value.length-percentage"),
    CssFeatureId::new("official.value.angle"),
    CssFeatureId::new("official.value.angle-percentage"),
    CssFeatureId::new("official.value.time"),
    CssFeatureId::new("official.value.time-percentage"),
    CssFeatureId::new("official.value.frequency"),
    CssFeatureId::new("official.value.frequency-percentage"),
    CssFeatureId::new("official.value.resolution"),
    CssFeatureId::new("official.value.calc"),
    CssFeatureId::new("official.value.color"),
    CssFeatureId::new("official.value.alpha"),
    CssFeatureId::new("official.value.hue"),
    CssFeatureId::new("official.value.rgb"),
    CssFeatureId::new("official.value.hex-color"),
    CssFeatureId::new("official.value.named-color"),
    CssFeatureId::new("official.value.system-color"),
    CssFeatureId::new("official.value.deprecated-system-color"),
    CssFeatureId::new("official.value.transparent"),
    CssFeatureId::new("official.value.currentcolor"),
    CssFeatureId::new("official.value.hsl"),
    CssFeatureId::new("official.value.hwb"),
    CssFeatureId::new("official.value.lab"),
    CssFeatureId::new("official.value.lch"),
    CssFeatureId::new("official.value.oklab"),
    CssFeatureId::new("official.value.oklch"),
    CssFeatureId::new("official.value.predefined-color"),
    CssFeatureId::new("ext.value.relative-color"),
    CssFeatureId::new("ext.value.relative-color.rgb"),
    CssFeatureId::new("ext.value.relative-color.hsl"),
    CssFeatureId::new("ext.value.relative-color.hwb"),
    CssFeatureId::new("ext.value.relative-color.lab"),
    CssFeatureId::new("ext.value.relative-color.oklab"),
    CssFeatureId::new("ext.value.relative-color.lch"),
    CssFeatureId::new("ext.value.relative-color.oklch"),
    CssFeatureId::new("ext.value.relative-color.predefined"),
    CssFeatureId::new("ext.value.color-mix"),
];

pub(super) fn parse_box_size_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::BoxSize)
}

pub(super) fn parse_inset_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::Inset)
}

pub(super) fn parse_margin_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::Margin)
}

pub(super) fn parse_padding_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::Padding)
}

pub(super) fn parse_border_width_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::BorderWidth)
}

pub(super) fn parse_radius_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::Radius)
}

pub(super) fn parse_shadow_length<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::ShadowOffset)
}

pub(super) fn parse_shadow_blur_length<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, LengthGrammar::ShadowBlur)
}

pub(super) fn parse_gap_value<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    if input
        .try_parse(|input| input.expect_ident_matching("normal"))
        .is_ok()
    {
        Ok(CssLength::Normal)
    } else {
        parse_length_with(input, LengthGrammar::Gap)
    }
}

#[derive(Clone, Copy)]
pub(super) enum LengthGrammar {
    BoxSize,
    Inset,
    Margin,
    Padding,
    BorderWidth,
    Radius,
    ShadowOffset,
    ShadowBlur,
    BorderSpacing,
    Clip,
    OutlineOffset,
    Gap,
    FontSize,
    LineHeight,
    TextIndent,
    VerticalAlign,
    LetterSpacing,
    WordSpacing,
    TextDecorationThickness,
    GridTrack,
    BackgroundSize,
    Position,
    NonNegativeLength,
}

impl LengthGrammar {
    const fn allows_percent(self) -> bool {
        matches!(
            self,
            Self::BoxSize
                | Self::Inset
                | Self::Margin
                | Self::Padding
                | Self::Radius
                | Self::Gap
                | Self::FontSize
                | Self::LineHeight
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn allows_auto(self) -> bool {
        matches!(self, Self::BoxSize | Self::Inset | Self::Margin)
    }

    const fn allows_intrinsic(self) -> bool {
        matches!(self, Self::BoxSize | Self::Inset)
    }

    const fn allows_normal(self) -> bool {
        matches!(self, Self::Gap | Self::LineHeight)
    }

    const fn allows_line_width_keyword(self) -> bool {
        matches!(self, Self::BorderWidth)
    }

    const fn allows_calc_percent(self) -> bool {
        matches!(
            self,
            Self::BoxSize
                | Self::Inset
                | Self::Margin
                | Self::Padding
                | Self::Radius
                | Self::Gap
                | Self::FontSize
                | Self::LineHeight
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn requires_non_negative(self) -> bool {
        matches!(
            self,
            Self::Padding
                | Self::BorderWidth
                | Self::Radius
                | Self::ShadowBlur
                | Self::BorderSpacing
                | Self::TextDecorationThickness
                | Self::GridTrack
                | Self::BackgroundSize
                | Self::NonNegativeLength
        )
    }

    const fn context(self) -> &'static str {
        match self {
            Self::BoxSize => "box size",
            Self::Inset => "inset",
            Self::Margin => "margin",
            Self::Padding => "padding",
            Self::BorderWidth => "border-width",
            Self::Radius => "border-radius",
            Self::ShadowOffset => "box-shadow",
            Self::ShadowBlur => "box-shadow blur",
            Self::BorderSpacing => "border-spacing",
            Self::Clip => "clip",
            Self::OutlineOffset => "outline-offset",
            Self::Gap => "gap",
            Self::FontSize => "font-size",
            Self::LineHeight => "line-height",
            Self::TextIndent => "text-indent",
            Self::VerticalAlign => "vertical-align",
            Self::LetterSpacing => "letter-spacing",
            Self::WordSpacing => "word-spacing",
            Self::TextDecorationThickness => "text-decoration-thickness",
            Self::GridTrack => "grid track",
            Self::BackgroundSize => "background-size",
            Self::Position => "position",
            Self::NonNegativeLength => "non-negative length",
        }
    }
}

pub(super) fn checked_percentage_value<'i>(
    location: cssparser::SourceLocation,
    unit_value: f32,
    non_finite_reason: impl Into<String>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    let value = unit_value * 100.0;
    if value.is_finite() {
        Ok(value)
    } else {
        Err(unsupported_value_at(location, None, non_finite_reason))
    }
}

pub(super) fn parse_length_with<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context(input, grammar, grammar.context())
}

pub(super) fn parse_length_with_context<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context_mode(
        input,
        grammar,
        context,
        typed_length_calculation_is_current_consumer(context),
    )
}

pub(super) fn parse_length_with_context_legacy<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context_mode(input, grammar, context, false)
}

fn parse_length_with_context_mode<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
    allow_typed_calculation: bool,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, .. } if !value.is_finite() => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported non-finite {context} length"),
        )),
        Token::Dimension { value, unit, .. } => match classify_length_unit(unit) {
            LengthUnitStatus::Supported(_) if grammar.requires_non_negative() && *value < 0.0 => {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported negative {context} length"),
                ))
            }
            LengthUnitStatus::Supported(unit) => Ok(CssLength::dimension(*value, unit)),
            LengthUnitStatus::Unknown => Err(unsupported_value_at(
                location,
                None,
                format!("unknown {context} unit `{unit}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                format!("unsupported non-finite {context} percentage"),
            )?;
            if grammar.requires_non_negative() && value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported negative {context} percentage"),
                ))
            } else if grammar.allows_percent() {
                Ok(CssLength::percent(value))
            } else {
                Err(unsupported_value_at(
                    location,
                    None,
                    format!("unsupported {context} percentage"),
                ))
            }
        }
        Token::Number { value, .. } if *value == 0.0 => Ok(CssLength::Zero),
        Token::Ident(ident) => match_ignore_ascii_case! { ident,
            "auto" if grammar.allows_auto() => Ok(CssLength::Auto),
            "normal" if grammar.allows_normal() => Ok(CssLength::Normal),
            "thin" if grammar.allows_line_width_keyword() => Ok(CssLength::Thin),
            "medium" if grammar.allows_line_width_keyword() => Ok(CssLength::Medium),
            "thick" if grammar.allows_line_width_keyword() => Ok(CssLength::Thick),
            "min-content" if grammar.allows_intrinsic() => Ok(CssLength::MinContent),
            "max-content" if grammar.allows_intrinsic() => Ok(CssLength::MaxContent),
            "fit-content" if grammar.allows_intrinsic() => Ok(CssLength::FitContent),
            _ => Err(unsupported_value_at(
                location,
                None,
                format!("unsupported {context} `{ident}`"),
            )),
        },
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let calc = input.parse_nested_block(|input| {
                if allow_typed_calculation {
                    parse_calc_length_with_grammar(input, grammar)
                } else {
                    parse_legacy_calc_length_with_grammar(input, grammar)
                }
            })?;
            Ok(CssLength::Calc(calc))
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported length function `{name}` for {context}"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

fn typed_length_calculation_is_current_consumer(context: &str) -> bool {
    matches!(
        context,
        "box size"
            | "inset"
            | "margin"
            | "padding"
            | "border-width"
            | "border-radius"
            | "box-shadow"
            | "box-shadow blur"
            | "border-spacing"
            | "clip"
            | "outline-offset"
            | "gap"
            | "font-size"
            | "line-height"
            | "text-indent"
            | "vertical-align"
            | "letter-spacing"
            | "word-spacing"
            | "text-decoration-thickness"
            | "grid track"
            | "grid fit-content"
            | "background-size"
            | "position"
            | "outline-width"
            | "translate"
            | "gradient stop"
            | "radial-gradient size"
            | "column-width"
            | "column-rule-width"
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "T2 root kinds are consumed by the staged T3 property integration"
    )
)]
pub(super) enum CalculationRoot {
    Number,
    Integer,
    Percentage,
    Length,
    Angle,
    Time,
    Frequency,
}

const CALCULATION_NESTING_LIMIT: u16 = 256;

pub(super) fn parse_typed_calculation<'i, 't>(
    input: &mut Parser<'i, 't>,
    root: CalculationRoot,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let expression = parse_calculation_sum(input, 0)?;
    input.expect_exhausted().map_err(basic)?;
    if calculation_root_accepts(root, expression.result_type()) {
        Ok(expression)
    } else {
        Err(calculation_error(location))
    }
}

fn parse_calculation_sum<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: u16,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    let first = parse_calculation_product(input, depth)?;
    let mut result_type = first.result_type();
    let mut terms = vec![CssCalculationSumTerm {
        operator: None,
        expression: first,
    }];

    loop {
        let state = input.state();
        let location = input.current_source_location();
        let operator = match input.next() {
            Ok(Token::Delim('+')) => Some(CssCalculationSumOperator::Add),
            Ok(Token::Delim('-')) => Some(CssCalculationSumOperator::Subtract),
            Ok(_) | Err(_) => None,
        };
        let Some(operator) = operator else {
            input.reset(&state);
            break;
        };
        let expression = parse_calculation_product(input, depth)?;
        result_type = calculation_sum_type(result_type, expression.result_type())
            .ok_or_else(|| calculation_error(location))?;
        terms.push(CssCalculationSumTerm {
            operator: Some(operator),
            expression,
        });
    }

    if terms.len() == 1 {
        return match terms.pop() {
            Some(term) => Ok(term.expression),
            None => Err(calculation_error(input.current_source_location())),
        };
    }
    let expression = CssCalculationExpression::Sum { terms, result_type };
    validate_calculation_arithmetic(&expression, input.current_source_location())?;
    Ok(expression)
}

fn parse_calculation_product<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: u16,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    let first = parse_calculation_unary(input, depth)?;
    let mut result_type = first.result_type();
    let mut factors = vec![CssCalculationProductFactor {
        operator: None,
        expression: first,
    }];

    loop {
        let state = input.state();
        let location = input.current_source_location();
        let operator = match input.next() {
            Ok(Token::Delim('*')) => Some(CssCalculationProductOperator::Multiply),
            Ok(Token::Delim('/')) => Some(CssCalculationProductOperator::Divide),
            Ok(_) | Err(_) => None,
        };
        let Some(operator) = operator else {
            input.reset(&state);
            break;
        };
        let expression = parse_calculation_unary(input, depth)?;
        result_type = match operator {
            CssCalculationProductOperator::Multiply => {
                calculation_product_type(result_type, expression.result_type())
            }
            CssCalculationProductOperator::Divide => {
                if !calculation_type_is_number(expression.result_type())
                    || matches!(calculation_numeric_value(&expression), Ok(Some(value)) if value == 0.0)
                {
                    None
                } else {
                    calculation_quotient_type(result_type, expression.result_type())
                }
            }
        }
        .ok_or_else(|| calculation_error(location))?;
        factors.push(CssCalculationProductFactor {
            operator: Some(operator),
            expression,
        });
    }

    if factors.len() == 1 {
        return match factors.pop() {
            Some(factor) => Ok(factor.expression),
            None => Err(calculation_error(input.current_source_location())),
        };
    }
    let expression = CssCalculationExpression::Product {
        factors,
        result_type,
    };
    validate_calculation_arithmetic(&expression, input.current_source_location())?;
    Ok(expression)
}

fn parse_calculation_unary<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: u16,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    if matches!(input.next(), Ok(Token::Delim('-'))) {
        let operand = parse_calculation_unary(input, depth)?;
        let expression = CssCalculationExpression::Negate(Box::new(operand));
        validate_calculation_arithmetic(&expression, location)?;
        return Ok(expression);
    }
    input.reset(&state);
    parse_calculation_value(input, depth)
}

fn parse_calculation_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    depth: u16,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let token_start = input.position();
    let token = input.next().map_err(basic)?.clone();
    let authored_token = input.slice_from(token_start);
    match token {
        Token::Number { value, .. } if !value.is_finite() => Err(calculation_error(location)),
        Token::Number {
            value: _,
            int_value: Some(integer),
            ..
        } if authored_token.parse::<i32>() == Ok(integer) => Ok(CssCalculationExpression::Value(
            CssCalculationValue::Integer(integer),
        )),
        Token::Number {
            int_value: Some(_), ..
        } => Err(calculation_error(location)),
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssCalculationValue::Number)
            .map(CssCalculationExpression::Value)
            .ok_or_else(|| calculation_error(location)),
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                unit_value,
                "unsupported non-finite calculation percentage",
            )?;
            CssFiniteNumber::try_new(value)
                .map(CssCalculationValue::Percentage)
                .map(CssCalculationExpression::Value)
                .ok_or_else(|| calculation_error(location))
        }
        Token::Dimension { value, .. } if !value.is_finite() => Err(calculation_error(location)),
        Token::Dimension { value, unit, .. } => parse_calculation_dimension(value, &unit, location),
        Token::ParenthesisBlock => {
            let nested_depth = checked_calculation_depth(depth, location)?;
            let operand =
                input.parse_nested_block(|input| parse_calculation_sum(input, nested_depth))?;
            Ok(CssCalculationExpression::Group(Box::new(operand)))
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let nested_depth = checked_calculation_depth(depth, location)?;
            let operand =
                input.parse_nested_block(|input| parse_calculation_sum(input, nested_depth))?;
            Ok(CssCalculationExpression::NestedCalc(Box::new(operand)))
        }
        _ => Err(calculation_error(location)),
    }
}

fn parse_calculation_dimension<'i>(
    value: f32,
    unit: &str,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssCalculationExpression, ParseError<'i, Error>> {
    let parsed = match classify_length_unit(unit) {
        LengthUnitStatus::Supported(unit) => {
            CssLengthDimension::try_new(value, unit).map(CssCalculationValue::Length)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("deg") => {
            CssAngleLiteral::try_new(value, CssAngleUnit::Degrees).map(CssCalculationValue::Angle)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("grad") => {
            CssAngleLiteral::try_new(value, CssAngleUnit::Gradians).map(CssCalculationValue::Angle)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("rad") => {
            CssAngleLiteral::try_new(value, CssAngleUnit::Radians).map(CssCalculationValue::Angle)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("turn") => {
            CssAngleLiteral::try_new(value, CssAngleUnit::Turns).map(CssCalculationValue::Angle)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("s") => {
            CssDelayLiteral::try_new(value, CssTimeUnit::Seconds).map(CssCalculationValue::Time)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("ms") => {
            CssDelayLiteral::try_new(value, CssTimeUnit::Milliseconds)
                .map(CssCalculationValue::Time)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("hz") => {
            CssFrequencyLiteral::try_new(value, CssFrequencyUnit::Hertz)
                .map(CssCalculationValue::Frequency)
        }
        LengthUnitStatus::Unknown if unit.eq_ignore_ascii_case("khz") => {
            CssFrequencyLiteral::try_new(value, CssFrequencyUnit::Kilohertz)
                .map(CssCalculationValue::Frequency)
        }
        LengthUnitStatus::Unknown => None,
    };
    parsed
        .map(CssCalculationExpression::Value)
        .ok_or_else(|| calculation_error(location))
}

fn checked_calculation_depth<'i>(
    depth: u16,
    location: cssparser::SourceLocation,
) -> std::result::Result<u16, ParseError<'i, Error>> {
    if depth >= CALCULATION_NESTING_LIMIT {
        Err(calculation_error(location))
    } else {
        Ok(depth + 1)
    }
}

const fn calculation_root_accepts(root: CalculationRoot, result_type: CssCalculationType) -> bool {
    match root {
        CalculationRoot::Number => matches!(
            result_type,
            CssCalculationType::Integer | CssCalculationType::Number
        ),
        CalculationRoot::Integer => matches!(result_type, CssCalculationType::Integer),
        CalculationRoot::Percentage => matches!(result_type, CssCalculationType::Percentage),
        CalculationRoot::Length => matches!(
            result_type,
            CssCalculationType::Length
                | CssCalculationType::Percentage
                | CssCalculationType::LengthPercentage
        ),
        CalculationRoot::Angle => matches!(
            result_type,
            CssCalculationType::Angle
                | CssCalculationType::Percentage
                | CssCalculationType::AnglePercentage
        ),
        CalculationRoot::Time => matches!(
            result_type,
            CssCalculationType::Time
                | CssCalculationType::Percentage
                | CssCalculationType::TimePercentage
        ),
        CalculationRoot::Frequency => matches!(
            result_type,
            CssCalculationType::Frequency
                | CssCalculationType::Percentage
                | CssCalculationType::FrequencyPercentage
        ),
    }
}

const fn calculation_type_is_number(result_type: CssCalculationType) -> bool {
    matches!(
        result_type,
        CssCalculationType::Integer | CssCalculationType::Number
    )
}

fn calculation_sum_type(
    left: CssCalculationType,
    right: CssCalculationType,
) -> Option<CssCalculationType> {
    if left == right {
        return Some(left);
    }
    match (left, right) {
        (CssCalculationType::Integer, CssCalculationType::Number)
        | (CssCalculationType::Number, CssCalculationType::Integer) => {
            Some(CssCalculationType::Number)
        }
        (CssCalculationType::Length, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::Length)
        | (CssCalculationType::LengthPercentage, CssCalculationType::Length)
        | (CssCalculationType::Length, CssCalculationType::LengthPercentage)
        | (CssCalculationType::LengthPercentage, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::LengthPercentage) => {
            Some(CssCalculationType::LengthPercentage)
        }
        (CssCalculationType::Angle, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::Angle)
        | (CssCalculationType::AnglePercentage, CssCalculationType::Angle)
        | (CssCalculationType::Angle, CssCalculationType::AnglePercentage)
        | (CssCalculationType::AnglePercentage, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::AnglePercentage) => {
            Some(CssCalculationType::AnglePercentage)
        }
        (CssCalculationType::Time, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::Time)
        | (CssCalculationType::TimePercentage, CssCalculationType::Time)
        | (CssCalculationType::Time, CssCalculationType::TimePercentage)
        | (CssCalculationType::TimePercentage, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::TimePercentage) => {
            Some(CssCalculationType::TimePercentage)
        }
        (CssCalculationType::Frequency, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::Frequency)
        | (CssCalculationType::FrequencyPercentage, CssCalculationType::Frequency)
        | (CssCalculationType::Frequency, CssCalculationType::FrequencyPercentage)
        | (CssCalculationType::FrequencyPercentage, CssCalculationType::Percentage)
        | (CssCalculationType::Percentage, CssCalculationType::FrequencyPercentage) => {
            Some(CssCalculationType::FrequencyPercentage)
        }
        _ => None,
    }
}

const fn calculation_product_type(
    left: CssCalculationType,
    right: CssCalculationType,
) -> Option<CssCalculationType> {
    match (
        calculation_type_is_number(left),
        calculation_type_is_number(right),
    ) {
        (true, true)
            if matches!(left, CssCalculationType::Number)
                || matches!(right, CssCalculationType::Number) =>
        {
            Some(CssCalculationType::Number)
        }
        (true, true) => Some(CssCalculationType::Integer),
        (true, false) => Some(right),
        (false, true) => Some(left),
        (false, false) => None,
    }
}

const fn calculation_quotient_type(
    numerator: CssCalculationType,
    denominator: CssCalculationType,
) -> Option<CssCalculationType> {
    if !calculation_type_is_number(denominator) {
        None
    } else if calculation_type_is_number(numerator) {
        Some(CssCalculationType::Number)
    } else {
        Some(numerator)
    }
}

fn validate_calculation_arithmetic<'i>(
    expression: &CssCalculationExpression,
    location: cssparser::SourceLocation,
) -> std::result::Result<(), ParseError<'i, Error>> {
    match calculation_numeric_value(expression) {
        Ok(_) => Ok(()),
        Err(()) => Err(calculation_error(location)),
    }
}

fn calculation_numeric_value(expression: &CssCalculationExpression) -> Result<Option<f32>, ()> {
    match expression {
        CssCalculationExpression::Value(CssCalculationValue::Integer(value)) => {
            Ok(Some(*value as f32))
        }
        CssCalculationExpression::Value(CssCalculationValue::Number(value)) => {
            Ok(Some(value.value()))
        }
        CssCalculationExpression::Value(
            CssCalculationValue::Percentage(_)
            | CssCalculationValue::Length(_)
            | CssCalculationValue::Angle(_)
            | CssCalculationValue::Time(_)
            | CssCalculationValue::Frequency(_),
        ) => Ok(None),
        CssCalculationExpression::Sum { terms, .. } => {
            let mut value = None;
            for term in terms {
                let term_value = calculation_numeric_value(&term.expression)?;
                value = match (value, term_value, term.operator) {
                    (None, next, None) => next,
                    (Some(current), Some(next), Some(CssCalculationSumOperator::Add)) => {
                        Some(current + next)
                    }
                    (Some(current), Some(next), Some(CssCalculationSumOperator::Subtract)) => {
                        Some(current - next)
                    }
                    _ => None,
                };
                if matches!(value, Some(value) if !value.is_finite()) {
                    return Err(());
                }
            }
            Ok(value)
        }
        CssCalculationExpression::Product { factors, .. } => {
            let mut value = None;
            for factor in factors {
                let factor_value = calculation_numeric_value(&factor.expression)?;
                value = match (value, factor_value, factor.operator) {
                    (None, next, None) => next,
                    (Some(current), Some(next), Some(CssCalculationProductOperator::Multiply)) => {
                        Some(current * next)
                    }
                    (Some(_), Some(0.0), Some(CssCalculationProductOperator::Divide)) => {
                        return Err(());
                    }
                    (Some(current), Some(next), Some(CssCalculationProductOperator::Divide)) => {
                        Some(current / next)
                    }
                    _ => None,
                };
                if matches!(value, Some(value) if !value.is_finite()) {
                    return Err(());
                }
            }
            Ok(value)
        }
        CssCalculationExpression::Negate(operand) => {
            let value = calculation_numeric_value(operand)?.map(|value| -value);
            if matches!(value, Some(value) if !value.is_finite()) {
                Err(())
            } else {
                Ok(value)
            }
        }
        CssCalculationExpression::Group(operand)
        | CssCalculationExpression::NestedCalc(operand) => calculation_numeric_value(operand),
    }
}

fn calculation_error<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    unsupported_value_at(location, None, "invalid typed calculation")
}

pub(super) fn parse_calc_length_with_grammar<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssCalcLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    if let Ok(legacy) =
        input.try_parse(|input| parse_legacy_calc_length_with_grammar(input, grammar))
    {
        return Ok(legacy);
    }

    let expression = parse_typed_calculation(input, CalculationRoot::Length)?;
    if !grammar.allows_calc_percent()
        && matches!(
            expression.result_type(),
            CssCalculationType::Percentage | CssCalculationType::LengthPercentage
        )
    {
        return Err(calculation_error(location));
    }
    Ok(CssCalcLength::Typed(CssLengthCalculation::from_expression(
        expression,
    )))
}

pub(super) fn parse_legacy_calc_length_with_grammar<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssCalcLength, ParseError<'i, Error>> {
    let first = CssCalcLengthTerm::add(parse_calc_component(input, grammar)?);
    let mut terms = Vec::new();

    while !input.is_exhausted() {
        let location = input.current_source_location();
        let operator = match input.next().map_err(basic)? {
            Token::Delim('+') => CssCalcLengthTerm::add,
            Token::Delim('-') => CssCalcLengthTerm::sub,
            token => {
                return Err(unsupported_value_at(
                    location,
                    None,
                    format!("expected calc operator, got `{}`", token.to_css_string()),
                ));
            }
        };
        let component = parse_calc_component(input, grammar)?;
        terms.push(operator(component));
    }

    Ok(CssCalcLength::sum(first, terms))
}

pub(super) fn parse_calc_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
) -> std::result::Result<CssCalcLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Dimension { value, .. } if !value.is_finite() => Err(unsupported_value_at(
            location,
            None,
            "unsupported non-finite calc length",
        )),
        Token::Dimension { value, unit, .. } => match classify_length_unit(unit) {
            LengthUnitStatus::Supported(_) if grammar.requires_non_negative() && *value < 0.0 => {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative calc length",
                ))
            }
            LengthUnitStatus::Supported(unit) => Ok(CssCalcLength::dimension(*value, unit)),
            LengthUnitStatus::Unknown => Err(unsupported_value_at(
                location,
                None,
                format!("unknown calc length unit `{unit}`"),
            )),
        },
        Token::Percentage { unit_value, .. } => {
            let value = checked_percentage_value(
                location,
                *unit_value,
                "unsupported non-finite calc percentage",
            )?;
            if grammar.requires_non_negative() && value < 0.0 {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported negative calc percentage",
                ))
            } else if grammar.allows_calc_percent() {
                Ok(CssCalcLength::percent(value))
            } else {
                Err(unsupported_value_at(
                    location,
                    None,
                    "unsupported calc percentage",
                ))
            }
        }
        Token::Number { value, .. } if *value == 0.0 => Ok(CssCalcLength::px(0.0)),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(|input| parse_legacy_calc_length_with_grammar(input, grammar))
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported calc function `{name}`"),
        )),
        token => Err(unsupported_value_at(
            location,
            None,
            format!("unexpected calc token `{}`", token.to_css_string()),
        )),
    }
}

pub(super) fn parse_number<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    input.expect_number().map_err(basic)
}

pub(super) fn parse_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<i32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number {
            int_value: Some(value),
            ..
        } => Ok(*value),
        Token::Number { .. } => Err(unsupported_value_at(
            location,
            None,
            format!("{context} must be an integer"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_positive_integer<'i, 't>(
    input: &mut Parser<'i, 't>,
    context: &str,
) -> std::result::Result<i32, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_integer(input, context)?;
    if value <= 0 {
        Err(unsupported_value_at(
            location,
            None,
            format!("{context} must be a positive integer"),
        ))
    } else {
        Ok(value)
    }
}

pub(super) fn parse_custom_ident_from_str_at<'i>(
    context: &str,
    ident: &str,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssCustomIdent, ParseError<'i, Error>> {
    if ident.is_empty()
        || parse_global_keyword(ident).is_some()
        || ident.eq_ignore_ascii_case("span")
        || ident.eq_ignore_ascii_case("auto")
    {
        Err(unsupported_value_at(
            location,
            None,
            format!("unsupported {context} `{ident}`"),
        ))
    } else {
        Ok(CssCustomIdent::new(ident))
    }
}

pub(super) fn next_is_delim<'i, 't>(input: &mut Parser<'i, 't>, delim: char) -> bool {
    let state = input.state();
    let is_delim = input.try_parse(|input| input.expect_delim(delim)).is_ok();
    input.reset(&state);
    is_delim
}

pub(super) fn next_is_comma<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_comma = input.try_parse(Parser::expect_comma).is_ok();
    input.reset(&state);
    is_comma
}

pub(super) fn next_is_ident<'i, 't>(input: &mut Parser<'i, 't>, expected: &str) -> bool {
    let state = input.state();
    let is_ident = input
        .try_parse(|input| input.expect_ident_matching(expected))
        .is_ok();
    input.reset(&state);
    is_ident
}

pub(super) fn parse_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssParsedColor, ParseError<'i, Error>> {
    let start = input.position();
    if next_is_authored_relative_color(input) {
        let current = parse_authored_relative_color(input)
            .map_err(|error| with_color_context(error, None))?;
        let i01_subset = parse_compatibility_color_text(input.slice_from(start));
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    if next_is_color_mix(input) {
        let current =
            parse_authored_color_mix(input).map_err(|error| with_color_context(error, None))?;
        let i01_subset = parse_compatibility_color_text(input.slice_from(start));
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    if let Ok(color) = input.try_parse(parse_compatibility_only_predefined_color) {
        return Ok(CssParsedColor::from_i01(color));
    }
    let start = input.position();
    if next_is_selected_authored_color(input) {
        let current = parse_selected_authored_color(input)
            .map_err(|error| with_color_context(error, None))?;
        let i01_subset = current
            .has_exact_i01_projection()
            .then(|| parse_compatibility_color_text(input.slice_from(start)))
            .flatten();
        return Ok(CssParsedColor::new(current, i01_subset));
    }
    parse_color_inner(input)
        .map(CssParsedColor::from_i01)
        .map_err(|error| with_color_context(error, None))
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

fn parse_compatibility_only_predefined_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let color = parse_color_inner(input)?;
    if matches!(
        color,
        CssColor::ColorFunction(ref value)
            if value.color_space() == CssPredefinedColorSpace::DisplayP3Linear
    ) {
        Ok(color)
    } else {
        Err(invalid_color(location, None))
    }
}

fn next_is_selected_authored_color<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let selected = match input.next() {
        Ok(Token::Ident(_) | Token::Hash(_) | Token::IDHash(_)) => true,
        Ok(Token::Function(name)) => {
            name.eq_ignore_ascii_case("rgb")
                || name.eq_ignore_ascii_case("rgba")
                || name.eq_ignore_ascii_case("hsl")
                || name.eq_ignore_ascii_case("hsla")
                || name.eq_ignore_ascii_case("hwb")
                || name.eq_ignore_ascii_case("lab")
                || name.eq_ignore_ascii_case("lch")
                || name.eq_ignore_ascii_case("oklab")
                || name.eq_ignore_ascii_case("oklch")
                || name.eq_ignore_ascii_case("color")
        }
        Ok(_) | Err(_) => false,
    };
    input.reset(&state);
    selected
}

fn parse_compatibility_color_text(source: &str) -> Option<CssColor> {
    let mut input = ParserInput::new(source);
    let mut parser = Parser::new(&mut input);
    let color = parse_color_inner(&mut parser).ok()?;
    parser.expect_exhausted().ok()?;
    Some(color)
}

fn parse_selected_authored_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("currentcolor") => {
            Ok(CssAuthoredColor::current_color())
        }
        Token::Ident(ident) if ident.eq_ignore_ascii_case("transparent") => {
            Ok(CssAuthoredColor::transparent())
        }
        Token::Ident(ident) => {
            if let Some(system) = authored_system_color(&ident) {
                return Ok(CssAuthoredColor::from_system(system));
            }
            if parse_compatibility_color_text(&ident).is_some() {
                return Ok(CssAuthoredColor::from_named(CssNamedColor::new(
                    ident.to_ascii_lowercase(),
                )));
            }
            Err(invalid_color(location, None))
        }
        Token::Hash(digits) | Token::IDHash(digits)
            if matches!(digits.len(), 3 | 4 | 6 | 8)
                && digits.bytes().all(|byte| byte.is_ascii_hexdigit()) =>
        {
            Ok(CssAuthoredColor::hex(CssHexColor::new(digits.as_ref())))
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("rgb") || name.eq_ignore_ascii_case("rgba") =>
        {
            input
                .parse_nested_block(parse_authored_rgb)
                .map(CssAuthoredColor::rgb)
        }
        Token::Function(name)
            if name.eq_ignore_ascii_case("hsl") || name.eq_ignore_ascii_case("hsla") =>
        {
            input
                .parse_nested_block(parse_authored_hsl)
                .map(CssAuthoredColor::hsl)
        }
        Token::Function(name) if name.eq_ignore_ascii_case("hwb") => input
            .parse_nested_block(parse_authored_hwb)
            .map(CssAuthoredColor::hwb),
        Token::Function(name) if name.eq_ignore_ascii_case("lab") => input
            .parse_nested_block(parse_authored_lab)
            .map(CssAuthoredColor::lab),
        Token::Function(name) if name.eq_ignore_ascii_case("lch") => input
            .parse_nested_block(parse_authored_lch)
            .map(CssAuthoredColor::lch),
        Token::Function(name) if name.eq_ignore_ascii_case("oklab") => input
            .parse_nested_block(parse_authored_lab)
            .map(CssAuthoredColor::oklab),
        Token::Function(name) if name.eq_ignore_ascii_case("oklch") => input
            .parse_nested_block(parse_authored_lch)
            .map(CssAuthoredColor::oklch),
        Token::Function(name) if name.eq_ignore_ascii_case("color") => input
            .parse_nested_block(parse_authored_predefined_color)
            .map(CssAuthoredColor::predefined),
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            None,
        )),
    }
}

fn parse_authored_rgb<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredRgbColor, ParseError<'i, Error>> {
    let first = parse_authored_color_component(input, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if first.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("red")));
        }
        let domain = first.domain();
        let second_location = input.current_source_location();
        let second = parse_authored_color_component(input, false)?;
        if second.domain() != domain {
            return Err(invalid_color(second_location, Some("component")));
        }
        input.expect_comma().map_err(basic)?;
        let third_location = input.current_source_location();
        let third = parse_authored_color_component(input, false)?;
        if third.domain() != domain {
            return Err(invalid_color(third_location, Some("component")));
        }
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredRgbColor::new(
            CssAuthoredColorSyntax::Legacy,
            [first, second, third],
            alpha,
        ))
    } else {
        let second = parse_authored_color_component(input, true)?;
        let third = parse_authored_color_component(input, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredRgbColor::new(
            CssAuthoredColorSyntax::Modern,
            [first, second, third],
            alpha,
        ))
    }
}

fn parse_authored_hsl<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredHslColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, true)?;
    if input.try_parse(Parser::expect_comma).is_ok() {
        if hue.is_none() {
            return Err(invalid_color(input.current_source_location(), Some("hue")));
        }
        let saturation = parse_authored_percentage_component(input, false)?;
        input.expect_comma().map_err(basic)?;
        let lightness = parse_authored_percentage_component(input, false)?;
        let alpha = if input.try_parse(Parser::expect_comma).is_ok() {
            Some(parse_authored_alpha(input, false)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredHslColor::new(
            CssAuthoredColorSyntax::Legacy,
            hue,
            saturation,
            lightness,
            alpha,
        ))
    } else {
        let saturation = parse_authored_percentage_component(input, true)?;
        let lightness = parse_authored_percentage_component(input, true)?;
        let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
            Some(parse_authored_alpha(input, true)?)
        } else {
            None
        };
        input.expect_exhausted().map_err(basic)?;
        Ok(CssAuthoredHslColor::new(
            CssAuthoredColorSyntax::Modern,
            hue,
            saturation,
            lightness,
            alpha,
        ))
    }
}

fn parse_authored_hwb<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredHwbColor, ParseError<'i, Error>> {
    let hue = parse_authored_hue(input, true)?;
    let whiteness = parse_authored_percentage_component(input, true)?;
    let blackness = parse_authored_percentage_component(input, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredHwbColor::new(hue, whiteness, blackness, alpha))
}

fn parse_authored_lab<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredLabColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, true)?;
    let a = parse_authored_color_component(input, true)?;
    let b = parse_authored_color_component(input, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredLabColor::new(lightness, a, b, alpha))
}

fn parse_authored_lch<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredLchColor, ParseError<'i, Error>> {
    let lightness = parse_authored_color_component(input, true)?;
    let chroma = parse_authored_color_component(input, true)?;
    let hue = parse_authored_hue(input, true)?;
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredLchColor::new(lightness, chroma, hue, alpha))
}

fn parse_authored_predefined_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredPredefinedColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color_space = match_ignore_ascii_case! { &ident,
        "srgb" => CssPredefinedColorSpace::Srgb,
        "srgb-linear" => CssPredefinedColorSpace::SrgbLinear,
        "display-p3" => CssPredefinedColorSpace::DisplayP3,
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
        parse_authored_color_component(input, true)?,
        parse_authored_color_component(input, true)?,
        parse_authored_color_component(input, true)?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_authored_alpha(input, true)?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredPredefinedColor::new(
        color_space,
        channels,
        alpha,
    ))
}

fn parse_authored_color_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssAuthoredColorComponent::None)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssAuthoredColorComponent::Number)
            .ok_or_else(|| invalid_color(location, Some("component"))),
        Token::Percentage { unit_value, .. } => CssFiniteNumber::try_new(unit_value * 100.0)
            .map(CssAuthoredColorComponent::Percentage)
            .ok_or_else(|| invalid_color(location, Some("component"))),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            parse_authored_number_or_percentage_calculation(input, location)
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("component"),
        )),
    }
}

fn parse_authored_percentage_component<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let value = parse_authored_color_component(input, allow_none)?;
    if matches!(
        value,
        CssAuthoredColorComponent::None
            | CssAuthoredColorComponent::Percentage(_)
            | CssAuthoredColorComponent::PercentageCalculation(_)
    ) {
        Ok(value)
    } else {
        Err(invalid_color(location, Some("percentage")))
    }
}

fn parse_authored_alpha<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    parse_authored_color_component(input, allow_none)
}

fn parse_authored_number_or_percentage_calculation<'i, 't>(
    input: &mut Parser<'i, 't>,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssAuthoredColorComponent, ParseError<'i, Error>> {
    if let Ok(expression) = input.try_parse(|input| {
        input.parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Number))
    }) {
        return Ok(CssAuthoredColorComponent::NumberCalculation(
            CssNumberCalculation::from_expression(expression),
        ));
    }
    input
        .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Percentage))
        .map(CssPercentageCalculation::from_expression)
        .map(CssAuthoredColorComponent::PercentageCalculation)
        .map_err(|_| invalid_color(location, Some("component")))
}

fn parse_authored_hue<'i, 't>(
    input: &mut Parser<'i, 't>,
    allow_none: bool,
) -> std::result::Result<CssAuthoredHue, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)?.clone() {
        Token::Ident(ident) if allow_none && ident.eq_ignore_ascii_case("none") => {
            Ok(CssAuthoredHue::None)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssAuthoredHue::Number)
            .ok_or_else(|| invalid_color(location, Some("hue"))),
        token @ Token::Dimension { .. } => {
            let Token::Dimension {
                value, ref unit, ..
            } = token
            else {
                unreachable!("matched dimension token")
            };
            let unit = match unit.to_ascii_lowercase().as_str() {
                "deg" => CssAngleUnit::Degrees,
                "grad" => CssAngleUnit::Gradians,
                "rad" => CssAngleUnit::Radians,
                "turn" => CssAngleUnit::Turns,
                _ => {
                    return Err(with_color_context(
                        location.new_unexpected_token_error::<Error>(token),
                        Some("hue"),
                    ));
                }
            };
            CssAngleLiteral::try_new(value, unit)
                .map(CssAuthoredHue::Angle)
                .ok_or_else(|| invalid_color(location, Some("hue")))
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            if let Ok(expression) = input.try_parse(|input| {
                input.parse_nested_block(|input| {
                    parse_typed_calculation(input, CalculationRoot::Number)
                })
            }) {
                return Ok(CssAuthoredHue::NumberCalculation(
                    CssNumberCalculation::from_expression(expression),
                ));
            }
            input
                .parse_nested_block(|input| parse_typed_calculation(input, CalculationRoot::Angle))
                .map(CssAngleCalculation::from_expression)
                .map(CssAuthoredHue::AngleCalculation)
                .map_err(|_| invalid_color(location, Some("hue")))
        }
        token => Err(with_color_context(
            location.new_unexpected_token_error::<Error>(token),
            Some("hue"),
        )),
    }
}

fn authored_system_color(ident: &str) -> Option<CssAuthoredSystemColor> {
    let value = match_ignore_ascii_case! { ident,
        "canvas" => CssAuthoredSystemColor::Canvas,
        "canvastext" => CssAuthoredSystemColor::CanvasText,
        "linktext" => CssAuthoredSystemColor::LinkText,
        "visitedtext" => CssAuthoredSystemColor::VisitedText,
        "activetext" => CssAuthoredSystemColor::ActiveText,
        "buttonface" => CssAuthoredSystemColor::ButtonFace,
        "buttontext" => CssAuthoredSystemColor::ButtonText,
        "buttonborder" => CssAuthoredSystemColor::ButtonBorder,
        "field" => CssAuthoredSystemColor::Field,
        "fieldtext" => CssAuthoredSystemColor::FieldText,
        "highlight" => CssAuthoredSystemColor::Highlight,
        "highlighttext" => CssAuthoredSystemColor::HighlightText,
        "mark" => CssAuthoredSystemColor::Mark,
        "marktext" => CssAuthoredSystemColor::MarkText,
        "graytext" => CssAuthoredSystemColor::GrayText,
        "selecteditem" => CssAuthoredSystemColor::SelectedItem,
        "selecteditemtext" => CssAuthoredSystemColor::SelectedItemText,
        "accentcolor" => CssAuthoredSystemColor::AccentColor,
        "accentcolortext" => CssAuthoredSystemColor::AccentColorText,
        "activeborder" => CssAuthoredSystemColor::ActiveBorder,
        "activecaption" => CssAuthoredSystemColor::ActiveCaption,
        "appworkspace" => CssAuthoredSystemColor::AppWorkspace,
        "background" => CssAuthoredSystemColor::Background,
        "buttonhighlight" => CssAuthoredSystemColor::ButtonHighlight,
        "buttonshadow" => CssAuthoredSystemColor::ButtonShadow,
        "captiontext" => CssAuthoredSystemColor::CaptionText,
        "inactiveborder" => CssAuthoredSystemColor::InactiveBorder,
        "inactivecaption" => CssAuthoredSystemColor::InactiveCaption,
        "inactivecaptiontext" => CssAuthoredSystemColor::InactiveCaptionText,
        "infobackground" => CssAuthoredSystemColor::InfoBackground,
        "infotext" => CssAuthoredSystemColor::InfoText,
        "menu" => CssAuthoredSystemColor::Menu,
        "menutext" => CssAuthoredSystemColor::MenuText,
        "scrollbar" => CssAuthoredSystemColor::Scrollbar,
        "threeddarkshadow" => CssAuthoredSystemColor::ThreeDDarkShadow,
        "threedface" => CssAuthoredSystemColor::ThreeDFace,
        "threedhighlight" => CssAuthoredSystemColor::ThreeDHighlight,
        "threedlightshadow" => CssAuthoredSystemColor::ThreeDLightShadow,
        "threedshadow" => CssAuthoredSystemColor::ThreeDShadow,
        "window" => CssAuthoredSystemColor::Window,
        "windowframe" => CssAuthoredSystemColor::WindowFrame,
        "windowtext" => CssAuthoredSystemColor::WindowText,
        _ => return None,
    };
    Some(value)
}

fn parse_color_inner<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    if let Ok(color) = input.try_parse(parse_relative_color) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_color_mix) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_absolute_color_with_cssparser_color) {
        return Ok(color);
    }
    if let Ok(color) = input.try_parse(parse_system_color) {
        return Ok(color);
    }
    Err(invalid_color(input.current_source_location(), None))
}

fn parse_authored_relative_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let Token::Function(name) = token else {
        return Err(location.new_unexpected_token_error(token));
    };
    let Some(function) = relative_color_function_from_name(&name) else {
        return Err(location.new_unexpected_token_error(Token::Function(name)));
    };
    input
        .parse_nested_block(|input| parse_authored_relative_color_arguments(input, function))
        .map(CssAuthoredColor::relative)
}

fn parse_authored_relative_color_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: RelativeColorFunction,
) -> std::result::Result<CssAuthoredRelativeColor, ParseError<'i, Error>> {
    input.expect_ident_matching("from").map_err(basic)?;
    let (source, _) = parse_color(input)?.into_parts();
    let (function, environment, domains) = relative_color_signature(input, function)?;
    let channels = [
        parse_typed_relative_color_expression(input, environment, domains[0])?,
        parse_typed_relative_color_expression(input, environment, domains[1])?,
        parse_typed_relative_color_expression(input, environment, domains[2])?,
    ];
    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_typed_relative_color_expression(
            input,
            environment,
            CssRelativeColorResultDomain::Alpha,
        )?)
    } else {
        None
    };
    input.expect_exhausted().map_err(basic)?;
    Ok(CssAuthoredRelativeColor::new(
        function,
        environment,
        source,
        channels,
        alpha,
    ))
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
    use CssRelativeColorResultDomain::{Hue, NumberPercentage};
    let signature = match function {
        RelativeColorFunction::Rgb => (
            CssRelativeColorFunction::Rgb,
            CssRelativeColorEnvironment::Rgb,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Hsl => (
            CssRelativeColorFunction::Hsl,
            CssRelativeColorEnvironment::Hsl,
            [Hue, NumberPercentage, NumberPercentage],
        ),
        RelativeColorFunction::Hwb => (
            CssRelativeColorFunction::Hwb,
            CssRelativeColorEnvironment::Hwb,
            [Hue, NumberPercentage, NumberPercentage],
        ),
        RelativeColorFunction::Lab => (
            CssRelativeColorFunction::Lab,
            CssRelativeColorEnvironment::Lab,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Lch => (
            CssRelativeColorFunction::Lch,
            CssRelativeColorEnvironment::Lch,
            [NumberPercentage, NumberPercentage, Hue],
        ),
        RelativeColorFunction::Oklab => (
            CssRelativeColorFunction::Oklab,
            CssRelativeColorEnvironment::Oklab,
            [NumberPercentage; 3],
        ),
        RelativeColorFunction::Oklch => (
            CssRelativeColorFunction::Oklch,
            CssRelativeColorEnvironment::Oklch,
            [NumberPercentage, NumberPercentage, Hue],
        ),
        RelativeColorFunction::Color => {
            let space = parse_relative_predefined_color_space(input)?;
            let environment = match space {
                CssPredefinedColorSpace::XyzD50 | CssPredefinedColorSpace::XyzD65 => {
                    CssRelativeColorEnvironment::Xyz(space)
                }
                _ => CssRelativeColorEnvironment::PredefinedRgb(space),
            };
            (
                CssRelativeColorFunction::Color(space),
                environment,
                [NumberPercentage; 3],
            )
        }
    };
    Ok(signature)
}

fn parse_typed_relative_color_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
    environment: CssRelativeColorEnvironment,
    result_domain: CssRelativeColorResultDomain,
) -> std::result::Result<CssTypedRelativeColorExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let value = match token {
        Token::Ident(ident) if ident.eq_ignore_ascii_case("none") => {
            CssRelativeColorExpressionValue::None
        }
        Token::Ident(ident) => {
            let Some(channel) = relative_color_channel(environment, &ident) else {
                return Err(with_color_context(
                    location.new_unexpected_token_error::<Error>(Token::Ident(ident)),
                    Some("relative channel"),
                ));
            };
            CssRelativeColorExpressionValue::Channel(channel)
        }
        Token::Number { value, .. } => CssFiniteNumber::try_new(value)
            .map(CssRelativeColorExpressionValue::Number)
            .ok_or_else(|| invalid_color(location, Some("relative channel")))?,
        Token::Percentage { unit_value, .. } => CssFiniteNumber::try_new(unit_value * 100.0)
            .map(CssRelativeColorExpressionValue::Percentage)
            .ok_or_else(|| invalid_color(location, Some("relative channel")))?,
        Token::Dimension { value, unit, .. }
            if matches!(result_domain, CssRelativeColorResultDomain::Hue) =>
        {
            parse_relative_angle(value, &unit)
                .map(CssRelativeColorExpressionValue::Angle)
                .ok_or_else(|| invalid_color(location, Some("relative hue")))?
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let mut references = Vec::new();
            let info = input.parse_nested_block(|input| {
                let info = parse_relative_calculation_sum(input, environment, 0, &mut references)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(info)
            })?;
            if !relative_result_domain_accepts(result_domain, info.result_type) {
                return Err(invalid_color(location, Some("relative channel")));
            }
            let authored = CssAuthoredDeclarationValue::new(input.slice_from(start).trim_end());
            CssRelativeColorExpressionValue::Calculation(CssRelativeColorCalculation::new(
                authored,
                info.result_type,
                references,
            ))
        }
        token => {
            return Err(with_color_context(
                location.new_unexpected_token_error::<Error>(token),
                Some("relative channel"),
            ));
        }
    };
    if !relative_direct_value_is_valid(result_domain, &value) {
        return Err(invalid_color(location, Some("relative channel")));
    }
    Ok(CssTypedRelativeColorExpression::new(
        environment,
        result_domain,
        value,
    ))
}

fn relative_direct_value_is_valid(
    domain: CssRelativeColorResultDomain,
    value: &CssRelativeColorExpressionValue,
) -> bool {
    match value {
        CssRelativeColorExpressionValue::None
        | CssRelativeColorExpressionValue::Channel(_)
        | CssRelativeColorExpressionValue::Calculation(_) => true,
        CssRelativeColorExpressionValue::Number(_) => true,
        CssRelativeColorExpressionValue::Percentage(_) => {
            !matches!(domain, CssRelativeColorResultDomain::Hue)
        }
        CssRelativeColorExpressionValue::Angle(_) => {
            matches!(domain, CssRelativeColorResultDomain::Hue)
        }
    }
}

fn relative_result_domain_accepts(
    domain: CssRelativeColorResultDomain,
    result_type: CssCalculationType,
) -> bool {
    match domain {
        CssRelativeColorResultDomain::NumberPercentage | CssRelativeColorResultDomain::Alpha => {
            matches!(
                result_type,
                CssCalculationType::Integer
                    | CssCalculationType::Number
                    | CssCalculationType::Percentage
            )
        }
        CssRelativeColorResultDomain::Hue => matches!(
            result_type,
            CssCalculationType::Integer | CssCalculationType::Number | CssCalculationType::Angle
        ),
    }
}

fn relative_color_channel(
    environment: CssRelativeColorEnvironment,
    ident: &str,
) -> Option<CssRelativeColorChannel> {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    let channel = match environment {
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
    environment: CssRelativeColorEnvironment,
    channel: CssRelativeColorChannel,
) -> CssCalculationType {
    use CssRelativeColorChannel::{A, Alpha, B, C, G, H, L, R, S, W, X, Y, Z};
    match (environment, channel) {
        (CssRelativeColorEnvironment::Hsl, H)
        | (CssRelativeColorEnvironment::Hwb, H)
        | (CssRelativeColorEnvironment::Lch, H)
        | (CssRelativeColorEnvironment::Oklch, H) => CssCalculationType::Angle,
        (CssRelativeColorEnvironment::Hsl, S | L)
        | (CssRelativeColorEnvironment::Hwb, W | B)
        | (CssRelativeColorEnvironment::Lab | CssRelativeColorEnvironment::Oklab, L)
        | (CssRelativeColorEnvironment::Lch | CssRelativeColorEnvironment::Oklch, L) => {
            CssCalculationType::Percentage
        }
        (_, R | G | B | A | C | X | Y | Z | Alpha | H | S | L | W) => CssCalculationType::Number,
    }
}

fn parse_relative_angle(value: f32, unit: &str) -> Option<CssAngleLiteral> {
    let unit = match unit.to_ascii_lowercase().as_str() {
        "deg" => CssAngleUnit::Degrees,
        "grad" => CssAngleUnit::Gradians,
        "rad" => CssAngleUnit::Radians,
        "turn" => CssAngleUnit::Turns,
        _ => return None,
    };
    CssAngleLiteral::try_new(value, unit)
}

#[derive(Clone, Copy)]
struct RelativeCalculationInfo {
    result_type: CssCalculationType,
    numeric_value: Option<f32>,
}

fn parse_relative_calculation_sum<'i, 't>(
    input: &mut Parser<'i, 't>,
    environment: CssRelativeColorEnvironment,
    depth: u16,
    references: &mut Vec<CssRelativeColorChannel>,
) -> std::result::Result<RelativeCalculationInfo, ParseError<'i, Error>> {
    let mut result = parse_relative_calculation_product(input, environment, depth, references)?;
    loop {
        let state = input.state();
        let location = input.current_source_location();
        let operator = match input.next() {
            Ok(Token::Delim('+')) => Some(1.0),
            Ok(Token::Delim('-')) => Some(-1.0),
            Ok(_) | Err(_) => None,
        };
        let Some(operator) = operator else {
            input.reset(&state);
            break;
        };
        let right = parse_relative_calculation_product(input, environment, depth, references)?;
        if relative_sum_type(result.result_type, right.result_type).is_none() {
            return Err(invalid_color(location, Some("relative calculation")));
        }
        result.numeric_value = match (result.numeric_value, right.numeric_value) {
            (Some(left), Some(right)) => {
                let value = left + operator * right;
                if !value.is_finite() {
                    return Err(invalid_color(location, Some("relative calculation")));
                }
                Some(value)
            }
            _ => None,
        };
    }
    Ok(result)
}

fn parse_relative_calculation_product<'i, 't>(
    input: &mut Parser<'i, 't>,
    environment: CssRelativeColorEnvironment,
    depth: u16,
    references: &mut Vec<CssRelativeColorChannel>,
) -> std::result::Result<RelativeCalculationInfo, ParseError<'i, Error>> {
    let mut result = parse_relative_calculation_unary(input, environment, depth, references)?;
    loop {
        let state = input.state();
        let location = input.current_source_location();
        let operator = match input.next() {
            Ok(Token::Delim('*')) => Some(true),
            Ok(Token::Delim('/')) => Some(false),
            Ok(_) | Err(_) => None,
        };
        let Some(is_multiply) = operator else {
            input.reset(&state);
            break;
        };
        let right = parse_relative_calculation_unary(input, environment, depth, references)?;
        let Some(result_type) =
            relative_product_type(result.result_type, right.result_type, is_multiply)
        else {
            return Err(invalid_color(location, Some("relative calculation")));
        };
        if !is_multiply && matches!(right.numeric_value, Some(value) if value == 0.0) {
            return Err(invalid_color(location, Some("relative calculation")));
        }
        result.numeric_value = match (result.numeric_value, right.numeric_value) {
            (Some(left), Some(right)) => {
                let value = if is_multiply {
                    left * right
                } else {
                    left / right
                };
                if !value.is_finite() {
                    return Err(invalid_color(location, Some("relative calculation")));
                }
                Some(value)
            }
            _ => None,
        };
        result.result_type = result_type;
    }
    Ok(result)
}

fn parse_relative_calculation_unary<'i, 't>(
    input: &mut Parser<'i, 't>,
    environment: CssRelativeColorEnvironment,
    depth: u16,
    references: &mut Vec<CssRelativeColorChannel>,
) -> std::result::Result<RelativeCalculationInfo, ParseError<'i, Error>> {
    let state = input.state();
    if matches!(input.next(), Ok(Token::Delim('-'))) {
        let mut value = parse_relative_calculation_unary(input, environment, depth, references)?;
        value.numeric_value = value.numeric_value.map(|value| -value);
        return Ok(value);
    }
    input.reset(&state);
    parse_relative_calculation_value(input, environment, depth, references)
}

fn parse_relative_calculation_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    environment: CssRelativeColorEnvironment,
    depth: u16,
    references: &mut Vec<CssRelativeColorChannel>,
) -> std::result::Result<RelativeCalculationInfo, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    let result = match token {
        Token::Number { value, .. } if value.is_finite() => RelativeCalculationInfo {
            result_type: CssCalculationType::Number,
            numeric_value: Some(value),
        },
        Token::Percentage { unit_value, .. } if unit_value.is_finite() => RelativeCalculationInfo {
            result_type: CssCalculationType::Percentage,
            numeric_value: Some(unit_value * 100.0),
        },
        Token::Dimension { value, unit, .. } => {
            let angle = parse_relative_angle(value, &unit)
                .ok_or_else(|| invalid_color(location, Some("relative calculation")))?;
            RelativeCalculationInfo {
                result_type: CssCalculationType::Angle,
                numeric_value: Some(angle.value()),
            }
        }
        Token::Ident(ident) => {
            let channel = relative_color_channel(environment, &ident)
                .ok_or_else(|| invalid_color(location, Some("relative calculation")))?;
            references.push(channel);
            RelativeCalculationInfo {
                result_type: relative_channel_type(environment, channel),
                numeric_value: None,
            }
        }
        Token::ParenthesisBlock => {
            let nested_depth = checked_calculation_depth(depth, location)?;
            input.parse_nested_block(|input| {
                let value =
                    parse_relative_calculation_sum(input, environment, nested_depth, references)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(value)
            })?
        }
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            let nested_depth = checked_calculation_depth(depth, location)?;
            input.parse_nested_block(|input| {
                let value =
                    parse_relative_calculation_sum(input, environment, nested_depth, references)?;
                input.expect_exhausted().map_err(basic)?;
                Ok(value)
            })?
        }
        _ => return Err(invalid_color(location, Some("relative calculation"))),
    };
    Ok(result)
}

fn relative_sum_type(
    left: CssCalculationType,
    right: CssCalculationType,
) -> Option<CssCalculationType> {
    (left == right).then_some(left)
}

fn relative_product_type(
    left: CssCalculationType,
    right: CssCalculationType,
    is_multiply: bool,
) -> Option<CssCalculationType> {
    let left_is_number = matches!(
        left,
        CssCalculationType::Integer | CssCalculationType::Number
    );
    let right_is_number = matches!(
        right,
        CssCalculationType::Integer | CssCalculationType::Number
    );
    if is_multiply {
        match (left_is_number, right_is_number) {
            (true, true) => Some(CssCalculationType::Number),
            (true, false) => Some(right),
            (false, true) => Some(left),
            (false, false) => None,
        }
    } else if right_is_number {
        Some(left)
    } else {
        None
    }
}

fn parse_relative_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) => {
            let Some(function) = relative_color_function_from_name(&name) else {
                input.reset(&state);
                return Err(location.new_unexpected_token_error::<Error>(Token::Function(name)));
            };
            input
                .parse_nested_block(|input| parse_relative_color_arguments(input, function))
                .map(CssColor::Relative)
        }
        token => {
            input.reset(&state);
            Err(location.new_unexpected_token_error::<Error>(token))
        }
    }
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

fn parse_relative_color_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
    function: RelativeColorFunction,
) -> std::result::Result<CssRelativeColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    input.expect_ident_matching("from").map_err(basic)?;
    let source = parse_color_inner(input)?;
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

    let mut components = Vec::with_capacity(function.component_count());
    for _ in 0..function.component_count() {
        components.push(parse_color_component_expression(input)?);
    }

    let alpha = if input.try_parse(|input| input.expect_delim('/')).is_ok() {
        Some(parse_color_component_expression(input)?)
    } else {
        None
    };

    input.expect_exhausted().map_err(basic)?;
    CssRelativeColor::try_new(function, source, components, alpha).ok_or_else(|| {
        unsupported_value_at(location, None, "unsupported relative color component count")
    })
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

fn parse_color_component_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorComponentExpression, ParseError<'i, Error>> {
    input.skip_whitespace();
    let start = input.position();
    consume_color_component_expression(input)?;
    let authored = CssAuthoredDeclarationValue::new(input.slice_from(start).trim_end());
    Ok(CssColorComponentExpression::new(authored, Vec::new()))
}

fn consume_color_component_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    if token.is_parse_error() {
        return Err(input.new_unexpected_token_error(token));
    }
    match token {
        Token::Ident(_)
        | Token::Number { .. }
        | Token::Percentage { .. }
        | Token::Dimension { .. } => Ok(()),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(parse_color_component_calc_expression)
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color component function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_color_component_calc_expression<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    parse_color_component_calc_operand(input)?;
    while !input.is_exhausted() {
        parse_color_component_calc_operator(input)?;
        parse_color_component_calc_operand(input)?;
    }
    Ok(())
}

fn parse_color_component_calc_operator<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Delim('+') | Token::Delim('-') | Token::Delim('*') | Token::Delim('/') => Ok(()),
        token => Err(unsupported_value_at(
            location,
            None,
            format!(
                "expected relative color calc operator, got `{}`",
                token.to_css_string()
            ),
        )),
    }
}

fn parse_color_component_calc_operand<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<(), ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    if token.is_parse_error() {
        return Err(input.new_unexpected_token_error(token));
    }
    match token {
        Token::Ident(_)
        | Token::Number { .. }
        | Token::Percentage { .. }
        | Token::Dimension { .. } => Ok(()),
        Token::Function(name) if name.eq_ignore_ascii_case("calc") => {
            input.parse_nested_block(parse_color_component_calc_expression)
        }
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported relative color calc function `{name}`"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_color_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let state = input.state();
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("color-mix") => input
            .parse_nested_block(parse_color_mix_arguments)
            .map(CssColor::ColorMix),
        token => {
            input.reset(&state);
            Err(location.new_unexpected_token_error::<Error>(token))
        }
    }
}

fn parse_authored_color_mix<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let token = input.next().map_err(basic)?.clone();
    match token {
        Token::Function(name) if name.eq_ignore_ascii_case("color-mix") => input
            .parse_nested_block(parse_authored_color_mix_arguments)
            .map(CssAuthoredColor::color_mix),
        token => Err(location.new_unexpected_token_error::<Error>(token)),
    }
}

fn parse_authored_color_mix_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColorMix, ParseError<'i, Error>> {
    input.expect_ident_matching("in").map_err(basic)?;
    let interpolation = parse_authored_color_mix_interpolation_method(input)?;
    input.expect_comma().map_err(basic)?;
    let left = parse_authored_color_mix_component(input)?;
    input.expect_comma().map_err(basic)?;
    let right = parse_authored_color_mix_component(input)?;
    input.expect_exhausted().map_err(basic)?;

    CssAuthoredColorMix::try_new(interpolation, left, right).ok_or_else(|| {
        invalid_color(
            input.current_source_location(),
            Some("color-mix interpolation"),
        )
    })
}

fn parse_authored_color_mix_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationMethod, ParseError<'i, Error>> {
    let space = parse_color_mix_interpolation_space(input)?;
    let hue_location = input.current_source_location();
    let hue = input.try_parse(parse_color_mix_hue_interpolation).ok();
    let interpolation = CssColorInterpolationMethod::new(space, hue);
    if hue.is_some() && !space.is_polar() {
        Err(invalid_color(hue_location, Some("hue interpolation")))
    } else {
        Ok(interpolation)
    }
}

fn parse_authored_color_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColorMixComponent, ParseError<'i, Error>> {
    let (color, _) = parse_color(input)?.into_parts();
    let percentage = if next_is_percentage(input) {
        Some(parse_authored_color_mix_percentage(input)?)
    } else {
        None
    };
    Ok(CssAuthoredColorMixComponent::new(color, percentage))
}

fn next_is_percentage<'i, 't>(input: &mut Parser<'i, 't>) -> bool {
    let state = input.state();
    let is_percentage = matches!(input.next(), Ok(Token::Percentage { .. }));
    input.reset(&state);
    is_percentage
}

fn parse_authored_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssAuthoredColorMixPercentage, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let percentage = input.expect_percentage().map_err(basic)? * 100.0;
    CssAuthoredColorMixPercentage::try_new(percentage)
        .ok_or_else(|| invalid_color(location, Some("color-mix component percentage")))
}

fn parse_color_mix_arguments<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorMix, ParseError<'i, Error>> {
    input.expect_ident_matching("in").map_err(basic)?;
    let interpolation = parse_color_mix_interpolation_method(input)?;
    input.expect_comma().map_err(basic)?;
    let left = parse_color_mix_component(input)?;
    input.expect_comma().map_err(basic)?;
    let right = parse_color_mix_component(input)?;

    Ok(CssColorMix::new(interpolation, left, right))
}

fn parse_color_mix_interpolation_method<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorInterpolationMethod, ParseError<'i, Error>> {
    let space = parse_color_mix_interpolation_space(input)?;
    let hue = input.try_parse(parse_color_mix_hue_interpolation).ok();
    Ok(CssColorInterpolationMethod::new(space, hue))
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

fn parse_color_mix_component<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColorMixComponent, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let color = parse_color_inner(input)?;
    let percentage = input.try_parse(parse_color_mix_percentage).ok();
    CssColorMixComponent::try_new(color, percentage).ok_or_else(|| {
        unsupported_value_at(location, None, "unsupported color-mix component percentage")
    })
}

fn parse_color_mix_percentage<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    input
        .expect_percentage()
        .map(|percentage| percentage * 100.0)
        .map_err(basic)
}

fn parse_absolute_color_with_cssparser_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    match parse_color_with(&DefaultColorParser, input) {
        Ok(parsed) => map_parsed_color(parsed, location),
        Err(_) => Err(invalid_color(location, None)),
    }
}

fn parse_system_color<'i, 't>(
    input: &mut Parser<'i, 't>,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let location = input.current_source_location();
    let ident = input.expect_ident_cloned().map_err(basic)?;
    let color = match_ignore_ascii_case! { &ident,
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
        _ => return Err(unsupported_value_at(
            location,
            None,
            format!("unsupported system color `{ident}`"),
        )),
    };
    Ok(CssColor::System(color))
}

fn map_parsed_color<'i>(
    parsed: ParsedColor,
    location: cssparser::SourceLocation,
) -> std::result::Result<CssColor, ParseError<'i, Error>> {
    let color = match parsed {
        ParsedColor::CurrentColor => CssColor::CurrentColor,
        ParsedColor::Rgba(color) => CssColor::Rgba(
            CssRgbaColor::try_new(color.red, color.green, color.blue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Hsl(color) => CssColor::Hsl(
            CssHslColor::try_new(color.hue, color.saturation, color.lightness, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Hwb(color) => CssColor::Hwb(
            CssHwbColor::try_new(color.hue, color.whiteness, color.blackness, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Lab(color) => CssColor::Lab(
            CssLabColor::try_new(color.lightness, color.a, color.b, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Lch(color) => CssColor::Lch(
            CssLchColor::try_new(color.lightness, color.chroma, color.hue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Oklab(color) => CssColor::Oklab(
            CssLabColor::try_new(color.lightness, color.a, color.b, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::Oklch(color) => CssColor::Oklch(
            CssLchColor::try_new(color.lightness, color.chroma, color.hue, color.alpha)
                .ok_or_else(|| invalid_color_component(location))?,
        ),
        ParsedColor::ColorFunction(color) => CssColor::ColorFunction(
            CssColorFunction::try_new(
                map_predefined_color_space(color.color_space),
                [color.c1, color.c2, color.c3],
                color.alpha,
            )
            .ok_or_else(|| invalid_color_component(location))?,
        ),
    };
    Ok(color)
}

fn invalid_color_component<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    invalid_color(location, Some("component"))
}

fn map_predefined_color_space(color_space: ParsedPredefinedColorSpace) -> CssPredefinedColorSpace {
    match color_space {
        ParsedPredefinedColorSpace::Srgb => CssPredefinedColorSpace::Srgb,
        ParsedPredefinedColorSpace::SrgbLinear => CssPredefinedColorSpace::SrgbLinear,
        ParsedPredefinedColorSpace::DisplayP3 => CssPredefinedColorSpace::DisplayP3,
        ParsedPredefinedColorSpace::DisplayP3Linear => CssPredefinedColorSpace::DisplayP3Linear,
        ParsedPredefinedColorSpace::A98Rgb => CssPredefinedColorSpace::A98Rgb,
        ParsedPredefinedColorSpace::ProphotoRgb => CssPredefinedColorSpace::ProphotoRgb,
        ParsedPredefinedColorSpace::Rec2020 => CssPredefinedColorSpace::Rec2020,
        ParsedPredefinedColorSpace::XyzD50 => CssPredefinedColorSpace::XyzD50,
        ParsedPredefinedColorSpace::XyzD65 => CssPredefinedColorSpace::XyzD65,
    }
}

#[cfg(test)]
mod typed_calculation_tests {
    use cssparser::{Parser, ParserInput};

    use super::*;
    use crate::error::{CssErrorCode, from_parse_error};

    fn parse(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::Error> {
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        parser
            .parse_entirely(|input| parse_typed_calculation(input, root))
            .map_err(|error| from_parse_error(source, error))
    }

    #[test]
    fn typed_root_parser_preserves_all_compound_node_kinds_and_precedence() {
        let number = parse("1 + 6 / 2", CalculationRoot::Number).unwrap();
        assert_eq!(number.result_type(), CssCalculationType::Number);
        let CssCalculationExpressionRef::Sum(sum) = number.as_ref() else {
            panic!("expected number sum");
        };
        assert_eq!(sum.len(), 2);
        assert_eq!(sum.term(0).unwrap().operator(), None);
        assert_eq!(
            sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Add)
        );
        let CssCalculationExpressionRef::Product(quotient) = sum.term(1).unwrap().expression()
        else {
            panic!("division must bind inside the sum");
        };
        assert_eq!(quotient.len(), 2);
        assert_eq!(quotient.factor(0).unwrap().operator(), None);
        assert_eq!(
            quotient.factor(1).unwrap().operator(),
            Some(CssCalculationProductOperator::Divide)
        );

        let integer = parse("1 + 2 * 3", CalculationRoot::Integer).unwrap();
        assert_eq!(integer.result_type(), CssCalculationType::Integer);
        let CssCalculationExpressionRef::Sum(integer_sum) = integer.as_ref() else {
            panic!("expected integer sum");
        };
        let CssCalculationExpressionRef::Product(integer_product) =
            integer_sum.term(1).unwrap().expression()
        else {
            panic!("expected integer product");
        };
        assert_eq!(
            integer_product.factor(1).unwrap().operator(),
            Some(CssCalculationProductOperator::Multiply)
        );
        assert!(integer_product.factor(integer_product.len()).is_none());

        let percentage = parse("10% - 20%", CalculationRoot::Percentage).unwrap();
        assert_eq!(percentage.result_type(), CssCalculationType::Percentage);
        let CssCalculationExpressionRef::Sum(percentage_sum) = percentage.as_ref() else {
            panic!("expected percentage sum");
        };
        assert_eq!(
            percentage_sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Subtract)
        );
        assert!(percentage_sum.term(percentage_sum.len()).is_none());

        let length = parse("1px + (2em * 3)", CalculationRoot::Length).unwrap();
        let CssCalculationExpressionRef::Sum(sum) = length.as_ref() else {
            panic!("expected length sum");
        };
        let CssCalculationExpressionRef::Group(group) = sum.term(1).unwrap().expression() else {
            panic!("expected retained authored group");
        };
        assert!(matches!(
            group.operand(),
            CssCalculationExpressionRef::Product(_)
        ));

        let angle = parse("1deg + calc(2turn)", CalculationRoot::Angle).unwrap();
        assert_eq!(angle.result_type(), CssCalculationType::Angle);
        let CssCalculationExpressionRef::Sum(sum) = angle.as_ref() else {
            panic!("expected angle sum");
        };
        let CssCalculationExpressionRef::NestedCalc(nested) = sum.term(1).unwrap().expression()
        else {
            panic!("expected retained nested calc");
        };
        assert!(matches!(
            nested.operand(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
                if value.value() == 2.0 && value.unit() == CssAngleUnit::Turns
        ));

        let time = parse("1s + -(2ms)", CalculationRoot::Time).unwrap();
        assert_eq!(time.result_type(), CssCalculationType::Time);
        let CssCalculationExpressionRef::Sum(sum) = time.as_ref() else {
            panic!("expected time sum");
        };
        let CssCalculationExpressionRef::Negate(negate) = sum.term(1).unwrap().expression() else {
            panic!("expected retained authored negation");
        };
        assert!(matches!(
            negate.operand(),
            CssCalculationExpressionRef::Group(_)
        ));

        let frequency = parse("1khz / 2", CalculationRoot::Frequency).unwrap();
        assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
        assert!(matches!(
            frequency.as_ref(),
            CssCalculationExpressionRef::Product(_)
        ));
    }

    #[test]
    fn typed_root_parser_promotes_only_compatible_percentage_dimensions() {
        for (root, expected) in [
            (
                CalculationRoot::Length,
                CssCalculationType::LengthPercentage,
            ),
            (CalculationRoot::Angle, CssCalculationType::AnglePercentage),
            (CalculationRoot::Time, CssCalculationType::TimePercentage),
            (
                CalculationRoot::Frequency,
                CssCalculationType::FrequencyPercentage,
            ),
        ] {
            let unit = match root {
                CalculationRoot::Length => "px",
                CalculationRoot::Angle => "deg",
                CalculationRoot::Time => "s",
                CalculationRoot::Frequency => "hz",
                CalculationRoot::Number
                | CalculationRoot::Integer
                | CalculationRoot::Percentage => unreachable!("test-owned root table"),
            };
            let expression = parse(&format!("1{unit} + 2%"), root).unwrap();
            assert_eq!(expression.result_type(), expected);
        }
    }

    #[test]
    fn typed_root_parser_rejects_invalid_dimensions_divisors_arithmetic_and_consumption() {
        let cases = [
            (
                "1px + 2deg",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px * 2em",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px / 2em",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px / 0",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px / -0",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px / calc(1 - 1)",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "3.4e38 * 2",
                CalculationRoot::Number,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1e999",
                CalculationRoot::Number,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "2147483648",
                CalculationRoot::Integer,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1e999%",
                CalculationRoot::Percentage,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1e999px",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1e999deg",
                CalculationRoot::Angle,
                CssErrorCode::UnexpectedEnd,
            ),
            ("1e999s", CalculationRoot::Time, CssErrorCode::UnexpectedEnd),
            (
                "1e999hz",
                CalculationRoot::Frequency,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px +",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedEnd,
            ),
            (
                "1px red",
                CalculationRoot::Length,
                CssErrorCode::UnexpectedToken,
            ),
            ("1px", CalculationRoot::Number, CssErrorCode::UnexpectedEnd),
        ];

        for (source, root, code) in cases {
            let error = parse(source, root).expect_err(source);
            assert_eq!(error.code(), code, "{source}: {error:?}");
        }
    }

    #[test]
    fn typed_root_parser_accepts_256_nested_calculations_and_rejects_257() {
        for depth in [255_usize, 256] {
            let source = format!("{}1px{}", "calc(".repeat(depth), ")".repeat(depth));
            let result_type = std::thread::scope(|scope| {
                std::thread::Builder::new()
                    .stack_size(16 * 1024 * 1024)
                    .spawn_scoped(scope, || {
                        parse(&source, CalculationRoot::Length)
                            .map(|expression| expression.result_type())
                    })
                    .unwrap()
                    .join()
                    .unwrap()
            })
            .unwrap();
            assert_eq!(result_type, CssCalculationType::Length);
        }

        let depth = 257_usize;
        let source = format!("{}1px{}", "calc(".repeat(depth), ")".repeat(depth));
        let error = std::thread::scope(|scope| {
            std::thread::Builder::new()
                .stack_size(16 * 1024 * 1024)
                .spawn_scoped(scope, || parse(&source, CalculationRoot::Length))
                .unwrap()
                .join()
                .unwrap()
        })
        .expect_err("depth 257 must fail");
        assert_eq!(error.code(), CssErrorCode::UnexpectedEnd);
    }
}
