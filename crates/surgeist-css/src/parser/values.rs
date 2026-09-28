use cssparser::{ParseError, Parser, Token, match_ignore_ascii_case};

use crate::error::{CssFeatureId, Error, basic, unsupported_value_at};
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

pub(super) fn parse_shadow_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::ShadowOffset)
}

pub(super) fn parse_shadow_blur_length<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with(input, numeric, LengthGrammar::ShadowBlur)
}

#[derive(Clone, Copy)]
pub(super) enum LengthGrammar {
    FlowTolerance,
    BorderWidth,
    ShadowOffset,
    ShadowBlur,
    BorderSpacing,
    Clip,
    OutlineOffset,
    TextIndent,
    VerticalAlign,
    TextDecorationThickness,
    BackgroundSize,
    Position,
}

impl LengthGrammar {
    const fn allows_percent(self) -> bool {
        matches!(
            self,
            Self::FlowTolerance
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn allows_line_width_keyword(self) -> bool {
        matches!(self, Self::BorderWidth)
    }

    const fn allows_calc_percent(self) -> bool {
        matches!(
            self,
            Self::FlowTolerance
                | Self::TextIndent
                | Self::VerticalAlign
                | Self::TextDecorationThickness
                | Self::BackgroundSize
                | Self::Position
        )
    }

    const fn requires_non_negative(self) -> bool {
        matches!(
            self,
            Self::BorderWidth
                | Self::ShadowBlur
                | Self::BorderSpacing
                | Self::TextDecorationThickness
                | Self::BackgroundSize
        )
    }

    const fn context(self) -> &'static str {
        match self {
            Self::FlowTolerance => "flow-tolerance",
            Self::BorderWidth => "border-width",
            Self::ShadowOffset => "box-shadow",
            Self::ShadowBlur => "box-shadow blur",
            Self::BorderSpacing => "border-spacing",
            Self::Clip => "clip",
            Self::OutlineOffset => "outline-offset",
            Self::TextIndent => "text-indent",
            Self::VerticalAlign => "vertical-align",
            Self::TextDecorationThickness => "text-decoration-thickness",
            Self::BackgroundSize => "background-size",
            Self::Position => "position",
        }
    }
}

pub(super) fn checked_percentage_value<'i>(
    location: cssparser::SourceLocation,
    token_css: &str,
    non_finite_reason: impl Into<String>,
) -> std::result::Result<f32, ParseError<'i, Error>> {
    // cssparser exposes percentages after dividing by 100 and rounding to f32.
    // Re-multiplication can change the authored magnitude (30% -> 30.000002).
    // The caller passes exactly the consumed percentage token, without trivia.
    let value = token_css
        .strip_suffix('%')
        .and_then(|numeric| numeric.parse::<f32>().ok())
        .filter(|value| value.is_finite());
    value.ok_or_else(|| unsupported_value_at(location, None, non_finite_reason))
}

pub(super) fn parse_length_with<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    grammar: LengthGrammar,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    parse_length_with_context(input, numeric, grammar, grammar.context())
}

pub(super) fn parse_length_with_context<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    let before_opener = input.state();
    if matches!(input.next().map_err(basic)?, Token::Function(name) if is_math_function(name)) {
        let root = if grammar.allows_calc_percent() {
            CalculationRoot::LengthPercentage
        } else {
            CalculationRoot::Length
        };
        let expression = parse_numeric_function(input, &before_opener, numeric, root)?;
        return Ok(CssLength::Calc(CssCalcLength::Typed(
            CssLengthPercentageCalculation::from_expression(expression),
        )));
    }
    input.reset(&before_opener);
    parse_literal_length_with_context(input, grammar, context)
}

/// Literal-only compatibility grammar; current calculations use the numeric owner.
pub(super) fn parse_literal_length_with_context<'i, 't>(
    input: &mut Parser<'i, 't>,
    grammar: LengthGrammar,
    context: &str,
) -> std::result::Result<CssLength, ParseError<'i, Error>> {
    let location = input.current_source_location();
    input.skip_whitespace();
    let token_start = input.position();
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
        Token::Percentage { .. } => {
            let value = checked_percentage_value(
                location,
                input.slice_from(token_start),
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
            "thin" if grammar.allows_line_width_keyword() => Ok(CssLength::Thin),
            "medium" if grammar.allows_line_width_keyword() => Ok(CssLength::Medium),
            "thick" if grammar.allows_line_width_keyword() => Ok(CssLength::Thick),
            _ => Err(unsupported_value_at(
                location,
                None,
                format!("unsupported {context} `{ident}`"),
            )),
        },
        Token::Function(name) => Err(unsupported_value_at(
            location,
            None,
            format!("unsupported length function `{name}` for {context}"),
        )),
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

use crate::numeric::NumericInputContext;
pub(super) use crate::numeric::{CalculationRoot, is_math_function};

pub(super) fn parse_numeric_function<'i, 't>(
    input: &mut Parser<'i, 't>,
    before_opener: &cssparser::ParserState,
    numeric: &NumericInputContext<'_>,
    root: CalculationRoot,
) -> Result<CssCalculationExpression, ParseError<'i, Error>> {
    input.reset(before_opener);
    input.skip_whitespace();
    let root_offset = input.position().byte_index();
    let location = input.current_source_location();
    let component = numeric.collect(input).map_err(|error| {
        calculation_error(numeric.error_location(&error, location, root_offset))
    })?;
    let values = crate::CssComponentValues::try_new(vec![component])
        .map_err(|_| calculation_error(location))?;
    numeric
        .admit(values, root)
        .map_err(|error| calculation_error(numeric.error_location(&error, location, root_offset)))
}

pub(super) fn calculation_error<'i>(location: cssparser::SourceLocation) -> ParseError<'i, Error> {
    unsupported_value_at(location, None, "invalid typed calculation")
}

/// Shared exact integer admission for authored number tokens and integer-root math.
pub(super) fn parse_integer_value<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssIntegerValue, ParseError<'i, Error>> {
    let numeric_start = input.state();
    let location = input.current_source_location();
    match input.next().map_err(basic)? {
        Token::Number { .. } => {
            input.reset(&numeric_start);
            parse_current_integer_literal(input, numeric)
        }
        Token::Function(name) if crate::numeric::is_math_function(name) => {
            parse_numeric_function(input, &numeric_start, numeric, CalculationRoot::Integer)
                .map(CssIntegerCalculation::from_expression)
                .map(CssIntegerValue::Calculation)
        }
        token => Err(location.new_unexpected_token_error::<Error>(token.clone())),
    }
}

pub(super) fn parse_current_integer_literal<'i, 't>(
    input: &mut Parser<'i, 't>,
    numeric: &NumericInputContext<'_>,
) -> Result<CssIntegerValue, ParseError<'i, Error>> {
    input.skip_whitespace();
    let location = input.current_source_location();
    let offset = input.position().byte_index();
    let component = numeric.collect(input).map_err(|error| {
        unsupported_value_at(
            numeric.error_location(&error, location, offset),
            None,
            "invalid integer component",
        )
    })?;
    crate::integer_value::admit_integer_literal(component)
        .map_err(|_| unsupported_value_at(location, None, "value must have integer token syntax"))
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

#[cfg(test)]
mod typed_calculation_tests {
    use cssparser::{Parser, ParserInput};

    use super::*;
    use crate::error::{CssErrorCode, from_parse_error};

    fn parse(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::Error> {
        parse_complete(&format!("calc({source})"), root)
    }

    fn parse_complete(
        source: &str,
        root: CalculationRoot,
    ) -> Result<CssCalculationExpression, crate::Error> {
        let snapshot = crate::CssSourceSnapshot::new(source);
        let numeric = NumericInputContext::parsed(&snapshot);
        let mut input = ParserInput::new(source);
        let mut parser = Parser::new(&mut input);
        let before = parser.state();
        parser
            .parse_entirely(|input| parse_numeric_function(input, &before, &numeric, root))
            .map_err(|error| from_parse_error(source, error))
    }

    fn body(expression: &CssCalculationExpression) -> CssCalculationExpressionRef<'_> {
        let CssCalculationExpressionRef::NestedCalc(root) = expression.as_ref() else {
            panic!("expected calc root")
        };
        root.operand()
    }

    #[test]
    fn typed_root_parser_preserves_all_compound_node_kinds_and_precedence() {
        let number = parse("1 + 6 / 2", CalculationRoot::Number).unwrap();
        assert_eq!(number.result_type(), CssCalculationType::Number);
        let CssCalculationExpressionRef::Sum(sum) = body(&number) else {
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
        assert_eq!(integer.result_type(), CssCalculationType::Number);
        let CssCalculationExpressionRef::Sum(integer_sum) = body(&integer) else {
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
        let CssCalculationExpressionRef::Sum(percentage_sum) = body(&percentage) else {
            panic!("expected percentage sum");
        };
        assert_eq!(
            percentage_sum.term(1).unwrap().operator(),
            Some(CssCalculationSumOperator::Subtract)
        );
        assert!(percentage_sum.term(percentage_sum.len()).is_none());

        let length = parse("1px + (2em * 3)", CalculationRoot::Length).unwrap();
        let CssCalculationExpressionRef::Sum(sum) = body(&length) else {
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
        let CssCalculationExpressionRef::Sum(sum) = body(&angle) else {
            panic!("expected angle sum");
        };
        let CssCalculationExpressionRef::NestedCalc(nested) = sum.term(1).unwrap().expression()
        else {
            panic!("expected retained nested calc");
        };
        assert!(matches!(
            nested.operand(),
            CssCalculationExpressionRef::Value(CssCalculationValueRef::Angle(value))
                if value.representation() == "2" && value.unit() == Some("turn")
        ));

        let time = parse("1s + (-2ms)", CalculationRoot::Time).unwrap();
        assert_eq!(time.result_type(), CssCalculationType::Time);
        let CssCalculationExpressionRef::Sum(sum) = body(&time) else {
            panic!("expected time sum")
        };
        let CssCalculationExpressionRef::Group(group) = sum.term(1).unwrap().expression() else {
            panic!("expected group")
        };
        assert!(
            matches!(group.operand(), CssCalculationExpressionRef::Value(CssCalculationValueRef::Time(v)) if v.representation() == "-2" && v.unit() == Some("ms"))
        );
        assert!(parse("1s + -(2ms)", CalculationRoot::Time).is_err());

        let frequency = parse("1khz / 2", CalculationRoot::Frequency).unwrap();
        assert_eq!(frequency.result_type(), CssCalculationType::Frequency);
        assert!(matches!(
            body(&frequency),
            CssCalculationExpressionRef::Product(_)
        ));
    }

    #[test]
    fn typed_root_parser_promotes_only_the_selected_percentage_context() {
        let mixed = parse("1px + 2%", CalculationRoot::LengthPercentage).unwrap();
        assert_eq!(mixed.result_type(), CssCalculationType::LengthPercentage);
        for (root, unit) in [
            (CalculationRoot::Length, "px"),
            (CalculationRoot::Angle, "deg"),
            (CalculationRoot::Time, "s"),
            (CalculationRoot::Frequency, "hz"),
        ] {
            assert!(parse(&format!("1{unit} + 2%"), root).is_err());
        }
    }

    #[test]
    fn typed_root_parser_rejects_invalid_types_and_grammar_but_defers_arithmetic() {
        for (source, root) in [
            ("1px + 2deg", CalculationRoot::Length),
            ("1px * 2em", CalculationRoot::Length),
            ("1px / 2em", CalculationRoot::Length),
            ("1px +", CalculationRoot::Length),
            ("1px red", CalculationRoot::Length),
            ("1px", CalculationRoot::Number),
        ] {
            assert!(parse(source, root).is_err(), "{source}");
        }
        for source in ["1px / 0", "1px / -0", "1px / calc(1 - 1)", "1e999px"] {
            assert_eq!(
                parse(source, CalculationRoot::Length)
                    .unwrap()
                    .result_type(),
                CssCalculationType::Length
            );
        }
        for source in ["3.4e38 * 2", "1e999", "2147483648"] {
            assert_eq!(
                parse(source, CalculationRoot::Integer)
                    .unwrap()
                    .result_type(),
                CssCalculationType::Number
            );
        }
        for (source, root) in [
            ("1e999%", CalculationRoot::Percentage),
            ("1e999deg", CalculationRoot::Angle),
            ("1e999s", CalculationRoot::Time),
            ("1e999hz", CalculationRoot::Frequency),
        ] {
            assert!(parse(source, root).is_ok(), "{source}");
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
                        parse_complete(&source, CalculationRoot::Length)
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
                .spawn_scoped(scope, || parse_complete(&source, CalculationRoot::Length))
                .unwrap()
                .join()
                .unwrap()
        })
        .expect_err("depth 257 must fail");
        assert_eq!(error.code(), CssErrorCode::UnexpectedEnd);
    }
}
